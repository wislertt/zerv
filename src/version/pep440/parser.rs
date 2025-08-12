use crate::error::ZervError;
use crate::version::pep440::core::{LocalSegment, PEP440Version, PreReleaseLabel};
use regex::Regex;
use std::str::FromStr;
use std::sync::LazyLock;

static PEP440_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?ix)
        v?
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
        (?:\+(?P<local>[a-z0-9]+(?:[-_\.][a-z0-9]+)*))?       # local version
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

fn parse_local_segments(local: &str) -> Vec<LocalSegment> {
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

        let post_number = captures
            .name("post_n1")
            .or_else(|| captures.name("post_n2"))
            .and_then(|m| m.as_str().parse().ok());

        let dev_number = captures.name("dev_n").and_then(|m| m.as_str().parse().ok());

        let local = captures
            .name("local")
            .map(|m| parse_local_segments(m.as_str()));

        Ok(PEP440Version {
            epoch,
            release,
            pre_label,
            pre_number,
            post_label: "post",
            post_number,
            dev_number,
            local,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_version() {
        let version: PEP440Version = "1.2.3".parse().unwrap();
        assert_eq!(version.release, vec![1, 2, 3]);
        assert_eq!(version.epoch, 0);
        assert!(version.pre_label.is_none());
    }

    #[test]
    fn test_parse_complex_version() {
        let version: PEP440Version = "42!2025.12.31a99.post123.dev456+deadbeef.abc123"
            .parse()
            .unwrap();
        assert_eq!(version.epoch, 42);
        assert_eq!(version.release, vec![2025, 12, 31]);
        assert_eq!(version.pre_label, Some(PreReleaseLabel::Alpha));
        assert_eq!(version.pre_number, Some(99));
        assert_eq!(version.post_number, Some(123));
        assert_eq!(version.dev_number, Some(456));
        assert_eq!(
            version.local,
            Some(vec![
                LocalSegment::String("deadbeef".to_string()),
                LocalSegment::String("abc123".to_string())
            ])
        );
    }

    #[test]
    fn test_parse_with_local() {
        let version: PEP440Version = "1.0.0+ubuntu.20.04".parse().unwrap();
        let local = version.local.unwrap();
        assert_eq!(
            local,
            vec![
                LocalSegment::String("ubuntu".to_string()),
                LocalSegment::Integer(20),
                LocalSegment::Integer(4)
            ]
        );
    }
}
