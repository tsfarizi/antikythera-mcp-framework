//! Contract compatibility tests — mechanical verification of golden artifacts.
//!
//! These tests are deliberately **text-only**: they read the golden files under
//! `contracts/` and the real transpiled TypeScript output under `npm/` and
//! assert bidirectional agreement. No WASM runtime, no browser, no wasm-bindgen.
//!
//! Contract under test (per `contracts/browser/README.md`):
//! the WASI component (`wasm32-wasip1`) transpiled by jco exposes the `runner`
//! namespace with 16 camelCase functions whose signatures must match
//! `npm/antikythera-sdk/component/interfaces/antikythera-agent-sdk-runner.d.ts`.
//!
//! Composite-artifact test (`composed_component_world_no_antikythera_imports`)
//! shells out to `wasm-tools component wit` on the composite
//! `dist/antikythera-sdk.wasm` and asserts the composed world imports only
//! `wasi:` interfaces and exports the pinned runner id. This test SKIPs
//! (not fails) when the build artifact or the tool is absent — the composite
//! is a build artifact, not a source unit.
//!
//! Falsification: if the transpiled d.ts drifts (renamed function, changed
//! parameter/return type, added/removed export) without the golden being
//! updated, these tests go RED.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Path resolution
// ---------------------------------------------------------------------------

/// Resolves the repository root.
///
/// The `antikythera-tests` package lives directly under the repo root
/// (`<root>/tests`), so the package manifest dir's parent is the root.
/// `CARGO_MANIFEST_DIR` is read at runtime with the compile-time baked value
/// as fallback; existence of `contracts/` and `npm/` is asserted so a wrong
/// resolution fails loudly instead of producing a confusing file-not-found.
fn repo_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));

    let root = manifest_dir
        .parent()
        .expect("antikythera-tests package must live directly under the repo root")
        .to_path_buf();

    assert!(
        root.join("contracts").is_dir(),
        "repo root must contain contracts/ (resolved root: {})",
        root.display()
    );
    assert!(
        root.join("npm").is_dir(),
        "repo root must contain npm/ (resolved root: {})",
        root.display()
    );
    root
}

fn read_repo_file(root: &Path, rel: &str) -> String {
    let path = root.join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("cannot read repo file {rel} (resolved: {}): {e}", path.display())
    })
}

/// Locates the `wasm-tools` executable for the composition test.
///
/// Resolution order (deterministic, first hit wins):
/// 1. `WASM_TOOLS` environment variable (explicit override),
/// 2. `PATH` entries,
/// 3. `$CARGO_HOME/bin`,
/// 4. `$HOME/.cargo/bin` (falling back to `$USERPROFILE/.cargo/bin` on Windows).
///
/// Returns `None` when no candidate is an existing file; the caller turns
/// that into a SKIP, never a failure — the composite is a build artifact and
/// wasm-tools is a build tool, neither of which belongs to the source unit.
fn find_wasm_tools() -> Option<PathBuf> {
    if let Some(override_path) = std::env::var_os("WASM_TOOLS") {
        let path = PathBuf::from(override_path);
        if path.is_file() {
            return Some(path);
        }
    }

    let exe_name = if cfg!(windows) { "wasm-tools.exe" } else { "wasm-tools" };
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            candidates.push(dir.join(exe_name));
        }
    }
    if let Some(cargo_home) = std::env::var_os("CARGO_HOME") {
        candidates.push(PathBuf::from(cargo_home).join("bin").join(exe_name));
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    if let Some(home) = home {
        candidates.push(PathBuf::from(home).join(".cargo").join("bin").join(exe_name));
    }

    candidates.into_iter().find(|p| p.is_file())
}

const GOLDEN_BROWSER_SIGS: &str = "contracts/browser/browser_function_signatures.golden.txt";
const DTS_RUNNER_INTERFACE: &str =
    "npm/antikythera-sdk/component/interfaces/antikythera-agent-sdk-runner.d.ts";
const DTS_SDK_ROOT: &str = "npm/antikythera-sdk/component/antikythera-sdk.d.ts";
const GOLDEN_PAYLOAD_CONTRACT: &str = "contracts/shared/payload_contract.golden.json";

const EXPECTED_RUNNER_FUNCTION_COUNT: usize = 16;

