# Docker Test Performance Optimization Plan

## 🎯 **Problem Statement**

Current Docker-based Git testing is slow due to:

- **80+ tests** using `should_run_docker_tests()`
- **37+ GitRepoFixture** usages creating new Docker containers
- **Multiple Docker containers per test** (each Git operation = new container)
- **Heavy Docker startup overhead** per container creation

**Current Performance**: Each test spawns 5-10+ Docker containers with full initialization overhead.

## 📊 **Performance Analysis**

### Current Docker Overhead Per Test:

- Container creation: ~200-500ms per container
- Volume mounting: ~50-100ms
- Security options setup: ~50ms
- User mapping: ~30ms
- Entrypoint setup: ~20ms
- **Total per container**: ~350-700ms
- **Per test (5-10 containers)**: ~1.75-7 seconds

### Test Scale Impact:

- 80+ Docker-dependent tests
- 37+ GitRepoFixture usages
- **Total Docker overhead**: ~140-560 seconds (2-9 minutes)

## 🚀 **Optimization Strategies**

### **Strategy 1: Container Reuse (Primary)**

**Impact**: 70-80% performance improvement
**Implementation**: Reuse single long-running Docker container per test session

```rust
pub struct OptimizedDockerGit {
    container_id: Option<String>,
    test_dir: Option<PathBuf>,
}

impl OptimizedDockerGit {
    fn ensure_container_running(&mut self, test_dir: &TestDir) -> io::Result<()> {
        if self.container_id.is_none() || self.test_dir.as_ref() != Some(test_dir.path()) {
            self.start_container(test_dir)?;
        }
        Ok(())
    }
}
```

### **Strategy 2: Batch Git Operations**

**Impact**: Additional 20-30% improvement
**Implementation**: Combine multiple Git operations into single Docker call

```rust
fn init_repo(&self, test_dir: &TestDir) -> io::Result<()> {
    let script = r#"
        git init -b main &&
        git config user.name 'Test User' &&
        git config user.email 'test@example.com' &&
        echo '# Test Repository' > README.md &&
        git add . &&
        git commit -m 'Initial commit'
    "#;
    self.run_docker_command(test_dir, script)
}
```

### **Strategy 3: Image Pre-pulling & Caching**

**Impact**: Additional 10-20% improvement
**Implementation**: Pre-pull Docker image and cache availability checks

### **Strategy 4: Test Parallelization**

**Impact**: Additional 40-50% improvement
**Implementation**: Parallel test execution with container pools

## 📋 **Implementation Plan**

### **Phase 1: Container Reuse (Immediate Impact) - ✅ COMPLETED**

🚨 **CRITICAL WARNING**: Container reuse has **very high flakiness risk**. Consider alternative approaches first.

**Alternative Approach - Container Pool (Lower Risk)**:

- Create a pool of pre-warmed containers
- Each test gets a fresh container from the pool
- No shared state between tests
- Better isolation than container reuse

**Container Reuse Implementation - COMPLETED**:

- ✅ Create `OptimizedDockerGit` struct with container reuse
- ✅ Implement `ensure_container_running()` method
- ✅ Add proper cleanup in `Drop` trait
- ✅ Maintain same `GitOperations` trait interface
- ✅ Update `get_git_impl()` to use optimized version
- ✅ **Expected Speedup**: 70-80%
- ✅ **MANDATORY**: Run flakiness detection loop before proceeding

**✅ PHASE 1 COMPLETION NOTES**:

- Container reuse implemented in `DockerGit` struct
- Race condition fixed in `ensure_container_running()` method
- Enhanced test error handling for better flakiness detection
- `test_get_vcs_data_with_commit` improved with detailed diagnostics

### **Phase 2: Parallelization (Advanced)**

- [ ] Implement container pool for parallel tests
- [ ] Add thread-safe container management
- [ ] Update test execution to use parallel containers
- [ ] **Expected Speedup**: Additional 40-50%

### **Phase 3: Batch Operations (Experimental - May Not Work)**

⚠️ **Note**: This phase may not work as expected because the Docker image already includes git command and we just pass its subcommands. The batching approach might conflict with how Docker handles git command execution.

- [ ] Modify `GitOperations` methods to batch operations
- [ ] Update `init_repo()`, `create_commit()`, `create_tag()` methods
- [ ] Test batch operation error handling
- [ ] **Expected Speedup**: Additional 20-30% (if feasible)

## 🎯 **Expected Performance Results**

