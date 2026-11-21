# 用户协议 / 服务条款（Terms of Service）

**适用仓库**：`codecloud-dev/Fenestra`（极小的浏览器空壳 / Browser shell）
**生效日期 / Effective date**：2026-11-15
**协议主体 / 版权人**：`codecloud-dev` —— 一个 GitHub 个人账号，由个人开发者运营，**不是公司、不是法人实体**；没有公司名称、注册号、注册地址或客服电话。`LICENSE` 与 `LICENSE-COMMERCIAL` 的版权人同为 `codecloud-dev`（GitHub 账号）及其书面指定的权利承继者。

> 中文在前，英文在后，同一份文件；口径一致，歧义以中文为准（`LICENSE-COMMERCIAL` 第 13.4 条亦规定中文文本为准）。
> 本文件与 `PRIVACY.md` 配套：数据行为以 `PRIVACY.md` 为准，使用条件以本文件为准。
> **本条款是服务与网站层面的说明，不是一份新许可。** Fenestra 的授权以仓库内 `LICENSE`（层 A）与 `LICENSE-COMMERCIAL`（层 B，FSL v1.0）为准；凡本条款与二者不一致，**以两份许可为准，商业条款以 `LICENSE-COMMERCIAL` 为准**。详见第二、六节。

---

## 一、接受条款

下载、编译、运行、修改 Fenestra，或使用其插件系统、官网 / 未来的商业授权服务，即表示你已阅读并同意本条款、`PRIVACY.md`，以及适用于你所使用部分的 `LICENSE` 或 `LICENSE-COMMERCIAL`。不同意请停止使用。

README「现在做到哪一步」如实标注：当前为 **Stage 1 脚手架**，`crates/gate`（18 测试通过）、`packages/core`（11 测试通过）、`packages/ui`（构建通过）、三个示例插件（类型检查通过）、双许可与文档守门已真跑通；但 **Tauri 2 外壳本地未编译、端到端集成尚未打通、打开真实网页 / 渲染引擎替换未做、敏感操作授权弹窗未做**。本条款不把未实现的能力写成已有能力。

---

## 二、双许可分层（避免与本条款冲突的关键）

| 层 | 覆盖 | 许可 | 规则 |
| --- | --- | --- | --- |
| **A** | 内核、权限闸门 / 能力仲裁、插件运行时、FEP 规范、示例插件、对外暴露的 API / ABI | `LICENSE` · *Fenestra Open License v1.0* | 自撰、符合 OSD 实质的**宽松开源许可**：永久、全球、免费、不可撤销，**允许商业使用、允许闭源派生、可再许可**，只需保留署名与许可文本；**不是「OSI 认证许可」**，也不得被称为任何既有许可证的变体；**已公开版本不得重新闭源** |
| **B** | 官方增值插件、未来企业版 / 加速 Pro、托管服务 | `LICENSE-COMMERCIAL` · *Fenestra Source License (FSL) v1.0* | **源码可见（source-available）、非 OSI**：非商业使用免费永久；**任何商业使用须经唯一入口（官网登录 → 签署《Fenestra 商业授权电子协议》→ 月付 / 年付两档付费 → 系统签发载明期限的电子授权凭证）**；授权**到期自动失效**；修改 / 派生版本对外提供时**必须公开对应完整源码**；第 6 条「五不丢」与第 6.8 条「四条绝对底线」不可退化，付费也不得豁免 |

**因此**：本条款第六节的「商用须授权、须付费」**只针对层 B 及包含层 B 的整体商业使用**，**绝不收回、不限制、不收费层 A 依 `LICENSE` 已永久授予你的权利**（含商业使用与闭源派生）。Fenestra 整体是**源码可见软件，不是开源软件**（`LICENSE-COMMERCIAL` 第 0 条）；层 A 可称「开源」，但不得称 Fenestra 整体为「OSI 认证开源 / 完全自由软件」。

凡本条款与 `LICENSE` 或 `LICENSE-COMMERCIAL` 有任何不一致，**以两份许可为准；其中商业使用、付费、授权范围、期限、终止、名称标识与争议，以 `LICENSE-COMMERCIAL` 为准**。本条款不扩大、不缩减、不修改任一许可授予或保留的权利。

---

## 三、按「现状」提供与免责

- 软件按 **「现状」（AS IS）** 提供（`LICENSE-COMMERCIAL` 第 10 条）：不附带任何明示或默示担保，包括适销性、特定用途适用性、不侵权、不中断、无缺陷、能打开任意网站。
- 版权人对任何直接、间接、附带、特殊或后果性损失不承担责任，即使已被告知可能发生。
- **浏览器中运行的网页、插件与扩展由各自作者负责，不由版权人担保**；你自行承担安装第三方插件的风险（`LICENSE-COMMERCIAL` 第 10 条）。
- 当前版本尚未打通端到端集成、尚未打开真实网页；规划中的 Servo 可替换引擎、Chrome / Edge / Firefox 三家 MV3 兼容层、官网商用授权通道**均非已交付承诺**（README / ROADMAP 明示「不承诺工期、不写没跑过的性能数字」）。
- 当前版本**未实现**：账号系统、云同步、遥测、自动更新、官方技术支持 SLA、商业 SLA / 赔偿。

---

## 四、可接受使用（AUP）

- 遵守你所在国家/地区的法律，以及 GitHub / Gitee / 系统 WebView / 各依赖与第三方服务各自条款。
- 不利用 Fenestra 入侵他人系统、抓取未授权数据、传播恶意软件、发起拒绝服务、规避网络管制或实施诈骗。
- 不把默认拒绝闸门 `Gate::check()`、最小权限交集 `Gate::effective()`、出站重校验 `Untrusted<T> → revalidate() → Trusted<T>` 当作可绕过的障碍去规避；`crates/gate/src/lib.rs` 顶层 `#[forbid(unsafe_code)]`。
- 隐私红线能力（`ClipboardRead` / `ProfileRead` / `CameraDevice` / `LocationDevice` / `Telemetry`）须**逐项、显式**确认，确认一个不连带放行另一个；不得在未确认情况下启用。
- **不得移除**用户卸载任意插件、更换渲染引擎 / 网络栈 / 界面 / 扩展运行时、导出全部本地数据、关闭任何内置服务、完全离线运行的能力；**不得**加入「只能从单一商店安装插件」的限制（`PRINCIPLES.md` §6、`LICENSE-COMMERCIAL` 第 6.3 条）——这既是用户权利，也是派生版本义务。
- 我们保留在明显滥用时拒绝提供官网服务 / 商业授权的权利，但**无义务监控**，也不为第三方网页、插件、扩展背书。

