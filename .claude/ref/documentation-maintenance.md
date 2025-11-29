# Documentation and Test Maintenance Guidelines

## Overview

This document provides guidelines for maintaining consistency between documentation (README.md and other docs) and their corresponding tests. Every code example in documentation should have a corresponding test that validates the example's accuracy.

## Principle: Test-Driven Documentation

**All code examples in documentation must be backed by tests.** This ensures:

- Documentation examples are always accurate and working
- Changes to functionality automatically validate documentation
- Future maintainers can easily update both documentation and tests together
- Examples reflect real-world usage scenarios

## Reference Comment Format

### Standard Format

```html
<!-- Corresponding test: path/to/test/file.rs:test_function_name -->
```

### Placement

- Place the comment immediately after the code block or section it validates
- Use HTML comment format (`<!-- -->`) for markdown compatibility
- Include the full test path and test function name

### Examples

````markdown
## Quick Start

```bash
# Install
cargo install zerv

# Try automated versioning
zerv flow
# → 1.0.0 (on main branch)
# → 1.0.1-alpha.59394.post.1+feature.example.1.g4e9af24 (on feature branch)
```
````

<!-- Corresponding test: tests/integration_tests/flow/docs/quick_start.rs:test_quick_start_documentation_examples -->

````

## Implementation Guidelines

### 1. Test Creation Process

When adding or updating documentation examples:

1. **Write the Test First**: Create the test that validates the documentation example
2. **Implement the Documentation**: Write the example in the documentation
3. **Add Reference Comment**: Add the corresponding test reference
4. **Validate Both**: Ensure test passes and documentation matches test output

### 2. Test Structure

```rust
#[test]
fn test_documentation_example_name() {
    // Set up test scenario
    let scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/example")
        .checkout("feature/example")
        .commit();

    // Assert the example matches exactly what's shown in documentation
    scenario.assert_command("flow --source stdin", "1.0.1-alpha.59394.post.1+feature.example.1.g{hex:7}");
}
````

### 3. Documentation Accuracy

**Exact Matching**: Documentation examples must exactly match test output, including:

- Version numbers and formats
- Branch names and patterns
- Hash formats (use realistic 7-character hex values)
- Command syntax and arguments
- Output formatting

### 4. Reference Comment Placement

Place comments after:

- Individual code blocks
- Complete sections with multiple examples
- Command-line examples
- Configuration examples
- Workflow examples

## File Organization

### Test Directory Structure

```
tests/
├── integration_tests/
│   └── docs/
│       └── [component]/
│           ├── mod.rs
│           ├── test_utils.rs
│           ├── quick_start.rs
│           └── [other_feature].rs
```

### Naming Conventions

- **Test files**: `[feature_name].rs` (e.g., `quick_start.rs`, `advanced_usage.rs`)
- **Test functions**: `test_[feature_name]_[scenario]` (e.g., `test_quick_start_basic_examples`)
- **Documentation sections**: Match test file and function names for easy mapping

## Maintenance Workflow

### Updating Documentation

1. **Locate Test**: Find the corresponding test using the reference comment
2. **Update Test First**: Modify the test to reflect new behavior
3. **Update Documentation**: Modify documentation to match new test output
4. **Validate**: Run the test to ensure it passes
5. **Update Reference**: Update reference comment if test path/name changed

### Adding New Examples

1. **Create Test**: Write comprehensive test for the new example
2. **Write Documentation**: Add example to appropriate section
3. **Add Reference**: Include corresponding test reference comment
4. **Review**: Ensure both are accurate and consistent

### Review Process

When reviewing changes:

- Verify all documentation examples have corresponding tests
- Check reference comments point to correct tests
- Ensure examples exactly match test output
- Validate test naming follows conventions

## Reference Comment Examples by Documentation Type

### README.md Examples

````markdown
## Installation

```bash
cargo install zerv
```
````

<!-- Corresponding test: tests/integration_tests/docs/installation.rs:test_cargo_install -->

````

### API Documentation Examples
```rust
/// Example usage
/// ```
/// use zerv::Version;
/// let version = Version::new(1, 2, 3);
/// println!("{}", version); // Outputs: 1.2.3
/// ```
///
/// <!-- Corresponding test: tests/unit/version/creation.rs:test_version_new_example -->
````

### Workflow Documentation

````markdown
## CI/CD Integration

```yaml
# .github/workflows/version.yml
- name: Generate Version
  run: zerv flow --output-format semver
```
````

<!-- Corresponding test: tests/integration_tests/ci/github_actions.rs:test_workflow_version_generation -->

```

## Quality Assurance

### Automated Validation
- Tests should run in CI to catch documentation inconsistencies
- Include documentation tests in regular test suites
- Fail builds if documentation examples don't match tests

### Manual Review Checklist
- [ ] Each code example has corresponding test
- [ ] Reference comment is present and correct
- [ ] Example exactly matches test output
- [ ] Test covers edge cases shown in documentation
- [ ] Documentation uses realistic, not placeholder, values

## Best Practices

### 1. Realistic Values
- Use actual commit hashes, not `{hex}` placeholders
- Use realistic version numbers, not generic examples
- Include real branch names and patterns

### 2. Comprehensive Coverage
- Test success cases, error cases, and edge cases
- Cover different input formats and output formats
- Include configuration variations

### 3. Clear Comments
- Explain what each test validates
- Document any special setup or requirements
- Reference the specific documentation section being tested

### 4. Maintainable Structure
- Keep related tests together
- Use descriptive test names
- Include helper functions for common test patterns

## Troubleshooting

### Common Issues
1. **Documentation examples don't match tests**
   - Check test output vs documentation
   - Update documentation to match test exactly
   - Verify reference comment points to correct test

2. **Missing reference comments**
   - Add corresponding test reference comment
   - Ensure comment format is correct
   - Place comment in appropriate location

3. **Test failures after documentation changes**
   - Update test to reflect new behavior
   - Ensure test covers all aspects of documentation
   - Run tests locally before submitting changes

### Debugging Tips
- Run specific test: `cargo test test_function_name`
- Use `--nocapture` flag to see test output
- Compare documentation example with actual test output
- Check for formatting differences (whitespace, line endings)

## Resources

- [Test Documentation Guidelines](./testing-overview.md)
- [Code Standards](../standards/)
- [Integration Test Patterns](./integration-testing.md)
```
