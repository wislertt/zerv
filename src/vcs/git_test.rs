// Tests for git utility functions

use crate::vcs::git_utils::GitUtils;

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::version::VersionObject;

    #[rstest]
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
    )]
    fn test_filter_only_valid_tags(
        #[case] format: &str,
        #[case] tags: Vec<String>,
        #[case] expected_valid_tags: Vec<(String, VersionObject)>,
    ) {
        let result = GitUtils::filter_only_valid_tags(&tags, format).unwrap();

        assert_eq!(result.len(), expected_valid_tags.len());

        // Compare both tags and version objects
        for (expected_tag, expected_version) in expected_valid_tags {
            assert!(result.iter().any(|(actual_tag, actual_version)| {
                actual_tag == &expected_tag && actual_version == &expected_version
            }));
        }
    }
}
