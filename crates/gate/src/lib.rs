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

use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Fenestra 核心能力。隐私红线能力须经用户逐项显式确认。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capability {
    DnsResolve,
    NetOutbound,
    StorageWrite,
    FileRead,
    UiNewTab,
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

    /// 把插件 `meta.permissions` 里声明的权限字符串解析为闸门能力。
    ///
    /// 这是 TS 侧权限字符串与 Rust 闸门之间的**唯一**映射入口；未知字符串
    /// 返回 `None`，由 `Gate::check_permission` 按默认拒绝处理，绝不猜测放行。
    pub fn from_permission(permission: &str) -> Option<Capability> {
        match permission.trim() {
            "ui.newtab" => Some(Capability::UiNewTab),
            "net.dns" | "dns" | "dns.resolve" => Some(Capability::DnsResolve),
            "net.preconnect" | "preconnect" | "net.outbound" => Some(Capability::NetOutbound),
            "storage.write" => Some(Capability::StorageWrite),
            "file.read" => Some(Capability::FileRead),
            "clipboard.read" => Some(Capability::ClipboardRead),
            "profile.read" => Some(Capability::ProfileRead),
            "camera" | "camera.device" => Some(Capability::CameraDevice),
            "location" | "location.device" => Some(Capability::LocationDevice),
            "telemetry" => Some(Capability::Telemetry),
            _ => None,
        }
    }
}

pub type PermissionSet = BTreeSet<Capability>;

// ---------------------------------------------------------------------------
// 外部链接出站校验（外壳「用系统浏览器打开」入口）
// ---------------------------------------------------------------------------

/// 外部链接被拒绝的原因。前端可据此给出明确提示，而不是只显示「打不开」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExternalUrlError {
    Empty,
    TooLong,
    UnsupportedScheme,
    MissingHost,
    CredentialsNotAllowed,
    InvalidPort,
    LocalTarget,
}

impl ExternalUrlError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Empty => "地址为空",
            Self::TooLong => "地址过长（上限 2048 字符）",
            Self::UnsupportedScheme => "只允许 http:// 或 https:// 链接",
            Self::MissingHost => "缺少主机名",
            Self::CredentialsNotAllowed => "外链不允许携带用户名密码",
            Self::InvalidPort => "端口号非法",
            Self::LocalTarget => "不允许用系统浏览器打开本机 / 内网地址",
        }
    }
}

fn parse_ipv4(host: &str) -> Option<[u8; 4]> {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    let mut out = [0u8; 4];
    for (i, p) in parts.iter().enumerate() {
        if p.is_empty() || p.len() > 3 || !p.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        out[i] = p.parse().ok()?;
    }
    Some(out)
}

fn is_local_ipv4(ip: [u8; 4]) -> bool {
    let [a, b, _, _] = ip;
    a == 0
        || a == 10
        || a == 127
        || (a == 100 && (64..=127).contains(&b))
        || (a == 169 && b == 254)
        || (a == 172 && (16..=31).contains(&b))
        || (a == 192 && (b == 0 || b == 168))
        || (a == 198 && (b == 18 || b == 19))
        || a >= 224
}

/// 校验「交给系统浏览器打开」的 URL。只接收 http(s) 公网链接，拒绝把
/// `file://` / `javascript:` / 本机与内网地址交给外部程序；返回规范化后的
/// 原串（仅去首尾空白），不改写用户输入的路径与查询。
pub fn validate_external_http_url(raw: &str) -> Result<String, ExternalUrlError> {
    let url = raw.trim();
    if url.is_empty() {
        return Err(ExternalUrlError::Empty);
    }
    if url.len() > 2048 || url.chars().any(|c| c.is_control()) {
        return Err(ExternalUrlError::TooLong);
    }
    let lower = url.to_ascii_lowercase();
    let rest = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
        .ok_or(ExternalUrlError::UnsupportedScheme)?;
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    if authority.is_empty() {
        return Err(ExternalUrlError::MissingHost);
    }
    if authority.contains('@') {
        return Err(ExternalUrlError::CredentialsNotAllowed);
    }
    let (host, port) = if let Some(inner) = rest.split('/').next().unwrap_or("").strip_prefix('[') {
        // IPv6 literal：外壳不构造这类外链，直接拒绝以免端口/区域 ID 解析歧义。
        let _ = inner;
        return Err(ExternalUrlError::LocalTarget);
    } else {
        match authority.rfind(':') {
            Some(i) => {
                let p = &authority[i + 1..];
                if p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(ExternalUrlError::InvalidPort);
                }
                let n: u32 = p.parse().map_err(|_| ExternalUrlError::InvalidPort)?;
                if n == 0 || n > 65535 {
                    return Err(ExternalUrlError::InvalidPort);
                }
                (&authority[..i], Some(n))
            }
            None => (authority, None),
        }
    };
    let _ = port;
    let host = host.trim_end_matches('.');
    if host.is_empty() {
        return Err(ExternalUrlError::MissingHost);
    }
    if host == "localhost" || host.ends_with(".localhost") || host.ends_with(".local") {
        return Err(ExternalUrlError::LocalTarget);
    }
    if let Some(ip) = parse_ipv4(host) {
        if is_local_ipv4(ip) {
            return Err(ExternalUrlError::LocalTarget);
        }
    }
    Ok(url.to_string())
}

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
// 失败可审计：有界的裁决记录
// ---------------------------------------------------------------------------

