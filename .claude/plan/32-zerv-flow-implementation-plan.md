# Zerv Flow

This document defines the planned `zerv flow` subcommand, an opinionated automation layer that builds on Zerv's existing `zerv version` functionality. While `zerv version` provides flexible configuration options, `zerv flow` adds intelligent branching strategies and automated version progression that make complete semantic versioning effortless. `zerv flow` eliminates manual version management decisions across all semantic components (major.minor.patch, pre-releases, build metadata), transforming any Git state into meaningful versions across multiple formats for seamless CI/CD workflows.

## Architecture

**Zerv Flow**: Automates intelligent semantic versioning decisions for any Git state, enabling continuous CI/CD workflows without manual version management.

### Core Principles

1. **Semantic state capture** - Extract semantic meaning from ANY Git state (any branch, any commit, uncommitted changes)
2. **Multi-format output** - Transform semantic meaning into various version formats (SemVer, PEP440, Docker SemVer, etc.) with customizable format support for different ecosystems
3. **Seamless semantic release integration** - Work seamlessly with semantic release tools that manage major.minor.patch, while Zerv provides fully automated pre-release versioning with no-brainer intuitive bumping, enabling seamless CI/CD throughout the entire SDLC
4. **Build traceability** - Include sufficient context to trace versions back to exact Git states

### Version Format Explained

**Full Example**: `1.0.1-alpha.12345.post.3.dev.1729924622+feature.auth.1.f4a8b9c`

**Structure**: `<BASE>-<PRE_RELEASE>.<POST>[.<DEV>][+BUILD_CONTEXT]`

- **`1.0.1`** - Base version (patch bump from `v1.0.0`)
- **`alpha.12345`** - Pre-release type and branch identification (alpha + hash)
- **`post.1`** - Commit distance from branch point
- **[.dev.timestamp]** - Optional dev timestamp for dirty state
- **[+BUILD_CONTEXT]** - Optional build context for traceability
    - Format: `+branch.name[.distance].commit-hash`
    - Distance shown only when distance != post distance

**Key Point**: The core version `<BASE>-<PRE_RELEASE>.<POST>[.<DEV>]` contains all semantic meaning needed to understand Git state. The build context `[+BUILD_CONTEXT]` is optional and provides additional verbose information for easier interpretation and traceability.

**Version Variations**:

- **Tagged release**: `1.0.1`
- **Tagged pre-release**: `2.0.1-rc.1.post.2`
- **Branch from Tagged release**: `1.0.1-alpha.54321.post.1+feature.login.1.f4a8b9c`
- **Branch from Tagged pre-release**: `2.0.1-alpha.98765.post.3+fix.auth.bug.1.c9d8e7f`
- **Uncommitted changes**: `2.0.1-alpha.98765.post.4.dev.1729924622+fix.auth.bug.1.c9d8e7f`

### Pre-release Resolution Strategy

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

## Examples

This section demonstrates how Zerv Flow works across different branching strategies and Git scenarios.

**Note**: To keep diagrams clean and readable, build context is omitted from version strings in the examples. Dirty state (`.dev.timestamp`) is shown in diagrams when applicable.

**Example**: A commit appears as `1.0.1-alpha.12345.post.3.dev.1729924622` in the diagrams. With build context enabled: `1.0.1-alpha.12345.post.3.dev.1729924622+feature.user-auth.3.a1b2c3d`

### Trunk-Based Development

**Purpose**: Shows Zerv Flow handling a complex trunk-based workflow with parallel feature development, branch synchronization, and nested feature branches.

**Scenario Overview**:

- Development starts from `v1.0.0` on main
- Two feature branches created in parallel: `feature-1` and `feature-2`
- `feature-1` gets completed and released first (`v1.0.1`)
- `feature-2` syncs with main to get `feature-1` changes, then continues development
- `feature-3` branches from `feature-2` to implement a sub-feature (nested feature branch)
- `feature-3` merges back to `feature-2`, which then completes development and releases as `v1.1.0`

