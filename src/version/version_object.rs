use std::str::FromStr;

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

    pub fn parse_with_format(tag: &str, format_str: &str) -> Option<Self> {
        match format_str.to_lowercase().as_str() {
            "pep440" => PEP440::from_str(tag).ok().map(VersionObject::PEP440),
            "semver" => SemVer::from_str(tag).ok().map(VersionObject::SemVer),
            _ => None,
        }
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
    fn test_version_object_parse_with_format(
        #[case] tag: &str,
        #[case] format: &str,
        #[case] expected_format: &str,
    ) {
        let version = VersionObject::parse_with_format(tag, format).unwrap();
        assert_eq!(version.format_str(), expected_format);
    }

    #[rstest]
    #[case("1.2.3", "unknown")]
    #[case("1.2.3", "invalid")]
    #[case("invalid", "semver")]
    fn test_version_object_parse_with_format_invalid(#[case] tag: &str, #[case] format: &str) {
        let version = VersionObject::parse_with_format(tag, format);
        assert!(version.is_none());
    }

    #[test]
    fn test_version_object_format_str() {
        let semver = VersionObject::SemVer("1.2.3".parse().unwrap());
        let pep440 = VersionObject::PEP440("1.2.3a1".parse().unwrap());

        assert_eq!(semver.format_str(), "semver");
        assert_eq!(pep440.format_str(), "pep440");
    }
}
