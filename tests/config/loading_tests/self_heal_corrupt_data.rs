#[test]
fn self_heals_when_toml_data_is_corrupt() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("app.toml");
    fs::write(&path, "this is not valid TOML {{{").expect("write");

    // Corrupt data triggers self-heal — returns a fresh default rather than an error.
    // Core is provider-agnostic: defaults are empty, requiring explicit configuration.
    let config = AppConfig::load(Some(&path)).expect("self-heal should succeed");
    assert!(config.default_provider().is_empty());
    assert!(config.model_name().is_empty());
    assert!(!config.prompts.template().is_empty(), "prompt template should have a built-in default");
}

#[test]
fn loads_routing_strings_from_toml() {
    let dir = tempdir().expect("tempdir");
    let mut pc = minimal_toml_config();
    pc.model.default_provider = "gemini".to_string();
    pc.model.model = "gemini-1.5-flash".to_string();
    let path = write_toml_config(dir.path(), &pc);

    let config = AppConfig::load(Some(&path)).expect("load config");
    assert_eq!(config.default_provider(), "gemini");
    assert_eq!(config.model_name(), "gemini-1.5-flash");
}

