# Zerv Flow GitFlow Implementation Plan

**Status**: In Progress (Phase 2: Branch Rules Completed)
**Priority**: High
**Context**: Complete GitFlow support for zerv flow with branch pattern rules and post mode configuration to enable GitFlow test matching the Mermaid diagram.

## Current Implementation Status Analysis

### ✅ **What's Already Working (40% Complete)**

#### **Core Infrastructure**

- Complete CLI argument parsing and validation (118 tests passing)
- Flow pipeline integration with version command (translation layer approach)
- Basic version generation with alpha pre-releases
- Hash-based branch identification (e.g., `feature/auth` → `alpha.12345`)
- Dirty state detection with dev timestamps
- Schema variants working (11 different output schemas)
- Comprehensive test infrastructure (trunk-based development test)
- GitRepoFixture with branch operations
- FlowTestScenario builder pattern

#### **VCS Integration**

- Git branch detection working (`current_branch` field)
- Commit hash and distance calculation (commit mode)
- Tag-based version parsing
- All VCS data properly mapped to ZervVars
- Git prefix implementation (`g{hex:7}` format)

#### **✅ NEW: Branch Rules System (Phase 1 Complete)**

- ✅ **Implemented**: `src/cli/flow/branch_rules.rs` with complete functionality
- ✅ **Pattern matching**: Exact (`develop`) and wildcard (`release/*`) patterns
- ✅ **Number extraction**: From branch names (`release/1` → `Some(1)`)
- ✅ **Type-safe enums**: `PreReleaseLabel` (Alpha, Beta, Rc) and `PostMode` (Tag, Commit)
- ✅ **RON configuration**: Simplified syntax (`pre_release_num: 1` vs `Some(1)`)
- ✅ **Strong validation**: Wildcard patterns vs exact pattern rules
- ✅ **Comprehensive testing**: 56 branch rules tests passing
- ✅ **Default GitFlow rules**: `develop` → `beta.1`, `release/*` → `rc`, post modes

### ❌ **Remaining GitFlow Components (60% Gap)**

#### **1. FlowArgs Integration (Phase 2 - In Progress)**

- **Missing**: FromStr trait for BranchRules with RON parsing
- **Missing**: Branch rules field in FlowArgs with default GitFlow rules
- **Missing**: Integration in FlowArgs validation method
- **Missing**: Current branch detection and rule matching
- **Missing**: Combining branch rule defaults with user CLI args
- **Missing**: Override hierarchy: user args → branch rules → defaults

#### **2. Post Mode Implementation**

- **Current**: All branches use commit distance mode
- **Missing**: Tag mode implementation for `release/*` branches
- **Missing**: Post mode differentiation in pipeline logic

#### **3. GitFlow Test Implementation (Phase 4)**

- **Missing**: Comprehensive GitFlow test matching Mermaid diagram
- **Missing**: Branch-specific version expectations:
    - `develop` → `1.0.1-beta.1.post.1`
    - `release/1` → `1.0.2-rc.1.post.1` (tag mode)
    - `feature/*` → `1.0.1-alpha.{hash}.post.1` (already working)
    - `hotfix/*` → `1.0.1-alpha.{hash}.post.1` (already working)

#### **4. Documentation (Phase 5)**

- **Missing**: Updated help text for branch rules
- **Missing**: GitFlow usage examples
- **Missing**: Architecture documentation

## GitFlow Test Requirements

Based on the Mermaid diagram in plan #32, the GitFlow test should produce:

| Branch            | Expected Version           | Current Status          |
| ----------------- | -------------------------- | ----------------------- |
| `main`            | `1.0.0`                    | ✅ Working              |
| `develop`         | `1.0.1-beta.1.post.1`      | ❌ Currently: `alpha.*` |
| `feature/auth`    | `1.0.1-alpha.12345.post.1` | ✅ Working              |
| `hotfix/critical` | `1.0.1-alpha.54321.post.1` | ✅ Working              |
| `release/1`       | `1.0.2-rc.1.post.1`        | ❌ Currently: `alpha.*` |

## Implementation Plan

### **Phase 1: Branch Rules System**

**Estimated Effort: 2-3 days**

#### **Step 1.1: Create Branch Rules Module**