## 五、禁止逆向滥用与绕过授权

- 允许：阅读、编译、运行、修改、制作派生版本、为非商业目的分发 —— 这些是 `LICENSE`（层 A）与 `LICENSE-COMMERCIAL` 第 3 条明确授予的权利；为安全审计、兼容性评测、互操作性而进行的分析亦不受限（`LICENSE-COMMERCIAL` 第 1.4 条把安全审计、兼容性评测列为非商业使用）。
- 禁止的是**以规避授权与安全机制为目的的滥用**，包括但不限于：
  - 绕过 / 关闭 / 弱化**默认拒绝的能力仲裁、进程 / 站点隔离、隔离单元出站数据重校验、锁定模式下的 JIT 限制**，或默认降低沙盒隔离级别（第 6.2 条、第 6.8 条②）；
  - 给 `Untrusted<T>` 开后门、伪造 `revalidate()` 结果、夹带未声明字段绕过出站校验，或提供读取载荷的捷径（`Deref` / `AsRef` / `into_inner` 或等价手段）（第 6.8 条④）；
  - 在用户未逐项确认时启用隐私红线能力，或默认收集 / 上传浏览历史、页面内容、剪贴板、位置、凭据、设备指纹、植入未明示同意且可随时关闭的遥测（第 6.1 条、第 6.8 条③）；
  - 人为引入启动等待、强制广告、限速、无谓后台常驻、联网阻塞，或以商业 / 推广 / 遥测目的故意降低性能（第 6.4 / 6.5 条）；人为阻断三家扩展兼容层（第 6.6 条）；移除「卸载插件 / 换引擎 / 导出数据 / 完全离线运行」能力或锁定单一商店（第 6.3 条）；
  - **绕过第六节的商业授权与付费门禁**：伪造授权、破解授权校验、伪造电子授权凭证、规避第 4.5 条到期自动失效、冒用他人商业授权、以非商业许可包装商业使用、「先用后补」；
  - 以任何方式规避本系列（moxsh-terminal / moxsh-proot / 账号服务 / 官网）已有的兑换码、设备绑定或授权门禁。
- 上述禁止不扩张为「禁止一切逆向」：两份许可授予的阅读、修改、审计、评测、再分发权利完整保留；被禁止的是**用于绕过授权、规避付费、或使第 6 条 / 第 6.8 条保护性条款事实上失效**的具体滥用行为。`LICENSE-COMMERCIAL` 第 6 条按「普通用户能否实际获得该能力」判定，而不是按代码里是否存在某个开关判定。

---

## 六、专项：商业使用条款（Fenestra）

> 本节为**服务 / 网站层面**的商用流程说明。**以 `LICENSE-COMMERCIAL` 为准**：它是版权的唯一商业授权依据；本节不新增、不缩减该许可的任何权利或义务。本节引用的条号均指 `LICENSE-COMMERCIAL`（FSL v1.0）**2026-11-15 修订后**的文本。

### 6.1 非商用免费

个人学习、研究、教学、公益、非营利组织内部使用、安全审计、兼容性评测、个人自建自用，**始终免费、无需登录、无需申请、无需付费、永久、全球、不可撤销**（第 3 条、第 4.1 条）。层 A 依 `LICENSE` 更允许免费商业使用与闭源派生，见第二节。

### 6.2 什么算商业使用

依第 1.4 条，以下任一即构成商业使用：以本软件或派生版本向他人提供有偿产品 / 服务 / 托管 / 订阅 / 代运维 / 技术支持；在营利组织内用于生产经营、内部业务流程或对外交付物；预装、分发、转售、贴牌，或与硬件、服务捆绑以获取对价；植入广告、流量变现、付费引导或数据交易；以本软件为核心提供相竞争的商业产品。

**情形难以判断时，以官网公示的判断口径为准**；开发者也可在官网登录后提交书面询问，版权人应在合理时间内答复，**答复不构成对第 12.3 条的变更，更不构成商业授权**（第 1.4 条）。

### 6.3 商用须在官网登录并签订电子协议、按月付或年付两档付费

- **任何层 B 商业使用，必须在开始之前取得商业授权**（第 4.1 条）。未取得就使用的，**不是「先用后补」，而是自始无权**。
- **唯一入口：官网登录 + 电子协议**（第 4.2 条）。商业授权**不通过邮件、私信、口头、仓库 Discussion 或任何线下沟通发放**。开发者须在 Fenestra 官网以实名账号登录，在线阅读并签署《Fenestra 商业授权电子协议》，完成付费后由系统签发第 1.5 条所述的**电子授权凭证**（凭证须载明授权主体、授权范围、计费档位、生效日与到期日，四者缺一均视为未取得授权）。**官网登录记录、协议签署日志、授权凭证与支付流水共同构成授权成立的证据；缺少任一项，均视为未取得授权**；版权人不以任何其他方式追认、补发或倒签授权，任何第三方（包括代理商、集成商）也无权代发。
- **两档计费：月付 / 年付**（第 4.3 条）：**月付档**授权期限自生效日起一个自然月，按每月一期续费；**年付档**自生效日起十二个自然月，按每年一期续费，价格低于连续十二期月付之和。**两档授权范围完全相同**（都能商业使用、预装、分发、转售、贴牌、提供托管与技术支持）；年付只是期限更长、单价更低，**不**买到更多权利，更不解除第 5 条、第 6 条与第 4.5 条的期限约束。不设第三档、不设隐藏档、不按流量或装机量另算。具体金额在官网签署电子协议时一次性写明、当期之内不变；定价遵循「**小额、透明、不歧视**」：对同等情形的不同主体不实行差别价格，价格调整只影响新签与续签、不追溯已生效的当期授权。授权主体、范围、档位与期限**以凭证为准**，任何官网页面文案或销售话术不得扩张或缩减凭证内容。
- **授权期限**（第 4.4 条）：以电子授权凭证载明的起止日为准，精确到日；**不自动扣款、不默认续订、不产生静默续期**，是否续费由你自行决定；当期授权在到期日 24 时（版权人所在地时区）终止；授权不因转让、合并、分立、更名而自动转移给新主体，主体变更须重新登录官网签署协议并取得新凭证。
- **付费不解除的义务**（第 4.7 条，原第 4.6 条口径）：付费只解除本条的「不得商用」限制；取得商业授权**不**解除第 5 条（修改 / 派生版本对外提供时必须公开对应完整源码与构建脚本、标注修改、保留许可文本、不得附加额外限制）与第 6 条（含第 6.8 条四条绝对底线），也**不**解除第 4.5 条的期限约束。**付了钱也不能闭源、不能弱化安全与隐私、不能锁死插件、不能拿掉默认拒绝，更不能把有期限的授权当成永久授权**。第 4.8 条的费用豁免须以官网出具的书面凭证为准，且不改变第 4.5 条、第 4.6 条与其余条款。
- **当前状态的诚实说明**：截至本条款生效日，官网在线签约与付费通道按规划推进，**尚未上线**，当前代码未实现账号登录、电子协议与授权校验，因此**当前不收集任何账号、订单或支付信息**。依第 4.2 条，商业授权**只**通过上述渠道发放，明确排除邮件、私信、Discussion 与任何线下沟通，且不追认、补发、倒签；因此**在该通道上线并取得电子授权凭证之前，不存在任何取得商业授权的替代途径，任何层 B 商业使用都不得开始**。**通道未上线不构成可无偿商用的许可**；未来上线后，新增的数据项与流程以届时更新的 `PRIVACY.md` 与本文件为准。