// Composite-component contract (toolrunner composition).
const COMPOSED_WASM_REL: &str = "dist/antikythera-sdk.wasm";
const COMPOSED_EXPECTED_RUNNER_EXPORT: &str = "export antikythera:agent-sdk/runner@1.0.0";
const COMPOSED_IMPORT_PREFIX: &str = "import wasi:";

// ---------------------------------------------------------------------------
// Parsers
// ---------------------------------------------------------------------------

/// Golden lines: non-blank, non-`#`-comment lines of the form
/// `name(param: type, ...): return`.
fn parse_golden_signatures(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

/// jco d.ts export lines: `export function name(params): ret;` → strip the
/// `export function ` prefix and the trailing `;` to obtain the bare signature.
fn parse_dts_signatures(content: &str) -> Vec<String> {
    let mut signatures = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("export function ") {
            let rest = rest.strip_suffix(';').unwrap_or(rest).trim();
            signatures.push(rest.to_string());
        }
    }
    signatures
}

/// Function name from a signature line `name(params): ret`.
fn signature_name(signature: &str) -> &str {
    signature
        .split('(')
        .next()
        .expect("signature must contain '(' before the parameter list")
        .trim()
}

// ---------------------------------------------------------------------------
// Browser runner signature tests
// ---------------------------------------------------------------------------

#[test]
fn browser_type_signatures_match_golden() {
    let root = repo_root();
    let golden_content = read_repo_file(&root, GOLDEN_BROWSER_SIGS);
    let dts_content = read_repo_file(&root, DTS_RUNNER_INTERFACE);

    let golden = parse_golden_signatures(&golden_content);
    let dts = parse_dts_signatures(&dts_content);

    let golden_names: BTreeSet<&str> = golden.iter().map(|s| signature_name(s)).collect();
    let dts_names: BTreeSet<&str> = dts.iter().map(|s| signature_name(s)).collect();

    // 1) Every golden function must exist in the transpiled d.ts.
    let missing_in_dts: Vec<&&str> = golden_names.difference(&dts_names).collect();
    assert!(
        missing_in_dts.is_empty(),
        "golden functions missing from {DTS_RUNNER_INTERFACE}: {missing_in_dts:?}"
    );

    // 2) Every runner export in the d.ts must be listed in the golden.
    let extra_in_dts: Vec<&&str> = dts_names.difference(&golden_names).collect();
    assert!(
        extra_in_dts.is_empty(),
        "functions exported by {DTS_RUNNER_INTERFACE} but not listed in the golden: {extra_in_dts:?}"
    );

    // 3) Full signature equality — catches parameter/return type drift (e.g. a
    //    jco re-render changing `bigint | undefined` or a renamed parameter).
    let golden_set: BTreeSet<&str> = golden.iter().map(String::as_str).collect();
    let dts_set: BTreeSet<&str> = dts.iter().map(String::as_str).collect();
    assert_eq!(
        golden_set, dts_set,
        "signature sets differ: golden documents signatures the d.ts no longer emits, \
         or the d.ts emits signatures the golden does not document"
    );

    // 4) Count guard — the runner interface is fixed at 16 functions.
    assert_eq!(
        golden.len(),
        EXPECTED_RUNNER_FUNCTION_COUNT,
        "golden must document exactly {EXPECTED_RUNNER_FUNCTION_COUNT} runner functions"
    );
    assert_eq!(
        dts.len(),
        EXPECTED_RUNNER_FUNCTION_COUNT,
        "runner d.ts must export exactly {EXPECTED_RUNNER_FUNCTION_COUNT} functions"
    );
}

#[test]
fn browser_runner_namespace_reexported_in_sdk_dts() {
    let root = repo_root();
    let sdk_dts = read_repo_file(&root, DTS_SDK_ROOT);

    assert!(
        sdk_dts.contains("export * as runner from './interfaces/antikythera-agent-sdk-runner.js';"),
        "{DTS_SDK_ROOT} must re-export the `runner` namespace from the runner interface"
    );
}

// ---------------------------------------------------------------------------
// Shared payload contract golden — artifact integrity
// ---------------------------------------------------------------------------
//
// NOTE ON SCOPE: the payload *semantics* (key names of the wire JSON) live in
// Rust structs and are verified by inspection of `wasm_agent` (see
// contracts/shared/README.md). This test only mechanically guards the golden
// artifact itself: valid JSON, exact top-level entries, non-empty type/fields.

