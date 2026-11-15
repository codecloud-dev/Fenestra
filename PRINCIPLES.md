# 原则 / Principles

> 把口号写成可验收条款。这些原则约束代码、文档与双许可，Stage 0 与 Stage 1 一脉相承（迁移设计，不迁移代码）。

## 1. 只是个壳，其余皆插件

窗口、地址栏、标签页、插件管理是壳；渲染引擎、网络栈、界面、加速能力都是插件。一切可改：默认新标签页可被替换，卸载后自动恢复；引擎抽象成 [apps/shell/src-tauri/src/lib.rs](./apps/shell/src-tauri/src/lib.rs) 的 `RenderEngine` trait，注册而非编译期绑定。

## 2. 默认拒绝 / Default deny

任何能力，未显式声明、不在策略上限、未经用户逐项确认，一律拒绝。实现见 [crates/gate](./crates/gate/) 的 `Gate::check()`，18 个测试守着。声明 ≠ 授权：插件 `meta.permissions` 只是声明，[packages/core](./packages/core/) 不自动授予。

## 3. 最小权限交集 / Least privilege

有效能力 = 声明集 ∩ 策略上限 ∩ 用户确认集（`Gate::effective()`），只取交集、不放宽。隐私红线能力（`ClipboardRead / ProfileRead / CameraDevice / LocationDevice / Telemetry`）必须**逐项**确认，确认一个不连带放行另一个。

## 4. 出站重校验 / BlastDoor

隔离单元产出的数据包成 `Untrusted<T>`，**不暴露内部值**；离开隔离边界前必须 `revalidate()` 通过才得到 `Trusted<T>`，校验失败则丢弃。不许留后门绕过校验。

## 5. 可逆登记 / Revertible everything

插件注册的任何东西都能逆序、无残留地撤销。Rust 侧 `RevertibleRegistry` / `Gate::revoke()`；TS 侧 Cordis 4 的 `ctx.effect / provide / on` + `fiber.dispose()`（[packages/core](./packages/core/) 11 个测试断言 LIFO 逆序与无残留）。装 / 卸插件**现装现跑、进程不重启**。

## 6. 隐私与自由

- 不默认收集或上传浏览历史、页面内容、剪贴板、位置、凭据、设备指纹；不植入未经明示同意且可随时关闭的遥测。
- 不得移除用户卸载任意插件、更换引擎 / 网络栈 / 界面、导出本地数据、完全离线运行的能力；不得加入「只能从单一商店装插件」的限制。
- 纯本地、零运营成本、不收任何用户数据。

## 7. 双许可边界不可侵蚀

- 层 A 的 API / ABI 与闸门本体**永久开源**，[LICENSE](./LICENSE) 写明**已公开版本不得重新闭源**；
- 层 B [LICENSE-COMMERCIAL](./LICENSE-COMMERCIAL) 不得弱化默认拒绝 / 沙箱隔离 / 隐私红线 / 出站重校验，**付了钱也不行**；
- 贡献者协议只覆盖层 A；层 B 授权与收款单独记账。
- 层 A 自撰、符合 OSD 实质，但**不是「OSI 认证许可」**，也不叫任何既有许可证的变体。

## 8. 引擎：不自己从零写，但可替换

不自研 HTML/CSS/JS 引擎。MVP 用系统 WebView（wry）保小体积与跨平台；Servo 是 Stage 1 可替换引擎插件，先在 Linux X11 跑通受控页面、量**真实**体积与兼容缺口，再据实决定。不把 Servo 说成能跑任意网站。

## 9. 兼容：自有最好，三家全兼容

MVP 不做兼容层，但内核 API 从第一天按「自有扩展一等公民 + Chrome / Edge / Firefox 三家可映射」设计。Stage 2 是**硬验收项**：三家 MV3 全兼容，兼容层允许性能差异，功能覆盖逐条列进文档。见 [ROADMAP.md](./ROADMAP.md) 与 [EXTENSIONS.md](./EXTENSIONS.md)。

## 10. 诚实 / Honesty

- 不吹牛：所有「通过」都来自真实执行，贴真实通过数 / 失败数；CI 不把红变绿，不用 `continue-on-error` 掩盖失败。
- 不写没跑过的性能数字，不承诺工期（工期等最小链路跑通再估）。
- 本地没编译的东西如实说没编译：本沙箱缺 `libwebkit2gtk-4.1-dev`，[apps/shell/src-tauri](./apps/shell/src-tauri/) 由 CI 编译，本地不假装跑过。
- 「现在做到哪一步」表见 [README.md](./README.md)：已做 / 部分 / 未做逐项标注。

架构见 [ARCHITECTURE.md](./ARCHITECTURE.md)，安全见 [SECURITY.md](./SECURITY.md)。