### 6.4 逾期授权自动失效

依第 4.5 条，期限届满而未完成下一期付费的，**商业授权于到期时自动失效**，无需版权人另行通知、无需催告、不设宽限期；失效是自动发生的法律状态，**不因你未读邮件、未登录官网、系统故障、支付渠道延迟或内部审批拖延而改变**。失效后，第 3 条的非商业许可**继续有效**：你仍可阅读、编译、修改、非商业地自用，已公开的版本也不会因此被收回；但「可以拿去卖」的那部分权利**即刻消失**。

该失效**仅针对层 B 商业授权**，并作如下限缩以保持与两份许可不冲突：

- **不影响** `LICENSE-COMMERCIAL` 第 3 条永久、全球、免费、不可撤销的**非商业许可**（非商用仍可继续免费使用），也不收回已公开版本；
- **不影响**层 A 依 `LICENSE` 已取得的权利（含商业使用、闭源派生、已公开版本不得重新闭源）；
- **失效本身不构成违约**（第 11 条明确「商业授权依第 4.5 条到期自动失效，其本身不构成违约」）；但**依第 4.6 条在失效后继续使用，构成违约**，并按第 4.6 条承担后果；
- **不改变**第 11 条的终止与恢复规则（违反第 4、5、6、7 条许可自动终止；首次违约者可在收到通知后 30 日内补救、许可自动恢复一次，同一主体第二次违约不再恢复）；但第 4.6(d) 条已另行规定「补足全部欠款与费用后，版权人**可以**（而非必须）书面决定是否恢复，且同一主体第二次发生的不予恢复」的，**优先适用第 4.6(d) 条**。

### 6.5 未经授权商用、失效后继续使用：保留法律追责权利

- **未取得授权即商用**：依第 4.1 条「自始无权」、第 4.2 条「缺少任一项证据均视为未取得授权」，不构成商业授权；依第 4 条与第 11 条，构成违约、第 3 条许可终止。版权人依第 2 条保留全部未明示授予的权利，**保留要求停止侵害、追究违约责任与损害赔偿的权利**，并依第 13.3 条将争议提交**版权人所在地有管辖权的人民法院**处理。
- **授权失效后仍继续任何商业使用**（包括继续对外提供已部署的服务、继续分发已打包的副本、继续以本软件交付客户项目、继续以含本软件的产品收取订阅费），属于**明知无权而使用**（第 4.6 条），下列后果可并行主张：
  - (a) 自失效之日起，按你原本所选档位费用的**三倍**计付占用使用费，不足一期的按一期计；
  - (b) 版权人有权要求立即停止使用、下架、召回、删除相关副本与服务，并就已发生的商业收益主张返还；
  - (c) 版权人因此产生的合理维权开支（取证、公证、律师费、差旅等）由你承担；
  - (d) 第 3 条许可依第 11 条终止；补足全部欠款与费用后，版权人**可以**（而非必须）书面决定是否恢复，且同一主体第二次发生的不予恢复；
  - (e) 情节严重时，版权人可依法主张侵权损害赔偿，并就不正当竞争、商标混淆另行主张权利。
- 版权人主张上述后果前，会先以**官网注册邮箱**发送一次书面告知；该告知**不是**授权恢复，也**不是**宽限期，只是给对方一个自行纠正的机会。不付费而主张「不知道到期了」「以为会自动续」「已经在走内部流程」，均不构成抗辩。
- 争议依第 13.3 条：**三十日内友好协商**；协商不成，任一方均可向**版权人所在地有管辖权的人民法院**提起诉讼，**不适用仲裁**；官网登录记录、电子协议签署日志、授权凭证、支付流水与官网公示的当期价格，可作为认定授权状态与费用的有效证据；争议解决期间**第 4.5 条「逾期自动失效」不中止执行**，除法院另有裁定外，未付费期间的商业使用应先行停止；本协议适用版权人所在地法律，冲突法规则不予适用。

### 6.6 与两份许可不冲突的声明（以 LICENSE-COMMERCIAL 为准）

- 本节任何措辞**不得**解释为收回层 A `LICENSE` 已授予的权利，或弱化 `LICENSE-COMMERCIAL` 第 5 条、第 6 条（含第 6.8 条四条绝对底线）；
- 本节与 `LICENSE` / `LICENSE-COMMERCIAL` 不一致时，**以两份许可为准；商业使用、付费、授权范围、期限、终止与追责，以 `LICENSE-COMMERCIAL` 为准**；
- **授权版本不追溯**：`LICENSE-COMMERCIAL` 第 12.1 条与本次修订记录明确，此前已获取的版本仍适用其获取当时的协议文本，本次修订不追溯收回已授予的权利；
- 商业授权联系**不**通过 GitHub Issues / Discussion / 私信 / 邮件办理（第 4.2 条）；本条款第九节的 GitHub 渠道**仅用于一般问题反馈**，使用任何编造的公司主体、邮箱、电话或地址都**不构成**授权。

---

## 七、不可退化条款（五不丢 + 四条绝对底线，付钱也不行）

派生版本**不得**有下列行为（第 6 条，按「普通用户能否实际获得该能力」判定）：**6.1 隐私不丢**、**6.2 安全不丢**、**6.3 自由不丢**、**6.4 快捷与快速不丢**、**6.5 流畅不丢**、**6.6 兼容不丢**；第 6.7 条：使任一条款事实上失效，即同时违反第 5.1(e) 与第 7 条。

