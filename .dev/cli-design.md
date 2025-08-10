# Zerv CLI Design

**Command**: `zerv`
**Purpose**: Dynamic version generation from VCS tags

## Basic Usage

### Default Behavior

```bash
# Auto-detect VCS and output current version
zerv
# Output: 1.2.3

# Same as above but explicit
zerv from any
# Output: 1.2.3
```

## Core Commands

### 1. Version Generation

```bash
# Auto-detect VCS
zerv                           # 1.2.3
zerv from any                  # 1.2.3

# Specific VCS
zerv from git                  # 1.2.3
zerv from mercurial           # 1.2.3
zerv from darcs               # 1.2.3
zerv from subversion          # 1.2.3

# Parse from stdin/string
zerv from stdin "1.2.3+g29045e8"     # Direct argument
echo "1.2.3" | zerv from stdin       # From pipeline
cat version.txt | zerv from stdin --format semver  # Format conversion
```

#### Formatting System

##### Version Object Methods

```bash
# Use version object methods (serialize as-is)
zerv --format-template "{{ version.serialize_pep440() }}"     # 1.2.3.post7.dev0+g29045e8
zerv --format-template "{{ version.serialize_semver() }}"      # 1.2.3-post.7+g29045e8
zerv --format-template "{{ version.serialize_pvp() }}"         # 1.2.3-post-7-g29045e8

# Override specific fields
zerv --format-template "{{ version.serialize_pep440(stage='rc') }}"     # 1.2.3rc7.post7.dev0+g29045e8
zerv --format-template "{{ version.serialize_pep440(metadata=false) }}" # 1.2.3.post7.dev0
zerv --format-template "{{ version.serialize_semver(pre=['alpha']) }}"  # 1.2.3-alpha+g29045e8
```

##### Standalone Serialization Functions

```bash
# Build version from scratch using individual variables
zerv --format-template "{{ serialize_pep440(base) }}"                    # 1.2.3
zerv --format-template "{{ serialize_pep440(major + '.' + minor + '.' + (patch + 1)) }}" # 1.2.4
zerv --format-template "{{ serialize_semver(major + '.' + minor + '.' + patch, pre=['dev', distance]) }}" # 1.2.3-dev.7
```

#### Zerv's Opinionated Versioning

Zerv implements a logical three-tier versioning system based on repository state:

##### Tier 1: Clean Tagged Commit

**State**: On a tagged commit with clean working directory
**Variables**: Only tag-derived values
**Example**:

```bash
# Repository state: On tag v1.2.3, clean working directory
zerv --format zerv-pep440    # 1.2.3
zerv --format zerv-semver    # 1.2.3
zerv --format zerv-pvp       # 1.2.3

# With pre-release tag v1.2.3-rc.1
zerv --format zerv-pep440    # 1.2.3rc1
zerv --format zerv-semver    # 1.2.3-rc.1
zerv --format zerv-pvp       # 1.2.3-rc-1
```

##### Tier 2: Distance from Tag

**State**: Clean working directory, commits after last tag
**Variables**: Adds post-release info with distance and build metadata
**Example**:

```bash
# Repository state: 7 commits after v1.2.3, branch=feature/auth, clean
zerv --format zerv-pep440    # 1.2.3.post7+feature.auth.7.g29045e8
zerv --format zerv-semver    # 1.2.3-post.7+feature.auth.7.g29045e8
zerv --format zerv-pvp       # 1.2.3-post-7-feature-auth-7-g29045e8

# With tag v1.2.3.post5, distance=7 → zerv_post=12
zerv --format zerv-pep440    # 1.2.3.post12+feature.auth.7.g29045e8
```

##### Tier 3: Dirty Working Directory

**State**: Uncommitted changes present
**Variables**: Adds dev timestamp for work-in-progress indication
**Example**:

```bash
# Repository state: 7 commits after v1.2.3, branch=feature/auth, dirty
zerv --format zerv-pep440    # 1.2.3.post7.dev20231215142530+feature.auth.7.g29045e8
zerv --format zerv-semver    # 1.2.3-post.7.dev.20231215142530+feature.auth.7.g29045e8
zerv --format zerv-pvp       # 1.2.3-post-7-dev-20231215142530-feature-auth-7-g29045e8
```

**Key Benefits**:

- **Progressive detail**: More context as you move away from releases
- **Clean releases**: Tagged commits produce clean version strings
- **CI-friendly**: Build metadata helps identify specific builds
- **Developer-friendly**: Dirty flag clearly indicates work-in-progress

##### Available Template Variables

**Standard Variables:**

- `{{ version }}` - Version object with methods (serialize_pep440(), serialize_semver(), etc.)

**Tag Variables:**

- `{{ base }}` - Base version (1.2.3)
- `{{ major }}` - Major version (1)
- `{{ minor }}` - Minor version (2)
- `{{ patch }}` - Patch version (3)
- `{{ stage }}` - Pre-release stage (alpha, beta, rc)
- `{{ revision }}` - Pre-release revision (1, 2, 3)
- `{{ post }}` - Post from Tag
- `{{ dev }}` - Dev from Tag
- `{{ epoch }}` - PEP 440 epoch
- `{{ tagged_metadata }}` - Metadata from tag

