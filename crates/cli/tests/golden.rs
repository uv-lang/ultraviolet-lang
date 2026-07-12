use assert_cmd::Command;
use pretty_assertions::assert_eq;
use std::error::Error;
use std::fs;
use std::path::Path;
use ultraviolet_core::errors::CommonError;

#[test]
fn golden_tests() -> Result<(), Box<dyn Error>> {
    let cases_dir = Path::new("../../tests/cases").canonicalize()?;

    for entry in fs::read_dir(cases_dir).unwrap() {
        let path = entry.unwrap().path();

        if path.extension().and_then(|s| s.to_str()) != Some("uv") {
            continue;
        }

        let expected_path = path.with_extension("out");

        let expected = fs::read_to_string(&expected_path)
            .map_err(|_| CommonError::new("Failed to read expected output"))?;

        let output = Command::cargo_bin("ultraviolet-cli")?.arg(&path).output()?;
        let actual = if !output.stdout.is_empty() {
            String::from_utf8(output.stdout).unwrap()
        } else {
            String::from_utf8(output.stderr).unwrap()
        };

        assert_eq!(
            actual,
            expected,
            "Test `{}` failed",
            path.components()
                .last()
                .unwrap()
                .as_os_str()
                .to_string_lossy()
        );
    }

    Ok(())
}
