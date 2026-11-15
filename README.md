<div align="center">

<h1>Fenestra · 只是个壳</h1>

<p align="center">
  <a href="https://github.com/codecloud-dev/Fenestra"><img src="https://img.shields.io/badge/Fenestra-%E5%8F%AA%E6%98%AF%E4%B8%AA%E5%A3%B3_%E5%85%B6%E4%BD%99%E7%9A%86%E6%8F%92%E4%BB%B6-8a7bff?style=for-the-badge&logo=github&logoColor=white" alt="Fenestra: 只是个壳 其余皆插件"></a>
  <a href="https://github.com/codecloud-dev/Fenestra/blob/main/README.md"><img src="https://img.shields.io/badge/Rust-%E5%AE%89%E5%85%A8%E9%97%B8%E9%97%A8-14181A?style=for-the-badge&logo=rust&logoColor=CE412B" alt="Rust: 安全闸门"></a>
  <a href="https://github.com/codecloud-dev/Fenestra/blob/main/README.md"><img src="https://img.shields.io/badge/TypeScript-%E6%8F%92%E4%BB%B6%E5%86%85%E6%A0%B8-14181A?style=for-the-badge&logo=typescript&logoColor=3178C6" alt="TypeScript: 插件内核"></a>
  <a href="https://github.com/codecloud-dev/Fenestra/blob/main/README.md"><img src="https://img.shields.io/badge/Tauri-%E8%B7%A8%E5%B9%B3%E5%8F%B0%E5%A4%96%E5%A3%B3-14181A?style=for-the-badge&logo=tauri&logoColor=FFC131" alt="Tauri: 跨平台外壳"></a>
  <a href="https://github.com/codecloud-dev/Fenestra/blob/main/README.md"><img src="https://img.shields.io/badge/2026-%E4%B8%AD_%2F_EN-14181A?style=for-the-badge&logo=github&logoColor=ff7ac3" alt="2026: 中 / EN"></a>
</p>

<p align="center">
  <a href="https://github.com/codecloud-dev/Fenestra/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/codecloud-dev/Fenestra/ci.yml?branch=main&style=flat-square&label=CI" alt="CI"></a>
  <a href="https://github.com/codecloud-dev/Fenestra/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-%E5%8F%8C%E8%AE%B8%E5%8F%AF_A%2FB-8a7bff?style=flat-square" alt="License: 双许可 A/B"></a>
  <a href="https://github.com/codecloud-dev/Fenestra/stargazers"><img src="https://img.shields.io/github/stars/codecloud-dev/Fenestra?style=flat-square&color=ff7ac3" alt="Stars"></a>
  <a href="https://github.com/codecloud-dev/Fenestra/commits/main"><img src="https://img.shields.io/github/last-commit/codecloud-dev/Fenestra?style=flat-square&color=8a7bff" alt="Last commit"></a>
  <a href="https://github.com/codecloud-dev/Fenestra"><img src="https://img.shields.io/badge/stage-Stage_1-ff7ac3?style=flat-square" alt="stage: Stage 1"></a>
  <a href="https://github.com/codecloud-dev/Fenestra"><img src="https://img.shields.io/badge/docs-%E4%B8%AD%E6%96%87_%2F_EN-2088FF?style=flat-square" alt="docs: 中文 / EN"></a>
  <a href="https://github.com/codecloud-dev/Fenestra/pulls"><img src="https://img.shields.io/badge/PRs-welcome-8a7bff?style=flat-square&logo=github&logoColor=white" alt="PRs welcome"></a>
</p>

<p><strong>只是个壳，其余皆插件 · Only a shell, everything else is a plugin</strong></p>

<p>
  <a href="./README.md">中文</a> ·
  <a href="./README.en.md">English</a> ·
  <a href="./ARCHITECTURE.md">架构 Architecture</a> ·
  <a href="./ROADMAP.md">路线 Roadmap</a> ·
  <a href="./PRINCIPLES.md">原则 Principles</a> ·
  <a href="./SECURITY.md">安全 Security</a> ·
  <a href="./CONTRIBUTING.md">贡献 Contributing</a> ·
  <a href="./CHANGELOG.md">更新日志 Changelog</a>
</p>

</div>

---

## 🔭 这是什么 / What is this

Fenestra 是一个**极小的浏览器空壳**：窗口、地址栏、标签页、插件管理是壳；渲染引擎、网络栈、界面、加速能力都是插件，现装现跑、现卸现回滚，**完全不需要重启**。壳本身用双许可分层发布，让「可信」和「能养活自己」不再二选一。

