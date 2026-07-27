use antikythera_cli::cli::{Cli, Command};
use clap::Parser;

#[test]
fn default_invocation_has_no_subcommand() {
    let cli = Cli::parse_from(["antikythera"]);
    assert!(cli.command.is_none());
}

#[test]
fn config_subcommand_parsed() {
    let cli = Cli::parse_from(["antikythera", "config", "show"]);
    match cli.command {
        Some(Command::Config { .. }) => {}
        other => panic!("expected Config subcommand, got {:?}", other),
    }
}

#[test]
fn wasm_harness_subcommand_parsed() {
    let cli = Cli::parse_from(["antikythera", "wasm-harness"]);
    match cli.command {
        Some(Command::WasmHarness { .. }) => {}
        other => panic!("expected WasmHarness subcommand, got {:?}", other),
    }
}

#[test]
fn cli_default_ollama_url_points_to_localhost() {
    let cli = Cli::parse_from(["antikythera"]);
    assert_eq!(cli.ollama_url, "http://127.0.0.1:11434");
}

#[test]
fn cli_stream_flag_is_false_by_default() {
    let cli = Cli::parse_from(["antikythera"]);
    assert!(!cli.stream);
}

#[test]
fn cli_stream_flag_enabled_by_long_flag() {
    let cli = Cli::parse_from(["antikythera", "--stream"]);
    assert!(cli.stream);
}

#[test]
fn cli_provider_and_model_overrides_are_optional() {
    let cli = Cli::parse_from(["antikythera"]);
    assert!(cli.provider.is_none());
    assert!(cli.model.is_none());
    assert!(cli.provider_endpoint.is_none());
}

#[test]
fn cli_provider_override_accepted() {
    let cli = Cli::parse_from(["antikythera", "--provider", "gemini"]);
    assert_eq!(cli.provider.as_deref(), Some("gemini"));
}

#[test]
fn cli_execution_mode_defaults_to_auto() {
    let cli = Cli::parse_from(["antikythera"]);
    assert_eq!(cli.execution_mode, "auto");
}

#[test]
fn cli_multi_agent_flag_default_false() {
    let cli = Cli::parse_from(["antikythera"]);
    assert!(!cli.multi_agent);
}

#[test]
fn cli_wasm_harness_accepts_wasm_path() {
    let cli = Cli::parse_from(["antikythera", "wasm-harness", "--wasm", "test.wasm"]);
    match cli.command {
        Some(Command::WasmHarness { wasm, .. }) => {
            assert_eq!(wasm.as_deref(), Some("test.wasm"));
        }
        other => panic!("expected WasmHarness, got {:?}", other),
    }
}
