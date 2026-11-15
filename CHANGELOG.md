# 变更记录

本项目使用语义化版本。Stage 0 期间版本号为 `0.1.x`。

## [0.1.0] — 2026-11-15

首个公开版本：**空壳内核与它的可验证契约**。不含图形界面，不能上网，见 [ROADMAP.md](ROADMAP.md)。

### 新增

- **内核**：零第三方依赖的 Rust crate，只有「读清单 / 仲裁能力 / 可逆登记」三件事。
- **FEP v1 清单**：纯文本逐行格式，自研解析器；单例字段重复、未知字段、未知能力一律报错。
- **能力模型**：封闭的 15 项 `Capability` 枚举；`Grant::intersect` 安装时求最小权限交集；`Grant::sanitized` 剔除隐私红线。
- **沙盒策略**：`SandboxPolicy` 三重闸门（锁定模式 / 隐私显式确认 / 默认拒绝）；`harden()` 冻结安全红线；`weakened()` 检测策略被削弱。
- **Containment**：`detonate()` → `Untrusted<T>` → `revalidate()`，类型级强制出站数据重校验，对标 Apple BlastDoor 思路。
- **可逆注册**：`Registry::register` / `revoke` / `revoke_plugin`；服务与订阅按 `PluginKind` 推导，卸载精确撤销。
- **三家扩展兼容层**：自研迷你 JSON 解析器；Chrome / Edge / Firefox `manifest.json` → FEP；方言差异输出 `warnings`；`to_hep()` 双向可审计。
- **平台**：`Target` 覆盖 Android / iOS / Windows / Linux / macOS 及对应 Rust triple；`engine.rs` 提供 `RenderEngine` / `NetworkStack` / `ShellUi` / `PlatformBackend` / `ExtensionRuntime` 五个可插拔 trait，内核无任何实现。
- **CLI**：`verify` / `compat` / `demo` / `policy` / `targets` / `--version`。
- **测试**：33 个单元测试 + 6 个集成测试，共 39 个，全绿；`cargo fmt --check`、`cargo clippy -D warnings` 干净。
- **CI**：Linux / Windows / macOS 三平台全流程；Android / iOS target 编译检查；零依赖与自撰许可证守门；文档齐备性检查。
- **许可证**：自撰 [Fenestra Source License v1.0](LICENSE)。源码可见、非 OSI 开源；作者可免费商用，他人商用需小额付费授权；任何改版必须公开完整对应源码；第 6 条把五不丢写成可判定条款。
- **文档**：中英 README、架构、安全、扩展、原则、路线图、贡献指南、行为准则，以及三份联网调研（DSH「一切皆插件」/ Apple 沙盒 / 2026 引擎格局）。

### 明确未包含

图形界面、真实渲染引擎插件、网络栈插件、OS 级进程隔离、插件签名、插件商店、安装包、性能基准数据。这些在 [ROADMAP.md](ROADMAP.md) 的 Stage 1–3 中逐条列出验收方式。

### 设计取舍

- 不使用现成 JSON 库、插件框架或许可证模板：内核必须在任何运行时启动之前可读清单，且保持零依赖。
- 不内置任何引擎：调研 2026 引擎格局后确认，从零造引擎不是本项目的战场，见 [`docs/research/browser-engines-2026.md`](docs/research/browser-engines-2026.md)。
- 不宣称"开源"：许可证对商业使用作了区分，不满足 OSD，见 LICENSE 第 0 条。
