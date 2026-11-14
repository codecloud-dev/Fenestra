# 调研：2026 浏览器引擎格局，以及为什么 Hollo 不做引擎

> 调研日期：2026-11-14 · 方式：联网检索引擎对比、WPT 通过率、扩展兼容性资料
> 目的：为「渲染引擎全部能自己改」这句口号找到现实依据，同时防止路线图走偏成"再造一个 Chromium"。

---

## 1. 谁在真正渲染 Web（2026）

| 引擎 | 代表 | 份额（约） | 备注 |
| --- | --- | --- | --- |
| **Blink**（Chromium） | Chrome / Edge / Brave / Opera / Vivaldi / Samsung Internet | ~81% | 事实标准，WPT 约 98% |
| **WebKit** | Safari；欧盟外 iOS 上所有浏览器 | ~14% | WPT 约 96%；iOS 上 Chrome/Firefox 也都是 WebKit 壳 |
| **Gecko** | Firefox 及衍生 | ~3% | WPT 约 96%；Stylo / WebRender 来自 Servo |
| **Servo** | 嵌入式 webview 实验 | ~1% 以下 | 纯 Rust，Linux Foundation Europe；WPT 约 78%，定位嵌入而非日常浏览器 |
| **Ladybird** | 独立 Alpha | — | 从零 C++（LibWeb/LibJS），2026 Alpha / 2027 Beta / 2028 稳定规划；WPT 约 88%（选定领域） |
| **Flow**（Ekioh） | 车载 / 机顶盒 | — | 商业授权，多核并行渲染，不面向消费者 |
| **Goanna** | Pale Moon | — | Gecko 28 分叉，保留 XUL 扩展 |

来源：

- Browser Engines 2026 深度对比 <https://labhub.hopto.org/blog/culture/2026-05-14-browser-engines-2026-chromium-gecko-webkit-servo-ladybird-comparison-deep-dive?lang=en>
- <https://ttlab.hopto.org/blog/culture/2026-05-16-browser-engines-2026-chromium-blink-gecko-webkit-servo-ladybird-libweb-flow-deep-dive?lang=en>
- 浏览器开发参考（引擎格局、进程模型）<https://www.skillsdirectory.com/skills/the-vibey-project-browser-engine-architecture-and-networking>
- Ladybird 架构解析 <https://chenxutan.com/d/4424.html>
- 中文格局分析 <https://blog.csdn.net/Frag0ut/article/details/167036299>

---

## 2. 两个必须记住的现实

**a) 从零造引擎极难，且不是 Hollo 的战场。**

浏览器引擎是数千万行、二十年兼容性长尾、国家级攻击者关注的产物。过去二十年没有人从零造出面向大众的竞争性引擎并交付；Ladybird 用约 10 名全职工程师、2026 Alpha / 2028 稳定的节奏在试，尚且把沙盒、GPU 隔离、Wasm GC 明确列为 **roadmap 而非已交付**。行业共识是"forking is the rational choice"——过去十年几乎每一款"新浏览器"都是 Chromium 壳换 UI 与策略。

> 这一条直接决定了 Hollo 的路线：**不自研引擎**。Stage 1 先把成熟引擎以插件形式接进来。

**b) 但"壳"这件事本身有价值，而且 iOS 上早就是常态。**

欧盟之外，iOS 上所有浏览器（包括 Chrome、Firefox、Edge）都必须是 WebKit 壳。这说明"引擎由平台给定、产品层由自己做"是被大规模验证过的形态。Hollo 的差别在于：把这个事实**显式化、通用化、可替换化**——不是被平台逼着只能用一个引擎，而是把引擎做成清单里可以换掉的一行，桌面端能同时装 Blink / Gecko / WebKit 适配器并按站点或按用户切换。

---

## 3. 兼容性长尾在哪里（决定了 Stage 1 的验收不能太轻）

