use super::Zerv;
use crate::error::ZervError;

impl Zerv {
    /// Bump bumped_timestamp based on dirty state
    /// Context control has already been applied, so we just respect the dirty state
    pub fn bump_bumped_timestamp(&mut self) -> Result<(), ZervError> {
        // bumped_timestamp should represent the timestamp of the current commit
        // If dirty, use current timestamp (uncommitted changes)
        // If clean, use the VCS commit timestamp (already set from VCS data)
        if self.vars.dirty == Some(true) {
            self.vars.bumped_timestamp = Some(chrono::Utc::now().timestamp() as u64);
        }
        // If dirty is false (either naturally or forced by --no-bump-context),
        // keep the existing timestamp (from VCS data)

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::zerv::ZervFixture;

    #[test]
    fn test_bumped_timestamp_dirty_with_bump_context() {
        let mut zerv = ZervFixture::zerv_version(1, 0, 0);
        zerv.vars.dirty = Some(true);

        // Set an old timestamp (1 hour ago)
        let old_timestamp = chrono::Utc::now().timestamp() as u64 - 3600;
        zerv.vars.bumped_timestamp = Some(old_timestamp);

        zerv.bump_bumped_timestamp().unwrap();

        // Should update to current timestamp when dirty (uncommitted changes)
        assert!(zerv.vars.bumped_timestamp.is_some());
        assert!(zerv.vars.bumped_timestamp.unwrap() > old_timestamp);
    }

    #[test]
    fn test_bumped_timestamp_clean_with_bump_context() {
        let mut zerv = ZervFixture::zerv_version(1, 0, 0);
        zerv.vars.dirty = Some(false);

        // Set a VCS commit timestamp (2 hours ago)
        let vcs_timestamp = chrono::Utc::now().timestamp() as u64 - 7200;
        zerv.vars.bumped_timestamp = Some(vcs_timestamp);

        zerv.bump_bumped_timestamp().unwrap();

        // Should keep VCS commit timestamp when clean (represents current commit)
        assert_eq!(zerv.vars.bumped_timestamp, Some(vcs_timestamp));
    }

    #[test]
    fn test_bumped_timestamp_no_bump_context() {
        let mut zerv = ZervFixture::zerv_version(1, 0, 0);
        // With --no-bump-context, dirty should be false (set by context control)
        zerv.vars.dirty = Some(false);

        // Set an old timestamp (30 minutes ago)
        let old_timestamp = chrono::Utc::now().timestamp() as u64 - 1800;
        zerv.vars.bumped_timestamp = Some(old_timestamp);

        zerv.bump_bumped_timestamp().unwrap();

        // Should keep existing timestamp when dirty=false (context control forces clean state)
        assert_eq!(zerv.vars.bumped_timestamp, Some(old_timestamp));
    }

    #[test]
    fn test_bumped_timestamp_clean_no_bump_context() {
        let mut zerv = ZervFixture::zerv_version(1, 0, 0);
        zerv.vars.dirty = Some(false);

        // Set a VCS timestamp (1 day ago)
        let vcs_timestamp = chrono::Utc::now().timestamp() as u64 - 86400;
        zerv.vars.bumped_timestamp = Some(vcs_timestamp);

        zerv.bump_bumped_timestamp().unwrap();

        // Should keep existing timestamp when bump context is disabled
        assert_eq!(zerv.vars.bumped_timestamp, Some(vcs_timestamp));
    }
}
