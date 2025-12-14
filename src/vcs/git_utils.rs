use crate::error::{
    Result,
    ZervError,
};
use crate::version::VersionObject;

pub struct GitUtils;

impl GitUtils {
    pub fn filter_only_valid_tags(tags: &[String], format: &str) -> Vec<(String, VersionObject)> {
        VersionObject::parse_with_format_batch(tags, format).unwrap_or_default()
    }

    pub fn find_max_version_tag(valid_tags: &[(String, VersionObject)]) -> Result<Option<String>> {
        if valid_tags.is_empty() {
            return Ok(None);
        }

        // Check that all tags are of the same type (all SemVer or all PEP440)
        if valid_tags.len() > 1 {
            let first_type = std::mem::discriminant(&valid_tags[0].1);
            for (_, version_obj) in valid_tags.iter().skip(1) {
                if std::mem::discriminant(version_obj) != first_type {
                    return Err(ZervError::InvalidArgument(
                        "All version objects must be of the same type (all SemVer or all PEP440)"
                            .to_string(),
                    ));
                }
            }
        }

        // Find the maximum version using custom comparison
        let max_tag = valid_tags
            .iter()
            .max_by(|(_, a), (_, b)| {
                // This should not fail since all types are now verified to be the same
                Self::compare_version_objects(a, b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(tag, _)| tag.clone());

        Ok(max_tag)
    }

    pub fn get_format_type(version_obj: &VersionObject) -> String {
        match version_obj {
            VersionObject::SemVer(_) => "semver".to_string(),
            VersionObject::PEP440(_) => "pep440".to_string(),
        }
    }

    pub fn compare_version_objects(
        a: &VersionObject,
        b: &VersionObject,
    ) -> Result<std::cmp::Ordering> {
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
            ("v1.0.0".to_string(), VersionObject::parse_semver("v1.0.0").unwrap()),
            ("v1.0.1".to_string(), VersionObject::parse_semver("v1.0.1").unwrap()),
            ("v2.0.0".to_string(), VersionObject::parse_semver("v2.0.0").unwrap()),
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
            ("v1.0.0".to_string(), VersionObject::parse_semver("v1.0.0").unwrap()),
            ("v1.0.1".to_string(), VersionObject::parse_semver("v1.0.1").unwrap()),
            ("v2.0.0-alpha.1".to_string(), VersionObject::parse_semver("v2.0.0-alpha.1").unwrap()),
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
            ("v1.0.0".to_string(), VersionObject::parse_semver("v1.0.0").unwrap()),
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
            ("v1.0.0".to_string(), VersionObject::parse_pep440("v1.0.0").unwrap()),
            ("v1.1.0".to_string(), VersionObject::parse_pep440("v1.1.0").unwrap()),
            ("1.0.0rc1".to_string(), VersionObject::parse_pep440("1.0.0rc1").unwrap()),
            ("1.1.0a1".to_string(), VersionObject::parse_pep440("1.1.0a1").unwrap()),
            ("1.2.0b2".to_string(), VersionObject::parse_pep440("1.2.0b2").unwrap()),
            ("v2.0.0-alpha.1".to_string(), VersionObject::parse_pep440("v2.0.0-alpha.1").unwrap()),
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
            ("1.0.0".to_string(), VersionObject::parse_pep440("1.0.0").unwrap()),
            ("1.0.0rc1".to_string(), VersionObject::parse_pep440("1.0.0rc1").unwrap()),
            ("1.1.0a1".to_string(), VersionObject::parse_pep440("1.1.0a1").unwrap()),
            ("1.2.0b2".to_string(), VersionObject::parse_pep440("1.2.0b2").unwrap()),
            ("2.0.0".to_string(), VersionObject::parse_pep440("2.0.0").unwrap()),
            ("1.0.0rc2".to_string(), VersionObject::parse_pep440("1.0.0rc2").unwrap()),
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
            ("v1.0.0".to_string(), VersionObject::parse_semver("v1.0.0").unwrap()),
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
            ("v1.0.1-rc.1.post.1".to_string(), VersionObject::parse_semver("v1.0.1-rc.1.post.1").unwrap()),
            ("v1.0.1-rc.1.post.2".to_string(), VersionObject::parse_semver("v1.0.1-rc.1.post.2").unwrap()),
            ("v1.0.0".to_string(), VersionObject::parse_semver("v1.0.0").unwrap()),
        ],
        Some("v1.0.1-rc.1.post.2".to_string()),
    )]
    // Realistic tags with prefix
    #[case(
        "auto",
        vec![
            "v1".to_string(),
            "v1.2".to_string(),
            "v1.2.3".to_string(),
        ],
        vec![
            ("v1".to_string(), VersionObject::parse_pep440("v1").unwrap()),
            ("v1.2".to_string(), VersionObject::parse_pep440("v1.2").unwrap()),
            ("v1.2.3".to_string(), VersionObject::parse_pep440("v1.2.3").unwrap()),
        ],
        Some("v1.2.3".to_string()),
    )]
    #[case(
        "auto",
        vec![
            "v1".to_string(),
            "v1.2".to_string(),
            "v1.2.3".to_string(),
            "v1.2.3-alpha.1.post.1.semver".to_string(),
            "v1.2.3-alpha.2.post.1.semver".to_string(),
        ],
        vec![
            ("v1.2.3".to_string(), VersionObject::parse_semver("v1.2.3").unwrap()),
            ("v1.2.3-alpha.1.post.1.semver".to_string(), VersionObject::parse_semver("v1.2.3-alpha.1.post.1.semver").unwrap()),
            ("v1.2.3-alpha.2.post.1.semver".to_string(), VersionObject::parse_semver("v1.2.3-alpha.2.post.1.semver").unwrap()),
        ],
        Some("v1.2.3".to_string()),
    )]
    #[case(
        "auto",
        vec![
            "v1".to_string(),
            "v1.2".to_string(),
            "v1.2.3".to_string(),
            "v1.2.3-alpha.1.post.1.semver".to_string(),
            "v1.2.3-alpha.2.post.1.semver".to_string(),
            "v1.2.3-alpha.3.post.1.semver".to_string(),
        ],
        vec![
            ("v1.2.3".to_string(), VersionObject::parse_semver("v1.2.3").unwrap()),
            ("v1.2.3-alpha.1.post.1.semver".to_string(), VersionObject::parse_semver("v1.2.3-alpha.1.post.1.semver").unwrap()),
            ("v1.2.3-alpha.2.post.1.semver".to_string(), VersionObject::parse_semver("v1.2.3-alpha.2.post.1.semver").unwrap()),
            ("v1.2.3-alpha.3.post.1.semver".to_string(), VersionObject::parse_semver("v1.2.3-alpha.3.post.1.semver").unwrap()),
        ],
        Some("v1.2.3".to_string()),
    )]
    #[case(
        "auto",
        vec![
            "v1".to_string(),
            "v1.2".to_string(),
            "v1.2.3".to_string(),
            "v1.2.4-alpha.1.post.1.semver".to_string(),
            "v1.2.4-alpha.2.post.1.semver".to_string(),
            "v1.2.4-alpha.3.post.1.semver".to_string(),
        ],
        vec![
            ("v1.2.3".to_string(), VersionObject::parse_semver("v1.2.3").unwrap()),
            ("v1.2.4-alpha.1.post.1.semver".to_string(), VersionObject::parse_semver("v1.2.4-alpha.1.post.1.semver").unwrap()),
            ("v1.2.4-alpha.2.post.1.semver".to_string(), VersionObject::parse_semver("v1.2.4-alpha.2.post.1.semver").unwrap()),
            ("v1.2.4-alpha.3.post.1.semver".to_string(), VersionObject::parse_semver("v1.2.4-alpha.3.post.1.semver").unwrap()),
        ],
        Some("v1.2.4-alpha.3.post.1.semver".to_string()),
    )]
    #[case(
        "auto",
        vec![
            "v1".to_string(),
            "v1.2".to_string(),
            "v1.2.3".to_string(),
            "v1.2.4-alpha.1.post.1.semver".to_string(),
            "v1.2.4-alpha.2.post.1.semver".to_string(),
        ],
        vec![
            ("v1.2.3".to_string(), VersionObject::parse_semver("v1.2.3").unwrap()),
            ("v1.2.4-alpha.1.post.1.semver".to_string(), VersionObject::parse_semver("v1.2.4-alpha.1.post.1.semver").unwrap()),
            ("v1.2.4-alpha.2.post.1.semver".to_string(), VersionObject::parse_semver("v1.2.4-alpha.2.post.1.semver").unwrap()),
        ],
        Some("v1.2.4-alpha.2.post.1.semver".to_string()),
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
        let filtered_tags = GitUtils::filter_only_valid_tags(&tags, format);

        assert_eq!(filtered_tags, expected_valid_tags);

        // Test find_max_version_tag with the filtered tags
        let actual_max_version_tag = GitUtils::find_max_version_tag(&filtered_tags).unwrap();
        assert_eq!(actual_max_version_tag, expected_max_version_tag);
    }
}
