//! Integration Tests with Conditional Execution
//!
//! These tests automatically skip if prerequisites (servers, configs, API keys)
//! are not available. Each skipped test provides clear instructions on how to
//! set up the required dependencies.

mod test_utils;

use test_utils::*;

/// Example test that requires configuration files

// Split into 5 parts for consistent test organization.
include!("integration_tests/config_loading_integration.rs");
include!("integration_tests/ollama_provider_integration.rs");
include!("integration_tests/gemini_provider_integration.rs");
include!("integration_tests/custom_mcp_server.rs");
include!("integration_tests/full_integration_env_check.rs");
