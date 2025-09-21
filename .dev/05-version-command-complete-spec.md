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
    - `zerv-default` (default) - Tier-aware schema
- `--schema-ron <RON>` - Custom RON schema definition

### VCS Override Options

- `--tag-version <TAG>` - Override detected tag version
- `--distance <NUM>` - Override distance from tag
- `--dirty <BOOL>` - Override dirty state (true/false/1/0/t/f/yes/no)
- `--clean` - Override to clean state (distance=0, dirty=false)
- `--current-branch <BRANCH>` - Override current branch name
- `--commit-hash <HASH>` - Override commit hash

### Output Options

- `--output-format <FORMAT>` - Target output format
    - `pep440` - Python PEP440 format
    - `semver` - Semantic Versioning format
    - `zerv` - Zerv RON format (for debugging/piping)
    - Default: `semver`
- `--output-template <TEMPLATE>` - Custom template string
- `--output-prefix [PREFIX]` - Add prefix (defaults to "v")

## Output Formats

### PEP440 Format

```bash
$ zerv version --output-format pep440
1.2.3

$ zerv version --output-format pep440  # With distance
1.2.3.post5+main.abc1234

$ zerv version --output-format pep440  # Dirty state
1.2.3.dev20241201123045+main.abc1234
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
        core: [VarField("major"), String("."), VarField("minor"), String("."), VarField("patch")],
        extra_core: [VarField("post"), String("+"), VarField("current_branch"), String("."), VarField("current_commit_hash")],
        build: [],
    ),
    vars: (
        major: Some(1),
        minor: Some(2),
        patch: Some(3),
        post: Some(5),
        current_branch: Some("main"),
        current_commit_hash: Some("abc1234"),
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
  zerv version --source stdin --schema custom.ron --output-format pep440
```

### Multi-step Processing

```bash
# Complex transformation pipeline
zerv version --schema tier1.ron --output-format zerv | \
  zerv version --source stdin --schema tier2.ron --output-format zerv | \
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
✗ Error: Not in a git repository
```

### No Tags Found

```bash
$ zerv version
✗ Error: No version tags found in repository
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
✗ Error: Unknown output format: unknown
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

## Behavior

1. **Auto-detects VCS** when no version argument provided
2. **Tier-based versioning** - output depends on repository state
3. **Format conversion** - transforms between version formats
4. **Clean output** - single line version string (no extra text)
5. **Error handling** - clear error messages with context
6. **Piping support** - Zerv RON format enables complex workflows with full data preservation
7. **Schema preservation** - Maintains schema through transformations

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
zerv version --schema-ron '(core: [VarField("major")])' --output-format zerv

# Compare schema outputs
zerv version --schema tier1 --output-format zerv > tier1.ron
zerv version --schema tier2 --output-format zerv > tier2.ron
diff tier1.ron tier2.ron
```
