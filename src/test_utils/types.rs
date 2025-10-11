use crate::version::zerv::core::PreReleaseLabel;

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
        section: String,
        index: usize,
        value: u64,
    },
}

/// Enum for override types - stores override values for testing
#[derive(Debug, Clone, PartialEq)]
pub enum OverrideType {
    TagVersion(String),
    Distance(u32),
    Dirty(bool),
    CurrentBranch(String),
    CommitHash(String),
    Major(u32),
    Minor(u32),
    Patch(u32),
    Post(u32),
    Dev(u32),
    PreReleaseLabel(String),
    PreReleaseNum(u32),
    Epoch(u32),
}
