# Semantic Versioning Bump Behavior Analysis

## Problem Statement

The current bump implementation is **additive only** - it just adds numbers without following semantic versioning standards. We need to define the correct behavior for ALL bump variables.

## Current Behavior (Additive Only)

**Examples:**

- `1.2.3` + `--bump-major` → `2.2.3` ❌ (Wrong!)
- `1.2.3` + `--bump-minor` → `1.3.3` ❌ (Wrong!)
- `1.2.3` + `--bump-patch` → `1.2.4` ✅ (Correct)

## Comprehensive Bump Behavior Design

### Version Component Bumps (Semantic Versioning Rules)

**Single Bump Behavior:**

- `--bump-major`: Increment major, reset minor and patch to 0
- `--bump-minor`: Increment minor, reset patch to 0
- `--bump-patch`: Increment patch only

**Multiple Bump Behavior:**

- Process all explicitly specified version bumps
- Reset only happens for levels not explicitly bumped
- User gets exactly what they asked for

**Examples:**

```bash
# Single version bumps
1.2.3 + --bump-major → 2.0.0 ✅
1.2.3 + --bump-minor → 1.3.0 ✅
1.2.3 + --bump-patch → 1.2.4 ✅

# Multiple version bumps
1.2.3 + --bump-major --bump-minor 2 → 2.2.0 ✅ (major to 2, minor from 0 to 2)
1.2.3 + --bump-minor --bump-patch 5 → 1.3.5 ✅ (minor to 3, patch from 0 to 5)
1.2.3 + --bump-major --bump-minor 2 --bump-patch 3 → 2.2.3 ✅ (all explicit)
```

### Pre-release Component Bumps

**Behavior:**

- `--bump-pre-release-num`: Add to pre-release number (creates alpha label if none exists)
- `--pre-release-label`: Change pre-release label, preserve number
- `--bump-pre-release-label`: Change pre-release label, reset number to 0
- Pre-release bumps reset post/dev components (lower precedence)

**Examples:**

```bash
# Pre-release bumps
1.2.3-alpha.1 + --bump-pre-release-num 2 → 1.2.3-alpha.3 ✅
1.2.3-beta.5 + --bump-pre-release-num → 1.2.3-beta.6 ✅
1.2.3 + --bump-pre-release-num 2 → 1.2.3-alpha.2 ✅ (creates alpha label)

# Pre-release label bumps
1.2.3-alpha.1 + --pre-release-label beta → 1.2.3-beta.1 ✅
1.2.3-beta.5 + --pre-release-label rc → 1.2.3-rc.5 ✅
1.2.3 + --pre-release-label alpha → 1.2.3-alpha.0 ✅ # (creates if doesn't exist)

# Pre-release label bump (resets number)
1.2.3-alpha.5 + --bump-pre-release-label beta → 1.2.3-beta.0 ✅
1.2.3-beta.3 + --bump-pre-release-label rc → 1.2.3-rc.0 ✅
1.2.3 + --bump-pre-release-label alpha → 1.2.3-alpha.0 ✅ # (creates with reset)

# Pre-release with post/dev components
1.2.3-alpha.1.post2.dev5 + --bump-pre-release-num 2 → 1.2.3-alpha.3 ✅ # (resets post/dev)
1.2.3-alpha.1.post2.dev5 + --pre-release-label beta → 1.2.3-beta.1.post2.dev5 ✅
1.2.3-alpha.1.post2.dev5 + --bump-pre-release-label rc → 1.2.3-rc.0 ✅ # (resets post/dev)
1.2.3.post2.dev5 + --pre-release-label alpha → 1.2.3-alpha.0.post2.dev5 ✅
1.2.3.post2.dev5 + --bump-pre-release-label beta → 1.2.3-beta.0 ✅ # (resets post/dev)
```

```bash
# Pre-release label + number bumps
1.2.3-alpha.1 + --pre-release-label beta --bump-pre-release-num 2 → 1.2.3-beta.3 ✅
1.2.3-beta.5 + --pre-release-label rc --bump-pre-release-num 1 → 1.2.3-rc.6 ✅
1.2.3 + --pre-release-label alpha --bump-pre-release-num 3 → 1.2.3-alpha.3 ✅ # (creates with bump)

# Pre-release with post/dev bumps
1.2.3-alpha.1.post2.dev5 + --bump-post 1 --bump-dev 2 → 1.2.3-alpha.1.post3.dev7 ✅
1.2.3-alpha.1.post2.dev5 + --bump-pre-release-num 1 --bump-post 2 --bump-dev 3 → 1.2.3-alpha.2.post2.dev3 ✅
```