**第 6.8 条「四条绝对底线」**（2026-11-15 修订新增）：**① 默认拒绝**（未显式声明、未经策略允许、未获用户逐项确认一律拒绝，不得改为默认允许、不得把「未拒绝」当成「已授权」）；**② 沙箱隔离**（不得默认降低隔离级别、不得让未隔离代码直接触及宿主或越过能力仲裁；确需降低须首启显著告知且不得使用 Fenestra 名称或标识）；**③ 隐私红线**（`telemetry` / `profile.read` / `clipboard.read` / `device.camera` / `device.location` 须逐项明示同意且可随时关闭，不得默认收集或上传浏览历史、页面内容、剪贴板、位置、凭据与设备指纹）；**④ 出站重校验**（离开隔离边界前必须经过 `Untrusted<T> → revalidate() → Trusted<T>`，不得提供任何绕过校验读取载荷的捷径）。**第 4 条的月付档、年付档、官网电子协议、付费凭证、豁免凭证，以及任何「企业定制条款」，都不构成对上述四条的豁免；与之冲突的约定一律无效**。付了钱、签了字、买了年付，这四条仍原封不动。层 A `LICENSE` 第 5 条另把「默认拒绝 / 沙箱隔离 / 隐私红线 / 出站重校验 / 可逆登记」列为永久开源不变量；`LICENSE-COMMERCIAL` 第 6 条按「普通用户能否实际获得该能力」判定，而不是按代码里是否存在某个开关判定。

## 八、第三方服务与依赖（链接指向，不为其背书）

| 服务 / 依赖 | 用途 | 条款 / 隐私政策入口 |
| --- | --- | --- |
| GitHub | 主仓库、Issues、Discussion（**仅一般交流，不发放商业授权**） | `https://docs.github.com/en/site-policy/github-terms/github-terms-of-service` · `https://docs.github.com/en/site-policy/privacy-policies/github-privacy-statement` |
| Gitee | 国内只读镜像（若启用） | `https://gitee.com/`（以其站内公布条款为准） |
| 爱发电 | 仅当未来赞助 / 兑换流程涉及（见官网、proot 政策） | `https://afdian.com/`（以其站内公布条款为准） |
| Cloudflare | 若未来官网 / 授权服务使用 | `https://www.cloudflare.com/privacypolicy/` |
| 系统 WebView / wry / Servo / Tauri 2 / Svelte 5 / Vite 8 / Cordis 4 | 运行时与构建依赖 | 见各项目公布的许可与条款；README 明示 Stage 1 真引入 Servo 时逐文件核依赖许可（Servo `MPL-2.0`，Cordis / Svelte / Vite `MIT`，Tauri `Apache-2.0 OR MIT`） |

以各站最新公布条款为准；我们**不代替它们解释、不担保可用性、不对其行为负责、不为其背书**。当前版本不向这些服务上传任何用户数据（见 `PRIVACY.md`）。

## 九、知识产权、名称与标识

- 层 A 与层 B 的授权范围以 `LICENSE` / `LICENSE-COMMERCIAL` 为准（第二节）。
- 「Fenestra」、空壳标识及其组合为版权人标识（第 7 条）：原样、非商业、未修改第 6 条行为的再分发可保留名称并注明来源；**修改了第 6 条任一行为、或以商业方式提供的派生版本，不得在名称、图标、商店标题、域名中暗示其为官方版本或获得背书，应改用自有名称**。商标法另行赋予的权利不受影响。
- 贡献：向本项目提交的贡献视为同意按本协议公开，并授予版权人在全球范围内永久、免费、可用于商业版本的非独占使用权；**你保留署名权、版权不转让、无需签署 CLA**，版权人不得抹去署名后单独再许可给第三方（第 8 条）。
- 专利：第 9 条授予为行使第 3 条权利所必需的非独占、免费专利许可；若你或关联方就本软件主张专利侵权，该许可及第 3 条许可自动终止。商业授权中的专利范围由双方另行书面约定，未约定的不超出第 3 条范围。

## 十、变更、终止与争议

- 本条款变更直接更新本仓库 `TERMS.md`，留痕于 Git 提交与 CHANGELOG；不收集联系方式，故**不主动推送**，重大变更在仓库公告。继续使用视为接受。
- **许可版本不追溯**：第 12.1 条规定你获取的某个版本永久适用该版本当时对应的协议文本，版权人发布新版本不追溯收回已授予的权利；第 12.3 条承诺**不会把已公开版本重新闭源**、不收回已授予权利；第 12.2 条的《开放转换声明》一经发布不可撤销。`LICENSE-COMMERCIAL` 2026-11-15 修订记录亦明确「此前已获取的版本仍适用其获取当时的协议文本；本次修订不追溯收回已授予的权利」。
- 终止：你可随时停止使用、删除本地副本与插件；许可的自动终止与恢复依第 11 条（见 6.4；**商业授权依第 4.5 条到期自动失效本身不构成违约，但依第 4.6 条在失效后继续使用构成违约，且第 4.6(d) 条的恢复规则优先适用**）。第 0、1、8、9、10、12、13 条在终止后继续有效。
- 因不可抗力、第三方服务终止、平台下架、依赖许可变化导致的功能不可用，版权人不承担责任（第 10 条）。
- 争议应先在**三十日内友好协商**；协商不成，依第 13.3 条向**版权人所在地有管辖权的人民法院**提起诉讼，**不适用仲裁**；官网登录记录、电子协议签署日志、授权凭证、支付流水与官网公示的当期价格可作为认定授权状态与费用的有效证据；争议解决期间第 4.5 条「逾期自动失效」不中止执行；本协议适用版权人所在地法律，冲突法规则不予适用。本条款以中文书写，英文仅为阅读便利，**歧义以中文为准**（第 13.4 条）。若某条被认定无效，其余条款继续有效并尽可能按原意解释（第 13.1 条）；本协议为双方就本软件达成的完整协议（第 13.2 条）。

## 十一、联系方式

- **唯一联系方式**：在对应仓库开 **GitHub Issues**（`https://github.com/codecloud-dev/<repo>/issues`），或联系 GitHub 账号 **`codecloud-dev`**。
- **商用授权不通过 GitHub Issues / Discussion / 私信 / 邮件办理**。依 `LICENSE-COMMERCIAL` 第 4.2 条与附「一眼看懂」，**唯一入口**为：Fenestra 官网登录 → 签署《Fenestra 商业授权电子协议》→ 选择月付或年付 → 系统签发载明期限的电子授权凭证；邮件、私信、Discussion、线下沟通一律**不发放**授权。当前该通道尚未上线（见 6.3）；GitHub Issues / `codecloud-dev` 仅用于一般问题反馈、安全报告与条款咨询。
- 不提供、不编造客服邮箱、电话、通信地址、公司名、注册号。请勿在公开 Issue 里贴密钥、凭据或授权材料。