| Optimization Phase | Current Time | Optimized Time | Speedup          |
| ------------------ | ------------ | -------------- | ---------------- |
| Container Reuse    | 100%         | 20-30%         | 3-5x             |
| + Parallelization  | 20-30%       | 10-15%         | 2x               |
| + Batch Operations | 10-15%       | 5-8%           | 2x (if feasible) |
| **Total**          | **100%**     | **5-8%**       | **12-20x**       |

## 🔧 **Implementation Details**

### **File Changes Required:**

1. `src/test_utils/git/docker.rs` - Add `OptimizedDockerGit`
2. `src/test_utils/git/mod.rs` - Update `get_git_impl()`
3. `src/test_utils/mod.rs` - Add optimization flags
4. `src/config.rs` - Add optimization environment variables

### **New Environment Variables:**

- `ZERV_TEST_DOCKER_OPTIMIZE=true` - Enable container reuse
- `ZERV_TEST_DOCKER_PARALLEL=true` - Enable parallel execution
- `ZERV_TEST_DOCKER_BATCH=true` - Enable batched operations

### **Backward Compatibility:**

- ✅ Keep same `GitOperations` trait interface
- ✅ Maintain same `GitRepoFixture` API
- ✅ Preserve all existing test logic
- ✅ Keep Docker isolation benefits
- ✅ No breaking changes to test code

## 🧪 **Focused Verification Strategy**

### **Phase 1: Container Reuse Flakiness Testing (CRITICAL)**

**Target Tests**: Only 3-5 Docker-dependent tests to minimize verification time:

```bash
# Focus on these specific tests that use GitRepoFixture
cargo test test_tagged_repo_clean
cargo test test_distance_repo_clean
cargo test test_dirty_repo
cargo test test_git_operations_trait
cargo test test_docker_git_validation
```

**Flakiness Detection Loop with Timing**:

```bash
# Run 20 iterations to detect flakiness (fast with only 5 tests)
echo "🚀 Starting flakiness detection with timing..."
start_time=$(date +%s)

for i in {1..20}; do
  iter_start=$(date +%s)
  echo "=== Iteration $i/20 ==="

  cargo test test_tagged_repo_clean test_distance_repo_clean test_dirty_repo test_git_operations_trait test_docker_git_validation --lib

  if [ $? -ne 0 ]; then
    echo "❌ FAILED at iteration $i"
    exit 1
  fi

  iter_end=$(date +%s)
  iter_duration=$((iter_end - iter_start))
  echo "⏱️  Iteration $i completed in ${iter_duration}s"
done

end_time=$(date +%s)
total_duration=$((end_time - start_time))
echo "✅ All 20 iterations passed - no flakiness detected"
echo "📊 Total time: ${total_duration}s (avg: $((total_duration / 20))s per iteration)"
```

**Expected Time**: ~2-3 minutes (vs 20+ minutes for full test suite)

### **Phase 2: Performance Benchmarking with Detailed Timing**

```bash
# Benchmark current implementation
echo "🔍 Benchmarking CURRENT implementation..."
echo "Start time: $(date)"
current_start=$(date +%s)

cargo test test_tagged_repo_clean test_distance_repo_clean test_dirty_repo test_git_operations_trait test_docker_git_validation --lib

current_end=$(date +%s)
current_duration=$((current_end - current_start))
echo "⏱️  CURRENT implementation: ${current_duration}s"
echo "End time: $(date)"
echo ""

# Benchmark optimized implementation
echo "🚀 Benchmarking OPTIMIZED implementation..."
echo "Start time: $(date)"
optimized_start=$(date +%s)

cargo test test_tagged_repo_clean test_distance_repo_clean test_dirty_repo test_git_operations_trait test_docker_git_validation --lib

optimized_end=$(date +%s)
optimized_duration=$((optimized_end - optimized_start))
echo "⏱️  OPTIMIZED implementation: ${optimized_duration}s"
echo "End time: $(date)"
echo ""

# Calculate improvement
if [ $current_duration -gt 0 ]; then
  improvement=$((100 - (optimized_duration * 100 / current_duration)))
  speedup=$((current_duration * 100 / optimized_duration))
  echo "📊 PERFORMANCE RESULTS:"
  echo "   Current: ${current_duration}s"
  echo "   Optimized: ${optimized_duration}s"
  echo "   Improvement: ${improvement}% faster"
  echo "   Speedup: ${speedup}x"
else
  echo "❌ Cannot calculate improvement (current duration was 0)"
fi
```

