//! Documentation tests for the config-file feature (Phase 8).
//!
//! Backs the README "Config File" section — each runnable example there maps to
//! an assertion here. stdin source carries a known version so no git data is
//! read — no Docker gating (mirrors the `config_file/mod.rs` wiring tests).

use super::{
    repo_with_config,
    stdin_version,
    stdin_version_on_branch,
};
use crate::util::{
    TestCommand,
    null_device_path,
};

#[test]
fn test_config_file_documentation_examples() {
    let dir = repo_with_config(
        r#"source = "stdin"
output_template = "v{{ major }}.{{ minor }}.{{ patch }}""#,
    );

    let out = TestCommand::new()
        .current_dir(dir.path())
        .stdin(stdin_version(1, 2, 3))
        .args_from_str("version")
        .assert_success();
    assert_eq!(
        out.stdout().trim(),
        "v1.2.3",
        "a committed zerv.toml must shape version output"
    );

    let out = TestCommand::new()
        .current_dir(dir.path())
        .stdin(stdin_version_on_branch(1, 2, 3, "main"))
        .args_from_str("flow --schema standard")
        .assert_success();
    assert_eq!(
        out.stdout().trim(),
        "v1.2.3",
        "shared-top config must reach the flow subcommand"
    );

    let out = TestCommand::new()
        .current_dir(dir.path())
        .stdin(stdin_version(1, 2, 3))
        .args_from_str(r#"version --output-template "release-{{ major }}""#)
        .assert_success();
    assert_eq!(
        out.stdout().trim(),
        "release-1",
        "a CLI flag must win over the file's value"
    );
}

#[test]
fn test_config_file_overrides_documentation_examples() {
    let discovered = repo_with_config(
        r#"source = "stdin"
output_template = "DISC{{ major }}""#,
    );
    let explicit = discovered.path().join("explicit.toml");
    std::fs::write(
        &explicit,
        r#"source = "stdin"
output_template = "EXPL{{ major }}""#,
    )
    .expect("write explicit config");

    let out = TestCommand::new()
        .current_dir(discovered.path())
        .stdin(stdin_version(1, 2, 3))
        .arg("--config-file")
        .arg(&explicit)
        .args_from_str("version")
        .assert_success();
    assert_eq!(
        out.stdout().trim(),
        "EXPL1",
        "--config-file must override discovery: the discovered value must not apply"
    );

    let out = TestCommand::new()
        .current_dir(discovered.path())
        .stdin(stdin_version(1, 2, 3))
        .arg("--config-file")
        .arg(null_device_path())
        .args_from_str("version --source stdin")
        .assert_success();
    assert_eq!(
        out.stdout().trim(),
        "1.2.3",
        "--config-file <null device> must disable the discovered config"
    );

    let bad = repo_with_config("bump_minor = true");
    let out = TestCommand::new()
        .current_dir(bad.path())
        .args_from_str("version")
        .assert_failure();
    let stderr = out.stderr();
    assert!(
        stderr.contains("unknown field"),
        "an unknown/ephemeral key must be rejected, not silently ignored. Got: {stderr}"
    );
    assert!(
        stderr.contains("bump_minor"),
        "the offending key must be named in the error. Got: {stderr}"
    );
}
