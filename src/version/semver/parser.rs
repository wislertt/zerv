use crate::error::ZervError;
use crate::version::semver::core::{BuildMetadata, PreReleaseIdentifier, SemVer};
use regex::Regex;
use std::str::FromStr;
use std::sync::LazyLock;

static SEMVER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x)
        ^v?(?P<major>0|[1-9]\d*)                            # major version
        \.(?P<minor>0|[1-9]\d*)                            # minor version
        \.(?P<patch>0|[1-9]\d*)                            # patch version
        (?:                                                 # optional prerelease
            -(?P<prerelease>
                (?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)    # prerelease identifier
                (?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))* # additional identifiers
            )
        )?
        (?:                                                 # optional build metadata
            \+(?P<buildmetadata>
                [0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*           # build metadata
            )
        )?$
        ",
    )
    .unwrap()
});

fn parse_identifiers(input: &str) -> Vec<PreReleaseIdentifier> {
    if input.is_empty() {
        return vec![PreReleaseIdentifier::String("".to_string())];
    }
    input
        .split('.')
        .map(|part| {
            if part.chars().all(|c| c.is_ascii_digit()) && (part == "0" || !part.starts_with('0')) {
                PreReleaseIdentifier::Integer(part.parse().unwrap_or(0))
            } else {
                PreReleaseIdentifier::String(part.to_string())
            }
        })
        .collect()
}

fn parse_build_metadata(input: &str) -> Vec<BuildMetadata> {
    if input.is_empty() {
        return vec![BuildMetadata::String("".to_string())];
    }
    input
        .split('.')
        .map(|part| {
            if part.chars().all(|c| c.is_ascii_digit()) && (part == "0" || !part.starts_with('0')) {
                BuildMetadata::Integer(part.parse().unwrap_or(0))
            } else {
                BuildMetadata::String(part.to_string())
            }
        })
        .collect()
}

impl FromStr for SemVer {
    type Err = ZervError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = SEMVER_REGEX
            .captures(s)
            .ok_or_else(|| ZervError::InvalidVersion(format!("Invalid SemVer version: {s}")))?;

        let major = captures
            .name("major")
            .unwrap()
            .as_str()
            .parse()
            .map_err(|_| ZervError::InvalidVersion("Invalid major version".to_string()))?;

        let minor = captures
            .name("minor")
            .unwrap()
            .as_str()
            .parse()
            .map_err(|_| ZervError::InvalidVersion("Invalid minor version".to_string()))?;

        let patch = captures
            .name("patch")
            .unwrap()
            .as_str()
            .parse()
            .map_err(|_| ZervError::InvalidVersion("Invalid patch version".to_string()))?;

        let mut version = SemVer::new(major, minor, patch);

        if let Some(pre_release_match) = captures.name("prerelease") {
            let pre_release = parse_identifiers(pre_release_match.as_str());
            version = version.with_pre_release(pre_release);
        }

        if let Some(build_metadata_match) = captures.name("buildmetadata") {
            let build_metadata = parse_build_metadata(build_metadata_match.as_str());
            version = version.with_build_metadata(build_metadata);
        }

        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    mod basic_parsing {
        use super::*;

        #[rstest]
        #[case("0.0.0", 0, 0, 0)]
        #[case("1.0.0", 1, 0, 0)]
        #[case("1.2.3", 1, 2, 3)]
        #[case("10.20.30", 10, 20, 30)]
        #[case("999.888.777", 999, 888, 777)]
        fn test_parse_basic_versions(
            #[case] input: &str,
            #[case] major: u64,
            #[case] minor: u64,
            #[case] patch: u64,
        ) {
            let parsed: SemVer = input.parse().unwrap();
            assert_eq!(parsed.major, major);
            assert_eq!(parsed.minor, minor);
            assert_eq!(parsed.patch, patch);
            assert!(parsed.pre_release.is_none());
            assert!(parsed.build_metadata.is_none());
        }

        #[test]
        fn test_parse_max_values() {
            let input = format!("{}.{}.{}", u64::MAX, u64::MAX, u64::MAX);
            let parsed: SemVer = input.parse().unwrap();
            assert_eq!(parsed.major, u64::MAX);
            assert_eq!(parsed.minor, u64::MAX);
            assert_eq!(parsed.patch, u64::MAX);
        }
    }

    mod pre_release_parsing {
        use super::*;

        #[test]
        fn test_parse_single_string_pre_release() {
            let parsed: SemVer = "1.0.0-alpha".parse().unwrap();
            assert_eq!(
                parsed.pre_release,
                Some(vec![PreReleaseIdentifier::String("alpha".to_string())])
            );
        }

