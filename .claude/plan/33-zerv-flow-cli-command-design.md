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
      --build-context              Include build context (+branch.commit) in output [default: true]
      --no-build-context            Exclude build context from output
      --dev-ts                     Include dev timestamp for dirty working directory [default: auto-detect]
      --no-dev-ts                  Exclude dev timestamp from output
      --with-pre-release           Include pre-release/post-release but no build context
      --base-only                  Base version only (major.minor.patch)
-v, --verbose                    Show verbose output including version resolution details
-h, --help                       Print help
-V, --version                    Print version
```

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

**Mutually exclusive with `--pre-release-from-branch`:**

```bash
# Force specific pre-release type and number
zerv flow --pre-release-label beta --pre-release-num 1

# Force rc for release-like branches
zerv flow --pre-release-label rc --pre-release-num 2

# Force alpha for feature branches (uses hash by default)
zerv flow --pre-release-label alpha
```

### Branch Override

**Test different branch scenarios without switching branches:**

```bash
zerv flow --bumped-branch develop --pre-release-from-branch
zerv flow --bumped-branch release/1 --pre-release-from-branch
```

## Output Modes

### Full Output (default)

```
1.0.1-alpha.12345.post.2.dev.1729924622+feature.auth.2.a1b2c3d
```

### Pre-release Output (`--with-pre-release`)

```
1.0.1-alpha.12345.post.2
```

### Base-Only Output (`--base-only`)

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
# Generate flow version with automatic pre-release
zerv flow

# Enable branch pattern detection (GitFlow)
zerv flow --pre-release-from-branch

# Force specific pre-release type
zerv flow --pre-release-label beta

# Include pre-release/post-release but no build context
zerv flow --with-pre-release

# Base version only
zerv flow --base-only
```

### Advanced Usage

```bash
# Complete control over pre-release
zerv flow --bumped-branch release/1 --pre-release-from-branch

# Custom template output
zerv flow --output-template "v{{version}}-{{pre_release}}"

# Different repository directory
zerv flow --directory ../other-repo

# Verbose output
zerv flow --verbose
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

1. **Mirror zerv version**: Same output/input options structure
2. **Intelligent defaults**: Smart branch-based pre-release detection
3. **Flexible overrides**: Manual control when needed
4. **Honest versioning**: Never hides Git state, always accurate
5. **Clean alternatives**: `--with-pre-release` and `--base-only` for simplified output

---

**Next Steps**: Implement basic `zerv flow` command structure and integrate with existing `zerv version` functions.