**File**: `src/cli/flow/branch_rules.rs` (new)

```rust
// Enums for type safety
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PreReleaseLabel {
    Alpha,
    Beta,
    Rc,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PostMode {
    Tag,
    Commit,
}

// Core structures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchRule {
    pub pattern: String,           // "develop", "release/*", "feature/*"
    pub pre_release_label: PreReleaseLabel, // "beta", "rc", "alpha"
    pub pre_release_num: Option<u32>, // "1" for release branches
    pub post_mode: PostMode,         // "tag" for release, "commit" for others
}

// Resolved branch arguments from branch rules
#[derive(Debug, Clone)]
pub struct ResolvedBranchArgs {
    pub pre_release_label: PreReleaseLabel,
    pub pre_release_num: Option<u32>,
    pub post_mode: PostMode,
}

#[derive(Debug, Clone, Default)]
pub struct BranchRules {
    rules: Vec<BranchRule>,
}

// BranchRule resolution methods
impl BranchRule {
    /// Resolve values for a specific branch that matches this rule's pattern
    pub fn resolve_for_branch(&self, branch_name: &str) -> ResolvedBranchArgs {
        ResolvedBranchArgs {
            pre_release_label: self.pre_release_label.clone(),
            pre_release_num: self.resolve_pre_release_num(branch_name),
            post_mode: self.post_mode.clone(),
        }
    }

    /// Resolve pre-release number for a branch that matches this rule
    fn resolve_pre_release_num(&self, branch_name: &str) -> Option<u32> {
        // 1. Use explicit number from rule
        if let Some(num) = self.pre_release_num {
            return Some(num);
        }

        // 2. Extract from branch pattern (e.g., "release/1" -> "1")
        self.extract_branch_number(branch_name)
    }

    /// Extract number from branch pattern (e.g., "release/1" -> "1" when pattern is "release/*")
    fn extract_branch_number(&self, branch_name: &str) -> Option<u32> {
        if !self.pattern.ends_with("/*") {
            return None;
        }

        let prefix = &self.pattern[..self.pattern.len() - 2]; // Remove "/*"
        if !branch_name.starts_with(prefix) {
            return None;
        }

        let remainder = &branch_name[prefix.len()..];
        let number_part: String = remainder.chars()
            .take_while(|c| c.is_numeric())
            .collect();

        if number_part.is_empty() {
            None
        } else {
            number_part.parse().ok()
        }
    }
}

// Pattern matching logic
impl BranchRules {
    pub fn find_rule(&self, branch: &str) -> Option<&BranchRule>;
    pub fn default_rules() -> Self;

    /// Find and resolve rule for a branch, or return default args
    pub fn resolve_for_branch(&self, branch_name: &str) -> ResolvedBranchArgs {
        if let Some(rule) = self.find_rule(branch_name) {
            rule.resolve_for_branch(branch_name)
        } else {
            // Default fallback for unmapped branches
            ResolvedBranchArgs {
                pre_release_label: PreReleaseLabel::Alpha,
                pre_release_num: Some(branch_hash_number(branch_name, 5).unwrap_or_default()),
                post_mode: PostMode::Commit,
            }
        }
    }
}

// Template functions for branch-based numbering
pub fn **branch_hash_number**(branch: &str, length: usize) -> Result<String, ZervError>;
```

#### **Step 1.2: Pattern Matching Logic**

- **Exact match**: `develop` matches only `develop` branch
- **Wildcard match**: `release/*` matches `release/1`, `release/feature-name`
- **Number extraction**:
    - `release/1` → extracts `1`
    - `release/1/auth` → extracts `1`
    - `release/feature` → no number (uses hash-based)

#### **Step 1.3: Default Branch Rules**

```ron
[
    (pattern: "develop", pre_release_label: Beta, pre_release_num: 1, post_mode: Commit),
    (pattern: "release/*", pre_release_label: Rc, post_mode: Tag),
]
```

**Note**: With enums and proper types, the RON format uses:

- `pre_release_label: Beta` (enum variant, not string)
- `pre_release_num: 1` (u32 integer, not string)
- `post_mode: Tag` (enum variant, not string)

All other branches fall back to the default behavior:

- **Alpha pre-release** with hash-based numbering
- **Commit post mode** (distance from branch point)
- **No special handling** for `main`, `master`, `feature/*`, `hotfix/*` branches

### **Phase 2: FlowArgs Integration with Branch Rules**

**Estimated Effort: 2 days**

#### **Step 2.1: ✅ COMPLETED - Branch Rules System**

**Status**: ✅ **COMPLETED** - Full branch rules system implemented with:

- **Type-safe enums** for `PreReleaseLabel` (Alpha, Beta, Rc) and `PostMode` (Tag, Commit)
- **Pattern matching** with exact ("develop") and wildcard ("release/\*") patterns
- **Number extraction** from branch names (e.g., "release/1" → `Some(1)`)
- **RON configuration** with simplified syntax support (`pre_release_num: 1` vs `Some(1)`)
- **Strong validation**:
    - Wildcard patterns must have `pre_release_num = None`
    - Exact patterns must have explicit `pre_release_num`
- **Comprehensive testing**: 56 tests passing
- **Clean rstest parameterization** for maintainable test suite

**Files**:

- ✅ `src/cli/flow/branch_rules.rs` (new) - Complete implementation
- ✅ `src/cli/flow/mod.rs` - Module exports updated

#### **Step 2.2: FromStr Implementation for BranchRules**

**Status**: 📋 **Planned** - FromStr trait implementation planned

**Implementation approach**:

1. **Add FromStr trait to BranchRules**:
    - Implement standard Rust `FromStr` trait for `BranchRules`
    - Direct RON string parsing using existing `from_ron()` method
    - Comprehensive test coverage for parsing success and error cases
    - Clean integration with Clap's `value_parser!(BranchRules)`

**Files to update**:

- `src/cli/flow/branch_rules.rs` - Add `FromStr` implementation and tests

#### **Step 2.3: FlowArgs Integration with Option<BranchRules>**

**File**: `src/cli/flow/args/main.rs` (update existing)

**Implementation approach**:

2. **Add typed --branch-rules field to FlowArgs**:
    - Use `Option<BranchRules>` for maximum type safety
    - Leverage Clap's built-in `value_parser!(BranchRules)`
    - No default value needed - `None` triggers GitFlow defaults
    - No custom parser structs needed

3. **Direct FlowArgs validation integration**:
    - Get typed `BranchRules` object (no parsing needed)
    - Use `unwrap_or_else(|| BranchRules::default_rules())` for defaults
    - Get current branch name from VCS data
    - Match current branch against branch rules
    - Get `ResolvedBranchArgs` from matching rule
    - Combine with parsed FlowArgs values for final args

```rust
// Add to src/cli/flow/branch_rules.rs
impl std::str::FromStr for BranchRules {
    type Err = ZervError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Direct RON string parsing
        BranchRules::from_ron(s)
    }
}

// Update FlowArgs struct in src/cli/flow/args/main.rs
#[derive(Debug)]
pub struct FlowArgs {
    // ... existing fields ...

    /// Branch rules in RON format (default: GitFlow rules, supports @file.ron syntax)
    #[arg(
        long = "branch-rules",
        help = "Branch rules in RON format (default: GitFlow rules, supports @file.ron syntax)",
        value_parser = clap::value_parser!(BranchRules)
    )]
    pub branch_rules: Option<BranchRules>,

    // ... existing fields ...
}

impl FlowArgs {
    /// Parse and apply branch rules during validation
    fn apply_branch_rules(&mut self) -> Result<(), ZervError> {
        // Get typed BranchRules object (defaults handled automatically)
        let branch_rules = self.branch_rules.clone().unwrap_or_else(|| {
            BranchRules::default_rules()
        });

        // Get Zerv object by running zerv version internally
        let zerv_object = self.get_zerv_object_from_zerv_version()?;

        // Extract current branch from Zerv context
        let branch_name = zerv_object.context.current_branch.as_deref().ok_or_else(|| {
            ZervError::CommandFailed(
                "No current branch detected. Make sure you're in a Git repository with a valid branch.".to_string()
            )
        })?;

        // Resolve branch defaults from matching rule
        let resolved_args = branch_rules.resolve_for_branch(branch_name);

        // Apply branch rule defaults only if not explicitly set by user
        if self.bump_pre_release_label.is_none() {
            self.bump_pre_release_label = Some(resolved_args.pre_release_label.to_string());
        }

        if self.bump_pre_release_num.is_none() {
            self.bump_pre_release_num = resolved_args.pre_release_num
                .map(|n| n.to_string());
        }

        if self.bump_post_mode.is_none() {
            self.bump_post_mode = Some(resolved_args.post_mode.to_string());
        }

        Ok(())
    }

    /// Get Zerv object by running zerv version internally
    fn get_zerv_object_from_zerv_version(&self) -> Result<Zerv, ZervError> {
        // Create minimal version args to get Zerv object with VCS data
        let version_args = VersionArgs {
            input: self.input.clone(),
            output: OutputConfig {
                output_format: "zerv".to_string(),
                output_template: None,
                output_prefix: None,
            },
            main: MainConfig {
                schema: Some("standard".to_string()),
                schema_ron: None,
            },
            overrides: Default::default(),
            bumps: BumpsConfig::default(),
        };

        // Run zerv version pipeline and parse output
        let ron_output = run_version_pipeline(version_args)?;
        let zerv_object: Zerv = from_str(&ron_output).map_err(|e| {
            ZervError::InvalidFormat(format!("Failed to parse zerv output: {}", e))
        })?;

        Ok(zerv_object)
    }

    /// Updated validation method with branch rules integration
    pub fn validate(&mut self) -> Result<(), ZervError> {
        // Existing validation logic...

        // Apply branch rules as final step
        self.apply_branch_rules()?;

        Ok(())
    }
}
```

