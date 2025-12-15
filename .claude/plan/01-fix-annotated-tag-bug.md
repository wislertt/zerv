# Plan: Fix Annotated Tag Detection Bug

**Status**: Planned
**Priority**: High
**Context**: zerv flow and zerv version commands do not properly detect annotated tags, causing version detection to fail when repositories use annotated tags instead of lightweight tags.

## Goals

1. Add comprehensive support for annotated tags in test utilities
2. Create test cases that replicate the annotated tag bug
3. Fix any issues in tag detection logic for annotated tags
4. Ensure both annotated and lightweight tags work correctly

## Implementation Plan

### Phase 1: Enhance Test Infrastructure

1. **Add annotated tag support to GitOperations trait**
    - Extend the `GitOperations` trait with a new method: `create_annotated_tag(&self, test_dir: &TestDir, tag: &str, message: &str) -> io::Result<()>`
    - Implement for both `NativeGit` and `DockerGit`
    - Use `git tag -a <tag> -m "<message>"` command

2. **Add annotated tag support to GitRepoFixture**
    - Add `create_annotated_tag(self, tag: &str, message: &str) -> Self` method
    - Add `with_annotated_tag(self, tag: &str, message: &str) -> Self` builder-style method
    - Add static constructor: `GitRepoFixture::tagged_annotated(tag: &str, message: &str) -> Result<Self>`

### Phase 2: Create Comprehensive Test Cases

1. **Create annotated tag test mirroring existing lightweight tag test**
    - Duplicate `test_get_latest_tag_comprehensive` as `test_get_latest_tag_comprehensive_annotated`
    - Use annotated tags instead of lightweight tags
    - Test all scenarios: single tag, multiple tags, mixed formats, commit distances

2. **Test mixed tag type scenarios**
    - Repository with both annotated and lightweight tags on same commit
    - Repository with annotated tags on different commits
    - Timestamp behavior differences between tag types

3. **Test edge cases**
    - Annotated tag with special characters in message
    - Multiple annotated tags with different timestamps
    - Annotated tags with GPG signatures (if applicable)

### Phase 3: Investigate and Fix Bug

1. **Debug current tag detection with annotated tags**
    - Run existing git commands against repositories with annotated tags
    - Identify which commands fail or return unexpected results
    - Check if `git tag --merged` handles annotated tags correctly
    - Verify timestamp extraction works for both tag types

2. **Fix tag detection logic if needed**
    - Update `get_merged_tags()` if it doesn't include annotated tags
    - Fix timestamp handling in `get_tag_timestamp()` if there are issues
    - Ensure `get_tag_commit_hash()` works correctly for annotated tags

3. **Update git command execution**
    - Check if any git commands need different flags for annotated tags
    - Verify error handling covers annotated tag-specific failures

### Phase 4: Integration Testing

1. **Test with zerv flow command**
    - Create integration test using `TestCommand` with annotated tag repository
    - Verify version output is correct
    - Test all format options (auto, semver, pep440)

2. **Test with zerv version command**
    - Similar integration test for standalone version command
    - Verify consistency between flow and version commands

3. **Test real-world scenarios**
    - Import a real git repository with annotated tags
    - Test version detection on complex histories

## Testing Strategy

### Unit Tests

- Test new annotated tag creation methods in test utilities
- Test tag type detection (`git cat-file -t`)
- Test timestamp extraction for both tag types
- Test version parsing with annotated tags

### Integration Tests

- Use `ZERV_TEST_NATIVE_GIT=false ZERV_TEST_DOCKER=true` for Docker-based tests
- Create comprehensive test scenarios covering all tag combinations
- Test with multiple version formats

### Manual Verification

- Create test repositories manually with annotated tags
- Verify zerv commands work correctly
- Compare behavior with lightweight tags

## Success Criteria

1. ✅ All tests pass with annotated tags
2. ✅ zerv flow detects annotated tags correctly
3. ✅ zerv version detects annotated tags correctly
4. ✅ Mixed repositories (both tag types) work correctly
5. ✅ No regression in lightweight tag functionality
6. ✅ Test coverage includes annotated tag scenarios

## Documentation Updates

1. Update testing documentation to mention annotated tag support
2. Add examples of creating annotated tags in tests
3. Document any differences in behavior between tag types

## Files to Modify

### New Files

- None

### Modified Files

- `src/test_utils/git/mod.rs` - Add annotated tag method to trait
- `src/test_utils/git/native.rs` - Implement annotated tag creation
- `src/test_utils/git/docker.rs` - Implement annotated tag creation with verification
- `src/test_utils/git/fixtures.rs` - Add annotated tag fixture methods
- `src/vcs/git.rs` - Add comprehensive annotated tag tests
- `tests/integration_tests/` - Add integration tests for annotated tags

## Risk Assessment

**Low Risk**:

- Adding new methods to test utilities doesn't affect existing code
- Tests are additive and don't change existing behavior

**Medium Risk**:

- Bug fixes to tag detection logic might affect lightweight tag behavior
- Need thorough regression testing

**Mitigation**:

- Run full test suite after each change
- Keep lightweight tag tests intact
- Test with real repositories before merging
