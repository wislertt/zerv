# Zerv Version Command - Ideal Specification

## Overview

The `zerv version` command generates version strings from version control system data using configurable schemas. It supports multiple input sources, output formats, and provides powerful templating capabilities for dynamic version generation.

## Command Syntax

```bash
zerv version [OPTIONS]
```

## Argument Categories

Arguments are organized into logical groups for better usability:

### Input-Related Options

- `--source <SOURCE>` - Version source
    - `git` (default) - Auto-detect Git repository
    - `stdin` - Read from stdin (Zerv RON format only)
- `--input-format <FORMAT>` - Input format for parsing version strings
    - `auto` (default) - Auto-detect format
    - `semver` - Semantic Versioning format
    - `pep440` - Python PEP440 format
- `--schema <SCHEMA>` - Version schema to apply
    - `zerv-standard` (default) - Standard semantic versioning
    - `zerv-calver` - Calendar versioning
- `--schema-ron <RON>` - Custom RON schema definition

### Override/Modification Options

#### Override Options (Absolute Values)

- `--tag-version <TAG>` - Override detected tag version
- `--distance <NUM>` - Override distance from tag
- `--dirty` - Override dirty state to true
- `--no-dirty` - Override dirty state to false
- `--clean` - Override to clean state (distance=0, dirty=false)
- `--current-branch <BRANCH>` - Override branch name
- `--commit-hash <HASH>` - Override commit hash
- `--post <NUM>` - Override post number
- `--dev <NUM>` - Override dev number
- `--pre-release-label <LABEL>` - Override prerelease label
- `--pre-release-num <NUM>` - Override prerelease number
- `--epoch <NUM>` - Override epoch number
- `--custom <JSON>` - Override custom variables in JSON format

#### Bump Options (Relative Modifications)

**Version Bumps (Semantic Versioning Rules):**

- `--bump-major [<NUM>]` - Increment major version, reset minor and patch to 0 (default: 1)
    - `1.2.3` → `2.0.0` (with `--bump-major`)
    - `1.2.3` → `3.0.0` (with `--bump-major 2`)
- `--bump-minor [<NUM>]` - Increment minor version, reset patch to 0 (default: 1)
    - `1.2.3` → `1.3.0` (with `--bump-minor`)
    - `1.2.3` → `1.5.0` (with `--bump-minor 3`)
- `--bump-patch [<NUM>]` - Increment patch version only (default: 1)
    - `1.2.3` → `1.2.4` (with `--bump-patch`)
    - `1.2.3` → `1.2.6` (with `--bump-patch 3`)

**Pre-release Bumps:**

- `--bump-pre-release-num [<NUM>]` - Add to prerelease number, creates alpha label if none exists (default: 1)
- `--bump-pre-release-label <LABEL>` - Change prerelease label, reset number to 0

**Other Bumps (Additive Only):**

- `--bump-post [<NUM>]` - Add to post number (default: 1)
- `--bump-dev [<NUM>]` - Add to dev number (default: 1)
- `--bump-epoch [<NUM>]` - Add to epoch number, resets all lower precedence components (default: 1)

##### Context Control Options

- `--bump-context` - Include VCS context qualifiers (default behavior)
- `--no-bump-context` - Pure tag version, no VCS context

### Output-Related Options

- `--output-format <FORMAT>` - Target output format
    - `semver` (default) - Semantic Versioning format
    - `pep440` - Python PEP440 format
    - `zerv` - Zerv RON format (for debugging/piping)
- `--output-template <TEMPLATE>` - Custom template string (Handlebars templating)
- `--output-prefix [PREFIX]` - Add prefix (defaults to "v")
- `-C <DIR>` - Change to directory before running command

## State-Based Versioning Tiers

### Tier 1: Clean (distance = 0, dirty = false)

```bash
$ zerv version  # On tagged commit, clean working tree
1.2.3
```

### Tier 2: Distance (distance > 0, dirty = false)

