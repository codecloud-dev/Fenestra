//! FEP（Fenestra Extension Package）清单解析。
//!
//! 我们不用 JSON 作为内核清单格式：内核要能在任何平台、任何语言运行时
//! 启动之前就读得懂自己要加载什么。FEP 是纯文本、逐行、无歧义的格式，
//! 解析器只依赖 `std`，约一百行，可被审计、可被移植、可被手写。
//!
//! ```text
//! # 注释以 # 或 // 开头
//! id: com.example.reader
//! name: Reader
//! version: 1.0.0
//! kind: shell-ui
//! entry: reader.wasm
//! target: linux
//! capability: storage.read
//! capability: ui.chrome
//! origin: https://example.com/*
//! ```

use std::fmt;

use crate::capability::Capability;
use crate::engine::Target;

/// 插件种类。
///
/// 注意这里包含 `render-engine` / `network-stack` / `sandbox-policy`：
/// 这正是 Fenestra 比"一切皆插件"的 DSH 更绝的地方——
/// DSH 的插件化止于应用层能力，渲染引擎与运行时本身仍是给定的；
/// Fenestra 连引擎、网络栈、沙盒策略都是清单里可替换的一行。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PluginKind {
    RenderEngine,
    NetworkStack,
    ShellUi,
    Storage,
    ExtensionRuntime,
    SandboxPolicy,
    Downloader,
    Bookmarks,
    History,
    Settings,
    Theme,
    Protocol,
    Other,
}

