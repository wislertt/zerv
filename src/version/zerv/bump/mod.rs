use super::Zerv;
use crate::cli::version::args::VersionArgs;
use crate::error::ZervError;

pub mod reset;
pub mod types;
pub mod vars_primary;
pub mod vars_secondary;
pub mod vars_timestamp;

impl Zerv {
    /// Apply component processing from VersionArgs using new process methods
    pub fn apply_component_processing(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // Process components in precedence order (highest to lowest)
        // This ensures reset logic works correctly

        // Epoch (highest precedence)
        self.process_epoch(args)?;

        // Major
        self.process_major(args)?;

        // Minor
        self.process_minor(args)?;

        // Patch
        self.process_patch(args)?;

        // Pre-release (label first, then number)
        self.process_pre_release_label(args)?;
        self.process_pre_release_num(args)?;

        // Post
        self.process_post(args)?;

        // Dev (lowest precedence)
        self.process_dev(args)?;

        // Update bumped_timestamp based on dirty state and context
        self.process_bumped_timestamp(args)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{VersionArgsFixture, ZervFixture};
    use crate::version::zerv::bump::types::BumpType;
    use crate::version::zerv::core::PreReleaseLabel;
    use rstest::*;

    // Test apply_component_processing method with reset logic
    #[rstest]
    #[case((1, 0, 0), vec![BumpType::Major(1), BumpType::Minor(2)], (Some(2), Some(2), Some(0)))]
    #[case((2, 5, 3), vec![BumpType::Patch(7)], (Some(2), Some(5), Some(10)))]
    #[case((0, 0, 0), vec![BumpType::Major(3), BumpType::Minor(2), BumpType::Patch(1)], (Some(3), Some(2), Some(1)))]
    fn test_apply_bumps_method(
        #[case] version: (u64, u64, u64),
        #[case] bumps: Vec<BumpType>,
        #[case] expected: (Option<u64>, Option<u64>, Option<u64>),
    ) {
        let mut zerv = ZervFixture::zerv_version(version.0, version.1, version.2);
        let args = VersionArgsFixture::new().with_bump_specs(bumps).build();

        zerv.apply_component_processing(&args).unwrap();

        assert_eq!(zerv.vars.major, expected.0);
        assert_eq!(zerv.vars.minor, expected.1);
        assert_eq!(zerv.vars.patch, expected.2);
    }

    // Test apply_bumps with zero increments
    #[test]
    fn test_apply_bumps_zero_increments() {
        let mut zerv = ZervFixture::zerv_version(0, 0, 0);
        let bumps = vec![BumpType::Post(0), BumpType::Dev(0)];
        let args = VersionArgsFixture::new().with_bump_specs(bumps).build();

        zerv.apply_component_processing(&args).unwrap();

        assert_eq!(zerv.vars.post, Some(0));
        assert_eq!(zerv.vars.dev, Some(0));
    }

    // Test apply_component_processing with complex combinations following reset logic
    #[test]
    fn test_apply_bumps_complex_combination() {
        // Create a Zerv with VCS data and secondary fields
        let mut zerv = ZervFixture::with_vcs_data(
            3,    // distance
            true, // dirty
            "feature/test".to_string(),
            "abc123def456".to_string(),
            "last123hash".to_string(),
            1234567890, // last_timestamp
            "main".to_string(),
        )
        .zerv()
        .clone();

        // Set up some initial version fields (these should be reset by epoch bump)
        zerv.vars.major = Some(1);
        zerv.vars.minor = Some(2);
        zerv.vars.patch = Some(3);
        zerv.vars.post = Some(5);
        zerv.vars.dev = Some(10);
        zerv.vars.epoch = Some(2);

        // Apply multiple bumps with reset logic
        // Processing order: Epoch → Major → Minor → Patch → Pre-release → Post → Dev
        let bumps = vec![
            BumpType::Epoch(1), // Resets ALL lower precedence components
            BumpType::Major(1), // Applied to reset value (0)
            BumpType::Minor(2), // Applied to reset value (0)
            BumpType::Post(2),  // Applied to reset value (0)
            BumpType::Dev(5),   // Applied to reset value (0)
        ];
        let args = VersionArgsFixture::new().with_bump_specs(bumps).build();
        zerv.apply_component_processing(&args).unwrap();

        // Verify fields follow reset logic:
        // 1. Epoch bump (2 + 1 = 3) resets all lower precedence to 0
        // 2. Then explicit bumps are applied from 0
        assert_eq!(zerv.vars.epoch, Some(3)); // 2 + 1 (epoch bump)
        assert_eq!(zerv.vars.major, Some(1)); // 0 + 1 (reset by epoch, then bumped)
        assert_eq!(zerv.vars.minor, Some(2)); // 0 + 2 (reset by epoch, then bumped)
        assert_eq!(zerv.vars.patch, Some(0)); // 0 (reset by epoch, no explicit bump)
        assert_eq!(zerv.vars.post, Some(2)); // 0 + 2 (reset by epoch, then bumped)
        assert_eq!(zerv.vars.dev, Some(5)); // 0 + 5 (reset by epoch, then bumped)

        // Verify VCS fields (should be preserved from original)
        assert_eq!(zerv.vars.dirty, Some(true));
        assert_eq!(zerv.vars.bumped_branch, Some("feature/test".to_string()));
        assert_eq!(
            zerv.vars.bumped_commit_hash,
            Some("abc123def456".to_string())
        );
        assert_eq!(zerv.vars.last_commit_hash, Some("last123hash".to_string()));
        assert_eq!(zerv.vars.last_timestamp, Some(1234567890));
        assert_eq!(zerv.vars.last_branch, Some("main".to_string()));

        // Verify timestamp was updated due to dirty=true
        assert!(zerv.vars.bumped_timestamp.is_some());
        assert!(zerv.vars.bumped_timestamp.unwrap() > 1234567890);
    }

    // Test semantic versioning reset behavior (major/minor bumps)
    #[test]
    fn test_apply_bumps_semantic_versioning_reset() {
        // Start with version 1.2.3 with some post/dev components
        let mut zerv = ZervFixture::zerv_version(1, 2, 3);
        zerv.vars.post = Some(5);
        zerv.vars.dev = Some(10);

        // Test major bump resets minor, patch, post, dev
        let bumps = vec![
            BumpType::Major(1), // Should reset minor, patch, post, dev
            BumpType::Minor(2), // Applied to reset value (0)
            BumpType::Patch(3), // Applied to reset value (0)
        ];
        let args = VersionArgsFixture::new().with_bump_specs(bumps).build();
        zerv.apply_component_processing(&args).unwrap();

        // Verify semantic versioning reset logic:
        // Major bump (1 + 1 = 2) resets minor, patch, post, dev to 0
        // Then explicit bumps are applied from 0
        assert_eq!(zerv.vars.major, Some(2)); // 1 + 1 (major bump)
        assert_eq!(zerv.vars.minor, Some(2)); // 0 + 2 (reset by major, then bumped)
        assert_eq!(zerv.vars.patch, Some(3)); // 0 + 3 (reset by major, then bumped)
        assert_eq!(zerv.vars.post, None); // Reset by major bump, no explicit bump
        assert_eq!(zerv.vars.dev, None); // Reset by major bump, no explicit bump
    }

    // Test minor bump reset behavior
    #[test]
    fn test_apply_bumps_minor_reset() {
        // Start with version 1.2.3 with some post/dev components
        let mut zerv = ZervFixture::zerv_version(1, 2, 3);
        zerv.vars.post = Some(5);
        zerv.vars.dev = Some(10);

        // Test minor bump resets patch, post, dev (but not major)
        let bumps = vec![
            BumpType::Minor(1), // Should reset patch, post, dev
            BumpType::Patch(2), // Applied to reset value (0)
        ];
        let args = VersionArgsFixture::new().with_bump_specs(bumps).build();
        zerv.apply_component_processing(&args).unwrap();

        // Verify minor bump reset logic:
        // Minor bump (2 + 1 = 3) resets patch, post, dev to 0
        // Major is preserved, explicit patch bump is applied
        assert_eq!(zerv.vars.major, Some(1)); // Preserved (higher precedence)
        assert_eq!(zerv.vars.minor, Some(3)); // 2 + 1 (minor bump)
        assert_eq!(zerv.vars.patch, Some(2)); // 0 + 2 (reset by minor, then bumped)
        assert_eq!(zerv.vars.post, None); // Reset by minor bump, no explicit bump
        assert_eq!(zerv.vars.dev, None); // Reset by minor bump, no explicit bump
    }

    // Test apply_bumps with clean state (no timestamp update)
    #[test]
    fn test_apply_bumps_clean_state() {
        // Create a Zerv with clean VCS data
        let mut zerv = ZervFixture::with_vcs_data(
            2,     // distance
            false, // clean
            "main".to_string(),
            "def456ghi789".to_string(),
            "last456hash".to_string(),
            9876543210, // last_timestamp
            "main".to_string(),
        )
        .zerv()
        .clone();

        // Set an initial bumped_timestamp
        let old_timestamp = 1234567890;
        zerv.vars.bumped_timestamp = Some(old_timestamp);

        // Apply bumps
        let bumps = vec![BumpType::Patch(1)];
        let args = VersionArgsFixture::new().with_bump_specs(bumps).build();
        zerv.apply_component_processing(&args).unwrap();

        // Verify fields were bumped
        assert_eq!(zerv.vars.patch, Some(1)); // 0 + 1

        // Verify VCS fields (should be preserved from original)
        assert_eq!(zerv.vars.dirty, Some(false)); // clean
        assert_eq!(zerv.vars.bumped_branch, Some("main".to_string()));
        assert_eq!(
            zerv.vars.bumped_commit_hash,
            Some("def456ghi789".to_string())
        );
        assert_eq!(zerv.vars.last_commit_hash, Some("last456hash".to_string()));
        assert_eq!(zerv.vars.last_timestamp, Some(9876543210));
        assert_eq!(zerv.vars.last_branch, Some("main".to_string()));

        // Verify timestamp was NOT updated due to clean state
        assert_eq!(zerv.vars.bumped_timestamp, Some(old_timestamp));
    }

    // Test apply_bumps with pre-release bump
    #[test]
    fn test_apply_bumps_pre_release() {
        // Create a Zerv with pre-release
        let mut zerv = ZervFixture::zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(3));

        // Apply pre-release bump
        let bumps = vec![BumpType::PreReleaseNum(2)];
        let args = VersionArgsFixture::new().with_bump_specs(bumps).build();
        zerv.apply_component_processing(&args).unwrap();

        // Verify pre-release was bumped
        assert!(zerv.vars.pre_release.is_some());
        assert_eq!(
            zerv.vars.pre_release.as_ref().unwrap().label,
            PreReleaseLabel::Alpha
        );
        assert_eq!(zerv.vars.pre_release.as_ref().unwrap().number, Some(5)); // 3 + 2
    }

