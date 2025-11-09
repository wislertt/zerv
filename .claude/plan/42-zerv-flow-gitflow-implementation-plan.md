# Zerv Flow GitFlow Implementation Plan

**Status**: Planned
**Priority**: High
**Context**: Complete GitFlow support for zerv flow with branch pattern rules and post mode configuration to enable GitFlow test matching the Mermaid diagram.

## Current Implementation Status Analysis

### ✅ **What's Already Working (70% Complete)**

#### **Core Infrastructure**

- Complete CLI argument parsing and validation (68 tests passing)
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

### ❌ **Missing GitFlow Components (30% Gap)**

#### **1. Branch Pattern Rules System**

- **Missing**: `src/cli/flow/branch_rules.rs`
- **Missing**: Pattern matching (exact: `develop`, wildcard: `release/*`)
- **Missing**: Number extraction from branch names (`release/1` → `rc.1`)
- **Missing**: Pre-release type mapping based on patterns
- **Missing**: Branch-specific post mode configuration

#### **2. Intelligent Branch Detection Logic**

- Current: All branches use hardcoded `alpha` defaults
- Missing: Automatic branch type detection:
    - `develop` → should be `beta.1` (currently: `alpha.*`)
    - `release/*` → should be `rc.{number}` (currently: `alpha.*`)
    - `feature/*` → should be `alpha.{hash}` ✅ (working)
    - `hotfix/*` → should be `alpha.{hash}` ✅ (working)

#### **3. Post Mode Differentiation**

- Current: All branches use commit distance mode
- Missing: Branch-specific post modes:
    - `release/*` should use tag distance from release tag
    - `develop` should use commit distance from branch point
    - `feature/*` should use commit distance from branch point

#### **4. Translation Layer**

- **Missing**: `src/cli/flow/translator.rs`
- **Missing**: `translate_flow_to_version_args()` function
- **Missing**: Branch pattern detection and application logic
- **Missing**: Override vs automatic detection logic

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

### **Phase 2: Translation Layer Implementation**

**Estimated Effort: 2 days**

#### **Step 2.1: Create Translation Module**

**File**: `src/cli/flow/translator.rs` (new)

```rust
pub fn translate_flow_to_version_args(
    flow_args: &FlowArgs,
    vcs_data: &VcsData,
) -> Result<VersionArgs, ZervError> {
    // Parse branch rules from FlowArgs
    let branch_rules = flow_args.parse_branch_rules()?;

    // Get current branch name
    let branch_name = vcs_data.current_branch.as_deref().unwrap_or("main");
    let rule = branch_rules.find_rule(branch_name);

    // Resolve final values (FlowArgs override branch rules)
    let pre_release_label = resolve_pre_release_label(flow_args, rule, branch_name)?;
    let pre_release_num = resolve_pre_release_num(flow_args, rule, branch_name)?;
    let post_mode = resolve_post_mode(flow_args, rule);

    // Build version args
    Ok(VersionArgs {
        input: flow_args.input.clone(),
        output: OutputConfig {
            output_format: "zerv".to_string(),
            output_template: None,
            output_prefix: None,
        },
        main: MainConfig {
            schema: flow_args.schema.clone(),
            schema_ron: None,
        },
        overrides: Default::default(),
        bumps: BumpsConfig {
            bump_pre_release_label: Some(pre_release_label),
            bump_pre_release_num: pre_release_num,
            bump_patch: flow_args.bump_patch(),
            bump_post: Some(post_mode),
            bump_dev: flow_args.bump_dev(),
            ..Default::default()
        },
    })
}

fn resolve_pre_release_label(
    flow_args: &FlowArgs,
    rule: Option<&BranchRule>,
    branch_name: &str
) -> Result<String, ZervError> {
    // 1. Explicit FlowArgs override
    if let Some(label) = flow_args.bump_pre_release_label() {
        return Ok(label);
    }

    // 2. Branch rule default
    if let Some(rule) = rule {
        return Ok(rule.pre_release_label.to_string());
    }

    // 3. Final fallback
    Ok("alpha".to_string())
}

fn resolve_pre_release_num(
    flow_args: &FlowArgs,
    rule: Option<&BranchRule>,
    branch_name: &str
) -> Result<Option<String>, ZervError> {
    // 1. Explicit FlowArgs override
    if let Some(num) = flow_args.bump_pre_release_num() {
        return Ok(Some(num));
    }

    // 2. Branch rule default
    if let Some(rule) = rule {
        if let Some(num) = &rule.pre_release_num {
            return Ok(Some(num.clone()));
        }

        // Extract number from branch pattern (e.g., "release/1" -> "1")
        if let Some(extracted) = extract_branch_number(&rule.pattern, branch_name) {
            return Ok(Some(extracted));
        }
    }

    // 3. Default: hash-based number
    Ok(Some(branch_hash_number(branch_name, 5)?))
}

fn resolve_post_mode(flow_args: &FlowArgs, rule: Option<&BranchRule>) -> String {
    // 1. Explicit FlowArgs override
    if let Some(mode) = flow_args.bump_post_mode() {
        return mode;
    }

    // 2. Branch rule default
    if let Some(rule) = rule {
        return rule.post_mode.to_string();
    }

    // 3. Final fallback
    "commit".to_string()
}

fn extract_branch_number(pattern: &str, branch_name: &str) -> Option<String> {
    // Extract "1" from "release/1" when pattern is "release/*"
    if !pattern.ends_with("/*") {
        return None;
    }

    let prefix = &pattern[..pattern.len() - 2]; // Remove "/*"
    if !branch_name.starts_with(prefix) {
        return None;
    }

    let remainder = &branch_name[prefix.len()..];
    // Extract first number from remainder
    let number_part: String = remainder.chars()
        .take_while(|c| c.is_numeric())
        .collect();

    if number_part.is_empty() {
        None
    } else {
        Some(number_part)
    }
}

// Helper implementations
impl PreReleaseLabel {
    pub fn to_string(&self) -> &'static str {
        match self {
            PreReleaseLabel::Alpha => "alpha",
            PreReleaseLabel::Beta => "beta",
            PreReleaseLabel::Rc => "rc",
        }
    }
}

impl PostMode {
    pub fn to_string(&self) -> &'static str {
        match self {
            PostMode::Tag => "tag",
            PostMode::Commit => "commit",
        }
    }
}
```

