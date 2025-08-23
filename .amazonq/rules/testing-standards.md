# Testing Standards

## Docker Isolation for VCS Tests

**MANDATORY: Use Docker for VCS operations that modify state**

- Use Docker for all Git/VCS tests to avoid interfering with local machine state
- Isolate test environment completely from host git config and repositories
- Use `DockerGit` utility from `src/test_utils/git.rs` for git operations in tests

## Race Condition Prevention

**Atomic Operations Required:**

- Use single Docker commands with shell scripts instead of multiple separate commands
- Combine git operations like `git init && git add . && git commit` in one Docker call
- Avoid multi-step Docker operations that can cause filesystem race conditions

**Flaky Test Detection:**

When user mentions:

- "check for flaky tests"
- "test stability"
- "race condition"
- "sometimes pass sometimes fail"

→ Run `make test` in a loop (default 10 iterations) to detect instability

## Test Stability Requirements

**Before committing:**

- All tests must pass consistently
- Use `make test` multiple times to verify stability
- Fix any intermittent failures before proceeding
- Document any Docker or environment dependencies
