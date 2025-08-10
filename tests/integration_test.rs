use std::process::Command;
use zerv::{hello, run};

#[test]
fn test_hello_function() {
    assert_eq!(hello(), "Hello, world!");
}

#[test]
fn test_main_run() {
    run();
}

#[test]
fn test_main_binary() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "zerv"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Hello, world!"
    );
}
