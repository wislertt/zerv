# Error Handling Improvements

## Current Error Issues

### 1. No Version Tags Found

**Current Error**:

```
Error: IO error: No version tag found in VCS data
```

**Issues**:

- Mentions "VCS data" instead of specific source
- Uses generic IO error wrapper
- Doesn't indicate which source was used

**Should Be**:

```
Error: No version tags found in git repository
```

### 2. Not in Git Repository

**Current Error**:

```
Error: VCS not found: No VCS repository found
```

**Issues**:

- Generic "VCS" instead of specific source
- Doesn't indicate which source was attempted

**Should Be**:

```
Error: Not in a git repository (--source git)
```

### 3. No Commits (Empty Repository)

**Current Error**:

```
Error: Command execution failed: Git command failed: fatal: ambiguous argument 'HEAD': unknown revision or path not in the working tree.
Use '--' to separate paths from revisions, like this:
'git <command> [<revision>...] -- [<file>...]'
```

**Issues**:

- Raw git error message exposed to user
- Confusing technical details
- No clear indication of the actual problem

**Should Be**:

```
Error: No commits found in git repository
```

## Implementation Plan

### 1. Add Source-Aware Error Types

```rust
// In src/error.rs
#[derive(Debug)]
pub enum ZervError {
    // Existing variants...

    /// No version tags found in specified source
    NoVersionTags(String), // source name

    /// Repository not found for specified source
    RepositoryNotFound(String), // source name

    /// No commits found in repository
    NoCommits(String), // source name

    /// Git command failed with user-friendly message
    GitCommandFailed(String), // user-friendly message
}
```

### 2. Update Error Display Messages

```rust
impl std::fmt::Display for ZervError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Existing cases...

            ZervError::NoVersionTags(source) => {
                write!(f, "No version tags found in {source} repository")
            }
            ZervError::RepositoryNotFound(source) => {
                write!(f, "Not in a {source} repository (--source {source})")
            }
            ZervError::NoCommits(source) => {
                write!(f, "No commits found in {source} repository")
            }
            ZervError::GitCommandFailed(msg) => {
                write!(f, "{msg}")
            }
        }
    }
}
```

### 3. Update VCS Detection

```rust
// In src/vcs/mod.rs
pub fn detect_vcs(path: &Path) -> Result<Box<dyn Vcs>> {
    let git_vcs = git::GitVcs::new(path)?;
    if git_vcs.is_available(path) {
        return Ok(Box::new(git_vcs));
    }

    // Use source-aware error
    Err(ZervError::RepositoryNotFound("git".to_string()))
}
```

### 4. Update Git VCS Implementation

```rust
// In src/vcs/git.rs
impl GitVcs {
    fn run_git_command(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| ZervError::CommandFailed(format!("Failed to execute git: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Translate common git errors to user-friendly messages
            if stderr.contains("fatal: ambiguous argument 'HEAD'") {
                return Err(ZervError::NoCommits("git".to_string()));
            }

            if stderr.contains("not a git repository") {
                return Err(ZervError::RepositoryNotFound("git".to_string()));
            }

            // Generic git command failure
            return Err(ZervError::GitCommandFailed(format!(
                "Git command failed: {stderr}"
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
```

### 5. Update Pipeline Error Handling

```rust
// In src/pipeline/vcs_data_to_zerv_vars.rs
pub fn vcs_data_to_zerv_vars(vcs_data: VcsData) -> Result<ZervVars, ZervError> {
    let version = if let Some(ref tag_version) = vcs_data.tag_version {
        parse_version_from_tag(tag_version, None).ok_or_else(|| {
            ZervError::InvalidFormat(format!(
                "Failed to parse version from tag: {tag_version}"
            ))
        })?
    } else {
        // Use source-aware error instead of generic IO error
        return Err(ZervError::NoVersionTags("git".to_string()));
    };

    // Rest of implementation...
}
```

## Additional Error Cases to Handle

### 1. Git Not Installed

**Detection**: Command execution fails with "command not found"

**Error Message**:

```
Error: Git command not found. Please install git and try again.
```

### 2. Permission Denied

**Detection**: Git command fails with permission error

**Error Message**:

```
Error: Permission denied accessing git repository
```

### 3. Shallow Clone Warning

**Current**: Warning printed to stderr
**Improvement**: Include in error context when distance is inaccurate

### 4. Detached HEAD State

**Current**: May cause branch detection to fail
**Improvement**: Handle gracefully with appropriate messaging

## Testing Strategy

### 1. Error Case Tests

```rust
#[test]
fn test_no_version_tags_error() {
    let temp_dir = setup_git_repo_no_tags();
    let result = run_version_pipeline(args, Some(temp_dir.path()));

    assert!(matches!(result, Err(ZervError::NoVersionTags(source)) if source == "git"));
    assert_eq!(result.unwrap_err().to_string(), "No version tags found in git repository");
}

#[test]
fn test_not_git_repo_error() {
    let temp_dir = TestDir::new();
    let result = run_version_pipeline(args, Some(temp_dir.path()));

    assert!(matches!(result, Err(ZervError::RepositoryNotFound(source)) if source == "git"));
    assert_eq!(result.unwrap_err().to_string(), "Not in a git repository (--source git)");
}

#[test]
fn test_no_commits_error() {
    let temp_dir = setup_empty_git_repo();
    let result = run_version_pipeline(args, Some(temp_dir.path()));

    assert!(matches!(result, Err(ZervError::NoCommits(source)) if source == "git"));
    assert_eq!(result.unwrap_err().to_string(), "No commits found in git repository");
}
```

### 2. Error Message Consistency Tests

```rust
#[rstest]
#[case(ZervError::NoVersionTags("git".to_string()), "No version tags found in git repository")]
#[case(ZervError::RepositoryNotFound("git".to_string()), "Not in a git repository (--source git)")]
#[case(ZervError::NoCommits("git".to_string()), "No commits found in git repository")]
fn test_error_message_format(#[case] error: ZervError, #[case] expected: &str) {
    assert_eq!(error.to_string(), expected);
}
```

## Implementation Priority

1. **High Priority**: Fix the three main error cases mentioned by user
2. **Medium Priority**: Add git command error translation
3. **Low Priority**: Handle edge cases (shallow clone, detached HEAD, etc.)

## Backward Compatibility

- New error types are additions, not changes to existing types
- Error message format changes are improvements, not breaking changes
- All existing error handling continues to work