| 引擎族 | 常见踩坑 |
| --- | --- |
| Safari / WebKit | 无 File System Access API；IndexedDB 事务锁冲突；PWA / Service Worker 存储与推送滞后；媒体编解码器偏好 H.264/HEVC |
| Firefox / Gecko | 激进反跟踪按设计就会打破依赖跟踪器的站点；WebGPU / OffscreenCanvas 比 Blink 晚半年到一年；移动端份额小导致站点基本不测 |
| Chromium | MV3 削弱了高级内容拦截与自动化扩展；Origin Trials 过期会静默失效；ChromeOS / Android WebView / Electron 之间有细微差异 |

一个小团队的最小合理测试矩阵是：Chrome 桌面 + Safari iOS（P0），Chrome Android + Safari macOS（P1），Firefox 桌面（P2，跨引擎兼容的金丝雀），Edge 桌面（P3）。**Hollo 既然承诺三平台与可扩展，Stage 1 的验收就必须包含至少一个非 Chromium 引擎的适配验证**，否则"引擎可换"只是一句类型定义。

---

## 4. 扩展生态的现实（2026）

三家都基于 WebExtensions，但方言不同：

- Chrome / Edge：MV3 only，service worker 后台约 30 秒空闲回收，`declarativeNetRequest` 静态规则约 300K 条；
- Firefox：MV3 + 部分 MV2，`background.scripts` 事件页更稳妥，仍保留部分 `webRequest` 动态拦截，DNR 约 150K 条；
- 命名空间：`browser.*` + Promise 是 Firefox 原生，Chromium 系到 **2026 年中**才全量补齐 `browser.*` 与 Promise；
- AMO 要求 `browser_specific_settings.gecko.id` 与 `data_collection_permissions`（桌面 ≥140 / Android ≥142），缺了 `web-ext lint` 直接失败；
- Firefox Android 不支持 omnibox 等 API，勾选移动端兼容会静默坏掉核心功能。

资料：

- MDN / extensionworkshop 浏览器兼容性 <https://extensionworkshop.com/documentation/develop/browser-compatibility/>
- Cross-Browser Extension Development 2026 <https://cwskit.khanakia.com/blog/cross-browser-extension-development>
- 一份 MV3 扩展发三家的实操记录 <https://dev.to/aialleyway/shipping-one-manifest-v3-extension-to-chrome-edge-and-firefox-from-a-single-source-cep>
- Chrome vs Firefox vs Edge 差异 <https://metadesignsolutions.com/blog/cross-browser-extension-development-chrome-vs-firefox-vs-edge>

Hollo 归一器把这些差异**逐条写成 `warnings`**，见 [EXTENSIONS.md](../../EXTENSIONS.md)。假装差异不存在，才是"完全兼容"里最危险的做法。

---

## 5. 由此得出的四条架构决策

| 决策 | 依据 |
| --- | --- |
| **不自研引擎**，Stage 1 接成熟引擎适配层 | 引擎是数千万行 / 二十年长尾，Ladybird 的节奏就是现实参照 |
| **引擎是插件，且可并列安装** | 桌面端没有 iOS 的 WebKit 强制约束，多引擎切换是壳能提供的增量价值 |
| **`RenderEngine` 只定义 `render` 与 `name`** | 内核不该理解布局、DOM、编解码，否则就被某一个引擎的形状绑住 |
| **兼容层在内核之上** | 第四家扩展格式出现时改插件，不动内核；内核只认 HEP |

---

## 6. 一句诚实结论

2026 年的引擎格局是 **Blink 约 81% + WebKit 约 14% + Gecko 约 3%**，新引擎还在 Alpha。Hollo 的机会不在"再做一个引擎"，而在"把引擎、网络栈、界面、扩展运行时全部降格为可替换插件，并且把这件事做得足够小、足够快、足够安全"。

壳要出名，靠的是**可替换性 + 好用**，不是靠重写 Blink。
