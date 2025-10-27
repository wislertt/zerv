use crate::version::zerv::core::PreReleaseLabel;
use crate::version::zerv::schema::SchemaPartName;

/// Enum for bump types - stores increment value and label (test-only)
#[derive(Debug, Clone, PartialEq)]
pub enum BumpType {
    Epoch(u64),
    Major(u64),
    Minor(u64),
    Patch(u64),
    PreReleaseLabel(PreReleaseLabel),
    PreReleaseNum(u64),
    Post(u64),
    Dev(u64),
    SchemaBump {
        section: SchemaPartName,
        index: i32,
        value: Option<u64>,
    },
}

/// Enum for override types - stores override values for testing
#[derive(Debug, Clone, PartialEq)]
pub enum OverrideType {
    TagVersion(String),
    Distance(u32),
    Dirty(bool),
    BumpedBranch(String),
    BumpedCommitHash(String),
    BumpedTimestamp(i64),
    Major(u32),
    Minor(u32),
    Patch(u32),
    Post(u32),
    Dev(u32),
    PreReleaseLabel(String),
    PreReleaseNum(u32),
    Epoch(u32),
}