#### Edge Cases and Concerns

##### 1. Conflicting Pre-release Label Operations

**Issue**: Both `--pre-release-label` and `--bump-pre-release-label` can be specified together
**Missing**: Validation to prevent conflicting operations
**Resolution**: Early validation error when both are specified
**Edge case**: `--pre-release-label` and `--bump-pre-release-label` together → **Early validation error** (conflicting operations)

##### 2. Pre-release Creation Behavior

**Issue**: No clear rules for creating pre-release when none exists
**Resolution**: `--bump-pre-release-num` creates pre-release with default "alpha" label
**Edge case**: `1.2.3 + --bump-pre-release-num 2` → `1.2.3-alpha.2` (creates with alpha label)

##### 3. Pre-release Label Validation

**Issue**: No validation rules for pre-release labels
**Missing**: What labels are valid? Case sensitivity? Special characters?
**Resolution**: Validate against SemVer spec (alphanumeric + hyphens only), case-sensitive
**Edge case**: `--pre-release-label "invalid!"` → **Error** (invalid characters)

##### 4. Complex Pre-release Identifiers

**Issue**: Assumes simple `label.number` format
**Missing**: Multiple identifiers like `alpha.1.2.3` or `alpha.beta.1`
**Resolution**: Only first recognized label becomes pre-release component, rest become extra_core components
**Edge case**: `1.2.3-alpha.beta.1` → pre-release: `{label: "alpha", num: 0}`, extra_core: `[VarField("pre_release"), String("beta"), Integer(1)]`

### Post-release Component Bumps

**Behavior:**

- `--bump-post`: Add to post number
- `--bump-dev`: Add to dev number
- No reset behavior for post-release components

**Examples:**

```bash
# Post-release bumps
1.2.3.post1 + --bump-post 2 → 1.2.3.post3 ✅
1.2.3.dev5 + --bump-dev 3 → 1.2.3.dev8 ✅
1.2.3 + --bump-post 1 → 1.2.3.post1 ✅ (creates if doesn't exist)
```

### Epoch Component Bumps

**Behavior:**

- `--bump-epoch`: Add to epoch number
- No reset behavior for epoch

**Examples:**

```bash
# Epoch bumps
1!1.2.3 + --bump-epoch 1 → 2!0.0.0 ✅
1.2.3 + --bump-epoch 1 → 1!0.0.0 ✅ (creates if doesn't exist)
```

### VCS Component Bumps

**Behavior:**

- `--bump-distance`: **REMOVED** - Distance is VCS metadata, not a version component
- Use `--distance` for override instead of bump
- No reset behavior for VCS components

**Examples:**

```bash
# VCS overrides (not bumps)
1.2.3.post5+main.abc123 + --distance 7 → 1.2.3.post7+main.abc123 ✅
1.2.3 + --distance 1 → 1.2.3.post1+main.abc123 ✅ (creates if doesn't exist)
```

### Mixed Component Bumps

**Behavior:**

- Version components follow semantic versioning rules
- Other components are always additive
- All explicitly specified bumps are processed

**Examples:**

```bash
# Mixed bumps
1.2.3 + --bump-major --bump-post 2 --bump-dev 1 → 2.0.0.post2.dev1 ✅
1.2.3-alpha.1 + --bump-minor --bump-pre-release-num 3 → 1.3.0-alpha.3 ✅
1.2.3 + --bump-patch --bump-epoch 1 → 1!0.0.1 ✅

# Complex pre-release scenarios
1.2.3-alpha.1.post2.dev5 + --bump-major → 2.0.0 ✅ (major resets minor/patch/pre/post/dev)
1.2.3-alpha.1.post2.dev5 + --bump-minor --bump-pre-release-num 2 → 1.3.0-alpha.2 ✅ (minor resets patch/pre/post/dev, pre-release creates alpha label)
1.2.3-alpha.1.post2.dev5 + --bump-patch --bump-post 1 --bump-dev 1 → 1.2.4.post1.dev1 ✅ (patch resets pre/post/dev, then bumps post/dev)
1.2.3-alpha.1.post2.dev5 + --bump-major --bump-minor 2 --bump-patch 3 --bump-pre-release-num 1 --bump-post 1 --bump-dev 1 → 2.2.3-alpha.1.post1.dev1 ✅ (all explicit)
```

