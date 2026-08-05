//! Config-file discovery on the real Git path (Docker-gated). Complements the
//! stdin-based `config_file/mod.rs` wiring: a real Git repo carries a
//! discovered `zerv.toml`, and `zerv version --source git` resolves it — where a
//! discovery/precedence bug would ship a wrong version in CI. Each test
//! early-returns when [`should_run_docker_tests`] is false.

use zerv::test_utils::{
    GitRepoFixture,
    should_run_docker_tests,
};
use zerv::utils::constants::config_files::PRIMARY;

use crate::util::{
    TestCommand,
    null_device_path,
};

/// `v1.2.3` tag at HEAD, distance 0 — deterministic output for assertions.
fn clean_tagged_repo() -> GitRepoFixture {
    GitRepoFixture::tagged("v1.2.3").expect("Failed to create tagged git fixture")
}

fn plant_config(repo: &GitRepoFixture, toml: &str) {
    std::fs::write(repo.path().join(PRIMARY), toml).expect("plant zerv.toml at repo root");
}

#[test]
fn config_template_applies_to_git_derived_version() {
    if !should_run_docker_tests() {
        return;
    }
    let repo = clean_tagged_repo();
    plant_config(
        &repo,
        "output_template = \"v{{major}}.{{minor}}.{{patch}}\"\n",
    );

    let output = TestCommand::new()
        .current_dir(repo.path())
        .args_from_str("version --source git")
        .assert_success();

    assert_eq!(
        output.stdout().trim(),
        "v1.2.3",
        "zerv.toml at the repo root must shape the git-derived version"
    );
}

#[test]
fn config_devnull_disables_for_git_source() {
    if !should_run_docker_tests() {
        return;
    }
    let repo = clean_tagged_repo();
    plant_config(&repo, "output_template = \"DISABLED{{major}}\"\n");

    let output = TestCommand::new()
        .current_dir(repo.path())
        .arg("--config-file")
        .arg(null_device_path())
        .args_from_str("version --source git")
        .assert_success();

    // Git default output carries non-deterministic build context — assert the
    // template didn't apply, not an exact string.
    let result = output.stdout().trim().to_string();
    assert!(
        !result.contains("DISABLED"),
        "--config-file <null device> must disable the discovered config on the git path. Got: {result}"
    );
    assert!(
        result.starts_with("1.2.3"),
        "default git behavior must be restored. Got: {result}"
    );
}

#[test]
fn cli_template_overrides_config_for_git_source() {
    if !should_run_docker_tests() {
        return;
    }
    let repo = clean_tagged_repo();
    plant_config(&repo, "output_template = \"FILE{{major}}\"\n");

    let output = TestCommand::new()
        .current_dir(repo.path())
        .args_from_str(r#"version --source git --output-template "CLI{{major}}""#)
        .assert_success();

    assert_eq!(
        output.stdout().trim(),
        "CLI1",
        "CLI --output-template must win over the discovered file on the git path"
    );
}