#[test]
fn payload_contract_shapes_match_golden() {
    let root = repo_root();
    let raw = read_repo_file(&root, GOLDEN_PAYLOAD_CONTRACT);
    let json: serde_json::Value = serde_json::from_str(&raw)
        .expect("payload contract golden must be valid JSON");

    let top = json
        .as_object()
        .expect("payload contract golden must be a JSON object");

    let expected_top: BTreeSet<&str> =
        ["prepared_turn", "commit_result", "tool_result", "tool_result_inner"]
            .into_iter()
            .collect();
    let actual_top: BTreeSet<&str> = top.keys().map(String::as_str).collect();
    assert_eq!(
        actual_top, expected_top,
        "payload contract golden top-level entries drifted"
    );

    for (name, entry) in top {
        let obj = entry.as_object().unwrap_or_else(|| {
            panic!("payload contract entry `{name}` must be a JSON object")
        });
        let type_name = obj
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        assert!(
            !type_name.is_empty(),
            "payload contract entry `{name}` must declare a non-empty `type`"
        );
        let fields = obj
            .get("fields")
            .and_then(|v| v.as_object())
            .unwrap_or_else(|| {
                panic!("payload contract entry `{name}` must declare a `fields` object")
            });
        assert!(
            !fields.is_empty(),
            "payload contract entry `{name}` must declare at least one field"
        );
    }
}

// ---------------------------------------------------------------------------
// Composite-component world contract (toolrunner composition)
// ---------------------------------------------------------------------------
//
// Contract clause (per `wit/antikythera.wit` + the `wasm-tools compose` wiring):
// the composite `dist/antikythera-sdk.wasm` (SDK + toolrunner) MUST expose a
// single world whose imports are exclusively `wasi:`-prefixed and whose export
// set pins `antikythera:agent-sdk/runner@1.0.0`. The `tool-registry` interface
// is consumed internally by the composite and MUST NOT leak as an import.
//
// Falsification: `wasm-tools component wit` renders the composite's world as
// text; any `import antikythera:` line, any non-wasi import, or a missing
// runner export goes RED. Assertions are substring searches — never
// line-order dependent — so the test stays deterministic across tool versions.
//
// SKIP (not fail) when the artifact or the tool is absent: the composite is a
// build artifact, not a source unit; CI without `dist/` must stay green.

#[test]
fn composed_component_world_no_antikythera_imports() {
    let root = repo_root();

    let wasm_path = root.join(COMPOSED_WASM_REL);
    if !wasm_path.is_file() {
        eprintln!("Skipping: {COMPOSED_WASM_REL} not found (composite is a build artifact)");
        return;
    }

    let Some(wasm_tools) = find_wasm_tools() else {
        eprintln!(
            "Skipping: wasm-tools not found in WASM_TOOLS, PATH, \
             $CARGO_HOME/bin, or $HOME/.cargo/bin"
        );
        return;
    };

    let output = std::process::Command::new(&wasm_tools)
        .args(["component", "wit"])
        .arg(&wasm_path)
        .output()
        .unwrap_or_else(|e| panic!("failed to execute {wasm_tools:?}: {e}"));

    assert!(
        output.status.success(),
        "wasm-tools component wit failed (status: {}):\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let wit_text = String::from_utf8_lossy(&output.stdout);

    // (a) tool-registry must NOT be imported out of the composite.
    let leaked_import = wit_text
        .lines()
        .find(|line| line.trim_start().starts_with("import antikythera:"));
    assert!(
        leaked_import.is_none(),
        "composite world leaks an `import antikythera:` line: {:?}; \
         tool-registry must be consumed internally by the composite, never imported",
        leaked_import
    );

    // (b) the runner interface is exported with the pinned package id.
    assert!(
        wit_text.contains(COMPOSED_EXPECTED_RUNNER_EXPORT),
        "composite world must export `{COMPOSED_EXPECTED_RUNNER_EXPORT}`"
    );

    // (c) every remaining import is wasi-only.
    let non_wasi_imports: Vec<&str> = wit_text
        .lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with("import ") && !line.starts_with(COMPOSED_IMPORT_PREFIX))
        .collect();
    assert!(
        non_wasi_imports.is_empty(),
        "composite world imports a non-wasi interface: {non_wasi_imports:?}"
    );
}