**Usage Examples**:

```bash
# Using default GitFlow rules (built-in, no args needed)
zerv flow

# Simple custom rule
zerv flow --branch-rules "[(pattern: develop, pre_release_label: beta, pre_release_num: 1, post_mode: commit)]"

# Multiple custom rules (concise RON format)
zerv flow --branch-rules "
  [
    (pattern: develop, pre_release_label: beta, pre_release_num: 1, post_mode: commit),
    (pattern: release/*, pre_release_label: rc, post_mode: tag),
    (pattern: hotfix/*, pre_release_label: alpha, post_mode: commit)
  ]
"

# Complex team configuration
zerv flow --branch-rules "
  [
    (pattern: main, pre_release_label: alpha, post_mode: commit),
    (pattern: develop, pre_release_label: beta, pre_release_num: 1, post_mode: commit),
    (pattern: release/*, pre_release_label: rc, post_mode: tag),
    (pattern: hotfix/*, pre_release_label: alpha, post_mode: commit),
    (pattern: feature/*, pre_release_label: alpha, post_mode: commit),
    (pattern: experimental/*, pre_release_label: dev, post_mode: commit)
  ]
"
```

**File-based configuration can be added later**:

```bash
# Future: Environment variable with file content
BRANCH_RULES="$(cat team-config.ron)" zerv flow --branch-rules "$BRANCH_RULES"
```

**Key integration points**:

1. **Maximum type safety**: `Option<BranchRules>` guarantees valid typed objects
2. **Standard Rust patterns**: Uses `FromStr` trait and built-in Clap value parser
3. **Complete VCS reuse**: Leverages existing `run_version_pipeline` and `Zerv` object
4. **Rich context access**: Full `ZervContext` available for future features
5. **No VCS duplication**: All VCS logic remains in `zerv version` module
6. **Future-proof**: When new fields added to Zerv, flow gets them automatically
7. **Override hierarchy**: User CLI args → branch rules → defaults
8. **Consistent behavior**: Flow uses exactly same data as `zerv version`

**Architecture benefits**:

- ✅ Zero VCS code duplication
- ✅ Reuses battle-tested `run_version_pipeline`
- ✅ Complete Zerv object with all context fields
- ✅ Single source of truth for VCS logic
- ✅ Clean separation of concerns
- ✅ Type safety from CLI parsing to final result
- ✅ Future features ready with rich context data

**Implementation simplicity**:

- ✅ One method call: `run_version_pipeline(version_args)`
- ✅ One parse: `Zerv::from_str(&ron_output)`
- ✅ One field access: `zerv_object.context.current_branch`
- ✅ No custom VCS code needed
- ✅ Reuses existing error handling patterns

#### **Step 2.3: Pipeline Integration Update**

