# 架构：一个只有三件事的内核

## 1. 边界定义

Fenestra 的架构由一条**否定式**约束定义：

> 内核里不允许存在任何"用户能感知到的浏览器功能"。

书签不是功能吗？是，所以它是插件。地址栏不是吗？是，所以它是插件。渲染引擎是不是？也是。下载、历史、密码管理、设置页、主题、协议处理、扩展运行时、沙盒策略本身——全部是插件。

内核剩下的、无法再被剥离的三件事：

| 职责 | 代码 | 为什么不能再拆 |
| --- | --- | --- |
| 读清单 | [`src/manifest.rs`](src/manifest.rs) | 总得有人知道要加载什么；拆掉就没有"安装"这个概念 |
| 仲裁能力 | [`src/capability.rs`](src/capability.rs) + [`src/sandbox.rs`](src/sandbox.rs) | 授权判定必须是所有插件都绕不过去的共同上游 |
| 可逆登记 | [`src/registry.rs`](src/registry.rs) | 卸载不留悬空状态，需要一个全局一致的注册账本 |

这三件事合起来约 1000 行纯 `std` Rust，零第三方依赖。

## 2. 分层

```
┌──────────────────────────────────────────────────────────┐
│  用户能看见的一切（全部是插件）                             │
│  shell-ui / theme / settings / bookmarks / history /       │
│  downloader / protocol / extension-runtime / ...           │
├──────────────────────────────────────────────────────────┤
│  能力提供者（全部是插件）                                   │
│  render-engine (Blink/WebKit/Gecko/Servo/自研 可并列安装)   │
│  network-stack / storage / platform-backend                │
│  sandbox-policy                                            │
├──────────────────────────────────────────────────────────┤
│  兼容层（只读归一，不执行扩展代码）                         │
│  Chrome MV3 / Edge MV3 / Firefox MV3 → FEP                 │
│  src/compat.rs（自带迷你 JSON 解析器）                      │
├──────────────────────────────────────────────────────────┤
│  Fenestra 内核（唯一不可替换的部分）                           │
│  Manifest.parse → Grant.intersect → Registry.register      │
│  SandboxPolicy.decide → Verdict                            │
│  Registry.revoke（精确撤销）                                │
└──────────────────────────────────────────────────────────┘
```

注意方向：**兼容层在内核之上，不在内核之内**。三家扩展的方言差异由插件侧归一，内核只认 FEP。这样将来出现第四家扩展格式时，改的是插件，不是内核。

## 3. FEP 清单格式

设计取舍：不用 JSON 作为内核清单格式。内核要能在任何 JS 运行时、任何渲染引擎启动**之前**读懂自己该加载什么，因此格式必须是纯文本、逐行、无歧义、可被审计、可被别的语言重新实现。

```
id: com.example.reader          # 必填，反向域名式，允许 @（兼容 Firefox gecko id）
name: Reader                    # 必填
version: 1.0.0                  # 必填，x.y.z
kind: shell-ui                  # 必填
entry: reader.wasm              # 必填
target: linux                   # 可选，缺省为任意平台
capability: storage.read        # 可多次
capability: ui.chrome
origin: https://example.com/*   # 可多次
```

规则：

- `#` / `//` 注释，空行忽略；
- 单例字段重复 → 报错（不允许"后一个覆盖前一个"这种含糊语义）；
- 未知字段 → 报错（宁可拒绝，也不悄悄忽略一个可能是安全声明的字段）；
- 未知能力 → 报错（权限膨胀必须显式改内核并升 FEP 版本）；
- `capability` / `origin` 自动去重。

`kind` 枚举里刻意包含 `render-engine`、`network-stack`、`sandbox-policy`——这是与 DSH 的关键区别，见 [`docs/research/dsh-everything-is-a-plugin.md`](docs/research/dsh-everything-is-a-plugin.md)。

## 4. 能力模型：三重闸门

安装时的**最小权限交集**：

```
实际能力 = 插件声明的能力 ∩ 用户授予的能力       (Grant::intersect)
```

运行时的三重闸门（`SandboxPolicy::decide`）：

1. 锁定模式 + 安全红线能力（`engine.swap` / `network.swap` / `sandbox.read` / `net.intercept`）→ `Deny`；
2. 隐私红线能力 + 未经用户显式确认 → `Deny`；
3. 授权里没有 → `Deny`（默认拒绝）。

隐私红线：`telemetry`、`profile.read`、`clipboard.read`、`device.camera`、`device.location`。