impl PluginKind {
    pub fn as_str(self) -> &'static str {
        match self {
            PluginKind::RenderEngine => "render-engine",
            PluginKind::NetworkStack => "network-stack",
            PluginKind::ShellUi => "shell-ui",
            PluginKind::Storage => "storage",
            PluginKind::ExtensionRuntime => "extension-runtime",
            PluginKind::SandboxPolicy => "sandbox-policy",
            PluginKind::Downloader => "downloader",
            PluginKind::Bookmarks => "bookmarks",
            PluginKind::History => "history",
            PluginKind::Settings => "settings",
            PluginKind::Theme => "theme",
            PluginKind::Protocol => "protocol",
            PluginKind::Other => "other",
        }
    }

    pub fn parse(s: &str) -> Option<PluginKind> {
        [
            PluginKind::RenderEngine,
            PluginKind::NetworkStack,
            PluginKind::ShellUi,
            PluginKind::Storage,
            PluginKind::ExtensionRuntime,
            PluginKind::SandboxPolicy,
            PluginKind::Downloader,
            PluginKind::Bookmarks,
            PluginKind::History,
            PluginKind::Settings,
            PluginKind::Theme,
            PluginKind::Protocol,
            PluginKind::Other,
        ]
        .into_iter()
        .find(|k| k.as_str() == s)
    }

    /// 该种类向注册表贡献的服务名。
    pub fn services(self) -> &'static [&'static str] {
        match self {
            PluginKind::RenderEngine => &["engine.render", "engine.lifecycle"],
            PluginKind::NetworkStack => &["net.fetch", "net.lifecycle"],
            PluginKind::ShellUi => &["ui.mount", "ui.chrome"],
            PluginKind::Storage => &["storage.get", "storage.put"],
            PluginKind::ExtensionRuntime => &["ext.load", "ext.message"],
            PluginKind::SandboxPolicy => &["sandbox.policy"],
            PluginKind::Downloader => &["download.enqueue"],
            PluginKind::Bookmarks => &["bookmarks.query"],
            PluginKind::History => &["history.query"],
            PluginKind::Settings => &["settings.ui"],
            PluginKind::Theme => &["theme.apply"],
            PluginKind::Protocol => &["protocol.handle"],
            PluginKind::Other => &[],
        }
    }

    /// 该种类关心的总线主题。
    pub fn subscriptions(self) -> &'static [&'static str] {
        match self {
            PluginKind::RenderEngine => &["shell.boot", "nav.commit"],
            PluginKind::NetworkStack => &["shell.boot"],
            PluginKind::ShellUi => &["shell.boot", "engine.frame"],
            PluginKind::ExtensionRuntime => &["shell.boot", "nav.commit"],
            PluginKind::SandboxPolicy => &["shell.boot", "capability.changed"],
            _ => &["shell.boot"],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub kind: PluginKind,
    pub entry: String,
    pub target: Option<Target>,
    pub capabilities: Vec<Capability>,
    pub origins: Vec<String>,
}

impl Manifest {
    /// 从 FEP 文本解析清单。
    pub fn parse(text: &str) -> Result<Manifest, ManifestError> {
        let mut id = None;
        let mut name = None;
        let mut version = None;
        let mut kind = None;
        let mut entry = None;
        let mut target = None;
        let mut capabilities: Vec<Capability> = Vec::new();
        let mut origins: Vec<String> = Vec::new();
        let mut seen: Vec<&'static str> = Vec::new();

        for (line_no, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                continue;
            }
            let (key, value) = line
                .split_once(':')
                .ok_or_else(|| ManifestError::MalformedLine(line_no + 1, line.to_string()))?;
            let key = key.trim();
            let value = value.trim();
            if value.is_empty() {
                return Err(ManifestError::MalformedLine(line_no + 1, line.to_string()));
            }

            match key {
                "id" => {
                    single(&mut seen, "id", line_no + 1)?;
                    validate_id(value)?;
                    id = Some(value.to_string());
                }
                "name" => {
                    single(&mut seen, "name", line_no + 1)?;
                    name = Some(value.to_string());
                }
                "version" => {
                    single(&mut seen, "version", line_no + 1)?;
                    validate_version(value)?;
                    version = Some(value.to_string());
                }
                "kind" => {
                    single(&mut seen, "kind", line_no + 1)?;
                    kind = Some(
                        PluginKind::parse(value)
                            .ok_or_else(|| ManifestError::InvalidKind(value.to_string()))?,
                    );
                }
                "entry" => {
                    single(&mut seen, "entry", line_no + 1)?;
                    entry = Some(value.to_string());
                }
                "target" => {
                    single(&mut seen, "target", line_no + 1)?;
                    target = Some(
                        Target::parse(value)
                            .ok_or_else(|| ManifestError::InvalidTarget(value.to_string()))?,
                    );
                }
                "capability" => {
                    let cap = Capability::parse(value)
                        .ok_or_else(|| ManifestError::InvalidCapability(value.to_string()))?;
                    if !capabilities.contains(&cap) {
                        capabilities.push(cap);
                    }
                }
                "origin" => {
                    if !origins.iter().any(|o| o == value) {
                        origins.push(value.to_string());
                    }
                }
                _ => return Err(ManifestError::UnknownKey(key.to_string())),
            }
        }

        Ok(Manifest {
            id: id.ok_or(ManifestError::MissingField("id"))?,
            name: name.ok_or(ManifestError::MissingField("name"))?,
            version: version.ok_or(ManifestError::MissingField("version"))?,
            kind: kind.ok_or(ManifestError::MissingField("kind"))?,
            entry: entry.ok_or(ManifestError::MissingField("entry"))?,
            target,
            capabilities,
            origins,
        })
    }
}

fn single(
    seen: &mut Vec<&'static str>,
    key: &'static str,
    line: usize,
) -> Result<(), ManifestError> {
    if seen.contains(&key) {
        return Err(ManifestError::DuplicateKey(line, key));
    }
    seen.push(key);
    Ok(())
}

fn validate_id(id: &str) -> Result<(), ManifestError> {
    let ok = id.len() >= 3
        && id.len() <= 128
        && id.contains('.')
        && id.chars().all(|c| {
            c.is_ascii_lowercase()
                || c.is_ascii_digit()
                || c == '.'
                || c == '-'
                || c == '_'
                || c == '@' // Firefox gecko id 常为 email 式，如 reader@example.org
        });
    if ok {
        Ok(())
    } else {
        Err(ManifestError::InvalidId(id.to_string()))
    }
}

