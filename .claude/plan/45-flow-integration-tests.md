# Create Integration Tests for "zerv flow" Command

## Status

**In Progress** - Steps 1-3 completed, Step 4.1-4.4 ready for implementation

## Priority

**High** - Essential for comprehensive CLI testing coverage

## Context

The existing `pipeline.rs` test uses `FlowTestScenario` which tests the flow pipeline directly using Rust function calls. We need integration tests that test the complete "zerv flow" command through the CLI interface using string-based command line arguments, similar to how users actually use the tool.

## Goals

1. Create integration tests for "zerv flow" command that test the complete CLI interface
2. Test flow command behavior with various output formats and options
3. Test different git scenarios (trunk-based, GitFlow) using command-line interface
4. Ensure the integration tests are similar to but separate from existing pipeline tests
5. Follow existing integration test patterns used for "zerv version" command

## Implementation Plan

### Step 1: Create Test Structure ✅ **COMPLETED**

- ✅ Create `tests/integration_tests/flow/mod.rs` - Main flow integration test module
- ✅ Create `tests/integration_tests/flow/main/` subdirectory for main flow command tests
- ✅ Create `tests/integration_tests/flow/scenarios/` subdirectory for complex workflow tests
- ✅ Create `tests/integration_tests/flow/args.rs` for flow-specific test utilities
- ✅ Update `tests/integration_tests/mod.rs` to include flow module

### Step 2: Create FlowTestUtils Structure ✅ **COMPLETED**

- ✅ Create `FlowIntegrationTestScenario` with the same fluent API as `FlowTestScenario` but for CLI integration tests
- ✅ Implement the same builder pattern methods: `create_tag`, `create_branch`, `checkout`, `commit`, `make_dirty`, `merge_branch`
- ✅ Add the same expectation methods: `expect_version(semver, pep440)` and `expect_schema_variants(test_cases)`
- ✅ The key difference: `expect_version` and `expect_schema_variants` will internally use CLI string arguments via `TestCommand` instead of calling `run_flow_pipeline` directly
- ✅ Reuse existing `GitRepoFixture` and `TestCommand` from test utilities
- ✅ Create `scenarios/test_utils.rs` that reuses utilities from `src/cli/flow/test_utils.rs`:
    - ✅ Re-implement `SchemaTestCase`, `SchemaTestExtraCore`, `SchemaTestBuild` structs
    - ✅ Re-implement `create_full_schema_test_cases()` and `create_base_schema_test_cases()` functions
    - ✅ Re-implement `expect_branch_hash()` function for predictable hash generation
    - ✅ Add CLI-specific test functions: `test_flow_pipeline_with_fixture()`, `test_flow_pipeline_with_fixture_and_schema_opt()`
- ✅ Add `FlowTestResult` with assertion methods: `assert_success()`, `assert_failure()`, `assert_stdout_eq()`, etc.
- ✅ Include proper Docker test gating using `if !should_run_docker_tests() { return; }`
- ✅ Ensure all code passes `make lint` and integration tests

### Step 3: Basic Flow Command Tests (using stdin input) ✅ **COMPLETED**

- ✅ Test basic "zerv flow" command with default settings using zerv stdin input
- ✅ Test "zerv flow" with different output formats (`--output-format semver`, `--output-format pep440`) using stdin
- ✅ Test "zerv flow" with schema options (`--schema standard`) using stdin
- ✅ Test "zerv flow" with pre-release handling (alpha, beta, rc) using stdin
- ✅ Test "zerv flow" with branch-based scenarios (main, feature/, develop) using stdin
- ✅ Test "zerv flow" error handling (empty stdin, invalid RON format, invalid options)
- ✅ **SOLVED**: All 15 basic flow command tests passing with 100% success rate

### Step 4: Workflow Scenario Tests (using git repositories)

#### Step 4.1: Trunk-Based Workflow - Foundation Tests

**Reference Unit Test**: `src/cli/flow/pipeline.rs::test_trunk_based_development_flow()`

**Convert First Step**: Initial setup on main branch with v1.0.0 tag
**Reference Unit Test Code** (lines 60-66):

```rust
// Step 1: Initial commit on main with v1.0.0
let scenario = FlowTestScenario::new()
    .expect("Failed to create test scenario")
    .create_tag("v1.0.0")
    .expect_version("1.0.0", "1.0.0")
    .expect_schema_variants(create_base_schema_test_cases("1.0.0", "main"));
```

