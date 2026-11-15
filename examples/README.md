# 示例

```bash
# FEP 清单校验
cargo run -- verify examples/engine.fep
cargo run -- verify examples/ui.fep
cargo run -- verify examples/annotate.fep

# 三家扩展 manifest.json → FEP 归一（会打印差异 warnings 与生成的 FEP）
cargo run -- compat examples/extensions/chrome.json
cargo run -- compat examples/extensions/edge.json
cargo run -- compat examples/extensions/firefox.json
```

`extensions/*.json` 与 `src/compat.rs` 里的内置测试样例保持一致，CI 也会拿 `firefox.json` 真跑一次归一。

注意：`.wasm` 入口文件此刻并不存在。Stage 0 只校验清单与能力，不加载、不执行插件代码；`fenestra verify` 检查的是清单本身是否合法。
