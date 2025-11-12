use std::str::FromStr;

use crate::error::ZervError;
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
}
