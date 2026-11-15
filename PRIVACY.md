# 隐私政策 / Privacy Policy

**适用仓库**：`codecloud-dev/Fenestra`（极小的浏览器空壳 / Browser shell）
**生效日期 / Effective date**：2026-11-15
**责任主体**：`codecloud-dev` —— 一个 GitHub 个人账号，由个人开发者运营，**不是公司、不是法人实体**；没有公司名称、注册号、注册地址或客服电话。版权人标识见 `LICENSE` 与 `LICENSE-COMMERCIAL`，均为 `codecloud-dev`（GitHub 账号）。

> 中文在前，英文在后，同一份文件；口径一致，歧义以中文为准。
> 本政策按 `PRINCIPLES.md` / `SECURITY.md` / `ARCHITECTURE.md` / `EXTENSIONS.md` / `README.md` 的「现在做到哪一步」表，以及 `packages/plugins/accelerate/src/index.ts`、`apps/shell/src-tauri/capabilities/default.json` 的**真实代码状态**写。README 明确要求「不吹牛、不写没跑过的性能数字」；本政策同样把「当前版本根本没实现的能力」写成「未实现 / 未收集」，不把设计承诺伪装成已上线的功能。
> 撰写口径参考 GDPR 与《个人信息保护法》的最小必要、目的限定、透明原则，不引用、不复制任何条文原文。

---

## 一、当前版本到底能不能上网、收不收数据

**当前版本（Stage 1：推翻重写后的脚手架 + 可验证的最小链路）**：

| 能力 | 状态（README「现在做到哪一步」原文口径） |
| --- | --- |
| Rust 权限闸门 `crates/gate` | ✅ 本地真跑通过（18 passed / 0 failed） |
| Cordis 4 插件运行时 `packages/core` | ✅ 本地真跑通过（11 passed / 0 failed） |
| Svelte 5 + Vite 8 前端壳 `packages/ui` | ✅ 本地构建通过（状态占位） |
| 三个示例插件 | ✅ 类型检查通过 |
| 双许可守门 / 文档守门 | ✅ 本地真跑通过 |
| Tauri 2 外壳 `apps/shell/src-tauri` | ⚠️ **本地未编译**（沙箱缺 `libwebkit2gtk-4.1-dev`，由 CI 编译） |
| 端到端集成（闸门 ↔ 运行时 ↔ IPC ↔ UI） | ⬜ **尚未打通** |
| **打开真实网页 / 渲染引擎替换** | ⬜ **未做**：UI 包为状态占位，渲染由系统 WebView 承载，Servo 为可替换插件 |
| **敏感操作授权弹窗 / 权限数据接真闸门** | ⬜ **未做**：插件管理页当前使用本地镜像数据 |

**结论**：当前版本**还没有打开真实网页的能力，没有插件联网下载，没有账号系统，没有云同步，没有任何数据上传**。因此「我们收集什么」的诚实答案是：**当前版本不收集任何用户数据**。

`apps/shell/src-tauri/capabilities/default.json` 只授予 `core:default`（主窗口与 WebView 基础权限）；`SECURITY.md` 明确 `gate_check` IPC 命令**当前是默认拒绝示意，生产版才改为调用 `Gate::check`**。

---

## 二、代码里已经落地的安全模型（这是设计约束，不是「已上线的数据流」）

`PRINCIPLES.md` / `SECURITY.md` / `crates/gate` 把隐私写成了可验收条款，并且被 `LICENSE`（层 A 第 5 条）与 `LICENSE-COMMERCIAL`（层 B 第 6 条「五不丢」）**双向锁死，付钱也不得弱化**：

