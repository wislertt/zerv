# Unit Testing Standards

## Environment-Aware Git Testing

**MANDATORY: Always use `get_git_impl()` for environment-aware Git operations.**

```rust
use crate::test_utils::{GitOperations, get_git_impl};

// ✅ GOOD
let git_impl = get_git_impl();
git_impl.init_repo(&test_dir)?;

// ❌ BAD
let git = DockerGit::new();
git.init_repo(&test_dir)?;
```

## Docker Test Gating

**MANDATORY: Use `should_run_docker_tests()` for all Docker-dependent tests.**

```rust
use crate::test_utils::should_run_docker_tests;

#[test]
fn test_docker_functionality() {
    if !should_run_docker_tests() {
        return;
    }
    // Docker-dependent test code
}
```

## Test Structure Template

```rust
#[test]
fn test_example() {
    // 1. Docker test gating (if needed)
    if !should_run_docker_tests() {
        return;
    }

    // 2. Setup - Create isolated resources
    let fixture = GitRepoFixture::tagged("v1.0.0")
        .expect("Failed to create tagged repo - check Docker availability");

    // 3. Verify Setup - Check preconditions
    assert!(fixture.path().join(".git").exists(),
        "Git repository should exist at: {}", fixture.path().display());

    // 4. Execute - Run the test operation
    let result = some_operation(&fixture);

    // 5. Verify Results - Check postconditions with detailed assertions
    let output = result.unwrap_or_else(|e| {
        panic!("Operation should succeed for tagged repo at {}: {}",
               fixture.path().display(), e);
    });

    assert!(output.contains("expected"),
        "Output should contain expected content. Got: {output}");

    // 6. Cleanup - Automatic via Drop trait
}
```

## Flaky Test Prevention - CRITICAL RULES

**✅ DO:**

- Create fresh Git implementation per directory: `get_git_impl()`
- Isolate resources per test: `GitRepoFixture::tagged("v1.0.0")?`
- Verify state after setup
- Use detailed error messages in `expect()`
- Include context in assertions

**❌ DON'T:**

- Reuse Git implementations across directories (race conditions!)
- Share state between tests (use `static` fixtures)
- Skip verification steps
- Use generic error messages

## Docker Flaky Test Troubleshooting

If you encounter flaky Docker-related Git test failures, check the retry logic at `src/test_utils/git/docker.rs:208-227`.

**Common transient error patterns** (should auto-retry):

- `"cannot update ref"` - Git reference update race condition
- `"nonexistent object"` - Object not yet visible to Git
- `"is not a valid object"` - Object consistency check failure

Add new patterns to the retry condition if needed. Do NOT add retry logic for legitimate errors.
