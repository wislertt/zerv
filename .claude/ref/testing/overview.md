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

## Code Reuse Standards

**MANDATORY: Always check existing utilities before creating new ones.**

1. **Check `src/test_utils/` first** before creating test utilities
2. **Reuse existing infrastructure**: `TestDir`, `GitOperations`, `GitRepoFixture`
3. **Use `get_git_impl()`** for environment-aware Git operations
4. **Prefer `GitOperations` trait methods** over direct Docker/Native calls
