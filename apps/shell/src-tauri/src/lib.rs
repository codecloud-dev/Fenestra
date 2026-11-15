//! # Fenestra Tauri 2 外壳（层 A）
//!
//! 渲染引擎被抽象成可替换接口层 [`RenderEngine`]：
//!   - MVP 用系统 WebView（wry，Tauri 2 默认），不打包引擎 —— 小体积、跨桌面 / iOS / Android；
//!   - Servo 降为 Stage 1 的「第一个可替换引擎插件」，实现同一 trait 后在注册表中接入；
//!   - 引擎是「注册」而非「编译期绑定」，运行时保留替换点。
//!
//! 安全说明：`gate_check` 是「Rust 闸门 ↔ 前端」IPC 的对接示意，遵循默认拒绝。
//! 真正的默认拒绝、最小权限交集、隐私逐项确认、`Untrusted<T>` 出站重校验与
//! 可逆登记，由 `crates/gate` 实现（纯 Rust、零系统依赖，可独立 `cargo test`）。

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

/// 默认拒绝示例：未显式声明的 (插件, 能力) 一律返回 `false`。
/// 生产版把这里替换为对 `fenestra_gate::Gate::check` 的调用。
#[tauri::command]
fn gate_check(plugin: String, capability: String) -> bool {
    matches!(
        (plugin.as_str(), capability.as_str()),
        ("newtab", "ui.newtab")
            | ("accelerate", "net.dns")
            | ("accelerate", "net.preconnect")
    )
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
        .invoke_handler(tauri::generate_handler![gate_check, engine_name])
        .run(tauri::generate_context!())
        .expect("error while running Fenestra shell");
}