fn validate_version(v: &str) -> Result<(), ManifestError> {
    let parts: Vec<&str> = v.split('.').collect();
    let ok = parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()));
    if ok {
        Ok(())
    } else {
        Err(ManifestError::InvalidVersion(v.to_string()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManifestError {
    MissingField(&'static str),
    DuplicateKey(usize, &'static str),
    UnknownKey(String),
    MalformedLine(usize, String),
    InvalidId(String),
    InvalidVersion(String),
    InvalidKind(String),
    InvalidCapability(String),
    InvalidTarget(String),
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManifestError::MissingField(k) => write!(f, "清单缺少必填字段: {k}"),
            ManifestError::DuplicateKey(l, k) => write!(f, "第 {l} 行重复字段: {k}"),
            ManifestError::UnknownKey(k) => write!(f, "未知字段: {k}"),
            ManifestError::MalformedLine(l, s) => write!(f, "第 {l} 行格式错误: {s}"),
            ManifestError::InvalidId(s) => write!(f, "非法插件 id（应为反向域名式小写标识）: {s}"),
            ManifestError::InvalidVersion(s) => write!(f, "非法版本号（应为 x.y.z）: {s}"),
            ManifestError::InvalidKind(s) => write!(f, "未知插件种类: {s}"),
            ManifestError::InvalidCapability(s) => write!(f, "未知能力: {s}"),
            ManifestError::InvalidTarget(s) => write!(f, "未知平台: {s}"),
        }
    }
}

impl std::error::Error for ManifestError {}

pub const EXAMPLE_ENGINE: &str = "\
# 渲染引擎也是一个插件
id: org.fenestra.engine.blink
name: Blink Engine Adapter
version: 0.1.0
kind: render-engine
entry: engines/blink.wasm
capability: engine.swap
capability: net.connect
";

pub const EXAMPLE_UI: &str = "\
id: org.fenestra.ui.minimal
name: Minimal Shell UI
version: 0.1.0
kind: shell-ui
entry: ui/minimal.wasm
capability: ui.chrome
capability: ui.content
";

pub const EXAMPLE_EXT: &str = "\
id: org.fenestra.ext.webextensions
name: WebExtensions Runtime
version: 0.1.0
kind: extension-runtime
entry: runtime/webext.wasm
capability: net.intercept
capability: storage.read
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_manifest() {
        let m = Manifest::parse(EXAMPLE_ENGINE).unwrap();
        assert_eq!(m.id, "org.fenestra.engine.blink");
        assert_eq!(m.kind, PluginKind::RenderEngine);
        assert_eq!(m.capabilities.len(), 2);
        assert!(m.capabilities.contains(&Capability::EngineSwap));
    }

    #[test]
    fn comments_and_duplicates_are_handled() {
        let text = "\
// 一行注释
# 另一行注释
id: com.example.x
name: X
version: 1.2.3
kind: theme
entry: x.wasm
capability: ui.chrome
capability: ui.chrome
origin: https://a.example/*
origin: https://a.example/*
";
        let m = Manifest::parse(text).unwrap();
        assert_eq!(m.capabilities, vec![Capability::UiChrome]);
        assert_eq!(m.origins.len(), 1);
    }

    #[test]
    fn rejects_duplicate_singleton_key() {
        let text =
            "id: com.example.x\nname: X\nname: Y\nversion: 1.0.0\nkind: theme\nentry: x.wasm\n";
        assert_eq!(
            Manifest::parse(text),
            Err(ManifestError::DuplicateKey(3, "name"))
        );
    }

    #[test]
    fn rejects_bad_id_version_kind_capability_and_unknown_key() {
        let base = "id: com.example.x\nname: X\nversion: 1.0.0\nkind: theme\nentry: x.wasm\n";
        assert!(matches!(
            Manifest::parse(&base.replace("com.example.x", "Bad_ID")),
            Err(ManifestError::InvalidId(_))
        ));
        assert!(matches!(
            Manifest::parse(&base.replace("1.0.0", "1.0")),
            Err(ManifestError::InvalidVersion(_))
        ));
        assert!(matches!(
            Manifest::parse(&base.replace("kind: theme", "kind: nope")),
            Err(ManifestError::InvalidKind(_))
        ));
        assert!(matches!(
            Manifest::parse(&format!("{base}capability: root.all\n")),
            Err(ManifestError::InvalidCapability(_))
        ));
        assert!(matches!(
            Manifest::parse(&format!("{base}mystery: 1\n")),
            Err(ManifestError::UnknownKey(_))
        ));
    }

    #[test]
    fn requires_all_mandatory_fields() {
        assert_eq!(
            Manifest::parse("id: com.example.x\n"),
            Err(ManifestError::MissingField("name"))
        );
    }
}
