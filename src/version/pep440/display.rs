use std::fmt;

use super::core::{
    DevLabel,
    PEP440,
    PostLabel,
};
use super::utils::LocalSegment;

/// Format local version segments into a dot-separated string
pub fn format_local_segments(segments: &[LocalSegment]) -> String {
    segments
        .iter()
        .map(|segment| match segment {
            LocalSegment::UInt(n) => n.to_string(),
            LocalSegment::Str(s) => s.clone(),
        })
        .collect::<Vec<_>>()
        .join(".")
}

/// Format release version (e.g., [1, 2, 3] -> "1.2.3")
pub fn format_release_version(release: &[u32]) -> String {
    release
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

/// Format epoch and release version (e.g., epoch=2, release=[1,2,3] -> "2!1.2.3" or "1.2.3" if epoch=0)
pub fn format_epoch_and_release(epoch: u32, release: &[u32]) -> String {
    let mut result = String::new();

    // Add epoch if present
    if epoch > 0 {
        result.push_str(&format!("{}!", epoch));
    }

    // Add release version
    result.push_str(&format_release_version(release));

    result
}

/// PEP440 separator configuration for flexible version formatting
pub struct PEP440Separators<'a> {
    /// Release to Pre-release separator ("", ".", "-", "_")
    /// Normalized: "" → "1.0.0a1"
    /// Examples: "1.0.0-a1", "1.0.0.a1", "1.0.0_a1"
    pub pre_separator: &'a str,

    /// Pre-label to Pre-number separator ("", ".")
    /// Normalized: "" → "a1"
    /// Examples: "a.1"
    pub pre_number_separator: &'a str,

    /// Pre-release to Post-release separator (".", "-", "_")
    /// Normalized: "." → "1.0.0a1.post1" (direct attachment to .post)
    /// Examples: "1.0.0a1.post1", "1.0.0a1-post1", "1.0.0a1_post1"
    pub post_separator: &'a str,

    /// Post-label to Post-number separator ("", ".", "-", "_")
    /// Normalized: "" → "1.0.0.post1"
    /// Examples: "1.0.0.post.1"
    pub post_number_separator: &'a str,

    /// Post-release to Dev-release separator (".", "-", "_")
    /// Normalized: "." → "1.0.0.post1.dev1" (direct attachment to .dev)
    /// Examples: "1.0.0.post1.dev1", "1.0.0.post1-dev1", "1.0.0.post1_dev1"
    pub dev_separator: &'a str,

    /// Dev-label to Dev-number separator ("", ".", "-", "_")
    /// Normalized: "" → "1.0.0.dev1"
    /// Examples: "1.0.0.dev.1"
    pub dev_number_separator: &'a str,
}

impl<'a> PEP440Separators<'a> {
    /// Create normalized form separators
    pub fn normalized() -> Self {
        Self {
            pre_separator: "",
            pre_number_separator: "",
            post_separator: ".", // .post1
            post_number_separator: "",
            dev_separator: ".", // .dev1
            dev_number_separator: "",
        }
    }
}

/// Format just the pre-release section (alpha/beta/rc + post + dev) with configurable separators
pub fn format_pre_release_section(
    pre_label: Option<crate::version::zerv::PreReleaseLabel>,
    pre_number: Option<u32>,
    post_label: Option<PostLabel>,
    post_number: Option<u32>,
    dev_label: Option<DevLabel>,
    dev_number: Option<u32>,
    separators: &PEP440Separators<'_>,
) -> String {
    let mut result = String::new();

    // Main pre-release (alpha/beta/rc)
    if let Some(main_pre) = pre_label {
        result.push_str(separators.pre_separator);
        result.push_str(main_pre.as_str());

        if let Some(number) = pre_number {
            result.push_str(separators.pre_number_separator);
            result.push_str(&number.to_string());
        }
    }

    // Post-release (postN format with separator from PEP440)
    if post_label.is_some() {
        result.push_str(separators.post_separator);
        result.push_str("post");
        result.push_str(separators.post_number_separator);

        if let Some(number) = post_number {
            result.push_str(&number.to_string());
        }
    }

    // Dev-release (devN format with separator from PEP440)
    if dev_label.is_some() {
        result.push_str(separators.dev_separator);
        result.push_str("dev");
        result.push_str(separators.dev_number_separator);

        if let Some(number) = dev_number {
            result.push_str(&number.to_string());
        }
    }

    result
}

