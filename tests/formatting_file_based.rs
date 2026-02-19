//! File-based formatting tests.
//!
//! This test module discovers test cases from `tests/formatting/cases/` directories.
//! Each test case is a directory containing:
//! - `input.sparql`: The unformatted SPARQL query
//! - `expected.sparql`: The expected formatted output
//! - `settings.toml` (optional): Custom FormatSettings to use
//!
//! To add a new test case, simply create a new directory with the required files.

use qlue_ls::FormatSettings;
use std::fs;
use std::path::{Path, PathBuf};

/// Discovers all test case directories under tests/formatting/cases/
fn discover_test_cases() -> Vec<PathBuf> {
    let cases_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("formatting")
        .join("cases");

    if !cases_dir.exists() {
        panic!("Test cases directory not found: {:?}", cases_dir);
    }

    let mut cases = Vec::new();
    for entry in fs::read_dir(&cases_dir).expect("Failed to read cases directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            let input_file = path.join("input.sparql");
            let expected_file = path.join("expected.sparql");
            if input_file.exists() && expected_file.exists() {
                cases.push(path);
            }
        }
    }

    cases.sort(); // Deterministic order
    cases
}

/// Runs a single test case
fn run_test_case(case_dir: &Path) {
    let case_name = case_dir
        .file_name()
        .expect("Case dir has no name")
        .to_string_lossy();

    let input_path = case_dir.join("input.sparql");
    let expected_path = case_dir.join("expected.sparql");
    let settings_path = case_dir.join("settings.toml");

    let input = fs::read_to_string(&input_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read input.sparql for case '{}': {}",
            case_name, e
        )
    });

    let expected = fs::read_to_string(&expected_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read expected.sparql for case '{}': {}",
            case_name, e
        )
    });

    // Read optional settings and parse them
    let format_settings: FormatSettings = if settings_path.exists() {
        let settings_toml = fs::read_to_string(&settings_path).unwrap_or_else(|e| {
            panic!(
                "Failed to read settings.toml for case '{}': {}",
                case_name, e
            )
        });
        toml::from_str(&settings_toml).unwrap_or_else(|e| {
            panic!(
                "Failed to parse settings.toml for case '{}': {}",
                case_name, e
            )
        })
    } else {
        FormatSettings::default()
    };

    let result = qlue_ls::format_with_settings(input.clone(), format_settings)
        .unwrap_or_else(|e| panic!("Formatting failed for case '{}': {}", case_name, e));

    pretty_assertions::assert_eq!(
        result,
        expected,
        "\n\nFormatting mismatch in test case: '{}'\n\nInput:\n{}\n",
        case_name,
        input
    );
}

#[test]
fn test_all_formatting_cases() {
    let mut cases = discover_test_cases();

    // Filter cases if FILTER env var is set
    if let Ok(filter) = std::env::var("FILTER") {
        cases.retain(|case_dir| {
            case_dir
                .file_name()
                .map(|name| name.to_string_lossy().contains(&filter))
                .unwrap_or(false)
        });
        println!("Filtering test cases with: '{}'", filter);
    }

    if cases.is_empty() {
        panic!("No test cases found (check tests/formatting/cases/ or FILTER env var)");
    }

    println!("Running {} formatting test cases", cases.len());

    let mut failures = Vec::new();

    for case_dir in &cases {
        let case_name = case_dir.file_name().unwrap().to_string_lossy();
        let result = std::panic::catch_unwind(|| {
            run_test_case(case_dir);
        });

        if let Err(e) = result {
            failures.push((case_name.to_string(), e));
        }
    }

    if !failures.is_empty() {
        let failure_names: Vec<_> = failures.iter().map(|(name, _)| name.as_str()).collect();
        panic!(
            "\n{} of {} formatting tests failed:\n  - {}\n",
            failures.len(),
            cases.len(),
            failure_names.join("\n  - ")
        );
    }
}
