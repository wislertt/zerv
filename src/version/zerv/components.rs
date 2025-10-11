use serde::{
    Deserialize,
    Serialize,
};
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};

/// Variable field enum for type-safe field references
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    EnumString,
    Display,
    EnumIter,
    AsRefStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum Var {
    // Core version fields
    Major,
    Minor,
    Patch,
    Epoch,

    // Pre-release fields
    PreRelease,

    // Post-release fields
    Post,
    Dev,

    // VCS state fields
    Distance,
    Dirty,

    // VCS context fields (bumped)
    BumpedBranch,
    BumpedCommitHash,
    BumpedCommitHashShort,
    BumpedTimestamp,

    // VCS context fields (last)
    LastBranch,
    LastCommitHash,
    LastTimestamp,

    // Legacy fields for backward compatibility
    Branch,
    CommitHashShort,

    // Custom fields
    #[serde(rename = "custom")]
    #[strum(disabled)]
    Custom(String),

    // Timestamp patterns
    #[serde(rename = "ts")]
    #[strum(disabled)]
    Timestamp(String),
}

/// Component enum for internal use with compact serialization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Component {
    #[serde(rename = "str")]
    Str(String),
    #[serde(rename = "int")]
    Int(u64),
    #[serde(rename = "var")]
    Var(Var),
}
