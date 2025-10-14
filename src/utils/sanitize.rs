#[derive(Debug, Clone, PartialEq)]
pub enum SanitizeTarget {
    /// Clean string for version identifiers (alphanumeric + separator)
    Str,
    /// Extract digits as unsigned integer string
    UInt,
}

#[derive(Debug, Clone)]
pub struct Sanitizer {
    /// What type of output to produce
    pub target: SanitizeTarget,
    /// Replace non-alphanumeric characters with this separator, or None to keep unchanged (Str target only)
    pub separator: Option<String>,
    /// Convert to lowercase (String target only)
    pub lowercase: bool,
    /// Keep leading zeros in numeric segments
    pub keep_zeros: bool,
    /// Maximum length (truncate if longer)
    pub max_length: Option<usize>,
}

impl Sanitizer {
    /// Apply sanitization to input string
    pub fn sanitize(&self, input: &str) -> String {
        match self.target {
            SanitizeTarget::Str => self.sanitize_to_string(input),
            SanitizeTarget::UInt => self.sanitize_to_integer(input),
        }
    }

    /// PEP440 local string sanitization: lowercase, dots, no leading zeros
    pub fn pep440_local_str() -> Self {
        Self {
            target: SanitizeTarget::Str,
            separator: Some(".".to_string()),
            lowercase: true,
            keep_zeros: false,
            max_length: None,
        }
    }

    /// SemVer string sanitization: preserve case, dots
    pub fn semver_str() -> Self {
        Self {
            target: SanitizeTarget::Str,
            separator: Some(".".to_string()),
            lowercase: false,
            keep_zeros: false,
            max_length: None,
        }
    }

    /// Extract unsigned integer from string
    pub fn uint() -> Self {
        Self {
            target: SanitizeTarget::UInt,
            separator: None,
            lowercase: false,
            keep_zeros: false,
            max_length: None,
        }
    }

    /// Custom string sanitizer
    pub fn str(
        separator: Option<&str>,
        lowercase: bool,
        keep_zeros: bool,
        max_length: Option<usize>,
    ) -> Self {
        Self {
            target: SanitizeTarget::Str,
            separator: separator.map(|s| s.to_string()),
            lowercase,
            keep_zeros,
            max_length,
        }
    }

    /// Key sanitizer - for sanitizing keys
    pub fn key() -> Self {
        Self {
            target: SanitizeTarget::Str,
            separator: Some(".".to_string()),
            lowercase: true,
            keep_zeros: false,
            max_length: None,
        }
    }

    /// Sanitize to clean string
    fn sanitize_to_string(&self, input: &str) -> String {
        let mut result = input.to_string();

        if self.lowercase {
            result = result.to_lowercase();
        }

        result = self.replace_non_alphanumeric(&result);

        if !self.keep_zeros {
            result = self.remove_leading_zeros(&result);
        }

        if let Some(max_len) = self.max_length {
            result.truncate(max_len);
        }

        if let Some(sep) = &self.separator {
            result = result
                .trim_start_matches(sep)
                .trim_end_matches(sep)
                .to_string();
        }

        result
    }

    /// Extract unsigned integer from string
    fn sanitize_to_integer(&self, input: &str) -> String {
        let trimmed = input.trim();

        // Only accept strings that are purely digits
        if trimmed.chars().all(|c| c.is_ascii_digit()) && !trimmed.is_empty() {
            if self.keep_zeros {
                trimmed.to_string()
            } else {
                let without_leading_zeros = trimmed.trim_start_matches('0');
                if without_leading_zeros.is_empty() {
                    "0".to_string()
                } else {
                    without_leading_zeros.to_string()
                }
            }
        } else {
            "".to_string()
        }
    }

    /// Replace non-alphanumeric characters with separator or keep unchanged
    fn replace_non_alphanumeric(&self, input: &str) -> String {
        let Some(sep) = &self.separator else {
            return input.to_string();
        };

        let mut result = String::new();
        let mut last_was_sep = false;

        for ch in input.chars() {
            if ch.is_alphanumeric() {
                result.push(ch);
                last_was_sep = false;
            } else if !last_was_sep {
                result.push_str(sep);
                last_was_sep = true;
            }
        }

        result.trim_end_matches(sep).to_string()
    }

    /// Remove leading zeros from numeric segments
    fn remove_leading_zeros(&self, input: &str) -> String {
        let Some(sep) = &self.separator else {
            return self.remove_leading_zeros_from_segment(input);
        };

        if input.is_empty() {
            return input.to_string();
        }

        input
            .split(sep)
            .map(|segment| self.remove_leading_zeros_from_segment(segment))
            .collect::<Vec<_>>()
            .join(sep)
    }

