//! # Fenestra —— 只是个壳
//!
//! > The browser is a shell. Everything else is a plugin.
//!
//! Fenestra 的内核只有三件事：
//! 1. 读清单（插件声明自己是谁、想要什么能力）；
//! 2. 仲裁能力（默认拒绝，隐私红线需用户显式确认）；
//! 3. 可逆登记（安装 = 登记一组可撤销效应，卸载 = 精确撤销）。
//!
//! 除此之外什么都没有：没有渲染引擎、没有网络栈、没有界面、没有书签、
//! 没有历史、没有设置页、没有遥测。它们都是插件，可以换、可以删、可以自己写。
//!
//! 这个 crate 零第三方依赖，可在 Android / iOS / Windows / Linux / macOS 上编译。

pub mod capability;
pub mod compat;
pub mod engine;
pub mod manifest;
pub mod registry;
pub mod sandbox;

pub use capability::{Capability, Grant, GrantError};
pub use compat::{from_json_manifest, parse_json, to_hep, CompatError, CompatReport, Json};
pub use engine::{
    EngineError, ExtensionFlavor, ExtensionRuntime, Frame, NetworkStack, PlatformBackend,
    RenderEngine, RenderRequest, ShellUi, Target,
};
pub use manifest::{Manifest, ManifestError, PluginKind};
pub use registry::{Registration, Registry, RegistryError};
pub use sandbox::{IsolationLevel, SandboxPolicy, Untrusted, Verdict};

