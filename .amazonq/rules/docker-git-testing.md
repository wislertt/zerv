# Docker Git Testing Bug - Complete Fix Guide

## Bug Symptoms

- Tests pass locally but fail in GitHub Actions CI
- Error: `fatal: not in a git directory`
- Multiple git-related test failures (typically 11+ tests)
- Docker commands work locally but not in CI environment

## Root Cause

**Docker Image Entrypoint Issue**: `alpine/git:latest` has `git` as the default entrypoint, not `sh`.

### Why Local vs CI Behaves Differently

**Docker Runtime Environment Differences:**

- **Local (Docker Desktop)**: More permissive, has compatibility layers that can "fix" some incorrect commands
- **CI (Docker Engine)**: Stricter, follows Docker specifications exactly

**Entrypoint Handling Differences:**

```bash
# What we were doing (WRONG):
docker run alpine/git:latest "git init"

# Local Docker Desktop might interpret this as:
# -> Run git with argument "git init"
# -> Sometimes works due to shell interpretation

# CI Docker Engine interprets this as:
# -> Run git "git init" (literally passes "git init" as argument to git)
# -> Always fails: git doesn't understand "git init" as a single argument
```

**Environment Masking:**

- **Local**: Your shell might have different PATH, git configs, or environment variables that mask issues
- **CI**: Clean, minimal environment exposes the exact Docker behavior

**The Core Issue:**
`alpine/git:latest` has `ENTRYPOINT ["git"]`, so:

```bash
# Without --entrypoint sh:
docker run alpine/git:latest "some command"
# Actually executes: git "some command"
# ❌ This tries to run git with "some command" as a single argument

# With --entrypoint sh:
docker run --entrypoint sh alpine/git:latest -c "some command"
# Actually executes: sh -c "some command"
# ✅ This runs the command in shell properly
```

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

## Local Strict Mode (IMPLEMENTED)

**Automatic Prevention**: The codebase now includes:

1. **Strict Docker Flags**: All Docker commands use `--security-opt=no-new-privileges --cap-drop=ALL` to behave like CI
2. **Validation Helper**: `validate_docker_args()` catches common anti-patterns:
    - Missing `--entrypoint sh` with `alpine/git:latest`
    - Git commands without proper entrypoint
3. **Fail Fast**: Invalid Docker commands fail locally with clear error messages

**Result**: You'll now get errors like `❌ Missing --entrypoint sh for alpine/git:latest (will fail in CI)` locally instead of discovering issues in CI.

## Files to Update When Fixing

1. `src/test_utils/git.rs` - Docker git utility functions
2. `src/vcs/git.rs` - Git VCS implementation test helpers
3. Any other files using Docker git commands

## Testing Verification

```bash
# Local test (now includes strict mode)
make test

# Test validation works
cargo test test_docker_validation --lib

# Check specific git tests
cargo test git --include-ignored

# Verify CI will pass (should be consistent now)
git push # Check GitHub Actions
```

## Why This Bug Repeats

1. **Local testing masks the issue** - Docker Desktop is more forgiving
2. **The error is subtle** - Commands might partially work locally
3. **CI environment is different** - Only fails in the strict CI environment
4. **Git directory context** - After `git init`, subsequent commands need `--git-dir=.git` in containers

This bug has occurred multiple times - always refer to this guide when Docker git tests fail in CI but pass locally.