        #[test]
        fn test_parse_single_integer_pre_release() {
            let parsed: SemVer = "1.0.0-1".parse().unwrap();
            assert_eq!(
                parsed.pre_release,
                Some(vec![PreReleaseIdentifier::Integer(1)])
            );
        }

        #[test]
        fn test_parse_mixed_pre_release() {
            let parsed: SemVer = "1.0.0-alpha.1".parse().unwrap();
            assert_eq!(
                parsed.pre_release,
                Some(vec![
                    PreReleaseIdentifier::String("alpha".to_string()),
                    PreReleaseIdentifier::Integer(1),
                ])
            );
        }

        #[test]
        fn test_parse_complex_pre_release() {
            let parsed: SemVer = "1.0.0-alpha.1.beta.2".parse().unwrap();
            assert_eq!(
                parsed.pre_release,
                Some(vec![
                    PreReleaseIdentifier::String("alpha".to_string()),
                    PreReleaseIdentifier::Integer(1),
                    PreReleaseIdentifier::String("beta".to_string()),
                    PreReleaseIdentifier::Integer(2),
                ])
            );
        }

        #[rstest]
        #[case("1.0.0-alpha")]
        #[case("1.0.0-beta")]
        #[case("1.0.0-rc")]
        #[case("1.0.0-x")]
        #[case("1.0.0-a")]
        #[case("1.0.0-alpha-beta")]
        #[case("1.0.0-alpha123")]
        #[case("1.0.0-123alpha")]
        fn test_parse_various_string_pre_release(#[case] input: &str) {
            let parsed: SemVer = input.parse().unwrap();
            assert!(parsed.pre_release.is_some());
            assert!(parsed.is_pre_release());
        }

        #[rstest]
        #[case("1.0.0-0")]
        #[case("1.0.0-1")]
        #[case("1.0.0-999")]
        fn test_parse_various_integer_pre_release(#[case] input: &str) {
            let parsed: SemVer = input.parse().unwrap();
            assert!(parsed.pre_release.is_some());
            assert!(parsed.is_pre_release());
        }

        #[test]
        fn test_parse_leading_zero_pre_release() {
            // Leading zeros are invalid in SemVer according to the spec
            let result: Result<SemVer, _> = "1.0.0-01".parse();
            assert!(result.is_err(), "Leading zeros should be invalid in SemVer");
        }

        #[test]
        fn test_parse_zero_pre_release() {
            // "0" is valid as integer
            let parsed: SemVer = "1.0.0-0".parse().unwrap();
            assert_eq!(
                parsed.pre_release,
                Some(vec![PreReleaseIdentifier::Integer(0)])
            );
        }
    }

    mod build_metadata_parsing {
        use super::*;

        #[test]
        fn test_parse_single_string_build_metadata() {
            let parsed: SemVer = "1.0.0+build".parse().unwrap();
            assert_eq!(
                parsed.build_metadata,
                Some(vec![BuildMetadata::String("build".to_string())])
            );
        }

        #[test]
        fn test_parse_single_integer_build_metadata() {
            let parsed: SemVer = "1.0.0+123".parse().unwrap();
            assert_eq!(
                parsed.build_metadata,
                Some(vec![BuildMetadata::Integer(123)])
            );
        }

        #[test]
        fn test_parse_mixed_build_metadata() {
            let parsed: SemVer = "1.0.0+build.123".parse().unwrap();
            assert_eq!(
                parsed.build_metadata,
                Some(vec![
                    BuildMetadata::String("build".to_string()),
                    BuildMetadata::Integer(123),
                ])
            );
        }

        #[test]
        fn test_parse_complex_build_metadata() {
            let parsed: SemVer = "1.0.0+commit.abc123.20240101".parse().unwrap();
            assert_eq!(
                parsed.build_metadata,
                Some(vec![
                    BuildMetadata::String("commit".to_string()),
                    BuildMetadata::String("abc123".to_string()),
                    BuildMetadata::Integer(20240101),
                ])
            );
        }

        #[test]
        fn test_parse_leading_zero_build_metadata() {
            // Leading zeros are treated as strings in build metadata
            let parsed: SemVer = "1.0.0+01".parse().unwrap();
            assert_eq!(
                parsed.build_metadata,
                Some(vec![BuildMetadata::String("01".to_string())])
            );
        }

        #[test]
        fn test_parse_zero_build_metadata() {
            // "0" is valid as integer
            let parsed: SemVer = "1.0.0+0".parse().unwrap();
            assert_eq!(parsed.build_metadata, Some(vec![BuildMetadata::Integer(0)]));
        }
    }

    mod combined_parsing {
        use super::*;

