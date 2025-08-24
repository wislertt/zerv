pub mod dir;
pub mod git;
pub mod vcs_fixtures;

use crate::config::ZervConfig;

pub use dir::TestDir;
pub use git::{DockerGit, GitOperations, NativeGit};
pub use vcs_fixtures::{get_real_pep440_vcs_data, get_real_semver_vcs_data};

pub fn should_use_native_git() -> bool {
    ZervConfig::load()
        .map(|config| config.should_use_native_git())
        .unwrap_or(false)
}

pub fn should_run_docker_tests() -> bool {
    ZervConfig::load()
        .map(|config| config.should_run_docker_tests())
        .unwrap_or(false)
}