    // Test apply_bumps with no pre-release (should fail)
    #[test]
    fn test_apply_bumps_pre_release_no_pre_release() {
        // Create a Zerv without pre-release
        let mut zerv = ZervFixture::zerv_version(1, 0, 0);

        // Try to apply pre-release bump
        let bumps = vec![BumpType::PreReleaseNum(1)];
        let args = VersionArgsFixture::new().with_bump_specs(bumps).build();
        let result = zerv.apply_component_processing(&args);

        // Should fail because there's no pre-release to bump
        assert!(result.is_err());
    }

    // Tests for combined bump and override specifications
    mod combined_bump_override_tests {
        use super::*;
        use crate::test_utils::version_args::OverrideType;

        #[test]
        fn test_bump_and_override_basic_combination() {
            // Create a Zerv with initial version 1.2.3
            let mut zerv = ZervFixture::zerv_version(1, 2, 3);

            // Apply both bumps and overrides using chaining approach
            // According to spec: "Overrides take precedence over bumps when both specified"
            let bumps = vec![BumpType::Major(1), BumpType::Minor(2)];
            let overrides = vec![
                OverrideType::Major(2), // Override: absolute value 2
                OverrideType::Minor(3), // Override: absolute value 3
                OverrideType::Distance(5),
                OverrideType::Dirty(true),
                OverrideType::CurrentBranch("feature/test".to_string()),
            ];

            let args = VersionArgsFixture::new()
                .with_bump_specs(bumps)
                .with_override_specs(overrides)
                .build();

            // Apply context overrides first (VCS overrides like distance, dirty, branch)
            zerv.vars.apply_context_overrides(&args).unwrap();

            // Then apply component processing (version component overrides and bumps)
            zerv.apply_component_processing(&args).unwrap();

            // Verify processing order: Context → Override → Bump Logic (per spec)
            // Expected behavior: Override sets absolute value, then bump modifies it
            assert_eq!(zerv.vars.major, Some(3)); // Override(2) + Bump(1) = 3
            assert_eq!(zerv.vars.minor, Some(5)); // Override(3) + Bump(2) = 5
            assert_eq!(zerv.vars.patch, Some(0));
            assert_eq!(zerv.vars.distance, Some(5));
            assert_eq!(zerv.vars.dirty, Some(true));
            assert_eq!(zerv.vars.bumped_branch, Some("feature/test".to_string()));

            // Verify that args structure contains both bumps and overrides
            assert_eq!(args.bump_major, Some(Some(1)));
            assert_eq!(args.bump_minor, Some(Some(2)));
            assert_eq!(args.major, Some(2));
            assert_eq!(args.minor, Some(3));
            assert_eq!(args.distance, Some(5));
            assert!(args.dirty);
            assert_eq!(args.current_branch, Some("feature/test".to_string()));
        }
    }
}
