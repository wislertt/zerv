//! Cross-category bump combination tests
//!
//! Tests for complex bump scenarios involving multiple categories:
//! - Primary + Secondary bump combinations
//! - Primary + Schema bump combinations
//! - Secondary + Schema bump combinations
//! - All category combinations
//! - Context behavior with complex combinations

use rstest::{
    fixture,
    rstest,
};
use zerv::schema::ZervSchemaPreset;
use zerv::test_utils::ZervFixture;
use zerv::utils::constants::formats::SEMVER;
use zerv::utils::constants::sources::STDIN;
use zerv::version::zerv::PreReleaseLabel;
use zerv::version::zerv::components::Component;

use crate::util::TestCommand;

/// Base fixture for combination tests
#[fixture]
fn combination_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
        .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease)
}

/// Complete fixture with all components for complex combinations
#[fixture]
fn full_combination_fixture() -> ZervFixture {
    combination_fixture()
        .with_epoch(1)
        .with_extra_core(Component::UInt(0))
        .with_extra_core(Component::UInt(1))
        .with_build(Component::UInt(2))
        .with_distance(5)
        .with_dirty(true)
}

mod primary_secondary_combinations {
    use super::*;

    #[rstest]
    #[case("2.1.1-alpha.1")] // major + minor + patch + prerelease num
    fn test_primary_secondary_simple(combination_fixture: ZervFixture, #[case] expected: &str) {
        let input = combination_fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-major --bump-minor --bump-patch --bump-pre-release-num --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.1.0-beta.0")] // primary bumps + label change
    #[case("2.1.0-alpha.1")] // primary bumps + prerelease num
    #[case("1.1.0-epoch.1.alpha.1")] // epoch + primary + prerelease num
    fn test_primary_secondary_variants(combination_fixture: ZervFixture, #[case] expected: &str) {
        let input = combination_fixture.build().to_string();
        let args = match expected {
            s if s.contains("beta") => format!(
                "version --source {} --bump-major --bump-minor --bump-pre-release-label beta --output-format {}",
                STDIN, SEMVER
            ),
            s if s.contains("epoch") => format!(
                "version --source {} --bump-epoch --bump-major --bump-minor --bump-pre-release-num --output-format {}",
                STDIN, SEMVER
            ),
            s if s.contains("2.1") => format!(
                "version --source {} --bump-major --bump-minor --bump-pre-release-num --output-format {}",
                STDIN, SEMVER
            ),
            _ => unreachable!("Unexpected expected value pattern"),
        };

        let output = TestCommand::run_with_stdin(&args, input);
        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("4.4.0-alpha.1", "3", "4", "1")] // Custom values for primary + prerelease num
    fn test_primary_secondary_custom_values(
        combination_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] major_value: &str,
        #[case] minor_value: &str,
        #[case] prerelease_value: &str,
    ) {
        let input = combination_fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-major {} --bump-minor {} --bump-pre-release-num {} --output-format {}",
            STDIN, major_value, minor_value, prerelease_value, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.1.1-alpha.1.post.1")] // All secondary with primary (Dev requires tier 3 schema)
    fn test_primary_all_secondary_combination(
        combination_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        // Use tier 2 schema (doesn't include Dev component)
        let fixture = combination_fixture
            .with_schema_preset(ZervSchemaPreset::StandardBasePrereleasePostContext);

        let input = fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-major --bump-minor --bump-patch --bump-pre-release-num --bump-post --bump-dev --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }
}

mod primary_schema_combinations {
    use super::*;

