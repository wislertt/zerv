use std::str::FromStr;

use crate::error::ZervError;
use crate::utils::constants::formats;
use crate::version::{
    PEP440,
    SemVer,
    Zerv,
    ZervVars,
};

#[derive(Debug, PartialEq)]
pub enum VersionObject {
    PEP440(PEP440),
    SemVer(SemVer),
}

impl VersionObject {
    pub fn format_str(&self) -> &'static str {
        match self {
            VersionObject::PEP440(_) => "pep440",
            VersionObject::SemVer(_) => "semver",
        }
    }

    /// Enhanced parsing with auto-detection and detailed error handling
    pub fn parse_with_format(tag: &str, format_str: &str) -> Result<Self, ZervError> {
        match format_str.to_lowercase().as_str() {
            "semver" => SemVer::from_str(tag)
                .map(VersionObject::SemVer)
                .map_err(|e| {
                    ZervError::InvalidFormat(format!("Invalid SemVer format '{tag}': {e}"))
                }),
            "pep440" => PEP440::from_str(tag)
                .map(VersionObject::PEP440)
                .map_err(|e| {
                    ZervError::InvalidFormat(format!("Invalid PEP440 format '{tag}': {e}"))
                }),
            "auto" => Self::parse_auto_detect(tag),
            _ => Err(ZervError::UnknownFormat(format!(
                "Unknown input format '{format_str}'. Supported formats: semver, pep440, auto"
            ))),
        }
    }

    /// Auto-detect version format (try SemVer first, then PEP440)
    fn parse_auto_detect(version_str: &str) -> Result<Self, ZervError> {
        // Try SemVer first
        if let Ok(semver) = SemVer::from_str(version_str) {
            return Ok(VersionObject::SemVer(semver));
        }

        // Fall back to PEP440
        if let Ok(pep440) = PEP440::from_str(version_str) {
            return Ok(VersionObject::PEP440(pep440));
        }

        Err(ZervError::InvalidVersion(format!(
            "Version '{version_str}' is not valid SemVer or PEP440 format"
        )))
    }

    /// Auto-detect version format for a list of version strings
    ///
    /// Returns a vector of tuples containing the original version string and the parsed VersionObject.
    /// The format is determined by majority vote - whichever format can parse more strings wins.
    /// In case of a tie, SemVer is preferred.
    pub fn parse_auto_detect_batch(
        version_strings: &[String],
    ) -> Result<Vec<(String, VersionObject)>, ZervError> {
        if version_strings.is_empty() {
            return Err(ZervError::InvalidArgument(
                "Version list cannot be empty".to_string(),
            ));
        }

        // Count successful parses for each format
        let mut semver_count = 0;
        let mut pep440_count = 0;

        // First pass: count parses for each format
        for version_str in version_strings {
            if SemVer::from_str(version_str).is_ok() {
                semver_count += 1;
            } else if PEP440::from_str(version_str).is_ok() {
                pep440_count += 1;
            }
        }

        // If nothing can be parsed, return error
        if semver_count == 0 && pep440_count == 0 {
            return Err(ZervError::InvalidVersion(
                "No version strings could be parsed as SemVer or PEP440".to_string(),
            ));
        }

        // Choose format based on counts (SemVer wins ties)
        let preferred_format = if semver_count >= pep440_count {
            formats::SEMVER
        } else {
            formats::PEP440
        };

        // Second pass: parse all strings using the preferred format ONLY
        let mut results = Vec::new();
        for version_str in version_strings {
            let parsed = match preferred_format {
                formats::SEMVER => {
                    if let Ok(semver) = SemVer::from_str(version_str) {
                        Some(VersionObject::SemVer(semver))
                    } else {
                        None // Skip if can't parse as SemVer
                    }
                }
                formats::PEP440 => {
                    if let Ok(pep440) = PEP440::from_str(version_str) {
                        Some(VersionObject::PEP440(pep440))
                    } else {
                        None // Skip if can't parse as PEP440
                    }
                }
                _ => unreachable!(), // We only use SEMVER or PEP440
            };

            if let Some(version_obj) = parsed {
                results.push((version_str.clone(), version_obj));
            }
        }

        Ok(results)
    }
}

