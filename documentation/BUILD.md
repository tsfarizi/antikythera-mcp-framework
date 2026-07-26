# BUILD

This guide covers the commands that match the current workspace layout and tooling.

## Build map

```mermaid
flowchart TD
    SRC[Workspace source] --> CARGO[cargo build --workspace]
    SRC --> CLI[cargo build -p antikythera-cli --release]
    SRC --> WIT[cargo run -p build-scripts --release -- validate]
    WIT --> COMPONENT[cargo component build -p antikythera-sdk --release --target wasm32-wasip1]
```

## What you can build

| Target | Status | Notes |
|:-------|:------:|:------|
| Workspace crates | ✅ | `cargo build --workspace` — all 11 framework crates + tests + scripts |
| `antikythera-sdk` component build | ✅ | Single WASM output via `cargo-component` + `wasm32-wasip1` |
| `antikythera` native binary (example) | ✅ | stdio, setup, and multi-agent modes — standalone, not a workspace member |
| `antikythera-config` native binary (example) | ✅ | Provider and server config management — standalone, not a workspace member |

## Prerequisites

- Rust 1.85+
- `cargo-component` for component builds
- Optional: `wasm-tools` for inspecting the generated component
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

### Build the example CLI crate

The CLI is a standalone crate (not a workspace member). Build it explicitly:

```bash
cargo build -p antikythera-cli --release
```

### Native binaries

| Binary | Command |
|:-------|:--------|
| `antikythera` | `cargo run -p antikythera-cli --bin antikythera` |
| `antikythera-config` | `cargo run -p antikythera-cli --bin antikythera-config -- --help` |

## WASM component build

### Validate WIT

```bash
cargo run -p build-scripts --release -- validate
```

This validates the checked-in WIT file against Rust source types.

### Build the WASM component

```bash
cargo component build -p antikythera-sdk --release --target wasm32-wasip1 \
  --no-default-features --features component
```

The helper binary in `scripts/build-component.rs` validates WIT conformance against Rust source types.

Expected component output is produced under:

```text
target/wasm32-wasip1/release/
```

Canonical artifact name for CI/release packaging:

```text
dist/antikythera-sdk.wasm
```

## CLI harness against WASM

Use the example CLI to execute the generated WASM via host runtime bridge (`WasmAgentRunner`):

```bash
cargo run -p antikythera-cli --bin antikythera -- \
    --mode wasm-harness \
    --wasm target/wasm32-wasip1/release/antikythera_sdk.wasm \
    --task "Smoke test"
```

Optional deterministic host callback payload:

```bash
cargo run -p antikythera-cli --bin antikythera -- \
    --mode wasm-harness \
    --wasm target/wasm32-wasip1/release/antikythera_sdk.wasm \
    --wasm-llm-response '{"content":"ok","model":"stub"}'
```

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
| `task build` | Build the WASM component |
| `task build-wasm-harness` | Build WASM artifact for CLI harness |
| `task build-all` | Build all WASM artifacts |
| `task build-web` | Build web frontend for production |
| `task wit` | Validate WIT conformance |
| `task run-cli` | Run interactive TUI CLI with auto-bootstrap config |
| `task run-wasm` | Run CLI as WASM FFI host harness |
| `task run-interactive` | Run interactive CLI (stdio mode) |
| `task run-tui` | Alias for interactive TUI-friendly CLI |
| `task run-web` | Run web frontend dev server (Vite) |
| `task setup-config` | Setup app.toml with selectable provider |
| `task test` | Run workspace tests |
| `task test-unit` | Run all CLI unit tests (centralized in tests/) |
| `task test-scenario` | Run hands-on CLI scenario test (real LLM) |
| `task test-cli-wasm-host` | Run CLI host-FFI WASM smoke test |
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

- The component build (`wasm32-wasip1`) is the WASM deployment target. Use it when embedding agent logic in a host application via wasmtime.
- The example CLI is a standalone crate, not a workspace member. It consumes framework crates via relative path dependencies.
- For browser or C FFI targets, implement those in the host application itself — the framework does not provide those bindings.
