# 安全设计与威胁模型

## 0. 先说清楚现在有什么、没有什么

**已有**：能力模型（默认拒绝）、隐私红线显式确认、锁定模式、隔离单元"引爆 → 出站重校验"的类型级约束、策略削弱检测、可逆卸载、三家扩展清单的只读归一。

**没有**：OS 级进程隔离实现（seccomp / AppContainer / Seatbelt / Landlock）、站点隔离、真实引擎沙盒、插件签名与来源验证、内存安全执行环境之外的审计。

策略写得对，不等于沙盒已经落地。ROADMAP Stage 2 才做平台后端插件里的真实隔离。把这些分开写，是为了不给自己发假绿卡。

---

## 1. 对标苹果，对标的是什么

联网调研（出处与原文摘录见 [`docs/research/apple-sandbox.md`](docs/research/apple-sandbox.md)）后，我们认为值得抄的不是某个 API，而是三条**思路**：

1. **Containment（围堵）**：Apple 在 Messages / Safari 上用 BlastDoor / Enhanced Security Extensions，把攻击者可控的复杂数据（图片、消息、网页内容）放进几乎无权限的隔离进程里解析，高权限进程只接收简单的、可校验的类型。即使隔离进程被打穿，攻击者也拿不到消息数据库、Cookie 或内核访问。
2. **出站数据重新校验**：Apple 明确说"要假设 containment 进程可能已经被攻陷，它返回的东西不可信"。所以 Messages 在显示图片前，会用极小的校验器重新确认"这确实是我期望格式的图片"。
3. **锁定模式削减攻击面**：Lockdown Mode 在 WebKit 上直接剥掉 JIT，避免 W^X 违规带来的 JIT 内存破坏面，代价是一点渲染开销。

对应到 Hollo：

| Apple | Hollo |
| --- | --- |
| BlastDoor 隔离进程 | `IsolationLevel::BlastDoor` 策略 + 平台后端插件负责真实进程 |
| 复杂数据在最低权限处 detonate | `SandboxPolicy::detonate()` |
| 返回数据仍不可信 | `Untrusted<T>`，`revalidate()` 前拿不到裸值 |
| Enhanced Security Extension | `sandbox-policy` 是一个可替换插件 kind |
| Lockdown Mode 禁 JIT | `deny_jit_in_lockdown` + `harden()` |
| App Sandbox 默认拒绝 | `default_deny` + `Capability` 三重闸门 |

**没有被照搬的**：Apple 的做法依赖 Darwin 内核、代码签名、AMFI/CoreTrust 与硬件级内存完整性（MIE / PAC），这些不是浏览器壳能提供的。Hollo 能做的是把"默认拒绝 + 最小权限 + 出站重校验"做成跨平台的公共契约，把 OS 特定实现交给平台插件。

---

## 2. 威胁模型

| 攻击者 | 能力 | 我们要挡住什么 | 当前手段 |
| --- | --- | --- | --- |
| 恶意网页 | 控制页面内容与脚本 | 拿到本地文件、剪贴板、位置、浏览历史 | 默认拒绝的能力仲裁；`origin` 声明 |
| 恶意／被投毒的插件 | 声明并请求超出需要的能力 | 权限提升、静默驻留、数据外传 | 最小权限交集、隐私红线显式确认、可逆卸载 |
| 恶意扩展（三家来源） | 利用方言差异绕过限制 | 用 `webRequest` 之类差异做隐蔽拦截 | 归一器显式 warnings；`net.intercept` 属安全红线 |
| 被攻陷的渲染／解析单元 | 完全控制该单元 | 借返回值污染高权限侧 | `Untrusted<T>` + `revalidate()` |
| 派生版本维护者 | 能改全部代码 | 弱化沙盒、加遥测、锁插件、故意变慢 | LICENSE 第 6 条 + `weakened()` 可检测 + 商标条款 |
| 供应链 | 往依赖里塞后门 | 内核被间接污染 | 零第三方依赖 + CI 守门 |

**明确不在范围内**：Stage 0 不防物理接触设备、不防已被攻陷的操作系统、不防用户主动安装并全权授权的插件。

---

## 3. 安全红线与隐私红线

`Capability::is_security_sensitive()`（`engine.swap` / `network.swap` / `sandbox.read` / `net.intercept`）：锁定模式下直接拒绝。理由很直接——"换引擎"和"改沙盒策略"正是攻击者拿到执行权后最想做的两件事，锁定模式就该把它们冻住。

`Capability::is_privacy_sensitive()`（`telemetry` / `profile.read` / `clipboard.read` / `device.camera` / `device.location`）：未经用户**逐项、显式、可撤销**确认，永不放行；确认状态不缓存到插件侧。

`SandboxPolicy::weakened()` 检测 `default_deny` 被关、出站重校验被关、隔离级别被降到 `None`。这是给 CI 与派生版本审计用的，也对应 LICENSE 第 6.2 条。

---

## 4. 报告漏洞

- 优先通过 GitHub 的 **Security → Report a vulnerability**（私有漏洞报告）提交，避免公开 issue 泄露细节。
- 如果该入口尚未开启，请先开一个**不含任何技术细节**的 issue 索要联系方式，或在仓库 Discussion 里留言；不要在公开场合贴 PoC。
- 请包含：影响版本、复现步骤、影响的能力边界、建议修法。
- 我们承诺不因善意的安全研究追究 LICENSE 第 11 条责任；第 3 条本来就允许安全审计。

修复节奏：确认后先出最小补丁与回归测试，再发布；CHANGELOG 记录而不掩盖。

---

## 5. 安全相关的验收清单（每阶段都要重跑）

- [x] `cargo test --all-targets` 全绿
- [x] `cargo clippy --all-targets -- -D warnings` 干净
- [x] 零第三方依赖守门
- [x] 未授予能力一律 `Deny`
- [x] 隐私红线未经确认一律 `Deny`
- [x] 锁定模式冻结安全红线
- [x] 卸载后服务／订阅／能力全部清零
- [ ] 平台后端插件真实进程隔离（Stage 2）
- [ ] 插件签名与来源验证（Stage 2）
- [ ] 出站流量可观测与阻断（Stage 2）
- [ ] 第三方安全审计（Stage 3）
