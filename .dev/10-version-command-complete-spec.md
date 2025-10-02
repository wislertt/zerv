# Zerv Version Command Complete Specification

## Command Syntax

```bash
zerv version [OPTIONS]
```

## Source Options

- `--source <SOURCE>` - Version source
    - `git` (default) - Auto-detect Git repository
    - `stdin` - Read from stdin
- `--input-format <FORMAT>` - Input format when using stdin
    - `semver` (default) - Semantic Versioning format
    - `pep440` - Python PEP440 format
    - `zerv` - Zerv RON format (for piping only)

### Schema Options

- `--schema <SCHEMA>` - Version schema to apply
    - `zerv-standard` (default) - Standard semantic versioning (major.minor.patch)
    - `zerv-calver` - Calendar versioning (YYYY.MM.DD.PATCH)
- `--schema-ron <RON>` - Custom RON schema definition

### Available Schemas

**zerv-standard (Default):**

- **Tier 1** (Tagged, clean): `major.minor.patch`
- **Tier 2** (Distance, clean): `major.minor.patch.post<post>+branch.<commit>`
    - Where `post = previous_post + distance` (optimized Zerv logic)
- **Tier 3** (Dirty): `major.minor.patch.dev<timestamp>+branch.<distance>.<commit>`

**zerv-calver:**

- **Tier 1** (Tagged, clean): `YYYY.MM.DD.PATCH`
- **Tier 2** (Distance, clean): `YYYY.MM.DD.PATCH.post<post>+branch.<commit>`
    - Where `post = previous_post + distance` (optimized Zerv logic)
- **Tier 3** (Dirty): `YYYY.MM.DD.PATCH.dev<timestamp>+branch.<distance>.<commit>`

**Schema Examples:**

```bash
# Standard semantic versioning (default)
zerv version --schema zerv-standard
# Output: 1.2.3 (clean) or 1.2.3.post8+main.abc1234 (post=5+3=8)

# Calendar versioning
zerv version --schema zerv-calver
# Output: 2023.12.21.1 (clean) or 2023.12.21.1.post8+main.abc1234 (post=5+3=8)
```

**Post-Release Logic:**

- **Default behavior**: `post = previous_post + distance` (optimized Zerv way)
- **Override**: `--post <NUM>` sets absolute post value
- **Bump**: `--bump-post [<NUM>]` adds to existing post value
- **Template bump**: `--bump-post={{ distance }}` implements optimized logic

### VCS Override Options (Absolute Values)

- `--tag-version <TAG>` - Override detected tag version (supports templating)
- `--distance <NUM>` - Override distance from tag (supports templating)
- `--dirty` - Override dirty state to true
- `--no-dirty` - Override dirty state to false
- `--clean` - Override to clean state (distance=0, dirty=false)
- `--current-branch <BRANCH>` - Override bumped branch name (supports templating)
- `--commit-hash <HASH>` - Override bumped commit hash (supports templating)
- `--post <NUM>` - Override post number (supports templating)
- `--dev <NUM>` - Override dev number (supports templating)
- `--pre-release-label <LABEL>` - Override prerelease label (supports templating)
- `--pre-release-num <NUM>` - Override prerelease number (supports templating)
- `--epoch <NUM>` - Override epoch number (supports templating)
- `--custom <JSON>` - Override custom variables in JSON format (supports templating)

### VCS Bump Options (Relative Modifications)

- `--bump-major [<NUM>]` - Add to major version (default: 1, supports templating)
- `--bump-minor [<NUM>]` - Add to minor version (default: 1, supports templating)
- `--bump-patch [<NUM>]` - Add to patch version (default: 1, supports templating)
- `--bump-distance [<NUM>]` - Add to distance from tag (default: 1, supports templating)
- `--bump-post [<NUM>]` - Add to post number (default: 1, supports templating)
- `--bump-dev [<NUM>]` - Add to dev number (default: 1, supports templating)
- `--bump-pre-release-num [<NUM>]` - Add to prerelease number (default: 1, supports templating)
- `--bump-epoch [<NUM>]` - Add to epoch number (default: 1, supports templating)

### Context Bump Options (Boolean Flags)

- `--bump-context` - Add VCS context qualifiers (post, dev, etc.) - **default behavior**
- `--no-bump-context` - Pure tag version, no VCS context

### Processing Order

**Context → Override → Bump Logic**: Context determines VCS data scope, overrides set absolute values, then bumps modify those values.

1. **Context Bumps** - Determine VCS data scope
    - `--bump-context` (default): Include full VCS metadata (distance, branch, commit, etc.)
    - `--no-bump-context`: Use tag version only (no VCS metadata)
2. **VCS Overrides** - Set absolute values (e.g., `--distance 5`)
3. **VCS Bumps** - Modify existing values (e.g., `--bump-distance 2` → final distance = 7)

### Conflict Resolution

**Boolean Flag Conflicts:**

- `--bump-context` and `--no-bump-context` are mutually exclusive

**VCS Bump Conflicts:**

- Multiple VCS bump flags can be used together (e.g., `--bump-major 2 --bump-patch 1`)

**Output Conflicts:**

- `--output-template` and `--output-prefix` are mutually exclusive
- Custom templates provide full control over output format
- Prefix is only applied to standard format outputs

**Processing Examples:**

```bash
# Context → Override → Bump: Full context, set distance to 5, then add 2
$ zerv version --bump-context --distance 5 --bump-distance 2
# Result: 1.2.3.post7+main.7.abc1234 (context + distance=5+2=7)

# Context → VCS Bump: Full context, add to distance and post
$ zerv version --bump-context --bump-distance 1 --bump-post 2
# Result: 1.2.3.post7+main.6.abc1234 (context + distance +1, post +2)

# No Context → Override → VCS Bump: Tag-only, set distance, bump patch
$ zerv version --no-bump-context --distance 0 --bump-patch
# Result: 1.2.4 (no context, distance ignored, patch bump)

# Multiple VCS bumps: Bump major by 2, patch by 1
$ zerv version --bump-major 2 --bump-patch 1
# Result: 3.2.4 (major +2, patch +1)
```

