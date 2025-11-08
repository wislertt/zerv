# Git Prefix Implementation Plan

## Overview

This document outlines the implementation plan to add a Git-specific 'g' prefix to commit hashes in Zerv version strings, solving the leading zero normalization issue in semver/pep440 parsing.

## Problem Statement

**Current Issue:**

- Template `{hex:7}` generates commit hashes that can start with leading zeros
- SemVer/PEP440 parsers normalize `0001234` → `1234` (loses leading zeros)
- Test expectations fail because expected 7 characters but get less
- Need to ensure commit hash parts always maintain exact length

**Example Problem:**

```
Expected: 1.0.1-alpha.12345.post.1+feature.test.2.0001234
Actual:   1.0.1-alpha.12345.post.1+feature.test.2.1234     (normalized)
```

## Solution: Git Prefix

Add a Git-specific 'g' prefix to commit hashes, following `git describe` convention:

- **Git**: `g3685e11` (following `git describe --tags --always` convention)

**Result:**

```
1.0.1-alpha.12345.post.1+feature.test.2.g3685e11  # Preserved as string, no normalization
```

## Architecture Analysis

Based on codebase exploration, here's the current Git-only VCS architecture:

### Current Git Implementation

- **VCS Detection**: `src/vcs/mod.rs` - `detect_vcs()` function (Git only)
- **VCS Trait**: `Vcs` trait with `get_vcs_data()` and `is_available()` methods
- **Git Implementation**: `src/vcs/git.rs` - `GitVcs` struct (only VCS implemented)
- **Data Structure**: `VcsData` in `src/vcs/vcs_data.rs`
- **Template Functions**: `src/cli/utils/template/functions.rs` - hash function `{hex:7}`

### Current Data Flow

1. VCS detection → `detect_vcs()` (always Git for now)
2. Data extraction → `vcs.get_vcs_data()` → `VcsData`
3. Conversion → `vcs_data_to_zerv_vars()` → `ZervVars`
4. Template rendering → Template functions resolve `{hex:7}`
5. Output formatting → Final version string

## Implementation Plan

### Phase 1: Add Git Prefix to VcsData

#### 1.1 Update VcsData Structure

**File**: `src/vcs/vcs_data.rs`

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct VcsData {
    // ... existing fields ...
    pub commit_hash_prefix: String,  // "g" for Git implementation
}
```

#### 1.2 Update GitVcs Implementation

**File**: `src/vcs/git.rs`

```rust
impl GitVcs {
    // In get_vcs_data() method:
    pub fn get_vcs_data(&self) -> Result<VcsData> {
        // ... existing implementation ...
        Ok(VcsData {
            // ... existing fields ...
            commit_hash_prefix: "g".to_string(),  // Git prefix following git describe convention
        })
    }
}
```

#### 1.3 Update Pipeline Data Conversion

**File**: `src/pipeline/vcs_data_to_zerv_vars.rs`

```rust
pub fn vcs_data_to_zerv_vars(vcs_data: VcsData) -> Result<ZervVars, ZervError> {
    let mut vars: ZervVars = /* existing conversion logic */;

    // Add Git prefix to commit hash if it exists
    if let Some(commit_hash) = vcs_data.commit_hash {
        vars.bumped_commit_hash = Some(format!("{}{}", vcs_data.commit_hash_prefix, commit_hash));
    }

    Ok(vars)
}
```

### Phase 2: Update Hash Length

#### 2.1 Update Hash Length to 8 Characters

**File**: `src/version/zerv/vars.rs`

```rust
fn derive_short_hash(hash: Option<&String>) -> Option<String> {
    hash.map(|h| {
        if h.len() >= 8 {
            h[..8].to_string()
        } else {
            h.clone()
        }
    })
}
```

Update corresponding tests to expect 8 characters:

```rust
#[case(Some("abcdef1234567890"), Some("abcdef12"))]
#[case(Some("abc123"), Some("abc123"))]
#[case(Some("a"), Some("a"))]
#[case(Some(""), Some(""))]
#[case(None, None)]

#[case(Some("fedcba0987654321"), Some("fedcba09"))]
#[case(Some("def456"), Some("def456"))]
#[case(Some("x"), Some("x"))]
#[case(Some(""), Some(""))]
#[case(None, None)]
```

### Phase 3: Update Test Expectations

#### 3.1 Update Test Expectations for Prefixed Hashes

**File**: `src/cli/flow/pipeline.rs`

Update all test expectations to account for the 'g' prefix that will now be included in `bumped_commit_hash_short`:

```rust
// Current:
&format!("1.0.1-alpha.{}.post.1+feature.1.1.{{hex:7}}", branch_feature_1_hash),

// New: hex:7 will now include the 'g' prefix automatically since bumped_commit_hash_short contains it
&format!("1.0.1-alpha.{}.post.1+feature.1.1.g{{hex:7}}", branch_feature_1_hash),
```

Note: Standard schema templates do NOT need to be updated since `bumped_commit_hash_short` will already contain the prefixed hash from the pipeline.

### Phase 5: Testing Strategy

#### 5.1 Unit Tests

- Test Git prefix field in VcsData is set to "g"
- Test pipeline concatenates prefix + hash correctly
- Test leading zero scenarios (e.g., `g0001234` stays as string)
- Test template rendering with explicit 'g' prefix

#### 5.2 Integration Tests

- Test complete pipeline with Git VCS
- Test version string generation with prefixed hashes
- Test that schema templates render correctly with 'g' prefix

## Implementation Order

1. **Phase 1**: Add `commit_hash_prefix` field to VcsData and update GitVcs
2. **Phase 2**: Update pipeline to concatenate prefix with commit hash
3. **Phase 3**: Update test expectations to account for prefixed hashes
4. **Phase 4**: Comprehensive testing

## Backward Compatibility

- Hash length remains 7 characters for consistency
- Prefix is added at data level in pipeline
- Schema templates remain unchanged (no template modifications needed)
- Only test expectations need updates

## Risk Mitigation

- **Breaking Changes**: Ensure existing functionality works during transition
- **Data Consistency**: Ensure prefix is consistently applied across all hash fields
- **Testing**: Comprehensive test coverage for all scenarios

## Success Criteria

- ✅ VcsData contains `commit_hash_prefix: "g"` for Git repositories
- ✅ Git version strings show `g` prefix: `g3685e11`
- ✅ Leading zeros preserved: `g0001234` stays as string
- ✅ Tests pass with exact 7-character expectations including prefix
- ✅ Existing schema templates work without modification
- ✅ Pipeline correctly concatenates prefix + hash
- ✅ `bumped_commit_hash_short` returns prefixed hash automatically
