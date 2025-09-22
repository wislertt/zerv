---
inclusion: fileMatch
fileMatchPattern: "src/test_utils/git/**/*.rs"
---

# Docker Git Testing Guide - Multi-Platform Architecture

## ✅ CURRENT STATUS: Multi-Platform CI Implemented

**Architecture Update**: The project now uses environment-aware Git testing:

- **Local Development**: Uses `DockerGit` for isolation
- **CI Environment**: Uses `NativeGit` for real platform testing
- **Platform Coverage**: Linux, macOS, Windows all tested with native Git

## Complete Fix Pattern

### 1. Use Correct Docker Image and Entrypoint

```rust
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

## Prevention Checklist

- [ ] Use `--entrypoint sh` with `alpine/git:latest`
- [ ] Include `--git-dir=.git` for git commands after init
- [ ] Create initial commit in `init_repo()`
- [ ] Test Docker commands locally before CI
