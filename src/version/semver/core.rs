#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreReleaseIdentifier {
    Str(String),
    UInt(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildMetadata {
    Str(String),
    UInt(u64),
}

#[derive(Debug, Clone)]
pub struct SemVer {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre_release: Option<Vec<PreReleaseIdentifier>>,
    pub build_metadata: Option<Vec<BuildMetadata>>,
}

impl SemVer {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build_metadata: None,
        }
    }

    pub fn with_pre_release(mut self, pre_release: Vec<PreReleaseIdentifier>) -> Self {
        self.pre_release = Some(pre_release);
        self
    }

    pub fn with_build_metadata(mut self, build_metadata: Vec<BuildMetadata>) -> Self {
        self.build_metadata = Some(build_metadata);
        self
    }

    pub fn is_pre_release(&self) -> bool {
        self.pre_release.is_some()
    }

    pub fn is_stable(&self) -> bool {
        !self.is_pre_release()
    }
}

impl Default for SemVer {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    mod construction {
        use super::*;

        #[test]
        fn test_new() {
            let version = SemVer::new(1, 2, 3);
            assert_eq!(version.major, 1);
            assert_eq!(version.minor, 2);
            assert_eq!(version.patch, 3);
            assert!(version.pre_release.is_none());
            assert!(version.build_metadata.is_none());
        }

        #[test]
        fn test_default() {
            let version = SemVer::default();
            assert_eq!(version.major, 0);
            assert_eq!(version.minor, 0);
            assert_eq!(version.patch, 0);
            assert!(version.pre_release.is_none());
            assert!(version.build_metadata.is_none());
        }

        #[test]
        fn test_with_pre_release() {
            let pre_release = vec![
                PreReleaseIdentifier::Str("alpha".to_string()),
                PreReleaseIdentifier::UInt(1),
            ];
            let version = SemVer::new(1, 0, 0).with_pre_release(pre_release.clone());
            assert_eq!(version.pre_release, Some(pre_release));
        }

        #[test]
        fn test_with_build_metadata() {
            let build_metadata = vec![
                BuildMetadata::Str("build".to_string()),
                BuildMetadata::UInt(123),
            ];
            let version = SemVer::new(1, 0, 0).with_build_metadata(build_metadata.clone());
            assert_eq!(version.build_metadata, Some(build_metadata));
        }

        #[test]
        fn test_method_chaining() {
            let pre_release = vec![PreReleaseIdentifier::Str("alpha".to_string())];
            let build_metadata = vec![BuildMetadata::Str("build".to_string())];

            let version = SemVer::new(1, 2, 3)
                .with_pre_release(pre_release.clone())
                .with_build_metadata(build_metadata.clone());

            assert_eq!(version.major, 1);
            assert_eq!(version.minor, 2);
            assert_eq!(version.patch, 3);
            assert_eq!(version.pre_release, Some(pre_release));
            assert_eq!(version.build_metadata, Some(build_metadata));
        }
    }

    mod properties {
        use super::*;

        #[test]
        fn test_is_stable() {
            let stable = SemVer::new(1, 0, 0);
            assert!(stable.is_stable());
            assert!(!stable.is_pre_release());
        }