**File**: `src/cli/flow/pipeline.rs` (update existing)

The pipeline will automatically use the updated FlowArgs after validation, so minimal changes needed:

```rust
pub fn run_flow_pipeline(mut args: FlowArgs) -> Result<String, ZervError> {
    // Validation now includes branch rules application
    args.validate()?;

    // Rest of pipeline unchanged - args now has resolved values
    let version_args = VersionArgs {
        // ... use validated args with branch defaults applied
    };

    // ... existing pipeline logic
}
```

#### **Step 2.3: Updated Pipeline Integration**

**File**: `src/cli/flow/pipeline.rs` (update existing)

```rust
pub fn run_flow_pipeline(args: FlowArgs) -> Result<String, ZervError> {
    tracing::debug!("Starting flow pipeline with args: {:?}", args);

    let mut args = args;
    args.validate()?; // Existing validation

    // Get VCS data for branch detection
    let vcs_data = get_vcs_data(&args.input)?;

    // Resolve branch defaults (applies branch rules during validation)
    let resolved_args = args.resolve_branch_defaults(&vcs_data)?;

    // Convert to version args using resolved values
    let version_args = VersionArgs {
        input: resolved_args.input.clone(),
        output: OutputConfig {
            output_format: "zerv".to_string(),
            output_template: None,
            output_prefix: None,
        },
        main: MainConfig {
            schema: resolved_args.schema.clone(),
            schema_ron: None,
        },
        overrides: Default::default(),
        bumps: BumpsConfig {
            bump_pre_release_label: resolved_args.bump_pre_release_label(),
            bump_pre_release_num: resolved_args.bump_pre_release_num(),
            bump_patch: resolved_args.bump_patch(),
            bump_post: resolved_args.bump_post_mode(),
            bump_dev: resolved_args.bump_dev(),
            ..Default::default()
        },
    };

    let ron_output = run_version_pipeline(version_args)?;
    let zerv_object: Zerv = from_str(&ron_output)
        .map_err(|e| ZervError::InvalidFormat(format!("Failed to parse version output: {}", e)))?;

    let output = OutputFormatter::format_output(
        &zerv_object,
        &resolved_args.output.output_format,
        resolved_args.output.output_prefix.as_deref(),
        &resolved_args.output.output_template,
    )?;

    Ok(output)
}
```

### **Phase 3: Pipeline Integration**

**Estimated Effort: 1 day**

**Note**: Phase 3 is already implemented in Step 2.3 above. The pipeline integration has been updated to use the new FlowArgs.resolve_branch_defaults() method.

### **Phase 4: GitFlow Test Implementation**

**Estimated Effort: 2-3 days**

#### **Step 4.1: Create GitFlow Test**

**File**: `src/cli/flow/pipeline.rs` (add to tests section)

```rust
#[test]
fn test_gitflow_branching_strategy() {
    test_info!("Starting GitFlow branching strategy test (exactly matching Mermaid diagram)");
    if !should_run_docker_tests() {
        return;
    }

    // Test all GitFlow scenarios with branch-specific expectations
    // Uses expect_branch_hash for consistent hash values
    // Tests develop → beta, release/* → rc, feature/* → alpha, hotfix/* → alpha
    // Tests both tag and commit post modes
}
```

#### **Step 4.2: Expected GitFlow Test Results**

```rust
// Initial state: main and develop branches
scenario
    .expect_version("1.0.0", "1.0.0")           // main: clean tag
    .expect_version("1.0.1-beta.1.post.1", "1.0.1b1.post1")  // develop: beta.1

// Feature branch from develop (alpha + hash)
scenario
    .create_branch("feature/auth")
    .expect_version("1.0.1-alpha.12345.post.1", "1.0.1a12345.post1")

// Release branch (rc + number extraction)
scenario
    .create_branch("release/1")
    .expect_version("1.0.2-rc.1.post.1", "1.0.1rc1.post1")  // tag mode

// Hotfix from main (alpha + hash)
scenario
    .create_branch("hotfix/critical")
    .expect_version("1.0.1-alpha.54321.post.1", "1.0.1a54321.post1")
```

#### **Step 4.3: Test Infrastructure Updates**

