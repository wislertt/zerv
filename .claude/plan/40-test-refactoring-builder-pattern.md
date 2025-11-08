# Test Refactoring: Builder Pattern for Flow Pipeline Tests

## Overview

Refactor the `test_trunk_based_development_flow()` function to use a builder pattern for better readability and maintainability. This will enable implementation of the comprehensive trunk-based development workflow from the design document while keeping the existing test intact until the new approach is fully functional.

## Current State Analysis

### Existing Test Structure

- **Location**: `src/cli/flow/pipeline.rs:186-310`
- **Function**: `test_trunk_based_development_flow()`
- **Length**: ~125 lines with basic trunk-based workflow
- **Current Coverage**:
    - Main branch state
    - Feature branch creation
    - Dirty state testing
    - Schema variant testing (6 variants)

### Issues with Current Approach

1. **Monolithic function**: All logic in one large test function
2. **Limited workflow**: Only covers basic scenarios, not full trunk-based development
3. **Hard to extend**: Adding more steps will make it unreadable
4. **Duplicated setup**: Repeated fixture and path handling
5. **Poor narrative**: Doesn't follow the design document workflow story

### Target Workflow (from plan/32-zerv-flow-implementation-plan.md)

The design document shows a comprehensive trunk-based development workflow:

- Multiple parallel feature branches (feature-1, feature-2, feature-3)
- Branch synchronization and merging
- Nested feature branches
- Version progression across different states
- Proper schema testing at each step

## Migration Strategy: Parallel Development

### Phase 1: Create Builder Pattern Infrastructure

**Goal**: Build the builder pattern alongside existing tests without breaking anything

1. **Create new test utilities file**
    - Location: `src/cli/flow/test_utils.rs`
    - Purpose: Separate growing test utilities from pipeline logic
    - Contains: FlowTestScenario, SchemaTestCase, and test helper functions

2. **Move existing test utilities**
    - Move `SchemaTestCase` from `pipeline.rs` to new file
    - Move existing test functions to new file
    - Update imports in `pipeline.rs`

3. **Create `FlowTestScenario` struct**
    - Purpose: Encapsulate fixture management and state tracking
    - Contains: GitRepoFixture, fixture_path, current_branch, current_hash

4. **Implement core builder methods**
    - `FlowTestScenario::new()` - Initial setup
    - `expect_version(semver, pep440)` - Test specific version expectations
    - `create_branch(branch_name)` - Branch creation with hash tracking
    - `commit()` - Commit simulation
    - `make_dirty()` - Dirty state simulation

5. **Add schema testing integration**
    - `expect_schema_variants(test_cases)` - Accept Vec<SchemaTestCase>

### Phase 2: Implement New Test Alongside Existing

**Goal**: Create new test function using builder pattern without removing old one

1. **Create `test_trunk_based_development_flow_builder()`**
    - Location: After existing test function
    - Purpose: Implement the same functionality using builder pattern
    - Validation: Ensure it produces identical results to existing test

2. **Map existing test steps to builder methods**
    - Setup: `FlowTestScenario::new().expect_version("1.0.0", "1.0.0")`
    - Branch creation: `.create_branch("feature-1").expect_version("1.0.0", "1.0.0")`
    - Dirty state: `.make_dirty().expect_schema_variants(schema_test_cases)`

3. **Validate parity with existing test**
    - Run both tests in parallel
    - Ensure identical outputs
    - Fix any discrepancies

### Phase 3: Extend Builder Test with Full Workflow

**Goal**: Implement the comprehensive trunk-based development workflow from design document

1. **Add advanced builder methods**
    - `checkout(branch_name)` - Branch switching
    - `merge_branch(source_branch)` - Branch merging
    - `create_tag(tag_name)` - Tag creation
    - `expect_dynamic_version(template)` - Template-based version expectations

2. **Implement comprehensive workflow steps**
    - Parallel feature branches (feature-1, feature-2)
    - Feature completion and releases
    - Branch synchronization scenarios
    - Nested feature branches (feature-3 from feature-2)
    - Final release workflow

3. **Add comprehensive schema testing**
    - Test all schema variants at each workflow step
    - Ensure schema behavior consistency across different states
    - Validate context handling for different branch scenarios

### Phase 4: Migration and Cleanup

**Goal**: Replace old test with new builder-based test

1. **Validate complete feature parity**
    - Ensure new test covers all scenarios from old test
    - Confirm identical behavior and outputs
    - Run full test suite to ensure no regressions

2. **Replace old test function**
    - Rename: `test_trunk_based_development_flow_builder()` → `test_trunk_based_development_flow()`
    - Remove old implementation
    - Update any references to old test function

3. **Add additional workflow tests**
    - Create separate tests for GitFlow workflow
    - Add tests for edge cases and error scenarios
    - Implement tests for different branching strategies

## Implementation Details

### FlowTestScenario Struct Design

```rust
struct FlowTestScenario {
    fixture: GitRepoFixture,
    fixture_path: String,
    current_branch: String,
    current_hash: u32,
}

impl FlowTestScenario {
    fn new() -> Result<Self, Box<dyn std::error::Error>>
    fn expect_version(self, semver: &str, pep440: &str) -> Self
    fn expect_schema_variants(self, test_cases: Vec<SchemaTestCase>) -> Self
    fn create_branch(self, branch_name: &str) -> Self
    fn commit(self) -> Self
    fn make_dirty(self) -> Self
    fn checkout(self, branch_name: &str) -> Self
    fn merge_branch(self, source_branch: &str) -> Self

    // Helper methods for creating common test cases
    fn create_base_schema_test_cases(base_version: &str) -> Vec<SchemaTestCase>
    fn create_alpha_schema_test_cases(base_version: &str, branch_hash: u32) -> Vec<SchemaTestCase>
    fn create_dirty_schema_test_cases(base_version: &str, branch_hash: u32, branch_name: &str) -> Vec<SchemaTestCase>
}
```

