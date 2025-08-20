# VCS Design Document

## Overview

Design for Version Control System (VCS) integration to populate `ZervVars` with repository metadata.

## Core Architecture

### VCS Trait

```rust
/// Version format specification for tag parsing
#[derive(Debug, Clone)]
pub enum VersionFormat {
    Auto,      // Try SemVer first, then PEP440
    SemVer,    // Force SemVer parsing only
    Pep440,    // Force PEP440 parsing only
}

/// Shared utility for parsing version tags across all VCS systems
pub fn parse_tag_to_zerv_vars(tag: &str, format: VersionFormat) -> Result<ZervVars, VcsError> {
    match format {
        VersionFormat::Auto => {
            // Try SemVer first (more common in Git repositories)
            if let Ok(semver) = SemVer::parse(tag) {
                return Ok(semver.to_zerv_vars());
            }
            // Try PEP440 second (Python-specific)
            if let Ok(pep440) = Pep440::parse(tag) {
                return Ok(pep440.to_zerv_vars());
            }
            Err(VcsError::ParseError(format!("Invalid version tag: {}", tag)))
        }
        VersionFormat::SemVer => {
            SemVer::parse(tag)
                .map(|v| v.to_zerv_vars())
                .map_err(|_| VcsError::ParseError(format!("Invalid SemVer tag: {}", tag)))
        }
        VersionFormat::Pep440 => {
            Pep440::parse(tag)
                .map(|v| v.to_zerv_vars())
                .map_err(|_| VcsError::ParseError(format!("Invalid PEP440 tag: {}", tag)))
        }
    }
}

pub trait Vcs {
    /// Detect if this VCS is available in the current directory
    fn detect() -> bool;

    /// Extract VCS data for version generation
    fn extract_data(&self) -> Result<VcsData, VcsError>;

    /// Extract complete ZervVars with version parsing (default implementation)
    fn extract_zerv_vars(&self, format: Option<VersionFormat>) -> Result<ZervVars, VcsError> {
        let vcs_data = self.extract_data()?;
        let version_format = format.unwrap_or(VersionFormat::Auto);

        let mut zerv_vars = if let Some(tag) = &vcs_data.tag_version {
            parse_tag_to_zerv_vars(tag, version_format)?
        } else {
            ZervVars::default()
        };

        zerv_vars.merge_vcs_data(vcs_data);
        Ok(zerv_vars)
    }
}

pub struct VcsData {
    pub tag_version: Option<String>,     // Raw version tag (e.g., "v1.2.3")
    pub tag_timestamp: Option<u64>,
    pub tag_branch: Option<String>,
    pub current_branch: Option<String>,
    pub distance: Option<u64>,
    pub dirty: Option<bool>,
    pub tag_commit_hash: Option<String>,
    pub current_commit_hash: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum VcsError {
    #[error("Repository not found")]
    NotFound,
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    #[error("Invalid output format: {0}")]
    ParseError(String),
    #[error("No version tags found in repository")]
    NoTagsFound(String),
    #[error("Empty repository - no commits found")]
    EmptyRepository,
    #[error("Detached HEAD state - cannot determine branch")]
    DetachedHead,
    #[error("Repository appears corrupted: {0}")]
    CorruptedRepository(String),
    #[error("Permission denied accessing repository: {0}")]
    PermissionDenied(String),
}
```

### Git Implementation