### **Phase 3: Add Debug Timing to Test Code**

**Add timing debug messages to Docker operations**:

```rust
// In src/test_utils/git/docker.rs
impl DockerGit {
    fn run_git_command(&self, command: &str) -> Result<String, ZervError> {
        let start = std::time::Instant::now();

        // ... existing Docker command execution ...

        let duration = start.elapsed();
        eprintln!("🐳 Docker Git command '{}' took {:?}", command, duration);

        // ... rest of implementation ...
    }
}

// In src/test_utils/git/fixtures.rs
impl GitRepoFixture {
    pub fn tagged(tag: &str) -> Result<Self, ZervError> {
        let start = std::time::Instant::now();

        // ... existing fixture creation ...

        let duration = start.elapsed();
        eprintln!("📦 GitRepoFixture::tagged('{}') took {:?}", tag, duration);

        Ok(fixture)
    }
}
```

**Enable debug output during testing**:

```bash
# Run tests with debug timing visible
RUST_LOG=debug cargo test test_tagged_repo_clean test_distance_repo_clean test_dirty_repo test_git_operations_trait test_docker_git_validation --lib
```

### **Quality Assurance (Minimal Set):**

- [ ] 5 focused tests pass 20 iterations without flakiness
- [ ] Performance improvement measured with detailed timing
- [ ] Debug timing shows where time is spent
- [ ] Error handling works correctly

## 🚨 **Risk Mitigation & Flakiness Analysis**

### **High Flakiness Risk - Container Reuse:**

⚠️ **CRITICAL**: Container reuse is the **highest risk** for flaky tests:

1. **Shared State Between Tests**: Reusing containers can cause test interference
2. **Container State Persistence**: Git state, files, and environment persist between tests
3. **Race Conditions**: Multiple tests accessing same container simultaneously
4. **Cleanup Failures**: Container not properly reset between tests
5. **Resource Contention**: Shared containers competing for resources

**Flakiness Symptoms:**

- Tests pass individually but fail when run together
- Intermittent "git directory not found" errors
- Tests affecting each other's git state
- Random failures in CI but not locally

### **Medium Flakiness Risk - Parallelization:**

1. **Resource Exhaustion**: Too many parallel containers
2. **Port Conflicts**: Multiple containers trying to use same ports
3. **File System Race Conditions**: Concurrent access to shared directories
4. **Memory Pressure**: System running out of memory with many containers

### **Low Flakiness Risk - Batch Operations:**

1. **Command Failure Propagation**: One failed command in batch affects others
2. **Error Masking**: Harder to identify which specific operation failed

### **Mitigation Strategies:**

- **Container Isolation**: Each test gets its own container (no sharing)
- **Atomic Operations**: Use `GitOperations` trait methods for consistency
- **Proper Cleanup**: Implement `Drop` trait with comprehensive cleanup
- **Resource Limits**: Set container memory and CPU limits
- **Fallback Mode**: Automatic fallback to current implementation on failure
- **State Verification**: Verify container state before each test
- **Error Context**: Detailed error messages for debugging

## 📈 **Success Metrics**

### **Performance Targets:**

- [ ] 70-80% reduction in test execution time
- [ ] 90%+ reduction in Docker container spawns
- [ ] Maintain 100% test pass rate
- [ ] Zero test flakiness increase

### **Quality Targets:**

- [ ] All existing tests pass unchanged
- [ ] Docker isolation maintained
- [ ] Error handling improved
- [ ] Resource cleanup verified

## 🎯 **Next Steps**

### **Immediate Actions:**

1. **Start with Phase 1**: Implement container reuse optimization
2. **Benchmark current performance**: Measure baseline test execution time
3. **Create optimized DockerGit**: Implement container reuse logic
4. **Test and validate**: Ensure no regressions

### **Short-term Goals:**

- Complete Phase 1 implementation
- Achieve 70-80% performance improvement
- Validate test stability and quality
- Document performance improvements

### **Long-term Goals:**

- Implement all optimization phases
- Achieve 15-25x overall speedup
- Add parallel test execution
- Optimize for CI/CD environments

## 📝 **Notes**

- **Priority**: High - Significant performance impact
- **Complexity**: Medium - Requires careful container management
- **Risk**: Low - Backward compatible, fallback available
- **Timeline**: 1-2 weeks for Phase 1, 2-3 weeks for full implementation

---

**Created**: $(date)
**Status**: Planning Phase
**Next Review**: After Phase 1 implementation
