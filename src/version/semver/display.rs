use std::fmt;

use super::core::{
    BuildMetadata,
    PreReleaseIdentifier,
    SemVer,
};

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted = format_semver_with_separators(
            self.major,
            self.minor,
            self.patch,
            self.pre_release.as_deref(),
            self.build_metadata.as_deref(),
            "-",
            "+",
        );
        write!(f, "{}", formatted)
    }
}

impl fmt::Display for PreReleaseIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PreReleaseIdentifier::Str(s) => write!(f, "{s}"),
            PreReleaseIdentifier::UInt(n) => write!(f, "{n}"),
        }
    }
}

impl fmt::Display for BuildMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildMetadata::Str(s) => write!(f, "{s}"),
            BuildMetadata::UInt(n) => write!(f, "{n}"),
        }
    }
}

/// Format release version (e.g., 1, 2, 3 -> "1.2.3")
pub fn format_release_version(major: u64, minor: u64, patch: u64) -> String {
    format!("{}.{}.{}", major, minor, patch)
}

/// Format pre-release identifiers into a dot-separated string
pub fn format_pre_release_identifiers(identifiers: &[PreReleaseIdentifier]) -> String {
    identifiers
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

/// Format build metadata into a dot-separated string
pub fn format_build_metadata(metadata: &[BuildMetadata]) -> String {
    metadata
        .iter()
        .map(|meta| meta.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

/// General function to format SemVer with custom separators
pub fn format_semver_with_separators(
    major: u64,
    minor: u64,
    patch: u64,
    pre_release: Option<&[PreReleaseIdentifier]>,
    build_metadata: Option<&[BuildMetadata]>,
    pre_separator: &str,
    build_separator: &str,
) -> String {
    let mut result = format_release_version(major, minor, patch);

    // Add pre-release part with custom separator
    if let Some(pre) = pre_release
        && !pre.is_empty()
    {
        result.push_str(pre_separator);
        result.push_str(&format_pre_release_identifiers(pre));
    }

    // Add build metadata part with custom separator
    if let Some(build) = build_metadata
        && !build.is_empty()
    {
        result.push_str(build_separator);
        result.push_str(&format_build_metadata(build));
    }

    result
}

/// Format docker-compatible version (base-pre-release-build with hyphens)
pub fn format_docker_version(
    major: u64,
    minor: u64,
    patch: u64,
    pre_release: Option<&[PreReleaseIdentifier]>,
    build_metadata: Option<&[BuildMetadata]>,
) -> String {
    format_semver_with_separators(major, minor, patch, pre_release, build_metadata, "-", "-")
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

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
                .with_pre_release(vec![PreReleaseIdentifier::Str("alpha".to_string())]);
            assert_eq!(version.to_string(), "1.0.0-alpha");
        }

        #[test]
        fn test_single_integer_pre_release() {
            let version =
                SemVer::new(1, 0, 0).with_pre_release(vec![PreReleaseIdentifier::UInt(1)]);
            assert_eq!(version.to_string(), "1.0.0-1");
        }

        #[test]
        fn test_mixed_pre_release() {
            let version = SemVer::new(1, 0, 0).with_pre_release(vec![
                PreReleaseIdentifier::Str("alpha".to_string()),
                PreReleaseIdentifier::UInt(1),
            ]);
            assert_eq!(version.to_string(), "1.0.0-alpha.1");
        }

        #[test]
        fn test_complex_pre_release() {
            let version = SemVer::new(2, 1, 0).with_pre_release(vec![
                PreReleaseIdentifier::Str("rc".to_string()),
                PreReleaseIdentifier::UInt(2),
                PreReleaseIdentifier::Str("build".to_string()),
                PreReleaseIdentifier::UInt(456),
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
                .with_pre_release(vec![PreReleaseIdentifier::Str(pre_release.to_string())]);
            assert_eq!(version.to_string(), format!("1.0.0-{pre_release}"));
        }

        #[rstest]
        #[case(0)]
        #[case(1)]
        #[case(999)]
        #[case(u64::MAX)]
        fn test_various_integer_pre_release(#[case] pre_release: u64) {
            let version = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::UInt(pre_release)]);
            assert_eq!(version.to_string(), format!("1.0.0-{pre_release}"));
        }
    }

    mod build_metadata_display {
        use super::*;

        #[test]
        fn test_single_string_build_metadata() {
            let version = SemVer::new(1, 0, 0)
                .with_build_metadata(vec![BuildMetadata::Str("build".to_string())]);
            assert_eq!(version.to_string(), "1.0.0+build");
        }

        #[test]
        fn test_single_integer_build_metadata() {
            let version = SemVer::new(1, 0, 0).with_build_metadata(vec![BuildMetadata::UInt(123)]);
            assert_eq!(version.to_string(), "1.0.0+123");
        }

        #[test]
        fn test_mixed_build_metadata() {
            let version = SemVer::new(1, 0, 0).with_build_metadata(vec![
                BuildMetadata::Str("commit".to_string()),
                BuildMetadata::Str("abc123".to_string()),
            ]);
            assert_eq!(version.to_string(), "1.0.0+commit.abc123");
        }

        #[test]
        fn test_complex_build_metadata() {
            let version = SemVer::new(1, 5, 2).with_build_metadata(vec![
                BuildMetadata::Str("build".to_string()),
                BuildMetadata::UInt(789),
                BuildMetadata::Str("sha".to_string()),
                BuildMetadata::Str("def456".to_string()),
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
                .with_build_metadata(vec![BuildMetadata::Str(metadata.to_string())]);
            assert_eq!(version.to_string(), format!("1.0.0+{metadata}"));
        }

        #[rstest]
        #[case(0)]
        #[case(1)]
        #[case(20240101)]
        #[case(u64::MAX)]
        fn test_various_integer_build_metadata(#[case] metadata: u64) {
            let version =
                SemVer::new(1, 0, 0).with_build_metadata(vec![BuildMetadata::UInt(metadata)]);
            assert_eq!(version.to_string(), format!("1.0.0+{metadata}"));
        }
    }

    mod combined_display {
        use super::*;

        #[test]
        fn test_pre_release_and_build_metadata() {
            let version = SemVer::new(1, 2, 3)
                .with_pre_release(vec![
                    PreReleaseIdentifier::Str("alpha".to_string()),
                    PreReleaseIdentifier::UInt(1),
                ])
                .with_build_metadata(vec![
                    BuildMetadata::Str("build".to_string()),
                    BuildMetadata::UInt(456),
                ]);
            assert_eq!(version.to_string(), "1.2.3-alpha.1+build.456");
        }

        #[test]
        fn test_complex_full_version() {
            let version = SemVer::new(10, 20, 30)
                .with_pre_release(vec![
                    PreReleaseIdentifier::Str("rc".to_string()),
                    PreReleaseIdentifier::UInt(2),
                    PreReleaseIdentifier::Str("hotfix".to_string()),
                ])
                .with_build_metadata(vec![
                    BuildMetadata::Str("commit".to_string()),
                    BuildMetadata::Str("abc123def".to_string()),
                    BuildMetadata::UInt(20240315),
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
                .with_build_metadata(vec![BuildMetadata::Str("build".to_string())]);
            assert_eq!(version.to_string(), "1.0.0+build");
        }

        #[test]
        fn test_pre_release_with_empty_build_metadata() {
            let version = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::Str("alpha".to_string())])
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
            let identifier = PreReleaseIdentifier::Str("alpha".to_string());
            assert_eq!(identifier.to_string(), "alpha");
        }

        #[test]
        fn test_pre_release_identifier_integer() {
            let identifier = PreReleaseIdentifier::UInt(123);
            assert_eq!(identifier.to_string(), "123");
        }

        #[test]
        fn test_build_metadata_string() {
            let metadata = BuildMetadata::Str("build".to_string());
            assert_eq!(metadata.to_string(), "build");
        }

        #[test]
        fn test_build_metadata_integer() {
            let metadata = BuildMetadata::UInt(456);
            assert_eq!(metadata.to_string(), "456");
        }
    }

    mod helper_functions {
        use super::*;

        #[test]
        fn test_format_identifiers_empty() {
            let identifiers = vec![];
            assert_eq!(format_pre_release_identifiers(&identifiers), "");
        }

        #[test]
        fn test_format_identifiers_single() {
            let identifiers = vec![PreReleaseIdentifier::Str("alpha".to_string())];
            assert_eq!(format_pre_release_identifiers(&identifiers), "alpha");
        }

        #[test]
        fn test_format_identifiers_multiple() {
            let identifiers = vec![
                PreReleaseIdentifier::Str("alpha".to_string()),
                PreReleaseIdentifier::UInt(1),
                PreReleaseIdentifier::Str("build".to_string()),
            ];
            assert_eq!(
                format_pre_release_identifiers(&identifiers),
                "alpha.1.build"
            );
        }

        #[test]
        fn test_format_build_metadata_empty() {
            let metadata = vec![];
            assert_eq!(format_build_metadata(&metadata), "");
        }

        #[test]
        fn test_format_build_metadata_single() {
            let metadata = vec![BuildMetadata::Str("build".to_string())];
            assert_eq!(format_build_metadata(&metadata), "build");
        }

        #[test]
        fn test_format_build_metadata_multiple() {
            let metadata = vec![
                BuildMetadata::Str("commit".to_string()),
                BuildMetadata::Str("abc123".to_string()),
                BuildMetadata::UInt(789),
            ];
            assert_eq!(format_build_metadata(&metadata), "commit.abc123.789");
        }
    }
}