```rust
pub struct GitVcs {
    repo_path: PathBuf,
}

impl GitVcs {
    pub fn new() -> Result<Self, VcsError> {
        let repo_path = Self::find_git_root()?;
        Ok(Self { repo_path })
    }

    fn find_git_root() -> Result<PathBuf, VcsError> {
        // Walk up directories looking for .git
    }

    fn run_git_command(&self, args: &[&str]) -> Result<String, VcsError> {
        // Execute git command and return output
    }

    fn get_latest_tag(&self) -> Result<Option<String>, VcsError> {
        // git describe --tags --abbrev=0
    }

    fn get_tag_info(&self, tag: &str) -> Result<TagInfo, VcsError> {
        // Detect tag type and get appropriate timestamp
        let tag_type = self.run_git_command(&["cat-file", "-t", tag])?;
        let timestamp = match tag_type.trim() {
            "tag" => self.get_tag_creation_date(tag)?,    // Annotated
            "commit" => self.get_commit_date(tag)?,       // Lightweight
            _ => return Err(VcsError::InvalidTagType),
        };
        // Extract other tag info...
    }

    fn get_tag_creation_date(&self, tag: &str) -> Result<u64, VcsError> {
        // git for-each-ref --format='%(taggerdate:unix)' refs/tags/<tag>
    }

    fn get_commit_date(&self, tag: &str) -> Result<u64, VcsError> {
        // git log -1 --format=%ct <tag>
    }

    fn get_current_commit(&self) -> Result<String, VcsError> {
        // git rev-parse HEAD
    }

    fn get_current_branch(&self) -> Result<Option<String>, VcsError> {
        // git branch --show-current
    }

    fn calculate_distance(&self, from_tag: &str) -> Result<u64, VcsError> {
        // Use Git's native graph traversal - handles non-linear history correctly
        let output = self.run_git_command(&[
            "rev-list", "--count", &format!("refs/tags/{}..HEAD", from_tag)
        ])?;
        output.trim().parse().map_err(|_| VcsError::ParseError("Invalid distance count".to_string()))
    }

    fn is_dirty(&self) -> Result<bool, VcsError> {
        // git status --porcelain
    }
}

struct TagInfo {
    commit_hash: String,
    timestamp: u64,
    branch: Option<String>,
}
```

## Data Flow

```
GitVcs::extract_zerv_vars()
    ↓
GitVcs::extract_data()
├── get_latest_tag() → Option<String>
├── get_tag_info(tag) → TagInfo
├── get_current_commit() → String
├── get_current_branch() → Option<String>
├── calculate_distance(tag) → u64
└── is_dirty() → bool
    ↓
VcsData {
    tag_version: get_latest_tag(),
    tag_timestamp: TagInfo.timestamp,
    tag_branch: TagInfo.branch,
    current_branch: get_current_branch(),
    distance: calculate_distance(),
    dirty: is_dirty(),
    tag_commit_hash: TagInfo.commit_hash,
    current_commit_hash: get_current_commit(),
}
    ↓
parse_tag_to_zerv_vars(tag_version, format) → ZervVars (major, minor, patch, etc.)
    ↓
zerv_vars.merge_vcs_data(vcs_data) → Complete ZervVars
```

## Git Commands Mapping

| Field                 | Git Command                        | Notes                 |
| --------------------- | ---------------------------------- | --------------------- |
| `tag_version`         | `git describe --tags --abbrev=0`   | Raw version tag       |
| `tag_timestamp`       | `git log -1 --format=%ct <tag>`    | Unix timestamp        |
| `tag_branch`          | `git branch --contains <tag>`      | Branch containing tag |
| `current_branch`      | `git branch --show-current`        | Current branch name   |
| `distance`            | `git rev-list --count <tag>..HEAD` | Commits since tag     |
| `dirty`               | `git status --porcelain`           | Working tree clean?   |
| `tag_commit_hash`     | `git rev-list -n 1 <tag>`          | Tag's commit hash     |
| `current_commit_hash` | `git rev-parse HEAD`               | Current commit hash   |

## ZervVars Integration

```rust
impl ZervVars {
    /// Merge VCS data into existing ZervVars
    pub fn merge_vcs_data(&mut self, vcs_data: VcsData) {
        self.tag_timestamp = vcs_data.tag_timestamp;
        self.tag_branch = vcs_data.tag_branch;
        self.current_branch = vcs_data.current_branch;
        self.distance = vcs_data.distance;
        self.dirty = vcs_data.dirty;
        self.tag_commit_hash = vcs_data.tag_commit_hash;
        self.current_commit_hash = vcs_data.current_commit_hash;
    }
}
```