```bash
$ zerv version  # 5 commits ahead of tag, clean working tree
1.2.3.post5+main.abc1234
```

### Tier 3: Dirty (dirty = true)

```bash
$ zerv version  # Uncommitted changes
1.2.3.dev20241201123045+main.abc1234
```

## Available Schemas

### zerv-standard (Default)

- **Tier 1** (Clean): `major.minor.patch`
- **Tier 2** (Distance): `major.minor.patch.post<post>+branch.<commit>`
- **Tier 3** (Dirty): `major.minor.patch.dev<timestamp>+branch.<distance>.<commit>`

### zerv-calver

- **Tier 1** (Clean): `YYYY.MM.DD.PATCH`
- **Tier 2** (Distance): `YYYY.MM.DD.PATCH.post<post>+branch.<commit>`
- **Tier 3** (Dirty): `YYYY.MM.DD.PATCH.dev<timestamp>+branch.<distance>.<commit>`

## Semantic Versioning Bump Behavior

**Critical Design Principle**: Version bumps follow standard semantic versioning rules where higher-level bumps reset lower-level components to zero.

### Current vs Ideal Behavior

**Current Implementation Issues (Additive Only):**

- `1.2.3` + `--bump-major` → `2.2.3` ❌ (Wrong! Should reset minor/patch)
- `1.2.3` + `--bump-minor` → `1.3.3` ❌ (Wrong! Should reset patch)
- `1.2.3` + `--bump-patch` → `1.2.4` ✅ (Correct)

**Ideal Behavior (Semantic Versioning Compliant):**

- `1.2.3` + `--bump-major` → `2.0.0` ✅ (Resets minor/patch to 0)
- `1.2.3` + `--bump-minor` → `1.3.0` ✅ (Resets patch to 0)
- `1.2.3` + `--bump-patch` → `1.2.4` ✅ (No reset needed)

### Bump Hierarchy and Reset Rules

1. **Major Bump** (`--bump-major`):
    - Increments major version
    - **Resets minor and patch to 0**
    - Example: `1.2.3` → `2.0.0`

2. **Minor Bump** (`--bump-minor`):
    - Increments minor version
    - **Resets patch to 0**
    - Example: `1.2.3` → `1.3.0`

3. **Patch Bump** (`--bump-patch`):
    - Increments patch version only
    - Example: `1.2.3` → `1.2.4`

### Multiple Bump Interactions

When multiple bumps are specified, higher precedence bumps reset lower precedence components, then explicitly specified components are bumped from 0:

```bash
# Major bump resets minor and patch, then minor is bumped from 0
zerv version --bump-major --bump-minor 2 --bump-patch 3
# 1.2.3 → 2.2.3 (major to 2, minor from 0 to 2, patch from 0 to 3)

# Minor bump resets patch, then patch is bumped from 0
zerv version --bump-minor --bump-patch 5
# 1.2.3 → 1.3.5 (minor to 3, patch from 0 to 5)

# Only patch bump
zerv version --bump-patch 2
# 1.2.3 → 1.2.5
```

### Component Precedence Hierarchy

**Precedence Order**: Epoch → Major → Minor → Patch → Pre-release → Post → Dev

- **Epoch bumps**: Reset all lower precedence components
- **Version bumps**: Reset lower precedence version components
- **Pre-release bumps**: Reset post/dev components
- **Post/Dev bumps**: Additive only (no reset behavior)

### Pre-release Bump Behavior

**Detailed Pre-release Examples:**

