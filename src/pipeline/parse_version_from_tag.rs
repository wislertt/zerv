use std::str::FromStr;

use crate::version::{
    PEP440,
    SemVer,
    VersionObject,
};

/// Parse version from tag string with optional format specification
pub fn parse_version_from_tag(tag: &str, input_format: Option<&str>) -> Option<VersionObject> {
    match input_format {
        Some(format) => VersionObject::parse_with_format(tag, format),
        None => {
            // Auto-detection: try SemVer first, then PEP440
            if let Ok(semver) = SemVer::from_str(tag) {
                return Some(VersionObject::SemVer(semver));
            }

            if let Ok(pep440) = PEP440::from_str(tag) {
                return Some(VersionObject::PEP440(pep440));
            }

            None
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    // Basic SemVer cases (auto-detection should pick SemVer first)
    #[case("1.2.3", "1.2.3", "semver")]
    #[case("v1.2.3", "1.2.3", "semver")]
    #[case("0.0.0", "0.0.0", "semver")]
    #[case("10.20.30", "10.20.30", "semver")]
    // SemVer with pre-release
    #[case("1.0.0-alpha", "1.0.0-alpha", "semver")]
    #[case("1.0.0-alpha.1", "1.0.0-alpha.1", "semver")]
    #[case("1.0.0-beta.2", "1.0.0-beta.2", "semver")]
    #[case("1.0.0-rc.1", "1.0.0-rc.1", "semver")]
    // SemVer with build metadata
    #[case("1.0.0+build.1", "1.0.0+build.1", "semver")]
    #[case("1.0.0-alpha.1+build.123", "1.0.0-alpha.1+build.123", "semver")]
    // PEP440-specific cases (should fall back to PEP440)
    #[case("1.2.3a1", "1.2.3a1", "pep440")]
    #[case("1.2.3b2", "1.2.3b2", "pep440")]
    #[case("1.2.3rc3", "1.2.3rc3", "pep440")]
    #[case("1.2.3.post1", "1.2.3.post1", "pep440")]
    #[case("1.2.3.dev1", "1.2.3.dev1", "pep440")]
    #[case("2!1.2.3", "2!1.2.3", "pep440")]
    // PEP440 unnormalized forms
    #[case("1.2.3_alpha1", "1.2.3a1", "pep440")]
    #[case("1.2.3.ALPHA.1", "1.2.3a1", "pep440")]
    #[case("1.2.3.POST.1", "1.2.3.post1", "pep440")]
    // Complex PEP440 cases
    #[case(
        "2!1.2.3a1.post1.dev1+local.1",
        "2!1.2.3a1.post1.dev1+local.1",
        "pep440"
    )]
    fn test_parse_version_from_tag_auto_detection(
        #[case] tag: &str,
        #[case] expected_str: &str,
        #[case] expected_format: &str,
    ) {
        let version = parse_version_from_tag(tag, None)
            .unwrap_or_else(|| panic!("Failed to parse tag: {tag}"));

        // Direct comparison - no conversion needed!
        assert_eq!(version.format_str(), expected_format);

        match expected_format {
            "semver" => {
                if let VersionObject::SemVer(semver) = version {
                    let expected: SemVer = expected_str.parse().unwrap_or_else(|_| {
                        panic!("Failed to parse expected SemVer: {expected_str}")
                    });
                    assert_eq!(semver, expected);
                } else {
                    panic!("Expected SemVer, got {version:?}");
                }
            }
            "pep440" => {
                if let VersionObject::PEP440(pep440) = version {
                    let expected: PEP440 = expected_str.parse().unwrap_or_else(|_| {
                        panic!("Failed to parse expected PEP440: {expected_str}")
                    });
                    assert_eq!(pep440, expected);
                } else {
                    panic!("Expected PEP440, got {version:?}");
                }
            }
            _ => panic!("Unknown expected format: {expected_format}"),
        }
    }

    #[rstest]
    #[case("1.2.3", "1.2.3")]
    #[case("v1.2.3", "1.2.3")]
    #[case("1.2.3a1", "1.2.3a1")]
    #[case("2!1.2.3.post1.dev1", "2!1.2.3.post1.dev1")]
    #[case("1.2.3_alpha1", "1.2.3a1")] // unnormalized
    fn test_parse_version_from_tag_explicit_pep440(#[case] tag: &str, #[case] expected_str: &str) {
        let version = parse_version_from_tag(tag, Some("pep440"))
            .unwrap_or_else(|| panic!("Failed to parse PEP440 tag: {tag}"));

        if let VersionObject::PEP440(pep440) = version {
            let expected: PEP440 = expected_str
                .parse()
                .unwrap_or_else(|_| panic!("Failed to parse expected PEP440: {expected_str}"));
            assert_eq!(pep440, expected);
        } else {
            panic!("Expected PEP440, got {version:?}");
        }
    }

    #[rstest]
    #[case("1.2.3", "1.2.3")]
    #[case("v1.2.3", "1.2.3")]
    #[case("1.0.0-alpha.1", "1.0.0-alpha.1")]
    #[case("1.0.0+build.123", "1.0.0+build.123")]
    #[case("1.0.0-alpha.1+build.123", "1.0.0-alpha.1+build.123")]
    fn test_parse_version_from_tag_explicit_semver(#[case] tag: &str, #[case] expected_str: &str) {
        let version = parse_version_from_tag(tag, Some("semver"))
            .unwrap_or_else(|| panic!("Failed to parse SemVer tag: {tag}"));

        if let VersionObject::SemVer(semver) = version {
            let expected: SemVer = expected_str
                .parse()
                .unwrap_or_else(|_| panic!("Failed to parse expected SemVer: {expected_str}"));
            assert_eq!(semver, expected);
        } else {
            panic!("Expected SemVer, got {version:?}");
        }
    }

    #[rstest]
    #[case("invalid")]
    #[case("")]
    #[case("abc.def.ghi")]
    fn test_parse_version_from_tag_invalid(#[case] tag: &str) {
        let version = parse_version_from_tag(tag, None);
        assert!(version.is_none());
    }

    #[rstest]
    #[case("1.2.3", "unknown")]
    #[case("1.2.3", "invalid")]
    #[case("1.2.3", "custom")]
    fn test_parse_version_from_tag_unknown_format(#[case] tag: &str, #[case] format: &str) {
        let version = parse_version_from_tag(tag, Some(format));
        assert!(version.is_none());
    }

    #[rstest]
    #[case("1.2.3a1", "semver")] // PEP440 format with SemVer parser
    #[case("1.2.3.post1", "semver")] // PEP440 post-release with SemVer parser
    fn test_parse_version_from_tag_wrong_format(#[case] tag: &str, #[case] format: &str) {
        let version = parse_version_from_tag(tag, Some(format));
        assert!(version.is_none());
    }
}