## Usage Pattern

```rust
// Simple usage - auto-detect format (default)
let git = GitVcs::new()?;
let zerv_vars = git.extract_zerv_vars(None)?;

// Force specific format
let zerv_vars = git.extract_zerv_vars(Some(VersionFormat::SemVer))?;

// Advanced usage - separate steps for custom processing
let vcs_data = git.extract_data()?;
let mut zerv_vars = if let Some(tag) = &vcs_data.tag_version {
    parse_tag_to_zerv_vars(tag, VersionFormat::Auto)?
} else {
    ZervVars::default()
};
zerv_vars.merge_vcs_data(vcs_data);

// Auto-detection pattern with format option
fn get_zerv_vars_from_vcs(format: Option<VersionFormat>) -> Result<ZervVars, VcsError> {
    if GitVcs::detect() {
        let git = GitVcs::new()?;
        git.extract_zerv_vars(format)
    } else {
        Err(VcsError::NotFound)
    }
}
```

## CLI Integration

```bash
# Auto-detect format (default behavior)
zerv version

# Force SemVer parsing
zerv version --tag-format semver

# Force PEP440 parsing
zerv version --tag-format pep440
```

**Benefits:**

- **Explicit control** - User specifies exact format when needed
- **Better error messages** - Clear about which format failed
- **Performance** - Skip unnecessary parsing attempts when format is known
- **Handles edge cases** - When auto-detection picks wrong format

## Performance Optimization: Batched Git Commands

```rust
fn extract_data_optimized(&self) -> Result<VcsData, VcsError> {
    // Batch 1: Get current commit info + latest tag
    let format = "%H|%ct|%D";  // hash|timestamp|refs
    let current_info = self.run_git_command(&[
        "log", "-1", "--format", format, "HEAD"
    ])?;

    // Batch 2: Get tag info if tag exists
    let tag = self.run_git_command(&["describe", "--tags", "--abbrev=0"])
        .ok();
    let tag_info = if let Some(ref t) = tag {
        let tag_format = "%H|%ct";  // tag_hash|tag_timestamp
        Some(self.run_git_command(&[
            "log", "-1", "--format", tag_format, t
        ])?)
    } else {
        None
    };

    // Batch 3: Get status and branch (can't batch these easily)
    let dirty = self.run_git_command(&["status", "--porcelain"])?;
    let branch = self.run_git_command(&["branch", "--show-current"])?;

    // Parse batched outputs
    let current_parts: Vec<&str> = current_info.trim().split('|').collect();

    Ok(VcsData {
        tag_version: tag,
        current_commit_hash: Some(current_parts[0].to_string()),
        tag_timestamp: tag_info.and_then(|info| {
            info.split('|').nth(1)?.parse().ok()
        }),
        dirty: Some(!dirty.trim().is_empty()),
        current_branch: if branch.trim().is_empty() { None } else { Some(branch.trim().to_string()) },
        // ... other fields
    })
}
```

### Performance Benefits

- **Reduced from 8 to 3-4 commands** (60-75% fewer process spawns)
- **Faster execution** especially on large repositories
- **Atomic data** - all info from consistent Git state
- **Better I/O efficiency** - less disk access

### Git Format Specifiers

| Specifier | Output                  | Example                     |
| --------- | ----------------------- | --------------------------- |
| `%H`      | Full commit hash        | `abc123def456...`           |
| `%h`      | Short commit hash       | `abc123d`                   |
| `%ct`     | Commit timestamp (Unix) | `1640995200`                |
| `%D`      | Ref names               | `HEAD -> main, tag: v1.2.3` |
| `%s`      | Subject line            | `Add new feature`           |
| `%an`     | Author name             | `John Doe`                  |

## Implementation Notes

