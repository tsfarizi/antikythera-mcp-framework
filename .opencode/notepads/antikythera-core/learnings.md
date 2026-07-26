# antikythera-core Learnings

## 2026-07-16: Feature Flags & Error Handling Cleanup

### Feature Flags Audit

All `#[cfg(feature = "...")]` annotations in `antikythera-core/src/application/` are correctly placed:

- **Module boundary** (mod.rs files): `mod` declarations and `pub use` re-exports are gated — these are the ideal locations.
- **Deep in implementation** (all genuinely necessary):
  - `sysinfo::System` import/usage in `agent/runner.rs` — platform-specific system monitoring, cannot be moved.
  - `McpProcess` import + `ServerInstance::Stdio` variant + match arms in `tooling/manager.rs` — the enum variant is feature-gated, so all references must also be gated.
  - `wizard` feature in `stdio/mod.rs` — behavior toggle inside a command handler, appropriate placement.
  - `transport_factory.rs` — conditional stdio transport creation, must be inline.

**Conclusion**: No changes needed. The feature flags are already clean.

### Error Handling Audit

Three error types used manual `Display`/`Error` implementations instead of `thiserror`:

| Type | File | Fix |
|---|---|---|
| `InputValidatorError` | `security/validation/mod.rs` | Added `#[derive(thiserror::Error)]` + `#[error("...")]` attributes, removed manual `Display`/`Error` impls |
| `SecretManagerError` | `security/secrets/error.rs` | Same — replaced manual impls with thiserror derive |
| `EnvelopeError` | `tooling/envelope.rs` | Same — replaced manual impls with thiserror derive |

All 12 error types in core now consistently use `#[derive(thiserror::Error)]`.

### Pre-existing Issues

- `multi_agent/orchestrator/runtime.rs:183`: Missing `event_sender` field in `AgentOptions` initialization. This causes `cargo check --all-features` to fail but is unrelated to the cleanup work. The `event_sender` field was added to `AgentOptions` but the orchestrator runtime was not updated.

### Feature Flags in Core

- `native-transport`: Gates STDIO transport, process spawning, sysinfo monitoring, discovery loader/startup
- `multi-agent`: Gates `multi_agent` module (orchestrator, task types)
- `wizard`: Gates interactive config editor in STDIO mode
- `component` is on `antikythera-sdk`, not `antikythera-core`