- **层 A（真开源）**：内核、权限闸门、能力仲裁、插件运行时、FEP 规范、示例插件 —— 自撰的宽松开源许可。
- **层 B（源码可见 / 商用付费）**：官方增值插件、未来企业版 —— FSL v1.0。

本项目当前处于 **Stage 1：推翻重写后的脚手架 + 可验证的最小链路**。方向定稿见仓库外的 `Fenestra_v0.1_方向定稿_2026-11-15.md`（§9 授权开工）。下表把「做到哪一步」说清楚，**不吹牛、不写没跑过的性能数字、不承诺工期**。

---

## 📍 现在做到哪一步 / Honest status

> 所有 ✅ 行的数字都来自本地真实执行；⚠️ / ⬜ 行如实标注未完成。

| 模块 / 事项 | 状态 | 证据（真实命令与结果） |
| --- | --- | --- |
| 🧱 monorepo 脚手架（pnpm + Cargo workspace） | ✅ 已落地 | 目录见下节 |
| 🔐 Rust 权限闸门 [crates/gate](./crates/gate/) | ✅ 本地真跑通过 | `cargo test -p fenestra-gate` → **18 passed / 0 failed**；`cargo clippy -D warnings` → 通过 |
| 🧩 Cordis 4 插件运行时 [packages/core](./packages/core/) | ✅ 本地真跑通过 | `pnpm --filter @fenestra/core test` → **11 passed / 0 failed**；typecheck → exit 0 |
| 🖼️ Svelte 5 + Vite 8 前端壳 [packages/ui](./packages/ui/) | ✅ 本地构建通过 | `pnpm --filter @fenestra/ui build` → vite 8.3.8，110 modules，**exit 0** |
| 🔌 三个示例插件 [packages/plugins](./packages/plugins/) | ✅ 类型检查通过 | `pnpm run plugins:typecheck` → **3/3 Done** |
| ⚖️ 双许可守门 | ✅ 本地真跑通过 | `bash scripts/check-licenses.sh` → 通过 |
| 📄 文档 / 链接 / 命名守门 | ✅ 本地真跑通过 | `python3 scripts/check-docs.py` → 通过 |
| 🪟 Tauri 2 外壳 [apps/shell/src-tauri](./apps/shell/src-tauri/) | ⚠️ **本地未编译** | 本沙箱缺 `libwebkit2gtk-4.1-dev`；由 CI 安装系统依赖后真编译（`cargo build`），本地不假装编译过 |
| 🔗 端到端集成（Rust 闸门 ↔ TS 运行时 ↔ Tauri IPC ↔ Svelte UI） | ⬜ 尚未打通 | 各包当前独立跑通；对接是下一步，暂无验收 |
| 🌐 打开真实网页 / 渲染引擎替换 | ⬜ 未做 | UI 包为状态占位；渲染由系统 WebView 承载，Servo 为 Stage 1 可替换插件 |
| 🔒 敏感操作授权弹窗 / 权限数据接真闸门 | ⬜ 未做 | 插件管理页当前使用本地镜像数据 |

**一句话**：安全闸门、插件生命周期、前端壳这三块「本体」已真跑通并有测试；把它们串成一个能打开真实网页、能弹授权框的完整浏览器，还没做完。

---

## 🧱 架构总览 / Architecture

```
Fenestra/
├── package.json / pnpm-workspace.yaml / tsconfig.base.json   # pnpm monorepo 根
├── Cargo.toml                                               # Cargo workspace（仅 gate）
├── crates/gate/            # 纯 Rust 权限闸门：零第三方依赖、零系统依赖
│   └── src/lib.rs          # 默认拒绝 / 最小权限交集 / Untrusted<T> / 可逆登记
├── packages/core/          # Cordis 4 + TS 插件运行时：load/enable/disable/unload
│   ├── src/index.ts        # PluginRuntime + FenestraApi（可逆登记）
│   └── test/runtime.test.ts# 11 个有断言的用例
├── packages/ui/            # Svelte 5 + Vite 8：地址栏/前进后退刷新/新标签页/插件管理页
├── packages/plugins/       # 默认新标签页 / 示例替换 / DNS 缓存+悬停预连接
│   ├── newtab/
│   ├── example-replace/
│   └── accelerate/
├── apps/shell/src-tauri/   # Tauri 2 外壳（Rust），引擎抽象成 RenderEngine 接口层
└── .github/workflows/ci.yml# 六 job CI
```

