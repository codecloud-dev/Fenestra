//! 三类扩展的兼容层（只读归一，不执行扩展代码）。
//!
//! 目标：Chrome / Edge / Firefox 的 `manifest.json` 都能被读成同一份
//! FEP 清单，挂进同一个能力仲裁器。差异不被悄悄抹平，而是变成
//! `warnings` 明确列出来——"完全兼容"不等于"假装没有差异"。
//!
//! 本文件自带一个极小的 JSON 解析器，只依赖 `std`。
//! 内核启动早于任何 JS 运行时，不能指望宿主提供 JSON.parse。

use std::fmt;

use crate::capability::Capability;
use crate::engine::ExtensionFlavor;
use crate::manifest::{Manifest, PluginKind};

// ---------------------------------------------------------------------------
// 迷你 JSON
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Num(n) => Some(*n),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonError {
    Eof,
    Unexpected(usize, char),
    Trailing(usize),
    BadString(usize),
    BadNumber(usize),
    BadEscape(usize),
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonError::Eof => f.write_str("JSON 意外结束"),
            JsonError::Unexpected(p, c) => write!(f, "位置 {p} 出现非法字符 {c:?}"),
            JsonError::Trailing(p) => write!(f, "位置 {p} 之后存在多余内容"),
            JsonError::BadString(p) => write!(f, "位置 {p} 字符串未闭合"),
            JsonError::BadNumber(p) => write!(f, "位置 {p} 数字格式错误"),
            JsonError::BadEscape(p) => write!(f, "位置 {p} 转义序列错误"),
        }
    }
}

impl std::error::Error for JsonError {}

pub fn parse_json(input: &str) -> Result<Json, JsonError> {
    let mut p = Parser {
        bytes: input.as_bytes(),
        pos: 0,
    };
    let value = p.value()?;
    p.ws();
    if p.pos != p.bytes.len() {
        return Err(JsonError::Trailing(p.pos));
    }
    Ok(value)
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn value(&mut self) -> Result<Json, JsonError> {
        self.ws();
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b't') => {
                self.literal(b"true")?;
                Ok(Json::Bool(true))
            }
            Some(b'f') => {
                self.literal(b"false")?;
                Ok(Json::Bool(false))
            }
            Some(b'n') => {
                self.literal(b"null")?;
                Ok(Json::Null)
            }
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(c) => Err(JsonError::Unexpected(self.pos, c as char)),
            None => Err(JsonError::Eof),
        }
    }

    fn literal(&mut self, word: &[u8]) -> Result<(), JsonError> {
        if self.bytes.get(self.pos..self.pos + word.len()) == Some(word) {
            self.pos += word.len();
            Ok(())
        } else {
            Err(JsonError::Unexpected(
                self.pos,
                self.peek().unwrap_or(b'?') as char,
            ))
        }
    }

    fn object(&mut self) -> Result<Json, JsonError> {
        self.pos += 1; // {
        let mut out = Vec::new();
        self.ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Json::Obj(out));
        }
        loop {
            self.ws();
            if self.peek() != Some(b'"') {
                return Err(JsonError::Unexpected(
                    self.pos,
                    self.peek().unwrap_or(b'?') as char,
                ));
            }
            let key = self.string()?;
            self.ws();
            if self.peek() != Some(b':') {
                return Err(JsonError::Unexpected(
                    self.pos,
                    self.peek().unwrap_or(b'?') as char,
                ));
            }
            self.pos += 1;
            let value = self.value()?;
            out.push((key, value));
            self.ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                Some(c) => return Err(JsonError::Unexpected(self.pos, c as char)),
                None => return Err(JsonError::Eof),
            }
        }
        Ok(Json::Obj(out))
    }

    fn array(&mut self) -> Result<Json, JsonError> {
        self.pos += 1; // [
        let mut out = Vec::new();
        self.ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Json::Arr(out));
        }
        loop {
            out.push(self.value()?);
            self.ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    break;
                }
                Some(c) => return Err(JsonError::Unexpected(self.pos, c as char)),
                None => return Err(JsonError::Eof),
            }
        }
        Ok(Json::Arr(out))
    }

    fn string(&mut self) -> Result<String, JsonError> {
        if self.peek() != Some(b'"') {
            return Err(JsonError::BadString(self.pos));
        }
        self.pos += 1;
        let mut out = String::new();
        loop {
            let c = self.peek().ok_or(JsonError::BadString(self.pos))?;
            self.pos += 1;
            match c {
                b'"' => return Ok(out),
                b'\\' => {
                    let esc = self.peek().ok_or(JsonError::BadEscape(self.pos))?;
                    self.pos += 1;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let start = self.pos;
                            let hex = self
                                .bytes
                                .get(self.pos..self.pos + 4)
                                .ok_or(JsonError::BadEscape(start))?;
                            let text = std::str::from_utf8(hex)
                                .map_err(|_| JsonError::BadEscape(start))?;
                            let code = u32::from_str_radix(text, 16)
                                .map_err(|_| JsonError::BadEscape(start))?;
                            out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                            self.pos += 4;
                        }
                        _ => return Err(JsonError::BadEscape(self.pos - 1)),
                    }
                }
                0..=0x1f => return Err(JsonError::BadString(self.pos - 1)),
                _ => {
                    // 非 ASCII 字节按 UTF-8 字符整体读回。
                    if c < 0x80 {
                        out.push(c as char);
                    } else {
                        let start = self.pos - 1;
                        let len = utf8_len(c);
                        let slice = self
                            .bytes
                            .get(start..start + len)
                            .ok_or(JsonError::BadString(start))?;
                        let s =
                            std::str::from_utf8(slice).map_err(|_| JsonError::BadString(start))?;
                        out.push_str(s);
                        self.pos = start + len;
                    }
                }
            }
        }
    }

    fn number(&mut self) -> Result<Json, JsonError> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || matches!(c, b'-' | b'+' | b'.' | b'e' | b'E') {
                self.pos += 1;
            } else {
                break;
            }
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| JsonError::BadNumber(start))?;
        let n = text
            .parse::<f64>()
            .map_err(|_| JsonError::BadNumber(start))?;
        Ok(Json::Num(n))
    }
}

