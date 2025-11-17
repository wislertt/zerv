# Fix Git Tag Resolution Bug

## Status

**Planned**

## Priority

**High** - Critical bug affecting version resolution in Zerv Flow

## Context

The current `get_latest_tag` function in `src/vcs/git.rs` has a critical bug that affects version resolution:

1. **Multiple tags on same commit**: When two tags exist on the same commit (e.g., `v1.0.2-rc.1.post.3` and `v1.1.0`), `git describe --tags --abbrev=0` incorrectly returns the pre-release tag instead of the clean release tag.

2. **Non-version tags**: The function doesn't filter out non-version tags like `some-tag-like-this`, which can cause parsing failures.

3. **Redundant code**: There are two similar parsing functions - `VersionObject::parse_with_format` and `InputFormatHandler::parse_version_string` that should be consolidated.

## Current Problematic Code

```rust
/// Get latest version tag
fn get_latest_tag(&self) -> Result<Option<String>> {
    match self.run_git_command(&["describe", "--tags", "--abbrev=0"]) {
        Ok(tag) if !tag.is_empty() => Ok(Some(tag)),
        Ok(_) => Ok(None),
        Err(ZervError::CommandFailed(_)) => Ok(None), // No tags found
        Err(e) => Err(e),
    }
}
```

## Updated Function Signature

The `get_latest_tag` function should accept a format parameter:

```rust
/// Get latest version tag
fn get_latest_tag(&self, format: &str) -> Result<Option<String>> {
    // Implementation with format parameter
}
```

## New Implementation (Based on Research)

```rust
/// Get latest version tag
fn get_latest_tag(&self, format: &str) -> Result<Option<String>> {
    // Get all tags traceable from current commit, ordered by reverse commit date
    let output = match self.run_git_command(&["tag", "--merged", "HEAD", "--sort=-committerdate"]) {
        Ok(tags) => tags,
        Err(ZervError::CommandFailed(_)) => return Ok(None), // No tags found
        Err(e) => return Err(e),
    };

    if output.is_empty() {
        return Ok(None);
    }

    // For each tag in chronological order (newest first)
    for tag in output.lines() {
        let trimmed_tag = tag.trim();
        if trimmed_tag.is_empty() {
            continue;
        }

        // Check if tag is parsable as a version using the provided format
        if VersionObject::parse_with_format(trimmed_tag, format).is_none() {
            continue; // Skip non-version tags
        }

        // Get the commit hash this tag points to
        let commit_hash = match self.run_git_command(&["rev-list", "-n", "1", trimmed_tag]) {
            Ok(hash) => hash.trim().to_string(),
            Err(_) => continue, // Skip if we can't get commit hash
        };

        // Get all tags pointing to this same commit
        let tags_on_commit = match self.run_git_command(&["tag", "--points-at", &commit_hash]) {
            Ok(tags) => tags,
            Err(_) => continue,
        };

        if tags_on_commit.is_empty() {
            continue;
        }

        // Find the highest semantic version among all tags on this commit
        // Use iterator pipeline for efficiency - no Vec allocation, no sorting needed
        let best_tag = tags_on_commit
            .lines()
            .map(|line| line.trim())
            .filter(|tag| !tag.is_empty())
            .filter(|tag| VersionObject::parse_with_format(tag, format).is_some())
            .max();

        if let Some(best_tag) = best_tag {
            return Ok(Some(best_tag.to_string()));
        }
    }

    Ok(None) // No valid version tags found
}
```

## Goals

1. **Fix multi-tag resolution**: When multiple tags exist on the same commit, prioritize clean releases over pre-releases
2. **Filter non-version tags**: Only return tags that can be parsed as valid versions
3. **Consolidate parsing logic**: Use `VersionObject::parse_with_format` as the single parsing method
4. **Maintain backward compatibility**: Ensure existing functionality continues to work

## Implementation Plan

### Step 1: Research Git Commands

- Test various git commands to find the best approach for getting all tags from HEAD
- Identify the optimal sorting strategy to prioritize clean releases over pre-releases
- Validate that the chosen approach works with the test case from `.cache/tmp/v1.1.0-tag-created-1762874069/.tmpGBjAVY`

### Step 2: Update get_latest_tag Function

- Add format parameter to function signature: `fn get_latest_tag(&self, format: &str)`
- Replace `git describe` with research-based algorithm using `git tag --merged HEAD --sort=-committerdate`
- Implement iterative approach: for each parsable tag, get its commit and all tags on that commit
- Use semantic version sorting to find highest version among tags on the same commit
- Return highest version tag from newest parsable commit (prioritizes clean releases over pre-releases)

