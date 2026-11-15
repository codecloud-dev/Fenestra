# 变更记录 / Changelog

> 格式：按版本倒序。所有「通过」均来自真实执行，不写未验证的性能数字、不承诺工期。

## 0.1.0 · Stage 1 推翻重写（2026-11-15）

按方向定稿 §9「开工」授权，从 `9ee4bd4` 之后重写主干。

### 归档 Stage 0（不删文件、不丢测试）

- 新建远端分支 `archive/stage0-rust-kernel` 与轻量 tag `stage0`，均指向 `9ee4bd4`。
- 归档分支 tree 与当时 main 完全一致（`git ls-tree -r` 计数 **37** 个追踪文件；定稿 §7 写「46 文件」为口径差异）；在归档 worktree 实跑 `cargo test` → **33 单元 + 6 集成 = 39 测试全绿**。
- **迁移设计，不迁移代码**：默认拒绝、最小权限交集、隐私逐项确认、`Untrusted<T>` 出站重校验、可逆登记在新架构重新实现。

### 双许可拆分

- 新增 [LICENSE](./LICENSE)：层 A *Fenestra Open License v1.0* —— 自撰、符合 OSD 的宽松开源许可，非 OSI 认证，写明**已公开版本不得重新闭源**、层 A 永久开源、层 B 不得弱化安全不变量。
- 新增 [LICENSE-COMMERCIAL](./LICENSE-COMMERCIAL)：层 B，原 `LICENSE`（*Fenestra Source License v1.0* FSL）**逐字节原文**搬入，商用付费、改动必须开源、五不丢。

### 新架构脚手架（技术栈钉死）

- 根：[Cargo.toml](./Cargo.toml)（workspace 仅含 gate + exclude shell）、[package.json](./package.json)、[pnpm-workspace.yaml](./pnpm-workspace.yaml)、[tsconfig.base.json](./tsconfig.base.json)。
- [crates/gate](./crates/gate/)：纯 Rust 零依赖权限闸门，18 测试；`cargo clippy -D warnings` 通过。
- [packages/core](./packages/core/)：Cordis 4 插件运行时（load / enable / disable / unload / install、逆序回滚、热装不重启），11 测试、typecheck 通过。
- [packages/ui](./packages/ui/)：Svelte 5 + Vite 8 前端壳（地址栏 / 前进后退刷新 / 新标签页 / 插件管理页），`vite build` 通过。
- [packages/plugins](./packages/plugins/)：默认新标签页、示例替换、DNS 缓存 + 悬停预连接三个插件，3/3 typecheck 通过。
- [apps/shell/src-tauri](./apps/shell/src-tauri/)：Tauri 2 外壳，`RenderEngine` 可替换引擎接口层。**本地未编译**（缺 `libwebkit2gtk-4.1-dev`），由 CI 编译验证。

### CI 与守门

- [.github/workflows/ci.yml](./.github/workflows/ci.yml) 六 job：`gate / core / ui / shell / license-guard / docs`；保留原有 [.github/dependabot.yml](./.github/dependabot.yml)。
- 新增 [scripts/check-licenses.sh](./scripts/check-licenses.sh)（双许可守门）与 [scripts/check-docs.py](./scripts/check-docs.py)（链接 / 命名 / 徽章守门）。

### 文档

- [README.md](./README.md) 全面重写：双行徽章（stage 改 Stage 1）、「现在做到哪一步」诚实表、双许可边界、MVP 六件事验收表、stage0 归档说明。
- 同步 [README.en.md](./README.en.md)、[ARCHITECTURE.md](./ARCHITECTURE.md)、[ROADMAP.md](./ROADMAP.md)、[PRINCIPLES.md](./PRINCIPLES.md)、[SECURITY.md](./SECURITY.md)、[CONTRIBUTING.md](./CONTRIBUTING.md)、[EXTENSIONS.md](./EXTENSIONS.md)。

### 诚实标注的未完成项

端到端集成（Rust 闸门 ↔ TS 运行时 ↔ Tauri IPC ↔ UI）、敏感操作授权弹窗、真实网页渲染与引擎替换、三家扩展兼容层（Stage 2）均**未做**，见 [README.md](./README.md) 状态表。

---

## Stage 0 · 9ee4bd4（已归档）

- 零依赖 Rust 内核，39 测试全绿；FEP 清单、自撰 FSL、三份联网调研、三家扩展归一器与 6 个集成测试。
- 全量改名完成，仓库元数据补齐；该状态由 `stage0` tag 与 `archive/stage0-rust-kernel` 分支永久保留。
