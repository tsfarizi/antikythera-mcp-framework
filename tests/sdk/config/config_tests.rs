//! Postcard Configuration Tests

use antikythera_sdk::config::*;

// Split into 5 parts for consistent test organization.
include!("config_tests/empty_placeholder.rs");
include!("config_tests/config_serialization.rs");
include!("config_tests/config_custom_values.rs");
include!("config_tests/config_size.rs");
include!("config_tests/config_defaults.rs");
