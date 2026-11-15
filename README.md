# Fenestra

> **只是个壳，其余皆插件。**
> *The browser is a shell. Everything else is a plugin.*

一个特别小、特别简单、跨手机与桌面的浏览器**空壳**。正常该有的都能装上，但没有任何一样是焊死在内核里的：渲染引擎、网络栈、地址栏、标签页、书签、历史、下载、设置页、主题、协议处理、扩展运行时，甚至沙盒策略本身，都是清单里可以换掉的一行。

- **公开仓库**：[github.com/codecloud-dev/Fenestra](https://github.com/codecloud-dev/Fenestra)
- **许可**：自撰 [Fenestra Source License v1.0](LICENSE) — 源码可见、**非** OSI 开源；自己商用免费，别人商用需小额付费授权，改了必须公开。
- **当前阶段**：**Stage 0 · 内核骨架**（见下方「现在到底做到哪一步」，不吹牛）

---

## 现在到底做到哪一步

这一步必须说清楚，免得被当成"又一个画饼的浏览器"：

| 已经真实存在并跑通 | 还没有 |
| --- | --- |
| 零第三方依赖的 Rust 内核（[`Cargo.toml`](Cargo.toml) 里 `[dependencies]` 是空的） | 能打开网页的图形界面 |
| 自研 FEP 清单格式 + 解析器 | 真实渲染引擎插件（没有内置 Blink/WebKit/Gecko） |
| 能力模型、默认拒绝、隐私红线显式确认 | 平台级进程沙盒（只有策略层与仲裁层） |
| 可逆注册：安装＝登记可撤销效应，卸载＝精确撤销 | Android / Windows / Linux 安装包 |
| Chrome / Edge / Firefox `manifest.json` → FEP 归一器（自带迷你 JSON 解析器） | 扩展商店、插件签名与分发 |
| 39 个测试（33 单测 + 6 集成）全绿，`cargo fmt` / `clippy -D warnings` 干净 | 性能基准与真机流畅度数据 |
| 三平台 CI + Android/iOS target 编译检查 | — |

**换句话说**：现在交付的是「壳」的内核与它的可验证契约，不是能日常上网的浏览器。Stage 1 才是把引擎插件接上并出可运行界面。路线图见 [ROADMAP.md](ROADMAP.md)，每条都写了验收方式，不写"预计很快"。

---

## 为什么叫 Fenestra

拉丁语 *fenestra*，意思是「窗」。浏览器本来就是一扇窗：窗框得够简、够硬、够好用，但窗外看到什么、怎么渲染、怎么交互，不该由窗框决定。

比「空壳」更准一点——壳强调的是里面没有东西，窗强调的是**你透过它看到的世界由插件提供**。Fenestra 只负责窗框，剩下的全是风景。

Fenestra 的内核只有三件事：

1. **读清单** — 插件声明自己是谁、想要什么能力；
2. **仲裁能力** — 默认拒绝，隐私红线要用户逐项点头；
3. **可逆登记** — 安装 = 登记一组可撤销效应，卸载 = 精确撤销，不留悬空状态。

除这三件事以外，内核里**没有**渲染、没有网络、没有界面、没有遥测。这不是简化，是设计目标：任何你不喜欢的部分，都可以拔掉换掉。

---

## 比「一切皆插件」更绝在哪

联网核查了 **DSH（DeepSeek Harness）** 的官方仓库与架构资料，结论在 [`docs/research/dsh-everything-is-a-plugin.md`](docs/research/dsh-everything-is-a-plugin.md)。先给 DSH 应得的评价：它把模型适配器、工具注册表、会话日志、agent loop、系统提示词、执行沙箱、UI 全做成了插件，靠 Cordis 的可逆 effect 做生命周期管理，架构确实干净，MIT 协议也大方。

但它的「绝」有一条边界：**插件化止步于应用层能力，运行时本身是给定的**。Node/TypeScript/Electron、Cordis 内核、进程模型、宿主 OS 沙盒都不在可替换清单里。而且官方自己标明是 *developer preview*、"THERE WILL BE COMPATIBILITY-BREAKING CHANGES"。

Fenestra 想再往前推一格：

| 维度 | DSH | Fenestra |
| --- | --- | --- |
| 内核职责 | Cordis：插件加载、依赖、事件、可逆 effect | 清单 + 能力仲裁 + 可逆登记（不绑定任何运行时） |
| 可替换的是 | Agent 的应用层能力 | **渲染引擎、网络栈、平台后端、UI、扩展运行时、沙盒策略** |
| 运行时 | TypeScript / Node / Electron 给定 | 内核零依赖，平台与运行时都由插件带进来 |
| 兼容承诺 | 开发者预览，允许破坏性变更 | 三类扩展兼容层 + 冻结内核 ABI（Stage 1 起） |
| 安全 | 执行沙箱是插件之一 | 对标 Apple containment：默认拒绝 + 隔离单元"引爆" + 出站数据重校验 |
| 许可 | MIT（谁都能拿去商用） | 自撰源码可见：作者可商用，他人商用需小额付费，改动必须公开 |

还有一句实话：**DSH 有成熟的插件市场和社区，Fenestra 现在只有 39 个测试**。架构上更绝，不等于产品上更成熟。这两件事别混为一谈。

---

## 五不丢

用户反复强调的五条，不是口号，写进了 [许可证第 6 条](LICENSE) 和 [PRINCIPLES.md](PRINCIPLES.md)，并且尽量落成可验证的代码：

| 原则 | 代码里的落地点 |
| --- | --- |
| **自由性** | 引擎/网络栈/UI/扩展运行时全是 trait，注册表支持 `uninstall` 精确撤销；许可证禁止单一商店锁死 |
| **隐私性** | `telemetry` / `profile.read` / `clipboard.read` / `device.*` 为隐私红线，未经用户显式确认永不放行；`Grant::sanitized()` 可直接剔除 |
| **快捷性** | 清单是纯文本、逐行、零依赖解析；内核不启动任何运行时、不联网、不等待 |
| **快速性** | 内核零第三方依赖，CI 强制 `cargo clippy -D warnings`；许可证禁止人为限速／后台常驻 |
| **流畅度** | 引擎通过 trait 可换，渲染路径不由内核介入；许可证禁止为商业目的故意降速 |

---

## 快速开始

```bash
git clone https://github.com/codecloud-dev/Fenestra.git
cd fenestra

cargo test --all-targets          # 39 个测试
cargo run -- demo                 # 内核自检演示
cargo run -- policy               # 打印默认沙盒策略
cargo run -- targets              # 支持平台与 target triple
cargo run -- verify examples/engine.fep
cargo run -- compat examples/extensions/firefox.json
```

`cargo run -- demo` 会真实跑一遍：安装渲染引擎／界面／扩展运行时三个插件 → 列出服务与订阅 → 演示引擎 trait 调用 → 能力仲裁（未授予一律拒绝）→ 锁定模式冻结 `engine.swap` → 三家扩展清单归一 → 卸载后服务与订阅一起消失。

一份 FEP 清单长这样（[`examples/engine.fep`](examples/engine.fep)）：

```text
# 渲染引擎也是一个插件
id: org.fenestra.engine.blink
name: Blink Engine Adapter
version: 0.1.0
kind: render-engine
entry: engines/blink.wasm
capability: engine.swap
capability: net.connect
```

---

## 扩展：三家完全兼容，自有更好

| 来源 | 归一方式 | 已知差异（不藏） |
| --- | --- | --- |
| Chrome MV3 | `manifest.json` → FEP | 阻塞式 `webRequest` 已被 `declarativeNetRequest` 取代（约 300K 规则） |
| Edge MV3 | 同一 MV3 包，按 `update_url` 识别 | 商店审核与 `update_url` 不同，能力集与 Chrome 一致 |
| Firefox MV3 | 按 `browser_specific_settings.gecko` 识别 | 事件页 `background.scripts` vs service worker；AMO 要求 `data_collection_permissions`（≥142）；仍保留部分 `webRequest` 动态拦截 |
| **FEP（自有）** | 原生格式，无需转换 | 能力声明在清单里，安装时即求最小权限交集；不用猜权限字符串的含义 |

"完全兼容"不是说这些差异不存在，而是说**差异被归一器显式列成 warnings，而不是悄悄抹平**。详见 [EXTENSIONS.md](EXTENSIONS.md)。

---

## 平台

| 平台 | 内核状态 | 说明 |
| --- | --- | --- |
| Linux | CI 真跑测试 | 一等公民 |
| Windows | CI 真跑测试 | 一等公民 |
| macOS | CI 真跑测试 | 一等公民 |
| Android | `aarch64-linux-android` 编译检查 | 平台差异由 `PlatformBackend` 插件承担 |
| iOS | `aarch64-apple-ios` 编译检查（macOS runner） | 同上；引擎选择受系统规则约束，见 ROADMAP |

内核是纯 `std`、零依赖，所以「一套内核、五个 target」不是口号，是 CI 里能查的东西。

---

## 安全

对标的是 Apple 在 Messages / Safari 上的 containment 思路：**把不可信数据的解析放进几乎无权限的隔离单元里"引爆"，并且不信任它返回的东西**。Fenestra 里对应 `SandboxPolicy::detonate()` 与 `Untrusted::revalidate()`，返回类型是刻意设计的类型级提醒——调用方拿不到裸值，必须显式重新校验。

调研与出处见 [`docs/research/apple-sandbox.md`](docs/research/apple-sandbox.md)，威胁模型与报告渠道见 [SECURITY.md](SECURITY.md)。

诚实边界：**Stage 0 只有策略层与仲裁层，还没有 OS 级进程隔离实现**。策略写得再对，不等于沙盒已经落地。

---

## 许可证，一句话说清

- 你（作者）可以随便商用，免费，不用跟谁解释；
- 别人**默认不能商用**；要商用可以，**付一笔不大的钱给作者**；
- 别人**改了必须公开**完整对应源码，付了钱也一样；
- 无论谁改，都**不能**锁死插件、偷传数据、弱化沙盒、人为变慢、阻断三类扩展兼容。

完整条款见 [LICENSE](LICENSE)。**这不是 OSI 开源许可证**，请不要这样称呼它；理由写在协议第 0 条。

---

## 文档索引

| 文档 | 内容 |
| --- | --- |
| [ARCHITECTURE.md](ARCHITECTURE.md) | 内核三件事、FEP 格式、能力模型、可逆注册、设计决策 |
| [SECURITY.md](SECURITY.md) | 威胁模型、沙盒设计、漏洞报告 |
| [EXTENSIONS.md](EXTENSIONS.md) | 三类扩展兼容矩阵与 FEP 自有格式 |
| [PRINCIPLES.md](PRINCIPLES.md) | 五不丢如何变成可验收条款 |
| [ROADMAP.md](ROADMAP.md) | Stage 0–3 与每条的验收方式 |
| [CONTRIBUTING.md](CONTRIBUTING.md) | 贡献方式与许可边界 |
| [CHANGELOG.md](CHANGELOG.md) | 变更记录 |
| [docs/research/](docs/research/) | DSH、Apple 沙盒、2026 引擎格局的联网调研 |
| [README.en.md](README.en.md) | English overview |