**Context Variables:**

- `{{ distance }}` - Commits since last tag
- `{{ commit }}` - Commit hash (short)
- `{{ dirty }}` - "dirty" or "clean"
- `{{ branch }}` - Current branch name
- `{{ escaped_branch }}` - Branch name with special chars as dots (feature/auth → feature.auth)
- `{{ pvp_branch }}` - Branch name with special chars as dashes (feature/auth → feature-auth)
- `{{ timestamp }}` - Commit timestamp (YYYYmmddHHMMSS)

**Zerv Variables:**

- `{{ zerv_post }}` - post + distance
- `{{ zerv_timestamp }}` - timestamp if dirty else None
- `{{ zerv_pre }}` - Pre-release components [stage, revision, 'post', zerv_post, 'dev', timestamp] if dirty else [stage, revision, 'post', zerv_post]
- `{{ zerv_branch }}` - escaped_branch if distance > 0 else None (uses dots)
- `{{ zerv_branch_pvp }}` - pvp_branch if distance > 0 else None (uses dashes)
- `{{ zerv_distance }}` - distance if distance > 0 else None
- `{{ zerv_commit_hash }}` - commit_hash if distance > 0 else None
- `{{ zerv_build }}` - Build metadata [zerv_branch, zerv_distance, zerv_commit_hash] (for PEP 440/SemVer)
- `{{ zerv_build_pvp }}` - Build metadata [zerv_branch_pvp, zerv_distance, zerv_commit_hash] (for PVP)

##### Built-in Format Presets

```bash
# Standard formats using correct dunamai function signatures

# Zerv formats (smart defaults with conditional logic)
zerv --format zerv-pep440      # {{ serialize_pep440(base, stage=stage, revision=revision, post=zerv_post, dev=zerv_timestamp, metadata=zerv_build) }}
zerv --format zerv-semver      # {{ serialize_semver(base, pre=zerv_pre, metadata=zerv_build) }}
zerv --format zerv-pvp         # {{ serialize_pvp(base, metadata=zerv_pre + zerv_build_pvp) }}
zerv --format zerv-git         # v{{ serialize_semver(base, pre=zerv_pre, metadata=zerv_build) }}
zerv --format zerv-docker      # {{ serialize_pvp(base, metadata=zerv_pre + zerv_build_pvp) }}

# Simple formats
zerv --format base            # {{ base }}
zerv --format major-minor     # {{ major }}.{{ minor }}
zerv --format major-minor-patch # {{ major }}.{{ minor }}.{{ patch }}
zerv --format snapshot        # {{ base }}-SNAPSHOT
zerv --format docker          # {{ base }}-{{ commit }}
```

##### Direct Template Strings

```bash
# Provide template string directly (Jinja-like syntax)
zerv --format-template "v{{ base }}"                    # v1.2.3
zerv --format-template "{{ base }}+{{ distance }}.{{ commit }}" # 1.2.3+7.g29045e8
zerv --format-template "v{{ major }}.{{ minor }}.{{ patch }}"   # v1.2.3
zerv --format-template "{{ version.serialize_pep440() }}"       # 1.2.3.post7.dev0+g29045e8
```

##### Available Serialization Methods

**Version Object Methods** (use current version data):

- `{{ version.serialize_pep440() }}` - Serialize current version as PEP 440
- `{{ version.serialize_semver() }}` - Serialize current version as SemVer
- `{{ version.serialize_pvp() }}` - Serialize current version as PVP
- `{{ version.serialize_pep440(stage='rc') }}` - Override stage to 'rc'
- `{{ version.serialize_pep440(metadata=false) }}` - Remove metadata

**Standalone Functions** (build from scratch):

- `{{ serialize_pep440(base) }}` - Build PEP 440 from base version
- `{{ serialize_semver(base, pre=['alpha', revision]) }}` - Build SemVer with prerelease
- `{{ serialize_pvp(major + '.' + minor + '.' + patch) }}` - Build PVP from components

#### Version Bumping

##### Bump Types

- Bump base

```bash
zerv --bump                    # Auto-bump based on distance
zerv --bump major             # 1.2.3 → 2.0.0
zerv --bump minor             # 1.2.3 → 1.3.0
zerv --bump patch             # 1.2.3 → 1.2.4
```

- Bump stage

```bash
# Only bump if distance > 0 or dirty, bump stage / revision
zerv --bump alpha    # 1.2.3 → 1.2.4-alpha.1
zerv --bump beta    # 1.2.3 → 1.2.4-beta.1
zerv --bump rc       # 1.2.3 → 1.2.4-rc.1
```

#### Pattern Matching