/// 审计日志容量上限。闸门内嵌在长期运行的浏览器外壳里，审计记录必须有界，
/// 否则反复的拒绝尝试会让内存无界增长（这本身就是一种 DoS 面）。
pub const AUDIT_CAPACITY: usize = 256;

/// 一条可审计的裁决记录：谁、要什么能力、结果如何（含拒绝原因）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionRecord {
    pub plugin: String,
    /// 解析出的能力；权限字符串无法解析时为 `None`（默认拒绝）。
    pub capability: Option<Capability>,
    /// 原始权限字符串（仅 `check_permission` 路径保留）。
    pub permission: Option<String>,
    pub decision: Decision,
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
    /// 最近 `AUDIT_CAPACITY` 条裁决记录（含拒绝原因），供「失败可审计」核对。
    audit: VecDeque<DecisionRecord>,
}

impl Gate {
    /// 默认拒绝：策略上限为空集。
    pub fn new() -> Self {
        Self {
            policy: PermissionSet::new(),
            declared: BTreeMap::new(),
            confirmed: BTreeMap::new(),
            registry: RevertibleRegistry::new(),
            audit: VecDeque::new(),
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

    /// 可审计的能力检查：在 [`Gate::check`] 的基础上记一条裁决（含拒绝原因）。
    pub fn check_audited(&mut self, plugin: &str, cap: Capability) -> Decision {
        self.record(plugin, Some(cap), None)
    }

    /// 权限字符串入口：先经 [`Capability::from_permission`] 解析，再做可审计检查。
    /// 未知权限字符串没有对应能力，按默认拒绝处理并同样记入审计。
    pub fn check_permission(&mut self, plugin: &str, permission: &str) -> Decision {
        let cap = Capability::from_permission(permission);
        self.record(plugin, cap, Some(permission.to_string()))
    }

    fn record(
        &mut self,
        plugin: &str,
        cap: Option<Capability>,
        permission: Option<String>,
    ) -> Decision {
        let decision = match cap {
            Some(c) => self.check(plugin, c),
            None => Decision::Deny(DenyReason::NotDeclared),
        };
        if self.audit.len() == AUDIT_CAPACITY {
            self.audit.pop_front();
        }
        self.audit.push_back(DecisionRecord {
            plugin: plugin.to_string(),
            capability: cap,
            permission,
            decision,
        });
        decision
    }

    /// 最近的裁决记录（最旧在前，最多 [`AUDIT_CAPACITY`] 条）。
    pub fn audit_log(&self) -> &VecDeque<DecisionRecord> {
        &self.audit
    }

    /// 只看被拒绝的裁决，供「失败可审计」核对。
    pub fn denials(&self) -> impl Iterator<Item = &DecisionRecord> {
        self.audit.iter().filter(|r| !r.decision.is_allowed())
    }

    /// 清空审计日志。
    pub fn clear_audit(&mut self) {
        self.audit.clear()
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

    // --- 权限字符串映射 / 可审计检查 ---

    #[test]
    fn permission_strings_map_to_capabilities() {
        assert_eq!(
            Capability::from_permission("ui.newtab"),
            Some(Capability::UiNewTab)
        );
        assert_eq!(
            Capability::from_permission("net.dns"),
            Some(Capability::DnsResolve)
        );
        assert_eq!(
            Capability::from_permission("net.preconnect"),
            Some(Capability::NetOutbound)
        );
        assert_eq!(
            Capability::from_permission("clipboard.read"),
            Some(Capability::ClipboardRead)
        );
        // 空白容错 + 未知字符串默认拒绝（不猜测放行）。
        assert_eq!(
            Capability::from_permission("  net.dns  "),
            Some(Capability::DnsResolve)
        );
        assert_eq!(Capability::from_permission("net.kernel"), None);
        assert_eq!(Capability::from_permission(""), None);
    }

    #[test]
    fn check_permission_matches_check_and_records_audit() {
        let mut gate = Gate::new();
        gate.set_policy(policy(&[Capability::DnsResolve]));
        // 两个能力都声明，但策略只放开 DnsResolve → net.outbound 命中 PolicyForbidden。
        gate.declare(
            "accel",
            caps(&[Capability::DnsResolve, Capability::NetOutbound]),
        );

        assert_eq!(
            gate.check_permission("accel", "net.dns"),
            gate.check("accel", Capability::DnsResolve)
        );
        assert!(gate.check_permission("accel", "net.dns").is_allowed());
        assert_eq!(
            gate.check_permission("accel", "net.outbound"),
            Decision::Deny(DenyReason::PolicyForbidden)
        );
        assert_eq!(gate.audit_log().len(), 3);

        let last = gate.audit_log().back().unwrap();
        assert_eq!(last.plugin, "accel");
        assert_eq!(last.permission.as_deref(), Some("net.outbound"));
        assert_eq!(last.capability, Some(Capability::NetOutbound));
        assert_eq!(last.decision, Decision::Deny(DenyReason::PolicyForbidden));
    }

    #[test]
    fn unknown_permission_is_default_denied_and_audited() {
        let mut gate = Gate::new();
        gate.set_policy(policy(&[Capability::DnsResolve]));
        gate.declare("accel", caps(&[Capability::DnsResolve]));
        assert_eq!(
            gate.check_permission("accel", "net.kernel"),
            Decision::Deny(DenyReason::NotDeclared)
        );
        let last = gate.audit_log().back().unwrap();
        assert_eq!(last.capability, None);
        assert_eq!(last.permission.as_deref(), Some("net.kernel"));
        assert_eq!(last.decision, Decision::Deny(DenyReason::NotDeclared));
    }

    #[test]
    fn denials_filters_only_rejected_decisions() {
        let mut gate = Gate::new();
        gate.set_policy(policy(&[Capability::DnsResolve]));
        gate.declare("accel", caps(&[Capability::DnsResolve]));
        gate.check_audited("accel", Capability::DnsResolve); // allow
        gate.check_audited("stranger", Capability::DnsResolve); // deny
        gate.check_audited("accel", Capability::ClipboardRead); // deny
        let denied: Vec<&DecisionRecord> = gate.denials().collect();
        assert_eq!(denied.len(), 2);
        assert_eq!(denied[0].plugin, "stranger");
        assert_eq!(denied[1].plugin, "accel");
        assert_eq!(denied[0].decision, Decision::Deny(DenyReason::NotDeclared));
        assert_eq!(denied[1].decision, Decision::Deny(DenyReason::NotDeclared));
    }

    #[test]
    fn audit_log_is_bounded_for_long_running_process() {
        let mut gate = Gate::new();
        gate.set_policy(policy(&[Capability::DnsResolve]));
        gate.declare("accel", caps(&[Capability::DnsResolve]));
        for _ in 0..(AUDIT_CAPACITY + 10) {
            gate.check_audited("stranger", Capability::DnsResolve);
        }
        assert_eq!(gate.audit_log().len(), AUDIT_CAPACITY);
        // 全部同源拒绝，最旧与最新都仍是 stranger 的拒绝记录。
        assert_eq!(gate.audit_log().front().unwrap().plugin, "stranger");
        assert_eq!(gate.audit_log().back().unwrap().plugin, "stranger");
        gate.clear_audit();
        assert!(gate.audit_log().is_empty());
    }

    #[test]
    fn external_url_validator_accepts_public_http_links() {
        assert_eq!(
            validate_external_http_url(" https://example.com/docs?a=1 "),
            Ok("https://example.com/docs?a=1".to_string())
        );
        assert!(validate_external_http_url("http://8.8.8.8:8080/x").is_ok());
    }

    #[test]
    fn external_url_validator_blocks_dangerous_and_local_targets() {
        for bad in [
            "javascript:alert(1)",
            "file:///etc/passwd",
            "https://",
            "https://user:pass@example.com",
            "https://localhost:5173",
            "http://127.0.0.1",
            "http://192.168.1.1",
            "http://169.254.169.254/latest/meta-data/",
            "http://10.0.0.1",
            "https://[::1]/",
            "https://example.com:70000",
            "https://example.com:abc",
        ] {
            assert!(
                validate_external_http_url(bad).is_err(),
                "危险外链必须被拒绝：{bad}"
            );
        }
    }

    #[test]
    fn external_url_error_has_actionable_message() {
        assert_eq!(
            validate_external_http_url("file:///etc/passwd").unwrap_err().message(),
            "只允许 http:// 或 https:// 链接"
        );
        assert_eq!(
            validate_external_http_url("http://127.0.0.1").unwrap_err().message(),
            "不允许用系统浏览器打开本机 / 内网地址"
        );
    }

    #[test]
    fn builtin_plugin_gate_checks_through_permission_strings() {
        // 外壳用同一套 API 构造：net.dns / net.preconnect 给 accelerate，
        // ui.newtab 给 newtab；其余默认拒绝。
        let mut gate = Gate::new();
        gate.set_policy(policy(&[
            Capability::UiNewTab,
            Capability::DnsResolve,
            Capability::NetOutbound,
        ]));
        gate.declare("newtab", caps(&[Capability::UiNewTab]));
        gate.declare(
            "accelerate",
            caps(&[Capability::DnsResolve, Capability::NetOutbound]),
        );
        assert!(gate.check_permission("newtab", "ui.newtab").is_allowed());
        assert!(gate
            .check_permission("accelerate", "net.dns")
            .is_allowed());
        assert!(gate
            .check_permission("accelerate", "net.preconnect")
            .is_allowed());
        // 跨插件越权：声明里没有，必须拒绝。
        assert_eq!(
            gate.check_permission("accelerate", "ui.newtab"),
            Decision::Deny(DenyReason::NotDeclared)
        );
        assert_eq!(
            gate.check_permission("stranger", "net.dns"),
            Decision::Deny(DenyReason::NotDeclared)
        );
    }
}
