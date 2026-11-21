//! # Fenestra Tauri 2 外壳（层 A）
//!
//! 渲染引擎被抽象成可替换接口层 [`RenderEngine`]：
//!   - MVP 用系统 WebView（wry，Tauri 2 默认），不打包引擎 —— 小体积、跨桌面 / iOS / Android；
//!   - Servo 降为 Stage 1 的「第一个可替换引擎插件」，实现同一 trait 后在注册表中接入；
//!   - 引擎是「注册」而非「编译期绑定」，运行时保留替换点。
//!
//! 安全说明：`gate_check` 不再用硬编码允许列表占位，而是真正调用 `fenestra_gate::Gate`
//! 的 `check_permission`：默认拒绝、声明 ∩ 策略、隐私逐项确认全部由同一套闸门裁决，
//! 且每次裁决（含拒绝原因）都进入有界审计日志，可由 `gate_audit` IPC 读取。
//! 闸门的 `Untrusted<T>` 出站重校验与可逆登记同样来自 `crates/gate`
//! （纯 Rust、零系统依赖，可独立 `cargo test`）。

use fenestra_gate::{validate_external_http_url, Capability, Gate};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

/// 可替换渲染引擎接口层。新增引擎（如 Servo）只需实现此 trait 并注册，
/// 外壳与插件层不感知具体引擎。
pub trait RenderEngine: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    /// 该引擎是否愿意处理某 URL；接口层据此挑选引擎（默认全部接受）。
    fn can_handle(&self, _url: &str) -> bool {
        true
    }
}

/// MVP 默认引擎：Tauri 2 的系统 WebView（wry）。
pub struct SystemWebView;

impl RenderEngine for SystemWebView {
    fn name(&self) -> &'static str {
        "system-webview (wry)"
    }

    fn description(&self) -> &'static str {
        "Tauri 2 默认系统 WebView，不打包渲染引擎"
    }
}

/// 引擎注册表。当前只装系统 WebView；Stage 1 在此注册 Servo 引擎插件。
pub struct EngineRegistry {
    engines: Vec<Box<dyn RenderEngine>>,
}

impl EngineRegistry {
    pub fn new() -> Self {
        Self {
            engines: Vec::new(),
        }
    }

    pub fn register<E>(&mut self, engine: E)
    where
        E: RenderEngine + 'static,
    {
        self.engines.push(Box::new(engine));
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.engines.iter().map(|e| e.name()).collect()
    }

    /// 按 URL 选出第一个可处理的引擎（引擎替换点）。
    pub fn engine_for(&self, url: &str) -> Option<&dyn RenderEngine> {
        self.engines
            .iter()
            .find(|e| e.can_handle(url))
            .map(|e| e.as_ref())
    }
}

impl Default for EngineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 外壳内唯一的闸门实例：进程内长期存活，策略 / 声明只在此初始化一次。
/// 用 `OnceLock` 而非 `static Mutex::new(...)`，因为初始化需要运行 `declare/set_policy`。
static GATE: OnceLock<Mutex<Gate>> = OnceLock::new();

fn gate() -> &'static Mutex<Gate> {
    GATE.get_or_init(|| {
        let mut g = Gate::new();
        // 策略上限只放开三个内置权限；其余能力即使被声明也不放行。
        g.set_policy(
            [
                Capability::UiNewTab,
                Capability::DnsResolve,
                Capability::NetOutbound,
            ]
            .into_iter()
            .collect(),
        );
        g.declare("newtab", [Capability::UiNewTab].into_iter().collect());
        g.declare(
            "accelerate",
            [Capability::DnsResolve, Capability::NetOutbound]
                .into_iter()
                .collect(),
        );
        Mutex::new(g)
    })
}

/// 统一权限检查 IPC：把前端传来的 (插件, 权限字符串) 交给真正的闸门裁决。
/// 未声明 / 不在策略上限 / 未逐项确认的，一律 `false`；每次裁决都记入审计。
#[tauri::command]
fn gate_check(plugin: String, capability: String) -> bool {
    let mut g = gate().lock().unwrap_or_else(|e| e.into_inner());
    g.check_permission(&plugin, &capability).is_allowed()
}

/// 失败可审计 IPC：返回最近的被拒绝裁决（插件 / 权限字符串 / 拒绝原因）。
/// 审计日志有界（见 `AUDIT_CAPACITY`），长期运行不会无界增长。
#[tauri::command]
fn gate_audit() -> Vec<String> {
    let g = gate().lock().unwrap_or_else(|e| e.into_inner());
    g.denials()
        .map(|r| {
            format!(
                "{}:{} -> {:?}",
                r.plugin,
                r.permission.as_deref().unwrap_or("<unknown>"),
                r.decision
            )
        })
        .collect()
}

/// 用系统浏览器打开外链。前端展示层不内嵌网页引擎，点击后走这里；
/// 地址先经闸门同款出站校验，拒绝 file:// / javascript: / 本机内网地址。
#[tauri::command]
fn open_external(url: String) -> Result<(), String> {
    let safe = validate_external_http_url(&url)
        .map_err(|e| e.message().to_string())?;

    #[cfg(target_os = "android")]
    let opened = Command::new("am")
        .args(["start", "-a", "android.intent.action.VIEW", "-d", &safe])
        .status();

    #[cfg(all(not(target_os = "android"), target_os = "windows"))]
    let opened = Command::new("cmd")
        .args(["/C", "start", "", &safe])
        .status();

    #[cfg(all(not(target_os = "android"), target_os = "macos"))]
    let opened = Command::new("open").arg(&safe).status();

    #[cfg(all(
        not(target_os = "android"),
        not(target_os = "windows"),
        not(target_os = "macos")
    ))]
    let opened = Command::new("xdg-open").arg(&safe).status();

    match opened {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("系统浏览器返回退出码 {}", s.code().unwrap_or(-1))),
        Err(e) => Err(format!("无法启动系统浏览器：{e}")),
    }
}

/// 返回当前注册的渲染引擎名（供前端 / 调试确认引擎替换是否生效）。
#[tauri::command]
fn engine_name() -> String {
    let mut registry = EngineRegistry::new();
    registry.register(SystemWebView);
    registry.names().join(", ")
}

/// 启动 Tauri 外壳。
/// `mobile` cfg 由 tauri-build 在 Android / iOS 构建时注入；
/// `mobile_entry_point` 生成 JNI 入口 `start_app` 符号（Tauri CLI 会检查），
/// 桌面构建下该属性为空操作，`main.rs` 仍可直接调用本函数。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            gate_check, gate_audit, engine_name, open_external
        ])
        .run(tauri::generate_context!())
        .expect("error while running Fenestra shell");
}
