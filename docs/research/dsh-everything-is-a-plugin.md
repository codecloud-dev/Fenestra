# 调研：DSH 为什么敢说「一切皆插件」

> 调研日期：2026-11-14 · 方式：联网检索 + 官方仓库页面核对 + 第三方架构资料交叉验证
> 目的：搞清楚「一切皆插件」到底指什么、边界在哪，以及 Hollo 要比它「更绝」究竟该往哪使劲。
> 本文只做技术对比，不对项目作价值评判。

---

## 1. DSH 是什么

**DSH = DeepSeek Harness**，DeepSeek AI 开源的 **Agent 运行框架（harness）**，不是模型、不是聊天工具。官方仓库标题就是 `DeepSeek Harness: Everything is a Plugin`。

核心公式是 `Agent = Model + Harness`：模型负责理解与推理，Harness 负责把模型、工具、界面、记忆组装成能干活的 Agent。2026 年 8 月发布开发者预览并开源，**MIT 协议**。

主要形态：自托管 Web 应用、Electron 桌面应用、TypeScript / Python / ACP 三种 SDK、CLI。

来源：

- 官方仓库 <https://github.com/deepseek-ai/deepseek-harness>（页面原文含 "It is built on an **everything-is-a-plugin** architecture and powered by [Cordis]"）
- 官方架构论文 *A Programming Paradigm for Spatiotemporal Composability* <https://arxiv.org/abs/2608.25512>
- DSH 插件社区目录 <https://dsh-plugin.org/zh>、<https://dshplugin.org/>
- Thoughtworks Technology Radar <https://www.thoughtworks.com/en-de/radar/tools/deepseek-harness-dsh>
- 架构拆解 <https://kongchang.com/en/articles/deepseek-harness-deep-dive-everything-is-a-plugin>
- 中文解读 <https://blog.csdn.net/2301_79034198/article/details/169365336>、<https://www.yesdotnet.com/archive/post/863977790582853.html>

---

## 2. 「一切皆插件」的确切含义

不是修辞，是字面意思。以下组件**都是插件**：

| 类别 | 具体组件 |
| --- | --- |
| 循环与调度 | agent loop 本身、工具执行层、调度 |
| 模型 | 模型适配器（DeepSeek / Kimi / GLM / 任意 OpenAI、Anthropic 兼容服务） |
| 能力 | 工具注册表、bash 工具、文件系统、Skills、MCP client |
| 状态 | 会话日志与存储、记忆与上下文 |
| 界面 | Web UI、Electron 桌面界面、TUI、侧边栏、皮肤 |
| 隔离 | bash sandbox、filesystem sandbox、subprocess 执行插件 |

最直观的证明是 **Creator mode**：因为「一切皆插件」，Agent 可以修改自己——让它"写一个注册 hello world 工具的插件"，它会生成插件目录并在新会话里调用成功。也有演示用自定义 agent loop 完全不调用模型，只回显用户输入。

社区侧把插件按界面体验／会话消息／记忆上下文／工具能力／技能智能体／工作流等十多个大类组织，官方建议给插件仓库加 `dsh-plugin` topic 以便发现。

---

## 3. 它凭什么做得到：Cordis

Cordis 是底层插件内核，**不是 DeepSeek 从零写的**——它的首次提交比 DeepSeek 成立还早约一年，作者后来加入 DeepSeek 并把它作为 DSH 的基础架构。Cordis 仓库不属于 DeepSeek，但把 DSH 官网列为官方项目页，论文由 DeepSeek 发表。

三个关键机制：

1. **插件导出 `apply(ctx)` 注册能力**，通过共享 context、依赖注入与事件协作；
2. **可逆 effect（reversible effects / spatiotemporal composability）**：插件的贡献登记为可撤销的注册，卸载／重载时能被精确 unwind，不留悬空状态；
3. **时空可组合性**：这是论文标题里的核心词，也是"开发者预览也能频繁重构"的底气。

配置层用 YAML profile（命名插件清单）做选择与覆盖，开发者不需要改 DSH 源码就能换模型适配器、换沙箱实现、定制 UI。`dsh web --patch` + `cordis.patch.yml` 可以在运行时加载自定义插件。

还有一点常被忽略：DSH 里其实有**两个 Cordis 应用**——后端与前端（Electron 界面本身也是一个 Cordis 应用），两者通过通信协作。

---

