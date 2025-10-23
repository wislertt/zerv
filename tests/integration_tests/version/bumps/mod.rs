//! Bump configuration tests
//!
//! Tests for all BumpsConfig options including:
//! - Primary component bumps (--bump-major, --bump-minor, --bump-patch)
//! - Secondary component bumps (--bump-epoch, --bump-post, --bump-dev, --bump-pre-release-*)
//! - Schema component bumps (--bump-core, --bump-extra-core, --bump-build)
//! - Context bumping (--bump-context, --no-bump-context)
//! - Cross-category bump combinations

use rstest::fixture;
use zerv::test_utils::ZervFixture;
use zerv::version::PreReleaseLabel;

pub mod context;
pub mod primary;
pub mod schema;
pub mod secondary;
// Other modules will be enabled as they're fixed
// pub mod combinations;

/// Base Zerv fixture for bump tests with version 1.2.3-alpha.1
#[fixture]
fn base_zerv_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
}

/// Zerv fixture with distance and dirty data for context bumping tests
#[fixture]
fn zerv_with_vcs_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
        .with_distance(5)
        .with_dirty(true)
}
