#!/usr/bin/env bash
# license-guard：校验双许可两文件齐备、层 A 自撰且符合 OSD、已公开版本不闭源、层 B 为 FSL v1.0。
set -euo pipefail

fail() {
  echo "[license-guard] FAIL: $*" >&2
  exit 1
}

echo "[license-guard] 校验双许可"

# --- 两文件存在且非空 ---
test -f LICENSE || fail "缺少 LICENSE（层 A）"
test -f LICENSE-COMMERCIAL || fail "缺少 LICENSE-COMMERCIAL（层 B）"
test -s LICENSE || fail "LICENSE 为空"
test -s LICENSE-COMMERCIAL || fail "LICENSE-COMMERCIAL 为空"

# --- 层 A：符合 OSD、自撰、诚实声明、已公开版本不得重新闭源 ---
grep -q "开源定义" LICENSE || fail "层 A 未声明符合开源定义(OSD)"
grep -q "OSI 认证" LICENSE || fail "层 A 缺少「非 OSI 认证」的诚实声明"
grep -q "不得重新闭源" LICENSE || fail "层 A 缺少「已公开版本不得重新闭源」"
grep -q "层 A 永久开源" LICENSE || fail "层 A 缺少永久开源边界"
grep -q "层 B 不得弱化" LICENSE || fail "层 A 缺少「层 B 不得弱化安全不变量」边界"

# 层 A 不能被误写成 FSL 的「不是开源软件」措辞（防层 B 文本回灌）
if grep -q "不是开源软件" LICENSE; then
  fail "层 A 出现 FSL 的否定开源措辞，疑似层 B 文本回灌"
fi

# 自撰性：不得含现成模板特征句
if grep -qiE "Permission is hereby granted|Apache License|THE SOFTWARE IS PROVIDED" LICENSE; then
  fail "层 A 出现现成许可证模板特征句，违反自撰要求"
fi

# --- 层 B：FSL v1.0 原文在位，商用付费 + 改动必须开源 ---
grep -q "Fenestra Source License (FSL)" LICENSE-COMMERCIAL || fail "层 B 不是 FSL v1.0"
grep -q "商业使用与付费" LICENSE-COMMERCIAL || fail "层 B 缺少商用付费条款"
grep -q "修改必须公开" LICENSE-COMMERCIAL || fail "层 B 缺少「改动必须开源」条款"
grep -q "不可退化" LICENSE-COMMERCIAL || fail "层 B 缺少不可退化条款"

echo "[license-guard] 通过：两份许可证齐备；层 A 符合 OSD / 自撰 / 已公开版本不闭源；层 B 为 FSL v1.0"
