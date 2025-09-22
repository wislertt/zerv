# Design Document

## Overview

This design improves error handling for the zerv CLI tool by enhancing error messages to be source-aware and user-friendly. The design keeps the existing `ZervError` enum structure but focuses on improving error message quality and organization while translating technical VCS errors into clear, actionable messages.

## Architecture

### Error Type Hierarchy

The design keeps the existing error structure but improves organization with clear groupings:

```rust
#[derive(Debug)]
pub enum ZervError {
    // VCS errors
    VcsNotFound(String),
    NoTagsFound,
    CommandFailed(String),

    // Version errors
    InvalidFormat(String),
    InvalidVersion(String),

    // Schema errors
    SchemaParseError(String),
    UnknownSchema(String),
    ConflictingSchemas(String),

    // CLI errors
    UnknownFormat(String),

    // System errors
    Io(io::Error),
    Regex(String),
}
```

This approach maintains the existing structure while improving readability and error message quality.

### Error Message Improvements

The main improvements focus on making error messages source-aware and user-friendly:

1. **VCS Error Messages**: Include specific source information (e.g., "git repository" instead of "VCS")
2. **Git Error Translation**: Convert technical git command output to user-friendly messages
3. **Actionable Context**: Provide clear guidance on what went wrong and how to fix it

## Components and Interfaces

### 1. Enhanced Error Display (`src/error.rs`)

**Responsibilities:**

- Improve error message formatting
- Add source-aware context to VCS errors
- Maintain existing error variant structure

**Interface:**

```rust
impl std::fmt::Display for ZervError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // VCS errors (improved messages)
            ZervError::VcsNotFound(msg) => write!(f, "{msg}"),
            ZervError::NoTagsFound => write!(f, "No version tags found in git repository"),
            ZervError::CommandFailed(msg) => write!(f, "{msg}"),

            // Version errors
            ZervError::InvalidFormat(msg) => write!(f, "Invalid version format: {msg}"),
            ZervError::InvalidVersion(msg) => write!(f, "Invalid version: {msg}"),

            // Schema errors
            ZervError::SchemaParseError(msg) => write!(f, "Schema parse error: {msg}"),
            ZervError::UnknownSchema(name) => write!(f, "Unknown schema: {name}"),
            ZervError::ConflictingSchemas(msg) => write!(f, "Conflicting schemas: {msg}"),

            // CLI errors
            ZervError::UnknownFormat(format) => write!(f, "Unknown format: {format}"),

            // System errors
            ZervError::Io(err) => write!(f, "IO error: {err}"),
            ZervError::Regex(msg) => write!(f, "Regex error: {msg}"),
        }
    }
}
```

### 2. Git Error Translation (`src/vcs/git.rs`)

**Responsibilities:**

- Execute git commands safely
- Translate common git errors to user-friendly messages
- Provide source-aware error context

**Interface:**

```rust
impl GitVcs {
    fn run_git_command(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| self.translate_command_error(e))?;

        if !output.status.success() {
            return Err(self.translate_git_error(&output.stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn translate_command_error(&self, error: std::io::Error) -> ZervError {
        match error.kind() {
            std::io::ErrorKind::NotFound => {
                ZervError::CommandFailed("Git command not found. Please install git and try again.".to_string())
            }
            std::io::ErrorKind::PermissionDenied => {
                ZervError::CommandFailed("Permission denied accessing git repository".to_string())
            }
            _ => ZervError::CommandFailed(format!("Failed to execute git: {error}"))
        }
    }

    fn translate_git_error(&self, stderr: &[u8]) -> ZervError {
        let stderr_str = String::from_utf8_lossy(stderr);

        // Pattern matching for common git errors
        if stderr_str.contains("fatal: ambiguous argument 'HEAD'") {
            return ZervError::CommandFailed("No commits found in git repository".to_string());
        }

        if stderr_str.contains("not a git repository") {
            return ZervError::VcsNotFound("Not in a git repository (--source git)".to_string());
        }

        if stderr_str.contains("Permission denied") {
            return ZervError::CommandFailed("Permission denied accessing git repository".to_string());
        }

        // Generic git command failure with cleaned up message
        ZervError::CommandFailed(format!("Git command failed: {stderr_str}"))
    }
}
```

