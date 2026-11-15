//! # Fenestra 权限闸门（层 A）
//!
//! 内核唯一钉死的不变量，全部在这里以纯 Rust、零第三方依赖实现：
//!
//! 1. **默认拒绝**：任何能力，未显式声明 / 未被策略允许 / 未获用户逐项确认，一律拒绝。
//! 2. **最小权限交集**：插件的有效能力 = 声明集 ∩ 策略上限 ∩ 用户确认集。
//! 3. **`Untrusted<T>` 出站重校验**（BlastDoor 式）：隔离单元产出的数据被包成
//!    `Untrusted<T>`，不暴露任何读取 `inner` 的公开途径；离开隔离边界前必须
//!    经过 `revalidate()` 校验，成功后才得到可读取的 `Trusted<T>`。
//! 4. **可逆登记**：任何登记都能按登记的逆序回滚，卸载插件后不留残余。
//!
//! 这份闸门刻意不依赖系统库与第三方 crate，可独立 `cargo test`。

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

/// Fenestra 核心能力。隐私红线能力须经用户逐项显式确认。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capability {
    DnsResolve,
    NetOutbound,
    StorageWrite,
    FileRead,
    // ---- 隐私红线 ----
    ClipboardRead,
    ProfileRead,
    CameraDevice,
    LocationDevice,
    Telemetry,
}

impl Capability {
    /// 是否为隐私红线能力（须逐项显式确认，默认拒绝）。
    pub fn is_privacy_sensitive(self) -> bool {
        matches!(
            self,
            Capability::ClipboardRead
                | Capability::ProfileRead
                | Capability::CameraDevice
                | Capability::LocationDevice
                | Capability::Telemetry
        )
    }
}

pub type PermissionSet = BTreeSet<Capability>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DenyReason {
    /// 插件未声明该能力（默认拒绝）。
    NotDeclared,
    /// 声明了，但不在系统策略上限内（被最小权限交集削减）。
    PolicyForbidden,
    /// 隐私红线能力，尚未获得用户逐项确认。
    ConsentRequired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny(DenyReason),
}

impl Decision {
    pub fn is_allowed(self) -> bool {
        matches!(self, Decision::Allow)
    }
}

// ---------------------------------------------------------------------------
// BlastDoor 式出站重校验
// ---------------------------------------------------------------------------

/// 来自隔离单元的、尚未验证的输出。
///
/// 关键约束：`inner` 私有，**不实现 `Deref`/`AsRef`，也不提供 `into_inner`**。
/// 想拿到里面的值，唯一路径是 `revalidate()`；校验不通过则值随 `self` 一同丢弃。
/// 这正是「出站重校验」：数据离开隔离边界前必须重新校验，且无法绕过。
#[derive(Debug)]
pub struct Untrusted<T> {
    inner: T,
}

impl<T> Untrusted<T> {
    /// 只能在隔离单元内部调用：把产出标记为不可信。
    pub fn wrap(inner: T) -> Self {
        Self { inner }
    }

    /// 出站重校验。`validator` 通过引用审视载荷，返回 `Ok(())` 才放行。
    /// 校验器原则上执行「内容是否符合出站策略」的检查，例如 scheme 白名单、
    /// 长度上限、结构完整性、是否夹带未声明字段等。
    pub fn revalidate<E, F>(self, validator: F) -> Result<Trusted<T>, RevalidationError<E>>
    where
        F: FnOnce(&T) -> Result<(), E>,
    {
        match validator(&self.inner) {
            Ok(()) => Ok(Trusted(self.inner)),
            Err(source) => Err(RevalidationError { source }),
        }
    }
}

/// 通过出站重校验的可信值。构造器私有，外部无法凭空伪造。
#[derive(Debug)]
pub struct Trusted<T>(T);