1. **Command Execution**: Use `std::process::Command` with proper error handling
2. **Path Handling**: Use `std::path::PathBuf` for cross-platform compatibility
3. **Output Parsing**: Trim whitespace, handle empty outputs gracefully
4. **Performance**: Use batched Git commands, cache Git root discovery
5. **Testing**: Mock command execution for unit tests
6. **Error Resilience**: Handle missing tags, empty repos, detached HEAD states

### Git Command Edge Cases

**Fatal Errors (Exit with clear message):**

- **No tags exist**: `git describe --tags --abbrev=0` fails
- **No commits**: New/empty repository
- **Detached HEAD**: Branch detection fails
- **Corrupted repo**: Git commands return unexpected output
- **Permission issues**: Git commands fail due to access rights

**Recoverable Warnings (Continue with degraded data):**

- **Shallow clone**: Missing history for distance calculation - warn user about potential inaccuracy

**Implementation Strategy:**

```rust
// Fatal error example
if no_tags_exist() {
    return Err(VcsError::NoTagsFound("No version tags found in repository"));
}

// Recoverable warning example
if is_shallow_clone() {
    eprintln!("Warning: Shallow clone detected - distance calculations may be inaccurate");
    // Continue with available data
}
```

### VCS Data Field Handling

**Tag Type Detection & Timestamp Strategy:**

- **Annotated tags**: Use tag creation date (preferred)
- **Lightweight tags**: Use commit date (fallback)
- **Detection**: `git cat-file -t <tag>` returns "tag" (annotated) or "commit" (lightweight)

**Multiple Tags on Same Commit:**

- Use first tag found
- Warn user about additional tags: "Warning: Multiple tags found on commit, using: v1.0.0"

**Version Metadata Handling:**

- **Pre-release info** (alpha, beta, rc): Handled by version objects (SemVer, PEP440)
- **Build metadata**: Extracted and parsed by version-specific parsers

**Implementation Strategy:**

```rust
// Tag type detection and timestamp
let tag_type = run_git_command(&["cat-file", "-t", tag])?;
let timestamp = match tag_type.trim() {
    "tag" => get_tag_creation_date(tag)?,      // Annotated tag
    "commit" => get_commit_date(tag)?,         // Lightweight tag
    _ => return Err(VcsError::InvalidTagType),
};

// Multiple tags handling
let all_tags = get_tags_on_commit(commit_hash)?;
if all_tags.len() > 1 {
    eprintln!("Warning: Multiple tags found on commit, using: {}", all_tags[0]);
}
```

## Future Extensions

- **Multiple VCS**: Add `HgVcs`, `SvnVcs` implementations
- **Caching**: Cache command results within single execution
- **Configuration**: Config file support for default format preferences
- **Validation**: Verify Git repository integrity before operations

## File Structure

```
src/vcs/
├── mod.rs          # VCS trait, error types, detection logic
└── git.rs          # Git implementation
```

### Git History Handling

**Distance Calculation Strategy:**
Use `git rev-list --count refs/tags/{tag}..HEAD` for robust distance calculation that handles:

- **Non-linear history** - Git natively traverses commit graph correctly
- **Merge commits** - Automatically handled in graph traversal
- **Complex branching** - Works with GitFlow, GitHub Flow, and any workflow
- **Tag reachability** - Git's `..` syntax handles reachability correctly

**Why This Works:**

- Industry standard approach used by mature tools (dunamai, etc.)
- Git's `rev-list` command natively handles all history complexities
- The `..` range syntax correctly excludes commits reachable from tag
- Works regardless of branching strategy or merge patterns

### Error Recovery & Fallbacks

**Error Handling Strategy:**

**Fatal Errors (Exit non-zero with clear message):**

- Git command failures (repository corruption, permission issues)
- No tags found when version detection is required
- Invalid repository state that prevents operation

**Recoverable Scenarios (Continue with warnings):**

- Partial data extraction - extract as much as possible
- Shallow clone - warn about distance calculation accuracy
- Missing optional information - continue with available data

**Implementation:**

