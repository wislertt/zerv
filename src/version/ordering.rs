use super::{Stage, Version};

impl Default for Version {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        // Compare epoch first (if present)
        match (self.epoch, other.epoch) {
            (Some(a), Some(b)) => {
                let epoch_cmp = a.cmp(&b);
                if epoch_cmp != Ordering::Equal {
                    return epoch_cmp;
                }
            }
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            (None, None) => {}
        }

        // Compare base version (major.minor.patch)
        let base_cmp =
            (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch));
        if base_cmp != Ordering::Equal {
            return base_cmp;
        }

        // Compare pre-release stage (None > Some for release vs pre-release)
        match (&self.stage, &other.stage) {
            (None, Some(_)) => return Ordering::Greater, // Release > pre-release
            (Some(_), None) => return Ordering::Less,    // Pre-release < release
            (Some(a), Some(b)) => {
                let stage_cmp = a.cmp(b);
                if stage_cmp != Ordering::Equal {
                    return stage_cmp;
                }
                // Same stage, compare revision
                let rev_cmp = self.revision.cmp(&other.revision);
                if rev_cmp != Ordering::Equal {
                    return rev_cmp;
                }
            }
            (None, None) => {}
        }

        // Compare post-release (higher post = newer)
        let post_cmp = self.post.cmp(&other.post);
        if post_cmp != Ordering::Equal {
            return post_cmp;
        }

        // Compare dev release (None > Some for release vs dev)
        match (self.dev, other.dev) {
            (None, Some(_)) => Ordering::Greater, // Release > dev
            (Some(_), None) => Ordering::Less,    // Dev < release
            (Some(a), Some(b)) => a.cmp(&b),      // Compare dev numbers
            (None, None) => Ordering::Equal,
        }
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl Eq for Version {}

impl PartialEq for Stage {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Stage::Alpha, Stage::Alpha) | (Stage::Beta, Stage::Beta) | (Stage::Rc, Stage::Rc)
        )
    }
}

impl Eq for Stage {}

impl PartialOrd for Stage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Stage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match (self, other) {
            (Stage::Alpha, Stage::Alpha) => Ordering::Equal,
            (Stage::Alpha, _) => Ordering::Less,
            (Stage::Beta, Stage::Alpha) => Ordering::Greater,
            (Stage::Beta, Stage::Beta) => Ordering::Equal,
            (Stage::Beta, Stage::Rc) => Ordering::Less,
            (Stage::Rc, Stage::Rc) => Ordering::Equal,
            (Stage::Rc, _) => Ordering::Greater,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_version_default() {
        let v = Version::default();
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_version_equality() {
        let v1 = Version::new(1, 2, 3);
        let v2 = Version::new(1, 2, 3);
        let v3 = Version::new(1, 2, 4);

        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[rstest]
    #[case(Stage::Alpha, Stage::Alpha, true)]
    #[case(Stage::Beta, Stage::Beta, true)]
    #[case(Stage::Rc, Stage::Rc, true)]
    #[case(Stage::Alpha, Stage::Beta, false)]
    fn test_stage_equality(#[case] left: Stage, #[case] right: Stage, #[case] expected: bool) {
        assert_eq!(left == right, expected);
    }

    #[rstest]
    #[case(Stage::Alpha, Stage::Beta, true)]
    #[case(Stage::Beta, Stage::Rc, true)]
    #[case(Stage::Alpha, Stage::Rc, true)]
    #[case(Stage::Beta, Stage::Alpha, false)]
    fn test_stage_ordering(#[case] left: Stage, #[case] right: Stage, #[case] expected_lt: bool) {
        assert_eq!(left < right, expected_lt);
    }

    #[rstest]
    // Epoch comparison
    #[case(Version::new(1, 0, 0), Version { epoch: Some(1), ..Version::new(1, 0, 0) }, true)]
    #[case(Version { epoch: Some(2), ..Version::new(1, 0, 0) }, Version { epoch: Some(1), ..Version::new(1, 0, 0) }, false)]
    // Base version comparison
    #[case(Version::new(1, 0, 0), Version::new(2, 0, 0), true)]
    #[case(Version::new(1, 2, 0), Version::new(1, 3, 0), true)]
    #[case(Version::new(1, 2, 3), Version::new(1, 2, 4), true)]
    // Pre-release vs release
    #[case(Version::new(1, 2, 3).with_stage(Stage::Alpha, Some(1)), Version::new(1, 2, 3), true)]
    #[case(Version::new(1, 2, 3).with_stage(Stage::Beta, Some(1)), Version::new(1, 2, 3), true)]
    #[case(Version::new(1, 2, 3).with_stage(Stage::Rc, Some(1)), Version::new(1, 2, 3), true)]
    // Stage comparison
    #[case(Version::new(1, 2, 3).with_stage(Stage::Alpha, Some(1)), Version::new(1, 2, 3).with_stage(Stage::Beta, Some(1)), true)]
    #[case(Version::new(1, 2, 3).with_stage(Stage::Beta, Some(1)), Version::new(1, 2, 3).with_stage(Stage::Rc, Some(1)), true)]
    // Revision comparison
    #[case(Version::new(1, 2, 3).with_stage(Stage::Alpha, Some(1)), Version::new(1, 2, 3).with_stage(Stage::Alpha, Some(2)), true)]
    // Post-release comparison
    #[case(Version::new(1, 2, 3).with_post(1), Version::new(1, 2, 3).with_post(2), true)]
    #[case(Version::new(1, 2, 3), Version::new(1, 2, 3).with_post(1), true)]
    // Dev release comparison
    #[case(Version::new(1, 2, 3).with_dev(1), Version::new(1, 2, 3), true)]
    #[case(Version::new(1, 2, 3).with_dev(1), Version::new(1, 2, 3).with_dev(2), true)]
    fn test_version_comparison(
        #[case] left: Version,
        #[case] right: Version,
        #[case] expected_lt: bool,
    ) {
        assert_eq!(left < right, expected_lt);
        assert_eq!(right > left, expected_lt);
        assert_eq!(left <= right, expected_lt);
        assert_eq!(right >= left, expected_lt);
    }

    #[test]
    #[allow(clippy::nonminimal_bool)]
    fn test_version_equal_comparison() {
        let v1 = Version::new(1, 2, 3);
        let v2 = Version::new(1, 2, 3);

        assert!(v1 <= v2);
        assert!(v1 >= v2);
        assert!(!(v1 < v2));
        assert!(!(v1 > v2));
    }

    #[test]
    #[allow(clippy::nonminimal_bool)]
    fn test_version_strict_inequality() {
        let v1 = Version::new(1, 2, 3);
        let v2 = Version::new(1, 2, 4);

        assert!(v1 < v2); // v1 is less than v2
        assert!(!(v2 < v1)); // v2 is not less than v1
        assert!(v2 > v1); // v2 is greater than v1
        assert!(!(v1 > v2)); // v1 is not greater than v2
    }

    #[test]
    fn test_complex_version_comparison() {
        let v1 = Version {
            major: 1,
            minor: 2,
            patch: 3,
            stage: Some(Stage::Alpha),
            revision: Some(1),
            post: Some(1),
            dev: Some(1),
            epoch: Some(1),
            ..Version::new(1, 2, 3)
        };

        let v2 = Version {
            major: 1,
            minor: 2,
            patch: 3,
            stage: Some(Stage::Alpha),
            revision: Some(1),
            post: Some(1),
            dev: Some(1),
            epoch: Some(1),
            ..Version::new(1, 2, 3)
        };

        assert_eq!(v1, v2);
        assert!(v1 <= v2);
        assert!(v1 >= v2);
    }
}