impl From<VersionObject> for ZervVars {
    fn from(version: VersionObject) -> Self {
        match version {
            VersionObject::SemVer(semver) => {
                let zerv: Zerv = semver.into();
                zerv.vars
            }
            VersionObject::PEP440(pep440) => {
                let zerv: Zerv = pep440.into();
                zerv.vars
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("1.2.3", "semver", "semver")]
    #[case("1.2.3a1", "pep440", "pep440")]
    #[case("1.0.0-alpha.1", "SEMVER", "semver")] // case insensitive
    #[case("2!1.2.3", "PEP440", "pep440")] // case insensitive
    #[case("1.2.3", "auto", "semver")] // auto detection - semver
    #[case("1.2.3a1", "auto", "pep440")] // auto detection - pep440
    fn test_version_object_parse_with_format(
        #[case] tag: &str,
        #[case] format: &str,
        #[case] expected_format: &str,
    ) {
        let version = VersionObject::parse_with_format(tag, format).unwrap();
        assert_eq!(version.format_str(), expected_format);
    }

    #[rstest]
    #[case("1.2.3", "unknown", "Unknown input format")]
    #[case("1.2.3", "invalid", "Unknown input format")]
    #[case("invalid", "semver", "Invalid SemVer format")]
    #[case("invalid", "pep440", "Invalid PEP440 format")]
    #[case("completely-invalid", "auto", "not valid SemVer or PEP440 format")]
    fn test_version_object_parse_with_format_invalid(
        #[case] tag: &str,
        #[case] format: &str,
        #[case] expected_error: &str,
    ) {
        let error = VersionObject::parse_with_format(tag, format).unwrap_err();
        let error_message = error.to_string();
        assert!(
            error_message.contains(expected_error),
            "Expected error message to contain '{}', got: '{}'",
            expected_error,
            error_message
        );
    }

    #[test]
    fn test_version_object_format_str() {
        let semver = VersionObject::SemVer("1.2.3".parse().unwrap());
        let pep440 = VersionObject::PEP440("1.2.3a1".parse().unwrap());

        assert_eq!(semver.format_str(), "semver");
        assert_eq!(pep440.format_str(), "pep440");
    }

    #[rstest]
    #[case::semver_majority(
        vec!["1.0.0", "2.1.3", "3.0.0-alpha.1", "1.2.3a1", "4.5.6"],
        vec![
            ("1.0.0", VersionObject::SemVer(SemVer::from_str("1.0.0").unwrap())),
            ("2.1.3", VersionObject::SemVer(SemVer::from_str("2.1.3").unwrap())),
            ("3.0.0-alpha.1", VersionObject::SemVer(SemVer::from_str("3.0.0-alpha.1").unwrap())),
            ("4.5.6", VersionObject::SemVer(SemVer::from_str("4.5.6").unwrap())),
            // "1.2.3a1" is filtered out because it can't be parsed as SemVer
        ]
    )]
    #[case::pep440_majority(
        vec!["1.2.3a1", "2.0.0b2", "1.0.0rc1", "1.2.3", "3.4.5a0"],
        vec![
            ("1.2.3a1", VersionObject::PEP440(PEP440::from_str("1.2.3a1").unwrap())),
            ("2.0.0b2", VersionObject::PEP440(PEP440::from_str("2.0.0b2").unwrap())),
            ("1.0.0rc1", VersionObject::PEP440(PEP440::from_str("1.0.0rc1").unwrap())),
            ("1.2.3", VersionObject::PEP440(PEP440::from_str("1.2.3").unwrap())),
            ("3.4.5a0", VersionObject::PEP440(PEP440::from_str("3.4.5a0").unwrap())),
        ]
    )]
    #[case::tie_semver_wins(
        vec!["1.0.0", "1.2.3a1"],
        vec![
            ("1.0.0", VersionObject::SemVer(SemVer::from_str("1.0.0").unwrap())),
            // "1.2.3a1" is filtered out because SemVer wins the tie and it can't be parsed as SemVer
        ]
    )]
    #[case::all_semver(
        vec!["1.0.0", "2.0.0", "3.0.0"],
        vec![
            ("1.0.0", VersionObject::SemVer(SemVer::from_str("1.0.0").unwrap())),
            ("2.0.0", VersionObject::SemVer(SemVer::from_str("2.0.0").unwrap())),
            ("3.0.0", VersionObject::SemVer(SemVer::from_str("3.0.0").unwrap())),
        ]
    )]
    #[case::all_pep440(
        vec!["1.0.0a1", "2.0.0b2", "3.0.0rc1"],
        vec![
            ("1.0.0a1", VersionObject::PEP440(PEP440::from_str("1.0.0a1").unwrap())),
            ("2.0.0b2", VersionObject::PEP440(PEP440::from_str("2.0.0b2").unwrap())),
            ("3.0.0rc1", VersionObject::PEP440(PEP440::from_str("3.0.0rc1").unwrap())),
        ]
    )]
    #[case::all_pep440(
        vec!["v0", "v0.7", "v0.7.84"],
        vec![
            ("v0", VersionObject::PEP440(PEP440::from_str("v0").unwrap())),
            ("v0.7", VersionObject::PEP440(PEP440::from_str("v0.7").unwrap())),
            ("v0.7.84", VersionObject::PEP440(PEP440::from_str("v0.7.84").unwrap())),
        ]
    )]
    fn test_parse_auto_detect_batch_majority(
        #[case] versions: Vec<&str>,
        #[case] expected: Vec<(&str, VersionObject)>,
    ) {
        let version_strings: Vec<String> = versions.into_iter().map(|s| s.to_string()).collect();
        let result = VersionObject::parse_auto_detect_batch(&version_strings).unwrap();

        // Convert expected to the actual format (String, VersionObject)
        let expected_formatted: Vec<(String, VersionObject)> = expected
            .into_iter()
            .map(|(version_str, version_obj)| (version_str.to_string(), version_obj))
            .collect();

        assert_eq!(result, expected_formatted);
    }

    #[test]
    fn test_parse_auto_detect_batch_empty_list() {
        let versions = vec![];
        let result = VersionObject::parse_auto_detect_batch(&versions);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ZervError::InvalidArgument(_)));
        assert!(error.to_string().contains("Version list cannot be empty"));
    }

    #[test]
    fn test_parse_auto_detect_batch_no_valid_versions() {
        let versions = vec![
            "completely-invalid".to_string(),
            "not-a-version".to_string(),
            "123.456.789.abc".to_string(),
        ];

        let result = VersionObject::parse_auto_detect_batch(&versions);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ZervError::InvalidVersion(_)));
        assert!(
            error
                .to_string()
                .contains("No version strings could be parsed")
        );
    }
}