---
---

# English — Terms of Service

**Repository**: `codecloud-dev/Fenestra` (a minimal browser shell)
**Effective date**: 2026-11-15
**Party / copyright holder**: `codecloud-dev` — a personal GitHub account run by an individual developer, **not a company or legal entity**; no company name, registration number, registered address, or support phone. `LICENSE` and `LICENSE-COMMERCIAL` name the same copyright holder, `codecloud-dev` (GitHub account) and its written-designated successors.

> Chinese first, English after, in one file; the Chinese text controls in case of conflict (`LICENSE-COMMERCIAL` §13.4 also provides that the Chinese text governs).
> Pairs with `PRIVACY.md`: data practices are governed by `PRIVACY.md`, terms of use by this document.
> **These terms are a service- and website-level explanation, not a new licence.** Authorisation for Fenestra is governed by `LICENSE` (Layer A) and `LICENSE-COMMERCIAL` (Layer B, FSL v1.0) in the repository; where these terms conflict with either, **the licences control, and commercial terms are governed by `LICENSE-COMMERCIAL`**. See §2 and §6.

## 1. Acceptance

Downloading, compiling, running, or modifying Fenestra, or using its plugin system, the website, or future commercial-licensing services, means you have read and agree to these terms, `PRIVACY.md`, and the `LICENSE` or `LICENSE-COMMERCIAL` applicable to the part you use. If you do not agree, stop using it.

The README's "Honest status" states plainly: this is **Stage 1 scaffolding**; `crates/gate` (18 tests pass), `packages/core` (11 tests pass), `packages/ui` (build passes), three example plugins (typecheck passes), and the dual-license / docs guards actually run locally; but the **Tauri 2 shell is not compiled locally, end-to-end integration is not yet wired, opening real web pages / engine replacement is not done, and the sensitive-action consent dialog is not done**. These terms do not present unimplemented capabilities as shipped features.

## 2. Dual-licensing split (the key to avoiding conflicts)

| Layer | Covers | Licence | Rules |
| --- | --- | --- | --- |
| **A** | Kernel, permission gate / capability arbitration, plugin runtime, FEP spec, example plugins, exposed API / ABI | `LICENSE` · *Fenestra Open License v1.0* | Self-written, permissive open-source licence meeting the substance of the OSD: perpetual, worldwide, free, irrevocable, **commercial use allowed, proprietary derivatives allowed, sublicensable**, requiring only attribution and the licence text; **not an "OSI-certified licence"** and not to be called a variant of any existing licence; **published versions must not be re-closed** |
| **B** | Official value-add plugins, future enterprise / Accelerate Pro, hosted services | `LICENSE-COMMERCIAL` · *Fenestra Source License (FSL) v1.0* | **Source-available, non-OSI**: non-commercial use free and perpetual; **any commercial use must go through the single channel (official website login → sign the electronic Fenestra commercial licence agreement → monthly or annual payment → system-issued, time-limited electronic licence certificate)**; the licence **expires automatically**; modified/derivative versions offered to third parties **must publish their complete corresponding source**; §6 "five must-not-lose" and §6.8 "four absolute bottom lines" are non-degradable and **cannot be waived by payment** |

**Therefore**: §6's "commercial use requires a licence and payment" **applies only to Layer B and to commercial use of the whole that includes Layer B**, and **never revokes, restricts, or charges for rights already granted perpetually under `LICENSE` for Layer A** (including commercial use and proprietary derivatives). Fenestra as a whole is **source-available software, not open-source software** (`LICENSE-COMMERCIAL` §0); Layer A may be called "open source", but Fenestra as a whole must not be called "OSI-certified open source / completely free software".

Where these terms conflict with `LICENSE` or `LICENSE-COMMERCIAL`, **the licences control; for commercial use, payment, licence scope, term, termination, naming and disputes, `LICENSE-COMMERCIAL` controls**. These terms expand, reduce, or modify no right granted or reserved by either licence.

## 3. "As is" and disclaimer

- Provided **"AS IS"** (`LICENSE-COMMERCIAL` §10): without any express or implied warranty of merchantability, fitness for a particular purpose, non-infringement, uninterrupted operation, defect-free operation, or ability to open any website.
- The copyright holder is not liable for any direct, indirect, incidental, special, or consequential loss, even if advised of the possibility.
- **Web pages, plugins and extensions running in the browser are the responsibility of their respective authors, not warranted by the copyright holder**; you bear the risk of installing third-party plugins (`LICENSE-COMMERCIAL` §10).
- The current version has not wired end-to-end integration and cannot open real web pages; the planned Servo replaceable engine, Chrome / Edge / Firefox MV3 compatibility layers, and the website commercial-licensing channel are **not delivered promises** (README / ROADMAP explicitly "no schedule promises, no unmeasured performance numbers").
- **Not implemented**: accounts, cloud sync, telemetry, automatic updates, an official support SLA, or a commercial SLA / indemnity.

## 4. Acceptable use (AUP)

- Comply with the law of your country/region and with the terms of GitHub / Gitee / the system WebView / dependencies and third-party services.
- Do not use Fenestra to intrude into others' systems, scrape unauthorised data, spread malware, launch denial-of-service, or commit fraud.
- Do not treat the default-deny `Gate::check()`, least-privilege intersection `Gate::effective()`, or outbound revalidation `Untrusted<T> → revalidate() → Trusted<T>` as obstacles to be circumvented; `crates/gate/src/lib.rs` declares `#[forbid(unsafe_code)]` at the top.
- Privacy red-line capabilities (`ClipboardRead` / `ProfileRead` / `CameraDevice` / `LocationDevice` / `Telemetry`) require **per-item, explicit** confirmation; confirming one never unlocks another; they must not be enabled without confirmation.
- **Do not remove** the user's ability to uninstall any plugin, swap the rendering engine / network stack / UI / extension runtime, export all local data, turn off any built-in service, or run fully offline; **do not** impose "plugins only from a single store" (`PRINCIPLES.md` §6, `LICENSE-COMMERCIAL` §6.3) — this is both a user right and a derivative-version duty.
- We reserve the right to deny website services / commercial licensing in cases of obvious abuse, but have **no duty to monitor** and do not endorse third-party pages, plugins, or extensions.

