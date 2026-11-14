# 贡献指南

欢迎贡献。先说三件容易踩坑的事。

## 1. 许可证边界（务必先读）

本项目使用自撰的 [Hollo Source License v1.0](LICENSE)，**不是 OSI 开源许可证**：

- 你可以免费阅读、修改、自建自用，用于非商业目的；
- 你提交的贡献视为同意按本协议公开，并授予版权人可用于商业版本的非独占使用权；**版权不转让，不需要签 CLA，署名保留**（LICENSE 第 8 条）；
- 你把改过的版本提供给第三方时，必须公开完整对应源码（第 5 条）；
- 任何派生版本不得弱化隐私、安全、自由、速度与兼容（第 6 条）；
- 商业使用需向版权人取得小额付费授权（第 4 条）。

如果你所在组织不允许在这类条款下贡献代码，请不要提交 PR；提出问题与建议同样有价值。

## 2. 内核的红线

- **零第三方依赖**：`Cargo.toml` 的 `[dependencies]` 必须保持为空，CI 有守门脚本。确实需要引入依赖，先开 issue 讨论并同步修改 ROADMAP 的贯穿约束。
- **内核只做三件事**：读清单、仲裁能力、可逆登记。任何"用户能感知的浏览器功能"都应放进插件，不要往 `src/lib.rs` 里塞。
- **能力枚举封闭**：新增 `Capability` 要同步更新 `ALL`、`as_str`、解析、测试、文档与 HEP 版本说明。
- **未知即拒绝**：清单里出现未知字段／未知能力／无法映射的权限，一律报错或拒绝授予，不允许"先放过再说"。
- **不弱化安全**：`default_deny`、`revalidate_untrusted_output`、隔离级别的改动必须有对应测试与说明。

## 3. 开发流程

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo run -- demo
cargo run -- compat examples/extensions/firefox.json
```

提交前请确认以上全绿。PR 描述里请写清：

1. 改了什么、为什么；
2. 是否触及能力模型、清单格式或沙盒策略；
3. 新增或修改了哪些测试；
4. 对五不丢（[PRINCIPLES.md](PRINCIPLES.md)）有无影响。

## 4. 提交插件 / 扩展

Stage 0 只收清单样例与归一器的改进，不收可执行的插件包（还没有运行时、签名与商店）。

- HEP 清单一律放 `examples/`，并保证 `cargo run -- verify <file>` 通过；
- 三家扩展的 `manifest.json` 样例放 `examples/extensions/`，并保证 `cargo run -- compat <file>` 通过；
- 发现新的跨浏览器差异，请在 `src/compat.rs` 的 `warnings` 里补一条，并加测试。

## 5. 报告安全问题

**不要**在公开 issue 里贴 PoC。走 [SECURITY.md](SECURITY.md) 里的私有渠道。善意安全研究适用 LICENSE 第 3 条，不追究第 11 条责任。

## 6. 行为准则

见 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)。简单说：就事论事，不搞人身攻击，不贴别人的隐私数据。