- Update `expect_branch_hash` function for consistent hash values
- Add GitFlow-specific test case helpers
- Extend FlowTestScenario with branch pattern testing
- Add post mode validation helpers

### **Phase 5: Documentation and Cleanup**

**Estimated Effort: 1 day**

#### **Step 5.1: Update Help Text**

- Add branch rules documentation to `--help`
- Document post mode behavior differences
- Add GitFlow usage examples

#### **Step 5.2: Update Existing Documentation**

- Update plan #32 with new implementation status
- Add GitFlow section to user documentation
- Create architecture documentation for branch rules

## Implementation Phases Summary

| Phase     | Components                   | Effort        | Status             |
| --------- | ---------------------------- | ------------- | ------------------ |
| Phase 1   | Branch rules system          | 2-3 days      | ✅ **Completed**   |
| Phase 2   | FlowArgs integration         | 2 days        | 🎯 **In Progress** |
| Phase 3   | Pipeline integration         | 1 day         | Pending            |
| Phase 4   | GitFlow test                 | 2-3 days      | Pending            |
| Phase 5   | Documentation                | 1 day         | Pending            |
| **Total** | **Complete GitFlow support** | **8-10 days** | **~40% Complete**  |

## Key Design Principles

### **1. Translation Layer Pattern**

- Flow acts as intelligent translation layer to existing `zerv version`
- Branch pattern detection converts flow args to version args
- Leverages all existing version calculation logic

### **2. Pattern Matching Hierarchy**

1. **Manual overrides** (`--pre-release-label`, `--pre-release-num`)
2. **Branch pattern rules** (`develop` → `beta`, `release/*` → `rc`)
3. **Default fallback** (`alpha` with hash-based number)

### **3. Post Mode Logic**

- **Tag mode**: For release branches (count from release tags)
- **Commit mode**: For development branches (count from branch point)
- **Consistent**: Same algorithm, different reference points

### **4. Backward Compatibility**

- All existing flow functionality preserved
- Manual overrides always take precedence
- Default behavior unchanged for unmapped branches

## Success Criteria

### **Functional Requirements**

- ✅ CLI arguments working (already complete)
- 🎯 Branch pattern detection for default rules
- 🎯 Pre-release resolution: develop→beta, release/*→rc, feature/*→alpha
- 🎯 Post mode configuration: tag vs commit distance
- 🎯 GitFlow test matching Mermaid diagram exactly
- 🎯 Manual override system working

### **Technical Requirements**

- 🎯 No performance regression (translation layer overhead minimal)
- 🎯 Full backward compatibility maintained
- 🎯 Comprehensive test coverage (branch rules, translation, GitFlow)
- 🎯 Proper error handling with detailed context
- 🎯 Clean integration with existing pipeline

### **Integration Requirements**

- 🎯 Works with existing VCS detection
- 🎯 Compatible with all output formats and schemas
- 🎯 Maintains consistency with `zerv version` output
- 🎯 Leverages existing Git operations and version parsing

## Risk Assessment

### **Technical Risks**

- **Pattern matching complexity**: Low - well-defined exact/wildcard patterns
- **Translation layer performance**: Low - minimal CPU overhead
- **Post mode logic**: Low - reuses existing distance calculation

### **Integration Risks**

- **Backward compatibility**: Low - manual overrides preserve existing behavior
- **Test stability**: Medium - need consistent hash generation
- **Pipeline complexity**: Low - clean separation of concerns

## Dependencies

### **Internal Dependencies**

- ✅ CLI argument parsing (complete)
- ✅ Version pipeline integration (complete)
- ✅ VCS data detection (complete)
- ✅ Output formatting (complete)

### **External Dependencies**

- RON (already used for configuration)
- No new Rust dependencies expected

## Next Steps

1. **Begin Phase 1**: Create branch rules module with pattern matching
2. **Implement Phase 2**: Build translation layer for branch detection
3. **Integrate Phase 3**: Update flow pipeline to use translation
4. **Test Phase 4**: Create comprehensive GitFlow test scenario
5. **Document Phase 5**: Update help text and documentation

---

This implementation plan builds on the solid foundation of the existing zerv flow system to add the missing 30% needed for complete GitFlow automation. The modular approach ensures minimal risk while providing powerful branch-based version intelligence.
