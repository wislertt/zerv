# Zerv Flow CLI Command Design

**Status**: Planned
**Priority**: High
**Context**: Simplified CLI design for `zerv flow` command, mirroring `zerv version` structure with flow-specific pre-release management.

## Command Structure

```bash
zerv flow [OPTIONS]
```

**Single command only** - no subcommands.

## Core Arguments

### Output Options (same as zerv version)

```bash
-o, --output-format <FORMAT>     Output format [default: semver] [possible values: semver, pep440, zerv]
-t, --output-template <TEMPLATE> Custom output template using handlebars syntax
-p, --output-prefix <PREFIX>     Add prefix to output version
```

### Input Options (same as zerv version)

```bash
-s, --source <SOURCE>            Version source [default: git] [possible values: git, hg, bzr, svn]
-f, --input-format <FORMAT>      Input format [default: semver] [possible values: semver, pep440, calver, docker, raw]
-d, --directory <DIR>            Repository directory [default: current directory]
```

### Flow-Specific Options

```bash
-l, --pre-release-label <LABEL>  Pre-release label [default: auto-detect from branch] [possible values: alpha, beta, rc]
-n, --pre-release-num <NUM>      Pre-release number [default: hash from branch name]
      --branch-rules <RON>          Custom branch rules (RON format) [default: release-only]
      --bumped-branch <BRANCH>     Override branch name for pre-release resolution (same as zerv version)
      --bumped-branch-hash-length <LENGTH> Branch hash length for pre-release numbers [default: 5] [range: 4..16]
      --post-mode <TYPE>            Post calculation mode [default: tag] [possible values: tag, commit]
      --schema <SCHEMA>             Schema variant for output components [default: standard] [possible values: standard, standard-no-context, standard-context, standard-base, standard-base-prerelease, standard-base-prerelease-post, standard-base-prerelease-post-dev]
-v, --verbose                    Show verbose output including version resolution details
-h, --help                       Print help
-V, --version                    Print version
```

### Schema System

**zerv flow uses the flexible schema system from zerv version but restricted to standard schema family only.**

#### Available Standard Schema Variants

- **`standard`** (default) - Smart context: includes context only for dirty/distance states, excludes for clean tagged
- **`standard-no-context`** - Never includes build context (branch.commit info)
- **`standard-context`** - Always includes build context
- **`standard-base`** - Base version only (e.g., `1.2.3`)
- **`standard-base-prerelease`** - Base + prerelease (e.g., `1.2.3-alpha.1`)
- **`standard-base-prerelease-post`** - Base + prerelease + post (e.g., `1.2.3-alpha.1.post.2`)
- **`standard-base-prerelease-post-dev`** - Base + prerelease + post + dev (e.g., `1.2.3-alpha.1.post.2.dev.123`)

#### Schema Behavior Examples

**Smart Context (`standard` - default):**

- Clean tagged commit: `1.0.1-rc.1.post.1`
- Dirty working directory: `1.0.1-rc.1.post.1.dev.1729924622+feature.auth.1.a1b2c3d`
- Distance from tag: `1.0.1-rc.1.post.2+feature.auth.2.c2d3e4f`

**No Context (`standard-no-context`):**

- Any state: `1.0.1-rc.1.post.1` (never includes +branch.commit)

**Always Context (`standard-context`):**

- Any state: `1.0.1-rc.1.post.1+feature.auth.1.a1b2c3d` (always includes context)

**Base Components:**

- `standard-base`: `1.2.3`
- `standard-base-prerelease`: `1.2.3-alpha.1`
- `standard-base-prerelease-post`: `1.2.3-alpha.1.post.2`
- `standard-base-prerelease-post-dev`: `1.2.3-alpha.1.post.2.dev.123`

#### Schema Validation

**Only standard schema family supported in zerv flow:**

- ✅ **Valid**: `standard`, `standard-no-context`, `standard-context`, `standard-base`, `standard-base-prerelease`, `standard-base-prerelease-post`, `standard-base-prerelease-post-dev`
- ❌ **Invalid**: Any `calver*` schema variants will produce error
- ❌ **Invalid**: Any deprecated tier-based schemas will produce error

