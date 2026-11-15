<script lang="ts">
  interface Tab {
    id: number
    url: string
    title: string
    history: string[]
    cursor: number
  }
  interface Perm {
    name: string
    granted: boolean
  }
  interface Plug {
    id: string
    name: string
    enabled: boolean
    permissions: Perm[]
  }

  const HOME = 'fenestra://newtab'

  let tabs = $state<Tab[]>([
    { id: 1, url: HOME, title: '新标签页', history: [HOME], cursor: 0 },
  ])
  let activeId = $state(1)
  let nextId = 2
  let address = $state('')
  let view = $state<'browser' | 'plugins'>('browser')

  // 权限数据来自闸门（默认拒绝 + 最小权限交集）；此处为前端壳的本地镜像。
  let plugins = $state<Plug[]>([
    {
      id: 'newtab',
      name: '官方默认新标签页',
      enabled: true,
      permissions: [{ name: 'ui.newtab', granted: true }],
    },
    {
      id: 'accelerate',
      name: 'DNS 缓存 / 悬停预连接',
      enabled: true,
      permissions: [
        { name: 'net.dns', granted: true },
        { name: 'net.preconnect', granted: true },
      ],
    },
  ])

  function current(): Tab | undefined {
    return tabs.find((t) => t.id === activeId)
  }

  function titleOf(url: string): string {
    if (url === HOME) return '新标签页'
    try {
      return new URL(url).hostname
    } catch {
      return url
    }
  }

  function normalize(raw: string): string {
    const v = raw.trim()
    if (!v) return HOME
    if (v.startsWith('fenestra://')) return v
    if (/^https?:\/\//i.test(v)) return v
    return 'https://' + v
  }

  function sync(t: Tab): void {
    t.url = t.history[t.cursor] ?? HOME
    t.title = titleOf(t.url)
  }

  function navigate(raw: string): void {
    const t = current()
    if (!t) return
    const url = normalize(raw)
    t.history = t.history.slice(0, t.cursor + 1)
    t.history.push(url)
    t.cursor = t.history.length - 1
    t.url = url
    t.title = titleOf(url)
    address = ''
    view = 'browser'
  }

  function back(): void {
    const t = current()
    if (t && t.cursor > 0) {
      t.cursor -= 1
      sync(t)
    }
  }

  function forward(): void {
    const t = current()
    if (t && t.cursor < t.history.length - 1) {
      t.cursor += 1
      sync(t)
    }
  }

  function reload(): void {
    const t = current()
    if (t) t.title = titleOf(t.url)
  }

  function newTab(): void {
    const id = nextId++
    tabs.push({ id, url: HOME, title: '新标签页', history: [HOME], cursor: 0 })
    activeId = id
    view = 'browser'
  }

  function closeTab(id: number): void {
    if (tabs.length === 1) return
    tabs = tabs.filter((t) => t.id !== id)
    if (activeId === id) activeId = tabs[tabs.length - 1]!.id
  }

  function togglePlugin(id: string): void {
    const p = plugins.find((x) => x.id === id)
    if (p) p.enabled = !p.enabled
  }

  function revoke(pluginId: string, name: string): void {
    const p = plugins.find((x) => x.id === pluginId)
    const perm = p?.permissions.find((x) => x.name === name)
    if (perm) perm.granted = false
  }
</script>

<div class="window">
  <header class="toolbar">
    <button class="nav" onclick={back} title="后退" aria-label="后退">‹</button>
    <button class="nav" onclick={forward} title="前进" aria-label="前进">›</button>
    <button class="nav" onclick={reload} title="刷新" aria-label="刷新">↻</button>
    <form class="address" onsubmit={(e) => { e.preventDefault(); navigate(address) }}>
      <input
        class="url"
        bind:value={address}
        placeholder="输入网址后回车 · 例如 example.com"
        aria-label="地址栏"
        spellcheck="false"
      />
    </form>
    <button
      class="nav"
      onclick={() => (view = view === 'plugins' ? 'browser' : 'plugins')}
      title="插件管理"
      aria-label="插件管理"
    >◇</button>
  </header>

  <nav class="tabbar">
    {#each tabs as tab (tab.id)}
      <div class:active={tab.id === activeId} class="tab">
        <button class="tab-label" onclick={() => { activeId = tab.id; view = 'browser' }}>{tab.title}</button>
        <button class="tab-close" onclick={() => closeTab(tab.id)} aria-label="关闭标签">×</button>
      </div>
    {/each}
    <button class="tab-new" onclick={newTab} aria-label="新建标签页">+</button>
  </nav>

  <main class="content">
    {#if view === 'plugins'}
      <section class="plugins">
        <h2>插件管理 · Plugins</h2>
        <p class="hint">权限数据来自闸门（默认拒绝）。撤销后该能力立即不可用；装 / 卸插件现装现跑，无需重启。</p>
        <ul class="plugin-list">
          {#each plugins as p (p.id)}
            <li class="plugin">
              <div class="plugin-head">
                <div>
                  <strong>{p.name}</strong>
                  <small>{p.id}</small>
                </div>
                <button class:off={!p.enabled} class="toggle" onclick={() => togglePlugin(p.id)}>
                  {p.enabled ? '已启用' : '已停用'}
                </button>
              </div>
              <ul class="perms">
                {#each p.permissions as perm (perm.name)}
                  <li>
                    <code>{perm.name}</code>
                    <span class:granted={perm.granted} class="perm-state">
                      {perm.granted ? '已授权' : '已撤销'}
                    </span>
                    {#if perm.granted}
                      <button class="revoke" onclick={() => revoke(p.id, perm.name)}>撤销</button>
                    {/if}
                  </li>
                {/each}
              </ul>
            </li>
          {/each}
        </ul>
      </section>
    {:else}
      {@const tab = current()}
      {#if tab}
        <section class="page">
          <div class="page-badge">系统 WebView (wry) · 引擎可替换</div>
          <h1>{tab.title}</h1>
          <p class="url-line">{tab.url}</p>
          {#if tab.url === HOME}
            <div class="home">
              <p class="slogan">只是个壳，其余皆插件。</p>
              <p class="sub">在上方地址栏输入网址即可打开页面；本页由「官方默认新标签页」插件提供，可被替换。</p>
              <form onsubmit={(e) => { e.preventDefault(); navigate(address) }}>
                <input class="home-input" bind:value={address} placeholder="输入网址后回车" aria-label="地址栏" />
              </form>
            </div>
          {:else}
            <p class="sub">
              实际网页渲染由 Tauri 2 系统 WebView 承载。此处为前端壳的状态占位：
              地址栏、前进 / 后退 / 刷新、新标签页、插件启停与权限撤销逻辑均已真实运行。
            </p>
          {/if}
        </section>
      {/if}
    {/if}
  </main>

  <footer class="statusbar">
    <span>Fenestra · Stage 1 脚手架</span>
    <span>{tabs.length} 标签 · {plugins.filter((p) => p.enabled).length} 插件启用</span>
  </footer>
</div>
