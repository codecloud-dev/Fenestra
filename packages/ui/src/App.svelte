<script lang="ts">
  import { onMount, tick } from 'svelte'

  // ==========================================================================
  // 状态：一个极简浏览器壳的数据模型。
  // 权限数据来自 Rust 闸门（默认拒绝 + 最小权限交集），此处是前端壳的本地镜像，
  // 只负责呈现与「撤销」交互，不放宽任何裁决。
  // ==========================================================================
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

  // 已授权能力数（用于顶部「默认拒绝」旁的计数；默认拒绝是仲裁策略，不是零授权）。
  const grantedCount = $derived(
    plugins.reduce(
      (n, p) => n + (p.enabled ? p.permissions.filter((q) => q.granted).length : 0),
      0,
    ),
  )

  function current(): Tab | undefined {
    return tabs.find((t) => t.id === activeId)
  }
  const activeTab = $derived(current())
  const SEARCH_PREFIX = 'fenestra://search?q='

  function titleOf(url: string): string {
    if (url === HOME) return '新标签页'
    // 搜索走内部 scheme，不默认发往任何外部搜索服务（隐私：出站最小化）。
    if (url.startsWith(SEARCH_PREFIX)) {
      const q = decodeURIComponent(url.slice(SEARCH_PREFIX.length))
      return `搜索 · ${q}`
    }
    try {
      return new URL(url).hostname
    } catch {
      return url
    }
  }
  // 地址栏与搜索合一：像一个输入框，既能输网址也能输搜索词。
  function normalize(raw: string): string {
    const v = raw.trim()
    if (!v) return HOME
    if (v.startsWith('fenestra://')) return v
    if (/^https?:\/\//i.test(v)) return v
    // 含空白、或不像主机名（域名 / localhost / IPv4）的输入，按搜索词处理。
    const looksLikeHost =
      /^[a-z0-9-]+(\.[a-z0-9-]+)+(:\d+)?(\/|$)/i.test(v) ||
      /^localhost(:\d+)?(\/|$)/i.test(v) ||
      /^\d{1,3}(\.\d{1,3}){3}(:\d+)?(\/|$)/.test(v)
    if (!looksLikeHost) return SEARCH_PREFIX + encodeURIComponent(v)
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
    // 移动端提交后收起软键盘：让地址栏失焦。
    ;(document.activeElement as HTMLElement | null)?.blur()
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
  function openPlugins(): void {
    view = 'plugins'
  }
  function closePlugins(): void {
    view = 'browser'
  }

  // ==========================================================================
  // 液态玻璃 · 层② 的 SDF 位移图生成（自有实现，非现成库 / 非复制）
  //
  // 思路：
  //  1) 用 IQ 圆角矩形 SDF：d(p) = |max(q,0)| + min(max(qx,qy),0) - r，内负外正。
  //  2) 只在最外 band（短边 10~15%）内让折射生效：中心 t=0 → 零位移 → 保持清晰；
  //     越靠外缘 t→1 → 位移越强。这就是「外圈白、内部黑」的 mask 语义。
  //  3) 用 SDF 的数值梯度当外向单位法线 n；feDisplacementMap 的公式为
  //     P'(x,y) = P(x + scale·(R/255−0.5), y + scale·(G/255−0.5))。
  //     令中心 R=G=128（零位移），边缘 R=128+n.x·127·t、G=128+n.y·127·t。
  //  4) 同时产出一张「外圈白 / 内部黑」的边缘 mask（RGB 与 alpha 同步编码，
  //     这样 mask-mode 取 alpha 还是 luminance 结果一致），供 CSS mask-image 使用。
  // ==========================================================================

  // feDisplacementMap 的 scale：官方约束 6~12px，取 10。
  const REFRACT_SCALE = 10

  function sdfRoundedBox(px: number, py: number, w: number, h: number, r: number): number {
    const qx = Math.abs(px - w / 2) - (w / 2 - r)
    const qy = Math.abs(py - h / 2) - (h / 2 - r)
    const outside = Math.hypot(Math.max(qx, 0), Math.max(qy, 0))
    const inside = Math.min(Math.max(qx, qy), 0)
    return outside + inside - r
  }

  function buildGlassMaps(w: number, h: number, radius: number) {
    // 限制生成分辨率上限，避免高分屏上逐像素循环过重；feImage 会拉伸回真实尺寸。
    const W = Math.max(64, Math.min(640, Math.round(w)))
    const H = Math.max(48, Math.min(320, Math.round(h)))
    const r = Math.max(2, Math.min(radius * (W / w), Math.min(W, H) / 2 - 2))
    const band = Math.max(6, Math.min(W, H) * 0.13) // 最外 13%

    const disp = document.createElement('canvas')
    const mask = document.createElement('canvas')
    disp.width = mask.width = W
    disp.height = mask.height = H
    const dctx = disp.getContext('2d')!
    const mctx = mask.getContext('2d')!
    const dImg = dctx.createImageData(W, H)
    const mImg = mctx.createImageData(W, H)
    const e = 1 // 有限差分步长（生成图坐标单位）

    for (let y = 0; y < H; y++) {
      for (let x = 0; x < W; x++) {
        const px = x + 0.5
        const py = y + 0.5
        const d = sdfRoundedBox(px, py, W, H, r)
        // 边缘衰减：d ∈ [-band, 0] 映射到 [0, 1]，内部（d < -band）恒 0。
        const raw = Math.min(1, Math.max(0, (d + band) / band))
        const smooth = raw * raw * (3 - 2 * raw) // smoothstep，避免硬边
        const bend = Math.pow(smooth, 1.35) // 越贴外缘折射越强

        // 数值梯度 → 外向单位法线
        let nx =
          sdfRoundedBox(px + e, py, W, H, r) - sdfRoundedBox(px - e, py, W, H, r)
        let ny =
          sdfRoundedBox(px, py + e, W, H, r) - sdfRoundedBox(px, py - e, W, H, r)
        const len = Math.hypot(nx, ny) || 1
        nx /= len
        ny /= len

        const i = (y * W + x) * 4
        dImg.data[i] = 128 + nx * bend * 127
        dImg.data[i + 1] = 128 + ny * bend * 127
        dImg.data[i + 2] = 128
        dImg.data[i + 3] = 255 // 位移图 alpha 必须为不透明

        // 边缘 mask：外圈白、内部黑；RGB 与 alpha 同步，兼容两种 mask-mode。
        const m = Math.round(bend * 255)
        mImg.data[i] = mImg.data[i + 1] = mImg.data[i + 2] = m
        mImg.data[i + 3] = m
      }
    }
    dctx.putImageData(dImg, 0, 0)
    mctx.putImageData(mImg, 0, 0)
    return { disp: disp.toDataURL('image/png'), mask: mask.toDataURL('image/png') }
  }

  const GLASS_KEYS = ['top', 'dock', 'sheet'] as const
  type GlassKey = (typeof GLASS_KEYS)[number]

  let svgDefs: SVGSVGElement | undefined = $state()
  let windowEl: HTMLElement | undefined = $state()
  let sheetEl: HTMLElement | undefined = $state()
  let tabsEl: HTMLElement | undefined = $state()
  let resizeObserver: ResizeObserver | undefined

  // 生成并注入某一块玻璃的位移图与边缘 mask。
  function setupGlass(key: GlassKey, el: HTMLElement): void {
    const rect = el.getBoundingClientRect()
    if (rect.width < 2 || rect.height < 2) return
    const radius = parseFloat(getComputedStyle(el).borderTopLeftRadius) || 20
    const { disp, mask } = buildGlassMaps(rect.width, rect.height, radius)

    const feImage = svgDefs?.querySelector<SVGFEImageElement>(
      `feImage[data-lg-key="${key}"]`,
    )
    const filter = svgDefs?.querySelector<SVGFilterElement>(
      `filter[data-lg-key="${key}"]`,
    )
    // 滤镜区域用真实像素尺寸；生成图若被降采样，由 preserveAspectRatio=none 拉伸回来。
    filter?.setAttribute('x', '0')
    filter?.setAttribute('y', '0')
    filter?.setAttribute('width', String(rect.width))
    filter?.setAttribute('height', String(rect.height))
    if (feImage) {
      feImage.setAttribute('href', disp)
      // xlink 兼容老版 Chromium 的 feImage 数据 URI 解析。
      feImage.setAttributeNS('http://www.w3.org/1999/xlink', 'href', disp)
      feImage.setAttribute('x', '0')
      feImage.setAttribute('y', '0')
      feImage.setAttribute('width', String(rect.width))
      feImage.setAttribute('height', String(rect.height))
      feImage.setAttribute('preserveAspectRatio', 'none')
    }
    el.style.setProperty('--lg-edge-mask', `url("${mask}")`)
    el.style.setProperty('--lg-refraction', `url("#lg-refract-${key}")`)
  }

  function setupAllGlass(): void {
    document.querySelectorAll<HTMLElement>('.lg[data-lg-key]').forEach((el) => {
      setupGlass(el.dataset.lgKey as GlassKey, el)
    })
  }

  // 指针位置驱动层④镜面高光（--lg-mx / --lg-my 相对元素百分比）。
  function onGlassMove(e: PointerEvent): void {
    const el = e.currentTarget as HTMLElement
    const r = el.getBoundingClientRect()
    if (!r.width || !r.height) return
    el.style.setProperty('--lg-mx', `${(((e.clientX - r.left) / r.width) * 100).toFixed(1)}%`)
    el.style.setProperty('--lg-my', `${(((e.clientY - r.top) / r.height) * 100).toFixed(1)}%`)
  }

  // ==========================================================================
  // 手写弹簧（不引入 Framer Motion 等任何动画库）
  //  阻尼比 ζ / 响应时长 τ（秒）→ 自然频率 ωn = 4/(ζ·τ)（调节时间 Ts≈4/(ζωn) 反解）。
  //  默认：ζ=1.0 / τ=0.4；手势抛动才用 ζ=0.8 / τ=0.3。
  //  动量投射：每帧速度按 decelerationRate=0.998 衰减，总位移 = vFrame·d/(1−d)。
  // ==========================================================================
  interface SpringCfg {
    zeta: number
    response: number
  }
  // 物理参数以 app.css 的自定义属性为唯一真源，运行时读取（单位 0.4s 由 parseFloat 去掉）。
  function readSpring(which: 'rest' | 'fling'): SpringCfg {
    const root = getComputedStyle(document.documentElement)
    const zeta =
      parseFloat(root.getPropertyValue(`--spring-zeta-${which}`)) ||
      (which === 'rest' ? 1.0 : 0.8)
    const response =
      parseFloat(root.getPropertyValue(`--spring-response-${which}`)) ||
      (which === 'rest' ? 0.4 : 0.3)
    return { zeta, response }
  }
  function decelerationRate(): number {
    const v = parseFloat(
      getComputedStyle(document.documentElement).getPropertyValue('--deceleration-rate'),
    )
    return Number.isFinite(v) ? v : 0.998
  }
  function springDeriv(cfg: SpringCfg) {
    const omega = 4 / (cfg.zeta * cfg.response)
    return { k: omega * omega, c: 2 * cfg.zeta * omega }
  }
  function reducedMotion(): boolean {
    return window.matchMedia('(prefers-reduced-motion: reduce)').matches
  }
  function clamp(v: number, lo: number, hi: number): number {
    return Math.max(lo, Math.min(hi, v))
  }

  // 半隐式欧拉逐帧积分，子步保证大刚度下稳定；返回取消函数。
  function springTo(
    set: (v: number) => void,
    from: number,
    to: number,
    cfg: SpringCfg,
  ): () => void {
    if (reducedMotion()) {
      set(to)
      return () => {}
    }
    const { k, c } = springDeriv(cfg)
    let x = from
    let v = 0
    let last = performance.now()
    let raf = 0
    const frame = (now: number) => {
      const dt = Math.min((now - last) / 1000, 1 / 30)
      last = now
      const steps = 2
      for (let i = 0; i < steps; i++) {
        const h = dt / steps
        const a = -k * (x - to) - c * v
        v += a * h
        x += v * h
      }
      if (Math.abs(x - to) < 0.01 && Math.abs(v) < 0.01) {
        set(to)
        return
      }
      set(x)
      raf = requestAnimationFrame(frame)
    }
    raf = requestAnimationFrame(frame)
    return () => cancelAnimationFrame(raf)
  }

  // 标签条：自定义横向拖拽 + 动量投射 + 抛动弹簧。
  // touch-action:none 由 CSS 设定，滚动完全由这里掌控，避免与原生惯性叠加两次。
  const drag = { active: false, startX: 0, startScroll: 0, lastX: 0, lastT: 0, v: 0 }
  let cancelFling: () => void = () => {}
  let dragMoved = false
  let suppressClick = false

  // 拖拽超过阈值后，紧随 pointerup 的那次 click 不应误触标签切换 / 新建。
  function onTabsClickCapture(ev: MouseEvent): void {
    if (!suppressClick) return
    ev.preventDefault()
    ev.stopPropagation()
    suppressClick = false
  }

  function onTabsDown(e: PointerEvent): void {
    if (e.pointerType === 'mouse' && e.button !== 0) return
    if (!tabsEl) return
    cancelFling()
    suppressClick = false
    dragMoved = false
    drag.active = true
    drag.startX = e.clientX
    drag.startScroll = tabsEl.scrollLeft
    drag.lastX = e.clientX
    drag.lastT = performance.now()
    drag.v = 0
    window.addEventListener('pointermove', onTabsMove)
    window.addEventListener('pointerup', onTabsUp, { once: true })
    window.addEventListener('pointercancel', onTabsUp, { once: true })
  }
  function onTabsMove(e: PointerEvent): void {
    if (!drag.active || !tabsEl) return
    const now = performance.now()
    const dt = Math.max(1, now - drag.lastT)
    const dx = e.clientX - drag.lastX
    if (Math.abs(e.clientX - drag.startX) > 4) dragMoved = true
    tabsEl.scrollLeft = drag.startScroll - (e.clientX - drag.startX)
    // 速度低通，抑制单点抖动；单位 px/ms。
    drag.v = drag.v * 0.6 + (dx / dt) * 0.4
    drag.lastX = e.clientX
    drag.lastT = now
  }
  function onTabsUp(): void {
    window.removeEventListener('pointermove', onTabsMove)
    window.removeEventListener('pointerup', onTabsUp)
    window.removeEventListener('pointercancel', onTabsUp)
    if (!drag.active || !tabsEl) return
    drag.active = false
    suppressClick = dragMoved
    if (reducedMotion() || Math.abs(drag.v) < 0.25) return
    const vFrame = drag.v * (1000 / 60) // px/帧
    const d = decelerationRate()
    const projected = (vFrame * d) / (1 - d)
    const max = Math.max(0, tabsEl.scrollWidth - tabsEl.clientWidth)
    // 手指向右滑（v>0）时 scrollLeft 继续减小，故取减号。
    const target = clamp(tabsEl.scrollLeft - projected, 0, max)
    cancelFling = springTo(
      (x) => {
        if (tabsEl) tabsEl.scrollLeft = x
      },
      tabsEl.scrollLeft,
      target,
      readSpring('fling'),
    )
  }

  // 每块玻璃只挂一次 pointermove；用 _lgWired 标记，避免 onMount 与 $effect 重复绑定。
  function wireGlass(el: HTMLElement): void {
    if ((el as HTMLElement & { _lgWired?: boolean })._lgWired) return
    el.addEventListener('pointermove', onGlassMove)
    ;(el as HTMLElement & { _lgWired?: boolean })._lgWired = true
  }

  onMount(() => {
    // Chromium 才具备 backdrop-filter 引用 SVG 滤镜的能力；用 UA + CSS.supports 双重判定，
    // 非 Chromium（Safari / Firefox）保持 data 属性缺省 → CSS 自动退回纯 CSS 玻璃。
    const uaData = (navigator as Navigator & {
      userAgentData?: { brands?: { brand: string }[] }
    }).userAgentData
    const brands = uaData?.brands?.map((b) => b.brand) ?? []
    const isChromium =
      brands.includes('Chromium') ||
      (/Chrome\/\d/.test(navigator.userAgent) && !/Firefox\//.test(navigator.userAgent))
    const svgCapable =
      CSS.supports('backdrop-filter', 'url("#lg-probe")') ||
      CSS.supports('-webkit-backdrop-filter', 'url("#lg-probe")')
    if (isChromium && svgCapable) {
      document.documentElement.dataset.lgRefraction = 'svg'
    }

    // 事件用原生监听挂在 .lg / .tabs 上：避免在纯装饰 div 上写 Svelte 事件属性
    // 触发 a11y 编译告警，也让事件绑定与 setupAllGlass 的生命周期一致。
    document.querySelectorAll<HTMLElement>('.lg[data-lg-key]').forEach(wireGlass)
    tabsEl?.addEventListener('pointerdown', onTabsDown)
    tabsEl?.addEventListener('click', onTabsClickCapture, true)

    setupAllGlass()
    // 尺寸变化（旋转 / 软键盘 / 标签增减）时重算位移图，保证 SDF 与真实盒模型一致。
    resizeObserver = new ResizeObserver(() => setupAllGlass())
    if (windowEl) resizeObserver.observe(windowEl)

    return () => {
      resizeObserver?.disconnect()
      cancelFling()
      tabsEl?.removeEventListener('pointerdown', onTabsDown)
      tabsEl?.removeEventListener('click', onTabsClickCapture, true)
      window.removeEventListener('pointermove', onTabsMove)
      window.removeEventListener('pointerup', onTabsUp)
      window.removeEventListener('pointercancel', onTabsUp)
      document.querySelectorAll<HTMLElement>('.lg[data-lg-key]').forEach((el) => {
        el.removeEventListener('pointermove', onGlassMove)
      })
    }
  })

  // 弹层出现 / 消失时：补生成 sheet 的玻璃图，并用默认弹簧做入场。
  $effect(() => {
    const current = view
    const count = tabs.length
    void count
    tick().then(() => {
      // 新出现的 .lg（sheet）需要补挂 pointermove 与位移图。
      document.querySelectorAll<HTMLElement>('.lg[data-lg-key]').forEach(wireGlass)
      setupAllGlass()
      if (current === 'plugins' && sheetEl) {
        sheetEl.style.setProperty('--sheet-p', '0')
        springTo(
          (v) => sheetEl?.style.setProperty('--sheet-p', String(v)),
          0,
          1,
          readSpring('rest'),
        )
      }
    })
  })
</script>

<!-- 图标：全部自绘 SVG 路径，不用 emoji、不用图标库。 -->
{#snippet ico(n: string)}
  <svg
    class="ico"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.7"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
    focusable="false"
  >
    {#if n === 'back'}
      <path d="M14.5 5.5 8 12l6.5 6.5" />
    {:else if n === 'forward'}
      <path d="M9.5 5.5 16 12l-6.5 6.5" />
    {:else if n === 'reload'}
      <path d="M19.1 9.3A7.6 7.6 0 1 0 19.4 14" />
      <path d="M19.4 4.6v4.9h-4.9" />
    {:else if n === 'shield'}
      <path d="M12 3.2 4.8 6v5.4c0 4.2 2.9 7.6 7.2 9.4 4.3-1.8 7.2-5.2 7.2-9.4V6L12 3.2Z" />
      <path d="m9 12 2.1 2.2L15.3 10" />
    {:else if n === 'plus'}
      <path d="M12 5.5v13M5.5 12h13" />
    {:else if n === 'close'}
      <path d="M6.5 6.5l11 11M17.5 6.5l-11 11" />
    {:else if n === 'grid'}
      <rect x="4.5" y="4.5" width="6" height="6" rx="1.4" />
      <rect x="13.5" y="4.5" width="6" height="6" rx="1.4" />
      <rect x="4.5" y="13.5" width="6" height="6" rx="1.4" />
      <rect x="13.5" y="13.5" width="6" height="6" rx="1.4" />
    {:else if n === 'search'}
      <circle cx="10.8" cy="10.8" r="5.3" />
      <path d="m14.7 14.7 4 4" />
    {:else if n === 'lock'}
      <rect x="5.5" y="10.5" width="13" height="9.5" rx="2" />
      <path d="M8.5 10.5V7.8a3.5 3.5 0 0 1 7 0v2.7" />
    {/if}
  </svg>
{/snippet}

<!-- 一块玻璃的 5 层材质叠层：折射(Chromium) / 色差环 / 镜面高光。
     注意：这些是「同一块玻璃」的内部层，不是叠加另一块玻璃。 -->
{#snippet spec()}
  <span class="lg__refract" aria-hidden="true"></span>
  <span class="lg__chroma" aria-hidden="true"></span>
  <span class="lg__spec" aria-hidden="true"></span>
{/snippet}

<div class="window" bind:this={windowEl}>
  <!-- 真实材质：极轻的颗粒 + 顶部柔光，避免 AI 味的紫渐变 / 发光球 / 网格。 -->
  <div class="bg" aria-hidden="true"></div>
  <div class="bg-grain" aria-hidden="true"></div>

  <!-- 内容层：绝不上玻璃。上下用 mask-image 渐变，让滚到玻璃下面的内容柔和融入。 -->
  <main class="viewport">
    {#if activeTab}
      {#if activeTab.url === HOME}
        <section class="home">
          <div class="home__mark">FENESTRA</div>
          <p class="home__slogan">只是个壳，其余皆插件。</p>
          <p class="home__hint">在下方地址栏输入网址或搜索词</p>
          <div class="home__rule"></div>
          <div class="home__sec">
            {@render ico('lock')}
            <span>默认拒绝生效 · 沙箱隔离 · 出站重校验</span>
          </div>
        </section>
      {:else}
        <section class="page">
          <div class="page__tag">系统 WebView · 引擎可替换</div>
          <h1 class="page__title">{activeTab.title}</h1>
          <p class="page__url">{activeTab.url}</p>
          {#if activeTab.url.startsWith(SEARCH_PREFIX)}
            <p class="page__note">
              搜索词交由「官方默认新标签页」插件路由，当前为前端壳状态占位，未发起任何网络请求。
            </p>
          {:else}
            <p class="page__note">
              网页由系统 WebView 承载。地址栏、前进 / 后退、标签切换与权限裁决均已真实运行。
            </p>
          {/if}
        </section>
      {/if}
    {/if}
  </main>

  <!-- 顶部导航层（玻璃 · Regular）。安全状态一眼可见：默认拒绝。 -->
  <header class="topbar lg" data-lg-key="top">
    {@render spec()}
    <div class="brand">
      <span class="brand__mark" aria-hidden="true"></span>
      <span class="brand__name">Fenestra</span>
    </div>
    <button class="security" onclick={openPlugins} aria-label="查看安全状态与插件权限">
      {@render ico('shield')}
      <span>默认拒绝</span>
      <span class="security__sep" aria-hidden="true"></span>
      <span class="security__count">{grantedCount} 项授权</span>
    </button>
  </header>

  <!-- 底部导航层（玻璃 · Regular）。标签条与地址栏同属这一块玻璃，
       子元素不自带 backdrop-filter，避免「玻璃叠玻璃」。 -->
  <div class="dock lg" data-lg-key="dock">
    {@render spec()}

    <div class="tabs" bind:this={tabsEl} aria-label="标签页">
      {#each tabs as tab (tab.id)}
        <div class="tab" class:active={tab.id === activeId}>
          <button
            class="tab__label"
            onclick={() => {
              activeId = tab.id
              view = 'browser'
            }}
            aria-label={`切换到 ${tab.title}`}
          >{tab.title}</button>
          <button class="tab__close" onclick={() => closeTab(tab.id)} aria-label={`关闭 ${tab.title}`}>
            {@render ico('close')}
          </button>
        </div>
      {/each}
      <button class="tab__new" onclick={newTab} aria-label="新建标签页">
        {@render ico('plus')}
      </button>
    </div>

    <form
      class="addressbar"
      onsubmit={(e) => {
        e.preventDefault()
        navigate(address)
      }}
    >
      <button class="icon-btn" type="button" onclick={back} aria-label="后退">
        {@render ico('back')}
      </button>
      <button class="icon-btn" type="button" onclick={forward} aria-label="前进">
        {@render ico('forward')}
      </button>
      <div class="field">
        <span class="field__search" aria-hidden="true">{@render ico('search')}</span>
        <input
          class="field__input"
          bind:value={address}
          placeholder="搜索或输入网址"
          aria-label="地址栏与搜索"
          type="text"
          inputmode="url"
          enterkeyhint="go"
          autocomplete="off"
          autocapitalize="off"
          spellcheck="false"
        />
      </div>
      <button class="icon-btn" type="button" onclick={reload} aria-label="刷新">
        {@render ico('reload')}
      </button>
      <button class="icon-btn icon-btn--accent" type="button" onclick={openPlugins} aria-label="插件与权限">
        {@render ico('grid')}
      </button>
    </form>
  </div>

  <!-- 插件管理：悬浮弹层（玻璃 · Regular），位于内容区中部，不与上下玻璃条重叠。 -->
  {#if view === 'plugins'}
    <button class="scrim" onclick={closePlugins} aria-label="关闭插件与权限"></button>
    <section class="sheet lg" data-lg-key="sheet" bind:this={sheetEl} aria-label="插件与权限">
      {@render spec()}
      <div class="sheet__head">
        <div>
          <h2 class="sheet__title">插件与权限</h2>
          <p class="sheet__sub">默认拒绝 · 撤销即时生效 · 装 / 卸无需重启</p>
        </div>
        <button class="sheet__close" onclick={closePlugins} aria-label="关闭">
          {@render ico('close')}
        </button>
      </div>

      <ul class="plugin-list">
        {#each plugins as p (p.id)}
          <li class="plugin">
            <div class="plugin__head">
              <div class="plugin__meta">
                <strong>{p.name}</strong>
                <small>{p.id}</small>
              </div>
              <button
                class="toggle"
                class:off={!p.enabled}
                onclick={() => togglePlugin(p.id)}
                aria-label={`${p.enabled ? '停用' : '启用'} ${p.name}`}
              >{p.enabled ? '已启用' : '已停用'}</button>
            </div>
            <ul class="perms">
              {#each p.permissions as perm (perm.name)}
                <li>
                  <code>{perm.name}</code>
                  <span class="perm-state" class:granted={perm.granted}>
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

      <p class="sheet__foot">
        闸门在 Rust 层按「声明 ∩ 策略 ∩ 确认」裁决；隐私红线能力须逐项显式确认。
        默认拒绝、沙箱隔离、出站重校验不可关闭。
      </p>
    </section>
  {/if}

  <!-- SVG 滤镜定义：每块玻璃一个滤镜，feImage 的位移图由上面的 SDF 生成器注入。
       隐藏方式用 0 尺寸而非 display:none，保证滤镜可被 backdrop-filter 引用。 -->
  <svg class="lg-defs" width="0" height="0" aria-hidden="true" focusable="false" bind:this={svgDefs}>
    <defs>
      {#each GLASS_KEYS as key}
        <filter
          id={`lg-refract-${key}`}
          data-lg-key={key}
          filterUnits="userSpaceOnUse"
          primitiveUnits="userSpaceOnUse"
          x="0"
          y="0"
          width="100"
          height="100"
          color-interpolation-filters="sRGB"
        >
          <feImage data-lg-key={key} result="sdf" preserveAspectRatio="none" />
          <feDisplacementMap
            in="SourceGraphic"
            in2="sdf"
            scale={REFRACT_SCALE}
            xChannelSelector="R"
            yChannelSelector="G"
          />
        </filter>
      {/each}
    </defs>
  </svg>
</div>