```bash
# Pre-release number bumps
1.2.3-alpha.1 + --bump-pre-release-num 2 → 1.2.3-alpha.3 ✅
1.2.3-beta.5 + --bump-pre-release-num → 1.2.3-beta.6 ✅ (default increment: 1)
1.2.3 + --bump-pre-release-num 2 → 1.2.3-alpha.2 ✅ (creates alpha label)

# Pre-release label overrides (preserve number)
1.2.3-alpha.1 + --pre-release-label beta → 1.2.3-beta.1 ✅
1.2.3-beta.5 + --pre-release-label rc → 1.2.3-rc.5 ✅
1.2.3 + --pre-release-label alpha → 1.2.3-alpha.0 ✅ (creates if doesn't exist)

# Pre-release label bumps (reset number to 0)
1.2.3-alpha.5 + --bump-pre-release-label beta → 1.2.3-beta.0 ✅
1.2.3-beta.3 + --bump-pre-release-label rc → 1.2.3-rc.0 ✅
1.2.3 + --bump-pre-release-label alpha → 1.2.3-alpha.0 ✅ (creates with reset)

# Pre-release with post/dev components (reset behavior)
1.2.3-alpha.1.post2.dev5 + --bump-pre-release-num 2 → 1.2.3-alpha.3 ✅ (resets post/dev)
1.2.3-alpha.1.post2.dev5 + --pre-release-label beta → 1.2.3-beta.1.post2.dev5 ✅ (preserves post/dev)
1.2.3-alpha.1.post2.dev5 + --bump-pre-release-label rc → 1.2.3-rc.0 ✅ (resets post/dev)
1.2.3.post2.dev5 + --pre-release-label alpha → 1.2.3-alpha.0.post2.dev5 ✅
1.2.3.post2.dev5 + --bump-pre-release-label beta → 1.2.3-beta.0 ✅ (resets post/dev)

# Combined pre-release operations
1.2.3-alpha.1 + --pre-release-label beta --bump-pre-release-num 2 → 1.2.3-beta.3 ✅
1.2.3-beta.5 + --pre-release-label rc --bump-pre-release-num 1 → 1.2.3-rc.6 ✅
1.2.3 + --pre-release-label alpha --bump-pre-release-num 3 → 1.2.3-alpha.3 ✅ (creates with bump)

# Pre-release with post/dev bumps
1.2.3-alpha.1.post2.dev5 + --bump-post 1 --bump-dev 2 → 1.2.3-alpha.1.post3.dev7 ✅
1.2.3-alpha.1.post2.dev5 + --bump-pre-release-num 1 --bump-post 2 --bump-dev 3 → 1.2.3-alpha.2.post2.dev3 ✅
```

### Post-release and Dev Bump Behavior

**Post-release and Dev bumps are additive only (no reset behavior):**

```bash
# Post-release bumps
1.2.3.post1 + --bump-post 2 → 1.2.3.post3 ✅
1.2.3.dev5 + --bump-dev 3 → 1.2.3.dev8 ✅
1.2.3 + --bump-post 1 → 1.2.3.post1 ✅ (creates if doesn't exist)
1.2.3 + --bump-dev 1 → 1.2.3.dev1 ✅ (creates if doesn't exist)
```

### Epoch Bump Behavior

**Epoch bumps reset ALL lower precedence components:**

```bash
# Epoch bumps (highest precedence - resets everything)
1!1.2.3 + --bump-epoch 1 → 2!0.0.0 ✅ (resets major/minor/patch)
1.2.3 + --bump-epoch 1 → 1!0.0.0 ✅ (creates epoch, resets major/minor/patch)
1.2.3-alpha.1.post2.dev5 + --bump-epoch 1 → 1!0.0.0 ✅ (resets all components)
```

### Mixed Component Bump Examples

**Complex scenarios combining multiple bump types:**

```bash
# Version + metadata bumps
1.2.3 + --bump-major --bump-post 2 --bump-dev 1 → 2.0.0.post2.dev1 ✅
1.2.3-alpha.1 + --bump-minor --bump-pre-release-num 3 → 1.3.0-alpha.3 ✅
1.2.3 + --bump-patch --bump-epoch 1 → 1!0.0.1 ✅

# Complex pre-release scenarios
1.2.3-alpha.1.post2.dev5 + --bump-major → 2.0.0 ✅ (major resets all lower precedence)
1.2.3-alpha.1.post2.dev5 + --bump-minor --bump-pre-release-num 2 → 1.3.0-alpha.2 ✅ (minor resets patch/pre/post/dev, pre-release creates alpha)
1.2.3-alpha.1.post2.dev5 + --bump-patch --bump-post 1 --bump-dev 1 → 1.2.4.post1.dev1 ✅ (patch resets pre/post/dev, then bumps post/dev)
1.2.3-alpha.1.post2.dev5 + --bump-major --bump-minor 2 --bump-patch 3 --bump-pre-release-num 1 --bump-post 1 --bump-dev 1 → 2.2.3-alpha.1.post1.dev1 ✅ (all explicit)
```

