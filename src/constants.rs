/// Format identifier for PEP440 version format
pub const FORMAT_PEP440: &str = "pep440";

/// Format identifier for SemVer version format
pub const FORMAT_SEMVER: &str = "semver";

/// Default schema preset name
pub const SCHEMA_ZERV_STANDARD: &str = "zerv-standard";

/// List of all supported version formats
pub const SUPPORTED_FORMATS: &[&str] = &[FORMAT_PEP440, FORMAT_SEMVER];
