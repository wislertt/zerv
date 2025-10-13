use crate::utils::sanitize::Sanitizer;
use crate::version::zerv::PreReleaseLabel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalSegment {
    Str(String),
    UInt(u32),
}

pub fn pre_release_label_to_pep440_string(label: &PreReleaseLabel) -> &'static str {
    match label {
        PreReleaseLabel::Alpha => "a",
        PreReleaseLabel::Beta => "b",
        PreReleaseLabel::Rc => "rc",
    }
}

impl LocalSegment {
    pub fn new_str(s: impl Into<String>) -> Self {
        let sanitized = Sanitizer::pep440_local_str().sanitize(&s.into());
        LocalSegment::Str(sanitized)
    }

    pub fn new_uint(n: u32) -> Self {
        LocalSegment::UInt(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_segment_new_str() {
        // String inputs that should remain strings
        assert_eq!(
            LocalSegment::new_str("ubuntu"),
            LocalSegment::Str("ubuntu".to_string())
        );
        assert_eq!(
            LocalSegment::new_str("Feature/API-v2"),
            LocalSegment::Str("feature.api.v2".to_string())
        );
        assert_eq!(
            LocalSegment::new_str("TEST_BRANCH"),
            LocalSegment::Str("test.branch".to_string())
        );
        assert_eq!(
            LocalSegment::new_str("abc123"),
            LocalSegment::Str("abc123".to_string())
        );

        // Pure numeric strings are now allowed
        assert_eq!(
            LocalSegment::new_str("123"),
            LocalSegment::Str("123".to_string())
        );
        assert_eq!(
            LocalSegment::new_str("000045445"),
            LocalSegment::Str("45445".to_string())
        );
        assert_eq!(
            LocalSegment::new_str("0"),
            LocalSegment::Str("0".to_string())
        );
    }

    #[test]
    fn test_local_segment_new_uint() {
        assert_eq!(LocalSegment::new_uint(42), LocalSegment::UInt(42));
        assert_eq!(LocalSegment::new_uint(0), LocalSegment::UInt(0));
        assert_eq!(
            LocalSegment::new_uint(u32::MAX),
            LocalSegment::UInt(u32::MAX)
        );
    }

    #[test]
    fn test_local_segment_sanitization() {
        // Test various sanitization scenarios
        assert_eq!(
            LocalSegment::new_str("Ubuntu-20.04"),
            LocalSegment::Str("ubuntu.20.4".to_string())
        );
        assert_eq!(
            LocalSegment::new_str("  FEATURE  "),
            LocalSegment::Str("feature".to_string())
        );
        assert_eq!(
            LocalSegment::new_str("test@#$%branch"),
            LocalSegment::Str("test.branch".to_string())
        );
    }
}