### Edge Cases and Validation

**Pre-release Label Validation:**

- **Valid labels**: `alpha`, `beta`, `rc` (case-sensitive)
- **Invalid labels**: Labels with special characters, numbers, or invalid formats
- **Error example**: `--pre-release-label "invalid!"` → **Error** (invalid characters)

**Conflicting Operations:**

- **Error**: `--pre-release-label` and `--bump-pre-release-label` together → **Early validation error**
- **Resolution**: Only one pre-release label operation allowed per command

**None Value Handling:**

- If version component is `None`, treat as `0` before bumping
- If metadata component is `None`, create it with the bump value
- Reset behavior applies to `None` values (set to `0` or remove)

**Override vs Bump Interaction:**

- **Overrides** (`--major 2`) set absolute values, no reset behavior
- **Bumps** (`--bump-major`) follow semantic versioning rules with reset
- **Precedence**: Overrides take precedence over bumps when both specified

## Processing Order

**Context → Override → Bump Logic**:

1. **Context Control** - Determine data scope
    - `--bump-context` (default): Include full VCS metadata
    - `--no-bump-context`: Force clean state (distance=0, dirty=false, no VCS context)
2. **Overrides** - Set absolute values (respects context control)
3. **Bumps** - Modify existing values (respects context control)

**Note**: Context control happens first and affects all subsequent processing. `--no-bump-context` eliminates VCS metadata, making VCS-related overrides and bumps meaningless.

## Templating Support

All override variables, bump increments, and output templates support Handlebars templating.

### Template Variables

**Version Components:**

- `{{ major }}`, `{{ minor }}`, `{{ patch }}` - Core version numbers
- `{{ epoch }}` - PEP440 epoch number
- `{{ post }}` - Post-release number
- `{{ dev }}` - Development number

**Pre-release Components:**

- `{{ pre_release.label }}` - Prerelease label (alpha, beta, rc)
- `{{ pre_release.num }}` - Prerelease number

**VCS Data:**

- `{{ distance }}` - Commits since tag
- `{{ dirty }}` - Working tree state (true/false)
- `{{ bumped_branch }}` - Bumped branch name
- `{{ bumped_commit_hash }}` - Full bumped commit hash
- `{{ bumped_commit_hash_short }}` - Short bumped commit hash (7 chars)
- `{{ bumped_timestamp }}` - Bumped commit timestamp
- `{{ last_commit_hash }}` - Last version commit hash
- `{{ last_branch }}` - Branch where last version was created
- `{{ last_timestamp }}` - Last version creation timestamp

**Custom Variables:**

- `{{ custom.* }}` - Any custom variables (e.g., `{{ custom.build_id }}`)

**Output Variables:**

- `{{ version }}` - Complete version string
- `{{ schema }}` - Complete schema in RON format

### RON Schema Variables

**Note**: RON Schema Variables use different syntax (`var("field_name")`) and are separate from Template Variables:

**Core Version Fields:**

- `var("major")`, `var("minor")`, `var("patch")` - Core version numbers
- `var("epoch")` - PEP440 epoch number
- `var("post")` - Post-release number
- `var("dev")` - Development number

**VCS State Fields:**

- `var("distance")` - Commits since tag
- `var("dirty")` - Working tree state
- `var("branch")` - Current branch name
- `var("commit_hash_short")` - Short commit hash
- `var("last_timestamp")` - Timestamp of last tagged version
- `var("last_branch")` - Branch of last tagged version
- `var("last_commit_hash")` - Commit hash of last tagged version

