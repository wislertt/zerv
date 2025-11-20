# Optimize FlowTestScenario to Use Stdin Fixtures Instead of Git Repositories

## Status

**Planned**

## Priority

**High** - Performance improvement for unit tests to reduce Docker usage and speed up CI/CD pipeline

## Context

The current `FlowTestScenario` implementation relies heavily on `GitRepoFixture` which requires Docker containers for Git operations. This makes unit tests slow and resource-intensive. The codebase already has comprehensive stdin-based testing patterns using `ZervFixture` that can provide the same test coverage without the overhead of Git operations.

### Current State Analysis

**FlowTestScenario (`src/cli/flow/test_utils.rs:46-276`)**:

- Uses `GitRepoFixture` for creating test repositories
- Provides methods like `create_tag()`, `create_branch()`, `checkout()`, `commit()`, `merge_branch()`
- Tests run through `run_flow_pipeline()` which reads from Git repositories
- Requires Docker for Git operations (`ZERV_TEST_DOCKER=true`)

**Existing Stdin Patterns (`tests/integration_tests/version/main/sources/stdin.rs`)**:

- Uses `ZervFixture` to create predefined Zerv objects
- Tests use `TestCommand::run_with_stdin()` to pass RON data via stdin
- No Git operations required, much faster execution
- Full control over test data and scenarios

**ZervFixture Capabilities (`src/test_utils/zerv/zerv.rs`)**:

- Complete control over version components (`with_version()`, `with_pre_release()`)
- Branch and commit hash simulation (`with_branch()`, `with_commit_hash()`)
- Distance and dirty state simulation (`with_distance()`, `with_dirty()`)
- Schema presets and custom schema support
- VCS data simulation for complete Git state representation

## Problems with Current Approach

1. **Performance**: Docker-based Git operations are slow and resource-intensive
2. **Reliability**: Docker containers can fail, timeout, or have networking issues
3. **CI/CD Impact**: Slow tests increase pipeline duration and cost
4. **Complexity**: Git state management adds unnecessary complexity to test scenarios
5. **Maintainability**: Git fixture logic can be brittle and hard to debug

## Goals

1. **Replace Git dependencies** with stdin-based `ZervFixture` approach
2. **Maintain all existing test functionality** and coverage
3. **Improve test execution speed** by eliminating Docker usage
4. **Simplify test scenario creation** with direct data manipulation
5. **Preserve the same builder pattern API** for `FlowTestScenario`
6. **Enable deterministic test outcomes** without Git variability

## Proposed Solution

Replace `GitRepoFixture` with `ZervFixture` directly, using a simple and clean approach. The `FlowTestScenario` should only contain a `ZervFixture` and all operations modify the fixture directly.

```rust
pub struct FlowTestScenario {
    fixture: ZervFixture,
}

impl FlowTestScenario {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            fixture: ZervFixture::new(),
        })
    }

    pub fn create_tag(mut self, tag: &str) -> Self {
        // Parse tag and set version in fixture directly
        let version = parse_tag_to_version(tag);
        self.fixture = self.fixture
            .with_version(version.major, version.minor, version.patch);
        self
    }

    pub fn create_branch(mut self, branch_name: &str) -> Self {
        self.fixture = self.fixture.with_branch(branch_name.to_string());
        self
    }

    pub fn checkout(mut self, branch_name: &str) -> Self {
        self.fixture = self.fixture.with_branch(branch_name.to_string());
        self
    }

    pub fn commit(mut self) -> Self {
        // Simple commit: just increment distance and generate hash
        let current_distance = self.fixture.zerv().vars.distance.unwrap_or(0) + 1;
        let commit_hash = generate_commit_hash(&self.fixture.zerv().vars.bumped_branch.as_deref().unwrap_or("main"), current_distance);
        self.fixture = self.fixture
            .with_distance(current_distance)
            .with_commit_hash(commit_hash)
            .with_dirty(false); // commits clean working directory
        self
    }

    pub fn make_dirty(mut self) -> Self {
        self.fixture = self.fixture.with_dirty(true);
        self
    }

    pub fn merge_branch(mut self, branch_name: &str) -> Self {
        // Simple merge: just generate new hash and update distance
        let current_distance = self.fixture.zerv().vars.distance.unwrap_or(0) + 1;
        let merge_hash = generate_commit_hash(&format!("merge-{}", branch_name), current_distance);
        self.fixture = self.fixture
            .with_distance(current_distance)
            .with_commit_hash(merge_hash);
        self
    }

    fn to_stdin_content(&self) -> String {
        self.fixture.build().to_string()
    }

    pub fn test_dir_path(&self) -> String {
        // Return dummy path since we're using stdin
        "dummy-path-for-stdin".to_string()
    }
}
```

## Implementation Plan

### Step 1: Clean Implementation

1. **Reset test_utils.rs to clean state** - revert all complex changes
2. **Implement simple `FlowTestScenario`** with only `fixture: ZervFixture`
3. **Add simple helper functions** for tag parsing and hash generation
4. **Update imports** to use `ZervFixture` instead of `GitRepoFixture`

### Step 2: Simple FlowTestScenario

1. **Replace complex struct with simple one**: Only `fixture: ZervFixture`
2. **Implement methods that modify fixture directly**: No tracking variables, no complex state
3. **Simple commit**: Just increment distance, generate hash, clean dirty state
4. **Simple merge**: Just generate hash, increment distance
5. **Preserve same builder pattern API**

### Step 3: Update Test Execution

1. **Modify test functions** to use `test_flow_pipeline_with_stdin()`
2. **Update `run_flow_pipeline()` calls** to pass stdin content from `ZervFixture`
3. **Remove Docker test gating** from flow tests
4. **Verify all format outputs** work identically