## 5. No abusive reverse engineering / no bypassing authorisation

- Allowed: reading, compiling, running, modifying, creating derivative versions, and distributing for non-commercial purposes — rights expressly granted by `LICENSE` (Layer A) and `LICENSE-COMMERCIAL` §3; analysis for security auditing, compatibility evaluation and interoperability is likewise unrestricted (`LICENSE-COMMERCIAL` §1.4 lists security audits and compatibility evaluation as non-commercial use).
- Prohibited is **abuse aimed at circumventing authorisation and security mechanisms**, including but not limited to:
  - Bypassing / disabling / weakening **default-deny capability arbitration, process/site isolation, outbound revalidation of isolated units, JIT restrictions in locked mode**, or lowering sandbox isolation by default (§6.2, §6.8 ②);
  - Backdooring `Untrusted<T>`, forging `revalidate()` results, smuggling undeclared fields past outbound validation, or providing any shortcut to read the payload (`Deref` / `AsRef` / `into_inner` or equivalents) (§6.8 ④);
  - Enabling privacy red-line capabilities without per-item user confirmation, or collecting/uploading by default browsing history, page content, clipboard, location, credentials, device fingerprints, or embedding telemetry without explicit consent that can be turned off (§6.1, §6.8 ③);
  - Artificially adding startup waits, forced ads, throttling, needless background residency, network blocking, or deliberately degrading performance for commercial / promotional / telemetry purposes (§6.4 / §6.5); artificially blocking the three extension compatibility layers (§6.6); removing the ability to uninstall plugins / swap engines / export data / run fully offline, or locking to a single store (§6.3);
  - **Bypassing §6's commercial-licensing and payment gates**: forging licences, cracking authorisation checks, forging electronic licence certificates, evading §4.5 automatic expiry, using another's commercial licence, disguising commercial use as non-commercial, or "use first, pay later";
  - In any way evading existing redeem-code, device-binding, or authorisation gates of this series (moxsh-terminal / moxsh-proot / account service / website).
- This does **not** expand into "all reverse engineering is forbidden": the reading, modification, auditing, evaluation and redistribution rights granted by both licences remain intact; what is prohibited is the specific abusive conduct of bypassing authorisation, evading payment, or making §6 / §6.8's protective clauses ineffective in fact. `LICENSE-COMMERCIAL` §6 is judged by "whether an ordinary user can actually obtain the capability", not by whether a switch exists in the code.

## 6. Special clause: commercial use (Fenestra)

> This section is a **service / website-level** description of the commercial flow. **`LICENSE-COMMERCIAL` controls**: it is the sole basis for commercial authorisation; this section adds or removes no right or duty under that licence. Clause numbers here refer to the **2026-11-15 revised** text of `LICENSE-COMMERCIAL` (FSL v1.0).

### 6.1 Non-commercial use is free

Personal study, research, teaching, public-interest use, internal use by nonprofits, security auditing, compatibility evaluation, and personal self-hosting are **always free, with no login, no application, no payment, perpetual, worldwide, and irrevocable** (§3, §4.1). Layer A under `LICENSE` further allows free commercial use and proprietary derivatives (§2).

### 6.2 What counts as commercial use

Per §1.4, any of the following is commercial use: providing paid products / services / hosting / subscriptions / managed operations / technical support to others using the software or derivatives; using it inside a for-profit organisation for production, internal business processes, or client deliverables; preinstalling, distributing, reselling, white-labelling, or bundling with hardware/services for consideration; embedding ads, traffic monetisation, paid funnels, or data trading; offering a competing commercial product built on the software.

**When a case is hard to judge, the judgement criteria published on the official website govern**; a developer may also submit a written enquiry after logging in on the website, and the copyright holder should reply within a reasonable time, but **the reply does not amend §12.3 and does not constitute a commercial licence** (§1.4).

### 6.3 Commercial use requires website login + an electronic agreement, with monthly or annual payment tiers

- **Any Layer B commercial use must obtain a commercial licence before starting** (§4.1). Using it without one is **not "use first, pay later" — you have no right from the outset**.
- **Single channel: website login + electronic agreement** (§4.2). A commercial licence is **not issued via email, private message, verbal statement, repository Discussion, or any offline communication**. A developer must log in with a real-name account on the Fenestra official website, read and sign the **《Fenestra 商业授权电子协议》 (Fenestra Electronic Commercial Licence Agreement)** online, and after payment the system issues the **electronic licence certificate** described in §1.5 (the certificate must state the licensed entity, licence scope, billing tier, effective date and expiry date; **missing any of the four means no licence was obtained**). **The website login record, agreement signing log, licence certificate and payment record together constitute the evidence that a licence was formed; missing any one means no licence was obtained.** The copyright holder does **not** retroactively recognise, reissue, or backdate a licence by any other means, and no third party (including agents and integrators) may issue one.
- **Two billing tiers: monthly / annual** (§4.3): the **monthly** term is one calendar month from the effective date, renewing monthly; the **annual** term is twelve calendar months, renewing yearly, at a price lower than twelve consecutive monthly payments. **Both tiers have exactly the same licence scope** (commercial use, preinstallation, distribution, resale, white-labelling, hosting and technical support); annual only buys a longer term and lower unit price, **not** more rights, and does not lift §5, §6 or §4.5's term constraint. There is no third tier, no hidden tier, and no separate charge by traffic or install count. The exact amount is written once when signing the e-agreement on the website and does not change within the current term; pricing follows **"small, transparent, non-discriminatory"**: no differential pricing between entities in equivalent situations, and price adjustments only affect new and renewed licences, not the current term already in effect. The licensed entity, scope, tier and term **follow the certificate**; no website copy or sales pitch may expand or reduce the certificate's content.
- **Licence term** (§4.4): follows the certificate's start/end dates, precise to the day; **no automatic charge, no default renewal, no silent renewal** — whether to renew is your decision; the current term ends at 24:00 on the expiry date (copyright holder's local timezone); the licence does not automatically transfer to a new entity through assignment, merger, division or renaming — an entity change requires a new website login, a new agreement and a new certificate.
- **Obligations payment does not lift** (§4.7, formerly §4.6): payment only lifts this clause's "no commercial use" restriction; obtaining a commercial licence does **not** lift §5 (when modified/derivative versions are offered to third parties, publish the complete corresponding source and build scripts, mark changes, retain the licence text, add no extra restrictions) or §6 (including §6.8's four absolute bottom lines), and does **not** lift §4.5's term constraint. **Paying does not buy the right to close the source, weaken security/privacy, lock down plugins, remove default-deny, or treat a time-limited licence as perpetual.** Fee waivers under §4.8 require a written certificate issued by the website and do not change §4.5, §4.6 or the remaining clauses.
- **Honest current status**: as of the effective date, the website's online signing and payment channel is **planned but not yet live**, and the current code implements no account login, e-agreement, or authorisation check, so **no account, order, or payment information is currently collected**. Per §4.2, a commercial licence is issued **only** through that channel, which explicitly excludes email, private messages, Discussions and any offline communication, and does not retroactively recognise, reissue, or backdate a licence; therefore **until that channel is live and an electronic licence certificate is obtained, there is no alternative way to obtain a commercial licence, and no Layer B commercial use may begin**. **The channel not being live is not a licence to use it commercially for free**; once launched, new data items and flows will be governed by the then-updated `PRIVACY.md` and this document.