        #[test]
        fn test_is_pre_release() {
            let pre_release = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::Str("alpha".to_string())]);
            assert!(pre_release.is_pre_release());
            assert!(!pre_release.is_stable());
        }

        #[test]
        fn test_build_metadata_does_not_affect_stability() {
            let version = SemVer::new(1, 0, 0)
                .with_build_metadata(vec![BuildMetadata::Str("build".to_string())]);
            assert!(version.is_stable());
            assert!(!version.is_pre_release());
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_zero_version() {
            let version = SemVer::new(0, 0, 0);
            assert_eq!(version.major, 0);
            assert_eq!(version.minor, 0);
            assert_eq!(version.patch, 0);
        }

        #[test]
        fn test_max_values() {
            let version = SemVer::new(u64::MAX, u64::MAX, u64::MAX);
            assert_eq!(version.major, u64::MAX);
            assert_eq!(version.minor, u64::MAX);
            assert_eq!(version.patch, u64::MAX);
        }

        #[test]
        fn test_empty_pre_release() {
            let version = SemVer::new(1, 0, 0).with_pre_release(vec![]);
            assert_eq!(version.pre_release, Some(vec![]));
            assert!(version.is_pre_release());
        }

        #[test]
        fn test_empty_build_metadata() {
            let version = SemVer::new(1, 0, 0).with_build_metadata(vec![]);
            assert_eq!(version.build_metadata, Some(vec![]));
        }

        #[test]
        fn test_overwrite_pre_release() {
            let first = vec![PreReleaseIdentifier::Str("alpha".to_string())];
            let second = vec![PreReleaseIdentifier::Str("beta".to_string())];

            let version = SemVer::new(1, 0, 0)
                .with_pre_release(first)
                .with_pre_release(second.clone());

            assert_eq!(version.pre_release, Some(second));
        }

        #[test]
        fn test_overwrite_build_metadata() {
            let first = vec![BuildMetadata::Str("build1".to_string())];
            let second = vec![BuildMetadata::Str("build2".to_string())];

            let version = SemVer::new(1, 0, 0)
                .with_build_metadata(first)
                .with_build_metadata(second.clone());

            assert_eq!(version.build_metadata, Some(second));
        }
    }

    mod identifiers {
        use super::*;

        #[rstest]
        #[case("alpha")]
        #[case("beta")]
        #[case("rc")]
        #[case("x")]
        #[case("")]
        fn test_pre_release_string_identifier(#[case] value: &str) {
            let identifier = PreReleaseIdentifier::Str(value.to_string());
            match identifier {
                PreReleaseIdentifier::Str(s) => assert_eq!(s, value),
                _ => panic!("Expected string identifier"),
            }
        }

        #[rstest]
        #[case(0)]
        #[case(1)]
        #[case(123)]
        #[case(u64::MAX)]
        fn test_pre_release_integer_identifier(#[case] value: u64) {
            let identifier = PreReleaseIdentifier::UInt(value);
            match identifier {
                PreReleaseIdentifier::UInt(n) => assert_eq!(n, value),
                _ => panic!("Expected integer identifier"),
            }
        }

        #[rstest]
        #[case("build")]
        #[case("commit")]
        #[case("sha")]
        #[case("")]
        fn test_build_metadata_string(#[case] value: &str) {
            let metadata = BuildMetadata::Str(value.to_string());
            match metadata {
                BuildMetadata::Str(s) => assert_eq!(s, value),
                _ => panic!("Expected string metadata"),
            }
        }

        #[rstest]
        #[case(0)]
        #[case(1)]
        #[case(20240101)]
        #[case(u64::MAX)]
        fn test_build_metadata_integer(#[case] value: u64) {
            let metadata = BuildMetadata::UInt(value);
            match metadata {
                BuildMetadata::UInt(n) => assert_eq!(n, value),
                _ => panic!("Expected integer metadata"),
            }
        }
    }

    mod complex_versions {
        use super::*;

        #[test]
        fn test_complex_pre_release() {
            let pre_release = vec![
                PreReleaseIdentifier::Str("alpha".to_string()),
                PreReleaseIdentifier::UInt(1),
                PreReleaseIdentifier::Str("build".to_string()),
                PreReleaseIdentifier::UInt(456),
            ];

            let version = SemVer::new(2, 0, 0).with_pre_release(pre_release.clone());
            assert_eq!(version.pre_release, Some(pre_release));
        }

        #[test]
        fn test_complex_build_metadata() {
            let build_metadata = vec![
                BuildMetadata::Str("commit".to_string()),
                BuildMetadata::Str("abc123".to_string()),
                BuildMetadata::UInt(20240101),
            ];

            let version = SemVer::new(1, 5, 0).with_build_metadata(build_metadata.clone());
            assert_eq!(version.build_metadata, Some(build_metadata));
        }

        #[test]
        fn test_full_version() {
            let pre_release = vec![
                PreReleaseIdentifier::Str("rc".to_string()),
                PreReleaseIdentifier::UInt(2),
            ];
            let build_metadata = vec![
                BuildMetadata::Str("build".to_string()),
                BuildMetadata::UInt(789),
            ];

            let version = SemVer::new(3, 1, 4)
                .with_pre_release(pre_release.clone())
                .with_build_metadata(build_metadata.clone());

            assert_eq!(version.major, 3);
            assert_eq!(version.minor, 1);
            assert_eq!(version.patch, 4);
            assert_eq!(version.pre_release, Some(pre_release));
            assert_eq!(version.build_metadata, Some(build_metadata));
            assert!(version.is_pre_release());
            assert!(!version.is_stable());
        }
    }
}