1. **默认拒绝 / Default deny**：任何能力，未显式声明、不在策略上限、未经用户逐项确认，一律拒绝（`Gate::check()`，18 个测试守着）。声明 ≠ 授权：插件 `meta.permissions` 只是声明，不自动授予。
2. **最小权限交集 / Least privilege**：有效能力 = `声明集 ∩ 策略上限 ∩ 用户确认集`（`Gate::effective()`）。隐私红线能力必须**逐项**确认：`ClipboardRead` · `ProfileRead` · `CameraDevice` · `LocationDevice` · `Telemetry`，确认一个不连带放行另一个。
3. **出站重校验 / BlastDoor**：隔离单元产出的数据包成 `Untrusted<T>`，**不实现 `Deref` / `AsRef`、不提供 `into_inner`**；离开隔离边界前必须 `revalidate(validator)` 通过（scheme 白名单、结构完整性、长度上限、不得夹带未声明字段）才得到 `Trusted<T>`，校验失败随对象一并丢弃，不许留后门。`crates/gate/src/lib.rs` 顶层 `#[forbid(unsafe_code)]`。
4. **可逆登记 / Revertible everything**：插件注册的任何东西都能逆序、无残留撤销（Rust `RevertibleRegistry` / `Gate::revoke()`；TS 侧 `ctx.effect / provide / on` + `fiber.dispose()`）。装/卸插件现装现跑、进程不重启。
5. **隐私与自由（PRINCIPLES §6 / SECURITY §3）**：不默认收集或上传**浏览历史、页面内容、剪贴板、位置、凭据、设备指纹**；不植入未经明示同意且可随时关闭的遥测；不得移除用户卸载任意插件、更换引擎/网络栈/界面、导出本地数据、**完全离线运行**的能力；不得加入「只能从单一商店装插件」的限制；**纯本地、零运营成本、不收任何用户数据、不做账号 / 云同步**。

### 示例加速插件的真实行为（`packages/plugins/accelerate/src/index.ts`）

- 声明权限 `net.dns` / `net.preconnect`，但**声明 ≠ 授权**，是否放行由 Rust 闸门按默认拒绝裁决。
- 代码里只做两件事：一个内存 `Map` 的 DNS 缓存（`lookup` / `put` / `size`）、一个 `ui/link-hover` 事件监听把悬停链接记成 `lastHover`。注释写明「**这里只登记意图，不发起真实连接；真实连接由宿主网络栈在闸门放行后执行**」。
- 红线：不碰系统内核 / VPN / eBPF / BBR，**不收任何用户数据**；卸载时 `cache.clear()`、`lastHover = null`，无残留。
- 即：当前版本**没有真实预连接、没有 DNS 查询外发、没有数据采集**。

---

## 三、我们收集什么

**当前版本：不收集。** 逐项说明：

- **不收集**：浏览历史、页面内容、地址栏输入、Cookie、剪贴板、位置、凭据、设备指纹、通讯录、相册、生物特征、崩溃、遥测、用量统计、安装统计、广告标识符。
- **不设置**：账号、登录、云同步、Cookie 会话（本仓库无服务端）、追踪像素、广告 SDK、统计 SDK、崩溃自动上报。
- **不上传**：任何数据到 `codecloud-dev` 或任何第三方服务器（本仓库无服务器、无后端地址）。
- **不落库**：无数据库；闸门、插件运行时、UI 均为本地进程内状态。

`EXTENSIONS.md` 的权限清单里涉及剪贴板 / 档案 / 摄像头 / 位置 / 遥测的能力属于**隐私红线，默认拒绝、须用户逐项显式确认**——这是规则，不是「当前已在收集」的证据。

---

## 四、存多久、存哪、与谁共享

| 项目 | 位置 | 说明 |
| --- | --- | --- |
| 内核 / 闸门 / 运行时 / UI 状态 | **你的本机进程内存** | 退出即消失；当前无持久化用户数据 |
| DNS 缓存 / 悬停链接（示例加速插件） | 本机内存 `Map` / 局部变量 | 卸载插件或关闭进程即清空，无残留 |
| 插件、配置、扩展 | 本机文件（由用户放置/安装） | 由用户自行管理、删除 |
| 仓库源码与文档 | GitHub（及可能的 Gitee 镜像） | 公开静态文件，随 Git 历史留存；非用户数据 |
| 你的个人数据 | **无** | 无留存、无共享、无跨境链路 |

**共享方**：当前版本只有代码托管平台 **GitHub**（以及国内只读镜像 **Gitee**，若启用）与其 CDN —— 它们只在**你克隆/浏览仓库**时产生常规访问日志，与浏览器运行无关。未来若打开网页，网页本身与第三方插件会按各自条款处理其收到的数据，但**那是用户主动发起的访问、由对方负责**，不是本壳在上传。

我们**不出售、不出租、不用于广告、不向数据经纪人提供**任何数据；`LICENSE-COMMERCIAL` 第 6.1 条把「隐私不丢」列为不可退化条款，**商业授权付费也不得解除**。

---

## 五、未来版本会改变什么（如实预告，不写成已有能力）

