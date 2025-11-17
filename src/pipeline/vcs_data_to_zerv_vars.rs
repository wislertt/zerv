use crate::error::ZervError;
use crate::vcs::VcsData;
use crate::version::{
    VersionObject,
    ZervVars,
};

/// Convert VCS data to ZervVars
pub fn vcs_data_to_zerv_vars(vcs_data: VcsData, input_format: &str) -> Result<ZervVars, ZervError> {
    tracing::debug!(
        "Converting VCS data to Zerv variables with input format: {}",
        input_format
    );
    tracing::debug!("VCS data: {:?}", vcs_data);

    // Parse version from tag_version using the provided input format
    let version = if let Some(ref tag_version) = vcs_data.tag_version {
        VersionObject::parse_with_format(tag_version, input_format).map_err(|e| {
            tracing::error!(
                "Failed to parse version from tag: {} with format {}: {}",
                tag_version,
                input_format,
                e
            );
            e
        })?
    } else {
        tracing::warn!("No tag version found in VCS data");
        return Err(ZervError::NoTagsFound);
    };

    let mut vars: ZervVars = version.into();

    // VCS-specific fields
    vars.distance = Some(vcs_data.distance as u64);
    vars.bumped_branch = vcs_data.current_branch;
    vars.dirty = Some(vcs_data.is_dirty);
    vars.bumped_commit_hash = Some(format!(
        "{}{}",
        vcs_data.commit_hash_prefix, vcs_data.commit_hash
    ));
    // Set last_commit_hash if available from tag_commit_hash
    vars.last_commit_hash = vcs_data
        .tag_commit_hash
        .map(|hash| format!("{}{}", vcs_data.commit_hash_prefix, hash));
    vars.bumped_timestamp = Some(vcs_data.commit_timestamp as u64);
    vars.last_timestamp = vcs_data.tag_timestamp.map(|t| t as u64);

    tracing::debug!("VCS data conversion complete");
    Ok(vars)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::{
        get_real_pep440_vcs_data,
        get_real_semver_vcs_data,
        should_run_docker_tests,
    };

    #[rstest]
    #[case::semver(get_real_semver_vcs_data(), (1, 2, 3), "SemVer", "auto")]
    #[case::pep440(get_real_pep440_vcs_data(), (2, 0, 1), "PEP440", "auto")]
    fn test_vcs_data_to_zerv_vars_real_formats(
        #[case] vcs_data: &VcsData,
        #[case] expected_version: (u64, u64, u64),
        #[case] format_name: &str,
        #[case] input_format: &str,
    ) {
        if !should_run_docker_tests() {
            return;
        }

        let vars = vcs_data_to_zerv_vars(vcs_data.clone(), input_format)
            .unwrap_or_else(|_| panic!("Failed to convert {format_name} VCS data to ZervVars"));

        assert_eq!(
            (vars.major, vars.minor, vars.patch),
            (
                Some(expected_version.0),
                Some(expected_version.1),
                Some(expected_version.2)
            ),
            "Version mismatch for {format_name}"
        );
        assert_eq!(
            vars.distance,
            Some(1),
            "Distance should be 1 commit after tag for {format_name}"
        );
        assert!(
            vars.bumped_commit_hash.is_some(),
            "Commit hash should be present for {format_name}"
        );
        assert!(
            vars.last_timestamp.is_some(),
            "Tag timestamp should be present for {format_name}"
        );
        assert!(
            vars.last_commit_hash.is_some(),
            "Last commit hash should be present for {format_name}"
        );
    }

    #[test]
    fn test_vcs_data_to_zerv_vars_no_tag() {
        let vcs_data = VcsData {
            tag_version: None,
            commit_hash: "abc1234".to_string(),
            ..Default::default()
        };
        let result = vcs_data_to_zerv_vars(vcs_data, "auto");
        assert!(result.is_err());

        match result {
            Err(ZervError::NoTagsFound) => {
                assert_eq!(
                    ZervError::NoTagsFound.to_string(),
                    "No version tags found in git repository"
                );
            }
            _ => panic!("Expected NoTagsFound error"),
        }
    }

    #[test]
    fn test_vcs_data_to_zerv_vars_with_tag_commit_hash() {
        let vcs_data = VcsData {
            tag_version: Some("v1.2.3".to_string()),
            distance: 5,
            commit_hash: "def456789".to_string(),
            commit_hash_prefix: "g".to_string(),
            tag_commit_hash: Some("abc123def456".to_string()),
            current_branch: Some("main".to_string()),
            commit_timestamp: 1703123456,
            tag_timestamp: Some(1703000000),
            is_dirty: false,
            is_shallow: false,
        };

        let vars =
            vcs_data_to_zerv_vars(vcs_data, "auto").expect("should convert vcs data to vars");

        // Check that last_commit_hash is set with prefix
        assert_eq!(
            vars.last_commit_hash,
            Some("gabc123def456".to_string()),
            "Last commit hash should be set with g prefix"
        );

        // Check other fields are also set correctly
        assert_eq!(vars.distance, Some(5));
        assert_eq!(vars.bumped_branch, Some("main".to_string()));
        assert_eq!(vars.dirty, Some(false));
        assert_eq!(vars.bumped_commit_hash, Some("gdef456789".to_string()));
        assert_eq!(vars.bumped_timestamp, Some(1703123456));
        assert_eq!(vars.last_timestamp, Some(1703000000));
    }

    #[test]
    fn test_vcs_data_to_zerv_vars_without_tag_commit_hash() {
        let vcs_data = VcsData {
            tag_version: Some("v1.2.3".to_string()),
            distance: 0,
            commit_hash: "abc123def456".to_string(),
            commit_hash_prefix: "g".to_string(),
            tag_commit_hash: None, // No tag commit hash
            current_branch: Some("main".to_string()),
            commit_timestamp: 1703123456,
            tag_timestamp: Some(1703000000),
            is_dirty: false,
            is_shallow: false,
        };

        let vars =
            vcs_data_to_zerv_vars(vcs_data, "auto").expect("should convert vcs data to vars");

        // Check that last_commit_hash is None when tag_commit_hash is None
        assert_eq!(
            vars.last_commit_hash, None,
            "Last commit hash should be None when tag_commit_hash is None"
        );

        // Other fields should still be set
        assert_eq!(vars.distance, Some(0));
        assert_eq!(vars.bumped_branch, Some("main".to_string()));
        assert_eq!(vars.bumped_commit_hash, Some("gabc123def456".to_string()));
    }

    #[rstest]
    #[case("invalid")]
    #[case("")]
    #[case("abc.def.ghi")]
    #[case("invalid-tag-format")]
    #[case("not-a-version")]
    #[case("v")]
    #[case("1.2.v")]
    fn test_vcs_data_to_zerv_vars_invalid_tag_formats(#[case] invalid_tag: &str) {
        let vcs_data = VcsData {
            tag_version: Some(invalid_tag.to_string()),
            commit_hash: "abc1234".to_string(),
            ..Default::default()
        };
        let result = vcs_data_to_zerv_vars(vcs_data, "auto");

        match result {
            Err(ZervError::InvalidFormat(msg)) => {
                // Check that the error message contains the tag name
                assert!(
                    msg.contains(invalid_tag),
                    "Error message should contain tag: {}",
                    msg
                );
                // Check that it contains format information
                assert!(
                    msg.contains("auto") || msg.contains("semver") || msg.contains("pep440"),
                    "Error message should contain format information: {}",
                    msg
                );
            }
            Err(ZervError::InvalidVersion(msg)) => {
                // Check that the error message contains the tag name for auto-detection failures
                assert!(
                    msg.contains(invalid_tag),
                    "Error message should contain tag: {}",
                    msg
                );
            }
            _ => panic!(
                "Expected InvalidFormat or InvalidVersion error for tag: {invalid_tag}, got: {:?}",
                result
            ),
        }
    }
}