fn utf8_len(first: u8) -> usize {
    match first {
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf7 => 4,
        _ => 1,
    }
}

// ---------------------------------------------------------------------------
// manifest.json → FEP
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct CompatReport {
    pub flavor: ExtensionFlavor,
    pub manifest_version: u8,
    pub entry: String,
    pub manifest: Manifest,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompatError {
    Json(JsonError),
    Missing(&'static str),
}

impl fmt::Display for CompatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompatError::Json(e) => write!(f, "{e}"),
            CompatError::Missing(k) => write!(f, "manifest.json 缺少字段: {k}"),
        }
    }
}

impl std::error::Error for CompatError {}

impl From<JsonError> for CompatError {
    fn from(e: JsonError) -> Self {
        CompatError::Json(e)
    }
}

/// 把一份浏览器扩展清单归一为 FEP 清单。
pub fn from_json_manifest(text: &str) -> Result<CompatReport, CompatError> {
    let root = parse_json(text)?;
    let name = root
        .get("name")
        .and_then(Json::as_str)
        .ok_or(CompatError::Missing("name"))?
        .to_string();
    let version_raw = root
        .get("version")
        .and_then(Json::as_str)
        .ok_or(CompatError::Missing("version"))?;
    let manifest_version = root
        .get("manifest_version")
        .and_then(Json::as_f64)
        .unwrap_or(3.0) as u8;

    let gecko = root
        .get("browser_specific_settings")
        .and_then(|o| o.get("gecko"))
        .is_some()
        || root
            .get("applications")
            .and_then(|o| o.get("gecko"))
            .is_some();
    let edge_hint = root
        .get("update_url")
        .and_then(Json::as_str)
        .map(|s| s.contains("edge.microsoft.com"))
        .unwrap_or(false);
    let flavor = if gecko {
        ExtensionFlavor::Firefox
    } else if edge_hint {
        ExtensionFlavor::Edge
    } else {
        ExtensionFlavor::Chrome
    };

    let gecko_id = root
        .get("browser_specific_settings")
        .and_then(|o| o.get("gecko"))
        .and_then(|o| o.get("id"))
        .and_then(Json::as_str);
    let id = gecko_id
        .map(str::to_string)
        .unwrap_or_else(|| format!("{}.ext", slug(&name)));

    let mut capabilities: Vec<Capability> = Vec::new();
    let mut origins: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    for field in ["permissions", "optional_permissions", "host_permissions"] {
        if let Some(items) = root.get(field).and_then(Json::as_array) {
            for item in items {
                let Some(p) = item.as_str() else { continue };
                if p.contains("://") || p == "<all_urls>" {
                    if !origins.iter().any(|o| o == p) {
                        origins.push(p.to_string());
                    }
                    continue;
                }
                match map_permission(p) {
                    Some(caps) => {
                        for cap in caps {
                            if !capabilities.contains(&cap) {
                                capabilities.push(cap);
                            }
                        }
                    }
                    None => warnings.push(format!(
                        "未映射的权限 `{p}`：按默认拒绝处理，不授予任何能力"
                    )),
                }
            }
        }
    }

    let mut entry = String::new();
    if let Some(bg) = root.get("background") {
        if let Some(sw) = bg.get("service_worker").and_then(Json::as_str) {
            entry = sw.to_string();
        } else if let Some(scripts) = bg.get("scripts").and_then(Json::as_array) {
            if let Some(first) = scripts.first().and_then(Json::as_str) {
                entry = first.to_string();
            }
        }
    }
    if entry.is_empty() {
        entry = "background.js".to_string();
        warnings.push("清单未声明 background，归一器回退到 background.js".to_string());
    }

    match flavor {
        ExtensionFlavor::Firefox => {
            if root
                .get("background")
                .and_then(|b| b.get("service_worker"))
                .is_some()
            {
                warnings.push("Firefox MV3 更稳妥的是 background.scripts 事件页；service_worker 可能导致后台状态被回收".to_string());
            }
            if root
                .get("browser_specific_settings")
                .and_then(|o| o.get("gecko"))
                .and_then(|o| o.get("data_collection_permissions"))
                .is_none()
            {
                warnings.push("AMO 要求声明 data_collection_permissions（strict_min_version ≥ 142），缺失会导致 web-ext lint 失败".to_string());
            }
            warnings.push("Firefox 仍允许部分 webRequest 动态拦截，而 Chrome/Edge 只提供 declarativeNetRequest（上限约 150K/300K 条规则）".to_string());
        }
        ExtensionFlavor::Chrome | ExtensionFlavor::Edge => {
            if capabilities.contains(&Capability::NetIntercept) {
                warnings.push("阻塞式 webRequest 在 MV3 已由 declarativeNetRequest 静态规则取代；运行时拦截会被降级".to_string());
            }
        }
        ExtensionFlavor::Fenestra => {}
    }

    if manifest_version < 3 {
        warnings.push(format!(
            "manifest_version={manifest_version} 属旧方言，建议迁移到 MV3"
        ));
    }
    warnings.push("命名空间差异：Firefox 用 browser.*/Promise，Chromium 系自 2026 年中才全量支持 browser.*；Fenestra 兼容层统一暴露 Promise 风格".to_string());
    if matches!(flavor, ExtensionFlavor::Edge) {
        warnings.push("Edge 与 Chrome 共用同一 MV3 包，差异主要在商店审核与 update_url；归一后按同一能力集处理".to_string());
    }

    capabilities.sort();
    capabilities.dedup();

    let manifest = Manifest {
        id,
        name,
        version: normalize_version(version_raw),
        kind: PluginKind::ExtensionRuntime,
        entry: entry.clone(),
        target: None,
        capabilities,
        origins,
    };

    Ok(CompatReport {
        flavor,
        manifest_version,
        entry,
        manifest,
        warnings,
    })
}

