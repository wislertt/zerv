---
inclusion: fileMatch
fileMatchPattern: "src/**/*test*.rs"
---

# Testing Standards

## Environment-Aware Git Testing

**MANDATORY: Use appropriate Git implementation based on environment**

**Local Development:**

- Use `DockerGit` for isolation to avoid interfering with local machine state
- Use `get_git_impl()` helper function for environment-aware selection

**CI Environment:**

- Use `NativeGit` for real platform testing (Windows/macOS/Linux)
- Enabled automatically via `ZERV_TEST_NATIVE_GIT=true` environment variable

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

## Test Stability Requirements

- All tests must pass consistently on all platforms (Linux, macOS, Windows)
- Use `make test` multiple times to verify stability
- Fix any intermittent failures before proceeding
