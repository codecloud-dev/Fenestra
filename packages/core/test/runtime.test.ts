import { test } from 'node:test'
import assert from 'node:assert/strict'
import { Context } from 'cordis'
import {
  PluginRuntime,
  definePlugin,
  type PluginDefinition,
} from '../src/index.ts'

// 让自定义事件名通过 cordis 的 keyof Events 约束（运行时本就支持字符串事件）。
declare module 'cordis' {
  interface Events {
    'fenestra/ping'(): void
  }
}

function mk(name: string, permissions: string[] = []): PluginDefinition {
  return definePlugin({
    meta: { name, permissions },
    apply: (_ctx, api) => {
      api.effect(
        'svc',
        () => api.provide(`svc/${name}`, name),
        () => {},
      )
    },
  })
}

// 1. 现装现跑 / 热装不重启
test('install 立即生效，且根 Context 不重建（热装，进程不重启）', async () => {
  const runtime = new PluginRuntime()
  const ctxBefore = runtime.ctx
  const plugin = definePlugin({
    meta: { name: 'newtab-default', permissions: ['ui.newtab'] },
    apply: (_ctx, api) => {
      api.effect('render', () => api.provide('newtab', { title: 'Fenestra' }), () => {})
    },
  })
  const record = await runtime.install('newtab', plugin)
  assert.equal(record.state, 'enabled')
  assert.deepEqual(runtime.ctx.get('newtab'), { title: 'Fenestra' })
  assert.equal(runtime.ctx, ctxBefore)
  assert.ok(runtime.auditLog.includes('setup:newtab:render'))
  assert.deepEqual(runtime.permissionsOf('newtab'), ['ui.newtab'])
})

// 2. 可逆 effect / 逆序卸载
test('disable 时多个 effect 按注册逆序回滚（LIFO，无残留）', async () => {
  const runtime = new PluginRuntime()
  const plugin = definePlugin({
    meta: { name: 'order' },
    apply: (_ctx, api) => {
      for (const label of ['A', 'B', 'C']) {
        api.effect(label, () => {}, () => {})
      }
    },
  })
  await runtime.install('order', plugin)
  assert.equal(runtime.auditLog.filter((e) => e.startsWith('cleanup:')).length, 0)
  await runtime.disable('order')
  assert.deepEqual(runtime.auditLog.filter((e) => e.startsWith('cleanup:')), [
    'cleanup:order:C',
    'cleanup:order:B',
    'cleanup:order:A',
  ])
  assert.equal(runtime.get('order')?.state, 'disabled')
})

// 3. 卸载后服务 / 监听 / 登记全部无残留
test('unload 逆序回滚且服务、监听、记录全部无残留', async () => {
  const runtime = new PluginRuntime()
  const hits: number[] = []
  const plugin = definePlugin({
    meta: { name: 'residue' },
    apply: (_ctx, api) => {
      api.effect(
        'svc',
        () => {
          api.provide('accel/dns', { cache: 'on' })
          api.on('fenestra/ping', () => {
            hits.push(1)
          })
        },
        () => {},
      )
    },
  })
  await runtime.install('residue', plugin)
  runtime.ctx.emit('fenestra/ping')
  assert.equal(hits.length, 1)
  assert.deepEqual(runtime.ctx.get('accel/dns'), { cache: 'on' })

  await runtime.unload('residue')
  assert.equal(runtime.ctx.get('accel/dns'), undefined)
  runtime.ctx.emit('fenestra/ping')
  assert.equal(hits.length, 1, '监听已被移除，不再收到事件')
  assert.equal(runtime.get('residue'), undefined)
  assert.equal(runtime.list().length, 0)
  assert.ok(runtime.auditLog.includes('cleanup:residue:svc'))
})

// 4. 禁用后再次启用，仍在同一 Context 生效（不重启）
test('disable 后可再次 enable，仍在同一 Context 生效', async () => {
  const runtime = new PluginRuntime()
  const ctxBefore = runtime.ctx
  let setups = 0
  const plugin = definePlugin({
    meta: { name: 'toggle' },
    apply: (_ctx, api) => {
      api.effect(
        'work',
        () => {
          setups += 1
          api.provide('toggle/state', setups)
        },
        () => {},
      )
    },
  })
  await runtime.install('toggle', plugin)
  assert.equal(runtime.ctx.get('toggle/state'), 1)
  await runtime.disable('toggle')
  assert.equal(runtime.ctx.get('toggle/state'), undefined)
  await runtime.enable('toggle')
  assert.equal(runtime.ctx.get('toggle/state'), 2)
  assert.equal(setups, 2)
  assert.equal(runtime.ctx, ctxBefore)
  assert.equal(runtime.get('toggle')?.state, 'enabled')
})

// 5. 多插件隔离
test('卸载一个插件不影响其它正在运行的插件', async () => {
  const runtime = new PluginRuntime()
  await runtime.install('a', mk('a'))
  await runtime.install('b', mk('b'))
  assert.equal(runtime.ctx.get('svc/a'), 'a')
  assert.equal(runtime.ctx.get('svc/b'), 'b')
  await runtime.unload('a')
  assert.equal(runtime.ctx.get('svc/a'), undefined)
  assert.equal(runtime.ctx.get('svc/b'), 'b')
  assert.equal(runtime.list().length, 1)
  assert.equal(runtime.get('b')?.state, 'enabled')
})

