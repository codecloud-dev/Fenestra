//! 能力（Capability）与授权（Grant）。
//!
//! Fenestra 的安全模型只有一句话：**默认拒绝**。
//! 插件在清单里声明想要什么，用户在授权里给什么，沙盒在运行时再仲裁一次。
//! 三方都不点头，能力就不存在。

use std::fmt;

/// 内核认识的全部能力。
///
/// 这份清单是刻意封闭的：新增能力需要修改内核并提升 FEP 版本，
/// 目的是让"权限膨胀"变成一件需要被审视的事，而不是某个插件自己说了算。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Capability {
    NetConnect,
    NetIntercept,
    StorageRead,
    StorageWrite,
    ClipboardRead,
    ClipboardWrite,
    UiChrome,
    UiContent,
    EngineSwap,
    NetworkStackSwap,
    SandboxPolicyRead,
    ProfileRead,
    DeviceCamera,
    DeviceLocation,
    Telemetry,
}

impl Capability {
    pub const ALL: [Capability; 15] = [
        Capability::NetConnect,
        Capability::NetIntercept,
        Capability::StorageRead,
        Capability::StorageWrite,
        Capability::ClipboardRead,
        Capability::ClipboardWrite,
        Capability::UiChrome,
        Capability::UiContent,
        Capability::EngineSwap,
        Capability::NetworkStackSwap,
        Capability::SandboxPolicyRead,
        Capability::ProfileRead,
        Capability::DeviceCamera,
        Capability::DeviceLocation,
        Capability::Telemetry,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Capability::NetConnect => "net.connect",
            Capability::NetIntercept => "net.intercept",
            Capability::StorageRead => "storage.read",
            Capability::StorageWrite => "storage.write",
            Capability::ClipboardRead => "clipboard.read",
            Capability::ClipboardWrite => "clipboard.write",
            Capability::UiChrome => "ui.chrome",
            Capability::UiContent => "ui.content",
            Capability::EngineSwap => "engine.swap",
            Capability::NetworkStackSwap => "network.swap",
            Capability::SandboxPolicyRead => "sandbox.read",
            Capability::ProfileRead => "profile.read",
            Capability::DeviceCamera => "device.camera",
            Capability::DeviceLocation => "device.location",
            Capability::Telemetry => "telemetry",
        }
    }

    pub fn parse(s: &str) -> Option<Capability> {
        Capability::ALL.into_iter().find(|c| c.as_str() == s)
    }

    /// 隐私红线能力：需要用户逐项、显式、可撤销地确认，永不自动授予。
    pub fn is_privacy_sensitive(self) -> bool {
        matches!(
            self,
            Capability::Telemetry
                | Capability::ProfileRead
                | Capability::ClipboardRead
                | Capability::DeviceCamera
                | Capability::DeviceLocation
        )
    }

    /// 安全红线能力：可能改变隔离边界，锁定模式下直接拒绝。
    pub fn is_security_sensitive(self) -> bool {
        matches!(
            self,
            Capability::EngineSwap
                | Capability::NetworkStackSwap
                | Capability::SandboxPolicyRead
                | Capability::NetIntercept
        )
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 用户授予某个插件的能力集合。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Grant {
    granted: Vec<Capability>,
}

impl Grant {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow(mut self, cap: Capability) -> Self {
        if !self.granted.contains(&cap) {
            self.granted.push(cap);
        }
        self
    }

    pub fn allows(&self, cap: Capability) -> bool {
        self.granted.contains(&cap)
    }

    pub fn caps(&self) -> &[Capability] {
        &self.granted
    }

    /// 生成"实际可用于安装"的授权：
    /// 未经用户显式确认时，所有隐私红线能力一律剔除。
    ///
    /// 这一步是隐私不可丢的落地点——即使开发者在清单里写了
    /// `capability: telemetry`，只要用户没单独点头，它就不会进注册表。
    pub fn sanitized(&self, user_confirmed: bool) -> Grant {
        let mut out = Grant::new();
        for cap in &self.granted {
            if cap.is_privacy_sensitive() && !user_confirmed {
                continue;
            }
            out = out.allow(*cap);
        }
        out
    }

    /// 求交集：插件声明的能力 ∩ 用户授予的能力。
    pub fn intersect(&self, requested: &[Capability]) -> Vec<Capability> {
        let mut out: Vec<Capability> = requested
            .iter()
            .copied()
            .filter(|c| self.allows(*c))
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GrantError {
    NotGranted(Capability),
    PrivacyConfirmationRequired(Capability),
}

impl fmt::Display for GrantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GrantError::NotGranted(c) => write!(f, "能力未被授予: {c}"),
            GrantError::PrivacyConfirmationRequired(c) => {
                write!(f, "隐私红线能力需要用户显式确认: {c}")
            }
        }
    }
}

impl std::error::Error for GrantError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_roundtrip_for_every_capability() {
        for cap in Capability::ALL {
            assert_eq!(Capability::parse(cap.as_str()), Some(cap), "{cap}");
        }
        assert_eq!(Capability::parse("not.a.capability"), None);
    }

    #[test]
    fn privacy_caps_are_stripped_without_confirmation() {
        let g = Grant::new()
            .allow(Capability::StorageRead)
            .allow(Capability::Telemetry)
            .allow(Capability::DeviceLocation);
        let clean = g.sanitized(false);
        assert!(clean.allows(Capability::StorageRead));
        assert!(!clean.allows(Capability::Telemetry));
        assert!(!clean.allows(Capability::DeviceLocation));

        let confirmed = g.sanitized(true);
        assert!(confirmed.allows(Capability::Telemetry));
    }

    #[test]
    fn intersect_is_least_privilege() {
        let g = Grant::new()
            .allow(Capability::NetConnect)
            .allow(Capability::UiChrome);
        let got = g.intersect(&[
            Capability::NetConnect,
            Capability::StorageWrite,
            Capability::NetConnect,
        ]);
        assert_eq!(got, vec![Capability::NetConnect]);
    }

    #[test]
    fn security_sensitive_cover_swaps_and_interception() {
        assert!(Capability::EngineSwap.is_security_sensitive());
        assert!(Capability::NetIntercept.is_security_sensitive());
        assert!(!Capability::NetConnect.is_security_sensitive());
    }
}
