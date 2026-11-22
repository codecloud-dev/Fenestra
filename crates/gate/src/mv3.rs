//! MV3 扩展清单兼容层（Stage 2 方向）。
//!
//! 把 Chrome / Edge / Firefox 的 Manifest V3 清单归一到 Fenestra 的能力模型：
//! 解析我们关心的字段、校验 MV3 硬约束、把 MV3 权限映射为 `Capability`。
//! 闸门保持零依赖，所以这里带一个只读取所需字段的极简 JSON 解析器。
//!
//! 设计口径：解析只认得的结构；未知字段忽略；任何歧义一律按"拒绝/报错"
//! 处理，绝不猜测放行——与闸门「默认拒绝」一致。

use std::collections::BTreeSet;

use crate::Capability;

/// 极简 JSON 值：足以读取 manifest 的字符串/字符串数组/对象字段。
#[derive(Clone, Debug, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self { Self { b: s.as_bytes(), i: 0 } }
    fn ws(&mut self) { while self.i < self.b.len() && self.b[self.i].is_ascii_whitespace() { self.i += 1; } }
    fn peek(&self) -> Option<u8> { self.b.get(self.i).copied() }
    fn eat(&mut self, c: u8) -> Result<(), Mv3Error> {
        self.ws();
        if self.peek() == Some(c) { self.i += 1; Ok(()) }
        else { Err(Mv3Error::Malformed(format!("expected '{}' at byte {}", c as char, self.i))) }
    }
    fn parse(mut self) -> Result<Json, Mv3Error> {
        let v = self.value()?;
        self.ws();
        if self.i != self.b.len() { return Err(Mv3Error::Malformed("trailing data".into())); }
        Ok(v)
    }
    fn value(&mut self) -> Result<Json, Mv3Error> {
        self.ws();
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b't') | Some(b'f') => self.boolean(),
            Some(b'n') => { self.literal(b"null")?; Ok(Json::Null) }
            Some(c) if c.is_ascii_digit() || c == b'-' => self.number(),
            _ => Err(Mv3Error::Malformed(format!("unexpected byte at {}", self.i))),
        }
    }
    fn literal(&mut self, lit: &[u8]) -> Result<(), Mv3Error> {
        if self.b.get(self.i..self.i + lit.len()) == Some(lit) { self.i += lit.len(); Ok(()) }
        else { Err(Mv3Error::Malformed("bad literal".into())) }
    }
    fn boolean(&mut self) -> Result<Json, Mv3Error> {
        if self.peek() == Some(b't') { self.literal(b"true")?; Ok(Json::Bool(true)) }
        else { self.literal(b"false")?; Ok(Json::Bool(false)) }
    }
    fn number(&mut self) -> Result<Json, Mv3Error> {
        let start = self.i;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || matches!(c, b'-' | b'+' | b'.' | b'e' | b'E') { self.i += 1; }
            else { break; }
        }
        let s = std::str::from_utf8(&self.b[start..self.i]).map_err(|_| Mv3Error::Malformed("bad number".into()))?;
        s.parse::<f64>().map(Json::Num).map_err(|_| Mv3Error::Malformed("bad number".into()))
    }
    fn string(&mut self) -> Result<String, Mv3Error> {
        self.eat(b'"')?;
        let mut out = String::new();
        while self.i < self.b.len() {
            let c = self.b[self.i];
            self.i += 1;
            match c {
                b'"' => return Ok(out),
                b'\\' => {
                    let e = *self.b.get(self.i).ok_or_else(|| Mv3Error::Malformed("bad escape".into()))?;
                    self.i += 1;
                    match e {
                        b'"' => out.push('"'), b'\\' => out.push('\\'), b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'), b'f' => out.push('\u{c}'), b'n' => out.push('\n'),
                        b'r' => out.push('\r'), b't' => out.push('\t'),
                        b'u' => {
                            let h = self.b.get(self.i..self.i + 4)
                                .and_then(|s| std::str::from_utf8(s).ok())
                                .and_then(|s| u16::from_str_radix(s, 16).ok())
                                .ok_or_else(|| Mv3Error::Malformed("bad unicode escape".into()))?;
                            self.i += 4;
                            if let Some(ch) = char::from_u32(h as u32) { out.push(ch); }
                        }
                        _ => return Err(Mv3Error::Malformed("bad escape".into())),
                    }
                }
                _ => {
                    // 逐字节复制；manifest 文本按 UTF-8 读取，非 ASCII 走原字节回填。
                    let s = std::str::from_utf8(&self.b[self.i - 1..self.i])
                        .map_err(|_| Mv3Error::Malformed("bad utf8".into()))?;
                    out.push_str(s);
                }
            }
        }
        Err(Mv3Error::Malformed("unterminated string".into()))
    }
    fn array(&mut self) -> Result<Json, Mv3Error> {
        self.eat(b'[')?;
        let mut out = Vec::new();
        self.ws();
        if self.peek() == Some(b']') { self.i += 1; return Ok(Json::Arr(out)); }
        loop {
            out.push(self.value()?);
            self.ws();
            match self.peek() {
                Some(b',') => { self.i += 1; }
                Some(b']') => { self.i += 1; return Ok(Json::Arr(out)); }
                _ => return Err(Mv3Error::Malformed("bad array".into())),
            }
        }
    }
    fn object(&mut self) -> Result<Json, Mv3Error> {
        self.eat(b'{')?;
        let mut out = Vec::new();
        self.ws();
        if self.peek() == Some(b'}') { self.i += 1; return Ok(Json::Obj(out)); }
        loop {
            self.ws();
            let k = self.string()?;
            self.eat(b':')?;
            let v = self.value()?;
            out.push((k, v));
            self.ws();
            match self.peek() {
                Some(b',') => { self.i += 1; }
                Some(b'}') => { self.i += 1; return Ok(Json::Obj(out)); }
                _ => return Err(Mv3Error::Malformed("bad object".into())),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mv3Error {
    Malformed(String),
    NotObject,
    MissingField(&'static str),
    WrongType(&'static str),
}

impl std::fmt::Display for Mv3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mv3Error::Malformed(s) => write!(f, "malformed MV3 manifest: {}", s),
            Mv3Error::NotObject => write!(f, "MV3 manifest root is not an object"),
            Mv3Error::MissingField(k) => write!(f, "MV3 manifest missing field: {}", k),
            Mv3Error::WrongType(k) => write!(f, "MV3 manifest field has wrong type: {}", k),
        }
    }
}

impl std::error::Error for Mv3Error {}

/// 归一后的 MV3 清单（只保留闸门关心的字段）。
#[derive(Debug, Clone, PartialEq)]
pub struct Mv3Manifest {
    pub name: String,
    pub version: String,
    pub manifest_version: i64,
    pub permissions: Vec<String>,
    pub host_permissions: Vec<String>,
    /// background.service_worker 的文件名（MV3 必须是字符串，且不能是持久页）。
    pub service_worker: Option<String>,
    /// action（MV3 统一了 browser_action / page_action）。
    pub has_action: bool,
    /// 是否声明了 web_accessible_resources（兼容层需记录但不自动放行）。
    pub web_accessible_resources: bool,
    /// 是否存在 code 注入式远程脚本字段（MV3 禁止）。
    pub has_remote_script: bool,
}

fn obj_get<'a>(obj: &'a [(String, Json)], key: &str) -> Option<&'a Json> {
    obj.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}
