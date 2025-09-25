# Implementation Plan

- [x]   1. Create fuzzy boolean parser for CLI arguments
    - Implement `FuzzyBool` struct with `FromStr` trait in `src/cli/utils/fuzzy_bool.rs`
    - Support true/false, t/f, yes/no, y/n, 1/0, on/off (case-insensitive)
    - Write comprehensive unit tests for all boolean value combinations
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [x]   2. Enhance CLI argument structure with new options
    - Add VCS override fields to `VersionArgs` in `src/cli/version.rs`
    - Add `tag_version`, `distance`, `dirty`, `clean`, `current_branch`, `commit_hash` fields
    - Update `input_format` field with proper default value
    - Add `output_template` and `output_prefix` fields for future extension
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

- [x]   3. Implement input format handler for version string parsing
    - Create `InputFormatHandler` in `src/cli/utils/format_handler.rs`
    - Implement `parse_version_string()` method with format-specific parsing
    - Support semver, pep440, and auto-detection modes
    - Implement `parse_stdin()` method for Zerv RON parsing from stdin
    - Add comprehensive error handling with format-specific messages
    - _Requirements: 5.1, 5.2, 5.3, 6.1, 6.2, 6.3, 6.4_

- [x]   4. Create VCS override processor with conflict validation
    - Implement `VcsOverrideProcessor` in `src/cli/utils/vcs_override.rs`
    - Add `apply_overrides()` method to merge CLI overrides with VCS data
    - Implement `validate_override_conflicts()` to detect conflicting options
    - Handle `--clean` flag conflicts with `--distance` and `--dirty`
    - Add support for parsing `--tag-version` with input format
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7_

- [x]   5. Enhance error handling with source-aware messages
    - Add new error variants to `ZervError` enum in `src/error.rs`
    - Add `UnknownSource`, `ConflictingOptions`, `StdinError`, `BooleanParseError`
    - Update git error translation in `GitVcs` to use source-aware messages
    - Implement user-friendly error messages for all new error cases
    - Add comprehensive error message tests
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7_

- [x]   6. Implement enhanced pipeline orchestrator
    - Update `run_version_pipeline()` in `src/cli/version.rs`
    - Add source-specific processing for git vs stdin inputs
    - Integrate input format parsing for git tags and overrides
    - Handle Zerv RON parsing from stdin with override application
    - Add proper error handling and validation throughout pipeline
    - _Requirements: 1.1, 1.2, 1.3, 5.4, 5.5_

- [x]   7. Add comprehensive input validation for stdin
    - Implement stdin format detection and validation
    - Add specific error messages for simple version strings vs RON format
    - Validate Zerv RON structure and required fields
    - Provide helpful suggestions for incorrect usage patterns
    - Add line/column error information for RON parsing failures
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [x]   8. Create comprehensive unit tests for new components
    - Write tests for `FuzzyBool` parser with all supported formats
    - Test `InputFormatHandler` with valid and invalid inputs
    - Test `VcsOverrideProcessor` conflict detection and application
    - Test enhanced error handling and message formatting
    - Test stdin parsing with various input formats and error cases
    - _Requirements: All requirements validation_

- [x]   9. Implement integration tests for end-to-end workflows
    - Test git source with various override combinations
    - Test stdin source with Zerv RON input and overrides
    - Test piping workflows between multiple Zerv commands
    - Test error scenarios with real git repositories
    - Test performance requirements with large repositories
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 10.1, 10.2, 10.3, 10.4, 10.5_

- [x]   10. Add CLI help and documentation updates
    - Update command help text with new options and examples
    - Add usage examples for override options and piping workflows
    - Document input format behavior and supported values
    - Add error message consistency validation
    - Ensure backward compatibility with existing command patterns
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6_

- [ ]   11. Implement output formatting enhancements
    - Add support for output prefix option
    - Ensure clean single-line output for all formats
    - Add template support infrastructure for future extension
    - Validate output format consistency across all scenarios
    - Test output with various version states and overrides
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6_

- [ ]   12. Add performance optimization and validation
    - Optimize git command execution to minimize calls
    - Add efficient RON parsing for large inputs
    - Implement memory usage optimization for large repositories
    - Add performance benchmarks and validation tests
    - Ensure sub-100ms response time for typical repositories
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_

- [ ]   13. Comprehensive testing and validation
    - Run full test suite with Docker and native git implementations
    - Test on multiple platforms (Linux, macOS, Windows via CI)
    - Validate error message consistency across all scenarios
    - Test backward compatibility with existing usage patterns
    - Perform end-to-end validation of all requirements
    - _Requirements: All requirements final validation_