**技术栈（钉死，不再选型）**：

| 模块 | 选型 | 已核验版本 |
| --- | --- | --- |
| 插件内核 | Cordis 4 + TypeScript | `cordis 4.0.0-rc.13`（revertible effects 正对「卸载无残留」） |
| 窗口外壳 | Tauri 2 + Rust | Tauri 2.11.5 稳定线；`@tauri-apps/cli 2.12.4`、`@tauri-apps/api 2.12.4`；Rust crate 用 `2` 线由 Cargo 锁定 |
| UI | Svelte 5 | `svelte 5.57.3` |
| 构建 | Vite 8 + pnpm 10 + Cargo | `vite 8.3.8`、`pnpm 10.28.2` |
| 默认渲染 | 系统 WebView（wry） | 不打包引擎；Servo 为 Stage 1「第一个可替换引擎插件」 |
| 安全闸门 | Tauri PCS + 迁移过来的 Rust 能力仲裁 | 落在 [crates/gate](./crates/gate/) |

引擎从第一天就抽象成 [apps/shell/src-tauri/src/lib.rs](./apps/shell/src-tauri/src/lib.rs) 里的 `RenderEngine` trait + `EngineRegistry`，运行时是「注册」而非「编译期绑定」，保留替换点。

---

## ⚖️ 双许可边界 / Dual licensing

| 层 | 覆盖 | 许可文件 | 规则 |
| --- | --- | --- | --- |
| **A** | 内核、权限闸门 / 能力仲裁、插件运行时、FEP 规范、示例插件 | [LICENSE](./LICENSE) · *Fenestra Open License v1.0* | 自撰、符合开源定义（OSD）的**宽松**开源许可；可商用、可闭源派生、只需署名；**不打「OSI 认证」旗号**；**已公开版本不得重新闭源** |
| **B** | 官方增值插件、未来企业版 / 加速 Pro、托管服务 | [LICENSE-COMMERCIAL](./LICENSE-COMMERCIAL) · *Fenestra Source License (FSL) v1.0* 原文 | 源码可见；商用按「小额、透明、不歧视」付费；**改动必须开源**；不可退化条款（五不丢） |

**层 A 覆盖的具体目录/文件**（永久开源清单）：[crates/gate](./crates/gate/)、[packages/core](./packages/core/)、[packages/plugins](./packages/plugins/) 下的示例插件、[apps/shell/src-tauri](./apps/shell/src-tauri/) 外壳、[EXTENSIONS.md](./EXTENSIONS.md)（FEP 规范），以及对外暴露的 API / ABI 与闸门本体。

**边界规则（防两层互相侵蚀）**：

1. 层 A 的 **API / ABI 与闸门本体永久开源**，不得因层 B 商业化而闭源；**不得把已公开版本重新闭源**（已公开副本与 fork 永久按当时许可）。
2. 层 B **不得弱化**层 A 的**默认拒绝、沙箱隔离、隐私红线、出站重校验**；付了钱也不行，只能在边界之上叠加增值能力。
3. 贡献者协议只覆盖层 A；层 B 的授权发放与收款通道单独记账，不混进层 A 治理。
4. 层 A 为自撰文本，符合 OSD 实质但**不是「OSI 认证许可」**，也不得被称为任何既有许可证的变体。依赖许可（Servo `MPL-2.0`、Cordis / Svelte / Vite `MIT`、Tauri `Apache-2.0 OR MIT`）在 Stage 1 真引入 Servo 时逐文件核。

CI 的 `license-guard` job 用 [scripts/check-licenses.sh](./scripts/check-licenses.sh) 校验两文件齐备、层 A 自撰且符合 OSD、层 B 为 FSL 原文。

---

## ✅ MVP 六件事：做了什么 / 怎么验收

> 原样照搬定稿 §6 的六件事，不加第七件。「当前状态」列如实标注完成度，未打通的端到端链路明确写「未做」。