**Custom Fields:**

- `var("custom.field_name")` - Nested custom variables (e.g., `var("custom.build_id")`)

**Timestamp Formats:**

- `ts("YYYY")`, `ts("MM")`, `ts("DD")` - Single component patterns
- `ts("compact_date")`, `ts("compact_datetime")` - Preset patterns
- `ts("%Y-%m-%d")` - Custom chrono format strings

### Template Helpers

**Built-in Handlebars Helpers:**

- `{{ if condition true_value false_value }}` - Conditional logic
- `{{ eq a b }}` - Equality check
- `{{ gt a b }}` - Greater than check
- `{{ lt a b }}` - Less than check
- `{{ gte a b }}` - Greater than or equal
- `{{ lte a b }}` - Less than or equal
- `{{ ne a b }}` - Not equal
- `{{ and a b }}` - Logical AND
- `{{ or a b }}` - Logical OR
- `{{ not condition }}` - Logical NOT

**Custom Zerv Helpers:**

- `{{ add a b }}` - Addition (a + b)
- `{{ subtract a b }}` - Subtraction (a - b)
- `{{ multiply a b }}` - Multiplication (a \* b)
- `{{ hash input [length] }}` - Generate hex hash from input (default: 7 chars)
- `{{ hash_int input [length] allow_leading_zero=false }}` - Generate integer hash from input
- `{{ prefix string [length] }}` - Get prefix of string to length
- `{{ format_timestamp timestamp format=format_string }}` - Format unit timestamp to string

**Pre-defined Format Variables:**

- `iso_date` - ISO date format (`%Y-%m-%d`) → "2023-12-21"
- `iso_datetime` - ISO datetime format (`%Y-%m-%dT%H:%M:%S`) → "2023-12-21T12:34:56"
- `compact_date` - Compact date format (`%Y%m%d`) → "20231221"
- `compact_datetime` - Compact datetime format (`%Y%m%d%H%M%S`) → "20231221123456"

## Output Formats

### PEP440 Format

```bash
$ zerv version --output-format pep440
1.2.3.post5+main.abc1234

$ zerv version --no-bump-context --output-format pep440
1.2.3
```

### SemVer Format

```bash
$ zerv version --output-format semver
1.2.3+main.abc1234

$ zerv version --no-bump-context --output-format semver
1.2.3
```

### Zerv RON Format

```bash
$ zerv version --output-format zerv
(
    schema: (
        core: [var("major"), var("minor"), var("patch")],
        extra_core: [var("epoch"), var("pre_release"), var("post"), ts("YYYY")],
        build: [var("branch"), var("commit_hash_short"), str("stable"), int(1)],
    ),
    vars: (
        major: Some(1),
        minor: Some(2),
        patch: Some(3),
        post: Some(5),
        branch: Some("main"),
        commit_hash_short: Some("abc1234"),
        distance: Some(5),
        dirty: Some(false),
        custom: {
            build_id: "123",
            environment: "prod",
            pipeline: "github-actions",
            metadata: {
                author: "ci",
                timestamp: 1703123456,
                debug: false
            }
        },
    ),
)
```

**Practical Schema Examples:**

**Semantic Versioning with Build Info:**

```ron
(
    schema: (
        core: [var("major"), var("minor"), var("patch")],
        extra_core: [var("pre_release")],
        build: [var("branch"), var("commit_hash_short"), ts("compact_date")]
    )
)
```

**PEP440 with Epoch and Post Release:**

```ron
(
    schema: (
        core: [var("epoch"), var("major"), var("minor"), var("patch")],
        extra_core: [var("pre_release"), var("post")],
        build: [var("branch"), ts("YYYY"), ts("MM")]
    )
)
```

**Custom Versioning with Environment:**

```ron
(
    schema: (
        core: [var("major"), var("minor"), var("patch")],
        extra_core: [var("custom.environment"), var("custom.build_number")],
        build: [ts("compact_datetime"), var("commit_hash_short")]
    )
)
```

