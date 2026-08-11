# BUILD

This guide covers the commands that match the current workspace layout and tooling.

## Build map

```mermaid
flowchart TD
    SRC[Workspace source] --> CARGO[cargo build --workspace]
    SRC --> WIT[cargo run -p build-scripts --release -- validate]
    WIT --> SDK[cargo component build -p antikythera-sdk ... --features component]
    WIT --> TR[cargo component build -p antikythera-toolrunner ... --features component]
    SDK --> COMPOSE[wasm-tools compose SDK + toolrunner]
    TR --> COMPOSE
    COMPOSE --> DIST[dist/antikythera-sdk.wasm - composite]
    DIST --> JCO[npx jco transpile dist/antikythera-sdk.wasm --out-dir npm/antikythera-sdk/component]
    JCO --> NPM[npm/antikythera-sdk/component/ ESM bindings]
    DIST --> HARNESS[examples/component-harness - wasmtime server proof]
```

## What you can build

| Target | Status | Notes |
|:-------|:------:|:------|
| Workspace crates | ✅ | `cargo build --workspace` — all framework crates + tests + scripts |
| `antikythera-sdk` component | ✅ | Standalone intermediate via `cargo-component` + `wasm32-wasip1` (feature `component`); imports `tool-registry` — **not** a consumable deliverable |
| `antikythera-toolrunner` component | ✅ | Standalone intermediate via `cargo-component` + `wasm32-wasip1` (feature `component`); exports `tool-registry` (builtin tools) |
| **Composite WASM deliverable** | ✅ | `wasm-tools compose` SDK + toolrunner → `dist/antikythera-sdk.wasm` (imports only WASI, exports `runner`) |
| Browser JS bindings | ✅ | `jco transpile` of the **composite** → `npm/antikythera-sdk/component/` (ESM, namespace `runner`) |

## Prerequisites

- recent stable Rust toolchain (edition 2024)
- `cargo-component` for component builds
- `wasm-tools` for composing the SDK + toolrunner components
- Node.js with ESM support and `@bytecodealliance/jco` (installed at repo root; the version pinned in the root package.json) for the browser JS bindings
- Optional: `task` for the helpers in `Taskfile.yml`

## Native builds

### Build all workspace crates

```bash
cargo build --workspace
```

### Build release artifacts

```bash
cargo build --workspace --release
```

## WASM component build

### Validate WIT

```bash
cargo run -p build-scripts --release -- validate
```

This validates the checked-in WIT file against Rust source types.

### Build the composite WASM component

The WASM deliverable is **composite**: the SDK component imports `tool-registry`, the toolrunner component exports it, and `wasm-tools compose` wires the two. Build the two components, then compose:

```bash
cargo component build -p antikythera-sdk --release --target wasm32-wasip1 \
  --no-default-features --features component
cargo component build -p antikythera-toolrunner --release --target wasm32-wasip1 \
  --no-default-features --features component
```

The helper binary in `scripts/build-component.rs` validates WIT conformance against Rust source types.

Expected component output is produced under:

```text
target/wasm32-wasip1/release/
```

Compose into the canonical composite artifact (the toolrunner is copied to a kebab-case name first because `wasm-tools compose` rejects underscores):

```bash
mkdir -p dist
cp target/wasm32-wasip1/release/antikythera_toolrunner.wasm \
  target/wasm32-wasip1/release/antikythera-toolrunner.wasm
wasm-tools compose target/wasm32-wasip1/release/antikythera_sdk.wasm \
  -d target/wasm32-wasip1/release/antikythera-toolrunner.wasm \
  -o dist/antikythera-sdk.wasm
```

Canonical artifact name for CI/release packaging:

```text
dist/antikythera-sdk.wasm
```

`task compose` wraps the copy + compose steps (and depends on both component builds); `task build` runs the full composite build (WIT validation + both components + compose).

> **Never transpile or embed the standalone SDK component.** `target/wasm32-wasip1/release/antikythera_sdk.wasm` still imports `tool-registry`; jco transpilation of it yields a module with an unmet import that fails Node smoke tests. Always consume the composite.

### Verify the composite server-side

`examples/component-harness` is a wasmtime server binary that reads `dist/antikythera-sdk.wasm` and proves the builtin tool `echo` executes inside the composite with no host round-trip:

```bash
cargo run -p component-harness --release
```

### Transpile the composite to browser JS bindings (jco)

The browser path reuses the **composite** WASI component and transpiles it to browser-safe ESM with `@bytecodealliance/jco`. Run `task compose` first (or `task transpile`, which depends on `compose`):

