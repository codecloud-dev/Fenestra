# 架构 / Architecture

> Fenestra 只是个壳，其余皆插件。本文描述 **Stage 1 重写后的新架构**，与 Stage 0 的零依赖 Rust 内核是两套代码：迁移的是设计，不是代码。现状见 [README.md](./README.md) 「现在做到哪一步」。

## 1. 分层

```
┌─────────────────────────────────────────────────────────────┐
│ apps/shell/src-tauri   Tauri 2 窗口外壳（Rust）              │
│   RenderEngine trait / EngineRegistry（引擎可替换接口层）     │
│   gate_check / engine_name（IPC，默认拒绝示意）              │
├─────────────────────────────────────────────────────────────┤
│ packages/ui            Svelte 5 + Vite 8 前端壳              │
│   地址栏 / 前进后退刷新 / 新标签页 / 插件管理页              │
├─────────────────────────────────────────────────────────────┤
│ packages/core          Cordis 4 插件运行时（TypeScript）      │
│   PluginRuntime: load / enable / disable / unload / install  │
│   FenestraApi: effect / provide / get / on / permissions     │
│ packages/plugins       默认新标签页 / 示例替换 / 加速插件     │
├─────────────────────────────────────────────────────────────┤
│ crates/gate            纯 Rust 权限闸门（零第三方依赖）       │
│   默认拒绝 / 最小权限交集 / Untrusted<T> / 可逆登记          │
└─────────────────────────────────────────────────────────────┘
```

依赖方向：`ui / plugins → core → cordis`，`gate` 独立，`shell` 独立 workspace。两层 workspace 刻意隔离。

## 2. 两个 workspace 为什么分开

- 根 [Cargo.toml](./Cargo.toml) 的 workspace 只含 [crates/gate](./crates/gate/)，并 `exclude = ["apps/shell/src-tauri"]`。这样 `cargo test -p fenestra-gate` 完全不触碰 `webkit2gtk` 等系统库，本地可零依赖真跑。
- [apps/shell/src-tauri/Cargo.toml](./apps/shell/src-tauri/Cargo.toml) 自带 `[workspace]`，是独立 Cargo workspace，依赖 Tauri 2；由 CI 安装系统依赖后编译。
- 前端由 [pnpm-workspace.yaml](./pnpm-workspace.yaml) 管理（`packages/*` 与 `packages/plugins/*`），[package.json](./package.json) 为根。

## 3. 插件内核（packages/core）

底座是 Cordis 4 的 **revertible effects**（已用探针实测其运行时语义）：

- `PluginRuntime` 持有一个长期存活的 `Context`。`load()` 只登记；`enable()` 在该 Context 上 `ctx.use(component)` 挂一根 **Fiber** 并 `await fiber` 等生效；`disable()` 调 `fiber.dispose()`；`unload()` 再删定义与记录。
- 插件通过 `FenestraApi` 登记的 `effect / provide / on` 都挂在自己的 Fiber 上。dispose 时 Cordis 按**注册逆序**执行 cleanup，并自动移除服务与监听 —— 这就是「卸载逆序回滚无残留」。
- `install()` = `load()` + `enable()`，`disable()` 后可再 `enable()`：**进程不重启**（同一个 Context），满足「现装现跑、现卸现回滚」。
- `auditLog` 记录 `load/setup/enable/cleanup/disable/unload`，供 [packages/core/test/runtime.test.ts](./packages/core/test/runtime.test.ts) 断言顺序（11 个用例）。

权限在插件 `meta.permissions` 中**声明**，`permissionsOf()` 只如实交出声明，**不自动授予**；是否放行由闸门裁决。

## 4. 权限闸门（crates/gate）

纯 Rust、零第三方依赖，见 [crates/gate/src/lib.rs](./crates/gate/src/lib.rs)：

| 不变量 | 实现 |
| --- | --- |
| 默认拒绝 | `Gate::check()` → `Decision::Deny(NotDeclared / PolicyForbidden / ConsentRequired)` |
| 最小权限交集 | `Gate::effective()` = 声明集 ∩ 策略上限 ∩ 用户确认集 |
| 隐私红线逐项确认 | `Capability::is_privacy_sensitive()` + `Gate::confirm()`，按能力逐项、不连带 |
| BlastDoor 式出站重校验 | `Untrusted<T>` 不暴露 `inner` / `Deref` / `into_inner`；唯一解封路径 `revalidate()` → `Trusted<T>`，失败丢弃 |
| 可逆登记 | `RevertibleRegistry::rollback()` / `extract()` 逆序回滚；`Gate::revoke()` 清空声明、确认、登记无残留 |

18 个测试覆盖上述全部不变量，`cargo clippy -D warnings` 通过。

## 5. 引擎可替换接口层（apps/shell/src-tauri）

[apps/shell/src-tauri/src/lib.rs](./apps/shell/src-tauri/src/lib.rs) 里的 `RenderEngine` trait（`name / description / can_handle`）+ `EngineRegistry` 把引擎抽象成**注册**而非编译期绑定：

- MVP 注册 `SystemWebView`（Tauri 2 默认 wry，不打包引擎，对应小体积与 iOS / Android 支持）；
- Servo 作为 **Stage 1 的第一个可替换引擎插件**，实现同一 trait 后注册，先在 Linux X11 跑通受控页面、量真实体积与兼容缺口，再据实决定是否上桌面默认；
- `gate_check` 是「Rust 闸门 ↔ 前端」IPC 的**默认拒绝示意**（生产版接 `Gate::check`），`engine_name` 暴露当前引擎。

## 6. 前端壳（packages/ui）

Svelte 5 runes + [packages/ui/src/App.svelte](./packages/ui/src/App.svelte)：地址栏输入、前进 / 后退 / 刷新、新标签页、标签关闭是真实状态机；插件管理页有列表、一键启停、权限查看 / 撤销。当前权限数据是**本地镜像**，真实网页渲染由系统 WebView 承载，前端包为状态占位。`pnpm --filter @fenestra/ui build` 真跑通过。

## 7. CI（[.github/workflows/ci.yml](./.github/workflows/ci.yml)）

六 job：`gate`（cargo test + clippy -D warnings）/ `core`（typecheck + test）/ `ui`（build）/ `shell`（装系统依赖后 Tauri 真编译）/ `license-guard`（[scripts/check-licenses.sh](./scripts/check-licenses.sh)）/ `docs`（[scripts/check-docs.py](./scripts/check-docs.py)）。不使用 `continue-on-error` 把红变绿。

## 8. 尚未打通（如实）

- Rust 闸门 ↔ TS 运行时 ↔ Tauri IPC ↔ Svelte UI 的端到端链路；
- 敏感操作授权弹窗、插件管理页权限数据接真闸门；
- 真实网页渲染与引擎替换、三家扩展兼容层（Stage 2）。

各包当前独立跑通，串起来是下一步。设计原则见 [PRINCIPLES.md](./PRINCIPLES.md)，路线见 [ROADMAP.md](./ROADMAP.md)。