### Templating Support

All override variables, bump increment, and output templates support Handlebars templating with the following syntax:

**Template Variables:**
All ZervVars fields are available in templates:

> **Note**: The field names in `ZervVars` need to be updated from `current_*` to `bumped_*` for consistency:
>
> - `current_branch` → `bumped_branch`
> - `current_commit_hash` → `bumped_commit_hash`
>   Additionally, `bumped_timestamp` needs to be added to `ZervVars` for the current commit timestamp.
>   This reflects that these fields contain the final processed values for version output, not raw VCS data.

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
- `{{ bumped_branch }}` - Bumped branch name (for version output)
- `{{ bumped_commit_hash }}` - Full bumped commit hash (for version output)
- `{{ bumped_commit_hash_short }}` - Short bumped commit hash (7 chars)
- `{{ bumped_timestamp }}` - Bumped commit timestamp (for version output)
- `{{ tag_commit_hash }}` - Tagged commit hash
- `{{ tag_branch }}` - Branch where tag was created
- `{{ tag_timestamp }}` - Tag creation timestamp

**Custom Variables:**

- `{{ custom.* }}` - Any custom variables (e.g., `{{ custom.build_id }}`)
- Set via `--custom <JSON>` - Override custom variables in JSON format
- Supports mixed data types: `{"build_id": "123", "version_num": 456, "is_release": true}`
- Nested objects: `{"config": {"env": "prod", "debug": false}}`

**Template Helpers:**

**Built-in Handlebars Helpers (Already Available):**

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

**Custom Zerv Helpers (Need Implementation):**

- `{{ add a b }}` - Addition (a + b)
- `{{ subtract a b }}` - Subtraction (a - b)
- `{{ multiply a b }}` - Multiplication (a \* b)
- `{{ hash input [length] }}` - Generate hex hash from input (default: 7 chars)
- `{{ hash_int input [length] allow_leading_zero=false }}` - Generate integer hash from input (default: allow_leading_zero=false)
- `{{ prefix string [length] }}` - Get prefix of string to length
- `{{ format_timestamp timestamp format=format_string }}` - Format unit timestamp to string

**Pre-defined Format Variables:**

- `iso_date` - ISO date format (`%Y-%m-%d`) → "2023-12-21"
- `iso_datetime` - ISO datetime format (`%Y-%m-%dT%H:%M:%S`) → "2023-12-21T12:34:56"
- `compact_date` - Compact date format (`%Y%m%d`) → "20231221"
- `compact_datetime` - Compact datetime format (`%Y%m%d%H%M%S`) → "20231221123456"

**Hash Helper Differences:**

- **`{{ hash input [length] }}`** - Returns hex string (e.g., "abc1234") for display/identification
- **`{{ hash_int input [length] allow_leading_zero=false }}`** - Returns integer (e.g., 1234567) for version components
    - `allow_leading_zero=false` (default): Natural length, up to specified max
    - `allow_leading_zero=true`: Pad with leading zeros to exact length

**Custom Variable Data Types:**

- **String**: `{"build_id": "123"}` → `{{ custom.build_id }}` = "123"
- **Integer**: `{"version_num": 456}` → `{{ custom.version_num }}` = 456
- **Boolean**: `{"is_release": true}` → `{{ custom.is_release }}` = true
- **Nested Objects**: `{"config": {"env": "prod"}}` → `{{ custom.config.env }}` = "prod"

**Structured Data Access:**

- **Pre-release**: `{{ pre_release.label }}` = "alpha", `{{ pre_release.num }}` = 1
- **Custom objects**: `{{ custom.config.env }}` = "prod", `{{ custom.metadata.author }}` = "ci"

**Template Examples:**

