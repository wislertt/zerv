# Testing Standards

## Environment-Aware Git Testing

**MANDATORY: Use appropriate Git implementation based on environment**

**Local Development:**

- Use `DockerGit` for isolation to avoid interfering with local machine state
- Isolate test environment completely from host git config and repositories
- Use `get_git_impl()` helper function for environment-aware selection

**CI Environment:**

- Use `NativeGit` for real platform testing (Windows/macOS/Linux)
- Enabled automatically via `ZERV_TEST_NATIVE_GIT=true` environment variable
- Tests actual platform-specific Git behavior, paths, line endings

**Implementation Pattern:**

```rust
use crate::test_utils::{GitOperations, should_use_native_git};
use crate::test_utils::git::{DockerGit, NativeGit};

fn get_git_impl() -> Box<dyn GitOperations> {
    if should_use_native_git() {
        Box::new(NativeGit::new())
    } else {
        Box::new(DockerGit::new())
    }
}
```

## Docker Test Control

**MANDATORY: Use `should_run_docker_tests()` for all Docker-dependent tests**

**Environment Variables:**

- `ZERV_TEST_DOCKER=true`: Enable Docker tests (requires Docker to be available)
- `ZERV_TEST_DOCKER=false`: Skip Docker tests (default)

**Policy Enforcement:**

- If Docker is available on system, Docker tests MUST be enabled
- Only skip Docker tests when Docker is genuinely unavailable
- Tests will fail if `ZERV_TEST_DOCKER=true` but Docker is not available

**Implementation Pattern:**

```rust
use crate::test_utils::should_run_docker_tests;

#[test]
fn test_docker_functionality() {
    if !should_run_docker_tests() {
        return; // Skip when Docker tests are disabled
    }
    // Docker-dependent test code
}
```

**Make Commands:**

- `make test_easy`: Docker Git + Docker tests skipped (fast, coverage gaps)
- `make test`: Docker Git + Docker tests enabled (full coverage)
- CI: Native Git + Docker tests on Linux only

## Race Condition Prevention

**Atomic Operations Required:**

- Use `GitOperations` trait methods for consistent behavior across implementations
- Prefer trait methods like `init_repo()`, `create_tag()`, `create_commit()` over raw commands
- Avoid multi-step operations that can cause filesystem race conditions
- Use shared logic in trait implementations for consistency

**Flaky Test Detection:**

When user mentions:

- "check for flaky tests"
- "test stability"
- "race condition"
- "sometimes pass sometimes fail"

→ Run `make test` in a loop (default 10 iterations) to detect instability

**Flaky Test Prevention Patterns:**

1. **Detailed Error Context**: Include specific error messages with context about what failed and why
2. **State Verification**: Verify intermediate states (e.g., `.git` directory exists) before proceeding
3. **Atomic Operations**: Use `GitOperations` trait methods that combine multiple Git operations atomically
4. **Error Propagation**: Use `.unwrap_or_else()` with detailed panic messages instead of `.expect()`
5. **Resource Isolation**: Each test gets its own `TestDir` and `GitRepoFixture` to prevent interference

## Test Stability Requirements

**Before committing:**

- All tests must pass consistently on all platforms (Linux, macOS, Windows)
- Use `make test` multiple times to verify stability
- Fix any intermittent failures before proceeding
- Ensure tests work in both local (Docker) and CI (Native) environments
- Use `#[cfg(target_os = "linux")]` for Docker-specific tests