- **打开真实网页**：规划由系统 WebView（wry）承载，Servo 作为可替换引擎插件。届时网络请求由**用户主动导航**产生，目标网站按其自身条款处理；壳仍不默认上传数据。
- **插件 / 扩展**：规划按 FEP（`EXTENSIONS.md`）与 Chrome / Edge / Firefox 三家 MV3 兼容层加载；权限仍走默认拒绝 + 最小权限交集 + 用户逐项确认。
- **商用授权**：规划中（README「未来商用授权」、层 B `LICENSE-COMMERCIAL`）。依 `LICENSE-COMMERCIAL` 第 4.2 条，商业授权的**唯一入口**是「Fenestra 官网登录 → 签署《Fenestra 商业授权电子协议》→ 月付 / 年付两档付费 → 系统签发载明期限的电子授权凭证」，**明确不通过邮件、私信、仓库 Discussion 或任何线下沟通发放**。**当前版本未实现**官网登录、电子协议、付费与授权校验，该通道尚未上线，因此**当前不收集任何账号、订单或支付信息，也不存在任何取得商业授权的替代途径，任何层 B 商业使用都不得开始**。若未来上线，官网登录记录、协议签署日志、授权凭证与支付流水涉及的数据项、目的、留存与共享，会在**届时更新的隐私政策**与 `TERMS.md` 中逐项写明，并以 `LICENSE-COMMERCIAL` 为准（见 `TERMS.md`）。
- 任何上述变化都**不得**违反 `PRINCIPLES.md` §6 与 `LICENSE-COMMERCIAL` 第 6 条「五不丢」；若某版本弱化了默认拒绝、沙箱隔离、隐私红线或出站重校验，即违反许可，用户可据此主张。

---

## 六、你能怎么查看、重置、删除

- 当前无账号、无云端数据，故**没有需要向我们申请删除的数据**。
- 本地产生的一切（插件、配置、扩展、缓存）都在你自己的设备上：删除对应文件 / 卸载插件 / 关闭进程即可。
- 想核实「壳有没有偷偷联网」，可直接审计 `crates/gate`、`packages/core`、`packages/plugins` 源码，或用系统网络工具观察；`SECURITY.md` 与 CI 守门脚本（`scripts/check-licenses.sh`、`scripts/check-docs.py`）可复跑。
- 若你克隆了仓库、在 GitHub 上开过 Issue / Discussion，那些内容由 **GitHub** 按其条款处理，请到 GitHub 平台自行管理。
- **当前版本未提供**「数据导出」「清空遥测」之类的开关，因为没有可导出的数据、也没有遥测；`PRINCIPLES.md` §6 承诺未来版本**不得移除**导出本地数据与完全离线运行的能力。

---

## 七、儿童

不面向未满 14 周岁的儿童；无账号、无年龄核验，不有意收集儿童信息。当前版本无任何数据入口；若未来版本引入账号或付费，将按届时政策补充儿童条款。

## 八、跨境

当前版本无数据传输：源码公开托管于 GitHub（及可能的 Gitee 镜像），读者可能位于任何国家/地区，平台按其条款分发这些**公开静态文件**。浏览器运行时不向任何服务器上传数据，故不存在个人数据跨境。未来用户主动访问境外网站时，由该网站按其条款处理，本壳不作承诺、不代为传输。

## 九、第三方服务条款入口（不为其背书）

| 服务 | 条款 / 隐私政策入口 |
| --- | --- |
| GitHub | `https://docs.github.com/en/site-policy/github-terms/github-terms-of-service` · `https://docs.github.com/en/site-policy/privacy-policies/github-privacy-statement` |
| Gitee | `https://gitee.com/`（以其站内公布的条款为准） |
| Cloudflare（若未来官网/授权服务使用） | `https://www.cloudflare.com/privacypolicy/` |
| 系统 WebView / wry / Servo / Tauri / Svelte / Vite / Cordis 等依赖 | 见各自项目公布的许可与条款；Stage 1 真引入 Servo 时逐文件核依赖许可（README 明示） |

以各站最新公布条款为准，我们不代替其解释、不担保可用性、不对其行为负责。

## 十、变更通知与联系方式

