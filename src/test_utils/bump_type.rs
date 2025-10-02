use crate::constants::shared_constants;

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
            BumpType::Major => shared_constants::MAJOR,
            BumpType::Minor => shared_constants::MINOR,
            BumpType::Patch => shared_constants::PATCH,
            BumpType::Distance => shared_constants::DISTANCE,
            BumpType::Post => shared_constants::POST,
            BumpType::Dev => shared_constants::DEV,
            BumpType::Epoch => shared_constants::EPOCH,
            BumpType::PreRelease => shared_constants::PRE_RELEASE,
        }
    }
}
