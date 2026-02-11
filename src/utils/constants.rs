// Template variable names (for output templates)
pub mod template_vars {
    // Re-export shared constants
    pub use super::shared_constants::*;

    // Template-specific VCS context fields
    pub const BUMPED_BRANCH: &str = "bumped_branch";
    pub const BUMPED_COMMIT_HASH: &str = "bumped_commit_hash";
    pub const BUMPED_COMMIT_HASH_SHORT: &str = "bumped_commit_hash_short";
    pub const BUMPED_TIMESTAMP: &str = "bumped_timestamp";
}

// Pre-release label constants
pub mod pre_release_labels {
    pub const ALPHA: &str = "alpha";
    pub const BETA: &str = "beta";
    pub const RC: &str = "rc";

    pub const VALID_LABELS: &[&str] = &[ALPHA, BETA, RC];
}

// Shared field names (same for both RON and template) - renamed from shared_fields
pub mod shared_constants {
    // Core version fields
    pub const MAJOR: &str = "major";
    pub const MINOR: &str = "minor";
    pub const PATCH: &str = "patch";
    pub const EPOCH: &str = "epoch";

    // Pre-release fields
    pub const PRE_RELEASE: &str = "pre_release";

    // Post-release fields
    pub const POST: &str = "post";
    pub const DEV: &str = "dev";

    // VCS state fields
    pub const DISTANCE: &str = "distance";
    pub const DIRTY: &str = "dirty";

    // Last version fields
    pub const LAST_BRANCH: &str = "last_branch";
    pub const LAST_COMMIT_HASH: &str = "last_commit_hash";
    pub const LAST_TIMESTAMP: &str = "last_timestamp";

    // Custom fields
    pub const CUSTOM: &str = "custom";
}

// Bump type field constants - defined from shared_constants
pub mod bump_types {
    use super::shared_constants;

    pub const EPOCH: &str = shared_constants::EPOCH;
    pub const MAJOR: &str = shared_constants::MAJOR;
    pub const MINOR: &str = shared_constants::MINOR;
    pub const PATCH: &str = shared_constants::PATCH;
    pub const PRE_RELEASE_LABEL: &str = "pre_release_label";
    pub const PRE_RELEASE_NUM: &str = "pre_release_num";
    pub const POST: &str = shared_constants::POST;
    pub const DEV: &str = shared_constants::DEV;
}

// Timestamp patterns
pub mod timestamp_patterns {
    pub const COMPACT_DATE: &str = "compact_date";
    pub const COMPACT_DATETIME: &str = "compact_datetime";

    // Single component patterns
    pub const YYYY: &str = "YYYY";
    pub const YY: &str = "YY";
    pub const MM: &str = "MM";
    pub const ZERO_M: &str = "0M";
    pub const DD: &str = "DD";
    pub const ZERO_D: &str = "0D";
    pub const HH: &str = "HH";
    pub const ZERO_H: &str = "0H";
    pub const MM_MINUTE: &str = "mm";
    pub const ZERO_M_MINUTE: &str = "0m";
    pub const SS: &str = "SS";
    pub const ZERO_S: &str = "0S";
    pub const WW: &str = "WW";
    pub const ZERO_W: &str = "0W";

    pub fn get_valid_timestamp_patterns() -> Vec<&'static str> {
        vec![
            // Preset patterns
            COMPACT_DATE,
            COMPACT_DATETIME,
            // Single component patterns
            YYYY,
            YY,
            MM,
            ZERO_M,
            DD,
            ZERO_D,
            HH,
            ZERO_H,
            MM_MINUTE,
            ZERO_M_MINUTE,
            SS,
            ZERO_S,
            WW,
            ZERO_W,
        ]
    }
}

// Source types
pub mod sources {
    pub const GIT: &str = "git";
    pub const STDIN: &str = "stdin";
    pub const NONE: &str = "none";
}

// Post distance calculation modes
pub mod post_modes {
    pub const TAG: &str = "tag";
    pub const COMMIT: &str = "commit";

    /// Used for validation of post-mode argument
    pub const VALID_MODES: &[&str] = &[TAG, COMMIT];
}

// Format names
pub mod formats {
    pub const AUTO: &str = "auto";
    pub const SEMVER: &str = "semver";
    pub const PEP440: &str = "pep440";
    pub const ZERV: &str = "zerv";

    /// Format arrays for CLI validation
    pub const SUPPORTED_FORMATS_ARRAY: [&str; 3] = [SEMVER, PEP440, ZERV];
    pub const SUPPORTED_FORMATS: &[&str] = &SUPPORTED_FORMATS_ARRAY;
}

// Format display names
pub mod format_names {
    pub const PEP440: &str = "PEP440";
    pub const SEMVER: &str = "SemVer";
    pub const ZERV: &str = "Zerv";
}

// Format display name arrays for CLI validation
pub const SUPPORTED_FORMAT_NAMES: &[&str] = &[
    format_names::PEP440,
    format_names::SEMVER,
    format_names::ZERV,
];
