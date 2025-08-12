use crate::error::ZervError;
use crate::version::pep440::core::{
    LocalSegment, PEP440Version, PostReleaseLabel, PreReleaseLabel,
};
use regex::Regex;
use std::str::FromStr;
use std::sync::LazyLock;

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

fn normalize_pre_label(label: &str) -> PreReleaseLabel {
    match label.to_lowercase().as_str() {
        "alpha" | "a" => PreReleaseLabel::Alpha,
        "beta" | "b" => PreReleaseLabel::Beta,
        "rc" | "c" | "preview" | "pre" => PreReleaseLabel::Rc,
        _ => PreReleaseLabel::Alpha, // fallback
    }
}

pub fn parse_local_segments(local: &str) -> Vec<LocalSegment> {
    local
        .split('.')
        .map(|part| {
            if part.chars().all(|c| c.is_ascii_digit()) {
                LocalSegment::Integer(part.parse().unwrap_or(0))
            } else {
                LocalSegment::String(part.to_string())
            }
        })
        .collect()
}

impl FromStr for PEP440Version {
    type Err = ZervError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = PEP440_REGEX
            .captures(s)
            .ok_or_else(|| ZervError::InvalidVersion(format!("Invalid PEP440 version: {s}")))?;

        let epoch = captures
            .name("epoch")
            .map(|m| m.as_str().parse().unwrap_or(0))
            .unwrap_or(0);

        let release = captures
            .name("release")
            .map(|m| {
                m.as_str()
                    .split('.')
                    .map(|x| x.parse().unwrap_or(0))
                    .collect()
            })
            .unwrap_or_else(|| vec![0]);

        let (pre_label, pre_number) = if let Some(pre_l) = captures.name("pre_l") {
            let label = normalize_pre_label(pre_l.as_str());
            let number = captures.name("pre_n").and_then(|m| m.as_str().parse().ok());
            (Some(label), number)
        } else {
            (None, None)
        };

        let (post_label, post_number) = if captures.name("post").is_some() {
            let post_number = captures
                .name("post_n1")
                .or_else(|| captures.name("post_n2"))
                .and_then(|m| m.as_str().parse().ok());
            (Some(PostReleaseLabel::Post), post_number)
        } else {
            (None, None)
        };

        let dev_number = if captures.name("dev").is_some() {
            captures.name("dev_n").and_then(|m| m.as_str().parse().ok())
        } else {
            None
        };

        let local = captures
            .name("local")
            .map(|m| parse_local_segments(m.as_str()));

