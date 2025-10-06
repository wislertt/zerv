use std::str::FromStr;
use std::sync::LazyLock;

use regex::Regex;

use crate::error::ZervError;
use crate::version::pep440::core::{
    LocalSegment,
    PEP440,
};
use crate::version::zerv::PreReleaseLabel;

static PEP440_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?ix)
        ^v?
        (?:
            (?:(?P<epoch>[0-9]+)!)?                           # epoch
            (?P<release>[0-9]+(?:\.[0-9]+)*)                  # release segment
            (?P<pre>                                          # pre-release
                [-_\.]?
                (?P<pre_l>alpha|a|beta|b|preview|pre|c|rc)
                [-_\.]?
                (?P<pre_n>[0-9]+)?
            )?
            (?P<post>                                         # post release
                (?:-(?P<post_n1>[0-9]+))
                |
                (?:
                    [-_\.]?
                    (?P<post_l>post|rev|r)
                    [-_\.]?
                    (?P<post_n2>[0-9]+)?
                )
            )?
            (?P<dev>                                          # dev release
                [-_\.]?
                (?P<dev_l>dev)
                [-_\.]?
                (?P<dev_n>[0-9]+)?
            )?
        )
        (?:\+(?P<local>[a-z0-9]+(?:[-_\.][a-z0-9]+)*))?$      # local version
    "#,
    )
    .unwrap()
});

pub fn parse_local_segments(local: &str) -> Vec<LocalSegment> {
    // Normalize separators: replace - and _ with .
    let normalized = local.replace(['-', '_'], ".");
    normalized
        .split('.')
        .map(|part| {
            if !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()) {
                LocalSegment::Integer(part.parse().unwrap_or(0))
            } else {
                LocalSegment::String(part.to_string())
            }
        })
        .collect()
}

impl FromStr for PEP440 {
    type Err = ZervError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = PEP440_REGEX
            .captures(s)
            .ok_or_else(|| ZervError::InvalidVersion(format!("Invalid PEP440 version: {s}")))?;

        let release = captures
            .name("release")
            .map(|m| {
                m.as_str()
                    .split('.')
                    .map(|x| x.parse().unwrap_or(0))
                    .collect()
            })
            .unwrap_or_else(|| vec![0]);

        let mut version = PEP440::new(release);

        if let Some(epoch_match) = captures.name("epoch") {
            let epoch = epoch_match.as_str().parse().unwrap_or(0);
            version = version.with_epoch(epoch);
        }

        if let Some(pre_l) = captures.name("pre_l") {
            let label = PreReleaseLabel::from_str_or_alpha(pre_l.as_str());
            let number = captures.name("pre_n").and_then(|m| m.as_str().parse().ok());
            version = version.with_pre_release(label, number);
        }

        if captures.name("post").is_some() {
            let post_number = captures
                .name("post_n1")
                .or_else(|| captures.name("post_n2"))
                .and_then(|m| m.as_str().parse().ok());
            version = version.with_post(post_number);
        }

        if captures.name("dev").is_some() {
            let dev_number = captures.name("dev_n").and_then(|m| m.as_str().parse().ok());
            version = version.with_dev(dev_number);
        }

        if let Some(local_match) = captures.name("local") {
            version = version.with_local(local_match.as_str());
        }

        Ok(version.normalize())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::version::pep440::core::PostLabel;

    #[rstest]
    #[case("1.2.3", vec![1, 2, 3])]
    #[case("0.1.0", vec![0, 1, 0])]
    #[case("10.20.30", vec![10, 20, 30])]
    fn test_parse_simple_versions(#[case] input: &str, #[case] release: Vec<u32>) {
        let parsed: PEP440 = input.parse().unwrap();
        let built = PEP440::new(release.clone());

        assert_eq!(parsed, built);
        assert_eq!(parsed.epoch, 0);
        assert_eq!(parsed.release, release);
        assert_eq!(parsed.pre_label, None);
        assert_eq!(parsed.pre_number, None);
        assert_eq!(parsed.post_number, None);
        assert_eq!(parsed.dev_number, None);
        assert_eq!(parsed.local, None);
    }

