//! 沙盒策略与能力仲裁。
//!
//! 对标的是 Apple 在 Messages / Safari 上用的 containment 思路
//! （BlastDoor、Enhanced Security Extensions）：
//! 把不可信数据的解析放进几乎无权限的隔离单元里"引爆"，
//! 并且**不信任它返回的东西**——高权限侧必须重新校验简单类型。
//!
//! 参考：Apple WWDC "Protect your app with Enhanced Security" / "Fortify your app"，
//! 见 docs/research/apple-sandbox.md。

use crate::capability::{Capability, Grant};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IsolationLevel {
    None,
    Process,
    Site,
    BlastDoor,
}

impl IsolationLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            IsolationLevel::None => "none",
            IsolationLevel::Process => "process",
            IsolationLevel::Site => "site-isolation",
            IsolationLevel::BlastDoor => "blastdoor",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxPolicy {
    pub level: IsolationLevel,
    /// 锁定模式下关闭 JIT（Apple Lockdown Mode 对 WebKit 的做法）。
    pub deny_jit_in_lockdown: bool,
    /// 隔离单元返回的数据一律视为不可信，必须重新校验。
    pub revalidate_untrusted_output: bool,
    pub lockdown: bool,
    /// 默认拒绝：未明确授予的能力一律不通过。
    pub default_deny: bool,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self {
            level: IsolationLevel::BlastDoor,
            deny_jit_in_lockdown: true,
            revalidate_untrusted_output: true,
            lockdown: false,
            default_deny: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Allow,
    Deny,
}

impl Verdict {
    pub fn is_allowed(self) -> bool {
        self == Verdict::Allow
    }
}

impl SandboxPolicy {
    /// 进入锁定模式：抬高隔离级别，冻结所有可改动隔离边界的能力。
    pub fn harden(&mut self) {
        self.lockdown = true;
        self.level = IsolationLevel::BlastDoor;
        self.deny_jit_in_lockdown = true;
        self.revalidate_untrusted_output = true;
        self.default_deny = true;
    }

    /// 策略是否被削弱。派生版本若弱化这些开关，即违反许可证第 6 条。
    pub fn weakened(&self) -> bool {
        !self.default_deny
            || !self.revalidate_untrusted_output
            || self.level == IsolationLevel::None
    }

    /// 运行时仲裁：三重闸门。
    ///
    /// 1. 锁定模式 + 安全红线能力 → 拒绝（换引擎/换网络栈/改沙盒策略先退出锁定模式）
    /// 2. 隐私红线能力 + 未经用户显式确认 → 拒绝
    /// 3. 授权里没有该能力 → 拒绝（默认拒绝）
    pub fn decide(&self, grant: &Grant, cap: Capability, user_confirmed: bool) -> Verdict {
        if self.lockdown && cap.is_security_sensitive() {
            return Verdict::Deny;
        }
        if cap.is_privacy_sensitive() && !user_confirmed {
            return Verdict::Deny;
        }
        if !grant.allows(cap) {
            return Verdict::Deny;
        }
        Verdict::Allow
    }

    /// 在最低权限单元里"引爆"不可信输入。
    ///
    /// 返回的是 `Untrusted<T>` 而不是 `T`：这是刻意的类型级提醒，
    /// 调用方无法直接拿走结果，必须显式 `revalidate`。
    pub fn detonate<T>(
        &self,
        input: &[u8],
        parse: impl FnOnce(&[u8]) -> Option<T>,
    ) -> Option<Untrusted<T>> {
        if input.is_empty() {
            return None;
        }
        parse(input).map(Untrusted)
    }
}

/// 从隔离单元返回的数据。默认不可信。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Untrusted<T>(T);

impl<T> Untrusted<T> {
    /// 高权限侧用自己的、极小的校验器重新确认简单类型后才放行。
    pub fn revalidate(self, check: impl FnOnce(&T) -> bool) -> Option<T> {
        if check(&self.0) {
            Some(self.0)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_hardened_and_default_deny() {
        let p = SandboxPolicy::default();
        assert_eq!(p.level, IsolationLevel::BlastDoor);
        assert!(p.default_deny);
        assert!(p.revalidate_untrusted_output);
        assert!(!p.weakened());
    }

    #[test]
    fn ungranted_capability_is_denied() {
        let p = SandboxPolicy::default();
        let g = Grant::new().allow(Capability::NetConnect);
        assert_eq!(p.decide(&g, Capability::NetConnect, false), Verdict::Allow);
        assert_eq!(p.decide(&g, Capability::StorageWrite, false), Verdict::Deny);
    }

    #[test]
    fn privacy_cap_needs_explicit_confirmation_even_when_granted() {
        let p = SandboxPolicy::default();
        let g = Grant::new().allow(Capability::Telemetry);
        assert_eq!(p.decide(&g, Capability::Telemetry, false), Verdict::Deny);
        assert_eq!(p.decide(&g, Capability::Telemetry, true), Verdict::Allow);
    }

    #[test]
    fn lockdown_freezes_boundary_changing_capabilities() {
        let mut p = SandboxPolicy::default();
        let g = Grant::new()
            .allow(Capability::EngineSwap)
            .allow(Capability::NetConnect);
        assert_eq!(p.decide(&g, Capability::EngineSwap, false), Verdict::Allow);
        p.harden();
        assert_eq!(p.decide(&g, Capability::EngineSwap, false), Verdict::Deny);
        assert_eq!(p.decide(&g, Capability::NetConnect, false), Verdict::Allow);
    }

    #[test]
    fn detonation_output_must_be_revalidated() {
        let p = SandboxPolicy::default();
        // 模拟隔离单元把一段图片数据"转码"成宽度值。
        let parsed = p
            .detonate(&[0x89, 0x50, 0x4e, 0x47], |bytes| Some(bytes.len()))
            .expect("应解析出结果");
        assert_eq!(parsed.revalidate(|n| *n == 4), Some(4));
        assert_eq!(parsed.revalidate(|n| *n < 4), None);
    }

    #[test]
    fn weakened_policy_is_detected() {
        let p = SandboxPolicy {
            revalidate_untrusted_output: false,
            ..Default::default()
        };
        assert!(p.weakened());
    }
}
