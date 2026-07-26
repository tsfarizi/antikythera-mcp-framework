//! CLI Test Suite — centralized in tests/cli/
//!
//! Entry point for entire CLI unit test suite, moved from `antikythera-cli/tests/`.
//! Each module corresponds to one source module.
//!
//! Run:
//!   cargo test -p antikythera-tests --test cli_tests

#[path = "error.rs"]
mod error;

#[path = "cli.rs"]
mod cli;

#[path = "config.rs"]
mod config;

#[path = "runtime.rs"]
mod runtime;

#[path = "wasm_harness.rs"]
mod wasm_harness;

#[path = "scenario.rs"]
mod scenario;

#[path = "commands_tests.rs"]
mod commands_tests;

#[path = "log_panel_tests.rs"]
mod log_panel_tests;

#[path = "stdio_tests.rs"]
mod stdio_tests;

// ============================================================================
// Tests moved from framework tests/ (depend on antikythera_cli)
// ============================================================================

#[path = "config/comprehensive_config_security_tests.rs"]
mod config_comprehensive_security;

#[path = "config/parsing_tests.rs"]
mod config_parsing;

#[path = "contract/compatibility_tests.rs"]
mod contract_compatibility;

#[path = "transport_tests/builtin_transport_tests.rs"]
mod builtin_transport;

#[path = "transport_tests/http_transport_cache.rs"]
mod http_transport_cache;

#[path = "provider/type_detection_tests.rs"]
mod provider_type_detection;

#[path = "security/validation_tests.rs"]
mod security_validation;

#[path = "security/rate_limit_tests.rs"]
mod security_rate_limit;

#[path = "security/rate_limit_concurrent.rs"]
mod security_rate_limit_concurrent;

#[path = "security/secrets_tests.rs"]
mod security_secrets;

#[path = "server/validation_tests.rs"]
mod server_validation;