    #[rstest]
    #[case("5!1.2.3", 5, vec![1, 2, 3])]
    #[case("42!2025.12.31", 42, vec![2025, 12, 31])]
    #[case("0!1.0.0", 0, vec![1, 0, 0])]
    fn test_parse_with_epoch(#[case] input: &str, #[case] epoch: u32, #[case] release: Vec<u32>) {
        let parsed: PEP440 = input.parse().unwrap();
        let built = PEP440::new(release.clone()).with_epoch(epoch);

        assert_eq!(parsed, built);
        assert_eq!(parsed.epoch, epoch);
        assert_eq!(parsed.release, release);
    }

    #[rstest]
    #[case("1.0.0a1", PreReleaseLabel::Alpha, Some(1))]
    #[case("1.0.0alpha2", PreReleaseLabel::Alpha, Some(2))]
    #[case("1.0.0b3", PreReleaseLabel::Beta, Some(3))]
    #[case("1.0.0beta4", PreReleaseLabel::Beta, Some(4))]
    #[case("1.0.0rc5", PreReleaseLabel::Rc, Some(5))]
    #[case("1.0.0c6", PreReleaseLabel::Rc, Some(6))]
    #[case("1.0.0preview7", PreReleaseLabel::Rc, Some(7))]
    #[case("1.0.0pre8", PreReleaseLabel::Rc, Some(8))]
    #[case("1.0.0a", PreReleaseLabel::Alpha, Some(0))]
    #[case("1.0.0b", PreReleaseLabel::Beta, Some(0))]
    #[case("1.0.0rc", PreReleaseLabel::Rc, Some(0))]
    fn test_parse_pre_release(
        #[case] input: &str,
        #[case] pre_label: PreReleaseLabel,
        #[case] pre_number: Option<u32>,
    ) {
        let parsed: PEP440 = input.parse().unwrap();
        let built = PEP440::new(vec![1, 0, 0]).with_pre_release(pre_label.clone(), pre_number);

        assert_eq!(parsed, built);
        assert_eq!(parsed.pre_label, Some(pre_label));
        assert_eq!(parsed.pre_number, pre_number);
    }

    #[rstest]
    #[case("1.0.0.post1", Some(1))]
    #[case("1.0.0-2", Some(2))]
    #[case("1.0.0.rev3", Some(3))]
    #[case("1.0.0.r4", Some(4))]
    #[case("1.0.0post5", Some(5))]
    fn test_parse_post_release(#[case] input: &str, #[case] post_number: Option<u32>) {
        let parsed: PEP440 = input.parse().unwrap();
        let built = PEP440::new(vec![1, 0, 0]).with_post(post_number);

        assert_eq!(parsed, built);
        assert_eq!(parsed.post_number, post_number);
    }

    #[rstest]
    #[case("1.0.0.dev1", Some(1))]
    #[case("1.0.0dev2", Some(2))]
    #[case("1.0.0.dev", Some(0))]
    #[case("1.0.0dev", Some(0))]
    fn test_parse_dev_release(#[case] input: &str, #[case] dev_number: Option<u32>) {
        let parsed: PEP440 = input.parse().unwrap();
        let built = PEP440::new(vec![1, 0, 0]).with_dev(dev_number);
        assert_eq!(parsed, built);

        assert_eq!(parsed.dev_number, dev_number);
    }

    #[rstest]
    #[case("1.0.0+ubuntu.20.04")]
    #[case("1.0.0+deadbeef.abc123")]
    #[case("1.0.0+123.456")]
    #[case("1.0.0+local")]
    fn test_parse_local_versions(#[case] input: &str) {
        let parsed: PEP440 = input.parse().unwrap();
        // Extract local string from input for with_local
        let local_str = input.split('+').nth(1).unwrap();
        let built = PEP440::new(vec![1, 0, 0]).with_local(local_str);
        let expected_local = parse_local_segments(local_str);

        assert_eq!(parsed, built);
        assert_eq!(parsed.local, Some(expected_local));
    }

