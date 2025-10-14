pub mod dir;
pub mod git;
pub mod output;
pub mod types;
pub mod vcs_fixtures;
pub mod version;
pub mod version_args;
pub mod zerv;

pub use dir::TestDir;
pub use git::{
    DockerGit,
    GitOperations,
    GitRepoFixture,
    NativeGit,
};
pub use output::TestOutput;
pub use types::{
    BumpType,
    OverrideType,
};
pub use vcs_fixtures::{
    get_real_pep440_vcs_data,
    get_real_semver_vcs_data,
};
pub use version::VersionTestUtils;
pub use version_args::VersionArgsFixture;
// Zerv fixtures
pub use zerv::{
    ZervFixture,
    ZervSchemaFixture,
    ZervVarsFixture,
};

use crate::config::ZervConfig;

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

/// Get appropriate Git implementation based on environment
pub fn get_git_impl() -> Box<dyn GitOperations> {
    if should_use_native_git() {
        Box::new(NativeGit::new())
    } else {
        Box::new(DockerGit::new())
    }
}
