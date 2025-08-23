use super::{DockerGit, TestDir};
use crate::vcs::{Vcs, VcsData, git::GitVcs};
use std::sync::OnceLock;

static SEMVER_VCS_DATA: OnceLock<VcsData> = OnceLock::new();
static PEP440_VCS_DATA: OnceLock<VcsData> = OnceLock::new();

/// Get real VCS data with SemVer tag (v1.2.3) and 1 commit distance
pub fn get_real_semver_vcs_data() -> &'static VcsData {
    SEMVER_VCS_DATA.get_or_init(|| {
        let test_dir = TestDir::new().expect("Failed to create test dir");
        let docker_git = DockerGit::new();

        docker_git
            .init_repo(&test_dir)
            .expect("Failed to init repo");
        docker_git
            .create_tag(&test_dir, "v1.2.3")
            .expect("Failed to create tag");

        test_dir
            .create_file("feature.txt", "new feature")
            .expect("Failed to create file");
        docker_git
            .create_commit(&test_dir, "Add feature")
            .expect("Failed to create commit");

        let git_vcs = GitVcs::new(test_dir.path()).expect("Failed to create GitVcs");
        git_vcs.get_vcs_data().expect("Failed to get VCS data")
    })
}

/// Get real VCS data with PEP440 tag (2.0.1a1) and 1 commit distance
pub fn get_real_pep440_vcs_data() -> &'static VcsData {
    PEP440_VCS_DATA.get_or_init(|| {
        let test_dir = TestDir::new().expect("Failed to create test dir");
        let docker_git = DockerGit::new();

        docker_git
            .init_repo(&test_dir)
            .expect("Failed to init repo");
        docker_git
            .create_tag(&test_dir, "2.0.1a1")
            .expect("Failed to create tag");

        test_dir
            .create_file("fix.txt", "bug fix")
            .expect("Failed to create file");
        docker_git
            .create_commit(&test_dir, "Fix bug")
            .expect("Failed to create commit");

        let git_vcs = GitVcs::new(test_dir.path()).expect("Failed to create GitVcs");
        git_vcs.get_vcs_data().expect("Failed to get VCS data")
    })
}
