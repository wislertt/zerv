use super::core::{BuildMetadata, PreReleaseIdentifier, SemVer};
use std::fmt;

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if let Some(ref pre_release) = self.pre_release
            && !pre_release.is_empty()
        {
            write!(f, "-{}", format_identifiers(pre_release))?;
        }

        if let Some(ref build_metadata) = self.build_metadata
            && !build_metadata.is_empty()
        {
            write!(f, "+{}", format_build_metadata(build_metadata))?;
        }

        Ok(())
    }
}

impl fmt::Display for PreReleaseIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PreReleaseIdentifier::String(s) => write!(f, "{s}"),
            PreReleaseIdentifier::Integer(n) => write!(f, "{n}"),
        }
    }
}

impl fmt::Display for BuildMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildMetadata::String(s) => write!(f, "{s}"),
            BuildMetadata::Integer(n) => write!(f, "{n}"),
        }
    }
}

fn format_identifiers(identifiers: &[PreReleaseIdentifier]) -> String {
    identifiers
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

fn format_build_metadata(metadata: &[BuildMetadata]) -> String {
    metadata
        .iter()
        .map(|meta| meta.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    mod basic_display {
        use super::*;

        #[test]
        fn test_simple_version() {
            let version = SemVer::new(1, 2, 3);
            assert_eq!(version.to_string(), "1.2.3");
        }

        #[test]
        fn test_zero_version() {
            let version = SemVer::new(0, 0, 0);
            assert_eq!(version.to_string(), "0.0.0");
        }

        #[test]
        fn test_large_numbers() {
            let version = SemVer::new(999, 888, 777);
            assert_eq!(version.to_string(), "999.888.777");
        }

        #[test]
        fn test_max_values() {
            let version = SemVer::new(u64::MAX, u64::MAX, u64::MAX);
            let expected = format!("{}.{}.{}", u64::MAX, u64::MAX, u64::MAX);
            assert_eq!(version.to_string(), expected);
        }
    }

    mod pre_release_display {
        use super::*;

        #[test]
        fn test_single_string_pre_release() {
            let version = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::String("alpha".to_string())]);
            assert_eq!(version.to_string(), "1.0.0-alpha");
        }

        #[test]
        fn test_single_integer_pre_release() {
            let version =
                SemVer::new(1, 0, 0).with_pre_release(vec![PreReleaseIdentifier::Integer(1)]);
            assert_eq!(version.to_string(), "1.0.0-1");
        }

        #[test]
        fn test_mixed_pre_release() {
            let version = SemVer::new(1, 0, 0).with_pre_release(vec![
                PreReleaseIdentifier::String("alpha".to_string()),
                PreReleaseIdentifier::Integer(1),
            ]);
            assert_eq!(version.to_string(), "1.0.0-alpha.1");
        }

        #[test]
        fn test_complex_pre_release() {
            let version = SemVer::new(2, 1, 0).with_pre_release(vec![
                PreReleaseIdentifier::String("rc".to_string()),
                PreReleaseIdentifier::Integer(2),
                PreReleaseIdentifier::String("build".to_string()),
                PreReleaseIdentifier::Integer(456),
            ]);
            assert_eq!(version.to_string(), "2.1.0-rc.2.build.456");
        }

        #[test]
        fn test_empty_pre_release() {
            let version = SemVer::new(1, 0, 0).with_pre_release(vec![]);
            assert_eq!(version.to_string(), "1.0.0");
        }

        #[rstest]
        #[case("alpha")]
        #[case("beta")]
        #[case("rc")]
        #[case("x")]
        #[case("")]
        #[case("0")]
        #[case("123abc")]
        fn test_various_string_pre_release(#[case] pre_release: &str) {
            let version = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::String(pre_release.to_string())]);
            assert_eq!(version.to_string(), format!("1.0.0-{pre_release}"));
        }

        #[rstest]
        #[case(0)]
        #[case(1)]
        #[case(999)]
        #[case(u64::MAX)]
        fn test_various_integer_pre_release(#[case] pre_release: u64) {
            let version = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::Integer(pre_release)]);
            assert_eq!(version.to_string(), format!("1.0.0-{pre_release}"));
        }
    }

    mod build_metadata_display {
        use super::*;

        #[test]
        fn test_single_string_build_metadata() {
            let version = SemVer::new(1, 0, 0)
                .with_build_metadata(vec![BuildMetadata::String("build".to_string())]);
            assert_eq!(version.to_string(), "1.0.0+build");
        }

        #[test]
        fn test_single_integer_build_metadata() {
            let version =
                SemVer::new(1, 0, 0).with_build_metadata(vec![BuildMetadata::Integer(123)]);
            assert_eq!(version.to_string(), "1.0.0+123");
        }

        #[test]
        fn test_mixed_build_metadata() {
            let version = SemVer::new(1, 0, 0).with_build_metadata(vec![
                BuildMetadata::String("commit".to_string()),
                BuildMetadata::String("abc123".to_string()),
            ]);
            assert_eq!(version.to_string(), "1.0.0+commit.abc123");
        }

        #[test]
        fn test_complex_build_metadata() {
            let version = SemVer::new(1, 5, 2).with_build_metadata(vec![
                BuildMetadata::String("build".to_string()),
                BuildMetadata::Integer(789),
                BuildMetadata::String("sha".to_string()),
                BuildMetadata::String("def456".to_string()),
            ]);
            assert_eq!(version.to_string(), "1.5.2+build.789.sha.def456");
        }

        #[test]
        fn test_empty_build_metadata() {
            let version = SemVer::new(1, 0, 0).with_build_metadata(vec![]);
            assert_eq!(version.to_string(), "1.0.0");
        }

        #[rstest]
        #[case("build")]
        #[case("commit")]
        #[case("sha")]
        #[case("")]
        #[case("abc123")]
        #[case("20240101")]
        fn test_various_string_build_metadata(#[case] metadata: &str) {
            let version = SemVer::new(1, 0, 0)
                .with_build_metadata(vec![BuildMetadata::String(metadata.to_string())]);
            assert_eq!(version.to_string(), format!("1.0.0+{metadata}"));
        }

        #[rstest]
        #[case(0)]
        #[case(1)]
        #[case(20240101)]
        #[case(u64::MAX)]
        fn test_various_integer_build_metadata(#[case] metadata: u64) {
            let version =
                SemVer::new(1, 0, 0).with_build_metadata(vec![BuildMetadata::Integer(metadata)]);
            assert_eq!(version.to_string(), format!("1.0.0+{metadata}"));
        }
    }

    mod combined_display {
        use super::*;

        #[test]
        fn test_pre_release_and_build_metadata() {
            let version = SemVer::new(1, 2, 3)
                .with_pre_release(vec![
                    PreReleaseIdentifier::String("alpha".to_string()),
                    PreReleaseIdentifier::Integer(1),
                ])
                .with_build_metadata(vec![
                    BuildMetadata::String("build".to_string()),
                    BuildMetadata::Integer(456),
                ]);
            assert_eq!(version.to_string(), "1.2.3-alpha.1+build.456");
        }

        #[test]
        fn test_complex_full_version() {
            let version = SemVer::new(10, 20, 30)
                .with_pre_release(vec![
                    PreReleaseIdentifier::String("rc".to_string()),
                    PreReleaseIdentifier::Integer(2),
                    PreReleaseIdentifier::String("hotfix".to_string()),
                ])
                .with_build_metadata(vec![
                    BuildMetadata::String("commit".to_string()),
                    BuildMetadata::String("abc123def".to_string()),
                    BuildMetadata::Integer(20240315),
                ]);
            assert_eq!(
                version.to_string(),
                "10.20.30-rc.2.hotfix+commit.abc123def.20240315"
            );
        }

        #[test]
        fn test_empty_pre_release_with_build_metadata() {
            let version = SemVer::new(1, 0, 0)
                .with_pre_release(vec![])
                .with_build_metadata(vec![BuildMetadata::String("build".to_string())]);
            assert_eq!(version.to_string(), "1.0.0+build");
        }

        #[test]
        fn test_pre_release_with_empty_build_metadata() {
            let version = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::String("alpha".to_string())])
                .with_build_metadata(vec![]);
            assert_eq!(version.to_string(), "1.0.0-alpha");
        }

        #[test]
        fn test_both_empty() {
            let version = SemVer::new(1, 0, 0)
                .with_pre_release(vec![])
                .with_build_metadata(vec![]);
            assert_eq!(version.to_string(), "1.0.0");
        }
    }

    mod identifier_display {
        use super::*;

        #[test]
        fn test_pre_release_identifier_string() {
            let identifier = PreReleaseIdentifier::String("alpha".to_string());
            assert_eq!(identifier.to_string(), "alpha");
        }

        #[test]
        fn test_pre_release_identifier_integer() {
            let identifier = PreReleaseIdentifier::Integer(123);
            assert_eq!(identifier.to_string(), "123");
        }

        #[test]
        fn test_build_metadata_string() {
            let metadata = BuildMetadata::String("build".to_string());
            assert_eq!(metadata.to_string(), "build");
        }

        #[test]
        fn test_build_metadata_integer() {
            let metadata = BuildMetadata::Integer(456);
            assert_eq!(metadata.to_string(), "456");
        }
    }

    mod helper_functions {
        use super::*;

        #[test]
        fn test_format_identifiers_empty() {
            let identifiers = vec![];
            assert_eq!(format_identifiers(&identifiers), "");
        }

        #[test]
        fn test_format_identifiers_single() {
            let identifiers = vec![PreReleaseIdentifier::String("alpha".to_string())];
            assert_eq!(format_identifiers(&identifiers), "alpha");
        }

        #[test]
        fn test_format_identifiers_multiple() {
            let identifiers = vec![
                PreReleaseIdentifier::String("alpha".to_string()),
                PreReleaseIdentifier::Integer(1),
                PreReleaseIdentifier::String("build".to_string()),
            ];
            assert_eq!(format_identifiers(&identifiers), "alpha.1.build");
        }

        #[test]
        fn test_format_build_metadata_empty() {
            let metadata = vec![];
            assert_eq!(format_build_metadata(&metadata), "");
        }

        #[test]
        fn test_format_build_metadata_single() {
            let metadata = vec![BuildMetadata::String("build".to_string())];
            assert_eq!(format_build_metadata(&metadata), "build");
        }

        #[test]
        fn test_format_build_metadata_multiple() {
            let metadata = vec![
                BuildMetadata::String("commit".to_string()),
                BuildMetadata::String("abc123".to_string()),
                BuildMetadata::Integer(789),
            ];
            assert_eq!(format_build_metadata(&metadata), "commit.abc123.789");
        }
    }
}