```bash
# Default behavior - auto-detects common patterns
zerv                                      # Tries v1.2.3, 1.2.3, release-v1.2.3, etc.

# Custom regex when needed
zerv --pattern "(?P<base>\d+\.\d+\.\d+)"                    # Simple version
zerv --pattern "^myapp-v(?P<base>\d+\.\d+\.\d+)"           # With prefix
zerv --pattern "^release-(?P<base>\d+\.\d+\.\d+)-final$"   # Complex pattern
```

#### Override State

##### Version Components

```bash
zerv --major 2              # Override major version
zerv --minor 5              # Override minor version
zerv --patch 1              # Override patch version
zerv --base 2.5.1           # Override entire base version
zerv --stage alpha          # Override pre-release stage
zerv --revision 3           # Override pre-release revision
zerv --distance 10          # Override commit distance
```

##### Repository State

```bash
zerv --branch main          # Override branch name
zerv --dirty                # Force dirty state
zerv --clean                # Force clean state
zerv --commit abc123        # Override commit hash
zerv --timestamp 20231215142530  # Override timestamp
```

##### Metadata Control

```bash
zerv --metadata             # Always include commit hash
zerv --no-metadata          # Never include commit hash
zerv --full-commit          # Use full commit hash
zerv --commit-length 8      # Use 8 characters of commit hash
zerv --commit-prefix g      # Prefix commit with 'g' (g29045e8)
```

#### VCS-Specific Options

##### Git Options

```bash
zerv from git --tag-branch main          # Find tags on main branch
zerv from git --ignore-untracked         # Ignore untracked files for dirty check
```

##### Subversion Options

```bash
zerv from subversion --tag-dir releases  # Look for tags in releases/ directory
```

#### Output Control

```bash
zerv                        # Default: only output version
zerv --verbose              # Show detailed information
```

**Verbose output includes:**

- VCS detected: `git`
- Latest tag found: `v1.2.3` (7 commits ago)
- Pattern matched: `^v(?P<base>\d+\.\d+\.\d+)`
- Branch: `feature/auth`
- Dirty state: `clean`
- Version components: `base=1.2.3, distance=7, commit=g29045e8`
- Final version: `1.2.3.post7.dev0+g29045e8`

### 2. Version Validation

```bash
# Check against all styles (default)
zerv check 1.2.3
# Output:
# PEP 440: ✓ Valid
# SemVer:  ✓ Valid
# PVP:     ✓ Valid

zerv check v1.2.3-alpha.1
# Output:
# PEP 440: ✗ Invalid (prefix 'v' not allowed)
# SemVer:  ✗ Invalid (prefix 'v' not allowed)
# PVP:     ✗ Invalid (prefix 'v' not allowed)

# Check against specific style
zerv check --style pep440 1.2.3     # ✓ Valid PEP 440
zerv check --style semver 1.2.3      # ✓ Valid SemVer
zerv check --style pvp 1.2.3         # ✓ Valid PVP
```

## Configuration System

### Config File Locations (Priority Order)

1. **Project level**: `./zerv.toml` (highest priority)
2. **User level**: `~/.config/zerv.toml`

### Config File Format

```toml
# ~/.config/zerv.toml or ./zerv.toml
[default]
style = "pep440"
strict = false

# User-defined format presets (extend built-in ones)
[formats]
my-release = "v{{ base }}"
my-docker = "v{{ base }}-{{ commit }}"
my-npm = "{{ major }}.{{ minor }}.{{ patch }}"
rc-version = "{{ version.serialize_pep440(stage='rc') }}"
hotfix = "{{ base }}.{{ distance }}"

# VCS-specific settings
[git]
ignore_untracked = true
tag_branch = "main"
commit_length = 7

[subversion]
tag_dir = "tags"
```

### Environment Variables

```bash
ZERV_CONFIG=/path/to/config.toml zerv     # Use specific config file
```

## Examples

### CI/CD Usage

```bash
# Get clean version for tagging
zerv --format base
# Output: 1.2.3

# Get development version for builds
zerv --format pep440-dev
# Output: 1.2.3.post7.dev0+g29045e8

# CI-specific format
zerv --format short
# Output: 1.2.3-7
```

### Package.json Integration

```bash
# Update package.json version
npm version $(zerv --format major-minor-patch)
# Uses format: "{{ major }}.{{ minor }}.{{ patch }}"

# Or with direct template
npm version $(zerv --format-template "{{ major }}.{{ minor }}.{{ patch }}")
```

### Docker Tagging

```bash
# Production tag
docker build -t myapp:$(zerv --format base) .
# Output: myapp:1.2.3

# Development tag
docker build -t myapp:$(zerv --format docker) .
# Output: myapp:1.2.3-g29045e8

# Custom format
docker build -t myapp:$(zerv --format-template "{{ base }}-{{ commit }}") .
# Output: myapp:1.2.3-g29045e8
```

## Help System

```bash
zerv --help                 # General help
zerv from --help           # Help for 'from' command
zerv check --help          # Help for 'check' command
zerv --version             # Show zerv version
```

## Exit Codes

- `0` - Success
- `1` - Error (any failure)