### 3. VCS Detection Updates (`src/vcs/mod.rs`)

**Responsibilities:**

- Use source-aware errors in VCS detection
- Provide clear error messages

**Interface:**

```rust
pub fn detect_vcs(path: &Path) -> Result<Box<dyn Vcs>> {
    let git_vcs = git::GitVcs::new(path)?;
    if git_vcs.is_available(path) {
        return Ok(Box::new(git_vcs));
    }

    Err(ZervError::VcsNotFound("Not in a git repository (--source git)".to_string()))
}

pub fn find_vcs_root(start_path: &Path) -> Result<PathBuf> {
    // ... existing logic ...

    Err(ZervError::VcsNotFound("Not in a git repository (--source git)".to_string()))
}
```

### 4. Pipeline Error Updates (`src/pipeline/vcs_data_to_zerv_vars.rs`)

**Responsibilities:**

- Use appropriate error types instead of generic IO errors
- Provide clear error context

**Interface:**

```rust
pub fn vcs_data_to_zerv_vars(vcs_data: VcsData) -> Result<ZervVars, ZervError> {
    let version = if let Some(ref tag_version) = vcs_data.tag_version {
        parse_version_from_tag(tag_version, None).ok_or_else(|| {
            ZervError::InvalidFormat(format!(
                "Failed to parse version from tag: {tag_version}"
            ))
        })?
    } else {
        return Err(ZervError::NoTagsFound);
    };

    // ... rest of implementation
}
```

## Error Handling

### Error Translation Strategy

1. **Git Command Errors**: Translate `std::io::Error` from command execution to user-friendly messages
2. **Git Command Failures**: Parse stderr output and map to clear error messages
3. **VCS Detection**: Use source-aware error messages that mention "git repository" specifically
4. **Pipeline Errors**: Use appropriate error types instead of generic IO errors

### Error Message Improvements

**Before:**

- "IO error: No version tag found in VCS data"
- "VCS not found: No VCS repository found"
- "Command execution failed: Git command failed: fatal: ambiguous argument 'HEAD'"

**After:**

- "No version tags found in git repository"
- "Not in a git repository (--source git)"
- "No commits found in git repository"

## Testing Strategy

### Unit Tests

1. **Error Display Tests**: Verify improved error messages
2. **Error Translation Tests**: Test git error pattern matching
3. **VCS Detection Tests**: Test source-aware error messages

### Integration Tests

1. **No Tags Scenario**: Test error message when repository has no version tags
2. **No Repository Scenario**: Test error message when not in a git repository
3. **Empty Repository Scenario**: Test error message when repository has no commits
4. **Permission Denied Scenario**: Test error message for permission issues
5. **Git Not Installed Scenario**: Test error message when git command is not available

### Error Message Consistency Tests

```rust
#[rstest]
#[case("No version tags found in git repository")]
#[case("Not in a git repository (--source git)")]
#[case("No commits found in git repository")]
#[case("Git command not found. Please install git and try again.")]
#[case("Permission denied accessing git repository")]
fn test_error_message_format(#[case] expected: &str) {
    // Test that specific error scenarios produce expected messages
}
```

## Implementation Considerations

### Minimal Changes

- Keep existing error enum structure
- Focus on improving error message quality
- Add source-aware context to VCS errors
- Translate technical git errors to user-friendly messages

### Performance

- Error translation adds minimal overhead (only on error paths)
- Pattern matching on stderr is efficient for small error messages
- No impact on happy path performance

### Maintainability

- Clear error message patterns
- Centralized git error translation logic
- Easy to test individual error scenarios
- Improved code organization with grouped error variants

### Future Extensibility

- Error structure ready for new VCS types
- Easy to add new error message patterns
- Clear separation between error detection and message formatting