**Integration Test Conversion**:

- Convert from: `FlowTestScenario::new().create_tag("v1.0.0").expect_version("1.0.0", "1.0.0")`
- To integration test: `FlowIntegrationTestScenario::new().create_tag("v1.0.0").expect_version("1.0.0", "1.0.0")`
- Use actual git repositories through CLI (not unit test internals)
- Test main branch development scenarios (tags, commits, clean working directory)
- Test simple branch creation and switching scenarios
- Test basic CLI flow command execution with git repositories
- Verify integration test matches unit test behavior exactly
- Add Docker test gating: `if !should_run_docker_tests() { return; }`

#### Step 4.2: Trunk-Based Workflow - Advanced Scenarios

- Test complex branch switching and merging scenarios through CLI
- Test dirty working directory scenarios through CLI
- Test multiple commit scenarios and version progression
- Test branch cleanup and tag management scenarios
- Test edge cases and error conditions in trunk-based workflows

#### Step 4.3: GitFlow Workflow - Foundation Tests

- Test GitFlow development workflow scenarios using actual git repositories
- Test feature branch creation, development, and merge scenarios
- Test release branch scenarios and version tagging
- Test hotfix branch workflows and emergency releases

#### Step 4.4: GitFlow Workflow - Advanced Scenarios

- Test complex GitFlow branching patterns and release management
- Test multi-developer collaboration scenarios through CLI
- Test GitFlow integration with flow command pre-release detection
- Test GitFlow edge cases and error handling scenarios

### Step 5: Error Handling Tests (using stdin input)

- Test flow command with invalid arguments using stdin input
- Test flow command with conflicting options using stdin input
- Test flow command error messages are helpful
- Test flow command with invalid schema formats using stdin

### Step 6: Template and Output Tests (using stdin input)

- Test custom output templates with flow command using stdin input
- Test output prefixes and suffixes using stdin input
- Test different schema presets with flow command using stdin input
- Verify template variables are correctly resolved using stdin

## Testing Strategy

### Test Organization

```
tests/integration_tests/flow/
├── mod.rs                    # Main module exports
├── args.rs                   # Flow-specific test utilities
├── main/
│   ├── mod.rs               # Main flow command tests (using git)
│   ├── basic_commands.rs    # Basic flow command tests (using git)
│   ├── output_formats.rs    # Output format tests (using git)
│   ├── schema_options.rs    # Schema option tests (using git)
│   └── error_handling.rs    # Error handling tests (using git)
└── scenarios/
    ├── mod.rs               # Complex workflow tests (using git)
    ├── test_utils.rs        # Reused test utilities for CLI integration
    ├── trunk_based.rs       # Trunk-based workflow tests (using git)
    └── gitflow.rs           # GitFlow workflow tests (using git)
```

### Integration Test Patterns

**All Flow Tests (using git repositories - flow command requires git context):**

```rust
// Basic functionality, schema options, output formats, error handling
// All use git repositories since flow command requires git branch context
let scenario = FlowIntegrationTestScenario::new()?
    .create_tag("v1.0.0")
    .expect_version("1.0.0", "1.0.0");

// Complex workflow scenarios (trunk-based, GitFlow)
// Same API as existing FlowTestScenario but uses CLI internally
let scenario = FlowIntegrationTestScenario::new()?
    .create_tag("v1.0.0")
    .create_branch("feature/test")
    .checkout("feature/test")
    .commit()
    .expect_version("1.0.1-alpha.XXXXX.post.1+feature.test.1.g{hex:7}",
                    "1.0.1aXXXXX.post1+feature.test.1.g{hex:7}")
    .expect_schema_variants(create_full_schema_test_cases(...));
```

**Key architectural achievement:**

- **SOLVED**: Flow command now supports stdin input through **stdin caching mechanism**
- Flow command can read input once and cache it for both passes (current state + bumped version)
- Flow command supports **dual modes**:
    - **Stdin input**: For basic functionality testing (like version command)
    - **Git repositories**: For intelligent pre-release management based on branch names
- **All basic tests use stdin**: Following same pattern as version command tests
- **Scenario tests use git**: For comprehensive workflow validation
- Flow tests validate both CLI interface layer AND git workflow behavior

### Reuse Existing Infrastructure

