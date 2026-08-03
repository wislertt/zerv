//! End-to-end config-file wiring. The merge layer mutates arg structs before
//! each pipeline, so handlers read merged values by construction — one field
//! (`output_template`) proves the field-agnostic path for all (per-field
//! correctness lives in `merge.rs`). Each temp dir plants a `.git` marker to
//! bound discovery to the temp root; stdin source carries a known version, so
//! no git data is read (no Docker gate).

use zerv::test_utils::{
    TestDir,
    ZervFixture,
};

use crate::util::TestCommand;

/// Temp dir with a `.git` boundary marker (bounds discovery to the temp root).
fn repo_root() -> TestDir {
    let dir = TestDir::new().expect("Failed to create config test dir");
    dir.create_dir(".git").expect("plant .git boundary");
    dir
}

/// Temp dir with a `.git` marker and a config file named `name`.
fn repo_with_config_named(name: &str, toml: &str) -> TestDir {
    let dir = repo_root();
    dir.create_file(name, toml).expect("write config file");
    dir
}

/// Temp dir with `.git` and a `zerv.toml`.
fn repo_with_config(toml: &str) -> TestDir {
    repo_with_config_named("zerv.toml", toml)
}

/// A stdin Zerv RON payload for a known `major.minor.patch`.
fn stdin_version(major: u64, minor: u64, patch: u64) -> String {
    ZervFixture::new()
        .with_version(major, minor, patch)
        .build()
        .to_string()
}

/// A stdin Zerv RON payload for a known `major.minor.patch` on `branch`. Flow
/// schemas shape output by branch, so the flow end-to-end tests carry one.
fn stdin_version_on_branch(major: u64, minor: u64, patch: u64, branch: &str) -> String {
    ZervFixture::new()
        .with_version(major, minor, patch)
        .with_branch(branch.to_string())
        .build()
        .to_string()
}

#[test]
fn file_output_template_reaches_version_output() {
    let dir = repo_with_config(
        r#"source = "stdin"
output_template = "v{{major}}.{{minor}}""#,
    );
    let output = TestCommand::new()
        .current_dir(dir.path())
        .stdin(stdin_version(1, 2, 3))
        .args_from_str("version")
        .assert_success();

    assert_eq!(
        output.stdout().trim(),
        "v1.2",
        "output_template from zerv.toml must reach the rendered version"
    );
}

