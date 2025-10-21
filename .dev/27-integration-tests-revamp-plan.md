# Integration Tests Revamp Plan

## Current State

Integration tests are currently disabled for faster development. The existing `tests/integration_tests/version` folder contains tests that use too many git fixtures, causing slow test execution due to Docker overhead.

## Existing Codebase Assets

The codebase already provides excellent test utilities in `src/test_utils/`:

- **`ZervFixture`**: Complete Zerv object creation with chainable methods
- **`ZervVarsFixture`**: ZervVars creation with version components
- **`ZervSchemaFixture`**: Schema creation with tier presets
- **`GitRepoFixture`**: Git repository creation (tagged, with_distance, dirty)
- **`TestCommand`**: Command execution utilities

These existing fixtures eliminate the need to create new RON files and provide type-safe, maintainable test data creation.

## Problem Analysis

1. **Git Fixture Overuse**: Current tests create too many git repositories via Docker, making tests slow
2. **Inefficient Test Structure**: Tests don't follow the CLI argument structure, making them hard to maintain
3. **Missing Coverage**: Some CLI features may not be adequately tested
4. **Performance Issues**: Docker-based git tests are necessary but should be minimized

## Solution Strategy

### 1. Minimize Git Dependencies

- **Limit Git Tests**: Use ≤5 test cases that actually require git fixtures
- **Use Zerv RON Fixtures**: Convert git states to Zerv RON format and use `--source stdin` for most tests
- **Focus Git Tests**: Only test git-specific functionality (VCS detection, branch parsing, etc.)

### 2. Restructure Test Organization

Organize tests to mirror `VersionArgs` structure:

```
tests/integration_tests/version/
├── main/           # MainConfig tests
│   ├── sources/        # --source git/stdin (git tests here)
│   │   ├── stdin.rs        # --source stdin tests
│   │   └── git.rs          # Basic git integration (≤3 tests total)
│   ├── formats.rs      # --input-format, --output-format
│   ├── schemas.rs      # --schema, --schema-ron
│   ├── templates.rs    # --output-template
│   ├── directory.rs    # -C flag
│   └── combinations.rs # MainConfig combinations
├── overrides/      # OverridesConfig tests
│   ├── vcs.rs          # --tag-version, --distance, --dirty, etc.
│   ├── components.rs   # --major, --minor, --patch, etc.
│   ├── schema_components.rs # --core, --extra-core, --build
│   └── combinations.rs # OverridesConfig combinations
├── bumps/          # BumpsConfig tests
│   ├── field_based.rs  # --bump-major, --bump-minor, etc.
│   ├── schema_based.rs # --bump-core, --bump-extra-core, etc.
│   ├── context.rs      # --bump-context, --no-bump-context
│   └── combinations.rs # BumpsConfig combinations
└── combinations/   # Cross-module integration tests
    ├── override_bump.rs    # Overrides + Bumps
    ├── schema_override.rs  # Schema + Overrides
    └── full_workflow.rs   # Complete workflows
```

### 3. Test Strategy by Category

#### Main Config Tests (`main/`)

- **Focus**: Individual MainConfig options in isolation
- **Method**: Primarily stdin-based with ZervFixture
- **Git Usage**: Only in `sources/` subfolder (≤3 total git tests)
- **Scope**: Test each option independently, fix other args to defaults

#### Override Tests (`overrides/`)

- **Focus**: OverridesConfig options individually + related combinations
- **Method**: ZervFixture with single/multiple related overrides
- **Git Usage**: None (all via stdin)
- **Scope**: Test individual overrides + combinations within override category

#### Bump Tests (`bumps/`)

- **Focus**: BumpsConfig options individually + related combinations
- **Method**: ZervFixture with single/multiple related bumps
- **Git Usage**: None (all via stdin)
- **Scope**: Test individual bumps + combinations within bump category

#### Combination Tests (`combinations/`)

- **Focus**: Cross-module interactions and complex scenarios
- **Method**: ZervFixture with multiple options combined
- **Git Usage**: None (all via stdin)
- **Scope**: Test interactions between main/overrides/bumps

#### Source Tests (`main/sources/`)

- **Focus**: Source switching and git-specific functionality
- **Method**: stdin tests + minimal Docker git fixtures (≤5 total)
- **Coverage**: stdin source, basic git pipeline integration, source validation, input format validation

### 4. Implementation Plan

#### Phase 1: Backup and Setup ✅ COMPLETED

