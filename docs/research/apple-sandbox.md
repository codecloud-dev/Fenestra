# 调研：对标「苹果的安全」，具体对标什么

> 调研日期：2026-11-14 · 方式：联网检索 Apple 开发者资料、WWDC 讲稿转录、iOS 固件差异分析、Safari/WebKit 安全审计
> 目的：把一句"对标苹果的安全"拆成可落地的工程决策，并划清"哪些我们能做、哪些根本不是浏览器壳能做的"。

---

## 1. 核心思路：Containment（围堵），不是"没有漏洞"

Apple 在 WWDC 的安全讲稿里把防线分成几层：用内存安全语言（Swift）让 bug 不可能、缩减攻击面让漏洞够不着、用缓解措施让漏洞不可利用，最后一层是 **containment——即使被攻陷，也要把攻击者困住**。

原文要点（Apple Developer "Protect your app with Enhanced Security" / "Fortify your app"）：

> "Apple has designed a multi-process architecture that moves processing of complex attacker-supplied data into a heavily sandboxed environment with almost no access to system resources such as other services or even the kernel. ... even in the event of a successful compromise, traps the attacker in an environment that's devoid of assets like messages or cookies."

> "The security policy is simple. Treat all assets that are received as malicious until they've been parsed and broken down into primitive types or detonated inside the blast door."

最关键、也最容易被忽略的是这一句：

> "we should consider that security containment process might be compromised. And so any data leaving it and being returned to Messages isn't trustworthy."

也就是说：**隔离单元的输出仍按敌意数据处理**。Messages 在显示图片之前，会用极小的校验器重新确认"这确实是我期望格式的合法图片"。

来源：

- Apple Developer, *Protect your app with Enhanced Security* <https://developer-mdn.apple.com/videos/play/meet-with-apple/278/?time=221>
- 讲稿转录 *Fortify your app* <https://nonstrict.eu/wwdcindex/meet-with-apple/265/>
- iOS 26.5.2 固件差异分析（BlastDoor / CoreTrust / WebKit 同步更新）<https://onejailbreak.com/blog/ios-26-5-2-deep-analyze/>
- Safari 浏览器安全审计报告（2026）<https://tsight.io/articles/17775406>
- Lockdown Mode 与 WebKit JIT <https://digitash.com/apple/ios/configure-lockdown-mode-system-hardening-sandboxing-ios>

---

## 2. 具体机制清单

| 机制 | 做什么 | 层次 |
| --- | --- | --- |
| **App Sandbox** | 应用默认只能访问自身数据与被授予的系统服务 | OS |
| **BlastDoor** | Messages 里解析不可信附件／内容的低权限隔离服务 | 进程 + OS |
| **Enhanced Security Extensions**（Xcode 26） | 把上述 containment 模式开放给第三方 App | 框架 |
| **出站数据重校验** | 高权限侧不信任 containment 返回值，只校验简单类型 | 设计约定 |
| **多进程 / 站点隔离** | 每页独立进程；WebKit2 有 Site Isolation | 进程 |
| **Lockdown Mode 禁 JIT** | WebKit 去掉 JIT，消除 W^X 违规带来的 ROP 面 | 引擎 |
| **MIE / PAC / 代码签名** | 内存完整性、指针认证、AMFI/CoreTrust | 硬件 + 内核 |
| **ITP** | 概率模型限制第三方 Cookie 跨站跟踪 | 策略 |

2026 年的一个重要信号：Apple 把"内存完整性强制（Memory Integrity Enforcement）"称为消费级 OS 历史上最重要的内存安全升级，但仍明确说"再强的缓解也可能被绕过，所以最后一道防线是 containment"。**这个"不赌不出漏洞"的态度，比任何单个技术都值得抄。**

---

## 3. 批判性一面：别把「苹果安全」当神话

Safari 安全审计报告（2026）提了几点值得记下：

- "围墙花园"把安全责任几乎全转移给 Apple，用户失去独立审计与数据控制权；
- WebKit 沙箱仍出现过逃逸漏洞，多进程隔离不等于不可逃逸；
- 闭源生态让外部审计困难。

