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

## Proactive Flaky Test Prevention

**MANDATORY: Follow these patterns when writing new tests to prevent flaky behavior**

### Git Test Anti-Patterns (AVOID)

❌ **DON'T: Reuse Git implementations across different directories**

```rust
// BAD - Can cause race conditions
let fixture = GitRepoFixture::tagged("v1.0.0")?;
fixture.git_impl.init_repo(&different_dir)?; // Race condition!
```

✅ **DO: Create fresh Git implementations for each directory**

```rust
// GOOD - Atomic and isolated
let git_impl = get_git_impl();
git_impl.init_repo(&test_dir)?;
```

❌ **DON'T: Multi-step Git operations without verification**

```rust
// BAD - No verification between steps
let fixture = GitRepoFixture::with_distance("v1.0.0", 1)?;
fixture.git_impl.execute_git(&fixture.test_dir, &["tag", "-d", "v1.0.0"])?;
```

✅ **DO: Atomic operations with state verification**

```rust
// GOOD - Atomic creation without tag removal
let test_dir = TestDir::new()?;
let git_impl = get_git_impl();
git_impl.init_repo(&test_dir)?;
// Verify state before proceeding
assert!(test_dir.path().join(".git").exists());
```

### Error Handling Anti-Patterns (AVOID)

❌ **DON'T: Generic error messages**

```rust
// BAD - No context about what failed
let fixture = GitRepoFixture::tagged("v1.0.0").expect("Failed");
```

✅ **DO: Specific error context with diagnostic information**

```rust
// GOOD - Clear context and diagnostic info
let fixture = GitRepoFixture::tagged("v1.0.0")
    .expect("Failed to create tagged repo - check Docker availability and Git operations");

// Verify state with detailed assertions
assert!(fixture.path().join(".git").exists(),
    "Git repository should exist at: {}", fixture.path().display());
```

### Resource Management Anti-Patterns (AVOID)

❌ **DON'T: Share resources between test cases**

```rust
// BAD - Shared state can cause interference
static SHARED_FIXTURE: OnceCell<GitRepoFixture> = OnceCell::new();
```

✅ **DO: Isolated resources per test**

```rust
// GOOD - Each test gets its own isolated resources
#[test]
fn test_something() {
    let fixture = GitRepoFixture::tagged("v1.0.0")?; // Isolated
    // Test logic here
}
```

### Assertion Anti-Patterns (AVOID)

❌ **DON'T: Assertions without context**

```rust
// BAD - No information when assertion fails
assert!(output.contains("schema"));
```

✅ **DO: Detailed assertions with diagnostic output**

```rust
// GOOD - Clear failure information
assert!(output.contains("schema"),
    "Output should contain 'schema' field. Got: {output}");
```

### Test Structure Best Practices

**Required Test Structure:**

1. **Setup**: Create isolated resources
2. **Verify Setup**: Check preconditions
3. **Execute**: Run the test operation
4. **Verify Results**: Check postconditions with detailed assertions
5. **Cleanup**: Automatic via RAII (TestDir, GitRepoFixture)

**Example Template:**

```rust
#[test]
fn test_example() {
    if !should_run_docker_tests() {
        return; // Skip when Docker tests are disabled
    }

    // 1. Setup - Create isolated resources
    let fixture = GitRepoFixture::tagged("v1.0.0")
        .expect("Failed to create tagged repo - check Docker and Git operations");

    // 2. Verify Setup - Check preconditions
    assert!(fixture.path().join(".git").exists(),
        "Git repository should exist at: {}", fixture.path().display());

    // 3. Execute - Run the test operation
    let result = some_operation(&fixture);

    // 4. Verify Results - Check postconditions with detailed assertions
    let output = result.unwrap_or_else(|e| {
        panic!("Operation should succeed for tagged repo at {}: {}",
               fixture.path().display(), e);
    });

    assert!(output.contains("expected"),
        "Output should contain expected content. Got: {output}");

    // 5. Cleanup - Automatic via Drop trait
}
```

### Code Review Checklist

When reviewing new tests, check for:

- [ ] Uses `get_git_impl()` for fresh Git implementations
- [ ] No reuse of Git implementations across different directories
- [ ] Detailed error messages with context
- [ ] State verification between operations
- [ ] Isolated resources per test (no shared state)
- [ ] Proper Docker test gating with `should_run_docker_tests()`
- [ ] Detailed assertions with diagnostic output
- [ ] Uses atomic `GitOperations` trait methods

## Test Stability Requirements

**Before committing:**

- All tests must pass consistently on all platforms (Linux, macOS, Windows)
- Use `make test` multiple times to verify stability
- Fix any intermittent failures before proceeding
- Ensure tests work in both local (Docker) and CI (Native) environments
- Use `#[cfg(target_os = "linux")]` for Docker-specific tests