### 6.4 Authorisation expires automatically on lapse

Per §4.5, if the term expires without the next period being paid, **the commercial licence lapses automatically at expiry**, without separate notice, without demand, and without a grace period; **lapse is an automatic legal state that does not change because you did not read an email, did not log in, a system failure, a payment-channel delay, or internal approval delays**. After lapse, the non-commercial licence under §3 **continues**: you may still read, compile, modify, and use it non-commercially, and already-published versions are not withdrawn; but the "right to sell" part **disappears immediately**.

That lapse **applies only to the Layer B commercial licence**, with the following limits to stay consistent with both licences:

- It **does not affect** the perpetual, worldwide, free, irrevocable **non-commercial licence** of §3 (non-commercial use may continue for free), and does not withdraw already-published versions;
- It **does not affect** rights already obtained for Layer A under `LICENSE` (including commercial use, proprietary derivatives, and published versions not being re-closed);
- **Lapse itself is not a breach** (§11 expressly states "a commercial licence expiring automatically under §4.5 is not itself a breach"); but **continued commercial use after lapse under §4.6 is a breach** and bears §4.6's consequences;
- It **does not change** §11's termination and restoration rules (breach of §§4, 5, 6, 7 automatically terminates the licence; a first-time breacher may cure within 30 days of notice with one automatic restoration, and a second breach by the same entity is not restorable); however, where §4.6(d) separately provides that "after all outstanding payments and fees are made up, the copyright holder **may** (but is not required to) decide in writing whether to restore, and a second occurrence by the same entity is not restored", **§4.6(d) takes priority**.

### 6.5 Unauthorised commercial use / continued use after lapse: legal action reserved

- **Commercial use without a licence**: under §4.1 ("no right from the outset") and §4.2 ("missing any one piece of evidence means no licence was obtained"), no commercial licence is formed; under §4 and §11 this constitutes breach and the §3 licence terminates. The copyright holder reserves all rights not expressly granted under §2 and **reserves the right to demand cessation of infringement, pursue breach liability and damages**, and to submit disputes to the **people's court of competent jurisdiction at the copyright holder's location** under §13.3.
- **Any continued commercial use after lapse** (including continuing to offer already-deployed services, continuing to distribute already-packaged copies, continuing to deliver client projects with the software, or continuing to charge subscription fees for products containing it) is **use while knowingly having no right** (§4.6), and the following consequences may be claimed cumulatively:
  - (a) from the lapse date, an occupation usage fee of **three times** the fee of the tier you originally selected, with a partial period counted as one full period;
  - (b) the copyright holder may demand immediate cessation of use, takedown, recall, deletion of relevant copies and services, and return of commercial revenue already received;
  - (c) reasonable enforcement expenses incurred by the copyright holder (evidence collection, notarisation, attorney fees, travel, etc.) are borne by you;
  - (d) the §3 licence terminates under §11; after all outstanding payments and fees are made up, the copyright holder **may** (but is not required to) decide in writing whether to restore, and a second occurrence by the same entity is not restored;
  - (e) in serious cases, the copyright holder may claim infringement damages according to law, and separately assert rights regarding unfair competition and trademark confusion.
- Before asserting the above consequences, the copyright holder will send one written notice to the **website-registered email**; that notice **is not licence restoration and is not a grace period**, only an opportunity to self-correct. Claiming "I didn't know it expired", "I thought it would auto-renew", or "internal approval is in progress" without paying is not a defence.
- Disputes under §13.3: **30 days of amicable negotiation**; failing that, either party may file suit in the **people's court of competent jurisdiction at the copyright holder's location**, **without arbitration**; the website login record, e-agreement signing log, licence certificate, payment record and the website's published current price may serve as valid evidence of licence status and fees; **during dispute resolution, §4.5 "automatic lapse" is not suspended**; unless the court rules otherwise, commercial use during the unpaid period should stop first; this licence is governed by the law of the copyright holder's location, and conflict-of-law rules do not apply.

### 6.6 Non-conflict declaration (LICENSE-COMMERCIAL controls)

