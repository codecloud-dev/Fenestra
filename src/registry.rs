//! 可逆注册表。
//!
//! 插件"安装"不是把代码拷进来，而是往内核登记一组**可撤销的效应**：
//! 服务、总线订阅、能力授权。卸载就是精确撤销这一组，不留悬空状态。
//! 这个思路与 Cordis 的 reversible effects 一致，区别是 Fenestra 把它
//! 建立在能力仲裁之上，而不是建立在信任插件自觉清理之上。

use std::fmt;

use crate::capability::{Capability, Grant};
use crate::manifest::{Manifest, PluginKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Registration {
    pub id: u64,
    pub plugin_id: String,
    pub kind: PluginKind,
    /// 实际生效的能力 = 插件声明 ∩ 用户授予。
    pub capabilities: Vec<Capability>,
    pub services: Vec<String>,
    pub subscriptions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryError {
    AlreadyInstalled(String),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::AlreadyInstalled(id) => write!(f, "插件已安装，不可重复登记: {id}"),
        }
    }
}

impl std::error::Error for RegistryError {}

#[derive(Default, Debug)]
pub struct Registry {
    next_id: u64,
    items: Vec<Registration>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        manifest: &Manifest,
        grant: &Grant,
    ) -> Result<Registration, RegistryError> {
        if self.items.iter().any(|r| r.plugin_id == manifest.id) {
            return Err(RegistryError::AlreadyInstalled(manifest.id.clone()));
        }
        self.next_id += 1;
        let registration = Registration {
            id: self.next_id,
            plugin_id: manifest.id.clone(),
            kind: manifest.kind,
            capabilities: grant.intersect(&manifest.capabilities),
            services: manifest
                .kind
                .services()
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            subscriptions: manifest
                .kind
                .subscriptions()
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        };
        self.items.push(registration.clone());
        Ok(registration)
    }

    /// 精确撤销一次登记：服务、订阅、能力一并消失。
    pub fn revoke(&mut self, id: u64) -> bool {
        let before = self.items.len();
        self.items.retain(|r| r.id != id);
        before != self.items.len()
    }

    /// 按插件 id 撤销（改名/重装场景可能留下多条登记）。
    pub fn revoke_plugin(&mut self, plugin_id: &str) -> usize {
        let before = self.items.len();
        self.items.retain(|r| r.plugin_id != plugin_id);
        before - self.items.len()
    }

    pub fn items(&self) -> &[Registration] {
        &self.items
    }

    pub fn services(&self) -> Vec<String> {
        let mut all: Vec<String> = self.items.iter().flat_map(|r| r.services.clone()).collect();
        all.sort();
        all.dedup();
        all
    }

    /// 发布主题，返回本次会收到通知的登记 id。
    pub fn publish(&self, topic: &str) -> Vec<u64> {
        self.items
            .iter()
            .filter(|r| r.subscriptions.iter().any(|t| t == topic))
            .map(|r| r.id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(id: &str, kind: PluginKind) -> Manifest {
        Manifest {
            id: id.to_string(),
            name: id.to_string(),
            version: "1.0.0".to_string(),
            kind,
            entry: format!("{id}.wasm"),
            target: None,
            capabilities: vec![Capability::NetConnect, Capability::UiChrome],
            origins: Vec::new(),
        }
    }

    #[test]
    fn registration_is_least_privilege() {
        let mut r = Registry::new();
        let g = Grant::new().allow(Capability::NetConnect);
        let reg = r
            .register(&manifest("a.b", PluginKind::RenderEngine), &g)
            .unwrap();
        assert_eq!(reg.capabilities, vec![Capability::NetConnect]);
        assert_eq!(reg.services, vec!["engine.render", "engine.lifecycle"]);
    }

    #[test]
    fn duplicate_registration_is_rejected() {
        let mut r = Registry::new();
        let g = Grant::new();
        let m = manifest("a.b", PluginKind::Theme);
        r.register(&m, &g).unwrap();
        assert_eq!(
            r.register(&m, &g),
            Err(RegistryError::AlreadyInstalled("a.b".to_string()))
        );
    }

    #[test]
    fn revoke_removes_services_and_subscriptions() {
        let mut r = Registry::new();
        let reg = r
            .register(&manifest("a.b", PluginKind::RenderEngine), &Grant::new())
            .unwrap();
        assert!(!r.services().is_empty());
        assert_eq!(r.publish("shell.boot"), vec![reg.id]);

        assert!(r.revoke(reg.id));
        assert!(r.services().is_empty());
        assert!(r.publish("shell.boot").is_empty());
        assert!(!r.revoke(reg.id));
    }

    #[test]
    fn revoke_by_plugin_id_clears_all_registrations() {
        let mut r = Registry::new();
        let m = manifest("a.b", PluginKind::Storage);
        r.register(&m, &Grant::new()).unwrap();
        assert_eq!(r.revoke_plugin("a.b"), 1);
        assert!(r.items().is_empty());
    }
}
