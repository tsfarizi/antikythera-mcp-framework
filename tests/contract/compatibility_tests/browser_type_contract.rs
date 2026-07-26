/// Parse exported function signatures from a TypeScript .d.ts file.
/// Returns lines like: `init(config_json: string): string`
fn extract_browser_function_signatures(dts_content: &str) -> Vec<String> {
    let mut output = Vec::new();

    for line in dts_content.lines() {
        let trimmed = line.trim();

        // Match lines starting with "export function"
        if let Some(rest) = trimmed.strip_prefix("export function ") {
            // Extract up to the closing semicolon
            if let Some(sig) = rest.strip_suffix(';') {
                // Normalize whitespace: collapse multiple spaces to single
                let normalized: String = sig.split_whitespace().collect::<Vec<_>>().join(" ");
                output.push(normalized);
            }
        }
    }

    output.sort();
    output
}

#[test]
#[serial_test::serial]
fn browser_type_signatures_match_golden() {
    let dts_path = repo_root()
        .join("example")
        .join("antikythera-web")
        .join("src")
        .join("shared")
        .join("wasm")
        .join("pkg")
        .join("antikythera_wasm_bindgen.d.ts");

    let dts_content = fs::read_to_string(&dts_path)
        .unwrap_or_else(|e| panic!("Failed to read .d.ts file at {}: {e}", dts_path.display()));

    let actual = extract_browser_function_signatures(&dts_content);

    let golden_path = repo_root()
        .join("contracts")
        .join("browser")
        .join("browser_function_signatures.golden.txt");

    let expected = fs::read_to_string(&golden_path)
        .unwrap_or_else(|e| panic!("Failed to read golden file at {}: {e}", golden_path.display()))
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        actual, expected,
        "Browser WASM contract changed; TypeScript signatures do not match golden file.\n\
         If this is an intentional change, update contracts/browser/browser_function_signatures.golden.txt"
    );
}

#[test]
fn browser_type_signature_count_matches() {
    let dts_path = repo_root()
        .join("example")
        .join("antikythera-web")
        .join("src")
        .join("shared")
        .join("wasm")
        .join("pkg")
        .join("antikythera_wasm_bindgen.d.ts");

    let dts_content = fs::read_to_string(&dts_path).unwrap();
    let signatures = extract_browser_function_signatures(&dts_content);

    // The browser WASM must export exactly these 17 functions.
    // If a function is added or removed, this test will fail.
    assert_eq!(
        signatures.len(),
        17,
        "Expected 17 exported functions in browser WASM, found {}: {:?}",
        signatures.len(),
        signatures
    );
}
