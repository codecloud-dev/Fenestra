/**
 * Fenestra 插件运行时（层 A）
 *
 * 底座是 Cordis 4 的可逆 effect（revertible effects）：
 *   - 插件通过 `api.effect / api.provide / api.on` 登记的一切，都挂在自己那根
 *     Fiber 的子 Context 上；Fiber 一 dispose，全部按登记逆序自动回滚，无残留。
 *   - 运行时本身是一个长期存活的 Context，装/卸插件只是增删 Fiber，
 *     **进程不重启**：这就是「现安装、现跑、现用、卸载立即回滚」。
 *
 * 注意：这里只负责「生命周期可逆」。能力能否被使用，由 Rust 闸门（crates/gate）
 * 按默认拒绝 + 最小权限交集裁决；本层只把插件声明的权限如实交给上层核对。
 */

import { Context } from 'cordis'
import type { Component, Fiber } from 'cordis'

export interface PluginMeta {
  name: string
  version?: string
  description?: string
  /** 插件声明想要的能力。声明 ≠ 授权：是否放行由闸门裁决。 */
  permissions?: string[]
}

/** 插件可用的最小 API。所有登记都是可逆的，随插件卸载自动回收。 */
export interface FenestraApi {
  /** 可逆登记一个服务（同名会覆盖；卸载后自动移除）。 */
  provide<T>(name: string, value: T): void
  /** 读取当前可见的服务。 */
  get<T>(name: string): T | undefined
  /** 可逆 effect：setup 在启用时执行，cleanup 在卸载时按逆序执行。 */
  effect(label: string, setup: () => void, cleanup: () => void): void
  /** 可逆事件监听（卸载后自动移除）。 */
  on(event: string, listener: (...args: unknown[]) => void): void
  /** 本插件声明的能力。 */
  permissions(): string[]
}

export interface PluginDefinition {
  meta: PluginMeta
  apply: (ctx: Context, api: FenestraApi) => void
}

/** 定义一个插件（仅做类型收拢，无运行时魔法）。 */
export function definePlugin(def: PluginDefinition): PluginDefinition {
  return def
}

export type PluginState = 'loaded' | 'enabled' | 'disabled'

export interface PluginRecord {
  id: string
  meta: PluginMeta
  state: PluginState
}

/**
 * 插件运行时。一个实例 = 一个长期存活的 Cordis Context。
 * load / enable / disable / unload 四态，支持热装与逆序回滚。
 */
export class PluginRuntime {
  /** 根 Context：构造后永不重建，热装插件不会重启进程。 */
  readonly ctx: Context
  /** 生命周期审计日志，供测试与插件管理页核对「登记 / 逆序回滚」顺序。 */
  readonly auditLog: string[] = []

  private readonly defs = new Map<string, PluginDefinition>()
  private readonly fibers = new Map<string, Fiber>()
  private readonly records = new Map<string, PluginRecord>()
  /** 正在启用中的插件 → 其 enable Promise，用于并发 enable 去重。 */
  private readonly enabling = new Map<string, Promise<PluginRecord>>()

  constructor(ctx: Context = new Context()) {
    this.ctx = ctx
  }

  /** 登记插件定义，不执行插件代码。 */
  load(id: string, def: PluginDefinition): PluginRecord {
    if (this.defs.has(id)) {
      throw new Error(`plugin already loaded: ${id}`)
    }
    this.defs.set(id, def)
    const record: PluginRecord = { id, meta: def.meta, state: 'loaded' }
    this.records.set(id, record)
    this.auditLog.push(`load:${id}`)
    return record
  }

  /**
   * 启用插件：在同一 Context 上挂一根新 Fiber 并等待生效。
   * 幂等；重复 enable（含并发调用）不会重复注册 effect。
   */
  async enable(id: string): Promise<PluginRecord> {
    // 同一个 id 的并发 enable 共享第一次的结果，避免重复挂 Fiber。
    const inflight = this.enabling.get(id)
    if (inflight) {
      return inflight
    }
    if (this.fibers.has(id)) {
      return this.records.get(id)!
    }
    const task = this.doEnable(id)
    this.enabling.set(id, task)
    try {
      return await task
    } finally {
      // 只清自己那一轮，避免误删后续重入新建的 task。
      if (this.enabling.get(id) === task) {
        this.enabling.delete(id)
      }
    }
  }

  private async doEnable(id: string): Promise<PluginRecord> {
    const def = this.defs.get(id)
    if (!def) {
      throw new Error(`plugin not loaded: ${id}`)
    }
    const component: Component.Function = (childCtx) => {
      const api = this.createApi(childCtx, id, def.meta)
      def.apply(childCtx, api)
    }
    const fiber: Fiber = this.ctx.use(component)
    await fiber
    this.fibers.set(id, fiber)
    const record = this.records.get(id)!
    record.state = 'enabled'
    this.auditLog.push(`enable:${id}`)
    return record
  }

  /** 现装现跑：load + enable 一步到位，进程不重启。 */
  async install(id: string, def: PluginDefinition): Promise<PluginRecord> {
    this.load(id, def)
    return this.enable(id)
  }

  /** 禁用：dispose Fiber，逆序回滚该插件全部登记；保留定义，可再次启用。 */
  async disable(id: string): Promise<void> {
    const fiber = this.fibers.get(id)
    if (!fiber) {
      return
    }
    await fiber.dispose()
    this.fibers.delete(id)
    const record = this.records.get(id)
    if (record) {
      record.state = 'disabled'
    }
    this.auditLog.push(`disable:${id}`)
  }

  /** 卸载：禁用 + 删除定义与记录，注册表 / 服务 / 监听全部无残留。 */
  async unload(id: string): Promise<void> {
    await this.disable(id)
    this.defs.delete(id)
    this.records.delete(id)
    this.auditLog.push(`unload:${id}`)
  }

  get(id: string): PluginRecord | undefined {
    return this.records.get(id)
  }

  list(): PluginRecord[] {
    return [...this.records.values()]
  }

  /** 插件声明的能力（声明 ≠ 授权，供闸门按默认拒绝核对）。 */
  permissionsOf(id: string): string[] {
    return [...(this.defs.get(id)?.meta.permissions ?? [])]
  }

  private createApi(ctx: Context, id: string, meta: PluginMeta): FenestraApi {
    const on = ctx.on.bind(ctx) as unknown as (
      name: string,
      cb: (...args: unknown[]) => void,
    ) => void
    return {
      provide: <T>(name: string, value: T): void => {
        ctx.provide(name, value)
      },
      get: <T>(name: string): T | undefined => ctx.get(name) as T | undefined,
      effect: (label, setup, cleanup): void => {
        ctx.effect(
          () => {
            setup()
            this.auditLog.push(`setup:${id}:${label}`)
            return () => {
              cleanup()
              this.auditLog.push(`cleanup:${id}:${label}`)
            }
          },
          `${id}:${label}`,
        )
      },
      on: (event, listener): void => on(event, listener),
      permissions: () => [...(meta.permissions ?? [])],
    }
  }
}
