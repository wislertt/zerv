use super::{DockerGit, TestDir, should_use_native_git};
use crate::vcs::{Vcs, VcsData, git::GitVcs};
use std::fs;
use std::process::Command;
use std::sync::OnceLock;

static SEMVER_VCS_DATA: OnceLock<VcsData> = OnceLock::new();
static PEP440_VCS_DATA: OnceLock<VcsData> = OnceLock::new();

fn create_vcs_data_with_tag(tag: &str, filename: &str, content: &str, commit_msg: &str) -> VcsData {
    let test_dir = TestDir::new().expect("Failed to create test dir");

    if should_use_native_git() {
        create_vcs_data_with_native_git(&test_dir, tag, filename, content, commit_msg)
    } else {
        create_vcs_data_with_docker(&test_dir, tag, filename, content, commit_msg)
    }
}

fn create_vcs_data_with_native_git(
    test_dir: &TestDir,
    tag: &str,
    filename: &str,
    content: &str,
    commit_msg: &str,
) -> VcsData {
    let path = test_dir.path();

    // Create initial file
    fs::write(path.join("README.md"), "# Test Repository").expect("Failed to create README");

    // Use isolated Git config
    let git_cmd = |args: &[&str]| {
        Command::new("git")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .args(args)
            .current_dir(path)
            .output()
            .expect("should execute git command")
    };

    // Initialize repo and create initial commit
    let commands = [
        &["init"][..],
        &["config", "user.name", "Test User"],
        &["config", "user.email", "test@example.com"],
        &["add", "."],
        &["commit", "-m", "Initial commit"],
        &["tag", tag],
    ];

    for args in commands {
        let output = git_cmd(args);
        assert!(
            output.status.success(),
            "Git command '{:?}' failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Create additional file and commit
    test_dir
        .create_file(filename, content)
        .expect("Failed to create file");

    let output = git_cmd(&["add", "."]);
    assert!(
        output.status.success(),
        "Git add failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = git_cmd(&["commit", "-m", commit_msg]);
    assert!(
        output.status.success(),
        "Git commit failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let git_vcs = GitVcs::new(test_dir.path()).expect("Failed to create GitVcs");
    git_vcs.get_vcs_data().expect("Failed to get VCS data")
}

fn create_vcs_data_with_docker(
    test_dir: &TestDir,
    tag: &str,
    filename: &str,
    content: &str,
    commit_msg: &str,
) -> VcsData {
    let docker_git = DockerGit::new();

    docker_git.init_repo(test_dir).expect("Failed to init repo");
    docker_git
        .create_tag(test_dir, tag)
        .expect("Failed to create tag");

    test_dir
        .create_file(filename, content)
        .expect("Failed to create file");
    docker_git
        .create_commit(test_dir, commit_msg)
        .expect("Failed to create commit");

    let git_vcs = GitVcs::new(test_dir.path()).expect("Failed to create GitVcs");
    git_vcs.get_vcs_data().expect("Failed to get VCS data")
}

/// Get real VCS data with SemVer tag (v1.2.3) and 1 commit distance
pub fn get_real_semver_vcs_data() -> &'static VcsData {
    SEMVER_VCS_DATA.get_or_init(|| {
        create_vcs_data_with_tag("v1.2.3", "feature.txt", "new feature", "Add feature")
    })
}

/// Get real VCS data with PEP440 tag (2.0.1a1) and 1 commit distance
pub fn get_real_pep440_vcs_data() -> &'static VcsData {
    PEP440_VCS_DATA
        .get_or_init(|| create_vcs_data_with_tag("2.0.1a1", "fix.txt", "bug fix", "Fix bug"))
}
