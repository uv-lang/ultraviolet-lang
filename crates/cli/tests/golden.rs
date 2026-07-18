use assert_cmd::Command;
use colored::Colorize;
use pretty_assertions::assert_eq;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

fn normalize(s: &str) -> String {
    s.replace("\r\n", "\n")
}

struct TestError {
    pub actual: String,
    pub expected: String,
    pub test_name: String,
}

fn get_expected(path: &PathBuf, ext: &str) -> String {
    let expected_path = path.with_extension(ext);
    fs::read_to_string(&expected_path).unwrap_or(String::from("Expected output file not found"))
}

#[test]
fn golden_tests() -> Result<(), Box<dyn Error>> {
    let cases_dir = Path::new("../../tests/cases").canonicalize()?;

    let mut latest_err: Option<TestError> = None;
    let mut succeed_count = 0;
    let mut failed_count = 0;
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

        let output = Command::cargo_bin("ultraviolet-cli")?.arg(&path).output()?;
        let should_panic = test_name.starts_with("err");

        let (succeed, actual, expected) = if !output.stdout.is_empty() {
            let stdout = normalize(&String::from_utf8(output.stdout).unwrap());
            let expected = normalize(&get_expected(&path, "out"));

            if !should_panic {
                (stdout == expected, stdout, expected)
            } else {
                (false, stdout, expected)
            }
        } else {
            let stderr = normalize(&String::from_utf8(output.stderr).unwrap());
            let expected = normalize(&get_expected(&path, "out"));

            if should_panic {
                (stderr.contains(&expected), stderr, expected)
            } else {
                (false, stderr, expected)
            }
        };

        if !succeed {
            println!("{}", "ERROR".red());
            failed_count += 1;
            latest_err = Some(TestError {
                actual: normalize(&actual),
                expected: normalize(&expected),
                test_name: test_name.to_string(),
            })
        } else {
            succeed_count += 1;
            println!("{}", "ok".green());
        }
    }

    println!(
        "Golden tests finished: {} succeed, {} failed",
        succeed_count, failed_count
    );

    if let Some(err) = latest_err {
        assert_eq!(err.actual, err.expected, "Test `{}` failed", err.test_name);
    }

    Ok(())
}
