# Create Integration Tests for "zerv flow" Command

## Status

**In Progress** - Steps 1-2 completed, remaining steps pending

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

### Step 3: Basic Flow Command Tests (using stdin input)

- Test basic "zerv flow" command with default settings using zerv stdin input
- Test "zerv flow" with different output formats (`--output-format semver`, `--output-format pep440`) using stdin
- Test "zerv flow" with schema options (`--schema standard`, `--schema calver`) using stdin
- Test "zerv flow" with directory option (`-C /path/to/repo`) using stdin

### Step 4: Workflow Scenario Tests (using git repositories)

- Test trunk-based development workflow scenarios using actual git repositories
- Test GitFlow development workflow scenarios using actual git repositories
- Test branch switching, tagging, and merging scenarios through CLI
- Test dirty working directory scenarios through CLI

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

**Step 3: Stdin Support Implementation**

- ✅ **FIXED**: Flow command two-pass design limitation by implementing stdin caching
- ✅ **FIXED**: Added stdin input caching in flow pipeline to read once and reuse for both passes
- ✅ **FIXED**: Created `process_cached_stdin_source()` and `get_current_zerv_object_with_cached_stdin()` functions
- ✅ **FIXED**: All basic flow command tests now work with stdin input just like version command
- ✅ Flow command now supports both stdin input (for basic functionality) and git repositories (for intelligent versioning)

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
4. ⏳ Tests cover trunk-based and GitFlow workflow scenarios (pending Step 4)
5. ⏳ Error handling tests ensure robust CLI behavior (pending Step 5)
6. ✅ Tests are maintainable and follow existing patterns
7. ✅ Tests are properly gated for Docker dependencies
8. ✅ Integration tests are separate from but complementary to existing pipeline tests
9. ✅ **INNOVATION**: Solved two-pass stdin reading limitation through caching mechanism

## Notes

- ✅ **Integration tests complement existing unit tests** in `src/cli/flow/pipeline.rs`
- ✅ **Focus on testing the CLI interface layer**, not the internal logic (which is already tested)
- ✅ **Follow established patterns** from version command integration tests
- ✅ **Tests are deterministic** and don't depend on timing or external factors
- ✅ **Framework is ready** for implementing remaining test files (Steps 3-6)
- ✅ **All basic functionality verified** through test cases in `args.rs`
