//! Comprehensive Configuration Security Tests
//!
//! Extensive test suite for antikythera-core configuration with focus on:
//! - Input validation and bounds checking
//! - Injection prevention (command, SQL, path traversal)
//! - Unicode and special character handling
//! - Empty and null-like inputs
//! - URL and protocol validation
//! - Transport type handling
//! - Performance under stress

use antikythera_cli::config::{ModelInfo as PcModelInfo, ProviderConfig};
use antikythera_cli::infrastructure::llm::{ModelInfo, ModelProviderConfig};
use antikythera_core::config::{ServerConfig, TransportType};
use std::collections::HashMap;
use std::path::PathBuf;

// Split by concern to keep file size manageable and improve readability.
include!("comprehensive_config_security_tests/provider_config_basics.rs");
include!("comprehensive_config_security_tests/server_config_basics.rs");
include!("comprehensive_config_security_tests/injection_prevention.rs");
include!("comprehensive_config_security_tests/edge_case_boundaries.rs");
include!("comprehensive_config_security_tests/url_protocol_validation.rs");
include!("comprehensive_config_security_tests/transport_type_tests.rs");
include!("comprehensive_config_security_tests/stress_performance.rs");