## 4. 边界：「绝」到哪里为止

这是 Hollo 最该看清楚的部分。**DSH 的插件化止步于应用层能力，运行时本身是给定的**：

| 不可替换的部分 | 说明 |
| --- | --- |
| TypeScript / Node / Electron 运行时 | 插件用 `apply(ctx)`、npm 包分发，宿主是既定的 |
| Cordis 内核 | 加载／依赖／事件／可逆 effect 的实现是固定底座 |
| 语言与执行模型 | JS 单线程事件循环 + 特定 SDK 形态 |
| 进程模型与宿主 OS 沙盒 | 由 Electron / Node 决定，不在插件清单里 |
| 会话日志的 append-only 语义 | 被当作一等设计约束内置 |

另外，官方 README 明确写着：

> "DeepSeek Harness is in *developer preview* and iterating rapidly. **THERE WILL BE COMPATIBILITY-BREAKING CHANGES.**"

也就是说，它的插件 API 稳定性目前不是承诺。仓库提交量很大（页面显示 27,219 commits，2026-11-13 仍在活跃合并），迭代确实快。

还有一个诚实观察：DSH 的"沙箱是插件"指的是**工具执行环境可替换**（bash sandbox / filesystem sandbox / 远程执行），不是**浏览器渲染进程的隔离边界**。两者名字都叫 sandbox，层次完全不同。

---

## 5. Hollo 要怎么「更绝」

| 维度 | DSH | Hollo 的做法 |
| --- | --- | --- |
| 插件边界 | 应用层能力 | 连**渲染引擎、网络栈、平台后端、UI、扩展运行时、沙盒策略**都是插件 |
| 内核依赖 | Cordis + TypeScript 生态 | **零第三方依赖**的纯 `std` Rust，清单纯文本自解析 |
| 运行时 | 给定 Node/Electron | 内核不绑定任何运行时，由插件带进来 |
| 兼容承诺 | 开发者预览，允许破坏性变更 | 冻结 HEP v1 ABI；Chrome/Edge/Firefox 三类扩展归一兼容 |
| 安全地基 | 工具执行沙箱是插件之一 | 对标 Apple containment：默认拒绝 + 隔离单元 detonate + 出站重校验 |
| 商业条款 | MIT，谁都能拿去商用 | 自撰源码可见：作者可商用，他人商用小额付费，改版必须公开 |

具体落到代码：

- `PluginKind` 枚举里直接列出 `render-engine` / `network-stack` / `sandbox-policy`；
- [`src/engine.rs`](../../src/engine.rs) 只有 trait，没有任何实现；
- `Shell` 只做「读清单 → 能力仲裁 → 可逆登记」三件事；
- `SandboxPolicy::weakened()` 让"沙盒策略被换弱"成为可检测事实，而不是自由到没人管。

---

## 6. 反面：必须承认的差距

写得再漂亮也别自欺：

| DSH 有 | Hollo Stage 0 没有 |
| --- | --- |
| 真实可用的产品（Web / 桌面 / CLI / 三种 SDK） | 只有 CLI demo，不能上网 |
| 活跃插件市场与社区分类体系 | 只有 3 个内置示例清单 |
| 27k+ 提交、持续迭代的成熟度 | 首日仓库 |
| MIT 带来的低门槛生态 | 自撰许可会明显限制商业参与（这是有意取舍，但要认） |
| 成熟的可观测性（append-only session log） | 没有 |
| 论文与工业界背书（Thoughtworks Radar） | 无 |

**结论**：Hollo 在**架构边界**上比 DSH 更绝，在**产品成熟度**上差着几个数量级。README 里那句"架构上更绝，不等于产品上更成熟"不是客套，是事实陈述。

---

## 7. 可以借鉴但不照抄的

- ✅ 借 **可逆 effect** 的思路 → [`Registry`](../../src/registry.rs) 的精确撤销，但地基换成能力仲裁；
- ✅ 借 **配置即插件清单** 的思路 → HEP 纯文本清单；
- ✅ 借 **topic 发现机制** 的思路 → 将来给 HEP 插件仓库定 topic；
- ❌ 不借 **MIT**：按项目愿景需要商业保护；
- ❌ 不借 **Node/Electron 绑定**：跨平台且要特别小，运行时必须由插件决定；
- ❌ 不借 **"开发者预览可以随便破坏"**：兼容三类扩展是硬承诺，ABI 要冻结。