fn as_str(v: &Json) -> Option<&str> { if let Json::Str(s) = v { Some(s) } else { None } }
fn as_str_vec(v: &Json) -> Option<Vec<String>> {
    if let Json::Arr(a) = v {
        if a.iter().all(|x| matches!(x, Json::Str(_))) {
            return Some(a.iter().filter_map(|x| as_str(x).map(str::to_owned)).collect());
        }
    }
    None
}

impl Mv3Manifest {
    /// 从一个 MV3 JSON 文本解析出归一结构。
    pub fn parse(raw: &str) -> Result<Self, Mv3Error> {
        let root = Parser::new(raw).parse()?;
        let obj = match root { Json::Obj(o) => o, _ => return Err(Mv3Error::NotObject) };

        let name = obj_get(&obj, "name").and_then(as_str)
            .ok_or(Mv3Error::MissingField("name"))?.to_owned();
        let version = obj_get(&obj, "version").and_then(as_str)
            .ok_or(Mv3Error::MissingField("version"))?.to_owned();
        let manifest_version = match obj_get(&obj, "manifest_version") {
            Some(Json::Num(n)) => *n as i64,
            _ => return Err(Mv3Error::WrongType("manifest_version")),
        };
        let permissions = match obj_get(&obj, "permissions") {
            Some(v) => as_str_vec(v).ok_or(Mv3Error::WrongType("permissions"))?,
            None => Vec::new(),
        };
        let host_permissions = match obj_get(&obj, "host_permissions") {
            Some(v) => as_str_vec(v).ok_or(Mv3Error::WrongType("host_permissions"))?,
            None => Vec::new(),
        };
        let service_worker = match obj_get(&obj, "background") {
            Some(Json::Obj(bg)) => obj_get(bg, "service_worker").and_then(as_str).map(str::to_owned),
            _ => None,
        };
        let has_action = matches!(obj_get(&obj, "action"), Some(Json::Obj(_)));
        let web_accessible_resources = matches!(obj_get(&obj, "web_accessible_resources"), Some(Json::Arr(_)));
        let has_remote_script = permissions.iter().any(|p| p == "scripting")
            && matches!(obj_get(&obj, "content_security_policy"), Some(Json::Obj(_)));

        Ok(Self {
            name, version, manifest_version, permissions, host_permissions,
            service_worker, has_action, web_accessible_resources, has_remote_script,
        })
    }