1. **Backup Current Tests** ✅

    ```bash
    mv tests/integration_tests/version tests/integration_tests/version_old_backup
    ```

2. **Enable Integration Tests** ✅
    - Uncommented code in `tests/integration.rs`:
        ```rust
        mod integration_tests;
        pub use integration_tests::*;
        ```
    - Commented out version module in `tests/integration_tests/mod.rs`:
        ```rust
        // pub mod version;  // Temporarily disabled during revamp
        ```
    - Ran `make test` and fixed one failing test in `cli_help.rs`
    - **Goal**: All integration tests pass except version command tests ✅
    - **Result**: 1954 tests pass with 91.96% coverage

3. **Create New Structure** ✅

    ```bash
    mkdir -p tests/integration_tests/version/{main/sources,overrides,bumps,combinations}
    ```

    - Directory structure created successfully
    - Ready for Phase 2 implementation

#### Phase 2: Implement Main Config Tests (`main/`) ✅ COMPLETED

- ✅ Created `tests/integration_tests/version/main/mod.rs`
- ✅ Implemented `sources/` tests:
    - `sources/stdin.rs`: 6 stdin tests using `ZervFixture` with `TestCommand.stdin()` (✅ PASSED)
    - `sources/git.rs`: 1 comprehensive git integration test with Docker gating (✅ PASSED)
- ✅ Enhanced `TestCommand` with `.stdin()` support for cleaner testing
- ✅ Refactored tests to use `rstest` for cleaner parameterized testing
- ✅ Enhanced `ZervFixture.with_vcs_data()` to accept `Option` types for better flexibility
- ✅ Implemented `formats.rs`: Comprehensive format conversion tests (30 tests)
- ✅ Implemented `schemas.rs`: Comprehensive schema tests (31 tests)
- ✅ Implemented `templates.rs`: Comprehensive template tests covering all helpers and edge cases (62 tests)
- ✅ Implemented `directory.rs`: Directory flag tests with Git integration and error handling (4 tests)
- ✅ Implemented `combinations.rs`: MainConfig option combinations (format + schema, template + format, etc.) (38 tests)
- **Result**: 210 tests passing (100% success rate) - 7 source tests + 30 format tests + 31 schema tests + 62 template tests + 4 directory tests + 38 combination tests + 38 other tests
- **Performance**: Tests run in <0.5 seconds without Docker

**MainConfig Tests Status:**

- ✅ `formats.rs`: Test `--input-format` (semver/pep440/auto) and `--output-format` (semver/pep440/zerv) combinations, format validation errors, error message consistency (✅ PASSED - 30 tests)
- ✅ `schemas.rs`: Test `--schema` (zerv-standard/zerv-calver) and `--schema-ron` (custom RON schema) options (✅ PASSED - 31 tests)
- ✅ `templates.rs`: Test `--output-template` with Handlebars template rendering, all helpers (sanitize, hash, prefix, timestamp, math), complex scenarios, edge cases (✅ PASSED - 62 tests)
- ✅ `directory.rs`: Test `-C` flag for changing working directory before execution (✅ PASSED - 4 tests: 2 Git integration + 2 error handling)
- ✅ `combinations.rs`: Test MainConfig option combinations (format + schema, template + format, etc.) (✅ PASSED - 38 tests)

#### Phase 3: Implement Override Tests (`overrides/`) 🔄 IN PROGRESS

- ✅ Created `tests/integration_tests/version/overrides/mod.rs`
- Implement individual OverridesConfig tests:
    - ✅ `vcs.rs`: --tag-version, --distance, --dirty, --clean, --current-branch, --commit-hash (37 tests total)
        - **Status**: Tests implemented with clean module structure and fixture helpers
        - **Test Results**: **35 passing ✅, 0 failing, 1 ignored (known bug)**
        - **Coverage**:
            - ✅ VCS field overrides correctly populate Zerv data structure fields
            - ✅ Template variables `{{bumped_branch}}` and `{{bumped_commit_hash}}` work correctly
            - ✅ Conflict detection works (--dirty/--no-dirty, --clean with --distance/--dirty)
            - ✅ Hash truncation to 7 characters works as expected
            - ✅ Distance and dirty overrides with zerv output format
        - **Ignored Test** (1 test - known bug):
            - `test_tag_version_and_distance`: Distance override doesn't affect tier calculation when combined with tag-version override
        - **Test Quality**: Tests follow new guidelines (module-level fixtures, `TestCommand::run_with_stdin`, rstest parameterization)
        - **Impact**: VCS overrides are fully functional except for one edge case (tag+distance tier calculation)
    - ❌ `components.rs`: --major, --minor, --patch individually
    - ❌ `schema_components.rs`: --core, --extra-core, --build individually
    - ❌ `schema_components.rs`: --core, --extra-core, --build individually
    - ❌ `combinations.rs`: Override combinations, conflicting options (clean vs distance/dirty), boolean flag behavior