```bash
# Version Components
--tag-version={{ major }}.{{ minor }}.{{ add patch 1 }}
--bump-increment={{ distance }}
--post={{ add post 1 }}
--dev={{ add dev 1 }}

# VCS Data Usage
--pre-release-num={{ bumped_commit_hash_short }}
--pre-release-label={{ if (eq bumped_branch "main") "rc" "dev" }}
--current-branch={{ if dirty "dirty-{{ bumped_branch }}" bumped_branch }}

# Pre-release JSON Access
--pre-release-label={{ pre_release.label }}                    # alpha, beta, rc
--pre-release-num={{ pre_release.num }}                        # 1, 2, 3, etc.
--pre-release-label={{ if pre_release.label "rc" "dev" }}      # Conditional logic

# Post-Release Logic Examples
--bump-post={{ distance }}                    # Optimized: post = previous_post + distance
--bump-post=1                                 # Increment: post = previous_post + 1
--post=10                                     # Override: post = 10 (absolute value)

# Custom Variables Usage
--custom '{"build_id": "123", "environment": "prod"}'
--pre-release-num={{ custom.build_id }}
--output-template="v{{ major }}.{{ minor }}.{{ patch }}-{{ custom.build_id }}"

# Mixed Data Types
--custom '{"build_id": "123", "version_num": 456, "is_release": true}'
--dev={{ custom.version_num }}
--pre-release-label={{ if custom.is_release "rc" "dev" }}

# Nested Objects
--custom '{"config": {"env": "prod", "debug": false}, "metadata": {"author": "ci"}}'
--output-template="v{{ major }}.{{ minor }}.{{ patch }}-{{ custom.config.env }}"

# Timestamp-based Versioning
--tag-version={{ format_timestamp tag_timestamp format='%Y.%m.%d' }}
--pre-release-num={{ format_timestamp bumped_timestamp format='%H%M' }}

# Bumped Timestamp Usage
--pre-release-num={{ bumped_timestamp }}
--custom-build-time={{ bumped_timestamp }}

# Timestamp Formatting Examples

# Pre-defined formats (recommended for common cases)
--pre-release-num={{ format_timestamp bumped_timestamp format=iso_date }}        # 2023-12-21
--custom-build-time={{ format_timestamp bumped_timestamp format=compact_datetime }} # 20231221123456
--tag-version={{ format_timestamp tag_timestamp format=iso_datetime }}           # 2023-12-21T12:34:56

# Custom format strings (for specific needs)
--tag-version={{ format_timestamp tag_timestamp format='%B %d, %Y' }}            # December 21, 2023
--pre-release-num={{ format_timestamp bumped_timestamp format='%Y-%m-%d %H:%M' }} # 2023-12-21 12:34
--custom-build-id={{ format_timestamp bumped_timestamp format='%Y%j' }}          # 2023355 (year + day of year)
--release-date={{ format_timestamp tag_timestamp format='%A, %B %d, %Y' }}       # Thursday, December 21, 2023
--build-timestamp={{ format_timestamp bumped_timestamp format='%Y-%m-%dT%H:%M:%SZ' }} # 2023-12-21T12:34:56Z

# Post-Release Logic Examples
# Tag: v1.2.3.post5, Distance: 3 commits
zerv version                                                    # Result: 1.2.3.post8 (5+3=8)
zerv version --bump-post={{ distance }}                        # Result: 1.2.3.post8 (5+3=8)
zerv version --bump-post=2                                     # Result: 1.2.3.post7 (5+2=7)
zerv version --post=10                                          # Result: 1.2.3.post10 (override)

# Complex Expressions
--pre-release-num={{ add (multiply major 1000) (add minor 100) patch }}
--post={{ add distance (if dirty 1 0) }}

# Pre-computed Hash Fields
--pre-release-num={{ bumped_commit_hash_short }}        # Short hash (7 chars)
--pre-release-num={{ bumped_commit_hash }}              # Full hash
--custom-build-id={{ bumped_commit_hash_short }}

# String Prefix Examples (for custom lengths)
--pre-release-num={{ prefix bumped_commit_hash 8 }}     # 8 characters
--pre-release-num={{ prefix bumped_commit_hash 12 }}    # 12 characters
--custom-branch-hash={{ prefix bumped_branch 6 }}      # Custom length

# Hash Generation Examples (for non-hash inputs)
--pre-release-num={{ hash bumped_branch }}             # Hex hash: "abc1234"
--custom-build-id={{ hash "build-123" 8 }}            # Hex hash: "abc12345"

# Integer Hash Examples (for version components)
--dev={{ hash_int bumped_branch }}                     # Integer: 1234567
--dev={{ hash_int bumped_branch 6 }}                   # Integer: 123456 (up to 6 digits)
--dev={{ hash_int bumped_branch 6 allow_leading_zero=true }}  # Integer: 000123 (exactly 6 digits)

# Integer Hash with Padding Examples (for display with padding)
--pre-release-num={{ hash_int bumped_branch 8 allow_leading_zero=true }}  # Integer: 00012345 (padded to 8)
--custom-build-id={{ hash_int "build-123" 12 allow_leading_zero=true }}  # Integer: 000123456789 (padded to 12)

# Positional arguments (traditional)
--commit-hash={{ git_hash commit_hash 8 }}
--pre-release-num={{ hash bumped_branch 6 }}

# Named arguments (more readable)
--commit-hash={{ git_hash input=commit_hash length=8 }}
--pre-release-num={{ hash input=bumped_branch length=6 }}

# Default values (when supported)
--commit-hash={{ git_hash commit_hash }}  # defaults to 8 characters
--pre-release-num={{ hash bumped_branch }}  # defaults to 7 characters
--timestamp={{ format_timestamp bumped_timestamp format=compact_datetime }}  # 20231221123456

# Output template examples (Handlebars)
--output-template="v{{ major }}.{{ minor }}.{{ patch }}{{ if pre_release_num }}-{{ pre_release_label }}.{{ pre_release_num }}{{ /if }}"
--output-template="{{ major }}.{{ minor }}.{{ patch }}+{{ bumped_branch }}.{{ distance }}"
--output-template="{{ format_timestamp bumped_timestamp format='%Y.%m.%d' }}-{{ bumped_commit_hash_short }}"
--output-template="{{ major }}.{{ minor }}.{{ patch }}{{ if post }}.post{{ post }}{{ /if }}{{ if dev }}.dev{{ dev }}{{ /if }}+{{ bumped_branch }}.{{ bumped_commit_hash_short }}"
--output-template="{{ epoch }}{{ if epoch }}!{{ /if }}{{ major }}.{{ minor }}.{{ patch }}{{ if dirty }}-dirty{{ /if }}"
--output-template="{{ format_timestamp bumped_timestamp format='%Y%m%d' }}.{{ bumped_commit_hash_short }}"
--output-template="{{ bumped_timestamp }}-{{ bumped_commit_hash_short }}"
--output-template="{{ major }}.{{ minor }}.{{ patch }}+{{ bumped_branch }}.{{ hash bumped_commit_hash }}"  # Hex hash
```

### Output Options

- `--output-format <FORMAT>` - Target output format
    - `pep440` - Python PEP440 format
    - `semver` - Semantic Versioning format
    - `zerv` - Zerv RON format (for debugging/piping)
    - Default: `semver`
- `--output-template <TEMPLATE>` - Custom template string (Handlebars templating)
- `--output-prefix [PREFIX]` - Add prefix (defaults to "v")

## Output Formats

### PEP440 Format

```bash
# Get latest git tag with v*** prefix
$ git describe --tags --abbrev=0
v1.2.3

# Basic usage (default: --bump-context)
$ zerv version --output-format pep440
1.2.3.post5+main.5.abc1234

# Clean version (no context)
$ zerv version --output-format pep440 --no-bump-context
1.2.3
```

### Bump Examples

