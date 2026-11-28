# Testing Standards Overview

## Environment Variables

- `ZERV_TEST_NATIVE_GIT=true`: Use native Git (set in CI)
- `ZERV_TEST_DOCKER=true`: Enable Docker-dependent tests

## Configuration

Centralized config in `src/config.rs`:

- Loads environment variables (`ZERV_TEST_NATIVE_GIT`, `ZERV_TEST_DOCKER`)
- Single source of truth for environment configuration

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name -- --exact

# Run without Docker (fast, coverage gaps)
ZERV_TEST_DOCKER=false cargo test

# Run with full coverage
ZERV_TEST_DOCKER=true cargo test
```

## rstest Testing Framework

**STANDARD: Use rstest for all test fixture and parameterization needs.**

### rstest Fixtures (Not Helper Functions)

```rust
use rstest::{fixture, rstest};

#[fixture]
fn tier_1_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 0, 0)
        .with_standard_tier_1()
}

#[rstest]
fn test_something(tier_1_fixture: ZervFixture) {
    let zerv_ron = tier_1_fixture.build().to_string();
    let output = TestCommand::run_with_stdin("version --source stdin", zerv_ron);
    assert_eq!(output, "1.0.0");
}
```

**Why fixtures are better**:

- Automatic injection by rstest
- Better test isolation
- Less boilerplate
- Can be combined with `#[case]` parameters

### rstest Parameterization

Use `#[case]` attributes for testing multiple variations of the same logic:

```rust
#[rstest]
#[case::semver("semver", "1.0.0")]
#[case::pep440("pep440", "1.0.0")]
#[case::zerv("zerv", "1.0.0")]
fn test_output_formats(#[case] format: &str, #[case] expected: &str) {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 0, 0)
        .build()
        .to_string();

    let output = TestCommand::run_with_stdin(
        &format!("version --source stdin --output-format {}", format),
        zerv_ron,
    );

    assert_eq!(output, expected);
}
```

The `::name` syntax after `#[case]` provides descriptive test names in output.

### Test Organization with Modules

```rust
mod output_format_basic {
    //! Tests for basic format conversions (semver ↔ pep440 ↔ zerv)
    use super::*;

    #[rstest]
    #[case::semver("semver", "1.2.3")]
    #[case::pep440("pep440", "1.2.3")]
    fn test_format(#[case] format: &str, #[case] expected: &str) {
        // Test implementation
    }
}
```

**Module documentation**: Use `//!` inner doc comments to explain what each test module covers.

## Code Reuse Standards

**MANDATORY: Always check existing utilities before creating new ones.**

1. **Check `src/test_utils/` first** before creating test utilities
2. **Reuse existing infrastructure**: `TestDir`, `GitOperations`, `GitRepoFixture`
3. **Use `get_git_impl()`** for environment-aware Git operations
4. **Prefer `GitOperations` trait methods** over direct Docker/Native calls