    #[rstest]
    #[case("42!2025.12.31a99.post123.dev456+deadbeef.abc123")]
    #[case("1!1.2.3b4.post5.dev6+local.meta")]
    #[case("0!1.0.0rc1.post2.dev3+build.123")]
    fn test_parse_complex_versions(#[case] input: &str) {
        let parsed: PEP440 = input.parse().unwrap();

        // Verify parsing succeeded and all components are present
        assert!(parsed.epoch > 0 || input.starts_with("0!"));
        assert!(!parsed.release.is_empty());
        assert!(parsed.pre_label.is_some());
        assert!(parsed.pre_number.is_some());
        assert!(parsed.post_number.is_some());
        assert!(parsed.dev_number.is_some());
        assert!(parsed.local.is_some());

        // Test round-trip: parse -> display -> parse should be equal
        let displayed = parsed.to_string();
        let reparsed: PEP440 = displayed.parse().unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[rstest]
    // Basic invalid formats
    #[case("invalid")]
    #[case("")]
    #[case("   ")]
    #[case(".")]
    #[case("..")]
    #[case("...")]
    // Invalid release segments
    #[case("1.")]
    #[case(".1")]
    #[case("1..2")]
    #[case("1.2.")]
    #[case("a.b.c")]
    // Invalid epochs
    #[case("!1.2.3")]
    #[case("a!1.2.3")]
    #[case("1.2!1.2.3")]
    #[case("-1!1.2.3")]
    // Invalid pre-release
    #[case("1.2.3x1")]
    #[case("1.2.3gamma1")]
    #[case("1.2.3aa1")]
    #[case("1.2.3a1a")]
    // Invalid post-release
    #[case("1.2.3.postx")]
    #[case("1.2.3.postpost1")]
    #[case("1.2.3--1")]
    #[case("1.2.3-")]
    // Invalid dev-release
    #[case("1.2.3.devx")]
    #[case("1.2.3.devdev1")]
    // Invalid local versions
    #[case("1.2.3+")]
    #[case("1.2.3++local")]
    #[case("1.2.3+local+")]
    #[case("1.2.3+local..build")]
    #[case("1.2.3+.local")]
    #[case("1.2.3+local.")]
    #[case("1.2.3+local@build")]
    #[case("1.2.3+local#build")]
    #[case("1.2.3+local$build")]
    #[case("1.2.3+local%build")]
    #[case("1.2.3+local^build")]
    #[case("1.2.3+local&build")]
    #[case("1.2.3+local*build")]
    #[case("1.2.3+local(build)")]
    #[case("1.2.3+local[build]")]
    #[case("1.2.3+local{build}")]
    #[case("1.2.3+local|build")]
    #[case("1.2.3+local\\build")]
    #[case("1.2.3+local/build")]
    #[case("1.2.3+local:build")]
    #[case("1.2.3+local;build")]
    #[case("1.2.3+local<build")]
    #[case("1.2.3+local>build")]
    #[case("1.2.3+local=build")]
    #[case("1.2.3+local?build")]
    #[case("1.2.3+local,build")]
    #[case("1.2.3+local build")]
    #[case("1.2.3+local\tuild")]
    #[case("1.2.3+local\nbuild")]
    // Mixed invalid cases
    #[case(
        "1.2.3.4.5.6.7.8.9.10.11.12.13.14.15.16.17.18.19.20.21.22.23.24.25.26.27.28.29.30.31.32.33.34.35.36.37.38.39.40.41.42.43.44.45.46.47.48.49.50+"
    )]
    #[case("v1.2.3+")]
    #[case("version1.2.3")]
    #[case("1.2.3.final")]
    #[case("1.2.3.stable")]
    #[case("1.2.3.release")]
    fn test_parse_invalid_versions(#[case] input: &str) {
        let result: Result<PEP440, _> = input.parse();
        assert!(
            result.is_err(),
            "Expected '{input}' to be invalid but it parsed successfully"
        );
    }

    #[test]
    fn test_parse_very_long_version() {
        // This should be valid as PEP440 doesn't limit release segment count
        let result: Result<PEP440, _> = "1.2.3.4.5.6.7.8.9.10.11.12".parse();
        assert!(result.is_ok());
        let version = result.unwrap();
        assert_eq!(version.release, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    }

    #[test]
    fn test_parse_local_segments() {
        let segments = parse_local_segments("ubuntu.20.04.build123");
        assert_eq!(
            segments,
            vec![
                LocalSegment::String("ubuntu".to_string()),
                LocalSegment::Integer(20),
                LocalSegment::Integer(4),
                LocalSegment::String("build123".to_string())
            ]
        );
    }

    // Test all PEP440 unnormalized forms
    #[rstest]
    // Pre-release label normalization: alpha/a, beta/b, rc/c/preview/pre
    #[case("1.0.0alpha1", PreReleaseLabel::Alpha, Some(1))]
    #[case("1.0.0ALPHA1", PreReleaseLabel::Alpha, Some(1))]
    #[case("1.0.0a1", PreReleaseLabel::Alpha, Some(1))]
    #[case("1.0.0A1", PreReleaseLabel::Alpha, Some(1))]
    #[case("1.0.0beta2", PreReleaseLabel::Beta, Some(2))]
    #[case("1.0.0BETA2", PreReleaseLabel::Beta, Some(2))]
    #[case("1.0.0b2", PreReleaseLabel::Beta, Some(2))]
    #[case("1.0.0B2", PreReleaseLabel::Beta, Some(2))]
    #[case("1.0.0rc3", PreReleaseLabel::Rc, Some(3))]
    #[case("1.0.0RC3", PreReleaseLabel::Rc, Some(3))]
    #[case("1.0.0c3", PreReleaseLabel::Rc, Some(3))]
    #[case("1.0.0C3", PreReleaseLabel::Rc, Some(3))]
    #[case("1.0.0preview4", PreReleaseLabel::Rc, Some(4))]
    #[case("1.0.0PREVIEW4", PreReleaseLabel::Rc, Some(4))]
    #[case("1.0.0pre5", PreReleaseLabel::Rc, Some(5))]
    #[case("1.0.0PRE5", PreReleaseLabel::Rc, Some(5))]
    fn test_parse_pre_release_normalization(
        #[case] input: &str,
        #[case] expected_label: PreReleaseLabel,
        #[case] expected_number: Option<u32>,
    ) {
        let parsed: PEP440 = input.parse().unwrap();
        assert_eq!(parsed.pre_label, Some(expected_label));
        assert_eq!(parsed.pre_number, expected_number);
    }

    #[rstest]
    // Pre-release separator normalization: ., -, _, or none
    #[case("1.0.0.a1", PreReleaseLabel::Alpha, Some(1))]
    #[case("1.0.0-a1", PreReleaseLabel::Alpha, Some(1))]
    #[case("1.0.0_a1", PreReleaseLabel::Alpha, Some(1))]
    #[case("1.0.0a1", PreReleaseLabel::Alpha, Some(1))]
    #[case("1.0.0.beta2", PreReleaseLabel::Beta, Some(2))]
    #[case("1.0.0-beta2", PreReleaseLabel::Beta, Some(2))]
    #[case("1.0.0_beta2", PreReleaseLabel::Beta, Some(2))]
    #[case("1.0.0beta2", PreReleaseLabel::Beta, Some(2))]
    #[case("1.0.0.rc3", PreReleaseLabel::Rc, Some(3))]
    #[case("1.0.0-rc3", PreReleaseLabel::Rc, Some(3))]
    #[case("1.0.0_rc3", PreReleaseLabel::Rc, Some(3))]
    #[case("1.0.0rc3", PreReleaseLabel::Rc, Some(3))]
    fn test_parse_pre_release_separators(
        #[case] input: &str,
        #[case] expected_label: PreReleaseLabel,
        #[case] expected_number: Option<u32>,
    ) {
        let parsed: PEP440 = input.parse().unwrap();
        assert_eq!(parsed.pre_label, Some(expected_label));
        assert_eq!(parsed.pre_number, expected_number);
    }

    #[rstest]
    // Post-release separator normalization: ., -, _, or none
    #[case("1.0.0.post1", Some(1))]
    #[case("1.0.0-post1", Some(1))]
    #[case("1.0.0_post1", Some(1))]
    #[case("1.0.0post1", Some(1))]
    #[case("1.0.0.rev2", Some(2))]
    #[case("1.0.0-rev2", Some(2))]
    #[case("1.0.0_rev2", Some(2))]
    #[case("1.0.0rev2", Some(2))]
    #[case("1.0.0.r3", Some(3))]
    #[case("1.0.0-r3", Some(3))]
    #[case("1.0.0_r3", Some(3))]
    #[case("1.0.0r3", Some(3))]
    // Implicit post-release (just -N)
    #[case("1.0.0-1", Some(1))]
    #[case("1.0.0-42", Some(42))]
    fn test_parse_post_release_separators(
        #[case] input: &str,
        #[case] expected_number: Option<u32>,
    ) {
        let parsed: PEP440 = input.parse().unwrap();
        assert_eq!(parsed.post_number, expected_number);
        assert_eq!(parsed.post_label, Some(PostLabel::Post));
    }

    #[rstest]
    // Dev-release separator normalization: ., -, _, or none
    #[case("1.0.0.dev1", Some(1))]
    #[case("1.0.0-dev1", Some(1))]
    #[case("1.0.0_dev1", Some(1))]
    #[case("1.0.0dev1", Some(1))]
    #[case("1.0.0.dev", Some(0))]
    #[case("1.0.0-dev", Some(0))]
    #[case("1.0.0_dev", Some(0))]
    #[case("1.0.0dev", Some(0))]
    fn test_parse_dev_release_separators(
        #[case] input: &str,
        #[case] expected_number: Option<u32>,
    ) {
        let parsed: PEP440 = input.parse().unwrap();
        assert_eq!(parsed.dev_number, expected_number);
    }

    #[rstest]
    // Mixed separator combinations
    #[case(
        "1.0.0-a1.post2_dev3",
        PreReleaseLabel::Alpha,
        Some(1),
        Some(2),
        Some(3)
    )]
    #[case(
        "1.0.0_beta1-post2.dev3",
        PreReleaseLabel::Beta,
        Some(1),
        Some(2),
        Some(3)
    )]
    #[case("1.0.0.rc1_post2-dev3", PreReleaseLabel::Rc, Some(1), Some(2), Some(3))]
    #[case(
        "1.0.0alpha1post2dev3",
        PreReleaseLabel::Alpha,
        Some(1),
        Some(2),
        Some(3)
    )]
    fn test_parse_mixed_separators(
        #[case] input: &str,
        #[case] expected_pre_label: PreReleaseLabel,
        #[case] expected_pre_number: Option<u32>,
        #[case] expected_post_number: Option<u32>,
        #[case] expected_dev_number: Option<u32>,
    ) {
        let parsed: PEP440 = input.parse().unwrap();
        assert_eq!(parsed.pre_label, Some(expected_pre_label));
        assert_eq!(parsed.pre_number, expected_pre_number);
        assert_eq!(parsed.post_number, expected_post_number);
        assert_eq!(parsed.dev_number, expected_dev_number);
    }

    #[rstest]
    // Case insensitive post-release labels
    #[case("1.0.0.POST1", Some(1))]
    #[case("1.0.0.Post1", Some(1))]
    #[case("1.0.0.REV2", Some(2))]
    #[case("1.0.0.Rev2", Some(2))]
    #[case("1.0.0.R3", Some(3))]
    fn test_parse_post_release_case_insensitive(
        #[case] input: &str,
        #[case] expected_number: Option<u32>,
    ) {
        let parsed: PEP440 = input.parse().unwrap();
        assert_eq!(parsed.post_number, expected_number);
    }

    #[rstest]
    // Case insensitive dev-release labels
    #[case("1.0.0.DEV1", Some(1))]
    #[case("1.0.0.Dev1", Some(1))]
    #[case("1.0.0.DEV", Some(0))]
    #[case("1.0.0.Dev", Some(0))]
    fn test_parse_dev_release_case_insensitive(
        #[case] input: &str,
        #[case] expected_number: Option<u32>,
    ) {
        let parsed: PEP440 = input.parse().unwrap();
        assert_eq!(parsed.dev_number, expected_number);
    }

    #[rstest]
    #[case("1.0.0+ubuntu-20-04")]
    #[case("1.0.0+ubuntu_20_04")]
    #[case("1.0.0+ubuntu.20.04")]
    fn test_parse_local_version_separators(#[case] input: &str) {
        let parsed: PEP440 = input.parse().unwrap();
        let local_str = input.split('+').nth(1).unwrap();
        let expected_local = parse_local_segments(local_str);

        assert_eq!(parsed.local, Some(expected_local));
    }

    #[test]
    fn test_parse_comprehensive_unnormalized() {
        // Test a complex version with all unnormalized forms
        let input = "1!2.0.0_ALPHA1-POST2.DEV3+build_123";
        let parsed: PEP440 = input.parse().unwrap();

        assert_eq!(parsed.epoch, 1);
        assert_eq!(parsed.release, vec![2, 0, 0]);
        assert_eq!(parsed.pre_label, Some(PreReleaseLabel::Alpha));
        assert_eq!(parsed.pre_number, Some(1));
        assert_eq!(parsed.post_number, Some(2));
        assert_eq!(parsed.dev_number, Some(3));
        assert_eq!(parsed.local, Some(parse_local_segments("build_123")));
    }

    #[rstest]
    #[case("0.0.0", vec![0, 0, 0])]
    #[case("4294967295.0.0", vec![4294967295, 0, 0])]
    #[case("1.01.0", vec![1, 1, 0])]
    fn test_parse_edge_cases(#[case] input: &str, #[case] expected_release: Vec<u32>) {
        let parsed: PEP440 = input.parse().unwrap();
        assert_eq!(parsed.release, expected_release);
    }

    #[test]
    fn test_parse_local_segments_edge_cases() {
        let segments = parse_local_segments("");
        assert_eq!(segments, vec![LocalSegment::String("".to_string())]);

        let segments = parse_local_segments("123");
        assert_eq!(segments, vec![LocalSegment::Integer(123)]);

        let segments = parse_local_segments("007.008");
        assert_eq!(
            segments,
            vec![LocalSegment::Integer(7), LocalSegment::Integer(8)]
        );
    }

    #[rstest]
    #[case("1.2.dev", Some(0))] // dev without number normalized to Some(0)
    #[case("1.2.dev5", Some(5))] // explicit dev number preserved
    #[case("1.2.post", Some(0))] // post without number normalized to Some(0)
    #[case("1.2.post3", Some(3))] // explicit post number preserved
    fn test_parse_normalization_dev_post(#[case] input: &str, #[case] expected: Option<u32>) {
        let parsed: PEP440 = input.parse().unwrap();

        if input.contains("dev") {
            assert_eq!(parsed.dev_number, expected);
        }
        if input.contains("post") {
            assert_eq!(parsed.post_number, expected);
        }
    }

    #[rstest]
    #[case("1.2.a", PreReleaseLabel::Alpha, Some(0))] // a without number normalized to Some(0)
    #[case("1.2.a2", PreReleaseLabel::Alpha, Some(2))] // explicit number preserved
    #[case("1.2.b", PreReleaseLabel::Beta, Some(0))] // b without number normalized to Some(0)
    #[case("1.2.rc", PreReleaseLabel::Rc, Some(0))] // rc without number normalized to Some(0)
    fn test_parse_normalization_pre_release(
        #[case] input: &str,
        #[case] expected_label: PreReleaseLabel,
        #[case] expected_number: Option<u32>,
    ) {
        let parsed: PEP440 = input.parse().unwrap();

        assert_eq!(parsed.pre_label, Some(expected_label));
        assert_eq!(parsed.pre_number, expected_number);
    }

    #[test]
    fn test_parse_normalization_comprehensive() {
        // Test version with all implicit numbers - now automatically normalized during parsing
        let parsed: PEP440 = "1.2.3a.post.dev".parse().unwrap();

        assert_eq!(parsed.pre_label, Some(PreReleaseLabel::Alpha));
        assert_eq!(parsed.pre_number, Some(0)); // automatically normalized to Some(0)
        assert_eq!(parsed.post_number, Some(0)); // automatically normalized to Some(0)
        assert_eq!(parsed.dev_number, Some(0)); // automatically normalized to Some(0)
    }
}