#[test]
fn cli_output_template_overrides_file() {
    let dir = repo_with_config(
        r#"source = "stdin"
output_template = "v{{major}}.{{minor}}""#,
    );
    let output = TestCommand::new()
        .current_dir(dir.path())
        .stdin(stdin_version(1, 2, 3))
        .args_from_str(r#"version --output-template "CLI{{major}}""#)
        .assert_success();

    assert_eq!(
        output.stdout().trim(),
        "CLI1",
        "CLI --output-template must win over the file's template"
    );
}

#[test]
fn no_config_file_leaves_behavior_unchanged() {
    let dir = repo_root();

    let output = TestCommand::new()
        .current_dir(dir.path())
        .stdin(stdin_version(1, 2, 3))
        .args_from_str("version --source stdin")
        .assert_success();

    assert_eq!(
        output.stdout().trim(),
        "1.2.3",
        "no zerv.toml must leave default behavior untouched"
    );
}

#[test]
fn discovery_walks_up_from_subdir_to_root_config() {
    let dir = repo_with_config(
        r#"source = "stdin"
output_template = "v{{major}}.{{minor}}""#,
    );
    dir.create_dir("sub").expect("create subdir");

    let output = TestCommand::new()
        .current_dir(dir.path().join("sub"))
        .stdin(stdin_version(4, 5, 6))
        .args_from_str("version")
        .assert_success();

    assert_eq!(
        output.stdout().trim(),
        "v4.5",
        "zerv run from a subdir must discover the repo-root zerv.toml by walking up"
    );
}

#[test]
fn fallback_hidden_config_is_discovered() {
    let dir = repo_with_config_named(
        ".zerv.toml",
        r#"source = "stdin"
output_template = "hid{{major}}""#,
    );
    let output = TestCommand::new()
        .current_dir(dir.path())
        .stdin(stdin_version(7, 0, 0))
        .args_from_str("version")
        .assert_success();

    assert_eq!(
        output.stdout().trim(),
        "hid7",
        ".zerv.toml (hidden fallback) must be discovered when zerv.toml is absent"
    );
}

#[test]
fn malformed_config_errors_loud() {
    let dir = repo_with_config("not = valid = toml");
    let output = TestCommand::new()
        .current_dir(dir.path())
        .args_from_str("version")
        .assert_failure();

    let stderr = output.stderr();
    assert!(
        stderr.contains("zerv.toml"),
        "malformed config must fail loud naming the file, not silently fall back. Got: {stderr}"
    );
}

#[test]
fn unknown_field_errors_loud() {
    // A real ephemeral CLI flag in the file must be rejected (deny_unknown_fields)
    // — ephemeral overrides can't sneak in via config.
    let dir = repo_with_config("bump_minor = true");
    let output = TestCommand::new()
        .current_dir(dir.path())
        .args_from_str("version")
        .assert_failure();

    let stderr = output.stderr();
    assert!(
        stderr.contains("unknown field"),
        "an unknown/typo'd key must be rejected, not silently ignored. Got: {stderr}"
    );
    assert!(
        stderr.contains("bump_minor"),
        "the offending key must be named in the error. Got: {stderr}"
    );
}

#[test]
fn config_file_flag_overrides_discovery() {
    let dir = repo_with_config(
        r#"source = "stdin"
output_template = "DISC{{major}}""#,
    );
    let explicit = dir.path().join("explicit.toml");
    std::fs::write(
        &explicit,
        r#"source = "stdin"
output_template = "EXPL{{major}}""#,
    )
    .expect("write explicit config");

    let output = TestCommand::new()
        .current_dir(dir.path())
        .stdin(stdin_version(1, 2, 3))
        .arg("--config-file")
        .arg(&explicit)
        .args_from_str("version")
        .assert_success();

    assert_eq!(
        output.stdout().trim(),
        "EXPL1",
        "--config-file must override discovery: the discovered template must not apply"
    );
}

#[test]
fn config_file_devnull_disables_discovered_config() {
    let dir = repo_with_config(
        r#"source = "stdin"
output_template = "DISABLED{{major}}""#,
    );

    let output = TestCommand::new()
        .current_dir(dir.path())
        .stdin(stdin_version(1, 2, 3))
        .arg("--config-file")
        .arg("/dev/null")
        .args_from_str("version --source stdin")
        .assert_success();

    assert_eq!(
        output.stdout().trim(),
        "1.2.3",
        "--config-file /dev/null must disable the discovered config: the template must not apply"
    );
}

#[test]
fn cli_flag_overrides_config_file_value() {
    let dir = repo_root();
    let explicit = dir.path().join("explicit.toml");
    std::fs::write(
        &explicit,
        r#"source = "stdin"
output_template = "FILE{{major}}""#,
    )
    .expect("write explicit config");

    let output = TestCommand::new()
        .current_dir(dir.path())
        .stdin(stdin_version(1, 2, 3))
        .arg("--config-file")
        .arg(&explicit)
        .args_from_str(r#"version --output-template "CLI{{major}}""#)
        .assert_success();

    assert_eq!(
        output.stdout().trim(),
        "CLI1",
        "CLI --output-template must win over the --config-file template"
    );
}

#[test]
fn config_file_missing_path_errors_loud() {
    let dir = repo_root();

    let output = TestCommand::new()
        .current_dir(dir.path())
        .arg("--config-file")
        .arg(dir.path().join("does-not-exist.toml"))
        .args_from_str("version")
        .assert_failure();

    let stderr = output.stderr();
    assert!(
        stderr.contains("--config-file"),
        "a missing --config-file path must error naming the flag. Got: {stderr}"
    );
}

#[test]
fn config_file_flag_works_after_subcommand() {
    // Guards --config-file as a global arg (parses after the subcommand;
    // dropping global=true would silently break it).
    let dir = repo_with_config(
        r#"source = "stdin"
output_template = "DISC{{major}}""#,
    );
    let explicit = dir.path().join("explicit.toml");
    std::fs::write(
        &explicit,
        r#"source = "stdin"
output_template = "EXPL{{major}}""#,
    )
    .expect("write explicit config");

    let output = TestCommand::new()
        .current_dir(dir.path())
        .stdin(stdin_version(1, 2, 3))
        .args_from_str("version --config-file")
        .arg(&explicit)
        .assert_success();

    assert_eq!(
        output.stdout().trim(),
        "EXPL1",
        "--config-file must work after the subcommand and override discovery"
    );
}

#[test]
fn flow_output_template_from_file_reaches_output() {
    let dir = repo_with_config(
        r#"source = "stdin"
output_template = "f{{major}}""#,
    );

    let output = TestCommand::new()
        .current_dir(dir.path())
        .stdin(stdin_version_on_branch(1, 2, 3, "main"))
        .args_from_str("flow --schema standard")
        .assert_success();

    assert_eq!(
        output.stdout().trim(),
        "f1",
        "output_template from zerv.toml must reach the rendered flow output"
    );
}

#[test]
fn flow_cli_output_template_overrides_file() {
    let dir = repo_with_config(
        r#"source = "stdin"
output_template = "f{{major}}""#,
    );

    let output = TestCommand::new()
        .current_dir(dir.path())
        .stdin(stdin_version_on_branch(1, 2, 3, "main"))
        .args_from_str(r#"flow --schema standard --output-template "CLI{{major}}""#)
        .assert_success();

    assert_eq!(
        output.stdout().trim(),
        "CLI1",
        "CLI --output-template must win over the file's template on flow"
    );
}

mod docs;
mod git_source;
mod precedence;
