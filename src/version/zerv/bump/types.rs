use std::collections::HashMap;

use once_cell::sync::Lazy;

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
    SchemaBump {
        section: String,
        index: usize,
        value: u64,
    },
}

impl BumpType {
    /// O(1) string -> index lookup map
    fn name_to_index() -> &'static HashMap<&'static str, usize> {
        static NAME_TO_INDEX: Lazy<HashMap<&'static str, usize>> = Lazy::new(|| {
            [
                (bump_types::EPOCH, 0),
                (bump_types::MAJOR, 1),
                (bump_types::MINOR, 2),
                (bump_types::PATCH, 3),
                (bump_types::PRE_RELEASE_LABEL, 4),
                (bump_types::PRE_RELEASE_NUM, 5),
                (bump_types::POST, 6),
                (bump_types::DEV, 7),
                ("schema_bump", 8),
            ]
            .iter()
            .map(|(name, index)| (*name, *index))
            .collect()
        });
        &NAME_TO_INDEX
    }

    /// O(1) precedence from BumpType
    pub fn precedence(&self) -> usize {
        Self::name_to_index()[self.to_str()]
    }

    /// Convert BumpType to string representation
    pub fn to_str(&self) -> &'static str {
        match self {
            BumpType::Epoch(_) => bump_types::EPOCH,
            BumpType::Major(_) => bump_types::MAJOR,
            BumpType::Minor(_) => bump_types::MINOR,
            BumpType::Patch(_) => bump_types::PATCH,
            BumpType::PreReleaseLabel(_) => bump_types::PRE_RELEASE_LABEL,
            BumpType::PreReleaseNum(_) => bump_types::PRE_RELEASE_NUM,
            BumpType::Post(_) => bump_types::POST,
            BumpType::Dev(_) => bump_types::DEV,
            BumpType::SchemaBump { .. } => "schema_bump",
        }
    }

    /// O(1) precedence from string
    pub fn precedence_from_str(component: &str) -> usize {
        Self::name_to_index()
            .get(component)
            .copied()
            .unwrap_or_else(|| panic!("Unknown component name: {component}"))
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    #[test]
    fn test_precedence_order() {
        // Verify precedence is in ascending order by checking each component
        let components = [
            BumpType::Epoch(0),
            BumpType::Major(0),
            BumpType::Minor(0),
            BumpType::Patch(0),
            BumpType::PreReleaseLabel(PreReleaseLabel::Alpha),
            BumpType::PreReleaseNum(0),
            BumpType::Post(0),
            BumpType::Dev(0),
            BumpType::SchemaBump {
                section: "core".to_string(),
                index: 0,
                value: 1,
            },
        ];

        for i in 1..components.len() {
            assert!(
                components[i - 1].precedence() < components[i].precedence(),
                "Precedence should be in ascending order: {} < {}",
                components[i - 1].precedence(),
                components[i].precedence()
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
    #[case(BumpType::SchemaBump { section: "core".to_string(), index: 0, value: 1 }, 8)]
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
    #[case("schema_bump", 8)]
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
    #[case(BumpType::SchemaBump { section: "core".to_string(), index: 0, value: 1 }, "schema_bump")]
    fn test_to_str(#[case] bump_type: BumpType, #[case] expected_field_name: &str) {
        assert_eq!(bump_type.to_str(), expected_field_name);
    }

    #[test]
    #[should_panic(expected = "Unknown component name: unknown")]
    fn test_precedence_from_str_invalid() {
        BumpType::precedence_from_str("unknown");
    }
}
