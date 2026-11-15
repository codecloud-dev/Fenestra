# Fenestra · Only a shell

> 只是个壳，其余皆插件 / *Only a shell, everything else is a plugin.*

Fenestra is a deliberately tiny **browser shell**. The window, address bar, tabs and plugin manager are the shell; the rendering engine, network stack, UI and acceleration are all plugins — installed, run and removed **live, without restarting the process**. It ships under a two-layer license so "trustworthy" and "sustainable" do not have to be a trade-off.

- **Layer A (truly open source)**: kernel, permission gate / capability arbitration, plugin runtime, FEP spec, example plugins.
- **Layer B (source-available / paid commercial)**: official value-added plugins, future enterprise edition.

This README is the English companion; the authoritative, fuller Chinese document is [README.md](./README.md). Badges and the detailed status table live there.

## Status (honest)

We are at **Stage 1: the rewrite scaffold + a verifiable minimal chain**. Numbers below come from real local runs.

| Piece | Status | Evidence |
| --- | --- | --- |
| Rust gate [crates/gate](./crates/gate/) | ✅ tested locally | `cargo test -p fenestra-gate` → 18 passed / 0 failed; clippy `-D warnings` clean |
| Cordis 4 runtime [packages/core](./packages/core/) | ✅ tested locally | `pnpm --filter @fenestra/core test` → 11 passed / 0 failed; typecheck exit 0 |
| Svelte 5 + Vite 8 UI [packages/ui](./packages/ui/) | ✅ built locally | `pnpm --filter @fenestra/ui build` → vite 8.3.8, 110 modules, exit 0 |
| Example plugins [packages/plugins](./packages/plugins/) | ✅ typechecked | `pnpm run plugins:typecheck` → 3/3 |
| Tauri 2 shell [apps/shell/src-tauri](./apps/shell/src-tauri/) | ⚠️ **not built locally** | the sandbox lacks `libwebkit2gtk-4.1-dev`; compiled for real in CI after installing system deps |
| End-to-end wiring (gate ↔ runtime ↔ IPC ↔ UI) | ⬜ not done | packages pass independently; integration is next |

We do **not** publish unbenchmarked performance numbers and do **not** promise a release date.

## Tech stack (pinned)

| Module | Choice | Verified version |
| --- | --- | --- |
| Plugin kernel | Cordis 4 + TypeScript | `cordis 4.0.0-rc.13` |
| Shell | Tauri 2 + Rust | Tauri 2.11.5 stable line; `@tauri-apps/cli 2.12.4`, `@tauri-apps/api 2.12.4`; Rust crate on the `2` line, Cargo-locked |
| UI | Svelte 5 | `svelte 5.57.3` |
| Build | Vite 8 + pnpm 10 + Cargo | `vite 8.3.8`, `pnpm 10.28.2` |
| Default render | system WebView (wry) | no bundled engine; Servo is the Stage 1 swappable-engine plugin |

The engine is abstracted as a `RenderEngine` trait in [apps/shell/src-tauri/src/lib.rs](./apps/shell/src-tauri/src/lib.rs) — registration, not compile-time binding.

## Dual licensing

| Layer | Covers | License |
| --- | --- | --- |
| **A** | kernel, gate / arbitration, runtime, FEP, example plugins | [LICENSE](./LICENSE) — *Fenestra Open License v1.0*, a self-written permissive OSD-conforming license (**not OSI-approved**); **released versions may never be re-proprietarized** |
| **B** | official value-added plugins, future enterprise edition | [LICENSE-COMMERCIAL](./LICENSE-COMMERCIAL) — *Fenestra Source License (FSL) v1.0* verbatim; paid commercial use, modifications must stay open |

Boundary rules: Layer A's API/ABI and the gate itself stay open forever; Layer B may **never** weaken default-deny, sandbox isolation, privacy red lines, or outbound revalidation — not even for paying customers. The contributor agreement covers Layer A only. See [README.md](./README.md) §双许可边界.

## Reproduce locally

```bash
cargo test -p fenestra-gate
cargo clippy -p fenestra-gate --all-targets -- -D warnings
pnpm install --frozen-lockfile
pnpm --filter @fenestra/core typecheck
pnpm --filter @fenestra/core test
pnpm run plugins:typecheck
pnpm --filter @fenestra/ui build
bash scripts/check-licenses.sh
python3 scripts/check-docs.py
```

The Tauri shell needs system libraries (`libwebkit2gtk-4.1-dev` and friends); it is built only in CI's `shell` job:

```bash
cargo build --manifest-path apps/shell/src-tauri/Cargo.toml
```

## Security invariants

Default deny, least-privilege intersection, explicit per-capability consent for privacy red lines, BlastDoor-style `Untrusted<T>` outbound revalidation, and revertible registration — all in [crates/gate](./crates/gate/) with 18 tests. Reversibility on the TS side comes from Cordis 4's revertible effects ([packages/core](./packages/core/), 11 tests). See [SECURITY.md](./SECURITY.md).

## Roadmap & docs

- [ARCHITECTURE.md](./ARCHITECTURE.md) · [ROADMAP.md](./ROADMAP.md) · [PRINCIPLES.md](./PRINCIPLES.md)
- [EXTENSIONS.md](./EXTENSIONS.md) (FEP) · [CONTRIBUTING.md](./CONTRIBUTING.md) · [CHANGELOG.md](./CHANGELOG.md)

**Stage 2 is a hard acceptance item**: full Chrome MV3 / Edge MV3 / Firefox MV3 compatibility, with first-class Fenestra-native plugins best in capability and performance. MVP does not build the compatibility layer, but the kernel API is designed for it from day one.

## Non-goals

No self-written HTML/CSS/JS engine, no feature-parity race, no kernel/VPN/cross-border acceleration, no accounts / cloud sync / telemetry, no user data collection, zero operating cost. No release-date promises.

*Fenestra · codecloud-dev · 2026 · [LICENSE](./LICENSE) / [LICENSE-COMMERCIAL](./LICENSE-COMMERCIAL)*
