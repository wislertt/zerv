pub mod dir;
pub mod git;
pub mod vcs_fixtures;

pub use dir::TestDir;
pub use git::DockerGit;
pub use vcs_fixtures::{get_real_pep440_vcs_data, get_real_semver_vcs_data};

/// Check if we should use native Git (CI only) or Docker (local development)
pub fn should_use_native_git() -> bool {
    std::env::var("CI").is_ok()
}