        #[test]
        fn test_parse_pre_release_and_build_metadata() {
            let parsed: SemVer = "1.2.3-alpha.1+build.456".parse().unwrap();

            assert_eq!(parsed.major, 1);
            assert_eq!(parsed.minor, 2);
            assert_eq!(parsed.patch, 3);
            assert_eq!(
                parsed.pre_release,
                Some(vec![
                    PreReleaseIdentifier::String("alpha".to_string()),
                    PreReleaseIdentifier::Integer(1),
                ])
            );
            assert_eq!(
                parsed.build_metadata,
                Some(vec![
                    BuildMetadata::String("build".to_string()),
                    BuildMetadata::Integer(456),
                ])
            );
        }

        #[test]
        fn test_parse_complex_full_version() {
            let parsed: SemVer = "10.20.30-rc.2.hotfix+commit.abc123def.20240315"
                .parse()
                .unwrap();

            assert_eq!(parsed.major, 10);
            assert_eq!(parsed.minor, 20);
            assert_eq!(parsed.patch, 30);
            assert_eq!(
                parsed.pre_release,
                Some(vec![
                    PreReleaseIdentifier::String("rc".to_string()),
                    PreReleaseIdentifier::Integer(2),
                    PreReleaseIdentifier::String("hotfix".to_string()),
                ])
            );
            assert_eq!(
                parsed.build_metadata,
                Some(vec![
                    BuildMetadata::String("commit".to_string()),
                    BuildMetadata::String("abc123def".to_string()),
                    BuildMetadata::Integer(20240315),
                ])
            );
        }
    }

    mod invalid_parsing {
        use super::*;

        #[rstest]
        // Invalid basic format
        #[case("")]
        #[case("1")]
        #[case("1.2")]
        #[case("1.2.3.4")]
        #[case("1.2.3-")]
        #[case("1.2.3+")]
        #[case("1.2.3-+")]
        // Leading zeros in version numbers
        #[case("01.2.3")]
        #[case("1.02.3")]
        #[case("1.2.03")]
        // Invalid characters
        #[case("1.2.a")]
        #[case("a.2.3")]
        #[case("1.b.3")]
        // Invalid pre-release
        #[case("1.2.3-")]
        #[case("1.2.3-.")]
        #[case("1.2.3-..")]
        #[case("1.2.3-.alpha")]
        #[case("1.2.3-alpha.")]
        // Invalid build metadata
        #[case("1.2.3+")]
        #[case("1.2.3+.")]
        #[case("1.2.3+..")]
        #[case("1.2.3+.build")]
        #[case("1.2.3+build.")]
        // Negative numbers
        #[case("-1.2.3")]
        #[case("1.-2.3")]
        #[case("1.2.-3")]
        // Spaces
        #[case("1 .2.3")]
        #[case("1. 2.3")]
        #[case("1.2 .3")]
        #[case("1.2. 3")]
        #[case("1.2.3 -alpha")]
        #[case("1.2.3- alpha")]
        #[case("1.2.3 +build")]
        #[case("1.2.3+ build")]
        // Invalid separators
        #[case("1,2,3")]
        #[case("1:2:3")]
        #[case("1;2;3")]
        fn test_parse_invalid_versions(#[case] input: &str) {
            let result: Result<SemVer, _> = input.parse();
            assert!(
                result.is_err(),
                "Expected '{input}' to be invalid but it parsed successfully"
            );
        }
    }

    mod helper_functions {
        use super::*;

        #[test]
        fn test_parse_identifiers_empty() {
            let identifiers = parse_identifiers("");
            assert_eq!(
                identifiers,
                vec![PreReleaseIdentifier::String("".to_string())]
            );
        }

        #[test]
        fn test_parse_identifiers_single_string() {
            let identifiers = parse_identifiers("alpha");
            assert_eq!(
                identifiers,
                vec![PreReleaseIdentifier::String("alpha".to_string())]
            );
        }

        #[test]
        fn test_parse_identifiers_single_integer() {
            let identifiers = parse_identifiers("123");
            assert_eq!(identifiers, vec![PreReleaseIdentifier::Integer(123)]);
        }

        #[test]
        fn test_parse_identifiers_zero() {
            let identifiers = parse_identifiers("0");
            assert_eq!(identifiers, vec![PreReleaseIdentifier::Integer(0)]);
        }

        #[test]
        fn test_parse_identifiers_leading_zero() {
            let identifiers = parse_identifiers("01");
            assert_eq!(
                identifiers,
                vec![PreReleaseIdentifier::String("01".to_string())]
            );
        }

        #[test]
        fn test_parse_identifiers_mixed() {
            let identifiers = parse_identifiers("alpha.1.beta.2");
            assert_eq!(
                identifiers,
                vec![
                    PreReleaseIdentifier::String("alpha".to_string()),
                    PreReleaseIdentifier::Integer(1),
                    PreReleaseIdentifier::String("beta".to_string()),
                    PreReleaseIdentifier::Integer(2),
                ]
            );
        }