// 6. load 不执行，enable 才生效
test('仅 load 不执行 effect，enable 后才生效', async () => {
  const runtime = new PluginRuntime()
  let ran = false
  const plugin = definePlugin({
    meta: { name: 'lazy' },
    apply: (_ctx, api) => api.effect('x', () => { ran = true }, () => {}),
  })
  const rec = runtime.load('lazy', plugin)
  assert.equal(rec.state, 'loaded')
  assert.equal(ran, false)
  await runtime.enable('lazy')
  assert.equal(ran, true)
  assert.equal(runtime.get('lazy')?.state, 'enabled')
})

// 7. 权限只声明、不自动授予（默认拒绝交给闸门）
test('权限只作声明，runtime 不自动授予', async () => {
  const runtime = new PluginRuntime()
  const plugin = definePlugin({
    meta: { name: 'accel', permissions: ['net.dns', 'net.preconnect'] },
    apply: () => {},
  })
  await runtime.install('accel', plugin)
  assert.deepEqual(runtime.permissionsOf('accel'), ['net.dns', 'net.preconnect'])
  assert.deepEqual(runtime.permissionsOf('missing'), [])

  let seen: string[] = []
  const p2 = definePlugin({
    meta: { name: 'p2', permissions: ['ui.newtab'] },
    apply: (_c, api) => {
      seen = api.permissions()
    },
  })
  await runtime.install('p2', p2)
  assert.deepEqual(seen, ['ui.newtab'])
})

// 8. 重复 load 报错 / 重复 enable 幂等
test('重复 load 报错，重复 enable 幂等（effect 不重复注册）', async () => {
  const runtime = new PluginRuntime()
  let setups = 0
  const plugin = definePlugin({
    meta: { name: 'idem' },
    apply: (_c, api) => api.effect('x', () => { setups += 1 }, () => {}),
  })
  runtime.load('idem', plugin)
  assert.throws(() => runtime.load('idem', plugin), /already loaded/)
  await runtime.enable('idem')
  await runtime.enable('idem')
  assert.equal(setups, 1)
  assert.equal(runtime.get('idem')?.state, 'enabled')
})

// 8b. 并发 enable 幂等（真实 bug 回归）：同一 id 同时两次 enable 只挂一次 Fiber。
test('并发 enable 幂等：apply 只执行一次，effect 不重复注册', async () => {
  const runtime = new PluginRuntime()
  let setups = 0
  const plugin = definePlugin({
    meta: { name: 'race' },
    apply: (_c, api) => api.effect('e', () => { setups += 1 }, () => {}),
  })
  runtime.load('race', plugin)
  // 不 await 第一次，制造两次 enable 在 doEnable 完成前的并发重叠。
  const [a, b] = await Promise.all([runtime.enable('race'), runtime.enable('race')])
  assert.equal(a, b, '两次并发 enable 返回同一条记录')
  assert.equal(setups, 1, 'apply 只执行一次')
  assert.equal(runtime.auditLog.filter((e) => e === 'setup:race:e').length, 1)
  assert.equal(runtime.auditLog.filter((e) => e === 'enable:race').length, 1)
  await runtime.disable('race')
  assert.equal(runtime.auditLog.filter((e) => e === 'cleanup:race:e').length, 1)
})

// 9. 审计日志完整生命周期顺序
test('auditLog 记录 load/setup/enable/cleanup/disable/unload 完整顺序', async () => {
  const runtime = new PluginRuntime()
  const plugin = definePlugin({
    meta: { name: 'life' },
    apply: (_c, api) => api.effect('e', () => {}, () => {}),
  })
  await runtime.install('life', plugin)
  await runtime.unload('life')
  assert.deepEqual(runtime.auditLog, [
    'load:life',
    'setup:life:e',
    'enable:life',
    'cleanup:life:e',
    'disable:life',
    'unload:life',
  ])
})

// 10. 运行中热装第二个，立即生效且不影响第一个
test('运行时热装第二个插件立即生效，已运行插件不受影响', async () => {
  const runtime = new PluginRuntime()
  await runtime.install('first', mk('first'))
  assert.equal(runtime.ctx.get('svc/first'), 'first')
  await runtime.install('second', mk('second'))
  assert.equal(runtime.ctx.get('svc/second'), 'second')
  assert.equal(runtime.ctx.get('svc/first'), 'first')
  assert.equal(runtime.list().length, 2)
})

// 11. 直接用 Cordis 原生 effect 验证底层可逆性
test('Cordis 原生 ctx.effect 的 cleanup 由 fiber.dispose 逆序触发', async () => {
  const ctx = new Context()
  const log: string[] = []
  const fiber = ctx.use((c) => {
    c.effect(() => {
      log.push('s1')
      return () => log.push('c1')
    })
    c.effect(() => {
      log.push('s2')
      return () => log.push('c2')
    })
  })
  await fiber
  assert.deepEqual(log, ['s1', 's2'])
  await fiber.dispose()
  assert.deepEqual(log, ['s1', 's2', 'c2', 'c1'])
})
