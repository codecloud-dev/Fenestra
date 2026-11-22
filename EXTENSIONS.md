# FEP 规范 / Fenestra Extension Protocol

> 层 A 覆盖 FEP 规范文本（见 [LICENSE](./LICENSE)）。本文描述 **Stage 1 自有插件**的最小协议；Chrome / Edge / Firefox 三家兼容是 Stage 2 硬验收项，见 [ROADMAP.md](./ROADMAP.md)。

## 1. 定位

自有扩展是**一等公民**：能力最全、性能最好。三家扩展未来通过兼容层映射，允许性能差异，功能覆盖逐条列进文档。MVP 六件事不含兼容层，但插件内核 API 从第一天按「自有 + 三家可映射」设计，不把路堵死。

## 2. 插件定义（TypeScript）

插件是一个 `PluginDefinition`（见 [packages/core/src/index.ts](./packages/core/src/index.ts)）：

```ts
import type { PluginDefinition } from '@fenestra/core'

export const myPlugin: PluginDefinition = {
  meta: {
    name: 'example.my-plugin',
    version: '0.1.0',
    description: '……',
    permissions: ['ui.newtab'], // 只声明，不自动授予
  },
  apply(ctx, api) {
    api.effect('label', () => { /* setup */ }, () => { /* cleanup，逆序执行 */ })
  },
}
```

- `meta.permissions`：**声明 ≠ 授权**。是否放行由 [crates/gate](./crates/gate/) 按默认拒绝 + 最小权限交集裁决。
- `apply(ctx, api)`：`ctx` 是 Cordis `Context`，`api` 是 `FenestraApi`。

## 3. FenestraApi（全部可逆）

| 方法 | 作用 | 卸载时 |
| --- | --- | --- |
| `provide<T>(name, value)` | 登记一个服务 | 自动移除 |
| `get<T>(name)` | 读取当前可见服务 | — |
| `effect(label, setup, cleanup)` | 登记 setup/cleanup 对 | cleanup **逆序**执行 |
| `on(event, listener)` | 登记事件监听 | 自动移除 |
| `permissions()` | 返回本插件声明的能力 | — |

插件登记的一切挂在自己的 Fiber 上，`fiber.dispose()` 逆序回滚、无残留（[packages/core](./packages/core/) 11 个测试）。**禁止**把状态挂到插件作用域之外，否则卸载会残留。

## 4. 生命周期

`load`（登记，不执行）→ `enable`（挂 Fiber、await 生效）→ `disable`（dispose、逆序回滚）→ `unload`（删除定义与记录）。`install = load + enable`；disable 后可再 enable，**进程不重启**，满足「现安装、现跑、现用、完全不需要重启」。

## 5. 示例插件

| 插件 | 目录 | 验证的承诺 |
| --- | --- | --- |
| 默认新标签页 | [packages/plugins/newtab](./packages/plugins/newtab/) | 提供 `ui/newtab` 服务 |
| 示例替换 | [packages/plugins/example-replace](./packages/plugins/example-replace/) | 同名服务覆盖默认；卸载靠可逆登记自动恢复 |
| DNS 加速 | [packages/plugins/accelerate](./packages/plugins/accelerate/) | DNS 缓存 + 悬停预连接意图登记，卸载清空无残留 |

三个插件 `pnpm run plugins:typecheck` 通过；端到端「替换 / 恢复」演示测试尚未写，见 [README.md](./README.md) 状态表。

## 6. 能力命名与隐私红线

能力用点分字符串（如 `ui.newtab`、`net.dns`、`net.preconnect`）。涉及剪贴板 / 档案 / 摄像头 / 位置 / 遥测的能力属于隐私红线，须用户逐项显式确认，默认拒绝，见 [SECURITY.md](./SECURITY.md) 与 [crates/gate](./crates/gate/) 的 `Capability`。

## 7. MV3 归一层（Stage 2 进行中）

- `crates/gate/src/mv3.rs` 已落地 Manifest V3 清单解析、合规校验与能力归一：
  - 解析 `manifest_version` / `permissions` / `host_permissions` / `background.service_worker` / `action` 等字段；
  - 校验 MV3 硬约束（必须为 v3、禁用 `webRequestBlocking`、主机权限不得混入 `permissions`）；
  - 把 MV3 权限按默认拒绝映射为 `Capability`，未知权限一律忽略，不猜测放行；
  - 提供 `to_fep_permissions()` 供兼容层把扩展登记进 Fenestra 运行时。
- 当前只做「清单 → 能力」的归一与守门；三家 API 运行时代理仍属后续工作。
- 自有插件在能力与性能上保持最优（一等公民）；兼容层允许性能差异。

设计原则见 [PRINCIPLES.md](./PRINCIPLES.md)，架构见 [ARCHITECTURE.md](./ARCHITECTURE.md)。
