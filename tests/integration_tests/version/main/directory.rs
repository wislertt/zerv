use std::sync::Mutex;

use serial_test::serial;
use zerv::test_utils::{
    GitRepoFixture,
    TestDir,
    should_run_docker_tests,
};

use crate::util::TestCommand;

static SHARED_FIXTURE_LOCK: Mutex<Option<(std::path::PathBuf, tempfile::TempDir)>> =
    Mutex::new(None);

fn get_or_create_shared_fixture() -> std::path::PathBuf {
    let mut guard = SHARED_FIXTURE_LOCK.lock().unwrap();

    if let Some((path, _)) = guard.as_ref() {
        return path.clone();
    }

    let fixture = GitRepoFixture::tagged("v1.0.0")
        .expect("Failed to create shared git fixture for directory tests");

    let path = fixture.path().to_path_buf();
    let temp_dir = fixture.test_dir.into_inner();

    *guard = Some((path.clone(), temp_dir));
    path
}

mod directory_git_integration {
    use super::*;

    #[test]
    #[serial(directory_shared_fixture)]
    fn test_directory_flag_with_subdirectory() {
        if !should_run_docker_tests() {
            return;
        }

        let git_repo_path = get_or_create_shared_fixture();

        let parent_dir = git_repo_path
            .parent()
            .expect("Git repo should have parent directory");

        let output = TestCommand::new()
            .current_dir(parent_dir)
            .args_from_str(format!(
                "version -C {} --source git --output-format semver",
                git_repo_path.file_name().unwrap().to_string_lossy()
            ))
            .assert_success();

        assert_eq!(
            output.stdout().trim(),
            "1.0.0",
            "Should detect version from Git repo in subdirectory using -C flag"
        );
    }

    #[test]
    #[serial(directory_shared_fixture)]
    fn test_directory_flag_relative_vs_absolute() {
        if !should_run_docker_tests() {
            return;
        }

        let git_repo_path = get_or_create_shared_fixture();

        let relative_output = TestCommand::new()
            .current_dir(git_repo_path.parent().unwrap())
            .args_from_str(format!(
                "version -C {} --source git --output-format semver",
                git_repo_path.file_name().unwrap().to_string_lossy()
            ))
            .assert_success();

        let absolute_output = TestCommand::new()
            .args_from_str(format!(
                "version -C {} --source git --output-format semver",
                git_repo_path.display()
            ))
            .assert_success();

        assert_eq!(
            relative_output.stdout().trim(),
            "1.0.0",
            "Relative path should work"
        );
        assert_eq!(
            absolute_output.stdout().trim(),
            "1.0.0",
            "Absolute path should work"
        );
        assert_eq!(
            relative_output.stdout(),
            absolute_output.stdout(),
            "Relative and absolute paths should produce identical output"
        );
    }
}

mod directory_error_handling {
    use super::*;

    #[test]
    fn test_directory_flag_nonexistent_path() {
        let output = TestCommand::new()
            .args_from_str("version -C /nonexistent/path/to/directory")
            .assert_failure();

        let stderr = output.stderr();
        assert!(
            stderr.contains("Error") && stderr.contains("VCS not found"),
            "Should show VCS not found error when directory doesn't exist. Got: {stderr}"
        );
    }

    #[test]
    fn test_directory_flag_exists_but_not_git() {
        let test_dir = TestDir::new().expect("Failed to create test directory");

        let output = TestCommand::new()
            .args_from_str(format!("version -C {}", test_dir.path().display()))
            .assert_failure();

        let stderr = output.stderr();
        assert!(
            stderr.contains("Error: VCS not found: Not in a git repository (--source git)"),
            "Should show proper error when directory exists but is not a git repo. Got: {stderr}"
        );
    }
}