    /// MV3 硬约束校验；返回问题清单，空表示合规。
    pub fn validate(&self) -> Vec<Mv3Issue> {
        let mut issues = Vec::new();
        if self.manifest_version != 3 {
            issues.push(Mv3Issue::Error(format!(
                "manifest_version 必须为 3，实际为 {}", self.manifest_version
            )));
        }
        // MV2 的 background.scripts / persistent 背景页在 MV3 里非法：
        // 若存在 background 但拿不到 service_worker，视为可疑。
        if self.permissions.iter().any(|p| p == "webRequestBlocking") {
            issues.push(Mv3Issue::Error(
                "webRequestBlocking 在 MV3 被禁用，请改用 declarativeNetRequest".into(),
            ));
        }
        for p in &self.permissions {
            if p.starts_with("http://") || p.starts_with("https://") {
                issues.push(Mv3Issue::Error(format!(
                    "主机权限必须写在 host_permissions，不能放 permissions: {}", p
                )));
            }
        }
        for h in &self.host_permissions {
            if !h.starts_with("http://") && !h.starts_with("https://") && h != "<all_urls>" {
                issues.push(Mv3Issue::Warning(format!("host_permissions 条目不是标准 URL 模式: {}", h)));
            }
        }
        if self.has_remote_script {
            issues.push(Mv3Issue::Warning(
                "检测到 scripting + content_security_policy，远程代码在 MV3 被禁止".into(),
            ));
        }
        issues
    }

    /// 把 MV3 权限 + 主机权限映射为 Fenestra 能力集（默认拒绝、未知忽略）。
    pub fn to_capabilities(&self) -> BTreeSet<Capability> {
        let mut caps = BTreeSet::new();
        for p in &self.permissions {
            if let Some(c) = map_permission(p) { caps.insert(c); }
        }
        if !self.host_permissions.is_empty() {
            // 任何主机访问都意味着出网能力。
            caps.insert(Capability::NetOutbound);
        }
        caps
    }

    /// 转成 FEP 插件权限字符串（与 `Capability::from_permission` 对齐），供
    /// 兼容层把 MV3 扩展登记进 Fenestra 运行时。
    pub fn to_fep_permissions(&self) -> Vec<String> {
        let mut out = BTreeSet::new();
        for c in self.to_capabilities() {
            out.insert(capability_to_permission(c).to_owned());
        }
        out.into_iter().collect()
    }
}

/// MV3 权限字符串 → Fenestra 能力。未知返回 None（默认拒绝）。
fn map_permission(p: &str) -> Option<Capability> {
    match p.trim() {
        "activeTab" | "tabs" => Some(Capability::UiNewTab),
        "storage" => Some(Capability::StorageWrite),
        "declarativeNetRequest" | "webRequest" | "proxy" => Some(Capability::NetOutbound),
        "dns" => Some(Capability::DnsResolve),
        "fileSystem" | "fileSystem.write" => Some(Capability::FileRead),
        "clipboardRead" => Some(Capability::ClipboardRead),
        "cookies" | "identity" => Some(Capability::ProfileRead),
        "geolocation" => Some(Capability::LocationDevice),
        "telemetryProbe" => Some(Capability::Telemetry),
        // 其余（scripting、notifications、alarms 等）当前无对应能力，默认拒绝。
        _ => Capability::from_permission(p),
    }
}

