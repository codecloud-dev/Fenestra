# 路线图 / Roadmap

> 不承诺工期。工期等最小链路跑通再估。本文只列「做什么、按什么顺序、怎么算做完」，不写日期。

方向定稿（`Fenestra_v0.1_方向定稿_2026-11-15.md`）§9 授权按「归档 stage0 → 双许可拆分 → 脚手架 + CI → 六件事逐件验收」开工。当前处于 **Stage 1**。

## Stage 0 · 已归档（不删、不丢）

- 分支 `archive/stage0-rust-kernel` 与轻量 tag `stage0` 指向 `9ee4bd4`。
- 保留全部 git 追踪文件（`git ls-tree -r` 计数 37；定稿写 46 为口径差异）与 39 个测试（33 单元 + 6 集成，实跑全绿）。
- 迁移的是设计，不是代码；归一器与 6 个集成测试留待 Stage 2 复用。
- 详见 [README.md](./README.md) 「stage0 归档说明」。

## Stage 1 · 当前：推翻重写后的脚手架

技术栈钉死：Cordis 4 + TypeScript / Tauri 2 + Rust / Svelte 5 + Vite 8 + pnpm + Cargo。默认渲染为系统 WebView（wry），**Servo 作为第一个可替换引擎插件**。

### 已完成（本地真跑通过）

| 项 | 证据 |
| --- | --- |
| monorepo 脚手架 + 双许可拆分 | 见 [ARCHITECTURE.md](./ARCHITECTURE.md) |
| Rust 权限闸门（4 个不变量，18 测试） | `cargo test -p fenestra-gate` |
| Cordis 插件运行时（逆序卸载 / 热装不重启，11 测试） | `pnpm --filter @fenestra/core test` |
| Svelte 前端壳（地址栏 / 前进后退 / 新标签页 / 插件页） | `pnpm --filter @fenestra/ui build` |
| 三个示例插件（新标签页 / 替换 / DNS 加速） | `pnpm run plugins:typecheck` |
| 六 job CI + 双许可 / 文档守门 | [.github/workflows/ci.yml](./.github/workflows/ci.yml) |

### 待做（明确写「未做」，见 [README.md](./README.md) 状态表）

- 端到端打通：Rust 闸门 ↔ TS 运行时 ↔ Tauri IPC ↔ Svelte UI；
- 敏感操作清晰授权弹窗、插件管理页权限数据接真闸门；
- 输入 URL 打开真实网页（渲染由系统 WebView 承载）；
- 示例插件「替换默认新标签页、卸载自动恢复」的端到端演示测试；
- 加速插件发起真实预连接（仍不碰系统内核 / VPN / eBPF / BBR，不写性能数字）。

### Stage 1 内的引擎验证（Servo）

Servo 降为「第一个可替换引擎插件」：在 Linux X11 跑通受控页面 → 量出**真实**二进制体积与兼容缺口 → 再据实决定要不要上桌面默认。不把 Servo 说成能跑任意网站，不把「2-3MB」和 Servo 同时写进 README。

## Stage 2 · 硬验收项（不是「生态起来再说」的软话）

- Chrome MV3 / Edge MV3 / Firefox MV3 **全部兼容**；
- 自有 API 插件在能力与性能上**最优**（一等公民）；
- 兼容层允许性能差异，但功能覆盖要**逐条列进文档**；
- 复用 Stage 0 归档的归一器与 6 个集成测试。

MVP 六件事**不含兼容层**（守住「多一件都不要加」），但插件内核 API 从第一天按「自有扩展一等公民 + 三家可映射」设计，不把路堵死。FEP 见 [EXTENSIONS.md](./EXTENSIONS.md)。

## 长期不做（红线）

不自研 HTML/CSS/JS 引擎、不比功能全、不改系统内核、不做 VPN / 跨境加速、不做账号 / 云同步 / 商业功能（层 B 增值插件除外）、不收任何用户数据、零运营成本。见 [PRINCIPLES.md](./PRINCIPLES.md)。