fn map_permission(p: &str) -> Option<Vec<Capability>> {
    let caps = match p {
        "storage" | "unlimitedStorage" | "bookmarks" => {
            vec![Capability::StorageRead, Capability::StorageWrite]
        }
        "tabs" | "activeTab" | "scripting" => vec![Capability::UiContent],
        "clipboardRead" => vec![Capability::ClipboardRead],
        "clipboardWrite" => vec![Capability::ClipboardWrite],
        "webRequest"
        | "webRequestBlocking"
        | "declarativeNetRequest"
        | "declarativeNetRequestWithHostAccess" => {
            vec![Capability::NetIntercept]
        }
        "downloads" => vec![Capability::StorageWrite],
        "history" | "cookies" => vec![Capability::ProfileRead],
        "geolocation" => vec![Capability::DeviceLocation],
        "notifications" | "alarms" | "contextMenus" | "sidePanel" => vec![Capability::UiChrome],
        "background" | "offscreen" => vec![],
        _ => return None,
    };
    Some(caps)
}

fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "extension".to_string()
    } else {
        trimmed
    }
}

fn normalize_version(v: &str) -> String {
    let parts: Vec<&str> = v.split('.').collect();
    let mut nums: Vec<String> = parts
        .iter()
        .take(3)
        .map(|p| {
            p.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
        })
        .collect();
    while nums.len() < 3 {
        nums.push("0".to_string());
    }
    nums.join(".")
}

