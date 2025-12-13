// Git utility functions and helper methods

use crate::error::{
    Result,
    ZervError,
};
use crate::version::VersionObject;

/// Git utility functions for version tag processing
pub struct GitUtils;

impl GitUtils {
    /// Filter valid tags and return Vec<(tag_string, version_object)> with consistent format
    pub fn filter_only_valid_tags(
        tags: &[String],
        format: &str,
    ) -> Result<Vec<(String, VersionObject)>> {
        // Handle empty list case - return empty vector instead of error
        if tags.is_empty() {
            return Ok(Vec::new());
        }

        // Use parse_with_format_batch to handle parsing and format consistency
        match VersionObject::parse_with_format_batch(tags, format) {
            Ok(result) => Ok(result),
            // Convert errors to empty vector for backward compatibility
            Err(_) => Ok(Vec::new()),
        }
    }

    /// Find max version tag by comparing version objects
    pub fn find_max_version_tag(valid_tags: &[(String, VersionObject)]) -> Result<Option<String>> {
        if valid_tags.is_empty() {
            return Ok(None);
        }

        // Find the maximum version using custom comparison
        let max_tag = valid_tags
            .iter()
            .max_by(|(_, a), (_, b)| {
                // This should not fail since filter_only_valid_tags ensures same format
                Self::compare_version_objects(a, b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(tag, _)| tag.clone());

        Ok(max_tag)
    }

    /// Get format type string for a VersionObject
    pub fn get_format_type(version_obj: &VersionObject) -> String {
        match version_obj {
            VersionObject::SemVer(_) => "semver".to_string(),
            VersionObject::PEP440(_) => "pep440".to_string(),
        }
    }

    /// Compare two VersionObjects for ordering
    pub fn compare_version_objects(
        a: &VersionObject,
        b: &VersionObject,
    ) -> Result<std::cmp::Ordering> {
        // Check if they're the same type first
        if std::mem::discriminant(a) == std::mem::discriminant(b) {
            match (a, b) {
                (VersionObject::SemVer(a_sem), VersionObject::SemVer(b_sem)) => {
                    Ok(a_sem.cmp(b_sem))
                }
                (VersionObject::PEP440(a_pep), VersionObject::PEP440(b_pep)) => {
                    Ok(a_pep.cmp(b_pep))
                }
                // This case handles any other VersionObject variants that might be added in the future
                _ => Err(ZervError::InvalidFormat(
                    "Unsupported version object type for comparison".to_string(),
                )),
            }
        } else {
            Err(ZervError::InvalidFormat(
                "Cannot compare different version object types".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::GitUtils;
    use crate::version::VersionObject;

    #[rstest]
    // Basic semver case
    #[case(
        "semver",
        vec![
            "v1.0.0".to_string(),
            "v1.0.1".to_string(),
            "v2.0.0".to_string(),
        ],
        vec![
            ("v1.0.0".to_string(), VersionObject::parse_with_format("v1.0.0", "semver").unwrap()),
            ("v1.0.1".to_string(), VersionObject::parse_with_format("v1.0.1", "semver").unwrap()),
            ("v2.0.0".to_string(), VersionObject::parse_with_format("v2.0.0", "semver").unwrap()),
        ],
        Some("v2.0.0".to_string()),
    )]
    // RC versions - should filter out PEP440 format and keep only SemVer majority
    #[case(
        "semver",
        vec![
            "v1.0.0".to_string(),
            "v1.0.1".to_string(),
            "v2.0.0-alpha.1".to_string(),
            "1.0.0rc1".to_string(),
            "1.1.0a1".to_string(),
        ],
        vec![
            ("v1.0.0".to_string(), VersionObject::parse_with_format("v1.0.0", "semver").unwrap()),
            ("v1.0.1".to_string(), VersionObject::parse_with_format("v1.0.1", "semver").unwrap()),
            ("v2.0.0-alpha.1".to_string(), VersionObject::parse_with_format("v2.0.0-alpha.1", "semver").unwrap()),
        ],
        Some("v2.0.0-alpha.1".to_string()),
    )]
    // Mixed formats with semver parsing - only semver tags parse successfully
    #[case(
        "semver",
        vec![
            "v1.0.0".to_string(),
            "1.0.0rc1".to_string(),
            "1.1.0a1".to_string(),
            "1.2.0b2".to_string(),
            "1.0.0rc2".to_string(),
        ],
        vec![
            ("v1.0.0".to_string(), VersionObject::parse_with_format("v1.0.0", "semver").unwrap()),
        ],
        Some("v1.0.0".to_string()),
    )]
    // Auto format with mixed versions - PEP440 wins majority (6 vs 3)
    #[case(
        "auto",
        vec![
            "v1.0.0".to_string(),
            "v1.1.0".to_string(),
            "1.0.0rc1".to_string(),
            "1.1.0a1".to_string(),
            "1.2.0b2".to_string(),
            "v2.0.0-alpha.1".to_string(),
        ],
        vec![
            ("v1.0.0".to_string(), VersionObject::parse_with_format("v1.0.0", "pep440").unwrap()),
            ("v1.1.0".to_string(), VersionObject::parse_with_format("v1.1.0", "pep440").unwrap()),
            ("1.0.0rc1".to_string(), VersionObject::parse_with_format("1.0.0rc1", "pep440").unwrap()),
            ("1.1.0a1".to_string(), VersionObject::parse_with_format("1.1.0a1", "pep440").unwrap()),
            ("1.2.0b2".to_string(), VersionObject::parse_with_format("1.2.0b2", "pep440").unwrap()),
            ("v2.0.0-alpha.1".to_string(), VersionObject::parse_with_format("v2.0.0-alpha.1", "pep440").unwrap()),
        ],
        Some("v2.0.0-alpha.1".to_string()),
    )]
    // PEP440 format with complex versions
    #[case(
        "pep440",
        vec![
            "1.0.0".to_string(),
            "1.0.0rc1".to_string(),
            "1.1.0a1".to_string(),
            "1.2.0b2".to_string(),
            "2.0.0".to_string(),
            "1.0.0rc2".to_string(),
        ],
        vec![
            ("1.0.0".to_string(), VersionObject::parse_with_format("1.0.0", "pep440").unwrap()),
            ("1.0.0rc1".to_string(), VersionObject::parse_with_format("1.0.0rc1", "pep440").unwrap()),
            ("1.1.0a1".to_string(), VersionObject::parse_with_format("1.1.0a1", "pep440").unwrap()),
            ("1.2.0b2".to_string(), VersionObject::parse_with_format("1.2.0b2", "pep440").unwrap()),
            ("2.0.0".to_string(), VersionObject::parse_with_format("2.0.0", "pep440").unwrap()),
            ("1.0.0rc2".to_string(), VersionObject::parse_with_format("1.0.0rc2", "pep440").unwrap()),
        ],
        Some("2.0.0".to_string()),
    )]
    // Tie breaker - first tag's format should win (SemVer in this case)
    #[case(
        "semver",
        vec![
            "v1.0.0".to_string(),
            "1.0.0rc1".to_string(),
        ],
        vec![
            ("v1.0.0".to_string(), VersionObject::parse_with_format("v1.0.0", "semver").unwrap()),
        ],
        Some("v1.0.0".to_string()),
    )]
    // Complex versions with post releases
    #[case(
        "semver",
        vec![
            "v1.0.1-rc.1.post.1".to_string(),
            "v1.0.1-rc.1.post.2".to_string(),
            "v1.0.0".to_string(),
        ],
        vec![
            ("v1.0.1-rc.1.post.1".to_string(), VersionObject::parse_with_format("v1.0.1-rc.1.post.1", "semver").unwrap()),
            ("v1.0.1-rc.1.post.2".to_string(), VersionObject::parse_with_format("v1.0.1-rc.1.post.2", "semver").unwrap()),
            ("v1.0.0".to_string(), VersionObject::parse_with_format("v1.0.0", "semver").unwrap()),
        ],
        Some("v1.0.1-rc.1.post.2".to_string()),
    )]
    // No valid tags - should return empty
    #[case(
        "semver",
        vec![
            "invalid".to_string(),
            "not-a-version".to_string(),
            "123abc".to_string(),
        ],
        vec![],
        None,
    )]
    // Empty input
    #[case(
        "semver",
        vec![],
        vec![],
        None,
    )]
    fn test_filter_only_valid_tags(
        #[case] format: &str,
        #[case] tags: Vec<String>,
        #[case] expected_valid_tags: Vec<(String, VersionObject)>,
        #[case] expected_max_version_tag: Option<String>,
    ) {
        let filtered_tags = GitUtils::filter_only_valid_tags(&tags, format).unwrap();

        assert_eq!(filtered_tags, expected_valid_tags);

        // Test find_max_version_tag with the filtered tags
        let actual_max_version_tag = GitUtils::find_max_version_tag(&filtered_tags).unwrap();
        assert_eq!(actual_max_version_tag, expected_max_version_tag);
    }
}