#### **Step 2.2: FlowArgs Integration with Branch Rules**

**File**: `src/cli/flow/args/mod.rs` (update existing)

```rust
impl FlowArgs {
    /// Parse and apply branch rules during validation
    pub fn resolve_branch_defaults(&self, vcs_data: &VcsData) -> Result<FlowArgs, ZervError> {
        // Parse branch rules (default or custom)
        let branch_rules = self.parse_branch_rules()?;

        // Get current branch name
        let branch_name = vcs_data.current_branch.as_deref().unwrap_or("main");

        // Resolve defaults from branch rules
        let branch_defaults = branch_rules.resolve_for_branch(branch_name);

        // Create new FlowArgs with branch defaults applied
        let mut resolved_args = self.clone();

        // Only set values if they weren't explicitly provided by user
        if resolved_args.bump_pre_release_label.is_none() {
            resolved_args.bump_pre_release_label = Some(branch_defaults.pre_release_label.to_string());
        }
        if resolved_args.bump_pre_release_num.is_none() {
            resolved_args.bump_pre_release_num = branch_defaults.pre_release_num.map(|n| n.to_string());
        }
        if resolved_args.bump_post_mode.is_none() {
            resolved_args.bump_post_mode = Some(branch_defaults.post_mode.to_string());
        }

        Ok(resolved_args)
    }

    /// Parse branch rules from FlowArgs (default or custom RON)
    pub fn parse_branch_rules(&self) -> Result<BranchRules, ZervError> {
        // If custom branch rules provided, parse them
        if let Some(ron_rules) = &self.branch_rules {
            let rules: Vec<BranchRule> = from_str(ron_rules)
                .map_err(|e| ZervError::InvalidFormat(format!("Failed to parse branch rules: {}", e)))?;
            return Ok(BranchRules { rules });
        }

        // Otherwise use default rules
        Ok(BranchRules::default_rules())
    }
}

// Add field to FlowArgs struct
pub struct FlowArgs {
    // ... existing fields ...
    /// Custom branch rules in RON format (optional)
    #[arg(long, help = "Custom branch rules in RON format")]
    pub branch_rules: Option<String>,
    // ... existing fields ...
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

| Phase     | Components                   | Effort        | Status      |
| --------- | ---------------------------- | ------------- | ----------- |
| Phase 1   | Branch rules system          | 2-3 days      | Pending     |
| Phase 2   | Translation layer            | 2 days        | Pending     |
| Phase 3   | Pipeline integration         | 1 day         | Pending     |
| Phase 4   | GitFlow test                 | 2-3 days      | Pending     |
| Phase 5   | Documentation                | 1 day         | Pending     |
| **Total** | **Complete GitFlow support** | **8-10 days** | **Planned** |

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