```bash
# Context bump (default behavior)
$ zerv version --output-format pep440 --distance=5
1.2.3.post5+main.5.abc1234

$ zerv version --output-format pep440 --dirty
1.2.3.dev20241201123045+main.abc1234

# No context (pure tag version)
$ zerv version --output-format pep440 --distance=5 --no-bump-context
1.2.3

$ zerv version --output-format pep440 --dirty --no-bump-context
1.2.3

# Version bumps
$ zerv version --output-format pep440 --bump-patch
1.2.4

$ zerv version --output-format pep440 --bump-minor
1.3.0

$ zerv version --output-format pep440 --bump-major
2.0.0

# VCS bumps (relative modifications)
$ zerv version --output-format pep440 --bump-distance 2
1.2.3.post7+main.7.abc1234  # distance increased by 2

$ zerv version --output-format pep440 --bump-post 3
1.2.3.post8+main.5.abc1234  # post increased by 3

# Combined overrides and bumps
$ zerv version --output-format pep440 --distance 5 --bump-distance 2
1.2.3.post7+main.7.abc1234  # distance = 5 + 2 = 7

$ zerv version --output-format pep440 --bump-patch --bump-distance 1
1.2.4.post6+main.6.abc1234  # patch bump + distance +1
```

### Multiple Bumps (Using Pipes)

```bash
# Version bump + context (patch bump with VCS metadata)
$ zerv version --bump-patch --output-format zerv | zerv version --bump-context --output-format pep440
1.2.4.post5+main.5.abc1234

# Context + version bump (context first, then patch bump)
$ zerv version --bump-context --output-format zerv | zerv version --bump-patch --output-format pep440
1.2.4.post5+main.5.abc1234

# Multiple version bumps (patch then minor)
$ zerv version --bump-patch --output-format zerv | zerv version --bump-minor --output-format zerv | zerv version --output-format pep440
1.3.0

# Complex pipeline: bump patch, add context, convert to SemVer
$ zerv version --bump-patch --output-format zerv | zerv version --bump-context --output-format zerv | zerv version --output-format semver
1.2.4-post.5+main.5.abc1234

# VCS bump + version bump + context
$ zerv version --bump-distance 2 --bump-patch --output-format zerv | zerv version --bump-context --output-format pep440
1.2.4.post7+main.7.abc1234
```

### Templating Examples

```bash
# Dynamic version based on branch
$ zerv version --tag-version={{ major }}.{{ minor }}.{{ add patch 1 }} --pre-release-label={{ if (eq bumped_branch "main") "rc" "dev" }}

# VCS bump with templating
$ zerv version --bump-distance={{ add distance 1 }} --bump-post={{ multiply distance 2 }}

# Distance-based increment for version bumps
$ zerv version --bump-increment={{ distance }}

# Hash-based prerelease number
$ zerv version --pre-release-num={{ hash_int bumped_commit_hash }}

# Complex conditional logic
$ zerv version --pre-release-label={{ if (gt distance 10) "alpha" (if (gt distance 5) "beta" "rc") }}

# Timestamp-based versioning
$ zerv version --tag-version={{ timestamp "%Y.%m.%d" }} --pre-release-num={{ timestamp "%H%M" }}

# Conditional VCS bumps
$ zerv version --bump-distance={{ if (eq bumped_branch "main") 0 1 }} --bump-context
```

### SemVer Format

```bash
$ zerv version --output-format semver
1.2.3

$ zerv version --output-format semver  # With distance
1.2.3+main.abc1234

$ zerv version --output-format semver  # Dirty state
1.2.3-dev.20241201123045+main.abc1234
```

### Zerv RON Format

```bash
$ zerv version --output-format zerv
(
    schema: (
        core: [var("major"), var("minor"), var("patch")],
        extra_core: [var("epoch"), var("pre_release"), var("post")],
        build: [var("bumped_branch"), var("bumped_commit_hash")],
    ),
    vars: (
        major: Some(1),
        minor: Some(2),
        patch: Some(3),
        post: Some(5),
        bumped_branch: Some("main"),
        bumped_commit_hash: Some("abc1234"),
        distance: Some(5),
        dirty: Some(false),
        // ... other fields
    ),
)
```

## Piping Workflows

### Basic Format Conversion

```bash
# Convert from git to different formats
zerv version --output-format zerv | zerv version --source stdin --output-format pep440
zerv version --output-format zerv | zerv version --source stdin --output-format semver
```

### Schema Transformation

```bash
# Apply different schema to same version data
zerv version --output-format zerv | \
  zerv version --source stdin --schema-ron custom.ron --output-format pep440
```

### Multi-step Processing

```bash
# Complex transformation pipeline
zerv version --schema-ron tier1.ron --output-format zerv | \
  zerv version --source stdin --schema-ron tier2.ron --output-format zerv | \
  zerv version --source stdin --output-format semver
```

### Debug Pipeline

```bash
# Inspect internal representation
zerv version --output-format zerv | jq '.'  # If RON → JSON converter available
zerv version --output-format zerv > debug.ron  # Save for inspection
```

## Source Modes

### Git Source (Default)

```bash
$ zerv version
1.2.3

$ zerv version --source git
1.2.3

# With overrides
$ zerv version --tag-version v2.0.0 --distance 5
2.0.0+main.abc1234

$ zerv version --dirty true --current-branch feature
1.2.3-dev.20241201123045+feature.abc1234

$ zerv version --clean  # Force clean state
1.2.3
```

### Stdin Source (For Zerv RON Piping Only)

```bash
# ✅ Valid: Zerv RON format
$ zerv version --output-format zerv | zerv version --source stdin --output-format pep440
1.2.3

# ❌ Invalid: Simple version strings not supported
$ echo "1.2.3" | zerv version --source stdin
✗ Error: stdin input must be Zerv RON format. For simple versions, use --tag-version instead.

# ✅ Alternative: Use VCS overrides for simple versions
$ zerv version --tag-version 1.2.3 --output-format pep440
1.2.3
```

## State-Based Versioning Tiers

### Tier 1: Tagged, Clean

```bash
$ zerv version  # On tagged commit, clean working tree
1.2.3
```

### Tier 2: Distance, Clean

```bash
$ zerv version  # 5 commits ahead of tag, clean working tree
1.2.3+main.abc1234
```

### Tier 3: Dirty

```bash
$ zerv version  # Uncommitted changes
1.2.3-dev.20241201123045+main.abc1234
```

## Input Format Requirements

When using `--source stdin`, Zerv **only accepts Zerv RON format**:

1. **Zerv RON Format** - Must contain `(schema: ..., vars: ...)` structure
2. **Simple version strings** - **Not supported** (use VCS overrides instead)
3. **Other formats** - **Not supported** (PEP440, SemVer strings will be rejected)

## Error Cases

### Not a Git Repository

```bash
$ zerv version
✗ Error: VCS not found: No VCS repository found
```

**Current Issue**: Error message mentions "VCS" instead of the specific source (git). Should be:

```bash
✗ Error: Not in a git repository (--source git)
```

### No Tags Found

```bash
$ zerv version
✗ Error: IO error: No version tag found in VCS data
```

**Current Issue**: Error message mentions "VCS data" instead of the specific source. Should be:

```bash
✗ Error: No version tags found in git repository
```

### No Commits (Empty Repository)

```bash
$ zerv version
✗ Error: Command execution failed: Git command failed: fatal: ambiguous argument 'HEAD': unknown revision or path not in the working tree.
Use '--' to separate paths from revisions, like this:
'git <command> [<revision>...] -- [<file>...]'
```

**Current Issue**: Shows raw git error. Should be:

```bash
✗ Error: No commits found in git repository
```

### Simple Version String (Not Supported)

```bash
$ echo "1.2.3" | zerv version --source stdin
✗ Error: stdin input must be Zerv RON format. For simple versions, use --tag-version instead.
```

### Invalid RON Input

```bash
$ echo "invalid" | zerv version --source stdin
✗ Error: stdin input must be Zerv RON format. For simple versions, use --tag-version instead.
```

### Invalid Zerv RON Format

```bash
$ echo "invalid ron" | zerv version --source stdin
✗ Error: Invalid Zerv RON format: expected '(' at line 1 column 1
```

### Unknown Format

```bash
$ zerv version --output-format unknown
✗ Error: Unknown format: unknown
Supported formats: pep440, semver, zerv
```

### No Input Available

```bash
$ zerv version --source stdin  # No stdin input
✗ Error: No input provided via stdin
```

### Conflicting Flags

```bash
$ zerv version --clean --distance 5
✗ Error: Cannot use --clean with --distance or --dirty (conflicting options)

$ zerv version --clean --dirty true
✗ Error: Cannot use --clean with --distance or --dirty (conflicting options)
```

## Error Handling Issues to Fix

**Priority Issues**:

1. **Source-Aware Error Messages**: All error messages should mention the specific source (git) instead of generic "VCS"
2. **User-Friendly Git Errors**: Raw git command errors should be translated to readable messages
3. **Empty Repository Handling**: "No commits" should be handled gracefully
4. **Consistent Error Format**: All errors should follow same format pattern

**Additional Error Cases to Consider**:

- **Shallow Clone**: Warning about inaccurate distance calculations
- **Detached HEAD**: Handle gracefully with appropriate messaging
- **Permission Denied**: Clear message when git commands fail due to permissions
- **Git Not Installed**: Clear message when git command is not available

## Implementation Details

### Zerv Format Structure

The Zerv RON format contains:

- **schema**: Complete schema definition with core/extra_core/build components
- **vars**: All version variables (major, minor, patch, distance, dirty, etc.)

This enables:

- **Complete roundtrip**: Parse → Transform → Output without data loss
- **Schema preservation**: Original schema travels through pipeline
- **Debug visibility**: See exact internal representation
- **Format conversion**: Transform between any supported formats

### Pipeline Architecture

```
Input → Version Object → Zerv Object → Transform → Output Version Object → Display
```

1. **Input**: Git VCS data or Zerv RON (via stdin)
2. **Version Object**: Parsed into PEP440/SemVer/Zerv
3. **Zerv Object**: Normalized internal representation
4. **Transform**: Apply schema, modify variables
5. **Output Version Object**: Convert to target format
6. **Display**: Serialize to string

### Source Processing

- `--source git`: VCS data → ZervVars → Zerv
- `--source stdin`: Read Zerv RON from stdin → Parse RON → Zerv

### Stdin Input Validation

1. **RON Structure Check** - Input must contain `(schema: ..., vars: ...)` structure
2. **RON Syntax Validation** - Must be valid RON format
3. **Zerv Schema Validation** - Schema and vars must be valid Zerv structure
4. **Rejection of Simple Strings** - Plain version strings like "1.2.3" are rejected

### Boolean Parsing

**Custom fuzzy boolean parser** (no external dependencies):

- **True values**: `true`, `t`, `yes`, `y`, `1`, `on` (case-insensitive)
- **False values**: `false`, `f`, `no`, `n`, `0`, `off` (case-insensitive)
- **Implementation**: Custom `FromStr` trait for user-friendly CLI experience

### Templating Implementation

**Handlebars Integration**:

- Uses `handlebars` crate for template processing
- Custom helper functions for version-specific operations
- Template context includes all ZervVars and VCS data
- Error handling for invalid template syntax

**Template Processing Pipeline**:

1. Parse template string for `{{ }}` syntax
2. Resolve variables from ZervVars context
3. Execute helper functions (add, hash, timestamp, etc.)
4. Return resolved string for override application

## Behavior

1. **Auto-detects VCS** when no version argument provided
2. **Tier-based versioning** - output depends on repository state
3. **Format conversion** - transforms between version formats
4. **Clean output** - single line version string (no extra text)
5. **Error handling** - clear error messages with context
6. **Piping support** - Zerv RON format enables complex workflows with full data preservation
7. **Schema preservation** - Maintains schema through transformations
8. **Templating support** - Dynamic variable resolution for all override options

## Exit Codes

- `0` - Version generated successfully
- `1` - Error occurred (invalid input, not a git repo, etc.)

## Use Cases

### Development Workflow

```bash
# Check current version
zerv version

# Generate release version
zerv version --output-format pep440 --output-prefix

# Debug version calculation
zerv version --output-format zerv

# Test different scenarios
zerv version --tag-version v1.0.0 --clean  # Simulate clean release
zerv version --tag-version v1.0.0 --distance 5  # Simulate post-release
zerv version --dirty true  # Simulate dirty working tree
```

### CI/CD Pipeline

