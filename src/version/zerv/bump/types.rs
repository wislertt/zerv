use crate::version::zerv::core::PreReleaseLabel;

/// Enum for bump types - stores increment value and label
/// This defines the core bump operations and their precedence
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
