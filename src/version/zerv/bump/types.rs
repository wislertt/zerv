use crate::constants::bump_types;
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
}

impl BumpType {
    /// Single source of truth for precedence order (highest to lowest precedence)
    /// To change precedence order, only modify this array
    pub const PRECEDENCE_ORDER: &'static [BumpType] = &[
        BumpType::Epoch(0),                                // 0 - highest precedence
        BumpType::Major(0),                                // 1
        BumpType::Minor(0),                                // 2
        BumpType::Patch(0),                                // 3
        BumpType::PreReleaseLabel(PreReleaseLabel::Alpha), // 4
        BumpType::PreReleaseNum(0),                        // 5
        BumpType::Post(0),                                 // 6
        BumpType::Dev(0),                                  // 7 - lowest precedence
    ];

    /// Get precedence level for this bump type (lower number = higher precedence)
    pub fn precedence(&self) -> usize {
        Self::PRECEDENCE_ORDER
            .iter()
            .position(|bump_type| std::mem::discriminant(self) == std::mem::discriminant(bump_type))
            .unwrap_or(0) // Default to highest precedence if not found
    }

    /// Get the field name constant for this bump type
    pub fn field_name(&self) -> &'static str {
        match self {
            BumpType::Epoch(_) => bump_types::EPOCH,
            BumpType::Major(_) => bump_types::MAJOR,
            BumpType::Minor(_) => bump_types::MINOR,
            BumpType::Patch(_) => bump_types::PATCH,
            BumpType::PreReleaseLabel(_) => bump_types::PRE_RELEASE_LABEL,
            BumpType::PreReleaseNum(_) => bump_types::PRE_RELEASE_NUM,
            BumpType::Post(_) => bump_types::POST,
            BumpType::Dev(_) => bump_types::DEV,
        }
    }

    /// Get precedence level from component string
    pub fn precedence_from_str(component: &str) -> usize {
        // Find the position in PRECEDENCE_ORDER by matching field names
        Self::PRECEDENCE_ORDER
            .iter()
            .position(|bump_type| bump_type.field_name() == component)
            .unwrap_or(0) // Default to highest precedence for unknown components
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[test]
    fn test_precedence_order() {
        let types = BumpType::PRECEDENCE_ORDER;

        // Verify precedence is in ascending order
        for i in 1..types.len() {
            assert!(
                types[i - 1].precedence() < types[i].precedence(),
                "Precedence should be in ascending order: {} < {}",
                types[i - 1].precedence(),
                types[i].precedence()
            );
        }
    }

    #[rstest]
    #[case(BumpType::Epoch(0), 0)]
    #[case(BumpType::Major(0), 1)]
    #[case(BumpType::Minor(0), 2)]
    #[case(BumpType::Patch(0), 3)]
    #[case(BumpType::PreReleaseLabel(PreReleaseLabel::Alpha), 4)]
    #[case(BumpType::PreReleaseNum(0), 5)]
    #[case(BumpType::Post(0), 6)]
    #[case(BumpType::Dev(0), 7)]
    fn test_precedence_values(#[case] bump_type: BumpType, #[case] expected_precedence: usize) {
        assert_eq!(bump_type.precedence(), expected_precedence);
    }

    #[rstest]
    #[case(bump_types::EPOCH, 0)]
    #[case(bump_types::MAJOR, 1)]
    #[case(bump_types::MINOR, 2)]
    #[case(bump_types::PATCH, 3)]
    #[case(bump_types::PRE_RELEASE_LABEL, 4)]
    #[case(bump_types::PRE_RELEASE_NUM, 5)]
    #[case(bump_types::POST, 6)]
    #[case(bump_types::DEV, 7)]
    fn test_precedence_from_str(#[case] component: &str, #[case] expected_precedence: usize) {
        assert_eq!(
            BumpType::precedence_from_str(component),
            expected_precedence
        );
    }

    #[rstest]
    #[case(BumpType::Epoch(0), bump_types::EPOCH)]
    #[case(BumpType::Major(0), bump_types::MAJOR)]
    #[case(BumpType::Minor(0), bump_types::MINOR)]
    #[case(BumpType::Patch(0), bump_types::PATCH)]
    #[case(
        BumpType::PreReleaseLabel(PreReleaseLabel::Alpha),
        bump_types::PRE_RELEASE_LABEL
    )]
    #[case(BumpType::PreReleaseNum(0), bump_types::PRE_RELEASE_NUM)]
    #[case(BumpType::Post(0), bump_types::POST)]
    #[case(BumpType::Dev(0), bump_types::DEV)]
    fn test_field_names(#[case] bump_type: BumpType, #[case] expected_field_name: &str) {
        assert_eq!(bump_type.field_name(), expected_field_name);
    }
}
