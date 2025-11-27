# README.md Documentation Tests Implementation Plan

**Status**: Planned
**Priority**: High
**Context**: Need to create working tests for Quick Start documentation examples in README.md to ensure documentation accuracy

## Goals

1. Create working `test_quick_start_documentation_examples` test that validates README.md examples
2. Implement `TestScenario` in `tests/integration_tests/flow/docs/test_utils.rs` similar to `FlowTestScenario`
3. Use `TestCommand` to implement `assert_command(command, expected_output)` and `assert_commands(commands, expected_output)` methods
4. Fix all failing tests in `tests/integration_tests/flow/docs/*`
5. Ensure documentation examples work exactly as shown in README.md

## Current Issues Analysis

### Existing TestScenario Problems

1. **Compilation errors**: String comparison `*arg == "--output-format"` fails
2. **Private method access**: `self.flow_scenario.to_stdin_content()` is private
3. **Complex implementation**: Current approach is overly complicated
4. **Wrong dependencies**: Trying to use `FlowTestScenario` infrastructure instead of `TestCommand`

### Test Infrastructure Requirements

1. **Fixture-based**: Should use test fixtures like `FlowTestScenario`, not real Git operations
2. **CLI command parsing**: Must parse actual CLI commands like `"flow --output-format semver"`
3. **Pattern matching**: Must support `{hex:7}`, `{timestamp}` patterns like existing tests
4. **Simple interface**: `assert_command(command, expected_output)` and `assert_commands(commands, expected_output)` methods

## Implementation Plan

### Step 1: Research Phase

1. Study how `TestCommand` works in existing integration tests
2. Examine `FlowTestScenario` implementation for fixture creation patterns
3. Understand how pattern assertion works in existing tests (`{hex:7}`, etc.)
4. Review existing integration test structure for CLI command testing

### Step 2: TestScenario Implementation

1. Create new `TestScenario` struct inspired by `FlowTestScenario` but using `TestCommand`
2. Implement fixture creation methods (create_tag, create_branch, checkout, commit, etc.)
3. Implement `assert_command(command, expected_output)` and `assert_commands(commands, expected_output)` using `TestCommand::run_with_stdin()`
4. Use existing `ZervFixture` and `ZervVarsFixture` for test data

### Step 3: Fix quick_start.rs Test

1. Update test to use new `TestScenario` implementation
2. Ensure all version patterns match actual CLI output
3. Verify pattern matching works with `{hex:7}` placeholders
4. Test passes consistently

### Step 4: Cleanup and Documentation

1. Remove any failing test code from current attempts
2. Ensure clean separation between fixture creation and command execution
3. Add proper error handling and documentation
4. Verify tests work with existing test infrastructure

## Technical Implementation Details

### TestScenario Structure

```rust
pub struct TestScenario {
    /// Branch name -> ZervVars for that branch
    branch_vars: HashMap<String, ZervVars>,

    /// Current active branch
    current_branch: String,

    /// Current branch's vars
    current_vars: ZervVars,
}

impl TestScenario {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>>
    pub fn create_tag(self, tag: &str) -> Self
    pub fn create_branch(self, branch_name: &str) -> Self
    pub fn checkout(self, branch_name: &str) -> Self
    pub fn commit(self) -> Self
    pub fn assert_command(&self, command: &str, expected_output: &str) -> &Self
    pub fn assert_commands(&self, commands: &[&str], expected_output: &str) -> &Self
}
```

### assert_command and assert_commands Implementation Strategy

#### assert_command (single command)

1. Use `TestCommand::run_with_stdin(command, stdin_content)` for execution
2. Convert fixture to RON format using `to_stdin_content()` method (replicate from FlowTestScenario)
3. Parse CLI command string using TestCommand's built-in `args_from_str()`
4. Use existing `assert_version_expectation()` for pattern validation
5. Return trimmed output string

#### assert_commands (multiple commands/pipeline)

1. **Note**: No piping examples found in codebase - this may be more complex than anticipated
2. Consider implementing as sequential commands with output capture between commands
3. Alternative: Focus on `assert_command` only for initial implementation
4. Use shell piping approach: `"cmd1 | cmd2 | cmd3"` as single command string

### Fixture Management

1. Use `ZervFixture` for creating Git state fixtures
2. Use `ZervVarsFixture` for environment variables
3. Convert fixture to stdin content using: `Zerv → RON` serialization (like FlowTestScenario::to_stdin_content())
4. **Key insight**: Create RON string with default schema for TestCommand stdin input
5. Ensure fixtures match scenarios described in README.md

### Key Implementation Insights (from Research)

1. **Stdin Content**: Replicate FlowTestScenario's private `to_stdin_content()` method:

    ```rust
    let schema = ZervSchema::semver_default().unwrap();
    let zerv = Zerv { schema, vars: self.current_vars.clone() };
    ron::to_string(&zerv).unwrap()
    ```

2. **TestCommand Usage**: Use established pattern from codebase:

    ```rust
    TestCommand::run_with_stdin("flow --output-format semver", zerv_ron)
    ```

3. **Pattern Assertion**: Reuse existing function directly:

    ```rust
    assert_version_expectation(expected_output, &actual_output)
    ```

4. **Command Parsing**: TestCommand's `args_from_str()` handles complex parsing automatically

## Success Criteria

- `test_quick_start_documentation_examples` test passes
- `TestScenario` works like `FlowTestScenario` but uses `TestCommand`
- All README.md Quick Start examples are validated
- Pattern matching with `{hex:7}` works correctly
- No compilation errors or runtime failures
- Tests are maintainable and follow existing patterns

## Dependencies

- `TestCommand` from `tests/integration_tests/util/command.rs`
- `ZervFixture`, `ZervVarsFixture` from existing test infrastructure
- Pattern assertion utilities from existing flow tests
- CLI argument parsing logic

## Risks and Mitigations

### Risk 1: Complex Fixture Creation

**Mitigation**: Study existing `FlowTestScenario` implementation and copy fixture creation patterns

### Risk 2: CLI Command Parsing Complexity

**Mitigation**: Use simple string splitting and existing `TestCommand` argument handling

### Risk 3: Pattern Assertion Integration

**Mitigation**: Reuse existing pattern assertion system from flow integration tests

### Risk 4: Test Environment Setup

**Mitigation**: Follow existing integration test patterns for environment variables and test isolation

## Notes

- This is a consolidation effort - we're creating proper test infrastructure for documentation validation
- Focus on simplicity and reusing existing patterns rather than creating new infrastructure
- The goal is to ensure README.md examples work exactly as documented
- Future documentation tests can reuse this `TestScenario` infrastructure