```bash
npx jco transpile dist/antikythera-sdk.wasm --out-dir npm/antikythera-sdk/component \
  -M wasi:cli/environment=./wasi-stubs/environment.js \
  -M wasi:cli/exit=./wasi-stubs/exit.js \
  -M wasi:cli/stderr=./wasi-stubs/stderr.js \
  -M wasi:cli/stdin=./wasi-stubs/stdin.js \
  -M wasi:cli/stdout=./wasi-stubs/stdout.js \
  -M wasi:clocks/monotonic-clock=./wasi-stubs/monotonic-clock.js \
  -M wasi:clocks/wall-clock=./wasi-stubs/wall-clock.js \
  -M wasi:filesystem/preopens=./wasi-stubs/preopens.js \
  -M wasi:filesystem/types=./wasi-stubs/types.js \
  -M wasi:io/error=./wasi-stubs/error.js \
  -M wasi:io/streams=./wasi-stubs/streams.js \
  -M wasi:random/random=./wasi-stubs/random.js
```

The 12 `-M` flags map every WASI import (cli, io, clocks, filesystem, random) to a browser-safe stub under `npm/antikythera-sdk/component/wasi-stubs/`. A `transpile` task is available in `Taskfile.yml` as a shortcut.

Consume from JS as:

```javascript
import { runner } from 'antikythera-agent/component';
const sessionId = runner.init(JSON.stringify({ session_id: 's1' }));
```

See [`WASM_ARCHITECTURE.md`](WASM_ARCHITECTURE.md) for the full `runner` contract and jco pitfalls (jco has no `--dts`; ES2022 target required for top-level await).

## Docs site build

The repository also includes an `mdBook` configuration that turns `README.md` plus the `documentation/` folder into a static documentation site.

```mermaid
flowchart LR
    README[README.md] --> SUMMARY[SUMMARY.md]
    DOCS[documentation/*.md] --> SUMMARY
    SUMMARY --> MDBOOK[mdbook build]
    MDBOOK --> SITE[book/]
    SITE --> PAGES[GitHub Pages deployment]
```

### Local commands

```bash
# Build static site
mdbook build

# Preview locally
mdbook serve --open
```

## Tests and quality checks

### Verification flow

```mermaid
flowchart LR
    CHECK[cargo check --workspace] --> TEST[cargo test --workspace]
    TEST --> FMT[cargo fmt --all -- --check]
    FMT --> CLIPPY[cargo clippy --workspace --lib --bins -- -D warnings -D deprecated]
```

### Workspace-wide

```bash
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --lib --bins -- -D warnings -D deprecated
```

### Common targeted checks

```bash
# SDK library tests
cargo test -p antikythera-sdk --lib

# Check all workspace crates without producing binaries
cargo check --workspace
```

## Taskfile helpers

The repository includes `Taskfile.yml` for common flows.

| Task | Purpose |
|:-----|:--------|
| `task build` | Build the composite WASM component (SDK + toolrunner + compose) |
| `task build-wasm-harness` | Build the standalone SDK WASM artifact (intermediate) |
| `task build-toolrunner` | Build the standalone toolrunner WASM artifact (intermediate) |
| `task compose` | Compose SDK + toolrunner into `dist/antikythera-sdk.wasm` |
| `task build-all` | Build all WASM artifacts (standalone SDK + toolrunner + composite) |
| `task wit` | Validate WIT conformance |
| `task transpile` | Transpile the **composite** to JS bindings with jco (depends on `compose`) |
| `task transpile-clean` | Remove transpiled JS output directory |
| `task test` | Run workspace tests |
| `task test-sdk` | Run SDK tests |
| `task test-ffi` | Run FFI tests |
| `task test-component` | Run component tests |
| `task check` | Run `cargo check --workspace` |
| `task check-wasm` | Check WASM compilation |
| `task lint` | Run Clippy |
| `task format` | Run rustfmt |
| `task inspect` | Inspect the built component with `wasm-tools` if installed |
| `task size` | Show binary sizes |
| `task clean` | Clean all build artifacts |
| `task clean-wasm` | Clean WASM build artifacts |

## Notes

- The composite build (`wasm32-wasip1`, feature `component` on both crates, then `wasm-tools compose`) is the WASM deployment target for both server and browser. Use `dist/antikythera-sdk.wasm` when embedding agent logic in a host application via wasmtime (`examples/component-harness` proves the builtin tool path), and transpile the composite with jco for browser consumption.
- The standalone `antikythera-sdk` component is an intermediate artifact: it imports `tool-registry` and must be composed with `antikythera-toolrunner` before any consumer (wasmtime harness, jco) touches it.
- The legacy wasm-bindgen path (`wasm32-unknown-unknown` + `wasm` feature, `plugin/antikythera-wasm-bindgen`, wasm-pack) is **deprecated**; it is kept only for crate-level compatibility during the transition. Do not use it for new browser integrations.
