# Implementation Plan

- [ ]   1. Reorganize error enum with clear groupings
    - Add comment blocks to group related error variants in `src/error.rs`
    - Improve code readability without changing functionality
    - _Requirements: 5.1, 5.2_

- [ ]   2. Fix VCS error message in NoTagsFound display
    - Update `ZervError::NoTagsFound` display message to "No version tags found in git repository"
    - Replace generic message with source-aware message
    - _Requirements: 1.1, 1.2, 1.3_

- [ ]   3. Implement git error translation in GitVcs
    - Add `translate_command_error` method to handle `std::io::Error` from git command execution
    - Add `translate_git_error` method to parse stderr and map common git errors
    - Handle "fatal: ambiguous argument 'HEAD'" → "No commits found in git repository"
    - Handle "not a git repository" → source-aware VCS not found error
    - Handle "Permission denied" → clear permission error
    - Handle git command not found → installation guidance
    - _Requirements: 3.1, 3.2, 3.3, 4.1, 4.2, 4.3_

- [ ]   4. Update VCS detection error messages
    - Modify `detect_vcs` function to return source-aware error message
    - Modify `find_vcs_root` function to return "Not in a git repository (--source git)"
    - Replace generic "VCS not found" messages with specific source information
    - _Requirements: 2.1, 2.2, 2.3_

- [ ]   5. Fix pipeline error usage
    - Replace `ZervError::Io("No version tag found in VCS data")` with `ZervError::NoTagsFound`
    - Remove inappropriate use of IO error for VCS-related problems
    - Update `vcs_data_to_zerv_vars` function in `src/pipeline/vcs_data_to_zerv_vars.rs`
    - _Requirements: 1.1, 1.2, 1.3_

- [ ]   6. Update git command execution in GitVcs
    - Modify `run_git_command` method to use new error translation functions
    - Replace direct `ZervError::CommandFailed` creation with translated errors
    - Ensure all git command failures go through translation layer
    - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [ ]   7. Add comprehensive error message tests
    - Create unit tests for all new error message formats
    - Test git error pattern matching and translation
    - Test VCS detection error messages
    - Test pipeline error handling
    - Verify error messages match requirements exactly
    - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 3.1, 3.2, 3.3_

- [ ]   8. Create integration tests for error scenarios
    - Add tests to existing `tests/integration_tests/version/errors.rs` file
    - Test "no version tags" scenario in git repository
    - Test "not in git repository" scenario
    - Test "empty git repository" scenario (no commits)
    - Test git command permission denied scenario (if feasible)
    - Test git command not installed scenario (if feasible)
    - Verify end-to-end error message flow from CLI to user
    - _Requirements: 1.1, 2.1, 3.1, 4.1, 4.2_

- [ ]   9. Update error equality and debug implementations
    - Ensure `PartialEq` implementation works correctly with updated error messages
    - Verify debug output is helpful for development
    - Update any error-related test cases that depend on specific message formats
    - _Requirements: 5.1, 5.2_

- [ ]   10. Validate error message consistency
    - Review all error messages for consistent terminology
    - Ensure source information is included where appropriate
    - Verify actionable guidance is provided in error messages
    - Test error messages with real git repositories in various states
    - _Requirements: 5.1, 5.2, 5.3_