    #[rstest]
    #[case("0.0.0-epoch.1.alpha.1.0.0+1")] // primary + extra-core + build
    fn test_primary_schema_simple(#[case] expected: &str) {
        // Create fixture with extra-core and build components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease)
            .with_extra_core(Component::UInt(0))
            .with_extra_core(Component::UInt(0))
            .with_build(Component::UInt(0));

        let input = fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-major --bump-minor --bump-patch --bump-extra-core 0 --bump-extra-core 1 --bump-build 0 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("3.0.0")] // major + core bump
    #[case("1.4.0")] // minor + core bump
    #[case("1.2.5")] // patch + core bump
    fn test_primary_core_combinations(combination_fixture: ZervFixture, #[case] expected: &str) {
        let input = combination_fixture.build().to_string();
        let args = match expected {
            s if s.starts_with("3.") => format!(
                "version --source {} --bump-major --bump-core 0 --output-format {}",
                STDIN, SEMVER
            ),
            s if s.starts_with("1.4") => format!(
                "version --source {} --bump-minor --bump-core 1 --output-format {}",
                STDIN, SEMVER
            ),
            s if s.starts_with("1.2.5") => format!(
                "version --source {} --bump-patch --bump-core 2 --output-format {}",
                STDIN, SEMVER
            ),
            _ => unreachable!("Unexpected expected value pattern"),
        };

        let output = TestCommand::run_with_stdin(&args, input);
        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("0.0.0-epoch.2.alpha.3.0.0+4.5")] // All primary + schema with custom values
    fn test_primary_schema_custom_values(#[case] expected: &str) {
        // Create fixture with extra-core and build components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease)
            .with_extra_core(Component::UInt(0))
            .with_extra_core(Component::UInt(0))
            .with_build(Component::UInt(0))
            .with_build(Component::UInt(0));

        let input = fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-major 3 --bump-minor 4 --bump-patch 5 --bump-extra-core 0=2 --bump-extra-core 1=3 --bump-build 0=4 --bump-build 1=5 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("3.0.0")] // major + core bump preserves prerelease
    fn test_primary_schema_preserve_prerelease(
        combination_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = combination_fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-major --bump-core 0 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }
}

mod secondary_schema_combinations {
    use super::*;

    #[rstest]
    #[case("0.0.0-epoch.2.alpha.1.0.0+1")] // epoch + extra-core + build
    fn test_secondary_schema_simple(#[case] expected: &str) {
        // Create fixture with extra-core and build components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease)
            .with_extra_core(Component::UInt(0))
            .with_extra_core(Component::UInt(0))
            .with_build(Component::UInt(0));

        let input = fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-epoch --bump-extra-core 0 --bump-extra-core 1 --bump-build 0 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("0.0.0-epoch.1.0+0", "beta")] // label + extra-core
    #[case("0.0.0-epoch.1.0+0", "num")] // prerelease num + extra-core
    #[case("1.2.3-alpha.1.0+1", "dev")] // dev + build
    fn test_secondary_schema_variants(#[case] expected: &str, #[case] variant: &str) {
        // Create fixture with extra-core and build components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease)
            .with_extra_core(Component::UInt(0))
            .with_build(Component::UInt(0));

        let input = fixture.build().to_string();
        let args = match variant {
            "beta" => format!(
                "version --source {} --bump-pre-release-label beta --bump-extra-core 0 --output-format {}",
                STDIN, SEMVER
            ),
            "num" => format!(
                "version --source {} --bump-pre-release-num --bump-extra-core 0 --output-format {}",
                STDIN, SEMVER
            ),
            "dev" => format!(
                "version --source {} --bump-dev --bump-build 0 --output-format {}",
                STDIN, SEMVER
            ),
            _ => unreachable!("Unexpected variant pattern"),
        };

        let output = TestCommand::run_with_stdin(&args, input);
        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("0.0.0-epoch.4.alpha.3.0.0+4.5")] // All secondary + schema with custom values
    fn test_secondary_schema_complex(#[case] expected: &str) {
        // Create fixture with extra-core and build components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease)
            .with_extra_core(Component::UInt(0))
            .with_extra_core(Component::UInt(0))
            .with_build(Component::UInt(0))
            .with_build(Component::UInt(0));

        let input = fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-epoch 2 --bump-pre-release-label beta --bump-post --bump-dev --bump-extra-core 0=2 --bump-extra-core 1=3 --bump-build 0=4 --bump-build 1=5 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("1.2.3-alpha.1+2")] // Secondary bumps preserve schema structure
    fn test_secondary_schema_preserve_structure(#[case] expected: &str) {
        // Create fixture with build component
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease)
            .with_build(Component::UInt(1));

        let input = fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-dev --bump-build 0 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }
}

mod all_category_combinations {
    use super::*;