Hollo 的立场应该是：**抄它的 containment 思路，不抄它的封闭**。代码公开、清单可读、策略可 `weakened()` 检测，这恰恰是 Apple 模式给不了的东西。

---

## 4. 映射到 Hollo

| Apple | Hollo 对应 | 状态 |
| --- | --- | --- |
| 把不可信数据放进低权限单元 detonate | `SandboxPolicy::detonate()` | ✅ Stage 0（策略层） |
| 输出仍不可信 | `Untrusted<T>` + `revalidate()` | ✅ Stage 0（类型级约束） |
| BlastDoor 隔离进程 | `IsolationLevel::BlastDoor` + `PlatformBackend` 插件 | ⚠️ 只有策略声明，真实进程在 Stage 2 |
| App Sandbox 默认拒绝 | `default_deny` + `Capability` 三重闸门 | ✅ Stage 0 |
| 锁定模式禁 JIT | `deny_jit_in_lockdown` + `harden()` | ✅ 策略层；引擎落地在 Stage 2 |
| Enhanced Security Extension 可替换 | `PluginKind::SandboxPolicy` | ✅ 沙盒策略本身是插件 |
| 多进程／站点隔离 | `IsolationLevel::Process` / `Site` | ⚠️ Stage 2 |
| MIE / PAC / AMFI / 代码签名 | — | ❌ 不属于浏览器壳能做到的事 |
| ITP | — | ⬜ Stage 2 以插件形式做，不进内核 |

### 代码里的三处刻意设计

**a) 类型级不可信**：`detonate()` 返回 `Untrusted<T>` 而不是 `T`。约定会被忘记，类型不会——调用方拿不到裸值，必须写 `revalidate()`。

```rust
let parsed = policy.detonate(&bytes, |b| Some(b.len())).expect("解析成功");
assert_eq!(parsed.revalidate(|n| *n == 4), Some(4));
assert_eq!(parsed.revalidate(|n| *n < 4), None);
```

**b) 锁定模式冻结边界**：`engine.swap` / `network.swap` / `sandbox.read` / `net.intercept` 是安全红线。攻击者拿到执行权后最想做的两件事就是"换引擎"和"改沙盒策略"，`harden()` 直接把它们冻住，而普通 `net.connect` 仍可用，不至于把浏览器变砖。

**c) 策略削弱可检测**：`weakened()` 检查 `default_deny`、出站重校验、隔离级别。因为沙盒策略本身是插件，必须有个办法判断"换上来的那个是不是把门拆了"。这也对应 [LICENSE 第 6.2 条](../../LICENSE)。

---

## 5. 明确做不到的部分（写下来，免得以后吹牛）

1. **硬件级内存安全**：MIE / PAC 依赖 Apple 芯片与内核，Hollo 只能用 Rust 降低内存错误，不能声称等价。
2. **代码签名与 AMFI**：插件签名在 Stage 2 做，但强度不等同于 OS 级强制签名。
3. **真正的进程隔离**：Stage 0 的 `IsolationLevel` 只是**策略枚举**，不创建进程、不调用 seccomp/AppContainer/Seatbelt。谁把这两者混为一谈，谁就是在发假绿卡。
4. **引擎内部沙盒**：Blink/WebKit/Gecko 自身的渲染器沙盒由引擎插件负责，Hollo 只约束它对外申请的能力。
5. **隐私 ≠ 沙盒**：ITP 这类反跟踪是策略插件，不是隔离机制，Stage 2 再做。

---

## 6. 结论

「对标苹果的安全」在 Hollo 里被翻译成三句可执行的话：

> 不赌没有漏洞 → 假设解析单元会被打穿 → 因此**默认拒绝、最小权限、出站重校验**。

再加一句 Apple 给不了、但公开源码能给的：

> 策略被削弱这件事，必须**可被外部检测**。

剩下的 OS 级隔离、签名、站点隔离，老老实实列在 [ROADMAP Stage 2](../../ROADMAP.md) 里，没做就是没做。
