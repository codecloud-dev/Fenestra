#!/usr/bin/env python3
"""docs 守门：相对链接有效性 + 命名一致性 + 徽章阶段 + 关键章节齐备。"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
errors: list[str] = []


def fail(msg: str) -> None:
    errors.append(msg)


# 1. 必备文件齐备且非空
required = [
    "README.md",
    "README.en.md",
    "LICENSE",
    "LICENSE-COMMERCIAL",
    "ARCHITECTURE.md",
    "ROADMAP.md",
    "PRINCIPLES.md",
    "SECURITY.md",
    "CONTRIBUTING.md",
    "CHANGELOG.md",
    "CODE_OF_CONDUCT.md",
    "Cargo.toml",
    "package.json",
    "pnpm-workspace.yaml",
    ".github/workflows/ci.yml",
    ".github/dependabot.yml",
]
for rel in required:
    p = ROOT / rel
    if not p.is_file() or p.stat().st_size == 0:
        fail(f"缺少或空文件: {rel}")

md_files = sorted(ROOT.glob("*.md"))

# 2. 相对链接指向的文件 / 目录存在
link_re = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
for md in md_files:
    for target in link_re.findall(md.read_text(encoding="utf-8")):
        t = target.strip()
        if not t or t.startswith(("http://", "https://", "mailto:", "#")):
            continue
        path_part = t.split("#")[0]
        if not path_part:
            continue
        if not (ROOT / path_part.removeprefix("./")).exists():
            fail(f"{md.name}: 链接目标不存在 -> {t}")

# 3. 命名一致性：不得残留旧名
for md in md_files:
    for bad in ("Hollo", "hollo"):
        if bad in md.read_text(encoding="utf-8"):
            fail(f"{md.name}: 残留旧名 {bad}")

# 4. 徽章阶段 = Stage 1（已开工），且不残留 Stage_0
readme = (ROOT / "README.md").read_text(encoding="utf-8")
if "Stage_1" not in readme and "Stage 1" not in readme:
    fail("README.md 徽章阶段未更新为 Stage 1")
if "Stage_0" in readme:
    fail("README.md 仍残留 Stage_0 徽章")

# 5. 关键章节关键词
for kw in ("双许可", "层 A", "层 B", "MVP", "stage0", "现在做到哪一步"):
    if kw not in readme:
        fail(f"README.md 缺少关键章节关键词: {kw}")

if errors:
    print("[docs] 检查失败:")
    for e in errors:
        print("  -", e)
    sys.exit(1)

print(f"[docs] 通过：{len(md_files)} 份根 markdown，链接 / 命名 / 徽章 / 章节齐备")
