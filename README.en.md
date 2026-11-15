# Fenestra

> **The browser is a shell. Everything else is a plugin.**

> **Why "Fenestra"?** Latin for *window*. A browser is a window: the frame should stay minimal, solid and replaceable, while everything you see through it is supplied by plugins. The name is about the view, not the frame.

A tiny, cross-platform browser **shell**. Everything a browser normally ships with — rendering engine, network stack, address bar, tabs, bookmarks, history, downloads, settings, themes, protocol handlers, extension runtimes, even the sandbox policy — is a replaceable line in a plugin manifest.

- **Repository**: [github.com/codecloud-dev/Fenestra](https://github.com/codecloud-dev/Fenestra)
- **License**: [Fenestra Source License v1.0](LICENSE) — source-available, **not** OSI open source. The author may use it commercially for free; anyone else needs a small paid commercial license; modifications must be published.
- **Status**: **Stage 0 — kernel skeleton**, not a daily-driver browser. See [ROADMAP.md](ROADMAP.md).

> The Chinese [README.md](README.md) is the canonical documentation. The Chinese LICENSE text governs.

---

## What actually exists today

| Shipped and verified | Not shipped yet |
| --- | --- |
| Rust kernel with **zero third-party dependencies** | A graphical browser you can surf with |
| FEP manifest format + hand-written parser | A real rendering-engine plugin (no bundled Blink/WebKit/Gecko) |
| Capability model: default-deny, privacy-sensitive capabilities require explicit consent | OS-level process sandbox (only policy + arbitration layers so far) |
| Reversible registry: install = register revocable effects, uninstall = exact revoke | Android / Windows / Linux installers |
| Chrome / Edge / Firefox `manifest.json` → FEP normalizer (with a self-contained mini JSON parser) | Extension store, plugin signing and distribution |
| 39 tests green (33 unit + 6 integration), `cargo fmt` and `clippy -D warnings` clean | Performance benchmarks and real-device smoothness data |
| CI on Linux / Windows / macOS + compile checks for Android and iOS targets | — |

The kernel does exactly three things: **read a manifest**, **arbitrate capabilities**, **register revocable effects**. It contains no rendering, no networking, no UI, and no telemetry.

---

## Why "more extreme" than "everything is a plugin"

We researched [DSH (DeepSeek Harness)](docs/research/dsh-everything-is-a-plugin.md), whose "everything is a plugin" architecture (Cordis kernel, MIT, reversible effects) is genuinely clean: model adapters, tool registry, session log, agent loop, system prompt, execution sandbox and UI are all plugins.

But DSH's plugin boundary stops at **application-level capabilities**. The runtime (TypeScript / Node / Electron), the Cordis kernel itself, the process model and the host OS sandbox are given, not replaceable. Its own README declares *developer preview* with compatibility-breaking changes ahead.

Fenestra pushes one step further: the **engine, network stack, platform backend, UI, extension runtime and sandbox policy are all plugins**. Honest counterpoint: DSH has a real plugin ecosystem; Fenestra currently has 39 tests. A more extreme architecture is not a more mature product.

---

## Five things that must not be lost

Freedom, privacy, handiness, speed and smoothness are written into [Article 6 of the LICENSE](LICENSE) as enforceable, testable clauses, and into [PRINCIPLES.md](PRINCIPLES.md). For example:

- `telemetry`, `profile.read`, `clipboard.read`, `device.camera`, `device.location` are privacy-red-line capabilities — never granted without explicit per-capability user confirmation.
- Every engine/network/UI/runtime is behind a trait; `uninstall()` revokes services, subscriptions and capabilities together.
- The kernel starts no runtime, opens no socket and waits for nothing.

---

## Try it

```bash
git clone https://github.com/codecloud-dev/Fenestra.git
cd fenestra
cargo test --all-targets
cargo run -- demo
cargo run -- compat examples/extensions/firefox.json
cargo run -- verify examples/engine.fep
cargo run -- policy
cargo run -- targets
```

A FEP manifest:

```text
id: org.fenestra.engine.blink
name: Blink Engine Adapter
version: 0.1.0
kind: render-engine
entry: engines/blink.wasm
capability: engine.swap
capability: net.connect
```

---

## Extension compatibility

Chrome MV3, Edge MV3 and Firefox MV3 manifests all normalize to the same FEP manifest and go through the same capability arbiter. Differences are **not** silently smoothed over: `declarativeNetRequest` vs `webRequest`, service worker vs event page, `browser_specific_settings.gecko.id`, `data_collection_permissions` (Firefox ≥ 142), and namespace/promise differences are surfaced as `warnings`. See [EXTENSIONS.md](EXTENSIONS.md).

Fenestra's native FEP extensions are strictly better than the compatibility layer: capabilities are declarative, the least-privilege intersection is computed at install time, and there is no guessing what a permission string means.

---

## Security

Modeled on Apple's containment approach in Messages / Safari (BlastDoor, Enhanced Security Extensions): untrusted data is "detonated" inside a near-privilegeless isolated unit, and **its output is still treated as untrusted** — the privileged side must re-validate simple types. In Fenestra this is `SandboxPolicy::detonate()` → `Untrusted<T>::revalidate()`, a type-level reminder, not a convention. Research and sources: [docs/research/apple-sandbox.md](docs/research/apple-sandbox.md).

Honest boundary: Stage 0 has the policy and arbitration layers, **not** an OS-level process isolation implementation.

---

## License in one paragraph

The author may use Fenestra commercially, free and without explanation. Everyone else may read, modify and self-host it for non-commercial purposes for free; **commercial use requires a small, transparent, non-discriminatory paid license**. Any modified version distributed or offered to third parties (including as a service) **must publish its complete corresponding source** under the same terms — paying does not buy the right to close the source. No derivative may lock down plugins, add covert telemetry, weaken the sandbox, deliberately slow the browser, or break Chrome/Edge/Firefox extension compatibility. Full text: [LICENSE](LICENSE).