fn capability_to_permission(c: Capability) -> &'static str {
    match c {
        Capability::DnsResolve => "net.dns",
        Capability::NetOutbound => "net.outbound",
        Capability::StorageWrite => "storage.write",
        Capability::FileRead => "file.read",
        Capability::UiNewTab => "ui.newtab",
        Capability::ClipboardRead => "clipboard.read",
        Capability::ProfileRead => "profile.read",
        Capability::CameraDevice => "camera",
        Capability::LocationDevice => "location",
        Capability::Telemetry => "telemetry",
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mv3Issue {
    Error(String),
    Warning(String),
}

impl Mv3Issue {
    pub fn is_error(&self) -> bool { matches!(self, Mv3Issue::Error(_)) }
}

impl std::fmt::Display for Mv3Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mv3Issue::Error(s) => write!(f, "error: {}", s),
            Mv3Issue::Warning(s) => write!(f, "warning: {}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHROME: &str = r#"{
      "manifest_version": 3,
      "name": "Reader",
      "version": "2.1",
      "permissions": ["storage", "activeTab", "declarativeNetRequest", "telemetryProbe"],
      "host_permissions": ["https://example.com/*"],
      "background": { "service_worker": "background.js" },
      "action": { "default_popup": "popup.html" }
    }"#;

    #[test]
    fn parse_chrome_mv3() {
        let m = Mv3Manifest::parse(CHROME).unwrap();
        assert_eq!(m.name, "Reader");
        assert_eq!(m.version, "2.1");
        assert_eq!(m.manifest_version, 3);
        assert_eq!(m.service_worker.as_deref(), Some("background.js"));
        assert!(m.has_action);
        assert!(m.validate().is_empty());
    }

    #[test]
    fn maps_capabilities_default_deny() {
        let m = Mv3Manifest::parse(CHROME).unwrap();
        let caps = m.to_capabilities();
        assert!(caps.contains(&Capability::StorageWrite));
        assert!(caps.contains(&Capability::UiNewTab));
        assert!(caps.contains(&Capability::NetOutbound));
        assert!(caps.contains(&Capability::Telemetry)); // 隐私红线也会映射，但需用户确认
        // declarativeNetRequest 与 host_permissions 都指向 NetOutbound，不重复。
        assert_eq!(caps.iter().filter(|c| **c == Capability::NetOutbound).count(), 1);
        let fep = m.to_fep_permissions();
        assert!(fep.contains(&"telemetry".to_string()));
        assert!(fep.contains(&"storage.write".to_string()));
    }

    #[test]
    fn rejects_mv2_and_blocking_webreq() {
        let bad = r#"{
          "manifest_version": 2,
          "name": "Legacy",
          "version": "1.0",
          "permissions": ["webRequestBlocking", "https://a.com/*"]
        }"#;
        let m = Mv3Manifest::parse(bad).unwrap();
        let issues = m.validate();
        assert!(issues.iter().any(|i| i.is_error() && i.to_string().contains("manifest_version")));
        assert!(issues.iter().any(|i| i.is_error() && i.to_string().contains("webRequestBlocking")));
        assert!(issues.iter().any(|i| i.is_error() && i.to_string().contains("host_permissions")));
    }

    #[test]
    fn unknown_permission_is_ignored() {
        let m = Mv3Manifest::parse(r#"{
          "manifest_version": 3, "name": "X", "version": "1",
          "permissions": ["someFutureApi", "notifications"]
        }"#).unwrap();
        assert!(m.to_capabilities().is_empty());
    }

    #[test]
    fn malformed_json_errors() {
        assert!(Mv3Manifest::parse("{not json}").is_err());
        assert!(Mv3Manifest::parse("[]").is_err());
        assert!(Mv3Manifest::parse(r#"{"name":"x"}"#).is_err());
    }
}