impl<T> Trusted<T> {
    pub fn get(&self) -> &T {
        &self.0
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RevalidationError<E> {
    /// 校验器给出的拒绝原因，交由调用方决定如何记录 / 提示。
    pub source: E,
}

// ---------------------------------------------------------------------------
// 可逆登记
// ---------------------------------------------------------------------------

struct Entry<T> {
    id: u64,
    value: T,
}

/// 可逆登记表：登记返回句柄；`rollback()` 按登记逆序回滚全部，`extract()`
/// 按逆序摘出满足条件的登记。回滚后不留残余。
pub struct RevertibleRegistry<T> {
    entries: Vec<Entry<T>>,
    next_id: u64,
}

impl<T> RevertibleRegistry<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 0,
        }
    }

    pub fn register(&mut self, value: T) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(Entry { id, value });
        id
    }

    pub fn get(&self, id: u64) -> Option<&T> {
        self.entries.iter().find(|e| e.id == id).map(|e| &e.value)
    }

    pub fn unregister(&mut self, id: u64) -> Option<T> {
        let pos = self.entries.iter().position(|e| e.id == id)?;
        Some(self.entries.remove(pos).value)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 全部逆序回滚：返回按登记逆序排列的值，调用后表为空。
    pub fn rollback(&mut self) -> Vec<T> {
        let mut values: Vec<T> = self.entries.drain(..).map(|e| e.value).collect();
        values.reverse();
        values
    }

    /// 按逆序摘出满足 `pred` 的登记，其余保留。用于「卸载某插件」。
    pub fn extract<F>(&mut self, mut pred: F) -> Vec<T>
    where
        F: FnMut(&T) -> bool,
    {
        let mut matched = Vec::new();
        let mut kept = Vec::new();
        for entry in self.entries.drain(..) {
            if pred(&entry.value) {
                matched.push(entry.value);
            } else {
                kept.push(entry);
            }
        }
        matched.reverse();
        self.entries = kept;
        matched
    }
}

impl<T> Default for RevertibleRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// 闸门
// ---------------------------------------------------------------------------

/// 一条可逆登记的能力授予记录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grant {
    pub plugin: String,
    pub capability: Capability,
}

/// 权限闸门。默认拒绝：新建时策略上限为空，一切能力都不被允许，直到显式
/// `set_policy`。插件 `declare` 只是声明意图，真正的授权在 `check` 时按
/// 「声明 ∩ 策略 ∩ 确认」判定。
pub struct Gate {
    policy: PermissionSet,
    declared: BTreeMap<String, PermissionSet>,
    confirmed: BTreeMap<String, PermissionSet>,
    registry: RevertibleRegistry<Grant>,
}

impl Gate {
    /// 默认拒绝：策略上限为空集。
    pub fn new() -> Self {
        Self {
            policy: PermissionSet::new(),
            declared: BTreeMap::new(),
            confirmed: BTreeMap::new(),
            registry: RevertibleRegistry::new(),
        }
    }

    /// 设置系统策略上限（例如由 Tauri PCS 能力清单下发）。
    pub fn set_policy(&mut self, policy: PermissionSet) {
        self.policy = policy;
    }

    pub fn policy(&self) -> &PermissionSet {
        &self.policy
    }

    /// 声明一个插件想要的能力。每条新声明都登记为可逆记录；重复声明不重复登记。
    pub fn declare(&mut self, plugin: &str, caps: PermissionSet) {
        let to_register: Vec<Capability> = {
            let set = self.declared.entry(plugin.to_string()).or_default();
            caps.iter().copied().filter(|c| set.insert(*c)).collect()
        };
        for cap in to_register {
            self.registry.register(Grant {
                plugin: plugin.to_string(),
                capability: cap,
            });
        }
    }

    /// 用户对某隐私红线能力逐项显式确认。
    pub fn confirm(&mut self, plugin: &str, cap: Capability) {
        self.confirmed
            .entry(plugin.to_string())
            .or_default()
            .insert(cap);
    }

