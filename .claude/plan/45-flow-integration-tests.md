# Create Integration Tests for "zerv flow" Command

## Status

**Planned**

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

### Step 1: Create Test Structure

- Create `tests/integration_tests/flow/mod.rs` - Main flow integration test module
- Create `tests/integration_tests/flow/main/` subdirectory for main flow command tests
- Create `tests/integration_tests/flow/scenarios/` subdirectory for complex workflow tests
- Create `tests/integration_tests/flow/args.rs` for flow-specific test utilities

### Step 2: Create FlowTestUtils Structure

- Create `FlowIntegrationTestScenario` with the same fluent API as `FlowTestScenario` but for CLI integration tests
- Implement the same builder pattern methods: `create_tag`, `create_branch`, `checkout`, `commit`, `make_dirty`, `merge_branch`
- Add the same expectation methods: `expect_version(semver, pep440)` and `expect_schema_variants(test_cases)`
- The key difference: `expect_version` and `expect_schema_variants` will internally use CLI string arguments via `TestCommand` instead of calling `run_flow_pipeline` directly
- Reuse existing `GitRepoFixture` and `TestCommand` from test utilities
- Create `scenarios/test_utils.rs` that reuses utilities from `src/cli/flow/test_utils.rs`:
    - Reuse `SchemaTestCase`, `SchemaTestExtraCore`, `SchemaTestBuild` structs
    - Reuse `create_full_schema_test_cases()` and `create_base_schema_test_cases()` functions
    - Reuse `expect_branch_hash()` function for predictable hash generation
    - Only add CLI-specific wrappers for integration testing

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
│   ├── mod.rs               # Main flow command tests (using stdin)
│   ├── basic_commands.rs    # Basic flow command tests (using stdin)
│   ├── output_formats.rs    # Output format tests (using stdin)
│   ├── schema_options.rs    # Schema option tests (using stdin)
│   └── error_handling.rs    # Error handling tests (using stdin)
└── scenarios/
    ├── mod.rs               # Complex workflow tests (using git)
    ├── test_utils.rs        # Reused test utilities for CLI integration
    ├── trunk_based.rs       # Trunk-based workflow tests (using git)
    └── gitflow.rs           # GitFlow workflow tests (using git)
```

### Integration Test Patterns

**Basic Tests (using stdin input - similar to version command tests):**

```rust
// For basic functionality, schema options, output formats, error handling
let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();
let output = TestCommand::run_with_stdin(
    "flow --output-format semver --schema standard",
    zerv_ron
);
assert_eq!(output, "1.0.0");
```

**Scenario Tests (using git repositories - same API as existing pipeline tests):**

```rust
// For complex workflow scenarios (trunk-based, GitFlow)
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

**Key distinction:**

- Basic tests use `ZervFixture` + `TestCommand::run_with_stdin()` (like version tests) - fast and deterministic
- Scenario tests use `FlowIntegrationTestScenario` with git repositories for workflow testing - comprehensive git workflow validation

### Reuse Existing Infrastructure

- Use `TestCommand` utility for running CLI commands
- Use `GitRepoFixture` for git repository setup
- Use `should_run_docker_tests()` for Docker test gating
- Follow existing patterns from `tests/integration_tests/version/`

## Success Criteria

1. ✅ Comprehensive integration test coverage for "zerv flow" command
2. ✅ Tests cover all major CLI options and output formats
3. ✅ Tests cover trunk-based and GitFlow workflow scenarios
4. ✅ Error handling tests ensure robust CLI behavior
5. ✅ Tests are maintainable and follow existing patterns
6. ✅ Tests are properly gated for Docker dependencies
7. ✅ Integration tests are separate from but complementary to existing pipeline tests

## Notes

- Integration tests will complement existing unit tests in `src/cli/flow/pipeline.rs`
- Focus on testing the CLI interface layer, not the internal logic (which is already tested)
- Follow the established patterns from version command integration tests
- Ensure tests are deterministic and don't depend on timing or external factors
