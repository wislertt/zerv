use std::sync::OnceLock;

use super::git::{
    DockerGit,
    NativeGit,
};
use super::{
    GitOperations,
    TestDir,
    should_use_native_git,
};
use crate::vcs::git::GitVcs;
use crate::vcs::{
    Vcs,
    VcsData,
};

static SEMVER_VCS_DATA: OnceLock<VcsData> = OnceLock::new();
static PEP440_VCS_DATA: OnceLock<VcsData> = OnceLock::new();

fn get_git_impl() -> Box<dyn GitOperations> {
    if should_use_native_git() {
        Box::new(NativeGit::new())
    } else {
        Box::new(DockerGit::new())
    }
}

fn create_vcs_data_with_tag(tag: &str, filename: &str, content: &str, commit_msg: &str) -> VcsData {
    let test_dir = TestDir::new().expect("Failed to create test dir");
    let git = get_git_impl();

    git.init_repo(&test_dir).expect("Failed to init repo");
    git.create_tag(&test_dir, tag)
        .expect("Failed to create tag");

    // Add file and commit
    test_dir
        .create_file(filename, content)
        .expect("Failed to create file");
    git.create_commit(&test_dir, commit_msg)
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