```rust
// Fatal error handling
if git_command_failed() {
    return Err(VcsError::CommandFailed("Git command failed: {}".format(error)));
}

// Recoverable warning handling
if is_shallow_clone() {
    eprintln!("Warning: Shallow clone detected - distance calculations may be inaccurate");
    // Continue with available data
}

// Partial data extraction
let mut vcs_data = VcsData::default();
vcs_data.tag_version = get_latest_tag().ok(); // Continue if fails
vcs_data.current_branch = get_current_branch().ok(); // Continue if fails
vcs_data.distance = calculate_distance().unwrap_or(0); // Use 0 as fallback
```

**Benefits:**

- Clear distinction between fatal vs recoverable issues
- User-friendly error messages and warnings
- Graceful degradation with partial data
- Robust handling of real-world repository states

### Testing Strategy

**Approach: Temp Folder + Real Git Commands**
Use temporary directories with actual Git repositories for testing instead of mocking.

**Implementation:**

```rust
#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use std::process::Command;

    fn setup_git_repo_with_tag(tag: &str) -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create real Git repo with specific state
        Command::new("git").args(["init"]).current_dir(path).output().unwrap();
        Command::new("git").args(["commit", "--allow-empty", "-m", "initial"])
            .current_dir(path).output().unwrap();
        Command::new("git").args(["tag", tag])
            .current_dir(path).output().unwrap();

        temp_dir
    }
}
```

**Cross-Platform Handling:**

- Use Git porcelain commands (stable across platforms)
- Normalize output (handle CRLF vs LF)
- CI testing on Windows, macOS, Linux
- Use `std::path::PathBuf` for cross-platform paths

**Benefits:**

- Tests real Git behavior instead of mocked behavior
- Easy reproduction of edge cases and various repository states
- Cross-platform compatibility validated through CI
- No complex mocking infrastructure needed

### Custom Tag Pattern Matching

**Current Implementation:**
Supports standard version formats via `VersionFormat` enum:

- `Auto` - Try SemVer first, then PEP440
- `SemVer` - Force SemVer parsing only
- `Pep440` - Force PEP440 parsing only

**Future Extension (TODO):**

```rust
pub enum VersionFormat {
    Auto,
    SemVer,
    Pep440,
    // TODO: Add custom regex support
    Custom(String),  // User-provided regex pattern
}

// TODO: Implementation for custom patterns
match format {
    VersionFormat::Custom(regex) => {
        // Parse using user-provided regex
        // Extract version components from captures
    }
    // ... existing cases
}
```

**Implementation Strategy:**

- Leave TODO comments in code for custom regex support
- Current design is easily extensible for future custom patterns
- Standard formats (SemVer/PEP440) cover majority of use cases for alpha

### Branch Filtering Options

**Current Implementation:**
Uses current branch/HEAD by default (covers 95% of use cases).

**Future Extension (TODO):**
Implement `--tag-branch` option following dunamai's approach:

```rust
// TODO: Add optional tag_branch parameter
pub struct GitVcs {
    repo_path: PathBuf,
    tag_branch: Option<String>,  // TODO: Add this field
}

// TODO: Modify tag discovery to use --merged filtering
fn get_tags_on_branch(&self, branch: Option<&str>) -> Result<Vec<String>, VcsError> {
    let branch = branch.unwrap_or("HEAD");
    let output = self.run_git_command(&[
        "for-each-ref", "refs/tags/**",
        "--merged", branch,  // TODO: Add --merged filtering
        "--format", "%(refname:short)"
    ])?;
    // Parse and return tags
}
```

**CLI Integration (TODO):**

```bash
# Default: use current branch
zerv version

# TODO: Add --tag-branch option
zerv version --tag-branch main
zerv version --tag-branch release/1.0
```

**Implementation Strategy:**

- Optional feature - defaults to current branch behavior
- Uses Git's native `--merged` filtering like dunamai
- Leave TODO comments for future implementation

This design provides a clean separation between VCS abstraction and Git-specific implementation, making it easy to populate `ZervVars` with repository metadata.