/// Format PEP440 version with configurable separators.
#[allow(clippy::too_many_arguments)]
pub fn format_pep440_with_separators(
    epoch: u32,
    release: &[u32],
    pre_label: Option<crate::version::zerv::PreReleaseLabel>,
    pre_number: Option<u32>,
    post_label: Option<PostLabel>,
    post_number: Option<u32>,
    dev_label: Option<DevLabel>,
    dev_number: Option<u32>,
    local: Option<&[LocalSegment]>,
    separators: &PEP440Separators<'_>,
    local_separator: &str, // Always "+" for PEP440
) -> String {
    let mut result = format_epoch_and_release(epoch, release);

    // Add pre-release section
    let pre_release_section = format_pre_release_section(
        pre_label,
        pre_number,
        post_label.clone(),
        post_number,
        dev_label.clone(),
        dev_number,
        separators,
    );

    if !pre_release_section.is_empty() {
        result.push_str(&pre_release_section);
    }

    // Local version
    if let Some(local) = local {
        result.push_str(local_separator);
        result.push_str(&format_local_segments(local));
    }

    result
}

impl fmt::Display for PEP440 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted = format_pep440_with_separators(
            self.epoch,
            &self.release,
            self.pre_label,
            self.pre_number,
            self.post_label.clone(),
            self.post_number,
            self.dev_label.clone(),
            self.dev_number,
            self.local.as_deref(),
            &PEP440Separators::normalized(),
            "+",
        );
        write!(f, "{}", formatted)
    }
}

