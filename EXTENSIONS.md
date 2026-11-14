# 扩展：三家完全兼容，自有更好

## 1. 目标

> Chrome / Edge / Firefox 三类扩展完全兼容；我们有自己的扩展，比这些兼容的更好用。

这里「完全兼容」的准确含义是：**三家现有的 `manifest.json` 都能被读成同一份 HEP 清单，进入同一套能力仲裁，挂到同一个插件体系里**；已知方言差异由归一器**显式列出**，而不是被悄悄抹平后装作不存在。

"自家扩展更好"的准确含义见第 4 节：能力是声明式的，安装时就算好最小权限交集，不需要猜权限字符串的含义，也不需要运行时再去问一遍。

---

## 2. 三家差异（2026 年联网核查，不藏）

| 维度 | Chrome | Edge | Firefox |
| --- | --- | --- | --- |
| Manifest | MV3 only | MV3 only（Chromium） | MV3，且仍兼容部分 MV2 |
| 后台 | `background.service_worker`，约 30s 空闲即回收 | 同 Chrome | 更稳妥的是 `background.scripts` 事件页，可选持久后台 |
| 网络拦截 | `declarativeNetRequest` 静态规则，约 300K 条 | 同 Chrome | 仍支持部分 `webRequest` 动态拦截，DNR 约 150K 条 |
| 命名空间 | `chrome.*`，2026 年中起全量支持 `browser.*` + Promise | 同 Chrome | `browser.*` + Promise 原生，另有 `chrome.*` 回调别名 |
| 商店要求 | 签名、审核 1–3 天 | 同包可上传，另有 `update_url` 与独立审核 | 需 `browser_specific_settings.gecko.id`；`data_collection_permissions`（桌面 ≥140 / Android ≥142）；`web-ext lint` |
| 移动端 | Chrome Android | — | Firefox Android 不支持 omnibox 等 API |

资料出处与原文摘录见 [`docs/research/browser-engines-2026.md`](docs/research/browser-engines-2026.md) 与 MDN / extensionworkshop / Thoughtworks 的公开资料。

**这些差异不可能靠"写一次到处跑"消失**。所以归一器的输出里带 `warnings: Vec<String>`，把降级、缺字段、命名空间差异全部说清楚。开发者看到的是事实，不是惊喜。

---

## 3. 归一器怎么工作

[`src/compat.rs`](src/compat.rs) 做了两件事，都是自研、零依赖：

**3.1 迷你 JSON 解析器**：支持 object / array / string（含 `\uXXXX` 与 UTF-8）/ number / bool / null，错误带位置。为什么不用现成库？因为内核要在任何 JS 运行时启动之前读清单，而且 `[dependencies]` 必须为空（CI 有守门）。

**3.2 方言识别与权限映射**：

```
browser_specific_settings.gecko / applications.gecko  → Firefox
update_url 含 edge.microsoft.com                      → Edge
其它                                                  → Chrome
```

权限映射表（节选）：

| 权限字符串 | 映射到的能力 |
| --- | --- |
| `storage` / `unlimitedStorage` / `bookmarks` | `storage.read` + `storage.write` |
| `tabs` / `activeTab` / `scripting` | `ui.content` |
| `clipboardRead` / `clipboardWrite` | `clipboard.read` / `clipboard.write` |
| `webRequest` / `webRequestBlocking` / `declarativeNetRequest*` | `net.intercept`（安全红线） |
| `history` / `cookies` | `profile.read`（隐私红线） |
| `geolocation` | `device.location`（隐私红线） |
| `notifications` / `alarms` / `contextMenus` / `sidePanel` | `ui.chrome` |
| `host_permissions` / `<all_urls>` | `origin` 列表 |
| 未知权限 | **不授予任何能力**，并写入 `warnings` |

最后一条很关键：遇到不认识的权限，归一器**默认拒绝**并报告，而不是"先给了再说"。

`to_hep()` 可以把归一结果写回纯文本 HEP，人能直接读懂、手工改。测试 `all_three_flavors_produce_installable_hep_manifests` 验证三家样例都能走完 `manifest.json → HEP → Manifest::parse → Shell::install` 全链路。

```bash
cargo run -- compat examples/extensions/firefox.json
```

---

## 4. HEP 自有扩展为什么更好

| 痛点（三家兼容层） | HEP 自有扩展 |
| --- | --- |
| 权限是字符串，含义靠文档约定，还会随版本变 | 能力是内核封闭枚举，未知即拒绝 |
| 安装时不算权限，运行时才拦截 | `Grant::intersect` 在安装时求最小权限交集 |
| 后台模型三家不同，状态容易被回收 | 生命周期由可逆注册管理，卸载精确撤销 |
| 沙盒策略由浏览器决定 | `sandbox-policy` 也是可换插件，`harden()` 可冻结能力 |
| 扩展只能改页面／UI | 渲染引擎、网络栈、UI、协议处理都是同一套插件体系 |
| 清单差异要靠构建期多份 manifest | 一份纯文本清单，跨平台、可审计 |

一句话：兼容层负责"把老世界接进来"，HEP 负责"让新东西不必背老世界的包袱"。

---

## 5. 目前还做不到的事

诚实清单，避免被当成已完成：

- [ ] **执行扩展代码**：归一器只读清单、只算能力，不加载 wasm、不启动 JS 运行时。真正的执行在 `ExtensionRuntime` 插件里（Stage 1）。
- [ ] **API shim**：`chrome.*` / `browser.*` 的运行时垫片尚未实现，需要 Promise 统一层。
- [ ] **DNR 规则转换**：`declarativeNetRequest` 规则集到 Hollo 能力模型的转换尚未实现。
- [ ] **商店与签名**：没有插件商店、没有签名验证、没有来源审计（Stage 2）。
- [ ] **移动端 API 差异**：Firefox Android 不支持 omnibox 这类差异目前只在 warnings 里提示，没有运行时降级。

---

## 6. 写一个 HEP 扩展

最小可用清单：

```text
# 注释：一个只读页面标注插件
id: com.example.annotate
name: Page Annotator
version: 1.0.0
kind: other
entry: annotate.wasm
capability: ui.content
capability: storage.read
origin: https://example.com/*
```

```bash
cargo run -- verify examples/annotate.hep
```

规则见 [ARCHITECTURE.md 第 3 节](ARCHITECTURE.md#3-hep-清单格式)。注意：声明了能力不代表拿到能力，用户授权与运行时仲裁各过一次。
