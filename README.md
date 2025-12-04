[![tests](https://img.shields.io/github/actions/workflow/status/wislertt/zerv/ci-test.yml?branch=main&label=tests&logo=github)](https://github.com/wislertt/zerv/actions/workflows/ci-test.yml)
[![release](https://img.shields.io/github/actions/workflow/status/wislertt/zerv/cd.yml?branch=main&label=release&logo=github)](https://github.com/wislertt/zerv/actions/workflows/cd.yml)
[![quality gate status](https://sonarcloud.io/api/project_badges/measure?project=wislertt_zerv&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=wislertt_zerv)
[![security rating](https://sonarcloud.io/api/project_badges/measure?project=wislertt_zerv&metric=security_rating)](https://sonarcloud.io/summary/new_code?id=wislertt_zerv)
[![vulnerabilities](https://sonarcloud.io/api/project_badges/measure?project=wislertt_zerv&metric=vulnerabilities)](https://sonarcloud.io/summary/new_code?id=wislertt_zerv)
[![codecov](https://img.shields.io/codecov/c/github/wislertt/zerv?token=549GL6LQBX&label=codecov&logo=codecov)](https://codecov.io/gh/wislertt/zerv)
[![crates.io](https://img.shields.io/crates/v/zerv?color=green)](https://crates.io/crates/zerv)
[![downloads](https://img.shields.io/crates/d/zerv?label=downloads&color=green)](https://crates.io/crates/zerv)

# zerv

**Automatic versioning for every commit** - Generate semantic versions from any commit across all branches, or dirty working directory, with seamless pre-release handling and flexible format support for any CI/CD workflow.

## Quick Start

```bash
# Install
cargo install zerv

# Try automated versioning (current branch determines output)
zerv flow
# → 1.0.0 (on main branch with tag v1.0.0)
# → 1.0.1-rc.1.post.3 (on release branch with pre-release tag)
# → 1.0.1-beta.1.post.3+develop.3.gf297dd0 (on develop branch)
# → 1.0.1-alpha.59394.post.1+feature.new.auth.1.g4e9af24 (on feature branch)
# → 1.0.1-alpha.17015.post.1.dev.1764382150+feature.dirty.work.1.g54c499a (on dirty feature branch)
```

<!-- Corresponding test: tests/integration_tests/flow/docs/quick_start.rs:test_quick_start_documentation_examples -->

## Key Features

- **zerv version**: Flexible, configurable version generation with full control
- **zerv flow**: Opinionated, automated pre-release management based on Git branches
- **Smart Schema System**: Auto-detects clean releases, pre-releases, and build context
- **Multiple Formats**: SemVer, PEP440 (Python), CalVer, custom schemas
- **CI/CD Integration**: Complements semantic release with branch-based pre-releases and full override control

## Usage Examples

### zerv flow: Automated branch-based versions

**Purpose**: Intelligent pre-release management that automatically generates meaningful versions from any Git state without manual decisions.

#### Core Principles

1. **Semantic state capture** - Extract semantic meaning from ANY Git state (any branch, any commit, uncommitted changes)
2. **Multi-format output** - Transform semantic meaning into various version formats with customizable format support
3. **Seamless semantic release integration** - Work with semantic release tools while providing fully automated pre-release versioning
4. **Build traceability** - Include sufficient context to trace versions back to exact Git states

#### Version Format Explained

**Full Example**: `1.0.1-alpha.12345.post.3.dev.1729924622+feature.auth.1.f4a8b9c`

**Structure**: `<BASE>-<PRE_RELEASE>.<POST>[.<DEV>][+BUILD_CONTEXT]`

- **`1.0.1`** - Base version (semantic meaning from tags)
- **`alpha.12345`** - Pre-release type and branch identification
- **`post.3`** - Commits since reference point
- **`[.dev.timestamp]`** - Optional dev timestamp for uncommitted changes
- **`[+BUILD_CONTEXT]`** - Optional build context for traceability

**Key Point**: The core version `<BASE>-<PRE_RELEASE>.<POST>[.<DEV>]` contains all semantic meaning needed to understand Git state. The build context `[+BUILD_CONTEXT]` is optional and provides additional verbose information for easier interpretation and traceability.

**Version Variations**:

- **Tagged release**: `1.0.1`
- **Tagged pre-release**: `2.0.1-rc.1.post.2`
- **Branch from Tagged release**: `1.0.1-alpha.54321.post.1+feature.login.1.f4a8b9c`
- **Branch from Tagged pre-release**: `2.0.1-alpha.98765.post.3+fix.auth.bug.1.c9d8e7f`
- **Uncommitted changes**: `2.0.1-alpha.98765.post.4.dev.1729924622+fix.auth.bug.1.c9d8e7f`

#### Pre-release Resolution Strategy

**Default behavior**: All branches start as `alpha.<hash-id>` (hash-based identification)

**Configurable branch patterns**: Users can configure specific branches to use custom pre-release types (alpha, beta, rc) with optional numbers:

- Example: `feature/user-auth` branch → `beta.12345` (label only, uses hash-based number)
- Example: `develop` branch → `beta.1` (label and custom number for stable branches)
- Any branch can be mapped to any pre-release type (alpha, beta, rc) with hash-based or custom numbers

**Branch name resolution**: Extract pre-release information from branch name patterns:

- Example: `release/1/feature-auth-fix` → `rc.1` (extracts number from branch pattern)
- Simplified GitFlow-inspired naming conventions

- **Note**: Branch names are conventions, not strict requirements - Zerv provides flexible pattern matching and user configuration.

**Clean branches**: `main`, `master` → No pre-release (clean releases)

**Post-release resolution logic**:

- **Configurable post representation** with two options:
    - **Tag Distance**: Count commits from last tag
    - **Commit Distance**: Count commits from branch creation point
- **Default**: Tag Distance (most common use case)
- **`post.0`**: Exactly on reference point (no commits since)
- **`post.N`**: N commits since reference point
- **Consistent across all branch types** (alpha, beta, rc, etc.)

**Examples**:

**Tag Distance (release branches):**

```
main: v1.0.0 (tag)
└── release/1 (created) → create tag v1.0.1-rc.1.post.1
    └── 1 commit → 1.0.1-rc.1.post.1.dev.1729924622  (same post, dev timestamp)
    └── 2 commits → 1.0.1-rc.1.post.1.dev.1729924623  (same post, dev timestamp)
    └── create tag → 1.0.1-rc.1.post.2  (new tag increments post)
    └── more commits → 1.0.1-rc.1.post.2.dev.1729924624  (new post, dev timestamp)
```

**Commit Distance (develop branch):**

```
main: v1.0.0 (tag)
└── develop (created from v1.0.0) → commit 1.0.1-beta.1.post.1  (1 commits since branch creation)
    └── 5 commits later → 1.0.1-beta.1.post.6  (6 commits since branch creation)
    └── 1 more commit → 1.0.1-beta.1.post.7  (7 commits since branch creation)
```

#### Workflow Examples

This section demonstrates how Zerv Flow works across different branching strategies and Git scenarios.

**Note**: To keep diagrams clean and readable, build context is omitted from version strings in the examples. Dirty state (`.dev.timestamp`) is shown in diagrams when applicable.

**Example**: A commit appears as `1.0.1-alpha.12345.post.3.dev.1729924622` in the diagrams. With build context enabled: `1.0.1-alpha.12345.post.3.dev.1729924622+feature.user-auth.3.a1b2c3d`

##### Trunk-Based Development

**Purpose**: Complex trunk-based workflow with parallel features, nested branches, and synchronization scenarios.

**Scenario**: Development from `v1.0.0` with parallel feature branches, synchronization, and nested development.

```mermaid
---
config:
  logLevel: 'debug'
  theme: 'base'
---
gitGraph
    %% Step 1: Initial commit on main with v1.0.0 tag
    commit id: "1.0.0"

    %% Step 2: Create parallel feature branches feature-1 and feature-2 from main
    branch feature-1 order: 2
    branch feature-2 order: 3

    %% Step 3: feature-2: Start development with dirty state
    checkout feature-2
    commit type:REVERSE id: "1.0.1-alpha.68031.post.0.dev.{timestamp}" tag: "uncommitted"

    %% Step 4: feature-2: Create first commit
    commit id: "1.0.1-alpha.68031.post.1"

    %% Step 5: feature-1: Create commits (parallel development)
    checkout feature-1
    commit id: "1.0.1-alpha.42954.post.1"
    commit id: "1.0.1-alpha.42954.post.2"

    %% Step 6: feature-1: Merge to main and release v1.0.1
    checkout main
    merge feature-1 id: "1.0.1" tag: "feature-1 released"

    %% Step 7: feature-2: Sync with main to get feature-1 changes
    checkout feature-2
    merge main id: "1.0.2-alpha.68031.post.2"

    %% Step 8: feature-2: Create additional commit
    commit id: "1.0.2-alpha.68031.post.3"

    %% Step 9: feature-3: Branch from feature-2 for sub-feature development
    branch feature-3 order: 4
    checkout feature-3
    commit id: "1.0.2-alpha.14698.post.4"

    %% Step 10: feature-3: Continue development with dirty state
    commit type:REVERSE id: "1.0.2-alpha.14698.post.4.dev.{timestamp}" tag: "uncommitted"

    %% Step 11: feature-3: Continue development with commits
    commit id: "1.0.2-alpha.14698.post.5"
    commit id: "1.0.2-alpha.14698.post.6"

    %% Step 12: feature-2: Merge feature-3 back to continue development
    checkout feature-2
    merge feature-3 id: "1.0.2-alpha.68031.post.6" tag: "feature-3 merged"

    %% Step 13: feature-2: Final development before release
    commit id: "1.0.2-alpha.68031.post.7"

    %% Step 14: Final release: feature-2 merges to main and releases v1.1.0
    checkout main
    merge feature-2 id: "1.1.0" tag: "feature-2 released"
```

**Key behaviors demonstrated**:

- **Parallel development**: `feature-1` and `feature-2` get unique hash IDs (`42954`, `68031`)
- **Version progression**: Base version updates when syncing (`1.0.1` → `1.0.2`)
- **Dirty state**: Uncommitted changes show `.dev.timestamp` suffix
- **Nested branches**: `feature-3` branches from `feature-2` with independent versioning
- **Clean releases**: Main branch maintains semantic versions on merges

<!-- Corresponding test: tests/integration_tests/flow/scenarios/trunk_based.rs:test_trunk_based_development -->

##### GitFlow Branching Strategy

**Purpose**: GitFlow methodology with proper pre-release type mapping and merge patterns.

**Scenario**: Main branch with `v1.0.0`, develop branch integration, feature development, hotfix emergency flow, and release preparation.

```mermaid
---
config:
  logLevel: 'debug'
  theme: 'base'
---
gitGraph
    %% Step 1: Initial state: main and develop branches
    commit id: "1.0.0"

    %% Step 2: Create develop branch with initial development commit
    branch develop order: 3
    checkout develop
    commit id: "1.0.1-beta.1.post.1"

    %% Step 3: Feature development from develop branch
    branch feature/auth order: 4
    checkout feature/auth
    commit id: "1.0.1-alpha.92409.post.2"
    commit id: "1.0.1-alpha.92409.post.3"

    checkout develop
    %% Step 4: Merge feature/auth back to develop
    merge feature/auth id: "1.0.1-beta.1.post.3" tag: "feature merged"

    %% Step 5: Hotfix emergency flow from main
    checkout main
    branch hotfix/critical order: 1
    checkout hotfix/critical
    commit id: "1.0.1-alpha.11477.post.1"

    checkout main
    %% Step 6: Merge hotfix to main and release v1.0.1
    merge hotfix/critical id: "1.0.1" tag: "hotfix released"

    %% Step 7: Sync develop with main changes and continue development
    checkout develop
    merge main id: "1.0.2-beta.1.post.4" tag: "sync main"

    %% Step 8: Continue development on develop branch
    commit id: "1.0.2-beta.1.post.5"

    %% Step 9: Release branch preparation
    branch release/1 order: 2
    checkout release/1
    commit id: "1.0.2-rc.1.post.1"
    commit id: "1.0.2-rc.1.post.2"
    commit type:REVERSE id: "1.0.2-rc.1.post.3.dev.{timestamp}" tag: "uncommitted"
    commit id: "1.0.2-rc.1.post.3"

    checkout main
    %% Step 10: Final release: merge release/1 to main
    merge release/1 id: "1.1.0" tag: "release 1.1.0"

    %% Step 11: Sync develop with release and prepare for next cycle
    checkout develop
    merge main id: "1.1.1-beta.1.post.1" tag: "sync release"
```

**Key behaviors demonstrated**:

- **Beta pre-releases**: Develop branch uses `beta` for integration builds
- **Alpha pre-releases**: Feature branches use `alpha` with hash-based identification
- **RC pre-releases**: Release branches use `rc` for release candidates
- **Clean releases**: Main branch maintains clean versions without pre-release suffixes
- **Hotfix flow**: Emergency fixes from main with proper version propagation
- **Branch synchronization**: Develop branch syncs with main releases

<!-- Corresponding test: tests/integration_tests/flow/scenarios/gitflow.rs:test_gitflow_development_flow -->

##### Complex Release Management

**Purpose**: Complex release branch scenarios including branch abandonment and cascading release preparation.

**Scenario**: Main branch with `v1.0.0`, release branch preparation with critical issues leading to abandonment, and selective branch creation for successful release.

```mermaid
---
config:
  logLevel: 'debug'
  theme: 'base'
---
gitGraph
    %% Step 1: Initial state: main branch with v1.0.0 tag
    commit id: "1.0.0" tag: "v1.0.0"

    %% Step 2: Create release/1 from main for next release preparation
    branch release/1 order: 2
    checkout release/1
    commit id: "1.0.1-rc.1.post.1"
    commit id: "1.0.1-rc.1.post.2"

    %% Step 3: Create release/2 from the second commit of release/1 (before issues)
    %% release/1 at this point: 1.0.1-rc.1.post.2, so release/2 continues from there
    checkout release/1
    branch release/2 order: 1
    checkout release/2
    commit id: "1.0.1-rc.2.post.3"

    %% Step 4: Go back to release/1 and add the problematic third commit (issues found)
    checkout release/1
    commit id: "1.0.1-rc.1.post.3" tag: "issues found"

    %% Step 5: release/2 completes preparation successfully
    checkout release/2
    commit id: "1.0.1-rc.2.post.4"

    %% Step 6: Merge release/2 to main and release v1.1.0
    checkout main
    merge release/2 id: "1.1.0" tag: "v1.1.0"

```

**Version progression details**:

- **release/1**: `1.0.1-rc.1.post.1` → `1.0.1-rc.1.post.2` → `1.0.1-rc.1.post.3` (abandoned)
- **release/2**: Created from `release/1`'s second commit (`1.0.1-rc.1.post.2`), continues as `1.0.1-rc.2.post.3` → `1.0.1-rc.2.post.4`
- **Main**: Clean progression `1.0.0` → `1.1.0` (only from successful `release/2` merge)

**Key behaviors demonstrated**:

- **Branch isolation**: Each release branch maintains independent versioning regardless of parent/child relationships
- **Selective branching**: Zerv Flow correctly handles branches created from specific historical commits
- **Abandonment handling**: Unmerged branches don't affect final release versions on main
- **Cascade management**: Complex branching scenarios where releases feed into other releases are handled transparently
- **Clean main branch**: Main only receives versions from successfully merged releases, maintaining clean semantic versioning

<!-- Corresponding test: tests/integration_tests/flow/scenarios/complex_release_branch.rs:test_complex_release_branch_abandonment -->

#### Schema Variants: 10+ Standard Schema Presets

**Purpose**: Complete control over version generation with 20+ schema presets and extensive customization options.

**Schema Selection Examples**:

```bash
zerv flow --schema standard-base
# → 1.0.1 (test case 1)

zerv flow --schema standard-base-context
# → 1.0.1+branch.name.g4e9af24 (test case 2)

zerv flow --schema standard-base-prerelease
# → 1.0.1-alpha.10192 (test case 3)

zerv flow --schema standard-base-prerelease-context
# → 1.0.1-alpha.10192+branch.name.1.g4e9af24 (test case 4)

zerv flow --schema standard-base-prerelease-post
# → 1.0.1-alpha.10192.post.1 (test case 5)

zerv flow --schema standard-base-prerelease-post-context
# → 1.0.1-alpha.10192.post.1+branch.name.1.g4e9af24 (test case 6)

zerv flow --schema standard-base-prerelease-post-dev
# → 1.0.1-alpha.10192.post.1.dev.1764382150 (test case 7)

zerv flow --schema standard-base-prerelease-post-dev-context
# → 1.0.1-alpha.10192.post.1.dev.1764382150+branch.name.1.g4e9af24 (test case 8)

zerv flow --schema standard
# → 1.0.0 (clean main - test case 9)
# → 1.0.1-rc.1 (release branch - test case 10)
# → 1.0.1-alpha.10192.post.1+branch.name.1.g4e9af24 (feature branch - test case 11)
# → 1.0.1-alpha.10192.post.1.dev.1764382150+branch.name.1.g4e9af24 (dirty feature branch - test case 12)

zerv flow --schema standard-no-context
# → 1.0.0 (clean main - test case 13)
# → 1.0.1-rc.1 (release branch - test case 14)
# → 1.0.1-alpha.10192.post.1 (feature branch - test case 15)
# → 1.0.1-alpha.10192.post.1.dev.1764382150 (dirty feature branch - test case 16)

zerv flow --schema standard-context
# → 1.0.0+main.g4e9af24 (clean main - test case 17)
# → 1.0.1-rc.1+release.1.do.something.g4e9af24 (release branch - test case 18)
# → 1.0.1-alpha.10192.post.1+branch.name.1.g4e9af24 (feature branch - test case 19)
# → 1.0.1-alpha.10192.post.1.dev.1764382150+branch.name.1.g4e9af24 (dirty feature branch - test case 20)
```

<!-- Corresponding test: tests/integration_tests/flow/docs/schema_variants.rs:test_schema_variants_documentation_examples -->

#### Branch Rules: Configurable Pattern Matching

**Purpose**: Map branch names to pre-release labels, numbers, and post modes for automated version generation.

**Default GitFlow Rules**:

```ron
[
    (pattern: "develop", pre_release_label: beta, pre_release_num: 1, post_mode: commit),
    (pattern: "release/*", pre_release_label: rc, post_mode: tag)
]
```

**Pattern Matching**:

- **Exact**: `"develop"` matches only `"develop"`
- **Wildcard**: `"release/*"` matches `"release/1"`, `"release/42"`, `"release/1/feature"`, etc.
- **Number extraction**:
    - With numbers: `release/1` → `rc.1`, `release/1/feature` → `rc.1`
    - Without numbers: `release/feature` → `rc.<hash-id>` (fallback to hash-based identification)
- **Other branches**: `*`, `feature/*`, `hotfix/*`, `bugfix/*`, etc. → `alpha.<hash-id>` (fallback to hash-based identification)

**Examples**:

```bash
# Default GitFlow behavior
zerv flow
# → 1.0.1-rc.1.post.1+release.1.do.something.1.g3a2b1c4 (release/1/do-something branch - test case 1)
# → 1.0.1-beta.1.post.1+develop.1.g8f7e6d5 (develop branch - test case 2)
# → 1.0.1-alpha.10192.post.1+branch.name.1.g9d8c7b6 (feature branch - test case 3)
# → 1.0.1-rc.48993.post.1+release.do.something.1.g5e4f3a2 (release/do-something branch - test case 4)

# Custom branch rules
zerv flow --branch-rules '[
    (pattern: "staging", pre_release_label: rc, pre_release_num: 1, post_mode: commit),
    (pattern: "qa/*", pre_release_label: beta, post_mode: tag)
]'
# → 1.0.1-rc.1.post.1+staging.1.g2c3d4e5 (staging branch - test case 5)
# → 1.0.1-beta.123.post.1+qa.123.1.g7b8c9d0 (qa branch - test case 6)
# → 1.0.1-alpha.20460.post.1+feature.new.feature.1.g1d2e3f4 (feature branch - test case 7)
```

**Configuration**:

- **`pattern`**: Branch name (exact) or wildcard (`/*`)
- **`pre_release_label`**: `alpha`, `beta`, or `rc`
- **`pre_release_num`**: Explicit number (exact) or extracted (wildcard)
- **`post_mode`**: `commit` (count commits) or `tag` (count tags)

<!-- Corresponding test: tests/integration_tests/flow/docs/branch_rules.rs:test_branch_rules_documentation_examples -->

#### Override Controls: Complete Version Customization

**Override Options**: VCS, version components, and pre-release controls

```bash
# VCS Overrides
zerv flow --tag-version "v2.1.0-beta.1"           # Override tag version
# → 2.1.0

zerv flow --distance 42                           # Override distance from tag
# → 1.0.1-alpha.60124.post.42+feature.test.42.g8f4e3a2

zerv flow --dirty                                 # Force dirty=true
# → 1.0.1-alpha.18373.dev.1729927845+feature.dirty.ga1b2c3d

zerv flow --no-dirty                              # Force dirty=false
# → 1.0.0+feature.clean.g4d5e6f7

zerv flow --clean                                 # Force clean state (distance=0, dirty=false)
# → 1.0.0+feature.clean.force.g8a9b0c1

zerv flow --bumped-branch "release/42"           # Override branch name
# → 1.0.1-rc.42.post.1+release.42.1.g2c3d4e5

zerv flow --bumped-commit-hash "a1b2c3d"         # Override commit hash
# → 1.0.1-alpha.48498.post.1+feature.hash.1.a1b2c3d

zerv flow --bumped-timestamp 1729924622          # Override timestamp
# → 1.0.1-alpha.18321.dev.1764598322+feature.timestamp.g7f8e9a0

# Version Component Overrides
zerv flow --major 2                              # Override major
# → 2.0.0

zerv flow --minor 5                              # Override minor
# → 1.5.0

zerv flow --patch 3                              # Override patch
# → 1.0.3

zerv flow --epoch 1                              # Override epoch
# → 1.0.0-epoch.1

zerv flow --post 7                               # Override post
# → 1.0.1-alpha.15355.post.8+feature.post.1.g6b7c8d9 (post affects build context)

# Pre-release Controls
zerv flow --pre-release-label rc                  # Set pre-release type
# → 1.0.1-rc.10180.post.1+feature.pr.label.1.g3d4e5f6

zerv flow --pre-release-num 3                    # Set pre-release number
# → 1.0.1-alpha.3.post.1+feature.pr.num.1.g9a0b1c2

zerv flow --post-mode commit                     # Set distance calculation method
# → 1.0.1-alpha.17003.post.1+feature.post.mode.1.g1d2e3f4
```

<!-- Corresponding test: tests/integration_tests/flow/docs/override_controls.rs:test_individual_override_options -->

**Usage Examples**:

```bash
# VCS overrides
zerv flow --tag-version "v2.0.0" --distance 5 --bumped-branch "release/candidate"
# → 2.0.1-rc.71808.post.1+release.candidate.5.gb2c3d4e

# Version component overrides
zerv flow --major 2 --minor 5 --patch 3
# → 2.5.3

# Mixed overrides: VCS + version components
zerv flow --distance 3 --major 2 --minor 1
# → 2.1.1-alpha.60124.post.3+feature.test.3.g8f4e3a2

# Clean release with overrides
zerv flow --clean --major 2 --minor 0 --patch 0
# → 2.0.0+feature.clean.force.g8a9b0c1

# Complex multi-override scenario
zerv flow --tag-version "v1.5.0-rc.1" --bumped-commit-hash "f4a8b9c" --major 1 --minor 6
# → 1.6.0-alpha.11178.post.2+dev.branch.2.f4a8b9c
```

<!-- Corresponding test: tests/integration_tests/flow/docs/override_controls.rs:test_override_controls_documentation_examples -->

### zerv version: Manual control with 4 main capability areas

**Purpose**: Complete manual control over version generation with flexible schema variants and granular customization options.

**Note**: Unlike `zerv flow`, `zerv version` generates versions as-is without opinionated auto-bumping logic. It does not automatically increment post-counts based on commits or tags, nor does it derive pre-release labels and numbers from branch patterns. This is general-purpose version generation without opinionated logic.

#### Schema Variants: 20+ presets (standard, calver families) and custom RON schemas

**Purpose**: Choose from 20+ predefined version schemas or create custom RON-based schemas for complete format control.

**Schema Selection Examples**:

```bash
zerv version --schema standard-base
# → 1.0.0 (test case 1)

zerv version --schema standard-base-context
# → 1.0.0+branch.name.1.g4e9af24 (test case 2)

zerv version --schema standard-base-prerelease
# → 1.0.0-alpha.1 (test case 3)
```

<!-- Corresponding test: tests/integration_tests/version/docs/schema_variants.rs:test_zerv_version_schema_variants_documentation_examples -->