| # | 事 | 做了什么 | 怎么验收（真实可跑的命令） | 当前状态 |
| --- | --- | --- | --- | --- |
| 1 | 窗口 + 地址栏 + 输入 URL 打开网页 + 前进 / 后退 / 刷新 | [packages/ui](./packages/ui/) 实现地址栏、前进 / 后退 / 刷新、新标签页的真实状态机；[apps/shell/src-tauri](./apps/shell/src-tauri/) 提供窗口与系统 WebView | `pnpm --filter @fenestra/ui build`（通过）；窗口由 CI `shell` job 编译 | **部分**：状态机已跑通；打开真实网页由 WebView 承载，端到端未打通 |
| 2 | 最小 Cordis 插件系统：加载 / 启用 / 禁用 / 卸载，卸载逆序回滚无残留；现装现跑不重启 | [packages/core](./packages/core/) 的 `PluginRuntime` + Cordis 4 可逆 effect | `pnpm --filter @fenestra/core test` → **11 用例**（逆序回滚 LIFO、热装不重启、无残留、多插件隔离等） | ✅ **核心已实现并有测试** |
| 3 | 最小安全闸门：核心能力统一权限检查，未声明直接拒绝，敏感操作弹清晰授权框 | [crates/gate](./crates/gate/) 实现默认拒绝、最小权限交集、`Untrusted<T>` 出站重校验、可逆登记 | `cargo test -p fenestra-gate` → **18 用例**；`cargo clippy -D warnings` | **部分**：闸门本体已实现并有测试；授权弹窗与 TS↔Rust 对接未做 |
| 4 | 官方默认新标签页插件 + 示例插件替换它、卸载自动恢复 | `newtab` 与 `example-replace` 插件源码（同名 `ui/newtab` 服务覆盖，靠 Cordis 可逆登记恢复默认） | `pnpm run plugins:typecheck`（通过） | **部分**：插件源码已有；「替换 / 卸载恢复」端到端演示未写成测试 |
| 5 | 极简插件管理页：插件列表、一键启停、查看 / 撤销每个插件权限 | [packages/ui](./packages/ui/) 插件页：列表、启停、权限查看 / 撤销 | `pnpm --filter @fenestra/ui build`（通过） | **部分**：交互已跑通；权限数据当前为本地镜像，未接真闸门 |
| 6 | 第一个加速插件：基础 DNS 缓存 + 鼠标悬停预连接 | `accelerate` 插件：DNS 缓存服务 + 悬停预连接意图登记，作用域内可逆、卸载清空无残留 | `pnpm run plugins:typecheck`（通过） | **部分**：插件源码已有；未发起真实连接，**不写任何性能数字** |

**你补的那条**：所有插件现安装、现跑、现用，完全不需要重启 —— 归入第 2 件验收：`install()` 在同一个 Cordis `Context` 上挂新 Fiber，`disable()` 只 dispose 该 Fiber，进程不重启；对应测试「热装立即生效」「disable→enable 仍在同一 Context」。

红线原样保留：不自研 HTML/CSS/JS 引擎、不比功能全、不改系统内核、不做 VPN / 跨境加速、不做账号 / 云同步 / 商业功能、不收任何用户数据、零运营成本。

---

## 🔐 安全模型 / Security invariants

内核唯一钉死的不变量，在 [crates/gate](./crates/gate/) 以纯 Rust、零依赖实现，并有 18 个测试：

- **默认拒绝**：`check()` 对未声明 / 不在策略上限 / 未逐项确认的能力一律 `Deny`。
- **最小权限交集**：有效能力 = 声明集 ∩ 策略上限 ∩ 用户确认集（`effective()`），不放宽。
- **隐私红线逐项确认**：`ClipboardRead / ProfileRead / CameraDevice / LocationDevice / Telemetry` 须 `confirm()` 后才放行，确认按能力逐项、不连带。
- **BlastDoor 式 `Untrusted<T>` 出站重校验**：隔离单元产出被包成 `Untrusted<T>`，**不暴露任何读取内部值的途径**，离开边界前必须 `revalidate()` 通过才得到 `Trusted<T>`；校验失败则值随对象一同丢弃。
- **可逆登记**：`RevertibleRegistry` / `Gate::revoke()` 按登记逆序回滚，卸载插件后声明、确认、服务全部清空，无残留。

TS 侧的「可逆」由 Cordis 4 `ctx.effect / provide / on` 保证：插件登记的一切挂在自己的 Fiber 上，dispose 时逆序回滚（[packages/core](./packages/core/) 11 个测试）。两层「可逆」语义对齐，但**端到端 IPC 对接尚未打通**，见状态表。

威胁模型、上报方式与隐私承诺见 [SECURITY.md](./SECURITY.md)。

---

## 🗂️ stage0 归档说明 / Stage 0 archive