- 变更直接更新本仓库 `PRIVACY.md`，留痕于 Git 提交与 CHANGELOG；不收集联系方式，故**不主动推送**，重大变更在仓库公告。若未来引入账号/付费，将同步更新本政策与 `TERMS.md`。
- **唯一联系方式**：在对应仓库开 **GitHub Issues**（`https://github.com/codecloud-dev/<repo>/issues`），或联系 GitHub 账号 **`codecloud-dev`**；该渠道**仅用于一般问题反馈、安全报告与条款咨询**，不用于发放商业授权。`LICENSE-COMMERCIAL` 第 4.2 条与附「一眼看懂」明确规定商业授权**只**通过官网登录、签署电子协议、付费后由系统签发凭证取得，邮件、私信、Discussion、线下沟通一律不发放。
- 不提供、不编造客服邮箱、电话、通信地址、公司名、注册号。请勿在公开 Issue 里贴密钥或凭据。

---
---

# English

**Repository**: `codecloud-dev/Fenestra` (a minimal browser shell)
**Effective date**: 2026-11-15
**Controller**: `codecloud-dev` — a personal GitHub account run by an individual developer, **not a company or legal entity**; no company name, registration number, registered address, or support phone. The copyright holder in `LICENSE` and `LICENSE-COMMERCIAL` is `codecloud-dev` (GitHub account).

> Chinese first, English after, in one file; the Chinese text controls in case of conflict.
> Written against the "Honest status" table in `PRINCIPLES.md` / `SECURITY.md` / `ARCHITECTURE.md` / `EXTENSIONS.md` / `README.md` and the **real code state** of `packages/plugins/accelerate/src/index.ts` and `apps/shell/src-tauri/capabilities/default.json`. The README demands "no boasting, no unmeasured performance numbers"; this policy likewise marks anything not yet built as "not implemented / not collected", never dressing a design promise up as a shipped feature.
> Written in the spirit of GDPR / PIPL minimisation, purpose limitation and transparency; no statutory text is quoted or copied.

## 1. Can the current version actually go online, and does it collect data?

**Current version (Stage 1: rewritten scaffold + a verifiable minimal chain)**:

| Capability | Status (wording from the README "Honest status" table) |
| --- | --- |
| Rust permission gate `crates/gate` | ✅ Actually runs locally (18 passed / 0 failed) |
| Cordis 4 plugin runtime `packages/core` | ✅ Actually runs locally (11 passed / 0 failed) |
| Svelte 5 + Vite 8 front-end shell `packages/ui` | ✅ Builds locally (state placeholder) |
| Three example plugins | ✅ Typecheck passes |
| Dual-license guard / docs guard | ✅ Actually runs locally |
| Tauri 2 shell `apps/shell/src-tauri` | ⚠️ **Not compiled locally** (sandbox lacks `libwebkit2gtk-4.1-dev`; CI compiles it) |
| End-to-end integration (gate ↔ runtime ↔ IPC ↔ UI) | ⬜ **Not yet wired** |
| **Opening real web pages / engine replacement** | ⬜ **Not done**: the UI package is a state placeholder; rendering is delegated to the system WebView, Servo is a replaceable plugin |
| **Sensitive-action consent dialog / permissions wired to the real gate** | ⬜ **Not done**: the plugin manager page currently uses local mirrored data |

**Conclusion**: the current version **cannot open real web pages, has no online plugin downloads, no account system, no cloud sync, and no data upload**. So the honest answer to "what do we collect" is: **the current version collects no user data**.

`apps/shell/src-tauri/capabilities/default.json` grants only `core:default` (main window + basic WebView permissions); `SECURITY.md` states the `gate_check` IPC command **is currently a default-deny stub, and only the production version will call `Gate::check`**.

## 2. The security model already in the code (design constraints, not a shipped data flow)

`PRINCIPLES.md` / `SECURITY.md` / `crates/gate` turn privacy into testable clauses, locked from both sides by `LICENSE` (Layer A, §5) and `LICENSE-COMMERCIAL` (Layer B, §6 "five must-not-lose"), **which payment cannot weaken**:

1. **Default deny**: any capability not explicitly declared, outside the policy ceiling, or not individually confirmed by the user is denied (`Gate::check()`, guarded by 18 tests). Declaration ≠ authorisation: a plugin's `meta.permissions` is only a declaration, never auto-granted.
2. **Least-privilege intersection**: effective capabilities = `declared ∩ policy ceiling ∩ user-confirmed` (`Gate::effective()`). Privacy red-line capabilities require **per-item** confirmation: `ClipboardRead` · `ProfileRead` · `CameraDevice` · `LocationDevice` · `Telemetry`; confirming one never unlocks another.
3. **Outbound revalidation / BlastDoor**: data produced by an isolated unit is wrapped in `Untrusted<T>`, which **implements neither `Deref` / `AsRef` nor exposes `into_inner`**; before leaving the isolation boundary it must pass `revalidate(validator)` (scheme whitelist, structural integrity, length caps, no undeclared fields) to become `Trusted<T>`; on failure the value is discarded with the object, with no bypass. `crates/gate/src/lib.rs` declares `#[forbid(unsafe_code)]` at the top.
4. **Revertible everything**: anything a plugin registers can be undone in reverse order with no residue (Rust `RevertibleRegistry` / `Gate::revoke()`; TS `ctx.effect / provide / on` + `fiber.dispose()`). Installing/uninstalling plugins takes effect immediately without restarting the process.
5. **Privacy and freedom (PRINCIPLES §6 / SECURITY §3)**: no default collection or upload of **browsing history, page content, clipboard, location, credentials, or device fingerprints**; no telemetry without explicit consent that can be turned off at any time; must not remove the user's ability to uninstall any plugin, swap the engine / network stack / UI, export local data, or **run fully offline**; must not impose "plugins only from one store"; **purely local, zero operating cost, collects no user data, no accounts / cloud sync**.

### Real behaviour of the example accelerate plugin (`packages/plugins/accelerate/src/index.ts`)

- It declares `net.dns` / `net.preconnect`, but **declaration ≠ authorisation**; the Rust gate decides under default deny.
- In code it does only two things: an in-memory `Map` DNS cache (`lookup` / `put` / `size`) and a `ui/link-hover` listener recording the hovered link as `lastHover`. The comment states: "**this only registers intent, it does not open a real connection; the real connection is made by the host network stack after the gate allows it**".
- Red line: does not touch the kernel / VPN / eBPF / BBR, **collects no user data**; on unload it calls `cache.clear()` and sets `lastHover = null`, leaving no residue.
- So: **no real preconnect, no outbound DNS query, no data collection in the current version**.

## 3. What we collect

**Current version: nothing.** Item by item:

- **Not collected**: browsing history, page content, address-bar input, cookies, clipboard, location, credentials, device fingerprint, contacts, photo library, biometrics, crashes, telemetry, usage statistics, install statistics, advertising identifiers.
- **Not present**: accounts, login, cloud sync, cookie sessions (this repository has no server), tracking pixels, ad SDKs, analytics SDKs, automatic crash reporting.
- **Not uploaded**: any data to `codecloud-dev` or any third-party server (there is no server and no backend address).
- **Not stored in a database**: there is no database; the gate, runtime and UI are all in-process local state.

The capability list in `EXTENSIONS.md` marks clipboard / profile / camera / location / telemetry as **privacy red lines, default-deny, requiring per-item explicit user confirmation** — that is a rule, not evidence that anything is currently being collected.

## 4. Retention, location, and sharing

| Item | Where | Notes |
| --- | --- | --- |
| Kernel / gate / runtime / UI state | **Your local process memory** | Disappears on exit; no persisted user data currently |
| DNS cache / hovered link (example accelerate plugin) | In-memory `Map` / local variable | Cleared on plugin unload or process exit, no residue |
| Plugins, config, extensions | Local files (placed/installed by you) | Managed and deleted by you |
| Repository source and docs | GitHub (and a possible Gitee mirror) | Public static files retained in Git history; not user data |
| Your personal data | **None** | No retention, no sharing, no cross-border pipeline |

**Recipients**: currently only the code hosting platforms **GitHub** (and a possible mainland-China read-only **Gitee** mirror) and their CDNs — they produce ordinary access logs only **when you clone/browse the repository**, unrelated to running the browser. In future, when real pages are opened, the websites and third-party plugins will handle the data they receive under their own terms — but that is **user-initiated navigation, their responsibility**, not the shell uploading anything.

We **do not sell, rent, use for advertising, or provide to data brokers** any data; `LICENSE-COMMERCIAL` §6.1 makes "privacy must not be lost" a non-degradable clause that **even a paid commercial licence cannot lift**.

## 5. What future versions may change (honest preview, not shipped features)

