/**
 * 手势物理的纯函数：可脱离浏览器单测。
 * 参数口径对齐 Apple WWDC：橡皮边界阻尼、减速投射、弹簧派生。
 */

/** Apple 式橡皮边界：越界位移递减，永远拖不"穿"。 */
const RUBBER = 0.55
const EXTENT = 72

export function rubberBand(x: number, min: number, max: number): number {
  if (max < min) return min
  if (x < min) {
    const over = min - x
    return min - (1 - 1 / (over * RUBBER / EXTENT + 1)) * EXTENT
  }
  if (x > max) {
    const over = x - max
    return max + (1 - 1 / (over * RUBBER / EXTENT + 1)) * EXTENT
  }
  return x
}

/** 动量投射总位移：每帧速度按 d 衰减，总位移 = vFrame·d/(1−d)。 */
export function projectFling(vPerMs: number, d = 0.998): number {
  const vFrame = vPerMs * (1000 / 60)
  return (vFrame * d) / (1 - d)
}

/** 阻尼比 ζ + 响应 τ → 弹簧角频率派生（ωn = 4/(ζτ)）。 */
export function springDeriv(zeta: number, responseSec: number) {
  const omega = 4 / (zeta * responseSec)
  return { k: omega * omega, c: 2 * zeta * omega }
}