pub const KERNEL_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const FEP_VERSION: u8 = 1;

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellError {
    Manifest(ManifestError),
    Registry(RegistryError),
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShellError::Manifest(e) => write!(f, "{e}"),
            ShellError::Registry(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ShellError {}

impl From<ManifestError> for ShellError {
    fn from(e: ManifestError) -> Self {
        ShellError::Manifest(e)
    }
}

impl From<RegistryError> for ShellError {
    fn from(e: RegistryError) -> Self {
        ShellError::Registry(e)
    }
}

/// 空壳本体。
#[derive(Debug)]
pub struct Shell {
    registry: Registry,
    policy: SandboxPolicy,
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

impl Shell {
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
            policy: SandboxPolicy::default(),
        }
    }

    pub fn with_policy(policy: SandboxPolicy) -> Self {
        Self {
            registry: Registry::new(),
            policy,
        }
    }

    /// 只校验清单，不安装。
    pub fn verify(text: &str) -> Result<Manifest, ManifestError> {
        Manifest::parse(text)
    }

    /// 安装插件：解析清单 → 最小权限求交 → 可逆登记。
    ///
    /// 注意这里不执行任何插件代码。内核不加载 wasm、不启动 JS 运行时，
    /// 真正的执行发生在平台后端插件里，并且发生在沙盒仲裁之后。
    pub fn install(&mut self, text: &str, grant: &Grant) -> Result<Registration, ShellError> {
        let manifest = Manifest::parse(text)?;
        let mut effective = Grant::new();
        for cap in grant.intersect(&manifest.capabilities) {
            effective = effective.allow(cap);
        }
        Ok(self.registry.register(&manifest, &effective)?)
    }

    pub fn uninstall(&mut self, registration_id: u64) -> bool {
        self.registry.revoke(registration_id)
    }

    pub fn uninstall_plugin(&mut self, plugin_id: &str) -> usize {
        self.registry.revoke_plugin(plugin_id)
    }

    /// 运行时能力仲裁。插件不存在、能力未授予、隐私未确认、锁定模式
    /// 命中安全红线——四种情况一律拒绝。
    pub fn request(&self, plugin_id: &str, cap: Capability, user_confirmed: bool) -> Verdict {
        let Some(registration) = self
            .registry
            .items()
            .iter()
            .find(|r| r.plugin_id == plugin_id)
        else {
            return Verdict::Deny;
        };
        let mut grant = Grant::new();
        for cap in &registration.capabilities {
            grant = grant.allow(*cap);
        }
        self.policy.decide(&grant, cap, user_confirmed)
    }

    pub fn active(&self) -> &[Registration] {
        self.registry.items()
    }

    pub fn services(&self) -> Vec<String> {
        self.registry.services()
    }

    pub fn publish(&self, topic: &str) -> Vec<u64> {
        self.registry.publish(topic)
    }

    pub fn policy(&self) -> &SandboxPolicy {
        &self.policy
    }

    pub fn harden(&mut self) {
        self.policy.harden();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{EXAMPLE_ENGINE, EXAMPLE_EXT, EXAMPLE_UI};

    fn generous_grant() -> Grant {
        Grant::new()
            .allow(Capability::EngineSwap)
            .allow(Capability::NetConnect)
            .allow(Capability::NetIntercept)
            .allow(Capability::UiChrome)
            .allow(Capability::UiContent)
            .allow(Capability::StorageRead)
            .allow(Capability::Telemetry)
    }

    #[test]
    fn shell_installs_plugins_and_exposes_services() {
        let mut shell = Shell::new();
        let g = generous_grant();
        shell.install(EXAMPLE_ENGINE, &g).unwrap();
        shell.install(EXAMPLE_UI, &g).unwrap();
        shell.install(EXAMPLE_EXT, &g).unwrap();
        assert_eq!(shell.active().len(), 3);
        assert!(shell.services().contains(&"engine.render".to_string()));
        assert!(shell.services().contains(&"ui.mount".to_string()));
        assert_eq!(shell.publish("shell.boot").len(), 3);
    }

    #[test]
    fn uninstall_is_reversible() {
        let mut shell = Shell::new();
        let g = generous_grant();
        let reg = shell.install(EXAMPLE_ENGINE, &g).unwrap();
        assert!(!shell.services().is_empty());
        assert!(shell.uninstall(reg.id));
        assert!(shell.services().is_empty());
        assert!(shell.publish("shell.boot").is_empty());
    }

    #[test]
    fn ungranted_capability_is_denied_at_runtime() {
        let mut shell = Shell::new();
        let g = Grant::new().allow(Capability::NetConnect);
        shell.install(EXAMPLE_ENGINE, &g).unwrap();
        assert_eq!(
            shell.request("org.fenestra.engine.blink", Capability::NetConnect, false),
            Verdict::Allow
        );
        assert_eq!(
            shell.request("org.fenestra.engine.blink", Capability::EngineSwap, false),
            Verdict::Deny
        );
        assert_eq!(
            shell.request("no.such.plugin", Capability::NetConnect, false),
            Verdict::Deny
        );
    }

    #[test]
    fn telemetry_is_blocked_until_user_confirms() {
        let mut shell = Shell::new();
        let text = "\
id: com.example.probe
name: Probe
version: 1.0.0
kind: other
entry: probe.wasm
capability: telemetry
";
        let g = Grant::new().allow(Capability::Telemetry);
        shell.install(text, &g).unwrap();
        assert_eq!(
            shell.request("com.example.probe", Capability::Telemetry, false),
            Verdict::Deny
        );
        assert_eq!(
            shell.request("com.example.probe", Capability::Telemetry, true),
            Verdict::Allow
        );
    }

    #[test]
    fn lockdown_freezes_engine_swapping() {
        let mut shell = Shell::new();
        let g = generous_grant();
        shell.install(EXAMPLE_ENGINE, &g).unwrap();
        let id = "org.fenestra.engine.blink";
        assert_eq!(
            shell.request(id, Capability::EngineSwap, false),
            Verdict::Allow
        );
        shell.harden();
        assert_eq!(
            shell.request(id, Capability::EngineSwap, false),
            Verdict::Deny
        );
        assert_eq!(
            shell.request(id, Capability::NetConnect, false),
            Verdict::Allow
        );
    }

    #[test]
    fn duplicate_install_is_rejected() {
        let mut shell = Shell::new();
        let g = generous_grant();
        shell.install(EXAMPLE_UI, &g).unwrap();
        assert!(matches!(
            shell.install(EXAMPLE_UI, &g),
            Err(ShellError::Registry(RegistryError::AlreadyInstalled(_)))
        ));
    }
}