- Use ZervFixture with stdin source for all tests
- Test and validate override functionality

#### Phase 4: Implement Bump Tests (`bumps/`)

- Create `tests/integration_tests/version/bumps/mod.rs`
- Implement individual BumpsConfig tests:
    - `field_based.rs`: --bump-major, --bump-minor individually
    - `schema_based.rs`: --bump-core, --bump-extra-core individually
    - `context.rs`: --bump-context, --no-bump-context individually
    - `combinations.rs`: Bump combinations
- Use ZervFixture with stdin source for all tests
- Test and validate bump functionality

#### Phase 5: Implement Cross-Module Combinations (`combinations/`)

- Create `tests/integration_tests/version/combinations/mod.rs`
- Implement cross-module interaction tests:
    - `override_bump.rs`: Overrides + Bumps interactions
    - `schema_override.rs`: Schema + Overrides interactions
    - `full_workflow.rs`: Complete multi-option workflows
- Use ZervFixture with complex scenarios
- Re-enable version module in `tests/integration_tests/mod.rs`
- Run full test suite and validate performance targets
- Test and validate complete integration test system

### 5. Performance Targets

- **Total Test Time**: <30 seconds for full integration test suite
- **Git Tests**: ≤3 test cases, <10 seconds total
- **RON Tests**: Majority of tests, <20 seconds total
- **Parallel Execution**: Enable parallel test execution where possible

### 6. Coverage Goals

Ensure comprehensive coverage of:

- All CLI arguments and combinations
- Error conditions and validation
- Format conversions (SemVer ↔ PEP440 ↔ Zerv)
- Schema behavior across different states
- Override and bump interactions
- Template rendering edge cases

### 7. Maintenance Strategy

- **Fixture Management**: Leverage existing ZervFixture for consistency
- **Test Organization**: Mirror CLI structure for easy maintenance
- **Documentation**: Document test patterns and fixture usage
- **CI Integration**: Ensure tests run efficiently in CI/CD pipeline

### 8. Test Code Quality Guidelines

- **Use `TestCommand::run_with_stdin`**: For simple stdin tests that only need stdout output, use the convenience method:

    ```rust
    let output = TestCommand::run_with_stdin("version --source stdin --output-format semver", zerv_ron);
    assert_eq!(output, "1.2.3");
    ```

- **Module-level fixture helpers**: Create clear, reusable fixture functions at module level (similar to `tests/integration_tests/version/main/schemas.rs`):

    ```rust
    // Test constants at module level
    const TEST_BRANCH: &str = "feature.branch";
    const TEST_COMMIT_HASH: &str = "abc123def456";

    // Helper functions for common fixtures
    fn create_tier_1_fixture(version: (u64, u64, u64)) -> ZervFixture {
        ZervFixture::new()
            .with_version(version.0, version.1, version.2)
            .with_standard_tier_1()
    }
    ```

- **Use rstest parameterization**: Leverage `#[rstest]` with `#[case]` for testing multiple scenarios:

    ```rust
    #[rstest]
    #[case::basic("1.0.0", "1.0.0")]
    #[case::prerelease("2.0.0-beta.1", "2.0.0-beta.1")]
    fn test_override(#[case] input: &str, #[case] expected: &str) {
        // Test implementation
    }
    ```

- **Clear test structure**: Organize tests with descriptive module names and group related tests together

## Implementation Steps

1. **Phase 1**: Backup and setup ✅ **COMPLETED**
2. **Phase 2**: Implement main config tests ✅ **COMPLETED**
3. **Phase 3**: Implement override tests 🔄 **IN PROGRESS**
4. **Phase 4**: Implement bump tests
5. **Phase 5**: Implement cross-module combinations and final integration

## Success Criteria

- ✅ Integration tests run in <30 seconds
- ✅ ≤3 git-dependent test cases
- ✅ Comprehensive CLI argument coverage
- ✅ Test structure mirrors VersionArgs organization
- ✅ RON fixtures enable fast, reliable testing
- ✅ Easy to add new tests and maintain existing ones