```bash
# Generate version with custom build metadata
zerv version --custom '{"build_id": "123", "environment": "prod", "pipeline": "github-actions"}'

# Generate version with custom variables in template
zerv version --custom '{"build_id": "123"}' --output-template "v{{ major }}.{{ minor }}.{{ patch }}-{{ custom.build_id }}"

# Generate version for different package managers
zerv version --output-format semver > VERSION
zerv version --output-format pep440 > python/VERSION
```

### Format Conversion

```bash
# Convert using VCS overrides (for simple versions)
zerv version --tag-version 1.2.3-alpha.1 --output-format pep440
# Output: 1.2.3a1

# Batch convert using overrides
cat versions.txt | while read v; do
  zerv version --tag-version "$v" --output-format pep440
done

# Piping Zerv RON format (full data preservation)
zerv version --output-format zerv | zerv version --source stdin --output-format pep440
```

### Schema Development

```bash
# Test custom schema
zerv version --schema-ron '(core: [var("major")])' --output-format zerv

# Compare schema outputs
zerv version --schema zerv-standard --output-format zerv > standard.ron
zerv version --schema zerv-calver --output-format zerv > calver.ron
diff standard.ron calver.ron
```

### Advanced Templating Workflows

```bash
# Dynamic versioning based on branch
zerv version --tag-version={{ major }}.{{ minor }}.{{ add patch 1 }} --pre-release-label={{ if (eq bumped_branch "main") "rc" "dev" }}

# Hash-based versioning for unique builds
zerv version --pre-release-num={{ hash_int bumped_commit_hash }}

# Timestamp-based versioning
zerv version --tag-version={{ format_timestamp tag_timestamp format="%Y.%m.%d" }} --pre-release-num={{ format_timestamp bumped_timestamp format="%H%M" }}

# Complex conditional logic
zerv version --pre-release-label={{ if (gt distance 10) "alpha" (if (gt distance 5) "beta" "rc") }}
```

## Required Changes for Improved RON Format

### Component Enum Updates

**Current State:**

```rust
pub enum Component {
    String(String),
    Integer(u64),
    VarField(String),
    VarTimestamp(String),
    VarCustom(String),
}
```

**Required Changes:**

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Component {
    #[serde(rename = "str")]
    String(String),
    #[serde(rename = "int")]
    Integer(u64),
    #[serde(rename = "var")]
    VarField(String),  // Remove VarCustom, use var("custom.xxx")
    #[serde(rename = "ts")]
    VarTimestamp(String),
}
```

### ZervVars Custom Fields

**Current State:**

```rust
pub struct ZervVars {
    // ... existing fields
    pub custom: std::collections::HashMap<String, serde_json::Value>,
}
```

**Required Changes:**

```rust
use serde_json::Value;

pub struct ZervVars {
    // ... existing fields
    pub custom: Value,  // Change to nested JSON structure
}
```

### RON Format Migration

**Current RON:**

```ron
core: [VarField("major"), VarField("minor"), VarField("patch")]
build: [VarField("current_branch"), VarCustom("build_id")]
vars: (
    major: Some(1),
    custom.build_id: "123"  // Flattened
)
```

**New RON:**

```ron
core: [var("major"), var("minor"), var("patch")]
build: [var("current_branch"), var("custom.build_id"), str("stable"), int(1)]
vars: (
    major: Some(1),
    custom: {
        build_id: "123",
        environment: "prod"
    }
)
```

### Implementation Tasks

1. **Update Component enum** with serde rename attributes
2. **Remove VarCustom variant** - use var("custom.xxx") instead
3. **Change ZervVars.custom** from HashMap to Value for nested JSON
4. **Update all schema presets** to use new component names
5. **Update parser tests** to use new RON format
6. **Update documentation** with new format examples
7. **Add migration guide** for existing RON files
8. **Add timestamp preset patterns** - `compact_date`, `compact_datetime`
9. **Add custom format string support** - patterns starting with `%`
10. **Update timestamp tests** - add new pattern test cases
11. **Update field name mappings** - `tag_branch` → `last_branch`, `tag_commit_hash` → `last_commit_hash`, `tag_timestamp` → `last_timestamp`
12. **Improve error messages** - include format string in `parse_timestamp_component` error messages
13. **Replace bare strings with constants** - use constants/enums for field names instead of magic strings

## Required Changes from Current Code to Ideal State

### Field Name Changes in ZervVars

**Current Implementation:**

```rust
pub struct ZervVars {
    // ... other fields
    pub tag_timestamp: Option<u64>,
    pub tag_branch: Option<String>,
    pub current_branch: Option<String>,
    pub tag_commit_hash: Option<String>,
    pub current_commit_hash: Option<String>,
    // ... other fields
}
```

**Ideal State Changes:**

- Rename `tag_branch` → `last_branch` (internal field)
- Rename `tag_commit_hash` → `last_commit_hash` (internal field)
- Rename `tag_timestamp` → `last_timestamp` (internal field)
- Map to schema access: `last_branch` → `bumped_branch` (for var() access)
- Map to schema access: `last_commit_hash` → `bumped_commit_hash` (for var() access)
- Map to schema access: `last_timestamp` → `bumped_timestamp` (for ts() format)
- Remove `current_branch` and `current_commit_hash` (not needed in ideal state)
- Add preset timestamp patterns: `compact_date`, `compact_datetime`
- Add custom format string support with `%` prefix

### Schema Preset Updates

**Current Schema Presets** (in `src/schema/presets/mod.rs`):

```rust
// Current usage
Component::VarField("current_branch".to_string())
Component::VarField("current_commit_hash".to_string())
```

**Required Changes:**

```rust
// Update to use ideal field names
Component::VarField("bumped_branch".to_string())           // maps to tag_branch
Component::VarField("bumped_commit_hash".to_string())      // maps to tag_commit_hash
```

### Format Handler Updates

**Current Implementation** (in `src/cli/utils/format_handler.rs`):

```rust
"current_branch" if zerv.vars.current_branch.is_none() => {
    missing_vars.push("current_branch")
}
"current_commit_hash" if zerv.vars.current_commit_hash.is_none() => {
    missing_vars.push("current_commit_hash")
}
```

**Required Changes:**

```rust
"bumped_branch" if zerv.vars.last_branch.is_none() => {
    missing_vars.push("bumped_branch")
}
"bumped_commit_hash" if zerv.vars.last_commit_hash.is_none() => {
    missing_vars.push("bumped_commit_hash")
}
```

### Version Format Updates

**PEP440 Format** (in `src/version/pep440/from_zerv.rs`):

```rust
// Current
"current_branch" => {
    if let Some(branch) = &zerv.vars.current_branch {
        // ...
    }
}
"current_commit_hash" => {
    if let Some(hash) = &zerv.vars.current_commit_hash {
        // ...
    }
}