**Error handling:**

- Non-standard schemas will result in: `Error: zerv flow only supports standard schema variants, got: 'calver'`
- Invalid schema names will result in: `Error: Unknown schema variant: 'invalid-schema'`

## Pre-release Resolution Logic

### Default Behavior (no flags)

- **Label**: `alpha` with hash-based number from branch name (e.g., `feature/auth` → `alpha.12345`)
- **Number**: Hash derived from branch name using `--bumped-branch-hash-length`

### Post Distance Logic

**Configurable post distance calculation with two methods:**

#### Tag Distance (default)

- **`post`** increments when new tags are created on the branch
- **Reference point**: Last tag created on the branch
- **`post.0`**: Exactly when a new tag is created
- **Untagged commits**: Same `post` number, different `dev.timestamp`
- **Use case**: Release branches where you want to tag specific milestones

```
release/1 created → tag v1.0.1-rc.1.post.1
1 commit → v1.0.1-rc.1.post.1.dev.1729924622 (same post)
2 commits → v1.0.1-rc.1.post.1.dev.1729924623 (same post)
new tag → v1.0.1-rc.1.post.2 (post increments)
```

#### Commit Distance

- **`post`** counts commits since branch creation point
- **Reference point**: Branch creation from tag/branch
- **`post.0`**: Exactly when branch is created
- **All commits**: Increment `post` with each commit
- **Use case**: Development branches tracking total work done

```
develop created from v1.0.0 → 1.0.1-beta.1.post.0
1 commit → 1.0.1-beta.1.post.1
2 commits → 1.0.1-beta.1.post.2
```

**Control via `--post-mode` argument:**

- `--post-mode tag` (default): Tag-based post calculation
- `--post-mode commit`: Commit-based post calculation

### Branch Pattern Detection (`--branch-rules`)

**Configurable branch rules via RON string or use defaults:**

```bash
# Use custom branch rules
zerv flow --branch-rules "[(pattern: \"develop\", pre_release: \"beta\", number: \"1\", post_mode: \"commit\")]"

# Use default rules (no argument needed)
zerv flow --branch-rules
```

**Default rules (when no RON provided):**

```ron
[
    (pattern: "develop", pre_release: "beta", number: "1", post_mode: "commit"),
    (pattern: "release/*", pre_release: "rc", post_mode: "tag"),
]
```

**Branch rules can specify:**

- **pattern**: Branch name pattern to match
- **pre_release**: Pre-release type (alpha, beta, rc)
- **number**: Fixed number or hash-based
- **post_mode**: Tag or commit distance calculation

**Advanced RON configuration examples:**

```ron
(
    [
        (pattern: "develop", pre_release: "beta", number: "1", post_mode: "commit"),
        (pattern: "release/*", pre_release: "rc", post_mode: "tag"),
        (pattern: "feature/*", pre_release: "alpha", post_mode: "tag"),
        (pattern: "hotfix/*", pre_release: "alpha", post_mode: "tag"),
    ],
)
```

**Pattern matching rules:**

- **Exact match**: `develop` matches only `develop` branch
- **Wildcard match**: `release/*` matches branches starting with `release/`
- **Number extraction**: For wildcard patterns, tries to extract number:
    - `release/1` → extracts `1`
    - `release/1/xxxxx` → extracts `1`
    - `release/feature-name` (no number) → uses hash-based numbering

**Branch behaviors (not on tagged commits):**

- `main` → `1.1.0-alpha.1.post.2+main.2.a1b2c3d`
- `develop` → `1.0.1-beta.1.post.3+develop.3.c2d3e4f`
- `release/1` → `1.0.1-rc.1.post.1+release.1.1.e4f5g6h`
- `release/hotfix` → `1.0.1-rc.12345.post.1+release.hotfix.1.g5h6i7j`
- `feature/auth` → `1.0.1-alpha.54321.post.2+feature.auth.2.h6i7j8k`