### Step 3: Integrate VersionObject Parsing

- Modify the function to use `VersionObject::parse_with_format(format)` for validation with the passed format parameter
- Remove or deprecate `InputFormatHandler::parse_version_string` if redundant
- Add proper error handling for parsing failures
- Update all callers of `get_latest_tag()` to pass appropriate format parameter

### Step 4: Update Error Handling

- Ensure parsing errors are handled gracefully
- Return appropriate errors when no valid version tags are found
- Maintain existing error handling patterns

### Step 5: Add Comprehensive Tests

- Test the specific bug scenario: `v1.0.2-rc.1.post.3` and `v1.1.0` on same commit
- Test with non-version tags mixed in
- Test with various tag formats (SemVer, PEP440)
- Test backward compatibility scenarios

### Step 6: Update Documentation and Comments

- Document the new tag resolution strategy
- Add examples of the prioritization logic
- Update any related documentation

## Technical Details

### Proposed Algorithm (Updated Based on Research)

1. Get all tags traceable from current commit: `git tag --merged HEAD --sort=-committerdate`
2. For each tag in chronological order (newest first):
    - Check if the tag is parsable using `VersionObject::parse_with_format` with auto-detection
    - If parsable, get the commit hash: `git rev-list -n 1 <tag>`
    - Get all tags pointing to that commit: `git tag --points-at <commit>`
    - From all tags on that commit, select the highest semantic version
    - Return the highest version tag (prioritizes clean releases over pre-releases)
3. If no parsable tags found in history, return None

### Key Insights from Research

- `git tag --merged HEAD --sort=-committerdate` gives all tags traceable from current commit in reverse commit order
- Multiple tags can point to the same commit (e.g., `v1.0.2-rc.1.post.3` and `v1.1.0` both point to `2cdf46bfcb9d3a52c2d6a98006eabe42152212f5`)
- Semantic version sorting (`--sort=-version:refname`) automatically prioritizes clean releases over pre-releases
- Algorithm must handle the case where tags from different commits are compared, not just tags on the same commit

### Tag Priority Strategy

1. **Clean releases** (e.g., `v1.1.0`) over pre-releases (e.g., `v1.0.2-rc.1.post.3`)
2. **Higher semantic versions** over lower versions (when both are same type)
3. **SemVer over PEP440** when both are valid and equivalent
4. **Annotated tags over lightweight tags** (when all else is equal)

### Integration Points

- `src/vcs/git.rs` - Main implementation location with updated `get_latest_tag(&self, format: &str)` signature
- `src/version/version_object.rs` - Use `VersionObject::parse_with_format`
- `src/cli/utils/format_handler.rs` - Potentially remove redundant code
- All callers of `get_latest_tag()` - Update to pass format parameter
- Test files in `src/vcs/git.rs` and related integration tests - Update to test with different formats

## Testing Strategy

### Unit Tests

- Test `get_latest_tag` with various git repository states
- Mock git commands to test error scenarios
- Test parsing integration with `VersionObject::parse_with_format`

### Integration Tests

- Use the cached test directory `.cache/tmp/v1.1.0-tag-created-1762874069/.tmpGBjAVY`
- Test with actual git repositories containing multiple tags
- Test Zerv Flow scenarios that depend on correct tag resolution

### Regression Tests

- Ensure existing tag resolution continues to work
- Test edge cases like shallow clones, no tags, only non-version tags
- Verify performance with large numbers of tags

## Success Criteria

1. ✅ `v1.1.0` is returned instead of `v1.0.2-rc.1.post.3` when both exist on same commit
2. ✅ Non-version tags are filtered out gracefully
3. ✅ Existing functionality remains intact
4. ✅ Tests pass for all scenarios including edge cases
5. ✅ Code uses `VersionObject::parse_with_format` consistently
6. ✅ Error messages are clear and helpful

## Documentation Updates

- Update inline documentation for `get_latest_tag` function
- Add examples to the Git VCS module documentation
- Update testing documentation if needed
- Consider updating the Zerv Flow plan documentation with the fix

## Risk Mitigation

- **Breaking changes**: Ensure the function signature and return types remain the same
- **Performance**: Test with repositories containing many tags to ensure no performance regression
- **Edge cases**: Handle repositories with no tags, only non-version tags, or corrupted git states
- **Compatibility**: Ensure the fix works across different git versions and configurations