**Component Types:**

- `var("field_name")` - ZervVars field references (e.g., "major", "custom.build_id")
- `ts("format")` - Timestamp format patterns (e.g., "YYYY", "MM", "DD")
- `str("literal")` - String literals
- `int(123)` - Integer literals

### Available Variables for `var()`

**Core Version Fields:**

- `major` - Major version number
- `minor` - Minor version number
- `patch` - Patch version number
- `epoch` - Epoch number (for PEP440 compatibility)

**Pre-release Fields:**

- `pre_release` - Pre-release identifier (alpha, beta, rc, etc.)

**Post-release Fields:**

- `post` - Post-release number
- `dev` - Development release number

**VCS State Fields:**

- `distance` - Number of commits since last tag
- `branch` - Current branch name
- `commit_hash_short` - Short commit hash

**Custom Fields (Nested JSON):**

- `custom.field_name` - Access nested custom fields
- `custom.build_id` - Example: custom build identifier
- `custom.environment` - Example: deployment environment
- `custom.pipeline` - Example: CI/CD pipeline name
- `custom.metadata.author` - Example: nested metadata fields

### Available Formats for `ts()`

**Note:** Timestamps are used with `ts()` format, not `var()`. The timestamp data comes from `bumped_timestamp` (maps to last_timestamp).

**Preset Patterns (Compact Only):**

- `compact_date` - Compact date format (`%Y%m%d`) → "20231221"
- `compact_datetime` - Compact datetime format (`%Y%m%d%H%M%S`) → "20231221123456"

**Single Component Patterns:**

- `YYYY` - 4-digit year (e.g., 2024)
- `YY` - 2-digit year (e.g., 24)
- `MM` - 1-digit month (1-12)
- `0M` - 2-digit month (01-12)
- `DD` - 1-digit day (1-31)
- `0D` - 2-digit day (01-31)
- `HH` - 1-digit hour (0-23)
- `0H` - 2-digit hour (00-23)
- `mm` - 1-digit minute (0-59)
- `0m` - 2-digit minute (00-59)
- `SS` - 1-digit second (0-59)
- `0S` - 2-digit second (00-59)
- `WW` - 1-digit week of year (1-53)
- `0W` - 2-digit week of year (01-53)

**Custom Format Strings:**

- `ts("%Y%m")` - Custom format: 202403
- `ts("%Y-%m-%d")` - Custom format: 2024-03-15
- `ts("%H:%M:%S")` - Custom format: 14:30:25
- `ts("%Y%m%d%H%M")` - Custom format: 202403151430

**Note:** Custom formats use chrono format strings. If a pattern doesn't match any preset, it's treated as a custom chrono format string.

**Implementation Note:** The following field names should be defined as constants in `src/constants.rs`:

- **ZervVars fields**: `MAJOR`, `MINOR`, `PATCH`, `PRE_RELEASE`, `POST`, `EPOCH`, `BRANCH`, `COMMIT_HASH_SHORT`, `LAST_TIMESTAMP`
- **Timestamp patterns**: `COMPACT_DATE`, `COMPACT_DATETIME`
- **Format names**: `PEP440`, `SEMVER`, `ZERV`

**Examples:**

- `ts("compact_date")` - 20231221
- `ts("compact_datetime")` - 20231221123456
- `ts("YYYY")` - 2024
- `ts("MM")` - 3 (March)
- `ts("0M")` - 03 (March with leading zero)
- `ts("%Y%m")` - 202403 (custom format)

**Example Schema with Variables and Timestamps:**

```ron
(
    schema: (
        core: [var("major"), var("minor"), var("patch")],
        extra_core: [var("pre_release"), ts("YYYY"), ts("MM")],
        build: [var("branch"), var("custom.build_id"), ts("compact_datetime")],
    ),
    vars: (
        major: Some(1),
        minor: Some(2),
        patch: Some(3),
        pre_release: Some("alpha"),
        branch: Some("main"),
        custom: {
            build_id: "123",
            environment: "prod"
        },
    ),
)
```

