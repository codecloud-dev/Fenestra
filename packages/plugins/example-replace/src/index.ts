import type { PluginDefinition } from '@fenestra/core'

/**
 * 示例替换插件（层 A · 示例插件）：验证「一切可改」。
 *
 * 它提供与官方默认新标签页同名的 `ui/newtab` 服务，启用后覆盖默认内容；
 * 一旦卸载，Cordis 的可逆登记会把该服务恢复成官方默认插件提供的版本，
 * 无需重启、无需手动回滚。这正是 MVP 第 4 件事要验证的核心承诺。
 */
export const exampleReplacePlugin: PluginDefinition = {
  meta: {
    name: 'example.replace',
    version: '0.1.0',
    description: '示例：替换默认新标签页；卸载后自动恢复官方默认',
    permissions: ['ui.newtab'],
  },
  apply(_ctx, api) {
    api.effect(
      'replace',
      () => {
        api.provide('ui/newtab', {
          title: 'My Start',
          blocks: ['greeting', 'custom-links'],
        })
      },
      () => {},
    )
  },
}

export default exampleReplacePlugin
