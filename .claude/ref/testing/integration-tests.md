# Integration Test Patterns

This document outlines the standard patterns for integration tests in Zerv.

## Default Pattern: `TestCommand::run_with_stdin()` (90% of tests)

Use this for simple stdin → stdout success tests:

```rust
let zerv_ron = ZervFixture::new()
    .with_version(1, 2, 3)
    .build()
    .to_string();

let output = TestCommand::run_with_stdin(
    "version --source stdin --output-format semver",
    zerv_ron,
);

assert_eq!(output, "1.2.3");
```

## Builder Pattern (10% of tests)

Use ONLY when you need:

- Stderr inspection
- Failure testing
- Multiple output assertions

```rust
// Failure testing
let output = TestCommand::new()
    .args_from_str("version --invalid-flag")
    .assert_failure();
assert!(output.stderr().contains("error"));

// Stderr inspection
let output = TestCommand::new()
    .args_from_str("version --source stdin -v")
    .stdin(zerv_ron)
    .assert_success();
assert!(output.stderr().contains("debug"));
assert_eq!(output.stdout().trim(), "1.2.3");
```

## Use rstest Fixtures (Not Helper Functions)

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
- Can override values when needed

## Organize with Modules (Not Comments)

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
