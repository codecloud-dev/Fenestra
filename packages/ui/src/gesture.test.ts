import { test } from 'node:test'
import assert from 'node:assert/strict'
import { rubberBand, projectFling, springDeriv } from './gesture.js'

test('橡皮边界：线性区内原样返回', () => {
  assert.equal(rubberBand(0, 0, 100), 0)
  assert.equal(rubberBand(50, 0, 100), 50)
  assert.equal(rubberBand(100, 0, 100), 100)
})

test('橡皮边界：越界被阻尼，且始终在界外但被拉回', () => {
  const lo = rubberBand(-20, 0, 100)
  const hi = rubberBand(120, 0, 100)
  assert.ok(lo < 0 && lo > -20, `左边越界应被阻尼，实际 ${lo}`)
  assert.ok(hi > 100 && hi < 120, `右边越界应被阻尼，实际 ${hi}`)
})

test('橡皮边界：拖得越远越沉（增量递减）', () => {
  const d1 = 0 - rubberBand(-20, 0, 100)   // 拖出 20 的实际位移
  const d2 = 0 - rubberBand(-40, 0, 100)   // 拖出 40 的实际位移
  assert.ok(d2 > d1, '越界越多实际位移越大')
  assert.ok(d2 < d1 * 2, '但增量应递减，不能线性')
})

test('橡皮边界：max<min 的退化输入不抛错', () => {
  assert.equal(rubberBand(10, 5, 0), 5)
})

test('动量投射：速度与方向成正比，d 越大投射越远', () => {
  assert.ok(projectFling(1) > 0)
  assert.ok(projectFling(2) > projectFling(1))
  assert.ok(projectFling(1, 0.999) > projectFling(1, 0.99))
})

test('弹簧派生：刚度随响应变快而增大，阻尼系数为正', () => {
  const a = springDeriv(1, 0.4)
  const b = springDeriv(1, 0.3)
  assert.ok(b.k > a.k, '响应更短 → 刚度更大')
  assert.ok(a.c > 0 && b.c > 0, '阻尼系数为正')
})
