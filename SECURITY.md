# 安全设计与威胁模型 / Security

> 内核唯一钉死的不变量，落在 [crates/gate](./crates/gate/)（纯 Rust、零第三方依赖、可独立 `cargo test`）。本文说明威胁模型、不变量、隐私承诺，以及**当前尚未做完**的部分。

## 1. 信任边界

```
不可信：网页内容 / 第三方插件 / 隔离单元输出 / 网络入站数据
   │  （必须经过闸门与出站重校验）
可信：壳本体（窗口、地址栏、插件运行时调度）/ Rust 闸门 / 用户显式确认
```

浏览器里运行的网页、插件与扩展由各自作者负责，不由 Fenestra 担保；安装第三方插件的风险由用户承担。

## 2. 五个不变量（18 个测试）

| 不变量 | 实现 | 测试 |
| --- | --- | --- |
| 默认拒绝 | `Gate::check()` 未声明 / 超策略 / 未确认一律 `Deny` | `default_deny_*` / `undeclared_*` |
| 最小权限交集 | `Gate::effective()` = 声明 ∩ 策略 ∩ 确认 | `effective_is_minimum_privilege_intersection` |
| 隐私红线逐项确认 | `is_privacy_sensitive()` + `confirm()` | `privacy_*` / `consent_is_granted_per_capability` |
| BlastDoor 出站重校验 | `Untrusted<T>` → `revalidate()` → `Trusted<T>`，无 bypass | `untrusted_*` / `revalidate_*` |
| 可逆登记 | `RevertibleRegistry` / `Gate::revoke()` 逆序回滚无残留 | `registry_*` / `gate_revoke_*` / `revoke_isolates_plugins` |

TS 侧的可逆性由 Cordis 4 revertible effects 保证（[packages/core](./packages/core/) 11 个测试：逆序 cleanup、服务/监听自动回收、卸载无残留、多插件隔离、热装不重启）。

## 3. 隐私红线 / Privacy

以下能力默认拒绝，须用户**逐项、显式**确认，确认不连带：

`ClipboardRead` · `ProfileRead` · `CameraDevice` · `LocationDevice` · `Telemetry`

- 不默认收集或上传浏览历史、页面内容、剪贴板、位置、凭据、设备指纹；
- 不植入未经用户明示同意且可随时关闭的遥测；
- 纯本地、零运营成本、不收任何用户数据、不做账号 / 云同步。

层 B（[LICENSE-COMMERCIAL](./LICENSE-COMMERCIAL)）的不可退化条款同样锁死这些红线，付钱也不得弱化；层 A [LICENSE](./LICENSE) 第 5 条把「默认拒绝 / 沙箱隔离 / 隐私红线 / 出站重校验 / 可逆登记」列为永久开源不变量。

## 4. 沙箱与出站重校验

隔离单元（网页 / 插件输出）产出的数据被包成 `Untrusted<T>`：该类型**不实现 `Deref` / `AsRef`，不提供 `into_inner`**，外部无法绕过校验取值。离开隔离边界前必须调用 `revalidate(validator)`，校验器通过内容策略（scheme 白名单、结构完整性、长度上限、是否夹带未声明字段等）后才返回 `Trusted<T>`；校验失败，值随对象一同丢弃。

`#[forbid(unsafe_code)]` 在 [crates/gate/src/lib.rs](./crates/gate/src/lib.rs) 顶层声明。

## 5. 权限清单与 Tauri PCS

[apps/shell/src-tauri/capabilities/default.json](./apps/shell/src-tauri/capabilities/default.json) 只授予 `core:default`（主窗口与 WebView 基础权限）；核心能力一律经 Rust 闸门按默认拒绝裁决。`gate_check` IPC 命令当前是**默认拒绝示意**，生产版改为调用 `Gate::check`。

## 6. 当前尚未做完（如实）

- Rust 闸门 ↔ TS 插件运行时 ↔ Tauri IPC ↔ Svelte UI 的端到端对接未打通；
- 敏感操作的**清晰授权弹窗**未实现（插件管理页权限数据当前为本地镜像）；
- 真实网页渲染由系统 WebView 承载，隔离边界的最终形态待端到端集成后确定；
- 沙箱隔离的进程 / 站点级落地依赖 Tauri 外壳集成，本地未编译验证。

各包独立测试通过不等于端到端安全闭环，见 [README.md](./README.md) 状态表。

## 7. 报告漏洞

发现漏洞请通过仓库 issue 或私信联系 `codecloud-dev`；未修复的严重漏洞请勿公开开 issue。我们遵循诚实原则：相关测试 / CI 未真正通过前，不会声称修复已验证。

原则见 [PRINCIPLES.md](./PRINCIPLES.md)，架构见 [ARCHITECTURE.md](./ARCHITECTURE.md)。