    #[rstest]
    #[case("0.0.0-epoch.1.0+1")] // primary + secondary + schema
    fn test_all_categories_simple(#[case] expected: &str) {
        // Create fixture with extra-core and build components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease)
            .with_extra_core(Component::UInt(0))
            .with_build(Component::UInt(0));

        let input = fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-major --bump-minor --bump-patch --bump-pre-release-label beta --bump-pre-release-num --bump-extra-core 0 --bump-build 0 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("0.0.0-epoch.5.alpha.3.0.0+4.5")] // Complex combination with custom values
    fn test_all_categories_complex(#[case] expected: &str) {
        // Create fixture with extra-core and build components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease)
            .with_extra_core(Component::UInt(0))
            .with_extra_core(Component::UInt(0))
            .with_build(Component::UInt(0))
            .with_build(Component::UInt(0));

        let input = fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-major 4 --bump-minor 5 --bump-patch 6 --bump-pre-release-label rc --bump-pre-release-num 2 --bump-epoch 3 --bump-extra-core 0=2 --bump-extra-core 1=3 --bump-build 0=4 --bump-build 1=5 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("0.0.0-epoch.1.0+1.5")] // All categories with VCS context
    fn test_all_categories_with_context(#[case] expected: &str) {
        // Create fixture with extra-core, build, and VCS components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease)
            .with_extra_core(Component::UInt(0))
            .with_build(Component::UInt(0))
            .with_distance(5)
            .with_dirty(true);

        let input = fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-major --bump-minor --bump-patch --bump-pre-release-label beta --bump-pre-release-num --bump-extra-core 0 --bump-build 0 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("0.0.0-epoch.1.0+1.0")] // All categories without VCS context
    fn test_all_categories_without_context(#[case] expected: &str) {
        // Create fixture with extra-core, build, and VCS components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease)
            .with_extra_core(Component::UInt(0))
            .with_build(Component::UInt(0))
            .with_distance(5)
            .with_dirty(true);

        let input = fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-major --bump-minor --bump-patch --bump-pre-release-label beta --bump-pre-release-num --bump-extra-core 0 --bump-build 0 --no-bump-context --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("0.0.0-epoch.8.alpha.3.0.1+5.5")] // Maximum complexity scenario
    fn test_maximum_complexity_combination(
        full_combination_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = full_combination_fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-major 5 --bump-minor 6 --bump-patch 7 --bump-pre-release-label rc --bump-pre-release-num 3 --bump-epoch 4 --bump-post 6 --bump-dev 7 --bump-core 0=5 --bump-core 1=6 --bump-core 2=7 --bump-extra-core 0=3 --bump-extra-core 1=3 --bump-build 0=3 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }
}

mod context_with_combinations {
    use super::*;

    #[rstest]
    #[case("2.1.1-beta.1+5", "2.1.1-beta.1+0")] // With vs without context
    fn test_context_impact_on_combinations(
        #[case] with_context: &str,
        #[case] without_context: &str,
    ) {
        // Create fixture with VCS data
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease)
            .with_distance(5)
            .with_dirty(true);

        let input = fixture.build().to_string();

        // Test with context (default)
        let args_with = format!(
            "version --source {} --bump-major --bump-minor --bump-patch --bump-pre-release-label beta --bump-pre-release-num --output-format {}",
            STDIN, SEMVER
        );
        let output_with = TestCommand::run_with_stdin(&args_with, input.clone());

        // Test without context
        let args_without = format!(
            "version --source {} --bump-major --bump-minor --bump-patch --bump-pre-release-label beta --bump-pre-release-num --no-bump-context --output-format {}",
            STDIN, SEMVER
        );
        let output_without = TestCommand::run_with_stdin(&args_without, input);

        assert_eq!(output_with.trim(), with_context);
        assert_eq!(output_without.trim(), without_context);
    }

    #[rstest]
    #[case("0.0.0-epoch.3.0.1+3.5")] // Context with complex combination
    fn test_context_with_schema_bumps(
        full_combination_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = full_combination_fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-epoch --bump-major --bump-patch --bump-pre-release-num --bump-core 1 --bump-extra-core 0 --bump-build 0 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("0.0.0-epoch.3.0.1+3.0")] // No context with complex combination
    fn test_no_context_with_schema_bumps(
        full_combination_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = full_combination_fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-epoch --bump-major --bump-patch --bump-pre-release-num --bump-core 1 --bump-extra-core 0 --bump-build 0 --no-bump-context --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.0.0")] // Context doesn't affect pure tag versions
    fn test_context_with_pure_tag_version(#[case] expected: &str) {
        // Create pure tag version (no VCS data)
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease);

        let input = fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-major --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }
}
