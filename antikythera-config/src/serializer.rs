use super::schema::AppConfig;

/// Convert AppConfig to a human-readable TOML string for display/debug.
///
/// This is a *presentation* format, not the canonical on-disk layout.
/// It inlines `system_prompt` (a runtime-only field) and flattens the
/// prompt template to a top-level `prompt_template` key so operators
/// can inspect the active configuration at a glance.
pub fn to_raw_toml_string(config: &AppConfig) -> String {
    render_display_toml(
        config.system_prompt.as_deref(),
        config.prompt_template(),
    )
}

fn render_display_toml(system_prompt: Option<&str>, prompt_template: &str) -> String {
    let mut raw = String::new();

    if let Some(sp) = system_prompt {
        raw.push_str(&format!("system_prompt = \"{}\"\n\n", escape_toml(sp)));
    }

    raw.push_str("prompt_template = \"\"\"\n");
    raw.push_str(prompt_template);
    if !prompt_template.ends_with('\n') {
        raw.push('\n');
    }
    raw.push_str("\"\"\"\n");

    raw
}

fn escape_toml(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