    pub fn declared(&self, plugin: &str) -> PermissionSet {
        self.declared.get(plugin).cloned().unwrap_or_default()
    }

    /// 最小权限交集：声明集 ∩ 策略上限，隐私项再 ∩ 已确认集。
    pub fn effective(&self, plugin: &str) -> PermissionSet {
        let declared = match self.declared.get(plugin) {
            Some(d) => d,
            None => return PermissionSet::new(),
        };
        let confirmed = self.confirmed.get(plugin);
        declared
            .iter()
            .copied()
            .filter(|c| self.policy.contains(c))
            .filter(|c| {
                !c.is_privacy_sensitive() || confirmed.is_some_and(|s| s.contains(c))
            })
            .collect()
    }

    /// 核心能力统一权限检查：默认拒绝。
    pub fn check(&self, plugin: &str, cap: Capability) -> Decision {
        if !self.declared.get(plugin).is_some_and(|s| s.contains(&cap)) {
            return Decision::Deny(DenyReason::NotDeclared);
        }
        if !self.policy.contains(&cap) {
            return Decision::Deny(DenyReason::PolicyForbidden);
        }
        if cap.is_privacy_sensitive()
            && !self.confirmed.get(plugin).is_some_and(|s| s.contains(&cap))
        {
            return Decision::Deny(DenyReason::ConsentRequired);
        }
        Decision::Allow
    }

    /// 卸载插件：逆序回滚该插件全部登记，清除声明与确认，无残留。
    /// 返回值按登记逆序排列，供宿主核对「撤销了什么」。
    pub fn revoke(&mut self, plugin: &str) -> Vec<Grant> {
        self.declared.remove(plugin);
        self.confirmed.remove(plugin);
        self.registry.extract(|grant| grant.plugin.as_str() == plugin)
    }

    /// 当前仍在册的登记数量（用于断言「卸载无残留」）。
    pub fn registrations(&self) -> usize {
        self.registry.len()
    }
}

impl Default for Gate {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(caps: &[Capability]) -> PermissionSet {
        caps.iter().copied().collect()
    }

    fn caps(items: &[Capability]) -> PermissionSet {
        items.iter().copied().collect()
    }

    // --- 默认拒绝 / 最小权限交集 ---

    #[test]
    fn default_deny_when_nothing_declared() {
        let gate = Gate::new();
        assert!(gate.policy().is_empty());
        assert_eq!(
            gate.check("p", Capability::NetOutbound),
            Decision::Deny(DenyReason::NotDeclared)
        );
        assert!(gate.effective("p").is_empty());
    }

    #[test]
    fn allow_after_declare_and_policy() {
        let mut gate = Gate::new();
        gate.set_policy(policy(&[
            Capability::DnsResolve,
            Capability::NetOutbound,
        ]));
        gate.declare("accel", caps(&[Capability::DnsResolve]));
        assert_eq!(
            gate.check("accel", Capability::DnsResolve),
            Decision::Allow
        );
        assert!(gate.check("accel", Capability::DnsResolve).is_allowed());
        assert_eq!(gate.registrations(), 1);
    }

    #[test]
    fn undeclared_capability_is_denied() {
        let mut gate = Gate::new();
        gate.set_policy(policy(&[Capability::NetOutbound]));
        assert_eq!(
            gate.check("accel", Capability::NetOutbound),
            Decision::Deny(DenyReason::NotDeclared)
        );
    }

    #[test]
    fn policy_trims_declared_set() {
        let mut gate = Gate::new();
        gate.set_policy(policy(&[Capability::DnsResolve]));
        gate.declare(
            "accel",
            caps(&[
                Capability::DnsResolve,
                Capability::NetOutbound,
                Capability::FileRead,
            ]),
        );
        assert_eq!(
            gate.check("accel", Capability::NetOutbound),
            Decision::Deny(DenyReason::PolicyForbidden)
        );
        assert_eq!(
            gate.check("accel", Capability::DnsResolve),
            Decision::Allow
        );
    }

