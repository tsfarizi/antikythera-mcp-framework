# Testing

This document describes the active test strategy for the workspace.

## Test Topology

```mermaid
flowchart TD
    Unit[Unit and module tests] --> Crate[crate-level validation]
    Crate --> Integration[integration test binaries]
    Integration --> Contracts[contract compatibility checks]
    Integration --> Runtime[runtime behavior checks]
```

## Current Structure

- Workspace-wide checks: build, test, fmt, and clippy gates.
- `tests/` contains integration suites and module-specific binaries.
- Large suites use part-based organization for readability and maintenance.
- Contract fixtures are used for compatibility detection.
- Storage tests are in `tests/storage/` covering cache, filesystem, config, backup, and integration.
- Example CLI tests are in `example/antikythera-cli/` (not a workspace member).

## Storage Tests

```bash
# Run all storage tests
cargo test -p antikythera-tests --test storage_cache_tests
cargo test -p antikythera-tests --test storage_filesystem_tests
cargo test -p antikythera-tests --test storage_config_tests
cargo test -p antikythera-tests --test storage_backup_tests
cargo test -p antikythera-tests --test storage_integration_tests
```

## Standard Commands

```bash
cargo test --workspace
cargo test -p antikythera-tests --no-run
cargo fmt --all -- --check
cargo clippy --workspace --lib --bins -- -D warnings -D deprecated
```
