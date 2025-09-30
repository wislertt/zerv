use crate::constants::shared_fields;

/// Enum for bump types - uses constants for field names
/// This is a test utility for organizing bump operations in tests
#[derive(Debug, Clone, PartialEq)]
pub enum BumpType {
    Major,
    Minor,
    Patch,
    Distance,
    Post,
    Dev,
    Epoch,
    PreRelease,
}

impl BumpType {
    /// Get the field name constant for this bump type
    pub fn field_name(&self) -> &'static str {
        match self {
            BumpType::Major => shared_fields::MAJOR,
            BumpType::Minor => shared_fields::MINOR,
            BumpType::Patch => shared_fields::PATCH,
            BumpType::Distance => shared_fields::DISTANCE,
            BumpType::Post => shared_fields::POST,
            BumpType::Dev => shared_fields::DEV,
            BumpType::Epoch => shared_fields::EPOCH,
            BumpType::PreRelease => shared_fields::PRE_RELEASE,
        }
    }
}
