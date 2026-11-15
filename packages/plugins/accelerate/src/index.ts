import type { PluginDefinition } from '@fenestra/core'

/**
 * 第一个加速插件（层 A · 示例插件）：基础 DNS 缓存 + 鼠标悬停预连接。
 *
 * 红线：不碰系统内核 / VPN / eBPF / BBR，不收任何用户数据。
 * 所有登记（缓存、服务、事件监听）都通过可逆 effect 注册，卸载时清空、无残留。
 * 能力 `net.dns` / `net.preconnect` 只是声明，是否放行由 Rust 闸门按默认拒绝裁决。
 */
export const acceleratePlugin: PluginDefinition = {
  meta: {
    name: 'fenestra.accelerate',
    version: '0.1.0',
    description: '基础 DNS 缓存 + 鼠标悬停预连接（不碰系统内核 / VPN / eBPF / BBR）',
    permissions: ['net.dns', 'net.preconnect'],
  },
  apply(_ctx, api) {
    const cache = new Map<string, string>()
    let lastHover: string | null = null

    api.effect(
      'dns-cache',
      () => {
        api.provide('accel/dns-cache', {
          lookup: (host: string) => cache.get(host),
          put: (host: string, address: string) => {
            cache.set(host, address)
          },
          size: () => cache.size,
        })

        // 悬停预连接：这里只登记意图，不发起真实连接；
        // 真实连接由宿主网络栈在闸门放行后执行。
        api.on('ui/link-hover', (...args: unknown[]) => {
          if (typeof args[0] === 'string') {
            lastHover = args[0]
          }
        })
        api.provide('accel/preconnect', {
          lastHover: () => lastHover,
        })
      },
      () => {
        // 卸载清空缓存，无残留。
        cache.clear()
        lastHover = null
      },
    )
  },
}

export default acceleratePlugin
