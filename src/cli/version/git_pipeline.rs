use std::path::Path;

use super::args::VersionArgs;
use super::zerv_draft::ZervDraft;
use crate::cli::utils::format_handler::InputFormatHandler;
use crate::error::ZervError;
use crate::pipeline::vcs_data_to_zerv_vars;

/// Process git source and return a ZervDraft object
pub fn process_git_source(work_dir: &Path, args: &VersionArgs) -> Result<ZervDraft, ZervError> {
    // Get git VCS data
    // If directory was specified via -C, only look in that directory (depth 0)
    // If no directory specified, allow unlimited depth search
    let max_depth = if args.input.directory.is_some() {
        Some(0)
    } else {
        None
    };
    let vcs_data = crate::vcs::detect_vcs_with_limit(work_dir, max_depth)?.get_vcs_data()?;

    // Parse git tag with input format if available and validate it
    if let Some(ref tag_version) = vcs_data.tag_version {
        let _parsed_version =
            InputFormatHandler::parse_version_string(tag_version, &args.input.input_format)?;
        // Validation passed - the tag is in a valid format
    }

    // Convert VCS data to ZervVars
    let vars = vcs_data_to_zerv_vars(vcs_data)?;

    // Return ZervDraft without schema (git source)
    Ok(ZervDraft::new(vars, None))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{
        GitRepoFixture,
        VersionArgsFixture,
        should_run_docker_tests,
    };

    #[test]
    fn test_process_git_source_basic() {
        if !should_run_docker_tests() {
            return; // Skip when `ZERV_TEST_DOCKER` are disabled
        }

        // Create a basic git repository with a tag
        let fixture = GitRepoFixture::tagged("v1.2.3").expect("Failed to create git fixture");

        // Create basic version args using fixture
        let args = VersionArgsFixture::new()
            .with_directory(&fixture.path().to_string_lossy())
            .build();

        // Process the git source
        let result = process_git_source(fixture.path(), &args);

        // Should succeed and return a ZervDraft object
        assert!(
            result.is_ok(),
            "process_git_source should succeed for basic git repo"
        );

        let draft = result.unwrap();

        // Verify basic structure
        assert!(draft.vars.major.is_some(), "Should have major version");
        assert!(draft.vars.minor.is_some(), "Should have minor version");
        assert!(draft.vars.patch.is_some(), "Should have patch version");

        // Verify version values match the tag
        assert_eq!(draft.vars.major, Some(1), "Major version should be 1");
        assert_eq!(draft.vars.minor, Some(2), "Minor version should be 2");
        assert_eq!(draft.vars.patch, Some(3), "Patch version should be 3");

        // Verify no schema initially (git source)
        assert!(
            draft.schema.is_none(),
            "Git source should not have schema initially"
        );
    }
}