## Command-Line Examples

### Basic Usage

```bash
# Get current version
zerv version

# Get version in specific format
zerv version --output-format pep440
zerv version --output-format semver

# Use custom schema
zerv version --schema-ron '(core: [var("major"), var("minor"), var("patch")])'
```

### Advanced Usage

```bash
# Override version components
zerv version --bump-major --bump-minor --output-format semver

# Use custom template
zerv version --output-template "v{{ major }}.{{ minor }}.{{ patch }}-{{ bumped_branch }}"

# Process from stdin
echo "1.2.3" | zerv version --source stdin --input-format semver --output-format pep440

# Use custom directory
zerv version -C /path/to/repo --output-format zerv

# Override specific fields
zerv version --override-major 2 --override-minor 0 --output-format semver
```

### Schema Examples

```bash
# Simple semantic versioning
zerv version --schema-ron '(core: [var("major"), var("minor"), var("patch")])'

# With pre-release and build info
zerv version --schema-ron '(core: [var("major"), var("minor"), var("patch")], extra_core: [var("pre_release")], build: [var("branch"), ts("compact_date")])'

# PEP440 with epoch
zerv version --schema-ron '(core: [var("epoch"), var("major"), var("minor"), var("patch")], extra_core: [var("pre_release"), var("post")])'
```

## Piping Workflows

### Basic Format Conversion

```bash
# Convert between formats
zerv version --output-format zerv | zerv version --source stdin --output-format pep440
zerv version --output-format zerv | zerv version --source stdin --output-format semver
```

### Schema Transformation

```bash
# Apply different schema to same version data
zerv version --output-format zerv | \
  zerv version --source stdin --schema-ron '(core: [var("major"), var("minor")])' --output-format pep440
```

### Multi-step Processing

```bash
# Complex transformation pipeline
zerv version --schema-ron '(core: [var("major"), var("minor")])' --output-format zerv | \
  zerv version --source stdin --schema-ron '(core: [var("major"), var("minor"), var("patch")])' --output-format zerv | \
  zerv version --source stdin --output-format semver
```

## Usage Examples

### Basic Version Generation

```bash
# Generate current version
zerv version

# Generate with specific format
zerv version --output-format pep440

# Generate with prefix
zerv version --output-prefix
```

### Version Overrides

```bash
# Override tag version
zerv version --tag-version v2.0.0 --distance 5

# Force clean state
zerv version --clean

# Override multiple values
zerv version --tag-version v1.5.0 --current-branch feature --dirty
```

### Version Bumps

```bash
# Bump patch version (1.2.3 → 1.2.4)
zerv version --bump-patch

# Bump minor version (1.2.3 → 1.3.0)
zerv version --bump-minor

# Bump major version (1.2.3 → 2.0.0)
zerv version --bump-major

# Multiple bumps - higher precedence resets lower, then explicit bumps applied
zerv version --bump-major --bump-minor 2 --bump-patch 3
# Result: 1.2.3 → 2.2.3 (major resets minor/patch to 0, then bumps to 2/3)

# Pre-release bumps reset post/dev components
zerv version --bump-pre-release-num 2
# Result: 1.2.3-alpha.1.post5.dev10 → 1.2.3-alpha.3 (post/dev reset)

# Epoch bumps reset all lower precedence components
zerv version --bump-epoch 1
# Result: 1.2.3 → 1!0.0.0 (epoch creates, resets major/minor/patch)
```

### Templating Examples

```bash
# Dynamic version based on branch
zerv version --tag-version={{ major }}.{{ minor }}.{{ add patch 1 }} --pre-release-label={{ if (eq bumped_branch "main") "rc" "dev" }}

# Hash-based prerelease number
zerv version --pre-release-num={{ hash_int bumped_commit_hash }}

# Timestamp-based versioning
zerv version --tag-version={{ format_timestamp last_timestamp format=iso_date }} --pre-release-num={{ format_timestamp bumped_timestamp format=compact_datetime }}

# Custom variables
zerv version --custom '{"build_id": "123", "environment": "prod"}' --output-template "v{{ major }}.{{ minor }}.{{ patch }}-{{ custom.build_id }}"
```

