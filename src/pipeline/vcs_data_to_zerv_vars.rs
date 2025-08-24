use super::parse_version_from_tag::parse_version_from_tag;
use crate::error::ZervError;
use crate::vcs::VcsData;
use crate::version::ZervVars;

/// Convert VCS data to ZervVars
pub fn vcs_data_to_zerv_vars(vcs_data: VcsData) -> Result<ZervVars, ZervError> {
    // Parse version from tag_version
    let version = if let Some(ref tag_version) = vcs_data.tag_version {
        parse_version_from_tag(tag_version, None).ok_or_else(|| {
            ZervError::Io(std::io::Error::other("Failed to parse version from tag"))
        })?
    } else {
        return Err(ZervError::Io(std::io::Error::other("No version tag found")));
    };

    let mut vars: ZervVars = version.into();

    // VCS-specific fields
    vars.distance = Some(vcs_data.distance as u64);
    vars.current_branch = vcs_data.current_branch;
    vars.dirty = Some(vcs_data.is_dirty);
    vars.current_commit_hash = Some(vcs_data.commit_hash_short);
    vars.tag_timestamp = vcs_data.tag_timestamp.map(|t| t as u64);

    Ok(vars)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{
        get_real_pep440_vcs_data, get_real_semver_vcs_data, should_run_docker_tests,
    };

    #[test]
    fn test_vcs_data_to_zerv_vars_real_semver() {
        if !should_run_docker_tests() {
            return;
        }
        let vcs_data = get_real_semver_vcs_data().clone();
        let vars = vcs_data_to_zerv_vars(vcs_data).unwrap();

        assert_eq!(
            (vars.major, vars.minor, vars.patch),
            (Some(1), Some(2), Some(3))
        );
        assert_eq!(vars.distance, Some(1)); // 1 commit after tag
        assert!(vars.current_commit_hash.is_some());
        assert!(vars.tag_timestamp.is_some());
    }

    #[test]
    fn test_vcs_data_to_zerv_vars_real_pep440() {
        if !should_run_docker_tests() {
            return;
        }
        let vcs_data = get_real_pep440_vcs_data().clone();
        let vars = vcs_data_to_zerv_vars(vcs_data).unwrap();

        assert_eq!(
            (vars.major, vars.minor, vars.patch),
            (Some(2), Some(0), Some(1))
        );
        assert_eq!(vars.distance, Some(1)); // 1 commit after tag
        assert!(vars.current_commit_hash.is_some());
        assert!(vars.tag_timestamp.is_some());
    }

    #[test]
    fn test_vcs_data_to_zerv_vars_no_tag() {
        let vcs_data = VcsData {
            tag_version: None,
            ..Default::default()
        };
        let result = vcs_data_to_zerv_vars(vcs_data);
        assert!(result.is_err());
    }
}