    fn remove_leading_zeros_from_segment(&self, segment: &str) -> String {
        if !segment.is_empty() && segment.chars().all(|c| c.is_ascii_digit()) {
            let trimmed = segment.trim_start_matches('0');
            if trimmed.is_empty() {
                "0".to_string()
            } else {
                trimmed.to_string()
            }
        } else {
            segment.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn semver() -> Sanitizer {
        Sanitizer::semver_str()
    }
    fn pep440() -> Sanitizer {
        Sanitizer::pep440_local_str()
    }
    fn uint() -> Sanitizer {
        Sanitizer::uint()
    }
    fn key() -> Sanitizer {
        Sanitizer::key()
    }

    #[test]
    fn test_semver_str_sanitization() {
        let s = semver();
        assert_eq!(s.sanitize("feature/test-branch"), "feature.test.branch");
        assert_eq!(s.sanitize("Build-ID-0051"), "Build.ID.51");
        assert_eq!(s.sanitize("test@#$%branch"), "test.branch");
        assert_eq!(s.sanitize("Feature/API-v2"), "Feature.API.v2");
        assert_eq!(s.sanitize("build-id-0051"), "build.id.51");
        assert_eq!(s.sanitize("123"), "123");
        assert_eq!(s.sanitize("000045445"), "45445");
    }

    #[test]
    fn test_pep440_local_str_sanitization() {
        let s = pep440();
        assert_eq!(s.sanitize("Feature/API-v2"), "feature.api.v2");
        assert_eq!(s.sanitize("Build-ID-0051"), "build.id.51");
        assert_eq!(s.sanitize("TEST_BRANCH"), "test.branch");
        assert_eq!(s.sanitize("000045445"), "45445");
        assert_eq!(s.sanitize("123"), "123");
        assert_eq!(s.sanitize("0"), "0");
        assert_eq!(s.sanitize("999999"), "999999");
        assert_eq!(s.sanitize("  42  "), "42");
        assert_eq!(s.sanitize("abc123"), "abc123");
        assert_eq!(s.sanitize("123abc"), "123abc");
        assert_eq!(s.sanitize("v1.2.3"), "v1.2.3");
    }

    #[test]
    fn test_uint_extraction() {
        let s = uint();
        assert_eq!(s.sanitize("123"), "123");
        assert_eq!(s.sanitize("0051"), "51");
        assert_eq!(s.sanitize("0000"), "0");
        assert_eq!(s.sanitize("00123"), "123");
        assert_eq!(s.sanitize("abc123def456"), "");
        assert_eq!(s.sanitize("no-digits"), "");
        assert_eq!(s.sanitize("abc"), "");
        assert_eq!(s.sanitize(""), "");
    }

    #[test]
    fn test_custom_config() {
        let sanitizer = Sanitizer::str(Some("_"), true, true, Some(10));

        assert_eq!(sanitizer.sanitize("Feature/Test-0051"), "feature_te");
        assert_eq!(sanitizer.sanitize("Build-ID-0051"), "build_id_0");
    }

    #[test]
    fn test_leading_zeros() {
        let sanitizer_remove = Sanitizer::str(Some("."), false, false, None);
        let sanitizer_keep = Sanitizer::str(Some("."), false, true, None);

        assert_eq!(sanitizer_remove.sanitize("test-0051"), "test.51");
        assert_eq!(sanitizer_keep.sanitize("test-0051"), "test.0051");
        assert_eq!(sanitizer_remove.sanitize("test-0000"), "test.0");
    }

    #[test]
    fn test_max_length() {
        let sanitizer = Sanitizer::str(Some("."), false, false, Some(10));

        assert_eq!(sanitizer.sanitize("very-long-branch-name"), "very.long");
    }

    #[test]
    fn test_edge_cases() {
        let s = semver();
        assert_eq!(s.sanitize(""), "");
        assert_eq!(s.sanitize("123"), "123");
        assert_eq!(s.sanitize("@#$%"), "");
        assert_eq!(s.sanitize("a@#$%b"), "a.b");
    }

    #[test]
    fn test_no_separator() {
        let s = Sanitizer::str(None, false, false, None);
        assert_eq!(s.sanitize("feature/test-branch"), "feature/test-branch");
        assert_eq!(s.sanitize("Build-ID-0051"), "Build-ID-0051");
    }

    #[test]
    fn test_key_sanitizer() {
        let s = key();
        assert_eq!(s.sanitize("custom_field"), "custom.field");
        assert_eq!(s.sanitize("feature/API-v2"), "feature.api.v2");
        assert_eq!(s.sanitize("Build-ID-0051"), "build.id.51");
        assert_eq!(s.sanitize("test@#$%branch"), "test.branch");
        assert_eq!(s.sanitize(""), "");
    }

    use rstest::rstest;

    #[rstest]
    #[case(false)]
    #[case(true)]
    fn test_separator_trimming(#[case] keep_zeros: bool) {
        let s = Sanitizer::str(Some("."), false, keep_zeros, None);
        assert_eq!(s.sanitize("abc-test-branch-def"), "abc.test.branch.def");
        assert_eq!(s.sanitize("---test---"), "test");
        assert_eq!(s.sanitize("@#$test@#$"), "test");

        let s_short = Sanitizer::str(Some("."), false, keep_zeros, Some(10));
        assert_eq!(s_short.sanitize("very-long-branch"), "very.long");
    }
}
