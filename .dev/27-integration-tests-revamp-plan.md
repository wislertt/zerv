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
- **Coverage**: stdin source, basic git pipeline integration

### 4. Implementation Plan

#### Phase 1: Backup and Setup

1. **Backup Current Tests**

    ```bash
    mv tests/integration_tests/version tests/integration_tests/version_old_backup
    ```

2. **Enable Integration Tests**
    - Uncomment code in `tests/integration.rs`:
        ```rust
        mod integration_tests;
        pub use integration_tests::*;
        ```
    - Comment out version module in `tests/integration_tests/mod.rs`:
        ```rust
        // pub mod version;  // Temporarily disabled during revamp
        ```
    - Run `make test` and fix any errors (likely test updates, not implementation)
    - **Goal**: All integration tests pass except version command tests
    - **Note**: If implementation changes needed, ask first before modifying

3. **Create New Structure**
    ```bash
    mkdir -p tests/integration_tests/version/{main/sources,overrides,bumps,combinations}
    ```

#### Phase 2: Implement Main Config Tests (`main/`)

- Create `tests/integration_tests/version/main/mod.rs`
- Implement individual MainConfig tests:
    - `sources/`: git vs stdin (≤3 git tests)
    - `formats.rs`: --input-format, --output-format individually
    - `schemas.rs`: --schema, --schema-ron individually
    - `templates.rs`: --output-template individually
    - `directory.rs`: -C flag individually
    - `combinations.rs`: MainConfig option combinations
- Use direct imports: `use zerv::test_utils::{ZervFixture, GitRepoFixture};`
- Test and validate main config functionality

#### Phase 3: Implement Override Tests (`overrides/`)

- Create `tests/integration_tests/version/overrides/mod.rs`
- Implement individual OverridesConfig tests:
    - `vcs.rs`: --tag-version, --distance, --dirty individually
    - `components.rs`: --major, --minor, --patch individually
    - `schema_components.rs`: --core, --extra-core, --build individually
    - `combinations.rs`: Override combinations
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

## Implementation Steps

1. **Phase 1**: Backup and setup
2. **Phase 2**: Implement main config tests
3. **Phase 3**: Implement override tests
4. **Phase 4**: Implement bump tests
5. **Phase 5**: Implement cross-module combinations and final integration

## Success Criteria

- ✅ Integration tests run in <30 seconds
- ✅ ≤3 git-dependent test cases
- ✅ Comprehensive CLI argument coverage
- ✅ Test structure mirrors VersionArgs organization
- ✅ RON fixtures enable fast, reliable testing
- ✅ Easy to add new tests and maintain existing ones
