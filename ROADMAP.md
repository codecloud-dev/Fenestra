# 路线图

> 版本：`v0.1.0-roadmap` · 最后复核：2026-11-15
> 原则：每条都写**验收方式**；没做到就标没做到，不写"预计很快"、"正在推进"。

## 总览

| Stage | 主题 | 状态 | 一句话验收 |
| --- | --- | --- | --- |
| **0** | 空壳内核与契约 | ✅ 已完成（2026-11-15） | 39 测试全绿、三平台 CI 绿、零依赖守门绿 |
| **1** | 装上引擎与界面，能上网 | ⬜ 未开始 | 三平台能打开真实网页并加载一个 Chrome MV3 扩展 |
| **2** | 安全与生态 | ⬜ 未开始 | OS 级隔离 + 插件签名 + 三平台 60fps 基准进 CI |
| **3** | 出名、好用 | ⬜ 未开始 | 第三方安全审计 + 稳定 API + 公开性能数据 |

---

## Stage 0 · 空壳内核与契约（已完成）

**交付清单（都有代码或 CI 可查）**

- [x] Rust 内核，`[dependencies]` 为空
- [x] FEP v1 清单格式与解析器，单例重复／未知字段／未知能力一律报错
- [x] 封闭能力枚举、`Grant::intersect` 最小权限交集、`Grant::sanitized`
- [x] `SandboxPolicy` 三重闸门、`harden()` 锁定模式、`weakened()` 检测
- [x] `detonate()` → `Untrusted<T>` → `revalidate()` 类型级出站重校验
- [x] `Registry` 可逆注册，`revoke` 精确撤销服务／订阅／能力
- [x] 自研迷你 JSON 解析器
- [x] Chrome / Edge / Firefox `manifest.json` → FEP 归一器 + 差异 warnings
- [x] `to_hep()` 双向可审计
- [x] 五个平台 target 声明与 Rust triple
- [x] CLI：`verify` / `compat` / `demo` / `policy` / `targets`
- [x] 39 测试（33 单测 + 6 集成），`cargo fmt --check`、`clippy -D warnings`
- [x] CI：Linux / Windows / macOS 三平台 + Android / iOS target 编译检查 + 零依赖守门
- [x] 自撰 FSL v1.0 许可证，五不丢写成可判定条款
- [x] 全套 `.md` 文档与联网调研（DSH / Apple 沙盒 / 2026 引擎格局）

**验收命令**

```bash
cargo test --all-targets      # 33 + 6 = 39 passed
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo run -- demo
```

**明确未交付**：图形界面、真实引擎插件、能上网、安装包、插件商店、OS 级沙盒。

---

## Stage 1 · 装上引擎与界面，能上网

目标：证明"壳"真的能承载一个引擎，而不是只有 trait。

| 工作项 | 验收方式 |
| --- | --- |
| 第一个 `RenderEngine` 插件（先接成熟引擎的嵌入接口，不自研引擎） | `cargo run -- demo` 能产出真实 `Frame`，非 stub |
| 第一个 `NetworkStack` 插件 | 能 fetch 一个真实 URL，且经过 `net.connect` 仲裁 |
| 最小 `ShellUi` 插件（地址栏 + 标签页 + 窗口） | 三平台可启动、可输入 URL、能看到页面 |
| `ExtensionRuntime` 插件 + `chrome.*` / `browser.*` Promise 统一垫片 | 一个真实 Chrome MV3 扩展在三平台跑通 |
| 五平台安装包 | Android APK、Windows、Linux、macOS 各出一个可安装产物；iOS 按系统规则处理引擎选择 |
| FEP ABI 冻结声明 | 发布 `FEP v1` 兼容性文档与回归测试，承诺 Stage 1 内不破坏 |

**门槛**：`net.connect` 未被授予时，引擎与网络插件**必须**无法出网——要有自动化测试，不能只靠代码阅读。

---

## Stage 2 · 安全与生态

| 工作项 | 验收方式 |
| --- | --- |
| 平台后端插件实现真实进程隔离（Linux seccomp/Landlock、Windows AppContainer、Apple Seatbelt） | 每个平台有独立测试用例，证明隔离进程无法读用户目录 |
| 站点隔离策略 | 跨站 iframe 不能读到彼此存储，有回归测试 |
| 锁定模式禁用 JIT 的真实落地 | 引擎插件在 `harden()` 后确实不创建可执行内存 |
| 插件签名与来源验证 | 未签名插件默认拒绝安装；签名可离线校验 |
| 出站流量可观测与阻断 | 未经 `net.connect` 的出站连接被记录并可阻断 |
| 插件目录／商店（**不锁定单一来源**） | 支持任意 URL 与本地文件安装，符合 LICENSE 第 6.3 条 |
| 三平台 60fps 与输入延迟基准 | 帧时间、输入延迟、冷启动时间进 CI，回归超阈值即失败 |
| 五不丢逐条真机验收 | 对照 [PRINCIPLES.md](PRINCIPLES.md) 的可验证动作逐项打勾 |

**门槛**：这一阶段之前，README 里不会出现"流畅"的性能数字。

---

## Stage 3 · 出名、好用

| 工作项 | 验收方式 |
| --- | --- |
| 稳定公开 API 与插件 SDK | 语义化版本承诺 + 兼容性测试套件 |
| 第三方安全审计 | 公开审计报告与修复记录 |
| 公开性能数据与对比 | 与主流浏览器的冷启动／内存／帧时间对比，方法可复现 |
| 生态：官方插件集（书签、历史、下载、设置、主题、密码） | 每个都是可卸载的独立插件，删光后浏览器仍能启动 |
| 社区治理与商业授权通道 | LICENSE 第 4.2 条的小额授权流程可在线完成 |
| 《开放转换声明》评估 | 按 LICENSE 第 12.2 条三个条件公开评估结论 |

---

## 贯穿全程的约束

1. **内核依赖永远为零**：任何新增依赖都必须先改这条路线图并说明理由。
2. **五不丢不可协商**：每个 Stage 收尾都重跑 [PRINCIPLES.md](PRINCIPLES.md) 的可验证动作。
3. **不写假绿**：CI 红就标红，功能没做就标没做。历史上"测不存在的接口还报 27/27 通过"这种事不允许再发生。
4. **平台一等公民**：手机／Windows／Linux 不是"以后适配"，是同一套内核的五个 target，CI 从 Stage 0 就开始查。
5. **对 DSH 的态度**：架构上比它更绝，产品上向它的生态看齐；不因为架构优越就假装自己更成熟。

## 变更记录

| 日期 | 变更 |
| --- | --- |
| 2026-11-15 | 建立路线图；Stage 0 完成并逐项验收；明确标注 Stage 1–3 未开始 |
