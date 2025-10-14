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
    pub fn try_new_str(s: impl Into<String>) -> Result<Self, String> {
        let sanitized = Sanitizer::pep440_local_str().sanitize(&s.into());
        if sanitized.contains('.') {
            Err(format!(
                "LocalSegment cannot contain dots after sanitization: {sanitized}"
            ))
        } else {
            Ok(LocalSegment::Str(sanitized))
        }
    }

    pub fn new_uint(n: u32) -> Self {
        LocalSegment::UInt(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_segment_try_new_str_success() {
        // String inputs that should succeed (no dots after sanitization)
        assert_eq!(
            LocalSegment::try_new_str("ubuntu").unwrap(),
            LocalSegment::Str("ubuntu".to_string())
        );
        assert_eq!(
            LocalSegment::try_new_str("abc123").unwrap(),
            LocalSegment::Str("abc123".to_string())
        );
        assert_eq!(
            LocalSegment::try_new_str("123").unwrap(),
            LocalSegment::Str("123".to_string())
        );
        assert_eq!(
            LocalSegment::try_new_str("0").unwrap(),
            LocalSegment::Str("0".to_string())
        );
    }

    #[test]
    fn test_local_segment_try_new_str_failure() {
        // Inputs that contain dots after sanitization should fail
        assert!(LocalSegment::try_new_str("Feature/API-v2").is_err());
        assert!(LocalSegment::try_new_str("TEST_BRANCH").is_err());
        assert!(LocalSegment::try_new_str("Ubuntu-20.04").is_err());
        assert!(LocalSegment::try_new_str("test@#$%branch").is_err());
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
    fn test_local_segment_sanitization_success() {
        // Test sanitization that doesn't introduce dots
        assert_eq!(
            LocalSegment::try_new_str("  FEATURE  ").unwrap(),
            LocalSegment::Str("feature".to_string())
        );
        assert_eq!(
            LocalSegment::try_new_str("test123").unwrap(),
            LocalSegment::Str("test123".to_string())
        );
    }
}