## Design Principles

### 1. Component Precedence Hierarchy

- **Higher precedence bumps reset all lower precedence components**
- **Component precedence order**: Epoch → Major → Minor → Patch → Pre-release → Post → Dev
- **Reset behavior**: When bumping a component, all components to its right are reset to 0 or removed
- **Example**: `--bump-major` resets minor, patch, pre-release, post, dev, etc.

### 2. Explicit vs Implicit Processing

- When user explicitly specifies multiple bumps, process all of them
- Higher precedence bumps reset lower precedence components, then explicitly specified components are bumped from 0
- Design is flexible enough to achieve what users are looking for in the most intuitive way

### 3. Error Handling

- Pre-release bumps create alpha label if none exists
- Other components can be created if they don't exist
- Clear and early error messages for invalid operations

## Edge Cases and Special Scenarios

### None Value Handling

- If version component is None, treat as 0 before bumping
- If metadata component is None, create it with the bump value
- Reset behavior applies to None values (set to 0)

### Override vs Bump Interaction

- Overrides (`--major 2`) set absolute values, no reset behavior
- Bumps (`--bump-major`) follow semantic versioning rules
- Overrides take precedence over bumps

### Complex Mixed Scenarios

- Version components can be mixed with metadata components
- Each component type follows its own rules
- All explicitly specified operations are processed

## Implementation Changes Required

### 1. Remove `--bump-distance` Flag

**Current Issue**: `--bump-distance` is a misuse of the bump concept
**Reason**: Distance is VCS metadata, not a version component that should be "bumped"
**Solution**: Remove `--bump-distance` entirely, keep only `--distance` for override

**Files to change:**

- `src/cli/version/args.rs` - Remove `bump_distance` field
- `src/version/zerv/bump/mod.rs` - Remove distance bump logic
- `src/version/zerv/bump/vars_secondary.rs` - Remove `bump_distance` method
- `src/test_utils/bump_type.rs` - Remove `Distance` variant
- All test files - Remove distance bump tests

### 2. Add `--bump-pre-release-label` Flag

**Current Issue**: Missing flag for pre-release label bumps with reset behavior
**Solution**: Add `--bump-pre-release-label` that changes label and resets number to 0

**Files to change:**

- `src/cli/version/args.rs` - Add `bump_pre_release_label` field
- `src/version/zerv/bump/vars_secondary.rs` - Add `bump_pre_release_label` method
- `src/version/zerv/bump/mod.rs` - Add bump logic for pre-release label

### 3. Update Precedence Reset Logic

**Current Issue**: Reset behavior not implemented for pre-release bumps
**Solution**: Implement reset logic where higher precedence bumps reset lower precedence components

**Files to change:**

- `src/version/zerv/bump/vars_primary.rs` - Update major/minor/patch bumps to reset pre-release
- `src/version/zerv/bump/vars_secondary.rs` - Update pre-release bumps to reset post/dev

### 4. Add Pre-release Creation Logic

**Current Issue**: No clear rules for creating pre-release when none exists
**Solution**: `--bump-pre-release-num` should create pre-release with "alpha" label if none exists

**Files to change:**

- `src/version/zerv/bump/vars_secondary.rs` - Update `bump_pre_release` method

### 5. Add Validation for Conflicting Flags

**Current Issue**: No validation for conflicting pre-release flags
**Solution**: Add early validation error when both `--pre-release-label` and `--bump-pre-release-label` are specified

**Files to change:**

- `src/cli/version/args.rs` - Add validation logic
- `src/error.rs` - Add new error type for conflicting flags

### 6. Add Pre-release Label Validation

**Current Issue**: No validation for pre-release labels
**Solution**: Validate labels against SemVer spec (alphanumeric + hyphens only)

**Files to change:**

- `src/version/zerv/bump/vars_secondary.rs` - Add label validation
- `src/error.rs` - Add validation error types
