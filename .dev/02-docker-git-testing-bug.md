# Docker Git Testing Bug - Complete Fix Guide

## Bug Symptoms

- Tests pass locally but fail in GitHub Actions CI
- Error: `fatal: not in a git directory`
- Multiple git-related test failures (typically 11+ tests)
- Docker commands work locally but not in CI environment

## Root Cause

**Docker Image Entrypoint Issue**: `alpine/git:latest` has `git` as the default entrypoint, not `sh`. This means:

- Local Docker might behave differently than CI
- Commands like `docker run alpine/git:latest "git init"` try to execute `git "git init"` instead of `sh -c "git init"`

## Complete Fix Pattern

### 1. Use Correct Docker Image and Entrypoint

```rust
// WRONG - Will fail in CI
let output = Command::new("docker")
    .args(["run", "--rm", "-v", &mount, "alpine/git:latest", "git", "init"])
    .output()?;

// CORRECT - Works everywhere
let output = Command::new("docker")
    .args([
        "run", "--rm", "-v", &mount,
        "--entrypoint", "sh",
        "alpine/git:latest",
        "-c", "git init"
    ])
    .output()?;
```

### 2. Git Commands After Init Need --git-dir Flag

```rust
// WRONG - Will fail after git init
let output = Command::new("docker")
    .args([
        "run", "--rm", "-v", &mount,
        "--entrypoint", "sh",
        "alpine/git:latest",
        "-c", "git add ."
    ])
    .output()?;

// CORRECT - Include --git-dir=.git
let output = Command::new("docker")
    .args([
        "run", "--rm", "-v", &mount,
        "--entrypoint", "sh",
        "alpine/git:latest",
        "-c", "git --git-dir=.git add ."
    ])
    .output()?;
```

### 3. Initial Commit Required for Tags

```rust
// Git tags need HEAD reference, so init_repo must create initial commit
pub fn init_repo(&self) -> Result<(), ZervError> {
    // 1. Git init
    self.run_git_command("git init")?;

    // 2. Configure user (required for commits)
    self.run_git_command("git config user.name 'Test User'")?;
    self.run_git_command("git config user.email 'test@example.com'")?;

    // 3. Create initial commit (REQUIRED for tags)
    self.run_git_command("echo 'initial' > README.md")?;
    self.run_git_command("git --git-dir=.git add .")?;
    self.run_git_command("git --git-dir=.git commit -m 'Initial commit'")?;

    Ok(())
}
```

## Complete Working Pattern

```rust
fn run_git_command(&self, command: &str) -> Result<String, ZervError> {
    let mount = format!("{}:/repo", self.temp_dir.path().display());

    let output = Command::new("docker")
        .args([
            "run", "--rm", "-v", &mount,
            "-w", "/repo",
            "--entrypoint", "sh",
            "alpine/git:latest",
            "-c", command
        ])
        .output()
        .map_err(|e| ZervError::Io(io::Error::other(format!("Docker command failed: {}", e))))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ZervError::Io(io::Error::other(format!(
            "Git command failed: {}\nCommand: {}", stderr, command
        ))));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
```

## Debugging Steps

### 1. Test Docker Locally First

```bash
# Test the exact Docker command
docker run --rm -v $(pwd):/repo -w /repo --entrypoint sh alpine/git:latest -c "git init"
```

### 2. Check CI Logs for Exact Error

- Look for "fatal: not in a git directory"
- Check if Docker commands are being executed correctly
- Verify mount paths and working directories

### 3. Verify Test Isolation

```bash
# Run tests multiple times to check for race conditions
make test
make test
make test
```

## Prevention Checklist

- [ ] Use `--entrypoint sh` with `alpine/git:latest`
- [ ] Include `--git-dir=.git` for git commands after init
- [ ] Create initial commit in `init_repo()`
- [ ] Test Docker commands locally before CI
- [ ] Verify all git operations work in isolated containers
- [ ] Check that temp directories are properly mounted

## Files to Update When Fixing

1. `src/test_utils/git.rs` - Docker git utility functions
2. `src/vcs/git.rs` - Git VCS implementation test helpers
3. Any other files using Docker git commands

## Testing Verification

```bash
# Local test
make test

# Check specific git tests
cargo test git --include-ignored

# Verify CI will pass
git push # Check GitHub Actions
```

This bug has occurred multiple times - always refer to this guide when Docker git tests fail in CI but pass locally.
