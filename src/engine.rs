//! 可插拔边界：内核只定义 trait，不提供任何实现。
//!
//! "我们的浏览器只是个壳"——这句话在代码里的含义就是本文件：
//! 这里没有渲染、没有网络、没有窗口，只有一组任何人都可以实现的接口。
//! 换引擎、换网络栈、换 UI，都是换一个插件，不是 fork 内核。

use std::fmt;

/// 目标平台。内核本身平台无关，由插件声明自己服务哪个平台。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    Android,
    Ios,
    Windows,
    Linux,
    MacOs,
}

impl Target {
    pub const SUPPORTED: [Target; 5] = [
        Target::Android,
        Target::Ios,
        Target::Windows,
        Target::Linux,
        Target::MacOs,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Target::Android => "android",
            Target::Ios => "ios",
            Target::Windows => "windows",
            Target::Linux => "linux",
            Target::MacOs => "macos",
        }
    }

    pub fn parse(s: &str) -> Option<Target> {
        Target::SUPPORTED.into_iter().find(|t| t.as_str() == s)
    }

    pub fn rust_target_triple(self) -> &'static str {
        match self {
            Target::Android => "aarch64-linux-android",
            Target::Ios => "aarch64-apple-ios",
            Target::Windows => "x86_64-pc-windows-msvc",
            Target::Linux => "x86_64-unknown-linux-gnu",
            Target::MacOs => "aarch64-apple-darwin",
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineError(pub String);

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "插件错误: {}", self.0)
    }
}

impl std::error::Error for EngineError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderRequest {
    pub url: String,
    pub origin: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub engine: String,
    pub bytes: usize,
}

/// 渲染引擎接口。Blink / WebKit / Gecko / Servo / Ladybird / 自研引擎
/// 只要实现它就能挂进 Hollo，彼此可以并列安装、按站点或按用户切换。
pub trait RenderEngine {
    fn name(&self) -> &str;
    fn render(&self, req: &RenderRequest) -> Result<Frame, EngineError>;
}

/// 网络栈接口。DNS、代理、TLS、缓存策略、本地优先网络都由插件决定。
pub trait NetworkStack {
    fn name(&self) -> &str;
    fn fetch(&self, url: &str) -> Result<Vec<u8>, EngineError>;
}

/// 外壳界面接口。地址栏、标签页、设置页、主题、快捷键全是 UI 插件的事。
pub trait ShellUi {
    fn name(&self) -> &str;
    fn mount(&self, slot: &str) -> Result<(), EngineError>;
}

/// 平台后端接口。移动端与桌面端的差异被关在这个 trait 后面。
pub trait PlatformBackend {
    fn target(&self) -> Target;
    fn name(&self) -> &str;
}

/// 扩展运行时接口。一个运行时负责一种扩展方言。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExtensionFlavor {
    Chrome,
    Edge,
    Firefox,
    Hollo,
}

impl ExtensionFlavor {
    pub fn as_str(self) -> &'static str {
        match self {
            ExtensionFlavor::Chrome => "chrome",
            ExtensionFlavor::Edge => "edge",
            ExtensionFlavor::Firefox => "firefox",
            ExtensionFlavor::Hollo => "hollo",
        }
    }
}

pub trait ExtensionRuntime {
    fn name(&self) -> &str;
    fn flavor(&self) -> ExtensionFlavor;
    fn load(&self, package: &str) -> Result<(), EngineError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_supported_targets_have_triples() {
        for t in Target::SUPPORTED {
            assert_eq!(Target::parse(t.as_str()), Some(t));
            assert!(!t.rust_target_triple().is_empty());
            assert!(t.rust_target_triple().contains('-'));
        }
    }

    #[test]
    fn mobile_and_desktop_are_both_first_class() {
        assert!(Target::SUPPORTED.contains(&Target::Android));
        assert!(Target::SUPPORTED.contains(&Target::Windows));
        assert!(Target::SUPPORTED.contains(&Target::Linux));
    }
}