impl fmt::Display for LocalSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocalSegment::Str(s) => write!(f, "{s}"),
            LocalSegment::UInt(i) => write!(f, "{i}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::PreReleaseLabel;

    #[test]
    fn test_display_simple_version() {
        let version = PEP440::new(vec![1, 2, 3]);
        assert_eq!(version.to_string(), "1.2.3");
    }

    #[test]
    fn test_display_complex_version() {
        let version = PEP440::new(vec![2025, 12, 31])
            .with_epoch(42)
            .with_pre_release(PreReleaseLabel::Alpha, Some(99))
            .with_post(Some(123))
            .with_dev(Some(456))
            .with_local("deadbeef.abc123");
        assert_eq!(
            version.to_string(),
            "42!2025.12.31a99.post123.dev456+deadbeef.abc123"
        );
    }

    #[test]
    fn test_display_with_local_mixed() {
        let version = PEP440::new(vec![1, 0, 0]).with_local("ubuntu.20.04");
        assert_eq!(version.to_string(), "1.0.0+ubuntu.20.4"); // "04" becomes "4"
    }

    #[test]
    fn test_display_pre_release_only() {
        let version = PEP440::new(vec![1, 2, 3]).with_pre_release(PreReleaseLabel::Beta, Some(2));
        assert_eq!(version.to_string(), "1.2.3b2");
    }

    #[test]
    fn test_display_local_segment() {
        assert_eq!(LocalSegment::Str("test".to_string()).to_string(), "test");
        assert_eq!(LocalSegment::UInt(42).to_string(), "42");
    }

    #[test]
    fn test_format_local_segments() {
        let segments = vec![
            LocalSegment::Str("ubuntu".to_string()),
            LocalSegment::UInt(20),
            LocalSegment::UInt(4),
            LocalSegment::Str("build123".to_string()),
        ];
        assert_eq!(format_local_segments(&segments), "ubuntu.20.4.build123");
    }

    #[test]
    fn test_parse_format_roundtrip() {
        use super::super::parser::parse_local_segments;
        let input = "deadbeef.123.abc";
        let segments = parse_local_segments(input);
        let output = format_local_segments(&segments);
        assert_eq!(output, input);
    }

    #[test]
    fn test_format_local_segments_edge_cases() {
        assert_eq!(format_local_segments(&[]), "");
        assert_eq!(format_local_segments(&[LocalSegment::UInt(42)]), "42");
        assert_eq!(
            format_local_segments(&[LocalSegment::Str("test".to_string())]),
            "test"
        );
        assert_eq!(
            format_local_segments(&[LocalSegment::UInt(u32::MAX)]),
            "4294967295"
        );
    }

    #[test]
    fn test_display_epoch_none() {
        let version = PEP440::new(vec![1, 0, 0]);
        assert_eq!(version.epoch, 0);
        assert_eq!(version.to_string(), "1.0.0");
    }

    #[test]
    fn test_display_epoch_zero() {
        let version = PEP440::new(vec![1, 0, 0]).with_epoch(0);
        assert_eq!(version.to_string(), "1.0.0");
    }

    #[test]
    fn test_display_edge_cases() {
        let mut version = PEP440::new(vec![1, 0, 0]);
        version.pre_label = Some(PreReleaseLabel::Alpha);
        version.pre_number = None;
        assert_eq!(version.to_string(), "1.0.0a");

        let mut version = PEP440::new(vec![1, 0, 0]);
        version.dev_number = None;
        assert_eq!(version.to_string(), "1.0.0");
    }

    #[test]
    fn test_display_with_dev_none() {
        let version = PEP440::new(vec![1, 2, 3]).with_dev(None);
        assert_eq!(version.to_string(), "1.2.3.dev");
    }

    #[test]
    fn test_display_with_post_none() {
        let version = PEP440::new(vec![1, 2, 3]).with_post(None);
        assert_eq!(version.to_string(), "1.2.3.post");
    }

    #[test]
    fn test_display_with_pre_release_none() {
        let version = PEP440::new(vec![1, 2, 3]).with_pre_release(PreReleaseLabel::Alpha, None);
        assert_eq!(version.to_string(), "1.2.3a");
    }

    #[test]
    fn test_display_individual_components() {
        // Test individual component formatting to ensure all write! lines are covered

        // Test release formatting (line 31)
        let version = PEP440::new(vec![9, 8, 7]);
        assert_eq!(version.to_string(), "9.8.7");

        // Test pre-release label formatting (line 35)
        let mut version = PEP440::new(vec![1, 0, 0]);
        version.pre_label = Some(PreReleaseLabel::Rc);
        version.pre_number = None;
        assert_eq!(version.to_string(), "1.0.0rc");

        // Test pre-release number formatting (line 37)
        let mut version = PEP440::new(vec![1, 0, 0]);
        version.pre_label = Some(PreReleaseLabel::Beta);
        version.pre_number = Some(5);
        assert_eq!(version.to_string(), "1.0.0b5");

        // Test post-release label formatting (line 43)
        let mut version = PEP440::new(vec![1, 0, 0]);
        version.post_label = Some(crate::version::pep440::core::PostLabel::Post);
        version.post_number = None;
        assert_eq!(version.to_string(), "1.0.0.post");

        // Test post-release number formatting (line 45)
        let mut version = PEP440::new(vec![1, 0, 0]);
        version.post_label = Some(crate::version::pep440::core::PostLabel::Post);
        version.post_number = Some(7);
        assert_eq!(version.to_string(), "1.0.0.post7");

        // Test dev-release label formatting (line 51)
        let mut version = PEP440::new(vec![1, 0, 0]);
        version.dev_label = Some(crate::version::pep440::core::DevLabel::Dev);
        version.dev_number = None;
        assert_eq!(version.to_string(), "1.0.0.dev");

        // Test dev-release number formatting (line 53)
        let mut version = PEP440::new(vec![1, 0, 0]);
        version.dev_label = Some(crate::version::pep440::core::DevLabel::Dev);
        version.dev_number = Some(3);
        assert_eq!(version.to_string(), "1.0.0.dev3");

        // Test local version formatting (line 59)
        let version = PEP440::new(vec![1, 0, 0]).with_local("test123");
        assert_eq!(version.to_string(), "1.0.0+test123");
    }
}
