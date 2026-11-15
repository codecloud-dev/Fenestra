# 贡献指南 / Contributing

> 欢迎贡献。Fenestra 只是个壳，其余皆插件 —— 但**贡献者协议只覆盖层 A**。

## 1. 先分清你在改哪一层

| 层 | 目录 | 许可 | 贡献 |
| --- | --- | --- | --- |
| **A** | [crates/gate](./crates/gate/)、[packages/core](./packages/core/)、[packages/plugins](./packages/plugins/) 示例、[apps/shell/src-tauri](./apps/shell/src-tauri/)、[EXTENSIONS.md](./EXTENSIONS.md) FEP 规范 | [LICENSE](./LICENSE)（自撰开源） | ✅ 欢迎 PR，见下文 |
| **B** | 官方增值插件 / 未来企业版（本仓库当前尚无） | [LICENSE-COMMERCIAL](./LICENSE-COMMERCIAL)（FSL v1.0） | 授权发放与收款单独记账，不混进层 A 治理 |

向层 A 提交贡献，视为同意按 [LICENSE](./LICENSE) 第 8 条公开：保留署名、版权不转让、**无需签署 CLA**；版权人不得抹去署名单独重新许可给第三方。

## 2. 本地复现（提交前请真跑）

```bash
cargo test -p fenestra-gate                              # 18 passed / 0 failed
cargo clippy -p fenestra-gate --all-targets -- -D warnings  # 通过
pnpm install --frozen-lockfile
pnpm --filter @fenestra/core typecheck                  # exit 0
pnpm --filter @fenestra/core test                       # 11 passed / 0 failed
pnpm run plugins:typecheck                             # 3/3 Done
pnpm --filter @fenestra/ui build                        # exit 0
bash scripts/check-licenses.sh                          # 双许可守门
python3 scripts/check-docs.py                           # 文档 / 链接 / 命名守门
```

CI 六 job（[.github/workflows/ci.yml](./.github/workflows/ci.yml)）：`gate / core / ui / shell / license-guard / docs`。**不许用 `continue-on-error` 或 `|| true` 把红变绿**；任何失败如实报告。

Tauri 外壳需 `libwebkit2gtk-4.1-dev` 等系统库，本沙箱缺、本地不编译；CI 的 `shell` job 安装依赖后 `cargo build --manifest-path apps/shell/src-tauri/Cargo.toml` 真编译。改了 [apps/shell/src-tauri](./apps/shell/src-tauri/) 请确保该 job 通过。

## 3. PR 要求

- 一个 PR 只做一件事，标题写清改哪层、为什么；
- 新增 / 修改行为**必须带测试**，且测试真实运行通过（贴通过数 / 失败数）；
- 不改 GitHub 用户名 / 显示名 / bio，不碰其它仓库，不发无关的 PR 评论；
- 不弱化默认拒绝 / 沙箱隔离 / 隐私红线 / 出站重校验 / 可逆登记（见 [PRINCIPLES.md](./PRINCIPLES.md)）；
- 不写没有真机基准的性能数字，不承诺工期；
- 许可证自撰、不用现成模板、不用 PGP。

## 4. 代码约定

- Rust：`cargo fmt` 风格、`#![forbid(unsafe_code)]`、clippy `-D warnings` 零警告；
- TypeScript：`strict`、可逆登记用 `FenestraApi`（`effect / provide / on`），不要把状态挂在插件外造成卸载残留；
- 插件权限**只声明不授予**，授权交给闸门；
- 文档中文为主、中英融合；链接用仓库内真实路径（[scripts/check-docs.py](./scripts/check-docs.py) 会校验）。

## 5. 行为准则

请读 [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md)。安全问题见 [SECURITY.md](./SECURITY.md)，架构见 [ARCHITECTURE.md](./ARCHITECTURE.md)，路线见 [ROADMAP.md](./ROADMAP.md)。
