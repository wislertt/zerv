# Testing Standards

## Environment-Aware Git Testing

**MANDATORY: Use appropriate Git implementation based on environment**

**Local Development:**

- Use `DockerGit` for isolation to avoid interfering with local machine state
- Isolate test environment completely from host git config and repositories
- Use `get_git_impl()` helper function for environment-aware selection

**CI Environment:**

- Use `NativeGit` for real platform testing (Windows/macOS/Linux)
- Enabled automatically via `ZERV_CI=true` environment variable
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

## Test Stability Requirements

**Before committing:**

- All tests must pass consistently on all platforms (Linux, macOS, Windows)
- Use `make test` multiple times to verify stability
- Fix any intermittent failures before proceeding
- Ensure tests work in both local (Docker) and CI (Native) environments
- Use `#[cfg(target_os = "linux")]` for Docker-specific tests
