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

impl Default for Sanitizer {
    fn default() -> Self {
        Self {
            target: SanitizeTarget::Str,
            separator: Some(".".to_string()),
            lowercase: false,
            keep_zeros: false,
            max_length: None,
        }
    }
}

impl Sanitizer {
    /// Apply sanitization to input string
    pub fn sanitize(&self, input: &str) -> String {
        match self.target {
            SanitizeTarget::Str => self.sanitize_to_string(input),
            SanitizeTarget::UInt => self.sanitize_to_integer(input),
        }
    }

    /// PEP440 compatible: lowercase, dots, no leading zeros
    pub fn pep440() -> Self {
        Self {
            target: SanitizeTarget::Str,
            separator: Some(".".to_string()),
            lowercase: true,
            keep_zeros: false,
            max_length: None,
        }
    }

    /// SemVer compatible: preserve case, dots
    pub fn semver() -> Self {
        Self {
            target: SanitizeTarget::Str,
            separator: Some(".".to_string()),
            lowercase: false,
            keep_zeros: false,
            max_length: None,
        }
    }

    /// Extract unsigned integer from string
    pub fn uint(keep_zeros: bool) -> Self {
        Self {
            target: SanitizeTarget::UInt,
            separator: None,
            lowercase: false,
            keep_zeros,
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
        let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.is_empty() {
            return "0".to_string();
        }

        if self.keep_zeros {
            digits
        } else {
            let trimmed = digits.trim_start_matches('0');
            if trimmed.is_empty() {
                "0".to_string()
            } else {
                trimmed.to_string()
            }
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

    #[test]
    fn test_basic_sanitization() {
        let sanitizer = Sanitizer::default();

        assert_eq!(
            sanitizer.sanitize("feature/test-branch"),
            "feature.test.branch"
        );
        assert_eq!(sanitizer.sanitize("Build-ID-0051"), "Build.ID.51");
        assert_eq!(sanitizer.sanitize("test@#$%branch"), "test.branch");
    }

    #[test]
    fn test_pep440_sanitization() {
        let sanitizer = Sanitizer::pep440();

        assert_eq!(sanitizer.sanitize("Feature/API-v2"), "feature.api.v2");
        assert_eq!(sanitizer.sanitize("Build-ID-0051"), "build.id.51");
        assert_eq!(sanitizer.sanitize("TEST_BRANCH"), "test.branch");
    }

    #[test]
    fn test_semver_sanitization() {
        let sanitizer = Sanitizer::semver();

        assert_eq!(sanitizer.sanitize("Feature/API-v2"), "Feature.API.v2");
        assert_eq!(sanitizer.sanitize("build-id-0051"), "build.id.51");
    }

    #[test]
    fn test_uint_extraction() {
        let sanitizer = Sanitizer::uint(false);

        assert_eq!(sanitizer.sanitize("abc123def456"), "123456");
        assert_eq!(sanitizer.sanitize("0051"), "51");
        assert_eq!(sanitizer.sanitize("no-digits"), "0");

        let sanitizer_keep_zeros = Sanitizer::uint(true);
        assert_eq!(sanitizer_keep_zeros.sanitize("0051"), "0051");
    }

    #[test]
    fn test_custom_config() {
        let sanitizer = Sanitizer::str(Some("_"), true, true, Some(10));

        assert_eq!(sanitizer.sanitize("Feature/Test-0051"), "feature_te");
        assert_eq!(sanitizer.sanitize("Build-ID-0051"), "build_id_0");
    }

    #[test]
    fn test_leading_zeros() {
        let sanitizer_remove = Sanitizer {
            keep_zeros: false,
            ..Default::default()
        };
        let sanitizer_keep = Sanitizer {
            keep_zeros: true,
            ..Default::default()
        };

        assert_eq!(sanitizer_remove.sanitize("test-0051"), "test.51");
        assert_eq!(sanitizer_keep.sanitize("test-0051"), "test.0051");
        assert_eq!(sanitizer_remove.sanitize("test-0000"), "test.0");
    }

    #[test]
    fn test_max_length() {
        let sanitizer = Sanitizer {
            max_length: Some(10),
            ..Default::default()
        };

        assert_eq!(sanitizer.sanitize("very-long-branch-name"), "very.long");
    }

    #[test]
    fn test_edge_cases() {
        let sanitizer = Sanitizer::default();

        assert_eq!(sanitizer.sanitize(""), "");
        assert_eq!(sanitizer.sanitize("123"), "123");
        assert_eq!(sanitizer.sanitize("@#$%"), "");
        assert_eq!(sanitizer.sanitize("a@#$%b"), "a.b");
    }

    #[test]
    fn test_no_separator() {
        let sanitizer = Sanitizer {
            separator: None,
            ..Default::default()
        };

        assert_eq!(
            sanitizer.sanitize("feature/test-branch"),
            "feature/test-branch"
        );
        assert_eq!(sanitizer.sanitize("Build-ID-0051"), "Build-ID-0051");
    }

    #[test]
    fn test_uint_edge_cases() {
        let sanitizer = Sanitizer::uint(false);

        assert_eq!(sanitizer.sanitize(""), "0");
        assert_eq!(sanitizer.sanitize("abc"), "0");
        assert_eq!(sanitizer.sanitize("0000"), "0");
        assert_eq!(sanitizer.sanitize("00123"), "123");

        let sanitizer_keep = Sanitizer::uint(true);
        assert_eq!(sanitizer_keep.sanitize("0000"), "0000");
        assert_eq!(sanitizer_keep.sanitize("00123"), "00123");
    }

    #[test]
    fn test_key_sanitizer() {
        let sanitizer = Sanitizer::key();

        // Key sanitizer uses lowercase and dots as separator
        assert_eq!(sanitizer.sanitize("custom_field"), "custom.field");
        assert_eq!(sanitizer.sanitize("feature/API-v2"), "feature.api.v2");
        assert_eq!(sanitizer.sanitize("Build-ID-0051"), "build.id.51");
        assert_eq!(sanitizer.sanitize("test@#$%branch"), "test.branch");
        assert_eq!(sanitizer.sanitize(""), "");
    }

    use rstest::rstest;

    #[rstest]
    #[case(false)]
    #[case(true)]
    fn test_separator_trimming(#[case] keep_zeros: bool) {
        let sanitizer = Sanitizer {
            keep_zeros,
            ..Default::default()
        };

        // Test prefix/suffix separator trimming
        assert_eq!(
            sanitizer.sanitize("abc-test-branch-def"),
            "abc.test.branch.def"
        );
        assert_eq!(sanitizer.sanitize("---test---"), "test");
        assert_eq!(sanitizer.sanitize("@#$test@#$"), "test");

        // Test with max length causing trailing separator
        let sanitizer_short = Sanitizer {
            max_length: Some(10),
            keep_zeros,
            ..Default::default()
        };
        assert_eq!(sanitizer_short.sanitize("very-long-branch"), "very.long");
    }
}
