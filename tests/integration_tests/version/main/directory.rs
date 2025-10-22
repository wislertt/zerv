use zerv::test_utils::{
    GitRepoFixture,
    TestDir,
    should_run_docker_tests,
};

use crate::util::TestCommand;

mod directory_git_integration {
    use super::*;

    #[test]
    fn test_directory_flag_with_subdirectory() {
        if !should_run_docker_tests() {
            return;
        }

        let git_repo =
            GitRepoFixture::tagged("v1.0.0").expect("Failed to create tagged Git repository");

        let parent_dir = git_repo
            .path()
            .parent()
            .expect("Git repo should have parent directory");

        let output = TestCommand::new()
            .current_dir(parent_dir)
            .args_from_str(format!(
                "version -C {} --source git --output-format semver",
                git_repo.path().file_name().unwrap().to_string_lossy()
            ))
            .assert_success();

        assert_eq!(
            output.stdout().trim(),
            "1.0.0",
            "Should detect version from Git repo in subdirectory using -C flag"
        );
    }

    #[test]
    fn test_directory_flag_relative_vs_absolute() {
        if !should_run_docker_tests() {
            return;
        }

        let git_repo =
            GitRepoFixture::tagged("v1.0.0").expect("Failed to create tagged Git repository");

        let relative_output = TestCommand::new()
            .current_dir(git_repo.path().parent().unwrap())
            .args_from_str(format!(
                "version -C {} --source git --output-format semver",
                git_repo.path().file_name().unwrap().to_string_lossy()
            ))
            .assert_success();

        let absolute_output = TestCommand::new()
            .args_from_str(format!(
                "version -C {} --source git --output-format semver",
                git_repo.path().display()
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