**Key Zerv Flow behaviors demonstrated**:

- **Uncommitted changes**: Shows dirty state with `.dev.timestamp` suffix
- **Parallel development**: Different branches get unique hash-based IDs (`68031`, `42954`, `14698`)
- **Version progression**: Base version updates when syncing with main (`1.0.1` → `1.0.2`)
- **Post-release distance continuity**: Distance counters continue accumulating across branches and merges
- **Nested feature branches**: `feature-3` branching from `feature-2` with independent versioning
- **Merge handling**: Clean version progression through complex merge scenarios
- **Alpha pre-releases**: All development branches use `alpha` pre-release identifiers

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

### GitFlow Branching Strategy

**Purpose**: Shows Zerv Flow handling GitFlow methodology with proper pre-release type mapping and merge patterns.

**Scenario Overview**:

- Main branch has `v1.0.0` while develop branch has progressed to `1.0.1-beta.1.post.1`
- Feature branch `feature/auth` develops authentication functionality from develop
- Hotfix branch `hotfix/critical` addresses emergency issue from main production
- Release branch `release/1` prepares the next release from develop
- GitFlow merge patterns demonstrate proper version progression through the workflow

**Key Zerv Flow behaviors demonstrated**:

- **Beta pre-releases**: Develop branch uses `beta` identifier for integration builds
- **Alpha pre-releases**: Feature branches use `alpha` with hash-based identification
- **RC pre-releases**: Release branches use `rc` identifier for release candidates
- **Clean releases**: Main branch maintains clean versions without pre-release suffixes
- **Hotfix emergency flow**: Critical fixes from main with proper version propagation
- **Release branch post mode**: `release/*` branches use post distance from release tag (commit distance)
- **Trunk-based post mode**: Other branches use post distance from branch point (for parallel development)
- **Base version propagation**: Version bumps when syncing branches with newer main releases

```mermaid
---
config:
  logLevel: 'debug'
  theme: 'base'
---
gitGraph
    %% Initial state: main and develop branches
    commit id: "1.0.0"

    branch develop order: 3
    checkout develop
    commit id: "1.0.1-beta.1.post.1"

    %% Feature development from develop branch (trunk-based post mode)
    branch feature/auth order: 4
    checkout feature/auth
    commit id: "1.0.1-alpha.12345.post.1"
    commit id: "1.0.1-alpha.12345.post.2"

    checkout develop
    merge feature/auth id: "1.0.1-beta.1.post.2" tag: "feature merged"

    %% Hotfix emergency flow from main
    checkout main
    branch hotfix/critical order: 1
    checkout hotfix/critical
    commit id: "1.0.1-alpha.54321.post.1"

    checkout main
    merge hotfix/critical id: "1.0.1" tag: "hotfix released"

    %% Sync develop with main changes and continue development
    checkout develop
    merge main id: "1.0.2-beta.1.post.3" tag: "sync main"
    commit id: "1.0.2-beta.1.post.4"

    %% Release branch preparation (release/* uses commit distance from tag)
    branch release/1 order: 2
    checkout release/1
    commit id: "1.0.2-rc.1.post.1" tag: "tagged"
    commit id: "1.0.2-rc.1.post.2" tag: "tagged"
    commit type:REVERSE id: "1.0.2-rc.1.post.2.dev.{timestamp}" tag: "untagged"

    checkout main
    merge release/1 id: "1.1.0" tag: "release 1.1.0"

    %% Sync develop with release and prepare for next cycle
    checkout develop
    merge main id: "1.1.1-beta.1.post.1" tag: "sync release"
```

## Scope and Limitations

- **Scope**: Git state → semantic version mapping
- **Out of Scope**: Git operations (handled by Git) and version usage (Docker tags, package releases, CI/CD deployment)
- **Known Limitations**: Hash collisions possible by design. Users can use longer hash lengths or distributed sequence numbering for zero collisions (requires external coordination)