推翻重写前，原零依赖 Rust 内核（Stage 0）已完整归档，**不删文件、不丢测试**：

- 远端分支 `archive/stage0-rust-kernel` 与轻量 tag `stage0` 均指向 `9ee4bd4`（归档时 main 的 HEAD）。
- 归档分支的 tree 与当时的 main **完全一致**：`git ls-tree -r` 计数 **37** 个追踪文件；在归档 worktree 实跑 `cargo test` → **33 单元 + 6 集成 = 39 个测试全绿**。
- 口径说明：方向定稿 §7 写「46 文件」，而 git 实际追踪为 37（含 `.github/` 等）。以 `git ls-tree` 为准；「保留全部文件、不丢 39 测试」的目标已达成，未删除任何追踪文件。
- **迁移的是设计，不是代码**：默认拒绝、最小权限交集、隐私红线显式确认、`Untrusted<T>` 出站重校验、可逆登记，都在新架构里重新实现（见 [crates/gate](./crates/gate/)）。
- 归档分支里的三家扩展归一器与 6 个集成测试，留待 Stage 2 做 Chrome / Edge / Firefox MV3 兼容时直接取回。

主干 `main` 已从 Cordis 4 + Tauri 2 + Svelte + Vite + pnpm + Cargo 重新起脚手架，旧内核代码不迁移。

---

## 🚀 本地复现 / Reproduce locally

```bash
# Rust 闸门（纯标准库，零系统依赖）
cargo test -p fenestra-gate                              # 18 passed / 0 failed
cargo clippy -p fenestra-gate --all-targets -- -D warnings  # 通过

# 前端 workspace
pnpm install --frozen-lockfile
pnpm --filter @fenestra/core typecheck                  # exit 0
pnpm --filter @fenestra/core test                       # 11 passed / 0 failed
pnpm run plugins:typecheck                             # 3/3 Done
pnpm --filter @fenestra/ui build                        # vite 8.3.8, 110 modules, exit 0

# 守门
bash scripts/check-licenses.sh                          # 通过
python3 scripts/check-docs.py                           # 通过
```

Tauri 外壳需要 `libwebkit2gtk-4.1-dev` 等系统库，本沙箱缺，**本地未编译**；CI 的 `shell` job 会先 `apt-get install` 再 `cargo build --manifest-path apps/shell/src-tauri/Cargo.toml` 真编译：

```bash
# 仅在装好系统依赖的 Linux 上
cargo build --manifest-path apps/shell/src-tauri/Cargo.toml
```

CI 六 job（[.github/workflows/ci.yml](./.github/workflows/ci.yml)）：`gate` / `core` / `ui` / `shell` / `license-guard` / `docs`；**不使用 `continue-on-error` 把红变绿**。

---

## 🧭 路线 / Roadmap

- **Stage 1（当前）**：脚手架 + 六件事逐件验收；Servo 作为第一个可替换引擎插件，先在 Linux X11 跑通受控页面，量出真实二进制体积与兼容缺口，再据实决定要不要上桌面默认。
- **Stage 2（硬验收项）**：Chrome MV3 / Edge MV3 / Firefox MV3 全部兼容；自有 API 插件在能力与性能上最优；兼容层允许性能差异，但功能覆盖逐条列进文档。MVP 阶段不做兼容层，但插件内核 API 从第一天按「自有扩展一等公民 + 三家可映射」设计。

完整路线见 [ROADMAP.md](./ROADMAP.md)，设计原则见 [PRINCIPLES.md](./PRINCIPLES.md)，FEP 规范见 [EXTENSIONS.md](./EXTENSIONS.md)。

---

## 🚫 不做什么 / Non-goals

不承诺「几周发版」，工期等最小链路跑通再估；不把 Servo 说成能跑任意网站；不把「2-3MB」和 Servo 同时写进 README；不写没有真机基准的流畅度数字；不复用任何需要后端账号 / 云同步 / 统计的全家桶；Fenestra 纯本地、零运营成本、不收任何用户数据。

---

## 📄 许可 / License

双许可分层：[LICENSE](./LICENSE)（层 A，自撰 OSD 宽松开源）、[LICENSE-COMMERCIAL](./LICENSE-COMMERCIAL)（层 B，FSL v1.0）。边界见上文「双许可边界」。贡献前请读 [CONTRIBUTING.md](./CONTRIBUTING.md)。

*Fenestra · codecloud-dev · 2026 · 只是个壳，其余皆插件*