### File Organization Strategy

**New File: `src/cli/flow/test_utils.rs`**

```rust
// Test utilities for flow pipeline tests
use crate::cli::flow::args::FlowArgs;
use crate::cli::flow::pipeline::run_flow_pipeline;
use crate::test_utils::{GitRepoFixture, assert_version_expectation};
use crate::cli::utils::template::Template;

// Move SchemaTestCase here
#[derive(Debug)]
pub struct SchemaTestCase {
    pub name: &'static str,
    pub semver_expectation: String,
    pub pep440_expectation: String,
}

// New FlowTestScenario struct
#[derive(Debug)]
pub struct FlowTestScenario {
    fixture: GitRepoFixture,
    fixture_path: String,
    current_branch: String,
    current_hash: u32,
}

impl FlowTestScenario {
    // ... implementation methods
}

// Helper functions for creating test cases
pub fn create_base_schema_test_cases(base_version: &str) -> Vec<SchemaTestCase> {
    // ... implementation
}

// Move existing test functions here
pub fn test_flow_pipeline_with_fixture(...) { /* moved from pipeline.rs */ }
pub fn test_flow_pipeline_with_fixture_and_schema(...) { /* moved from pipeline.rs */ }
pub fn test_flow_pipeline_with_schema_test_cases(...) { /* moved from pipeline.rs */ }
```

**Updated: `src/cli/flow/pipeline.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use super::test_utils::*; // Import from new test_utils.rs

    // ... existing test functions
}
```

### Schema Integration Strategy

1. **Move SchemaTestCase to separate test utilities file**
2. **Reuse existing test utilities** (test_flow_pipeline_with_schema_test_cases)
3. **Add builder methods for schema testing**
4. **Maintain backward compatibility with existing schema test patterns**

### Test Organization

```rust
#[test]
fn test_trunk_based_development_flow() {
    // Old implementation (will be removed)
}

#[test]
fn test_trunk_based_development_flow_builder() {
    // New implementation using explicit expectations
    // Example usage:
    FlowTestScenario::new()?
        .expect_version("1.0.0", "1.0.0")
        .create_branch("feature-1")
        .expect_version("1.0.0", "1.0.0")
        .make_dirty()
        .expect_schema_variants(create_dirty_schema_test_cases("1.0.0", branch_hash, "feature-1"))
}

#[test]
fn test_trunk_based_development_comprehensive() {
    // Extended workflow from design document
    // Uses explicit expectations throughout
}

#[test]
fn test_gitflow_workflow() {
    // GitFlow-specific workflow testing
    // Uses explicit expectations throughout
}
```

## Success Criteria

### Phase 1 Success ✅ **COMPLETED**

- [x] FlowTestScenario struct implemented
- [x] Core builder methods working (new, expect_version, expect_schema_variants, create_branch, make_dirty, checkout)
- [x] Schema integration functional with explicit expectations
- [x] New test produces identical results to existing test
- [x] All 67 flow tests passing with zero regressions
- [x] Test utilities successfully moved to separate file
- [x] Helper functions for creating common schema test cases implemented

### Phase 2 Success ✅ **COMPLETED**

- [x] New builder test passes all existing scenarios
- [x] Zero changes to existing test functionality
- [x] Both tests can run in parallel without conflicts

**Phase 2 Implementation Details:**

- Created `test_trunk_based_development_flow_builder()` function alongside existing test
- Successfully mapped all original test steps to builder pattern methods:
    - Initial setup: `FlowTestScenario::new().expect_version("1.0.0", "1.0.0")`
    - Branch creation: `.create_branch("feature-1").expect_version("1.0.0", "1.0.0")`
    - Dirty state testing: `.make_dirty().expect_schema_variants(...)`
- Fixed schema pattern mismatches (dots vs dashes in version strings)
- Fixed context patterns (branch name conversion from `feature-1` to `feature`)
- Validated identical outputs between original and builder-based tests
- Both tests running successfully with zero regressions

### Phase 3 Success

- [ ] Comprehensive trunk-based workflow implemented
- [ ] All workflow steps from design document covered
- [ ] Schema testing works at each workflow step
- [ ] Test remains readable and maintainable

### Phase 4 Success

- [ ] Old test successfully replaced
- [ ] No regressions in test functionality
- [ ] Additional workflow tests implemented
- [ ] Code organization is clean and maintainable

## Risk Mitigation

### Technical Risks

- **Breaking existing tests**: Keep old test until new one is fully validated
- **Builder complexity**: Start simple and add complexity incrementally
- **Schema integration issues**: Test schema integration thoroughly

### Timeline Risks

- **Scope creep**: Focus on trunk-based workflow first, add others later
- **Over-engineering**: Keep builder methods focused and purposeful

## Dependencies

### Required Code Changes

- Modify `src/cli/flow/pipeline.rs` test module
- Add new test functions
- Update imports if needed

### No Breaking Changes

- Existing test utilities remain unchanged
- SchemaTestCase struct remains unchanged
- Current test functionality preserved during migration

## Next Steps

1. **Implement Phase 1**: Create FlowTestScenario and core builder methods
2. **Validate Phase 2**: Create new test and ensure parity
3. **Extend Phase 3**: Implement comprehensive workflow
4. **Complete Phase 4**: Migration and cleanup

This plan ensures zero-risk migration while providing a clear path to the desired readable, comprehensive test structure.