    #[test]
    fn effective_is_minimum_privilege_intersection() {
        let mut gate = Gate::new();
        gate.set_policy(policy(&[
            Capability::DnsResolve,
            Capability::StorageWrite,
        ]));
        gate.declare(
            "accel",
            caps(&[Capability::DnsResolve, Capability::FileRead]),
        );
        let eff = gate.effective("accel");
        assert_eq!(eff.len(), 1);
        assert!(eff.contains(&Capability::DnsResolve));
        assert!(!eff.contains(&Capability::FileRead));
        assert!(!eff.contains(&Capability::StorageWrite));
    }

    #[test]
    fn declaring_twice_does_not_duplicate_registration() {
        let mut gate = Gate::new();
        gate.set_policy(policy(&[Capability::DnsResolve]));
        for _ in 0..3 {
            gate.declare("accel", caps(&[Capability::DnsResolve]));
        }
        assert_eq!(gate.registrations(), 1);
        assert_eq!(gate.effective("accel").len(), 1);
    }

    // --- 隐私红线逐项确认 ---

    #[test]
    fn privacy_capability_requires_explicit_consent() {
        let mut gate = Gate::new();
        gate.set_policy(policy(&[Capability::ClipboardRead]));
        gate.declare("p", caps(&[Capability::ClipboardRead]));
        assert_eq!(
            gate.check("p", Capability::ClipboardRead),
            Decision::Deny(DenyReason::ConsentRequired)
        );
        gate.confirm("p", Capability::ClipboardRead);
        assert_eq!(
            gate.check("p", Capability::ClipboardRead),
            Decision::Allow
        );
    }

    #[test]
    fn consent_is_granted_per_capability() {
        let mut gate = Gate::new();
        gate.set_policy(policy(&[
            Capability::ClipboardRead,
            Capability::ProfileRead,
        ]));
        gate.declare(
            "p",
            caps(&[Capability::ClipboardRead, Capability::ProfileRead]),
        );
        gate.confirm("p", Capability::ClipboardRead);
        assert_eq!(
            gate.check("p", Capability::ClipboardRead),
            Decision::Allow
        );
        assert_eq!(
            gate.check("p", Capability::ProfileRead),
            Decision::Deny(DenyReason::ConsentRequired)
        );
    }

    #[test]
    fn privacy_flag_covers_only_redline_capabilities() {
        assert!(Capability::Telemetry.is_privacy_sensitive());
        assert!(Capability::LocationDevice.is_privacy_sensitive());
        assert!(Capability::ClipboardRead.is_privacy_sensitive());
        assert!(!Capability::DnsResolve.is_privacy_sensitive());
        assert!(!Capability::NetOutbound.is_privacy_sensitive());
        assert!(!Capability::StorageWrite.is_privacy_sensitive());
    }

    // --- Untrusted<T> 出站重校验 ---

    #[test]
    fn untrusted_revalidation_success_yields_trusted() {
        let u = Untrusted::wrap("https://example.com".to_string());
        let t = u
            .revalidate(|s| {
                if s.starts_with("https://") {
                    Ok::<(), &'static str>(())
                } else {
                    Err("only https allowed")
                }
            })
            .expect("https 应通过重校验");
        assert_eq!(t.get(), "https://example.com");
        assert_eq!(t.into_inner(), "https://example.com");
    }

    #[test]
    fn untrusted_revalidation_failure_blocks_egress() {
        let u = Untrusted::wrap("javascript:alert(1)".to_string());
        let result = u.revalidate(|s| {
            if s.starts_with("https://") {
                Ok::<(), &'static str>(())
            } else {
                Err("scheme not allowed")
            }
        });
        let err = result.expect_err("危险 scheme 必须被出站重校验拦截");
        assert_eq!(err.source, "scheme not allowed");
    }

