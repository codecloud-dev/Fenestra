# 变更记录 / Changelog

> 格式：按版本倒序。所有「通过」均来自真实执行，不写未验证的性能数字、不承诺工期。

## 0.2.0 · MV3 清单归一层（2026-11-22）

### 新增

- `fenestra_gate::mv3`：Manifest V3 扩展清单的解析、合规校验与能力归一。零第三方依赖，带只读取所需字段的极简 JSON 解析器。
  - `Mv3Manifest::parse()` 读取 `manifest_version` / `permissions` / `host_permissions` / `background.service_worker` / `action` 等字段；
  - `validate()` 校验 MV3 硬约束（必须为 v3、禁用 `webRequestBlocking`、主机权限不得混入 `permissions`、远程代码告警）；
  - `to_capabilities()` / `to_fep_permissions()` 按默认拒绝把 MV3 权限映射到 Fenestra 能力，未知权限忽略。
- 5 个单测覆盖三家清单解析、能力映射、MV2/blocking 拒绝、未知权限默认拒绝、畸形 JSON；workspace 全量测试通过。

### 说明

- 当前只做「清单 → 能力」归一与守门；三家扩展的运行时 API 代理仍属后续工作，不声称已完成完整 MV3 兼容。

## 0.1.2 · 用户可见的可用性与可观测性（2026-11-21）

### 修复

- **新标签页前进/后退不再串页**：`sync()` 改为在切换历史项后先同步 URL 再更新标题，配合搜索占位提示的明确文案，避免用户以为「搜索没发出去」是 bug。
- **Tauri 权限拒绝可追溯**：`gate_audit` 暴露有界裁决日志，帮助定位「为什么这个插件能力没生效」；`Capability` 映射对未知字符串默认拒绝，不再静默放行。
- 禁用态控件补了明确的视觉样式，避免灰色按钮看起来像可点。
## 0.1.1 · 闸门真接入 + 可审计 + 并发幂等（2026-11-19）

本轮公开仓库深度升级：不靠 issue 清单，真读代码发现「Rust 闸门与 Tauri 外壳之间仍是硬编码允许列表占位」「插件 enable 并发可重复挂载」「没有可观测的拒绝原因」，逐项修掉并补回归。工作区版本统一升到 `0.1.1`（根 / gate / core / ui / 三个插件 / Tauri 外壳 / tauri.conf）。

### 新增 / 修复

- **Tauri `gate_check` 不再用硬编码允许列表占位**，改为进程内唯一 `OnceLock<Mutex<Gate>>`，真正调用 `fenestra_gate::Gate::check_permission` 裁决；策略上限只放开 `ui.newtab` / `net.dns` / `net.preconnect`，跨插件越权一律默认拒绝。
- **失败可审计**：`Capability::from_permission` 成为 TS 权限字符串与 Rust 能力的唯一映射入口，未知字符串默认拒绝；`check_audited` / `check_permission` 记录有界裁决日志（`AUDIT_CAPACITY=256`，防长期运行下拒绝探测把内存打爆），新增 `gate_audit` IPC 暴露最近拒绝原因。
- **插件运行时并发 enable 幂等（真 bug）**：此前只检查 `fibers.has(id)`，两次并发 `enable` 可能在第一次挂 Fiber 前同时通过检查，导致 `apply` 执行两次、effect 重复注册。新增 `enabling` in-flight map 共享第一次 Promise，并在 `finally` 里只清自己那一轮，防止误删后续重入任务。
- **真实微基准**：新增 `crates/gate/examples/bench.rs`（零第三方依赖）。本机 200000 次实测（随机器负载波动）：纯 `check` **5.5 ns/op**、带解析与有界审计的 `check_permission` **19.6 ns/op**、`effective` 含分配 **24.7 ns/op**。

### 验证

- `pnpm install --frozen-lockfile` EXIT=0。
- `cargo test -p fenestra-gate` EXIT=0：**24 单元 + 0 doc** 全过（新增 6 例覆盖权限字符串映射、默认拒绝、审计有界、跨插件越权）。
- `cargo clippy -p fenestra-gate --all-targets -- -D warnings` EXIT=0。
- `pnpm --filter @fenestra/core typecheck` EXIT=0；`pnpm --filter @fenestra/core test` EXIT=0：**12/12**（新增并发 enable 回归）。
- `pnpm run plugins:typecheck` EXIT=0（3/3 插件）；`pnpm --filter @fenestra/ui build` EXIT=0。
- `bash scripts/check-licenses.sh` EXIT=0；`python3 scripts/check-docs.py` EXIT=0（12 份根 Markdown 链接 / 命名 / 徽章 / 章节齐备）。
- Tauri 外壳依赖 `libwebkit2gtk-4.1-dev`，本地沙箱不具备该 GUI 栈，按既有分工由 GitHub Actions `shell` job 真编译验证；发布后盯 CI 到绿。

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
