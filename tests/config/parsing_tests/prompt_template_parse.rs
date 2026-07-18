#[test]
fn parses_prompt_template_from_toml() {
    let dir = tempdir().expect("tempdir");
    let mut pc = minimal_toml_config();
    pc.prompts.template = "Be helpful.".to_string();
    let path = write_toml_config(dir.path(), &pc);

    let config = AppConfig::load(Some(&path)).expect("load config");

    assert_eq!(config.prompt_template(), "Be helpful.");
}