### CI/CD Pipeline Examples

```bash
# Generate version with custom build metadata
zerv version --custom '{"build_id": "123", "environment": "prod", "pipeline": "github-actions"}'

# Generate version for different package managers
zerv version --output-format semver > VERSION
zerv version --output-format pep440 > python/VERSION

# Complex conditional logic
zerv version --pre-release-label={{ if (gt distance 10) "alpha" (if (gt distance 5) "beta" "rc") }}
```

## Error Handling

The command provides clear, actionable error messages for common scenarios:

### Common Error Examples

**Invalid Timestamp Format:**

```bash
Error: Unknown timestamp pattern: "HHmmss". Valid patterns: YYYY, MM, DD, HH, mm, SS, compact_date, compact_datetime, or custom format starting with %
```

**Invalid Field Reference:**

```bash
Error: Unknown field "invalid_field" in schema. Available fields: major, minor, patch, pre_release, post, epoch, dev, branch, commit_hash_short, last_timestamp, custom.*
```

**Schema Parsing Error:**

```bash
Error: Failed to parse RON schema: expected ')' at line 3, column 15
```

**Not in a Git Repository:**

```bash
Error: Not in a git directory. Use -C <dir> to specify directory or --source string to parse version string
```

**Conflicting Options:**

```bash
Error: Cannot use --format with --input-format or --output-format
Error: Cannot use --no-bump-context with --dirty (conflicting options)
Error: Cannot use --clean with --dirty (conflicting options)
Error: Cannot use --clean with --no-dirty (conflicting options)
Error: Cannot use --clean with --distance (conflicting options)
```

## Troubleshooting

### Common Issues

**"Not in a git directory"**

- **Solution**: Use `-C <dir>` to specify directory or `--source string` to parse version string
- **Example**: `zerv version -C /path/to/repo` or `zerv version --source string "1.2.3"`

**"Unknown timestamp pattern"**

- **Solution**: Check pattern against supported formats (YYYY, MM, DD, HH, mm, SS, compact_date, compact_datetime, or custom format starting with %)
- **Example**: Use `ts("compact_date")` instead of `ts("YYYYMMDD")`

**"Schema parsing failed"**

- **Solution**: Validate RON syntax, ensure proper parentheses and commas
- **Example**: `(core: [var("major"), var("minor")])` not `(core: [var("major"), var("minor")]`

**"Field not found"**

- **Solution**: Check field name against available variables
- **Example**: Use `var("branch")` not `var("bumped_branch")`

**"Conflicting options"**

- **Solution**: Don't use `--format` with `--input-format` or `--output-format`
- **Example**: Use `--output-format pep440` instead of `--format pep440 --output-format semver`

**"--no-bump-context with --dirty"**

- **Solution**: `--no-bump-context` eliminates VCS metadata, making `--dirty` meaningless
- **Example**: Use `--no-bump-context` alone for clean tag versions, or `--bump-context` with `--dirty` for VCS-aware versions

**"--clean with other options"**

- **Solution**: `--clean` forces clean state, conflicting with dirty/distance overrides
- **Example**: Use `--clean` alone, or use individual overrides like `--dirty` or `--distance` without `--clean`

## Exit Codes

- `0` - Version generated successfully
- `1` - Error occurred (invalid input, not a git repo, etc.)

## Design Principles

1. **Composable**: Arguments can be combined in logical ways
2. **Predictable**: Processing order is consistent and documented
3. **Flexible**: Supports multiple input sources and output formats
4. **Powerful**: Rich templating system for complex scenarios
5. **Debuggable**: Zerv RON format enables inspection and debugging
6. **Pipeline-friendly**: Supports complex workflows through piping
7. **User-friendly**: Clear error messages and logical argument grouping
