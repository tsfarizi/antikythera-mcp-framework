// Wizard tests - verifying configuration wizard functionality
//
// Tests for wizard configuration generation including HTTP server support.

mod http_server_generation_tests {
    use std::collections::HashMap;

    /// Helper to generate HTTP server TOML block (mirrors client::add_http_server logic)
    fn generate_http_server_toml(
        name: &str,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> String {
        let headers_toml = if headers.is_empty() {
            String::new()
        } else {
            let pairs: Vec<String> = headers
                .iter()
                .map(|(k, v)| format!("{} = \"{}\"", k, v))
                .collect();
            format!("\nheaders = {{ {} }}", pairs.join(", "))
        };

        format!(
            r#"
[[servers]]
name = "{}"
url = "{}"{}"#,
            name, url, headers_toml
        )
    }

// Split into 5 parts for consistent test organization.
include!("wizard_tests/http_server_toml_headers.rs");
include!("wizard_tests/toml_generation_stdio.rs");
include!("wizard_tests/mask_sensitive_values.rs");
include!("wizard_tests/mask_boundary_transport.rs");
include!("wizard_tests/transport_config_creation.rs");