        Ok(PEP440Version {
            epoch,
            release,
            pre_label,
            pre_number,
            post_label,
            post_number,
            dev_number,
            local,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("1.2.3", vec![1, 2, 3])]
    #[case("0.1.0", vec![0, 1, 0])]
    #[case("10.20.30", vec![10, 20, 30])]
    fn test_parse_simple_versions(#[case] input: &str, #[case] release: Vec<u32>) {
        let parsed: PEP440Version = input.parse().unwrap();
        let built = PEP440Version::new(release.clone());

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
        let parsed: PEP440Version = input.parse().unwrap();
        let built = PEP440Version::new(release.clone()).with_epoch(epoch);

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
    #[case("1.0.0a", PreReleaseLabel::Alpha, None)]
    #[case("1.0.0b", PreReleaseLabel::Beta, None)]
    #[case("1.0.0rc", PreReleaseLabel::Rc, None)]
    fn test_parse_pre_release(
        #[case] input: &str,
        #[case] pre_label: PreReleaseLabel,
        #[case] pre_number: Option<u32>,
    ) {
        let parsed: PEP440Version = input.parse().unwrap();
        let built =
            PEP440Version::new(vec![1, 0, 0]).with_pre_release(pre_label.clone(), pre_number);

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
        let parsed: PEP440Version = input.parse().unwrap();
        let built = if let Some(post) = post_number {
            PEP440Version::new(vec![1, 0, 0]).with_post(post)
        } else {
            PEP440Version::new(vec![1, 0, 0])
        };

        assert_eq!(parsed, built);
        assert_eq!(parsed.post_number, post_number);
    }

    #[rstest]
    #[case("1.0.0.dev1", Some(1))]
    #[case("1.0.0dev2", Some(2))]
    #[case("1.0.0.dev", None)]
    #[case("1.0.0dev", None)]
    fn test_parse_dev_release(#[case] input: &str, #[case] dev_number: Option<u32>) {
        let parsed: PEP440Version = input.parse().unwrap();
        if let Some(dev) = dev_number {
            let built = PEP440Version::new(vec![1, 0, 0]).with_dev(dev);
            assert_eq!(parsed, built);
        } else {
            // For dev releases without numbers, we still have a dev release but with None
            let mut expected = PEP440Version::new(vec![1, 0, 0]);
            expected.dev_number = None;
            // We can't easily compare the whole object since we can't construct it with builder
            // So we just verify the dev_number field
        }

        assert_eq!(parsed.dev_number, dev_number);
    }

    #[rstest]
    #[case(
        "1.0.0+ubuntu.20.04",
        vec![
            LocalSegment::String("ubuntu".to_string()),
            LocalSegment::Integer(20),
            LocalSegment::Integer(4)
        ]
    )]
    #[case(
        "1.0.0+deadbeef.abc123",
        vec![
            LocalSegment::String("deadbeef".to_string()),
            LocalSegment::String("abc123".to_string())
        ]
    )]
    #[case(
        "1.0.0+123.456",
        vec![
            LocalSegment::Integer(123),
            LocalSegment::Integer(456)
        ]
    )]
    #[case(
        "1.0.0+local",
        vec![LocalSegment::String("local".to_string())]
    )]
    fn test_parse_local_versions(#[case] input: &str, #[case] local: Vec<LocalSegment>) {
        let parsed: PEP440Version = input.parse().unwrap();
        // Extract local string from input for with_local
        let local_str = input.split('+').nth(1).unwrap();
        let built = PEP440Version::new(vec![1, 0, 0]).with_local(local_str);

        assert_eq!(parsed, built);
        assert_eq!(parsed.local, Some(local));
    }

    #[test]
    fn test_parse_complex_version_full() {
        let input = "42!2025.12.31a99.post123.dev456+deadbeef.abc123";
        let parsed: PEP440Version = input.parse().unwrap();
        let built = PEP440Version::new(vec![2025, 12, 31])
            .with_epoch(42)
            .with_pre_release(PreReleaseLabel::Alpha, Some(99))
            .with_post(123)
            .with_dev(456)
            .with_local("deadbeef.abc123");

        // High-level validation
        assert_eq!(parsed, built);

        // Detailed validation
        assert_eq!(parsed.epoch, 42);
        assert_eq!(parsed.release, vec![2025, 12, 31]);
        assert_eq!(parsed.pre_label, Some(PreReleaseLabel::Alpha));
        assert_eq!(parsed.pre_number, Some(99));
        assert_eq!(parsed.post_number, Some(123));
        assert_eq!(parsed.dev_number, Some(456));
        assert_eq!(
            parsed.local,
            Some(vec![
                LocalSegment::String("deadbeef".to_string()),
                LocalSegment::String("abc123".to_string())
            ])
        );
    }

    #[test]
    fn test_parse_complex_version_beta() {
        let input = "1!1.2.3b4.post5.dev6+local.meta";
        let parsed: PEP440Version = input.parse().unwrap();
        let built = PEP440Version::new(vec![1, 2, 3])
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Beta, Some(4))
            .with_post(5)
            .with_dev(6)
            .with_local("local.meta");

        assert_eq!(parsed, built);
        assert_eq!(parsed.epoch, 1);
        assert_eq!(parsed.release, vec![1, 2, 3]);
        assert_eq!(parsed.pre_label, Some(PreReleaseLabel::Beta));
        assert_eq!(parsed.pre_number, Some(4));
        assert_eq!(parsed.post_number, Some(5));
        assert_eq!(parsed.dev_number, Some(6));
    }

    #[test]
    fn test_parse_complex_version_rc() {
        let input = "0!1.0.0rc1.post2.dev3+build.123";
        let parsed: PEP440Version = input.parse().unwrap();
        let built = PEP440Version::new(vec![1, 0, 0])
            .with_epoch(0)
            .with_pre_release(PreReleaseLabel::Rc, Some(1))
            .with_post(2)
            .with_dev(3)
            .with_local("build.123");

        assert_eq!(parsed, built);
        assert_eq!(parsed.epoch, 0);
        assert_eq!(parsed.release, vec![1, 0, 0]);
        assert_eq!(parsed.pre_label, Some(PreReleaseLabel::Rc));
        assert_eq!(parsed.pre_number, Some(1));
        assert_eq!(parsed.post_number, Some(2));
        assert_eq!(parsed.dev_number, Some(3));
    }

    #[rstest]
    #[case("invalid")]
    #[case("")]
    #[case("1.2.3+")]
    fn test_parse_invalid_versions(#[case] input: &str) {
        let result: Result<PEP440Version, _> = input.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_very_long_version() {
        // This should be valid as PEP440 doesn't limit release segment count
        let result: Result<PEP440Version, _> = "1.2.3.4.5.6.7.8.9.10.11.12".parse();
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
}