**On tagged commits:** Clean versions (e.g., `1.0.0`, `1.0.1`)

**Branch name processing:**

- Slashes (`/`) converted to dots (`.`): `feature/auth` → `feature.auth`
- Hash generation uses simplified branch name: `feature.auth` → `12345`

### Manual Override

**Schema can be combined with manual pre-release overrides:**

```bash
# Force specific pre-release type and number with context
zerv flow --pre-release-label beta --pre-release-num 1 --schema standard-context

# Force rc for release-like branches, no context
zerv flow --pre-release-label rc --pre-release-num 2 --schema standard-base-prerelease-post

# Force alpha for feature branches with full context
zerv flow --pre-release-label alpha --schema standard-base-prerelease-post-dev-context

# Manual overrides with different schema levels
zerv flow --pre-release-label beta --schema standard-base-prerelease
zerv flow --pre-release-label rc --schema standard-base
```

### Branch Override

**Test different branch scenarios without switching branches:**

```bash
# Test develop branch with different schemas
zerv flow --bumped-branch develop --schema standard
zerv flow --bumped-branch develop --schema standard-no-context

# Test release branch with specific schema
zerv flow --bumped-branch release/1 --schema standard-base-prerelease-post-context

# Test feature branch scenarios
zerv flow --bumped-branch feature/auth --schema standard-base-prerelease-post-dev
```

## Output Modes

### Full Output (default - `standard` schema)

```
1.0.1-alpha.12345.post.2.dev.1729924622+feature.auth.2.a1b2c3d
```

### Pre-release Output (`--schema standard-base-prerelease-post`)

```
1.0.1-alpha.12345.post.2
```

### Base-Only Output (`--schema standard-base`)

```
1.0.1
```

### On Reference Point (post.0)

```
1.0.1-rc.1.post.0 (exactly on branch point/tag)
```

## Format Variations

### SemVer (default)

```
1.0.1-alpha.12345.post.1.dev.1729924622+feature.auth.1.a1b2c3d
```

### PEP440

```
1.0.1a12345.post1.dev1729924622
```

### Zerv (RON format)

```
<zerv ron>
```

## Usage Examples

### Basic Usage

```bash
# Generate flow version with smart context (default schema)
zerv flow

# Force specific pre-release type
zerv flow --pre-release-label beta

# Include pre-release/post-release but no build context
zerv flow --schema standard-base-prerelease-post

# Base version only
zerv flow --schema standard-base

# Never include build context
zerv flow --schema standard-no-context

# Always include build context
zerv flow --schema standard-context
```

### Advanced Usage

```bash
# Complete control over pre-release with schema
zerv flow --bumped-branch release/1 --schema standard-base-prerelease-post-context

# Custom template output with specific schema
zerv flow --schema standard-base --output-template "v{{version}}-{{pre_release}}"

# Different repository directory
zerv flow --directory ../other-repo --schema standard

# Verbose output
zerv flow --verbose --schema standard-base-prerelease-post-dev

# Error case - this will fail with calver schema
zerv flow --schema calver  # Error: zerv flow only supports standard schema variants
```

## Future Configuration

**RON configuration files (not in initial implementation):**

```ron
# .zerv.ron
(
    branch_patterns: [
        (pattern: "develop", pre_release: "beta", number: "1"),
        (pattern: "release", pre_release: "rc"),
        (pattern: "feature", pre_release: "alpha"),
        (pattern: "hotfix", pre_release: "alpha"),
    ],
)
```

**Usage:**

```bash
zerv flow --config .zerv.ron
```

## Key Design Principles

1. **Mirror zerv version**: Same output/input options structure with shared schema system
2. **Intelligent defaults**: Smart branch-based pre-release detection with smart context schema
3. **Flexible overrides**: Manual control when needed, including schema selection
4. **Honest versioning**: Never hides Git state, always accurate (unless explicitly requested via schema)
5. **Schema-based flexibility**: Single `--schema` argument replaces multiple context/control flags

---

**Next Steps**: Implement basic `zerv flow` command structure and integrate with existing `zerv version` functions.
