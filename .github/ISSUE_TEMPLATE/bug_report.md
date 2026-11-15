---
name: 问题报告
about: 报告内核行为、清单解析、能力仲裁或归一器的错误
title: "[Bug] "
labels: bug
assignees: ''
---

## 现象

一句话说清哪里不对。

## 复现

```bash
cargo test --all-targets
cargo run -- verify <你的清单.fep>
# 或
cargo run -- compat <你的 manifest.json>
```

请贴出**完整命令与完整输出**。若能附上最小清单／manifest 片段最好。

## 期望行为

## 实际行为

## 环境

- 操作系统：
- `rustc --version`：
- Fenestra 版本 / commit：

## 是否涉及安全边界

- [ ] 是 —— **请停手，改走 SECURITY.md 的私有渠道，不要在这里贴细节**
- [ ] 否

## 补充

是否触及能力模型、FEP 清单格式、沙盒策略或三家扩展归一？