- Nothing in this section may be read as revoking rights already granted for Layer A under `LICENSE`, or weakening `LICENSE-COMMERCIAL` §5 or §6 (including §6.8's four absolute bottom lines);
- Where this section conflicts with `LICENSE` / `LICENSE-COMMERCIAL`, **the licences control; for commercial use, payment, licence scope, term, termination and legal action, `LICENSE-COMMERCIAL` controls**;
- **Licence versions are not retroactive**: §12.1 and the 2026-11-15 revision record of `LICENSE-COMMERCIAL` expressly provide that versions obtained earlier remain governed by the licence text corresponding to them at the time of acquisition, and this revision does not retroactively revoke granted rights;
- **Commercial licensing is not handled through GitHub Issues / Discussions / private messages / email** (§4.2); the GitHub channels in §11 are **only for general feedback**, and no invented company entity, email, phone, or address **constitutes** a licence.

## 7. Non-degradable clauses ("five must-not-lose" + "four absolute bottom lines", payment cannot lift them)

Derivative versions **must not** (§6, judged by "whether an ordinary user can actually obtain the capability"): **6.1 privacy not lost**, **6.2 security not lost**, **6.3 freedom not lost**, **6.4 speed and quickness not lost**, **6.5 smoothness not lost**, **6.6 compatibility not lost**; §6.7: making any clause ineffective in fact simultaneously breaches §5.1(e) and §7.

**§6.8 "four absolute bottom lines"** (added in the 2026-11-15 revision): **① Default deny** (any capability not explicitly declared, not allowed by policy, and not individually confirmed by the user is denied; it must not be changed to default-allow, and "not denied" must not be treated as "authorised"); **② Sandbox isolation** (isolation units for plugins/sites must not have their level lowered by default, and unisolated code must not directly touch the host or bypass capability arbitration; any necessary reduction must be prominently disclosed at first launch and **must not use the Fenestra name or marks**); **③ Privacy red lines** (`telemetry` / `profile.read` / `clipboard.read` / `device.camera` / `device.location` require per-item explicit user consent that can be turned off at any time; browsing history, page content, clipboard, location, credentials and device fingerprints must not be collected or uploaded by default); **④ Outbound revalidation** (data produced by an isolated unit must pass `Untrusted<T> → revalidate() → Trusted<T>` before leaving the isolation boundary; no shortcut to read the payload bypassing validation may be provided). **The §4 monthly tier, annual tier, website e-agreement, payment certificate, waiver certificate, and any "enterprise custom terms" do not constitute a waiver of these four; conflicting agreements are void.** Paying, signing, or buying annual leaves these four untouched. Layer A `LICENSE` §5 additionally lists "default deny / sandbox isolation / privacy red lines / outbound revalidation / revertible registration" as permanent open-source invariants; `LICENSE-COMMERCIAL` §6 is judged by "whether an ordinary user can actually obtain the capability", not by whether a switch exists in the code.

## 8. Third-party services and dependencies (links only, no endorsement)

| Service / dependency | Purpose | Terms / privacy policy |
| --- | --- | --- |
| GitHub | Canonical repository, Issues, Discussions (**general communication only, no commercial licences issued**) | `https://docs.github.com/en/site-policy/github-terms/github-terms-of-service` · `https://docs.github.com/en/site-policy/privacy-policies/github-privacy-statement` |
| Gitee | Mainland-China read-only mirror (if enabled) | `https://gitee.com/` (see its published terms) |
| Afdian | Only where future sponsorship / redemption flows apply (see the website and proot policies) | `https://afdian.com/` (see its published terms) |
| Cloudflare | If a future website / licensing service uses it | `https://www.cloudflare.com/privacypolicy/` |
| System WebView / wry / Servo / Tauri 2 / Svelte 5 / Vite 8 / Cordis 4 | Runtime and build dependencies | See each project's published licence and terms; the README states dependency licences are checked file-by-file when Servo is actually introduced in Stage 1 (Servo `MPL-2.0`, Cordis / Svelte / Vite `MIT`, Tauri `Apache-2.0 OR MIT`) |

The latest published terms govern; we **do not interpret them, guarantee availability, take responsibility for them, or endorse them**. The current version uploads no user data to these services (see `PRIVACY.md`).

## 9. Intellectual property, name and marks

- The scope of Layer A and Layer B is governed by `LICENSE` / `LICENSE-COMMERCIAL` (§2).
- "Fenestra", the shell mark and their combinations are the copyright holder's marks (§7): unmodified, non-commercial redistribution that does not alter §6 behaviour may keep the name with attribution; **a derivative version that alters any §6 behaviour or is offered commercially must not imply in its name, icon, store title or domain that it is official or endorsed, and must use its own name**. Rights separately granted by trademark law are unaffected.
- Contributions: contributions you submit are deemed agreed to be published under this licence and grant the copyright holder a perpetual, worldwide, free, non-exclusive right to use them in commercial versions; **you retain attribution, do not transfer copyright, and sign no CLA**, and the copyright holder may not separately relicense your contribution to a third party with your attribution removed (§8).
- Patents: §9 grants a non-exclusive, free patent licence necessary to exercise §3 rights; if you or an affiliate assert patent infringement over the software, that licence and the §3 licence terminate automatically. The patent scope in a commercial licence is agreed separately in writing; absent agreement it does not exceed §3.

## 10. Changes, termination and disputes

- Changes update this repository's `TERMS.md` directly and appear in Git history / CHANGELOG. We collect no contact details, so **we do not push notifications**; significant changes are announced in the repository. Continued use constitutes acceptance.
- **Licence versions are not retroactive**: §12.1 provides that a version you obtain is permanently governed by the licence text corresponding to that version at the time, and a new version published by the copyright holder does not retroactively revoke granted rights; §12.3 promises **published versions will not be re-closed** and granted rights will not be revoked; a §12.2 "Open Conversion Declaration" is irrevocable once published. The 2026-11-15 revision record of `LICENSE-COMMERCIAL` also expressly states that "versions obtained earlier remain governed by the licence text at the time of acquisition; this revision does not retroactively revoke granted rights".
- Termination: you may stop using it and delete local copies and plugins at any time; automatic termination and restoration of the licence follow §11 (see 6.4; **a commercial licence expiring automatically under §4.5 is not itself a breach, but continued commercial use after lapse under §4.6 is a breach, and §4.6(d)'s restoration rule takes priority**). §§0, 1, 8, 9, 10, 12, 13 survive termination.
- The copyright holder is not liable for unavailability caused by force majeure, third-party service termination, platform takedown, or dependency licence changes (§10).
- Disputes should first undergo **30 days of amicable negotiation**; failing that, under §13.3 either party may file suit in the **people's court of competent jurisdiction at the copyright holder's location**, **without arbitration**; the website login record, e-agreement signing log, licence certificate, payment record and the website's published current price may serve as valid evidence of licence status and fees; during dispute resolution §4.5 "automatic lapse" is not suspended; this licence is governed by the law of the copyright holder's location, and conflict-of-law rules do not apply. These terms are written in Chinese; the English text is for convenience only and **the Chinese controls** (§13.4). If any clause is held invalid, the rest remain effective and are interpreted as close to the original intent as possible (§13.1); this licence is the entire agreement between the parties regarding the software (§13.2).

## 11. Contact

- **Only contact channels**: open a **GitHub Issue** at `https://github.com/codecloud-dev/<repo>/issues`, or reach the GitHub account **`codecloud-dev`**.
- **Commercial licensing is NOT handled through GitHub Issues / Discussions / private messages / email**. Per `LICENSE-COMMERCIAL` §4.2 and its "At a glance" section, the **single channel** is: log in on the Fenestra official website → sign the 《Fenestra 商业授权电子协议》 → choose monthly or annual → the system issues a time-limited electronic licence certificate. Email, private messages, Discussions, and offline communication **do not issue** licences. That channel is not yet live (see 6.3); GitHub Issues / `codecloud-dev` are only for general feedback, security reports, and terms enquiries.
- No support email, phone, postal address, company name, or registration number is provided or invented here. Please do not post secrets, credentials, or licensing materials in a public Issue.