// Required changes
"bumped_branch" => {
    if let Some(branch) = &zerv.vars.last_branch {
        // ...
    }
}
"bumped_commit_hash" => {
    if let Some(hash) = &zerv.vars.last_commit_hash {
        // ...
    }
}
```

**SemVer Format** (in `src/version/semver/from_zerv.rs`):

```rust
// Current
"current_branch" => zerv.vars.current_branch.as_ref().map(|s| s.as_str()),
"current_commit_hash" => zerv.vars.current_commit_hash.as_ref().map(|s| s.as_str()),

// Required changes
"bumped_branch" => zerv.vars.last_branch.as_ref().map(|s| s.as_str()),
"bumped_commit_hash" => zerv.vars.last_commit_hash.as_ref().map(|s| s.as_str()),
```

### Timestamp Format Updates

**Current Implementation** (in `src/version/zerv/utils.rs`):

```rust
pub fn resolve_timestamp(pattern: &str, timestamp: Option<u64>) -> Result<u64> {
    // ... existing code ...
    let result = match pattern {
        "YYYY" => parse_timestamp_component(&dt, "%Y", "year")?,
        "YY" => parse_timestamp_component(&dt, "%y", "year")?,
        "MM" => parse_timestamp_component(&dt, "%-m", "month")?,
        "0M" => parse_timestamp_component(&dt, "%m", "month")?,
        // ... other single components
        _ => {
            return Err(ZervError::InvalidFormat(format!(
                "Unknown timestamp pattern: {pattern}"
            )));
        }
    };
    Ok(result)
}
```

**Required Changes:**

```rust
pub fn resolve_timestamp(pattern: &str, timestamp: Option<u64>) -> Result<u64> {
    // ... existing code ...

    let result = match pattern {
        // Preset patterns first (exact matches)
        "compact_date" => parse_timestamp_component(&dt, "%Y%m%d", "compact-date")?,
        "compact_datetime" => parse_timestamp_component(&dt, "%Y%m%d%H%M%S", "compact-datetime")?,
        "YYYY" => parse_timestamp_component(&dt, "%Y", "year")?,
        "YY" => parse_timestamp_component(&dt, "%y", "year")?,
        "MM" => parse_timestamp_component(&dt, "%-m", "month")?,
        "0M" => parse_timestamp_component(&dt, "%m", "month")?,
        "DD" => parse_timestamp_component(&dt, "%-d", "day")?,
        "0D" => parse_timestamp_component(&dt, "%d", "day")?,
        "HH" => parse_timestamp_component(&dt, "%-H", "hour")?,
        "0H" => parse_timestamp_component(&dt, "%H", "hour")?,
        "mm" => parse_timestamp_component(&dt, "%-M", "minute")?,
        "0m" => parse_timestamp_component(&dt, "%M", "minute")?,
        "SS" => parse_timestamp_component(&dt, "%-S", "second")?,
        "0S" => parse_timestamp_component(&dt, "%S", "second")?,
        "WW" => parse_timestamp_component(&dt, "%-W", "week")?,
        "0W" => parse_timestamp_component(&dt, "%W", "week")?,

        // Custom format fallback - try to parse as chrono format
        _ => {
            // Try to parse as custom chrono format
            parse_timestamp_component(&dt, pattern, "custom format")
                .map_err(|_| ZervError::InvalidFormat(format!(
                    "Unknown timestamp pattern: {pattern}"
                )))?
        }
    };

    Ok(result)
}
```

**New Test Cases to Add:**

```rust
#[rstest]
// Preset patterns
#[case(1710511845, "compact_date", 20240315)] // 2024-03-15 14:10:45
#[case(1710511845, "compact_datetime", 20240315141045)]

// Custom format strings (no % prefix required)
#[case(1710511845, "%Y%m", 202403)] // Custom format
#[case(1710511845, "%Y-%m-%d", 20240315)] // Custom format with separators
#[case(1710511845, "%H:%M:%S", 141045)] // Custom time format

// Edge cases
#[case(1710511845, "", 0)] // Empty string - should fail
#[case(1710511845, "invalid", 0)] // Invalid pattern - should fail
fn test_resolve_timestamp_new_patterns(
    #[case] timestamp: u64,
    #[case] pattern: &str,
    #[case] expected: u64,
) {
    let result = resolve_timestamp(pattern, Some(timestamp));
    if expected == 0 {
        // Should fail for edge cases
        assert!(result.is_err());
    } else {
        assert_eq!(result.unwrap(), expected);
    }
}
```

### Replace Bare Strings with Constants

**Current Problem:**
The codebase uses **extensive bare strings** throughout (280+ instances found), which is error-prone and hard to maintain:

```rust
// Field names - 280+ instances
Component::VarField("major".to_string())
Component::VarField("minor".to_string())
Component::VarField(ron_fields::PATCH.to_string())
Component::VarField("distance".to_string())
Component::VarField("dirty".to_string())
Component::VarField("current_branch".to_string())
Component::VarField("current_commit_hash".to_string())
Component::VarField("tag_branch".to_string())
Component::VarField("tag_commit_hash".to_string())
Component::VarField("tag_timestamp".to_string())

// Complex timestamp patterns - 10+ instances
Component::VarTimestamp("compact_date".to_string())
Component::VarTimestamp("compact_datetime".to_string())
// Single patterns like "YYYY", "MM" can stay as bare strings