`Grant::sanitized(false)` 是给 UI 用的便捷工具：一次剔除所有隐私红线，保证"用户没逐项点头"时它们根本进不了注册表。

## 5. 可逆注册

Cordis 的 reversible effects 是个好设计，Fenestra 借了思路但换了地基：插件登记的不是"我注册了什么"，而是**内核记下的一组可撤销效应**。

```rust
Registration {
    id,
    plugin_id,
    kind,
    capabilities,   // 实际生效的最小权限交集
    services,       // 由 kind 推导，如 engine.render
    subscriptions,  // 由 kind 推导，如 shell.boot
}
```

`revoke(id)` 用一条 `retain` 把服务、订阅、能力同时抹掉。测试 `registry::tests::revoke_removes_services_and_subscriptions` 与 `tests::uninstall_is_reversible` 验证撤销后 `services()` 为空、`publish("shell.boot")` 为空。

好处是热重载：换引擎 = `revoke(旧)` + `register(新)`，没有"旧插件残留的监听器还在跑"这类幽灵状态。

## 6. 沙盒：策略层与隔离层分离

Stage 0 只实现**策略层与仲裁层**，不假装已有 OS 级隔离。

- `IsolationLevel::{None, Process, Site, BlastDoor}` 是策略声明；
- `SandboxPolicy::weakened()` 让"策略被削弱"成为可检测事实（许可证第 6.2 条的落地）；
- `detonate()` 返回 `Untrusted<T>` 而不是 `T`：无法直接拿走结果，必须 `revalidate()`；
- `harden()` 进入锁定模式，冻结所有可改变隔离边界的能力。

真正创建进程、应用 seccomp / AppContainer / Seatbelt 的是**平台后端插件**（`PlatformBackend` trait），不同平台不同实现——这也是它必须是插件的原因。

## 7. 可插拔边界（trait 一览）

[`src/engine.rs`](src/engine.rs) 里只有接口，没有任何实现：

| Trait | 关键方法 | 谁来实现 |
| --- | --- | --- |
| `RenderEngine` | `render(RenderRequest) -> Frame` | 引擎插件 |
| `NetworkStack` | `fetch(url) -> Vec<u8>` | 网络插件 |
| `ShellUi` | `mount(slot)` | UI 插件 |
| `PlatformBackend` | `target() -> Target` | 平台插件 |
| `ExtensionRuntime` | `load(package)` | 扩展运行时插件 |

`Target::SUPPORTED` 覆盖 Android / iOS / Windows / Linux / macOS，每个都带 `rust_target_triple()`，CI 会真的对每个 target 做编译检查。

## 8. 关键设计决策（ADR 摘要）

| 编号 | 决策 | 理由 | 代价 |
| --- | --- | --- | --- |
| ADR-01 | 内核零第三方依赖 | 启动早于任何运行时；可审计；五平台可编译 | JSON、清单、注册表全部自己写 |
| ADR-02 | FEP 用纯文本而非 JSON | 内核不该依赖解析器复杂度 | 生态工具要重新实现一个小解析器 |
| ADR-03 | 能力枚举封闭在内核 | 权限膨胀需要被审视，不能由插件自称 | 新能力要发新版内核 |
| ADR-04 | 兼容层在内核之上 | 第四家扩展格式出现时不动内核 | 内核只认 FEP，转译有成本 |
| ADR-05 | 沙盒策略本身是插件 | "对标苹果"不等于绑死一种 OS 沙盒 | 需要 `weakened()` 守门防止被削弱 |
| ADR-06 | `Untrusted<T>` 类型级不可信 | 约定会忘，类型不会 | API 略啰嗦 |
| ADR-07 | 不内置任何引擎 | 用户要能换掉一切 | Stage 0 不能上网，路线图必须说清 |
| ADR-08 | 许可限制商用但强制改版公开 | 保护作者收益，同时防止生态被闭源收割 | 不是 OSI 开源，社区贡献会受限，已在 LICENSE 第 0 条明说 |

## 9. 数据流（一次导航）

```
用户在 UI 插件里输入 URL
  → UI 插件调用 ext/nav 服务
  → 内核不做任何事（它不管导航）
  → 扩展运行时插件按 flavor 处理
  → 网络栈插件 fetch（需 net.connect，过 decide()）
  → 渲染引擎插件 render（产出 Frame）
  → Frame 经 Untrusted 包装，UI 侧 revalidate 后才上屏
  → 引擎插件 publish("engine.frame")，UI 插件订阅
```

内核全程只做了一件事：在 `net.connect` 那一步说了句 `Allow` 或 `Deny`。这就是"壳"的字面意思。
