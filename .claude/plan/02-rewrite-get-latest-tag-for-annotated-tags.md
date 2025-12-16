# Plan: Rewrite get_latest_tag to Properly Detect Annotated Tags

**Status**: Planned
**Priority**: High
**Context**: The current `get_latest_tag` implementation in src/vcs/git.rs cannot properly detect annotated tags because it uses `git tag --merged HEAD` which doesn't correctly handle the topological order of commits and their associated tags. The TODO comment in the code outlines the correct approach.

## Goals

1. Rewrite `get_latest_tag` method to use topological commit traversal
2. Ensure both annotated and lightweight tags are detected correctly
3. Make `test_get_latest_tag_comprehensive_annotated` test pass
4. Maintain backward compatibility with existing functionality

## Implementation Plan

### Step 1: Understand the Current Algorithm's Limitations

The current algorithm:

1. Gets all merged tags using `git tag --merged HEAD --sort=-committerdate`
2. Finds the latest valid version tag from this list
3. Gets the commit hash for that tag
4. Gets all tags pointing to that commit
5. Filters and finds the maximum version tag

**Problems:**

- `git tag --merged` doesn't guarantee topological order
- May miss tags on commits that are not direct ancestors
- Doesn't properly handle multiple tags on the same commit

### Step 2: Implement the New Algorithm

Following the TODO comment's guidance, the new algorithm will:

1. **Get all commits from HEAD in topological order**

    ```bash
    git rev-list --topo-order HEAD
    ```

    This returns commits in topological order (parents before children)

2. **Iterate through each commit and find tags pointing at it**

    ```bash
    git tag --points-at <commit-hash>
    ```

3. **Filter and validate tags for each commit**
    - Use `GitUtils::filter_only_valid_tags(&tags, format)`
    - If no valid tags found, continue to next commit

4. **Find the maximum version tag among valid tags**
    - Use `GitUtils::find_max_version_tag(&valid_tags)`
    - Return the first found (since we're in topological order)

### Step 3: Code Implementation

First, add a helper method to get commits in topological order:

```rust
/// Get all commits from HEAD in topological order
fn get_commits_in_topo_order(&self) -> Result<Vec<String>> {
    let commits_output = self.run_git_command(&["rev-list", "--topo-order", "HEAD"])?;
    Ok(commits_output
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|hash| !hash.is_empty())
        .collect())
}
```

Then, the new `get_latest_tag` method will reuse the existing `get_all_tags_from_commit_hash`:

```rust
fn get_latest_tag(&self, format: &str) -> Result<Option<String>> {
    // Get all commits from HEAD in topological order
    let commits = self.get_commits_in_topo_order()?;

    // Process each commit in topological order
    for commit_hash in commits {
        // Get all tags pointing to this commit (reusing existing function)
        let tags = self.get_all_tags_from_commit_hash(&commit_hash)?;

        // If no tags, continue to next commit
        if tags.is_empty() {
            continue;
        }

        // Filter tags by format
        let valid_tags = GitUtils::filter_only_valid_tags(&tags, format);

        // If no valid tags, continue to next commit
        if valid_tags.is_empty() {
            continue;
        }

        // Find and return the maximum version tag
        if let Some(max_tag) = GitUtils::find_max_version_tag(&valid_tags)? {
            return Ok(Some(max_tag));
        }
    }

    // No valid tags found
    Ok(None)
}
```

Note: No changes needed to `get_tags_pointing_at_commit` or `get_all_tags_from_commit_hash` - we'll reuse the existing `get_all_tags_from_commit_hash` function.

### Step 4: Testing Strategy

1. **Enable the commented-out test cases**
    - Uncomment all test scenarios in `test_get_latest_tag_comprehensive_annotated`
    - Ensure the test uses proper annotated tag creation

2. **Run the test to verify the fix**

    ```bash
    ZERV_TEST_NATIVE_GIT=false ZERV_TEST_DOCKER=true cargo test test_get_latest_tag_comprehensive_annotated
    ```

3. **Run existing tests to ensure no regression**
    ```bash
    ZERV_TEST_NATIVE_GIT=false ZERV_TEST_DOCKER=true cargo test
    ```

### Step 5: Additional Considerations

1. **Performance**: The new algorithm may be slower as it checks each commit
    - Consider adding early termination if we find a tag on HEAD
    - Cache results if needed

2. **Edge cases to handle**:
    - Empty repositories (no commits)
    - Repositories with no tags
    - Mixed annotated and lightweight tags
    - Tags with GPG signatures

3. **Error handling**:
    - Handle empty output from `git rev-list`
    - Ensure proper error propagation

## Success Criteria

1. ✅ `test_get_latest_tag_comprehensive_annotated` test passes
2. ✅ All existing tag-related tests still pass
3. ✅ Both annotated and lightweight tags work correctly
4. ✅ No performance regression for typical repositories
5. ✅ Proper error handling for edge cases

## Files to Modify

### Modified Files

- `src/vcs/git.rs` - Add the `get_commits_in_topo_order` helper method and rewrite the `get_latest_tag` method (lines 131-151) to use topological commit traversal

## Risk Assessment

**Medium Risk**:

- Changing a core algorithm that affects version detection
- Potential performance impact on large repositories

**Mitigation**:

- Thorough testing with various repository types
- Performance benchmarking on large commit histories
- Keep the existing algorithm as a fallback if needed

## Notes

- The implementation follows the exact guidance from the TODO comment
- This approach is more robust as it checks commits in topological order
- The algorithm will work correctly for both annotated and lightweight tags
- No changes needed to test utilities as they already support annotated tags
