use super::Zerv;
use crate::cli::version::args::VersionArgs;
use crate::error::ZervError;

pub mod vars_primary;
pub mod vars_secondary;
pub mod vars_timestamp;

impl Zerv {
    /// Apply bump operations from VersionArgs
    pub fn apply_bumps(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        if let Some(Some(increment)) = args.bump_major {
            self.bump_major(increment as u64)?;
        }

        if let Some(Some(increment)) = args.bump_minor {
            self.bump_minor(increment as u64)?;
        }

        if let Some(Some(increment)) = args.bump_patch {
            self.bump_patch(increment as u64)?;
        }

        if let Some(Some(increment)) = args.bump_distance {
            self.bump_distance(increment as u64)?;
        }

        if let Some(Some(increment)) = args.bump_post {
            self.bump_post(increment as u64)?;
        }

        if let Some(Some(increment)) = args.bump_dev {
            self.bump_dev(increment as u64)?;
        }

        if let Some(Some(increment)) = args.bump_pre_release_num {
            self.bump_pre_release(increment as u64)?;
        }

        if let Some(Some(increment)) = args.bump_epoch {
            self.bump_epoch(increment as u64)?;
        }

        // Update bumped_timestamp based on dirty state
        self.bump_bumped_timestamp()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{BumpType, VersionArgsFixture, ZervFixture};
    use rstest::*;

    // Test apply_bumps method - only test the main coordinator method
    #[rstest]
    #[case((1, 0, 0), vec![(BumpType::Major, 1), (BumpType::Minor, 2)], (Some(2), Some(2), Some(0)))]
    #[case((2, 5, 3), vec![(BumpType::Major, 0), (BumpType::Patch, 7)], (Some(2), Some(5), Some(10)))]
    #[case((0, 0, 0), vec![(BumpType::Major, 3), (BumpType::Minor, 2), (BumpType::Patch, 1)], (Some(3), Some(2), Some(1)))]
    fn test_apply_bumps_method(
        #[case] version: (u64, u64, u64),
        #[case] bumps: Vec<(BumpType, u64)>,
        #[case] expected: (Option<u64>, Option<u64>, Option<u64>),
    ) {
        let mut zerv = ZervFixture::zerv_version(version.0, version.1, version.2);
        let args = VersionArgsFixture::with_bump_specs(bumps);

        zerv.apply_bumps(&args).unwrap();

        assert_eq!(zerv.vars.major, expected.0);
        assert_eq!(zerv.vars.minor, expected.1);
        assert_eq!(zerv.vars.patch, expected.2);
    }

    // Test apply_bumps with distance, post, and dev fields
    #[test]
    fn test_apply_bumps_distance_post_dev() {
        let mut zerv = ZervFixture::zerv_version(1, 0, 0);
        let bumps = vec![
            (BumpType::Distance, 5),
            (BumpType::Post, 3),
            (BumpType::Dev, 2),
        ];
        let args = VersionArgsFixture::with_bump_specs(bumps);

        zerv.apply_bumps(&args).unwrap();

        assert_eq!(zerv.vars.distance, Some(5));
        assert_eq!(zerv.vars.post, Some(3));
        assert_eq!(zerv.vars.dev, Some(2));
    }

    // Test apply_bumps with epoch and distance fields
    #[test]
    fn test_apply_bumps_epoch_distance() {
        let mut zerv = ZervFixture::zerv_version(2, 1, 0);
        let bumps = vec![(BumpType::Epoch, 1), (BumpType::Distance, 4)];
        let args = VersionArgsFixture::with_bump_specs(bumps);

        zerv.apply_bumps(&args).unwrap();

        assert_eq!(zerv.vars.epoch, Some(1));
        assert_eq!(zerv.vars.distance, Some(4));
        assert_eq!(zerv.vars.post, None); // Not bumped
    }

    // Test apply_bumps with zero increments
    #[test]
    fn test_apply_bumps_zero_increments() {
        let mut zerv = ZervFixture::zerv_version(0, 0, 0);
        let bumps = vec![
            (BumpType::Distance, 0),
            (BumpType::Post, 0),
            (BumpType::Dev, 0),
        ];
        let args = VersionArgsFixture::with_bump_specs(bumps);

        zerv.apply_bumps(&args).unwrap();

        assert_eq!(zerv.vars.distance, Some(0));
        assert_eq!(zerv.vars.post, Some(0));
        assert_eq!(zerv.vars.dev, Some(0));
    }

    // Test apply_bumps with complex combinations including timestamps
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

        // Set up some initial secondary fields
        zerv.vars.post = Some(5);
        zerv.vars.dev = Some(10);
        zerv.vars.epoch = Some(2);

        // Apply multiple bumps
        let bumps = vec![
            (BumpType::Major, 1),
            (BumpType::Minor, 2),
            (BumpType::Distance, 3),
            (BumpType::Post, 2),
            (BumpType::Dev, 5),
            (BumpType::Epoch, 1),
        ];
        let args = VersionArgsFixture::with_bump_specs(bumps);
        zerv.apply_bumps(&args).unwrap();

        // Verify primary fields
        assert_eq!(zerv.vars.major, Some(2)); // 1 + 1
        assert_eq!(zerv.vars.minor, Some(2)); // 0 + 2
        assert_eq!(zerv.vars.patch, Some(0)); // unchanged

        // Verify secondary fields
        assert_eq!(zerv.vars.distance, Some(6)); // 3 + 3
        assert_eq!(zerv.vars.post, Some(7)); // 5 + 2
        assert_eq!(zerv.vars.dev, Some(15)); // 10 + 5
        assert_eq!(zerv.vars.epoch, Some(3)); // 2 + 1

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
        let bumps = vec![(BumpType::Patch, 1), (BumpType::Distance, 1)];
        let args = VersionArgsFixture::with_bump_specs(bumps);
        zerv.apply_bumps(&args).unwrap();

        // Verify fields were bumped
        assert_eq!(zerv.vars.patch, Some(1)); // 0 + 1
        assert_eq!(zerv.vars.distance, Some(3)); // 2 + 1

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
        use crate::version::zerv::core::PreReleaseLabel;

        // Create a Zerv with pre-release
        let mut zerv = ZervFixture::zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(3));

        // Apply pre-release bump
        let bumps = vec![(BumpType::PreRelease, 2)];
        let args = VersionArgsFixture::with_bump_specs(bumps);
        zerv.apply_bumps(&args).unwrap();

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
        let bumps = vec![(BumpType::PreRelease, 1)];
        let args = VersionArgsFixture::with_bump_specs(bumps);
        let result = zerv.apply_bumps(&args);

        // Should fail because there's no pre-release to bump
        assert!(result.is_err());
    }

    // Test BumpType enum functionality
    #[test]
    fn test_bump_type_field_names() {
        assert_eq!(BumpType::Major.field_name(), "major");
        assert_eq!(BumpType::Minor.field_name(), "minor");
        assert_eq!(BumpType::Patch.field_name(), "patch");
        assert_eq!(BumpType::Distance.field_name(), "distance");
        assert_eq!(BumpType::Post.field_name(), "post");
        assert_eq!(BumpType::Dev.field_name(), "dev");
        assert_eq!(BumpType::Epoch.field_name(), "epoch");
        assert_eq!(BumpType::PreRelease.field_name(), "pre_release");
    }
}