    #[test]
    fn revalidate_validator_observes_payload_exactly_once() {
        use std::cell::Cell;
        let calls = Cell::new(0usize);
        let u = Untrusted::wrap(vec![1u8, 2, 3]);
        let t = u
            .revalidate(|payload: &Vec<u8>| {
                calls.set(calls.get() + 1);
                assert_eq!(payload, &vec![1u8, 2, 3]);
                Ok::<(), ()>(())
            })
            .unwrap();
        assert_eq!(calls.get(), 1);
        assert_eq!(t.into_inner().len(), 3);
    }

    #[test]
    fn untrusted_payload_carries_no_unsafe_shortcut() {
        // Untrusted<T> 不提供 Deref / into_inner：无法在不经 revalidate 的情况下取值。
        // 这里通过「只有 revalidate 返回 Trusted 才能读到 inner」间接断言该约束。
        let u = Untrusted::wrap(42u32);
        let trusted = u.revalidate(|_n| Ok::<(), ()>(())).unwrap();
        assert_eq!(*trusted.get(), 42);
    }

    // --- 可逆登记 / 逆序回滚 / 无残留 ---

    #[test]
    fn registry_rollback_is_lifo_and_leaves_no_residue() {
        let mut reg = RevertibleRegistry::new();
        reg.register("first");
        reg.register("second");
        reg.register("third");
        assert_eq!(reg.len(), 3);
        let order = reg.rollback();
        assert_eq!(order, vec!["third", "second", "first"]);
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
        assert_eq!(reg.rollback().len(), 0);
    }

    #[test]
    fn registry_unregister_removes_single_entry() {
        let mut reg = RevertibleRegistry::new();
        let a = reg.register("a");
        let b = reg.register("b");
        assert_eq!(reg.unregister(a), Some("a"));
        assert_eq!(reg.get(a), None);
        assert_eq!(reg.get(b), Some(&"b"));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn registry_extract_is_reverse_ordered() {
        let mut reg = RevertibleRegistry::new();
        reg.register(("a", 1));
        reg.register(("b", 2));
        reg.register(("a", 3));
        let removed = reg.extract(|(tag, _)| *tag == "a");
        assert_eq!(removed, vec![("a", 3), ("a", 1)]);
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.get(1), Some(&("b", 2)));
    }

    #[test]
    fn gate_revoke_clears_declaration_and_consent_without_residue() {
        let mut gate = Gate::new();
        gate.set_policy(policy(&[
            Capability::DnsResolve,
            Capability::ClipboardRead,
        ]));
        gate.declare(
            "accel",
            caps(&[Capability::DnsResolve, Capability::ClipboardRead]),
        );
        gate.confirm("accel", Capability::ClipboardRead);
        assert_eq!(gate.registrations(), 2);

        let removed = gate.revoke("accel");
        assert_eq!(removed.len(), 2);
        // 逆序：ClipboardRead 后于 DnsResolve 声明（枚举顺序），故先被撤销。
        assert_eq!(gate.registrations(), 0);
        assert_eq!(
            gate.check("accel", Capability::DnsResolve),
            Decision::Deny(DenyReason::NotDeclared)
        );
        assert_eq!(
            gate.check("accel", Capability::ClipboardRead),
            Decision::Deny(DenyReason::NotDeclared)
        );
        assert!(gate.effective("accel").is_empty());
    }

    #[test]
    fn revoke_isolates_plugins() {
        let mut gate = Gate::new();
        gate.set_policy(policy(&[Capability::DnsResolve]));
        for p in ["a", "b"] {
            gate.declare(p, caps(&[Capability::DnsResolve]));
        }
        assert_eq!(gate.registrations(), 2);
        gate.revoke("a");
        assert_eq!(
            gate.check("a", Capability::DnsResolve),
            Decision::Deny(DenyReason::NotDeclared)
        );
        assert_eq!(gate.check("b", Capability::DnsResolve), Decision::Allow);
        assert_eq!(gate.registrations(), 1);
    }
}
