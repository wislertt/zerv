use super::parse_version_from_tag::parse_version_from_tag;
use crate::error::ZervError;
use crate::vcs::VcsData;
use crate::version::ZervVars;

/// Convert VCS data to ZervVars
pub fn vcs_data_to_zerv_vars(vcs_data: VcsData) -> Result<ZervVars, ZervError> {
    tracing::debug!("Converting VCS data to Zerv variables");
    tracing::debug!("VCS data: {:?}", vcs_data);

    // Parse version from tag_version
    let version = if let Some(ref tag_version) = vcs_data.tag_version {
        parse_version_from_tag(tag_version, None).ok_or_else(|| {
            tracing::error!("Failed to parse version from tag: {}", tag_version);
            ZervError::InvalidFormat(format!("Failed to parse version from tag: {tag_version}"))
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
    vars.bumped_commit_hash = Some(vcs_data.commit_hash);
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
    #[case::semver(get_real_semver_vcs_data(), (1, 2, 3), "SemVer")]
    #[case::pep440(get_real_pep440_vcs_data(), (2, 0, 1), "PEP440")]
    fn test_vcs_data_to_zerv_vars_real_formats(
        #[case] vcs_data: &VcsData,
        #[case] expected_version: (u64, u64, u64),
        #[case] format_name: &str,
    ) {
        if !should_run_docker_tests() {
            return;
        }

        let vars = vcs_data_to_zerv_vars(vcs_data.clone())
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
    }

    #[test]
    fn test_vcs_data_to_zerv_vars_no_tag() {
        let vcs_data = VcsData {
            tag_version: None,
            commit_hash: "abc1234".to_string(),
            ..Default::default()
        };
        let result = vcs_data_to_zerv_vars(vcs_data);
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
        let result = vcs_data_to_zerv_vars(vcs_data);

        match result {
            Err(ZervError::InvalidFormat(msg)) => {
                assert_eq!(
                    msg,
                    format!("Failed to parse version from tag: {invalid_tag}")
                );
            }
            _ => panic!("Expected InvalidFormat error for tag: {invalid_tag}"),
        }
    }
}
