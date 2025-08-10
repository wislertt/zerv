use rstest::rstest;
use std::process::Command;

#[test]
fn test_default_output() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "zerv"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("1.2.3"));
    assert!(stdout.contains("Debug: Version"));
}

#[rstest]
#[case("-V")]
#[case("--version")]
fn test_version_flags(#[case] flag: &str) {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "zerv", "--", flag])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("zerv 0.0.0"));
}