        #[test]
        fn test_parse_build_metadata_empty() {
            let metadata = parse_build_metadata("");
            assert_eq!(metadata, vec![BuildMetadata::String("".to_string())]);
        }

        #[test]
        fn test_parse_build_metadata_single_string() {
            let metadata = parse_build_metadata("build");
            assert_eq!(metadata, vec![BuildMetadata::String("build".to_string())]);
        }

        #[test]
        fn test_parse_build_metadata_single_integer() {
            let metadata = parse_build_metadata("123");
            assert_eq!(metadata, vec![BuildMetadata::Integer(123)]);
        }

        #[test]
        fn test_parse_build_metadata_zero() {
            let metadata = parse_build_metadata("0");
            assert_eq!(metadata, vec![BuildMetadata::Integer(0)]);
        }

        #[test]
        fn test_parse_build_metadata_leading_zero() {
            let metadata = parse_build_metadata("01");
            assert_eq!(metadata, vec![BuildMetadata::String("01".to_string())]);
        }

        #[test]
        fn test_parse_build_metadata_mixed() {
            let metadata = parse_build_metadata("commit.abc123.20240101");
            assert_eq!(
                metadata,
                vec![
                    BuildMetadata::String("commit".to_string()),
                    BuildMetadata::String("abc123".to_string()),
                    BuildMetadata::Integer(20240101),
                ]
            );
        }
    }

    mod roundtrip_tests {
        use super::*;

        #[rstest]
        // Basic versions
        #[case("0.0.0")]
        #[case("1.0.0")]
        #[case("1.2.3")]
        #[case("10.20.30")]
        // Pre-release versions
        #[case("1.0.0-alpha")]
        #[case("1.0.0-1")]
        #[case("1.0.0-alpha.1")]
        #[case("1.0.0-alpha.1.beta.2")]
        #[case("1.0.0-0")]
        // Build metadata versions
        #[case("1.0.0+build")]
        #[case("1.0.0+123")]
        #[case("1.0.0+build.123")]
        #[case("1.0.0+commit.abc123.20240101")]
        #[case("1.0.0+0")]
        // Combined versions
        #[case("1.2.3-alpha.1+build.456")]
        #[case("10.20.30-rc.2.hotfix+commit.abc123def.20240315")]
        #[case("1.0.0-alpha+beta")]
        #[case("1.0.0-0+0")]
        fn test_roundtrip_parsing(#[case] input: &str) {
            let parsed: SemVer = input.parse().unwrap();
            let output = parsed.to_string();
            assert_eq!(input, output, "Roundtrip failed for: {input}");

            // Also verify the reparsed version is equal
            let reparsed: SemVer = output.parse().unwrap();
            assert_eq!(parsed, reparsed, "Reparsed version should be equal");
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_parse_very_large_numbers() {
            let input = format!("{}.{}.{}", u64::MAX, u64::MAX, u64::MAX);
            let parsed: SemVer = input.parse().unwrap();
            assert_eq!(parsed.major, u64::MAX);
            assert_eq!(parsed.minor, u64::MAX);
            assert_eq!(parsed.patch, u64::MAX);
        }

        #[test]
        fn test_parse_long_pre_release() {
            let long_pre_release = (0..100)
                .map(|i| format!("part{i}"))
                .collect::<Vec<_>>()
                .join(".");
            let input = format!("1.0.0-{long_pre_release}");
            let parsed: SemVer = input.parse().unwrap();
            assert!(parsed.pre_release.is_some());
            assert_eq!(parsed.pre_release.as_ref().unwrap().len(), 100);
        }

        #[test]
        fn test_parse_long_build_metadata() {
            let long_build_metadata = (0..100)
                .map(|i| format!("meta{i}"))
                .collect::<Vec<_>>()
                .join(".");
            let input = format!("1.0.0+{long_build_metadata}");
            let parsed: SemVer = input.parse().unwrap();
            assert!(parsed.build_metadata.is_some());
            assert_eq!(parsed.build_metadata.as_ref().unwrap().len(), 100);
        }

        #[test]
        fn test_parse_alphanumeric_identifiers() {
            let parsed: SemVer = "1.0.0-alpha123beta456+build789meta012".parse().unwrap();
            assert_eq!(
                parsed.pre_release,
                Some(vec![PreReleaseIdentifier::String(
                    "alpha123beta456".to_string()
                )])
            );
            assert_eq!(
                parsed.build_metadata,
                Some(vec![BuildMetadata::String("build789meta012".to_string())])
            );
        }
    }
}