/// 把归一后的 FEP 清单写回纯文本，保证兼容层双向可审计：
/// 任何从三家扩展转来的插件，都能被人直接读懂、手工修改。
pub fn to_hep(m: &Manifest) -> String {
    let mut s = format!(
        "# 由 {} 扩展 manifest.json 归一\nid: {}\nname: {}\nversion: {}\nkind: extension-runtime\nentry: {}\n",
        "Chrome/Edge/Firefox", m.id, m.name, m.version, m.entry
    );
    for c in &m.capabilities {
        s.push_str(&format!("capability: {}\n", c.as_str()));
    }
    for o in &m.origins {
        s.push_str(&format!("origin: {o}\n"));
    }
    s
}

pub const CHROME_MV3: &str = r#"{
  "manifest_version": 3,
  "name": "Reader",
  "version": "2.1",
  "permissions": ["storage", "activeTab", "declarativeNetRequest", "telemetryProbe"],
  "host_permissions": ["https://example.com/*"],
  "background": { "service_worker": "background.js" }
}"#;

pub const EDGE_MV3: &str = r#"{
  "manifest_version": 3,
  "name": "Reader for Edge",
  "version": "2.1.0",
  "update_url": "https://edge.microsoft.com/extensionwebstorebase/v1/crx",
  "permissions": ["storage", "tabs"],
  "background": { "service_worker": "background.js" }
}"#;

pub const FIREFOX_MV3: &str = r#"{
  "manifest_version": 3,
  "name": "Reader for Firefox",
  "version": "2.1.0",
  "browser_specific_settings": {
    "gecko": {
      "id": "reader@example.org",
      "strict_min_version": "142.0",
      "data_collection_permissions": { "required": ["none"] }
    }
  },
  "permissions": ["storage", "webRequest", "bookmarks"],
  "background": { "scripts": ["background.js"] }
}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mini_json_handles_nested_values_and_escapes() {
        let v = parse_json(r#"{"a":[1,true,null,{"b":"x\"y\n"}],"n":-1.5e2}"#).unwrap();
        assert_eq!(v.get("a").and_then(Json::as_array).unwrap().len(), 4);
        assert_eq!(
            v.get("a")
                .and_then(Json::as_array)
                .and_then(|a| a.get(3))
                .and_then(|o| o.get("b"))
                .and_then(Json::as_str),
            Some("x\"y\n")
        );
        assert_eq!(v.get("n").and_then(Json::as_f64), Some(-150.0));
        assert!(parse_json("{\"a\":1} tail").is_err());
        assert!(parse_json("{\"a\":}").is_err());
    }

    #[test]
    fn chrome_manifest_normalizes_with_warnings() {
        let r = from_json_manifest(CHROME_MV3).unwrap();
        assert_eq!(r.flavor, ExtensionFlavor::Chrome);
        assert_eq!(r.manifest_version, 3);
        assert_eq!(r.entry, "background.js");
        assert_eq!(r.manifest.version, "2.1.0");
        assert!(r.manifest.capabilities.contains(&Capability::StorageRead));
        assert!(r.manifest.capabilities.contains(&Capability::NetIntercept));
        assert_eq!(r.manifest.origins, vec!["https://example.com/*"]);
        assert!(r.warnings.iter().any(|w| w.contains("未映射的权限")));
        assert!(r
            .warnings
            .iter()
            .any(|w| w.contains("declarativeNetRequest")));
    }

    #[test]
    fn edge_is_detected_by_update_url() {
        let r = from_json_manifest(EDGE_MV3).unwrap();
        assert_eq!(r.flavor, ExtensionFlavor::Edge);
        assert!(r.warnings.iter().any(|w| w.contains("Edge 与 Chrome")));
    }

    #[test]
    fn firefox_is_detected_by_gecko_settings() {
        let r = from_json_manifest(FIREFOX_MV3).unwrap();
        assert_eq!(r.flavor, ExtensionFlavor::Firefox);
        assert_eq!(r.manifest.id, "reader@example.org");
        assert_eq!(r.entry, "background.js");
        assert!(!r.warnings.iter().any(|w| w.contains("AMO 要求声明")));
        assert!(r.warnings.iter().any(|w| w.contains("webRequest")));
    }

    #[test]
    fn all_three_flavors_produce_installable_hep_manifests() {
        for text in [CHROME_MV3, EDGE_MV3, FIREFOX_MV3] {
            let r = from_json_manifest(text).unwrap();
            let hep = to_hep(&r.manifest);
            let parsed = crate::manifest::Manifest::parse(&hep).unwrap();
            assert_eq!(parsed.id, r.manifest.id);
        }
    }

    #[test]
    fn malformed_json_is_rejected_cleanly() {
        assert!(from_json_manifest("not json").is_err());
        assert_eq!(from_json_manifest("{}"), Err(CompatError::Missing("name")));
    }
}