- Use `TestCommand` utility for running CLI commands
- Use `GitRepoFixture` for git repository setup
- Use `should_run_docker_tests()` for Docker test gating
- Follow existing patterns from `tests/integration_tests/version/`

## Progress Summary

### ✅ **Completed Components:**

**Step 1: Test Structure**

- ✅ Complete directory structure created
- ✅ All module files properly connected
- ✅ Integration test root updated

**Step 2: FlowTestUtils Structure**

- ✅ `FlowIntegrationTestScenario` with full API implementation
- ✅ Same fluent API as existing `FlowTestScenario`
- ✅ CLI-based implementation using `TestCommand`
- ✅ Complete `FlowTestResult` with all assertion methods
- ✅ Schema test utilities re-implemented for integration testing
- ✅ Proper Docker test gating
- ✅ All code passes `make lint` and tests

**Step 3: Basic Flow Command Tests** ✅ **COMPLETED**

- ✅ **15/15 tests passing** with 100% success rate
- ✅ Basic "zerv flow" command functionality with stdin input verified
- ✅ Different output formats (semver, pep440) tested and working
- ✅ Schema options (`--schema standard`) tested and working
- ✅ Pre-release handling (alpha, beta, rc) tested and working
- ✅ Branch-based scenarios (main, feature/, develop) tested and working
- ✅ Error handling (empty stdin, invalid RON, invalid options) tested and working
- ✅ Help command functionality tested and working
- ✅ **ACHIEVEMENT**: Flow command now works identically to version command with stdin input

**Step 4: Centralized Stdin Implementation** ✅ **COMPLETED**

- ✅ **SOLVED**: Centralized stdin approach implemented in `app.rs` - stdin extracted once and passed to all pipelines
- ✅ **SOLVED**: Both `run_version_pipeline()` and `run_flow_pipeline()` now accept `stdin_content: Option<&str>` parameter
- ✅ **SOLVED**: Simplified `process_cached_stdin_source()` function for unified stdin handling
- ✅ **SOLVED**: Cleaned up unused `get_current_zerv_object()` function and renamed function for cleaner API
- ✅ **SOLVED**: Flow command supports centralized stdin with the same mechanism as version command
- ✅ **SOLVED**: All 15 basic flow command tests pass (100% success rate) following same patterns as version command tests
- ✅ **ARCHITECTURAL ACHIEVEMENT**: Two-pass design limitation completely resolved through centralized stdin approach

### 🔄 **Implementation Details:**

**API Compatibility:**

- Same builder methods: `create_tag`, `create_branch`, `checkout`, `commit`, `make_dirty`, `merge_branch`
- Same expectation methods: `expect_version(semver, pep440)`, `expect_schema_variants(test_cases)`
- Same assertion methods: `assert_success()`, `assert_failure()`, `assert_stdout_eq()`, etc.

**CLI Integration:**

- Uses `TestCommand` for actual CLI execution
- Tests complete command-line interface layer
- Tests both semver and pep440 formats automatically
- Supports all flow command options and schemas

**Code Quality:**

- Passes `make lint` with no warnings
- Follows repo coding standards
- Proper error handling and test gating
- Clean, maintainable implementation

## Success Criteria

1. ✅ Comprehensive integration test coverage for "zerv flow" command
2. ✅ **ACHIEVED**: Flow command stdin support works like version command
3. ⏳ Tests cover all major CLI options and output formats (pending Steps 4-6)
4. ⏳ Tests cover trunk-based workflow scenarios (pending Steps 4.1-4.2)
5. ⏳ Tests cover GitFlow workflow scenarios (pending Steps 4.3-4.4)
6. ⏳ Error handling tests ensure robust CLI behavior (pending Step 5)
7. ✅ Tests are maintainable and follow existing patterns
8. ✅ Tests are properly gated for Docker dependencies
9. ✅ Integration tests are separate from but complementary to existing pipeline tests
10. ✅ **INNOVATION**: Solved two-pass stdin reading limitation through caching mechanism

## Notes

- ✅ **Integration tests complement existing unit tests** in `src/cli/flow/pipeline.rs`
- ✅ **Focus on testing the CLI interface layer**, not the internal logic (which is already tested)
- ✅ **Follow established patterns** from version command integration tests
- ✅ **Tests are deterministic** and don't depend on timing or external factors
- ✅ **Framework is ready** for implementing remaining test files (Steps 3-6)
- ✅ **All basic functionality verified** through test cases in `args.rs`