- **Opening real web pages**: planned via the system WebView (wry), with Servo as a replaceable engine plugin. Network requests would then arise from **user-initiated navigation**, with target sites handling data under their own terms; the shell still would not upload data by default.
- **Plugins / extensions**: planned per FEP (`EXTENSIONS.md`) and a Chrome / Edge / Firefox MV3 compatibility layer; permissions still go through default deny + least-privilege intersection + per-item user confirmation.
- **Commercial licensing**: planned (README "future commercial licensing", Layer B `LICENSE-COMMERCIAL`). Per `LICENSE-COMMERCIAL` §4.2, the **single channel** for a commercial licence is "log in on the Fenestra official website → sign the 《Fenestra 商业授权电子协议》 (electronic commercial licence agreement) → monthly / annual payment → the system issues a time-limited electronic licence certificate", and it **explicitly is not issued via email, private message, repository Discussion, or any offline communication**. **The current version does not implement** website login, e-agreements, payment, or authorisation checks, and that channel is not yet live, so **no account, order, or payment information is currently collected, no alternative way to obtain a commercial licence exists, and no Layer B commercial use may begin**. If shipped later, the data items (website login records, agreement signing logs, licence certificates, payment records), purposes, retention and sharing will be spelled out field-by-field in an **updated privacy policy** and in `TERMS.md`, with `LICENSE-COMMERCIAL` controlling (see `TERMS.md`).
- None of these changes may violate `PRINCIPLES.md` §6 or `LICENSE-COMMERCIAL` §6 "five must-not-lose"; a version that weakens default deny, sandbox isolation, privacy red lines, or outbound revalidation breaches the licence and users may assert that.

## 6. How to view, reset and delete

- There are no accounts and no cloud data, so **there is no data to ask us to delete**.
- Everything produced locally (plugins, config, extensions, caches) is on your device: delete the files / uninstall the plugin / close the process.
- To verify the shell is not quietly going online, audit `crates/gate`, `packages/core`, `packages/plugins` directly, or observe with OS network tools; `SECURITY.md` and the CI guard scripts (`scripts/check-licenses.sh`, `scripts/check-docs.py`) can be re-run.
- If you cloned the repo or opened an Issue / Discussion on GitHub, that content is handled by **GitHub** under its terms — manage it on the platform.
- **No toggle for "export data" or "clear telemetry" exists in the current version**, because there is no data to export and no telemetry; `PRINCIPLES.md` §6 promises future versions **must not remove** the ability to export local data and run fully offline.

## 7. Children

Not directed at children under 14; no accounts, no age verification, no knowing collection of children's data. The current version has no data entry point; if a future version introduces accounts or payment, a children's clause will be added in the then-current policy.

## 8. Cross-border transfers

There is no data transfer in the current version: source is publicly hosted on GitHub (with a possible Gitee mirror) and readers may be in any country, with the platform distributing these **public static files** under its terms. The browser runtime uploads nothing, so no personal-data cross-border transfer exists. When a user actively visits a foreign site in future, that site handles the data under its terms; the shell makes no promise and performs no proxy transfer.

## 9. Third-party terms (we do not endorse them)

| Service | Terms / privacy policy |
| --- | --- |
| GitHub | `https://docs.github.com/en/site-policy/github-terms/github-terms-of-service` · `https://docs.github.com/en/site-policy/privacy-policies/github-privacy-statement` |
| Gitee | `https://gitee.com/` (see its published terms) |
| Cloudflare (if a future website/licensing service uses it) | `https://www.cloudflare.com/privacypolicy/` |
| System WebView / wry / Servo / Tauri / Svelte / Vite / Cordis and other dependencies | See each project's published licence and terms; dependency licences are checked file-by-file when Servo is actually introduced in Stage 1 (per the README) |

The latest published terms govern. We do not interpret them, guarantee availability, or take responsibility for them.

## 10. Changes and contact

- Changes update this repository's `PRIVACY.md` directly and appear in Git history / CHANGELOG. We collect no contact details, so **we do not push notifications**; significant changes are announced in the repository. If accounts/payment are introduced later, this policy and `TERMS.md` will be updated together.
- **Only contact channels**: open a **GitHub Issue** at `https://github.com/codecloud-dev/<repo>/issues`, or reach the GitHub account **`codecloud-dev`**; these channels are **only for general feedback, security reports and terms enquiries**, and do **not** issue commercial licences. `LICENSE-COMMERCIAL` §4.2 and its "At a glance" section provide that a commercial licence is obtained **only** through website login, signing the e-agreement, payment and a system-issued certificate; email, private messages, Discussions, and offline communication do not issue licences.
- No support email, phone, postal address, company name, or registration number is provided or invented here. Please do not post secrets or credentials in a public Issue.
