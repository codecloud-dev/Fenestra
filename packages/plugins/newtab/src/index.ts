import type { PluginDefinition } from '@fenestra/core'

/**
 * 官方默认新标签页插件（层 A · 示例插件）。
 * 提供 `ui/newtab` 服务；示例替换插件用同名服务覆盖它，卸载后由 Cordis 可逆登记自动恢复。
 */
export const newtabPlugin: PluginDefinition = {
  meta: {
    name: 'fenestra.newtab',
    version: '0.1.0',
    description: '官方默认新标签页：地址栏占位、常用站点、隐私提示',
    permissions: ['ui.newtab'],
  },
  apply(_ctx, api) {
    api.effect(
      'newtab',
      () => {
        api.provide('ui/newtab', {
          title: 'Fenestra',
          blocks: ['addressbar', 'top-sites', 'privacy-note'],
        })
      },
      () => {},
    )
  },
}

export default newtabPlugin