### Step 4: Simple Validation

1. **Update test scenarios** in `pipeline.rs` to use new approach
2. **Ensure simple hash generation** produces expected results
3. **Run comprehensive test suite** to ensure no regressions
4. **Fix any issues with simple approach first**

## Detailed Implementation Specifications

### Hash Generation Strategy

```rust
fn generate_commit_hash() -> String {
    // Generate deterministic commit hash using existing Template system
    Template::<u32>::new("{{ hash_int(value='commit', length=7) }}")
        .render_unwrap(None)
        .to_string()
}

fn generate_merge_hash(branch_name: &str) -> String {
    // Generate deterministic merge hash based on branch name
    Template::<u32>::new(format!("{{{{ hash_int(value='merge-{}', length=7) }}}}", branch_name))
        .render_unwrap(None)
        .to_string()
}
```

### Tag Parsing Logic

```rust
fn parse_tag_to_version(tag: &str) -> VersionComponents {
    let tag = tag.strip_prefix('v').unwrap_or(tag);

    if let Ok(semver) = SemVer::from_str(tag) {
        VersionComponents {
            major: semver.major,
            minor: semver.minor,
            patch: semver.patch,
            pre_release: semver.pre_release.map(|pr| pr.into()),
        }
    } else if let Ok(pep440) = PEP440::from_str(tag) {
        pep440.into()
    } else {
        panic!("Unable to parse tag '{}' as version", tag);
    }
}
```

### Test Execution Update

```rust
pub fn test_flow_pipeline_with_stdin(
    stdin_content: &str,
    schema: Option<&str>,
    semver_expectation: &str,
    pep440_expectation: &str,
) {
    let test_cases = vec![
        ("semver", semver_expectation),
        ("pep440", pep440_expectation),
    ];

    for (format_name, expectation) in test_cases {
        let mut args = FlowArgs::default();
        args.input.source = "stdin".to_string();
        args.output.output_format = format_name.to_string();

        if let Some(schema_value) = schema {
            args.schema = Some(schema_value.to_string());
        }

        let result = run_flow_pipeline(args, Some(stdin_content));
        // ... rest of validation logic remains the same
    }
}
```

## Migration Strategy

### Phase 1: Direct Replacement

- Replace `GitRepoFixture` with `ZervFixture` in `FlowTestScenario`
- Update all method implementations to use `ZervFixture` methods
- Test the new implementation with existing scenarios
- Verify output parity

### Phase 2: Test Migration

- Update test functions to use stdin instead of directory paths
- Run comprehensive test suite to ensure no regressions
- Fix any discrepancies found

### Phase 3: Cleanup

- Remove `GitRepoFixture` import from `FlowTestScenario` and Docker test gating
- Clean up unused Git-related code in FlowTestScenario test utilities
- Update documentation
- **Note**: `GitRepoFixture` remains in the codebase for other tests that still need it

### Phase 4: Validation

- Benchmark test execution time improvements
- Verify Docker usage elimination
- Document performance improvements

## Testing Strategy

### Unit Tests

- Test hash generation functions for determinism
- Validate tag parsing logic with various formats
- Ensure branch state management works correctly
- Test builder pattern methods individually

### Integration Tests

- Run existing test scenarios with both old and new implementations
- Verify identical outputs for complex git histories
- Test edge cases (merges, tags, dirty states)
- Validate schema variant testing

### Performance Tests

- Benchmark execution time improvements
- Measure memory usage differences
- Verify Docker dependency elimination
- Test scalability with large test suites

### Regression Tests

- Ensure all existing test expectations remain valid
- Verify error handling behavior is preserved
- Test output format consistency
- Validate debug functionality still works

## Success Criteria

1. ✅ **All existing tests pass** with new stdin-based approach
2. ✅ **Significant performance improvement** (target: 5-10x faster execution)
3. ✅ **Zero Docker dependency** for flow pipeline tests
4. ✅ **Identical test outputs** compared to Git-based implementation
5. ✅ **Maintained builder pattern API** for ease of migration
6. ✅ **Deterministic test behavior** without Git variability
7. ✅ **Reduced test flakiness** and improved reliability
8. ✅ **Simplified debugging** with direct data control

## Risk Mitigation

### Breaking Changes

- **Risk**: Changes to `FlowTestScenario` API could break existing tests
- **Mitigation**: Maintain exact same API surface, only change internal implementation

### Output Differences

- **Risk**: New implementation might produce different version outputs
- **Mitigation**: Extensive parallel testing to ensure output parity before migration

### Hash Consistency

- **Risk**: Different hash generation could break test expectations
- **Mitigation**: Use existing deterministic hash functions and validate outputs

### Complex Scenarios

- **Risk**: Complex Git workflows might be hard to replicate with stdin
- **Mitigation**: Incremental migration with thorough validation at each step

### Performance Regression

- **Risk**: New implementation could be slower in some cases
- **Mitigation**: Performance benchmarking and optimization throughout development

## Expected Benefits

1. **Performance**: 5-10x faster test execution
2. **Reliability**: Eliminate Docker-related test failures
3. **CI/CD**: Faster pipeline execution and reduced resource usage
4. **Maintainability**: Simpler test code with direct data control
5. **Debugging**: Easier to debug failing tests with deterministic data
6. **Scalability**: Can run more tests in parallel without Docker conflicts

## Decision Criteria

Choose this approach if:

- You want significantly faster test execution
- Docker usage is causing performance or reliability issues
- You need more control over test data and scenarios
- You want to reduce CI/CD pipeline costs and duration
- You prefer deterministic test behavior over Git realism

This approach is **recommended** because it maintains all existing functionality while dramatically improving performance and reliability. The stdin-based testing patterns are already well-established in the codebase, making this a low-risk, high-benefit optimization.
