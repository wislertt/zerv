pub const SCHEMA_ZERV_STANDARD: &str = "zerv-standard";

pub const FORMAT_PEP440: &str = "pep440";
pub const FORMAT_SEMVER: &str = "semver";
pub const FORMAT_ZERV: &str = "zerv";

pub const SUPPORTED_FORMATS_ARRAY: [&str; 3] = [FORMAT_SEMVER, FORMAT_PEP440, FORMAT_ZERV];
pub const SUPPORTED_FORMATS: &[&str] = &SUPPORTED_FORMATS_ARRAY;

pub const FORMAT_NAME_PEP440: &str = "PEP440";
pub const FORMAT_NAME_SEMVER: &str = "SemVer";
pub const FORMAT_NAME_ZERV: &str = "Zerv";
pub const SUPPORTED_FORMAT_NAMES: &[&str] =
    &[FORMAT_NAME_PEP440, FORMAT_NAME_SEMVER, FORMAT_NAME_ZERV];
