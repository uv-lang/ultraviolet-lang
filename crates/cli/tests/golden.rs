use assert_cmd::Command;
use colored::Colorize;
use pretty_assertions::assert_eq;
use std::error::Error;
use std::fs;
use std::path::Path;
use ultraviolet_core::errors::CommonError;

fn normalize(s: &str) -> String {
    s.replace("\r\n", "\n")
}

struct TestError {
    pub actual: String,
    pub expected: String,
    pub test_name: String,
}

#[test]
fn golden_tests() -> Result<(), Box<dyn Error>> {
    let cases_dir = Path::new("../../tests/cases").canonicalize()?;

    let mut latest_err: Option<TestError> = None;
    for entry in fs::read_dir(cases_dir).unwrap() {
        let path = entry.unwrap().path();

        if path.extension().and_then(|s| s.to_str()) != Some("uv") {
            continue;
        }

        let test_name = path
            .components()
            .next_back()
            .unwrap()
            .as_os_str()
            .to_string_lossy();

        print!("test golden::{test_name} ... ");

        let expected_path = path.with_extension("out");
        let expected = fs::read_to_string(&expected_path)
            .map_err(|_| CommonError::new("Failed to read expected output"))?;

        let output = Command::cargo_bin("ultraviolet-cli")?.arg(&path).output()?;
        let actual = if !output.stdout.is_empty() {
            String::from_utf8(output.stdout).unwrap()
        } else {
            String::from_utf8(output.stderr).unwrap()
        };

        if normalize(&actual) != normalize(&expected) {
            println!("{}", "ERROR".red());
            latest_err = Some(TestError {
                actual: normalize(&actual),
                expected: normalize(&expected),
                test_name: test_name.to_string(),
            })
        } else {
            println!("{}", "ok".green());
        }
    }

    if let Some(err) = latest_err {
        assert_eq!(err.actual, err.expected, "Test `{}` failed", err.test_name);
    }

    Ok(())
}