// Source types and format names - 14+ instances
"git" => { ... }        // Source type
"stdin" => { ... }      // Source type
"auto" => { ... }       // Format name
"semver" => { ... }     // Format name
"pep440" => { ... }     // Format name
```

**Required Changes:**

```rust
// Define constants for all string literals
pub mod constants {
    // Field names
    pub mod fields {
        // Core version fields
        pub const MAJOR: &str = "major";
        pub const MINOR: &str = "minor";
        pub const PATCH: &str = "patch";
        pub const EPOCH: &str = "epoch";

        // Pre-release fields
        pub const PRE_RELEASE: &str = "pre_release";

        // Post-release fields
        pub const POST: &str = "post";
        pub const DEV: &str = "dev";

        // VCS state fields
        pub const DISTANCE: &str = "distance";
        pub const DIRTY: &str = "dirty";
        pub const CURRENT_BRANCH: &str = "current_branch";
        pub const CURRENT_COMMIT_HASH: &str = "current_commit_hash";
        pub const TAG_BRANCH: &str = "tag_branch";
        pub const TAG_COMMIT_HASH: &str = "tag_commit_hash";
        pub const TAG_TIMESTAMP: &str = "tag_timestamp";
        pub const BUMPED_BRANCH: &str = "bumped_branch";
        pub const BUMPED_COMMIT_HASH: &str = "bumped_commit_hash";
        pub const BUMPED_TIMESTAMP: &str = "bumped_timestamp";

        // Custom fields
        pub const CUSTOM: &str = "custom";
    }

    // Timestamp patterns (only complex ones)
    pub mod timestamp_patterns {
        pub const COMPACT_DATE: &str = "compact_date";
        pub const COMPACT_DATETIME: &str = "compact_datetime";
    }

    // Source types
    pub mod sources {
        pub const GIT: &str = "git";
        pub const STDIN: &str = "stdin";
    }

    // Format names
    pub mod formats {
        pub const AUTO: &str = "auto";
        pub const SEMVER: &str = "semver";
        pub const PEP440: &str = "pep440";
        pub const ZERV: &str = "zerv";
    }
}

// Usage - type-safe and maintainable
Component::VarField(constants::fields::MAJOR.to_string())
Component::VarField(constants::fields::MINOR.to_string())
Component::VarField(constants::fields::PATCH.to_string())
Component::VarTimestamp(constants::timestamp_patterns::COMPACT_DATE.to_string())
// Single patterns like "YYYY", "MM", "DD" can stay as bare strings
Component::VarTimestamp("YYYY".to_string())
Component::VarTimestamp("MM".to_string())
```

**Benefits:**

- **Type Safety** - Compile-time checking of field names
- **Refactoring Safety** - IDE can rename all usages automatically
- **Documentation** - Constants serve as documentation of available fields
- **Consistency** - Prevents typos and ensures consistent naming
- **Maintainability** - Easy to add/remove/modify field names

**Files to Update (280+ instances across):**

- `src/version/zerv/utils.rs` - Field name matching in `extract_core_values`
- `src/cli/utils/format_handler.rs` - Missing variable checks (15+ instances)
- `src/version/pep440/from_zerv.rs` - Field name matching
- `src/version/semver/from_zerv.rs` - Field name matching
- `src/schema/presets/mod.rs` - Schema component creation
- `src/version/zerv/test_utils.rs` - Test data creation (100+ instances)
- `src/version/zerv/core.rs` - Test data and component creation
- `src/version/zerv/display.rs` - Display components
- `src/version/zerv/parser.rs` - Parser test data
- `src/schema/parser.rs` - Parser test data
- `src/schema/presets/calver.rs` - CalVer timestamp patterns
- `src/cli/version.rs` - Format names and CLI values

### Error Message Improvements

**Current Implementation** (in `src/version/zerv/utils.rs`):

```rust
fn parse_timestamp_component(
    dt: &chrono::DateTime<chrono::Utc>,
    format_str: &str,
    component_type: &str,
) -> Result<u64> {
    dt.format(format_str)
        .to_string()
        .parse()
        .map_err(|_| ZervError::InvalidFormat(format!("Failed to parse {component_type}")))
}
```

**Required Changes:**

```rust
fn parse_timestamp_component(
    dt: &chrono::DateTime<chrono::Utc>,
    format_str: &str,
    component_type: &str,
) -> Result<u64> {
    dt.format(format_str)
        .to_string()
        .parse()
        .map_err(|_| ZervError::InvalidFormat(format!(
            "Failed to parse {component_type} with format '{format_str}'"
        )))
}
```

**Benefits:**

- **Better debugging** - Shows exactly which format string failed
- **Clearer error messages** - Users can see what format was attempted
- **Easier troubleshooting** - Helps identify invalid chrono format strings

### Test Updates

**Test Utilities** (in `src/version/zerv/test_utils.rs`):

```rust
// Current test data
Component::VarField("current_branch".to_string())
Component::VarField("current_commit_hash".to_string())

// Required changes
Component::VarField("bumped_branch".to_string())
Component::VarField("bumped_commit_hash".to_string())
```

**Schema Tests** (in `src/schema/mod.rs`):

```rust
// Current test data
current_branch: Some("main".to_string()),
current_commit_hash: Some("abc123".to_string()),

// Required changes - update test expectations
// Tests should expect the new field names in schema output
```

### Migration Strategy

1. **Phase 1**: Add backward compatibility
    - Keep both old and new field names working
    - Add deprecation warnings for old names

2. **Phase 2**: Update all internal usage
    - Update schema presets
    - Update format handlers
    - Update version formatters
    - Update tests

3. **Phase 3**: Remove old field names
    - Remove backward compatibility
    - Clean up deprecated code

### Breaking Changes

**Schema RON Format:**

```ron
# Old format (will break)
core: [var("current_branch"), var("current_commit_hash")]

# New format (required)
core: [var("bumped_branch"), var("bumped_commit_hash")]
```

**Custom Field Access:**

```ron
# Old format (will break)
custom.current_branch

# New format (required)
custom.build_id  # nested JSON access
```
