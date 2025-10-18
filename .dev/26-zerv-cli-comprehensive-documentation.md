# Zerv CLI - Comprehensive Documentation

## Overview

Zerv is a dynamic versioning tool that generates version strings from version control system (VCS) data using configurable schemas. It supports multiple input sources, output formats, and advanced override capabilities for CI/CD workflows.

## Installation

### Quick Install (Recommended)

```bash
# Install latest version
curl -sSL https://raw.githubusercontent.com/wislertt/zerv/main/scripts/install.sh | bash

# Install specific version
curl -sSL https://raw.githubusercontent.com/wislertt/zerv/main/scripts/install.sh | bash -s v0.4.3

# Or using environment variable
curl -sSL https://raw.githubusercontent.com/wislertt/zerv/main/scripts/install.sh | ZERV_VERSION=v0.4.3 bash
```

### Manual Download

Download pre-built binaries from [GitHub Releases](https://github.com/wislertt/zerv/releases)

### From Source (Cargo)

```bash
cargo install zerv
```

## Commands

Zerv provides two main commands:

- `zerv version` - Generate version strings from VCS data
- `zerv check` - Validate version string format compliance

## Command: `zerv version`

Generate version strings from version control system data using configurable schemas.

### Basic Syntax

```bash
zerv version [OPTIONS]
```

### Input Sources

#### Git Repository (Default)

```bash
# Extract version data from git repository
zerv version --source git  # default
zerv version               # same as above
```

#### Standard Input

```bash
# Read Zerv RON format from stdin for piping workflows
echo "..." | zerv version --source stdin
```

### Output Formats

#### Semantic Versioning (Default)

```bash
zerv version --output-format semver  # default
zerv version                         # same as above
# Output: 0.7.72-post.4+dev.b3dd492
```

#### Python PEP440

```bash
zerv version --output-format pep440
# Output: 0.7.72.post4+dev.b3dd492
```

#### Zerv RON Format (For Piping)

```bash
zerv version --output-format zerv
# Output: Complete RON structure with schema and variables
```

### Schema System

#### Built-in Schema Presets

**Standard Schema (Default)**

```bash
zerv version --schema zerv-standard  # default
zerv version                         # same as above
# Output: 0.7.72-post.4+dev.b3dd492
```

**Calendar Versioning Schema**

```bash
zerv version --schema zerv-calver
# Output: 2025.10.16-72.post.4+dev.b3dd492
```

#### Custom RON Schema

```bash
zerv version --schema-ron '(core: [var(Major), var(Minor), var(Patch)])'
```

### VCS Override Options

Override detected VCS values for testing and simulation:

#### Tag Version Override

```bash
# Override detected tag version
zerv version --tag-version v2.0.0
zerv version --tag-version 1.5.0-beta.1
```

#### Distance Override

```bash
# Override distance from tag (number of commits since tag)
zerv version --distance 5
```

#### Dirty State Override

```bash
# Override dirty state to true
zerv version --dirty

# Override dirty state to false
zerv version --no-dirty

# Force clean release state (distance=0, dirty=false)
zerv version --clean
```

#### Branch and Commit Override

```bash
# Override current branch name
zerv version --current-branch feature-branch

# Override commit hash (full or short form)
zerv version --commit-hash abc1234
zerv version --commit-hash abc1234567890abcdef1234567890abcdef123456
```

### Version Component Overrides

Override specific version components:

#### Core Version Components

```bash
# Override major version number
zerv version --major 2

# Override minor version number
zerv version --minor 5

# Override patch version number
zerv version --patch 10
```

#### Extended Version Components

```bash
# Override epoch number
zerv version --epoch 1

# Override post number
zerv version --post 3

# Override dev number
zerv version --dev 7

# Override pre-release label (alpha, beta, rc)
zerv version --pre-release-label beta

# Override pre-release number
zerv version --pre-release-num 2
```

#### Custom Variables

```bash
# Override custom variables in JSON format
zerv version --custom '{"build_id": "123", "environment": "prod"}'
```

### Schema Component Overrides

Override schema components by index:

```bash
# Override core schema component by index=value
zerv version --core 0=5 --core 1=2

# Override extra-core schema component by index=value
zerv version --extra-core 0=alpha --extra-core 1=3

# Override build schema component by index=value
zerv version --build 0=main --build 1=abc1234
```

### Version Bumping

#### Field-Based Bumps

**Core Version Bumps**

```bash
# Add to major version (default: 1)
zerv version --bump-major
zerv version --bump-major 2

# Add to minor version (default: 1)
zerv version --bump-minor
zerv version --bump-minor 3

# Add to patch version (default: 1)
zerv version --bump-patch
zerv version --bump-patch 5
```

**Extended Component Bumps**

```bash
# Add to post number (default: 1)
zerv version --bump-post
zerv version --bump-post 2

# Add to dev number (default: 1)
zerv version --bump-dev
zerv version --bump-dev 3

# Add to pre-release number (default: 1)
zerv version --bump-pre-release-num
zerv version --bump-pre-release-num 2

# Add to epoch number (default: 1)
zerv version --bump-epoch
zerv version --bump-epoch 1
```

**Pre-release Label Bumps**

```bash
# Bump pre-release label and reset number to 0
zerv version --bump-pre-release-label alpha
zerv version --bump-pre-release-label beta
zerv version --bump-pre-release-label rc
```

#### Schema-Based Bumps

```bash
# Bump core schema component by index[=value]
zerv version --bump-core 0
zerv version --bump-core 0=5

# Bump extra-core schema component by index[=value]
zerv version --bump-extra-core 0
zerv version --bump-extra-core 1=2

# Bump build schema component by index[=value]
zerv version --bump-build 0
zerv version --bump-build 1=abc
```

#### Context Control

```bash
# Include VCS context qualifiers (default behavior)
zerv version --bump-context

# Pure tag version, no VCS context
zerv version --no-bump-context
```

### Output Customization

#### Output Prefix

```bash
# Add prefix to version output
zerv version --output-prefix v
# Output: v0.7.72-post.4+dev.b3dd492
```

#### Output Template (Handlebars)

```bash
# Custom template for output formatting
zerv version --output-template "v{{major}}.{{minor}}.{{patch}}-{{bumped_branch}}"
# Output: v0.7.72-dev
```

### Directory Control

```bash
# Change to directory before running command
zerv version -C /path/to/repo
```

### Input Format Control

```bash
# Input format for version string parsing
zerv version --input-format auto    # default (auto-detect)
zerv version --input-format semver  # semantic versioning
zerv version --input-format pep440  # Python PEP440
```

## Command: `zerv check`

Validate that version strings conform to specific format requirements.

### Basic Syntax

```bash
zerv check [OPTIONS] <VERSION>
```

### Format Validation

```bash
# Validate SemVer format
zerv check "1.2.3" --format semver
# Output: ✓ Valid SemVer format

# Validate PEP440 format
zerv check "1.2.3.post1" --format pep440
# Output: ✓ Valid PEP440 format

# Auto-detect format (default)
zerv check "1.2.3"
```

## Practical Examples

### Basic Usage

```bash
# Get current version from git
zerv version

# Get version in PEP440 format
zerv version --output-format pep440

# Use calendar versioning schema
zerv version --schema zerv-calver
```

### Testing and Simulation

```bash
# Simulate a specific version state
zerv version --tag-version v2.0.0 --distance 5 --dirty

# Test clean release state
zerv version --clean

# Override multiple components
zerv version --tag-version v1.5.0 --current-branch feature --dirty
```

### Version Bumping

```bash
# Bump patch version
zerv version --bump-patch

# Bump minor version (resets patch to 0)
zerv version --bump-minor

# Bump major version (resets minor and patch to 0)
zerv version --bump-major

# Multiple bumps with semantic versioning rules
zerv version --bump-major --bump-minor 2
# Result: major incremented, then minor set to 2, patch reset to 0
```

### CI/CD Integration

```bash
# Generate version with custom build metadata
zerv version --custom '{"build_id": "123", "environment": "prod"}'

# Generate version for different package managers
zerv version --output-format semver > VERSION
zerv version --output-format pep440 > python/VERSION

# Use in different repository
zerv version -C /path/to/repo --output-format pep440
```

### Piping Workflows

```bash
# Convert between formats
zerv version --output-format zerv | zerv version --source stdin --output-format pep440

# Apply different schema to same data
zerv version --output-format zerv | \
  zerv version --source stdin --schema zerv-calver --output-format semver

# Complex transformation pipeline
zerv version --schema zerv-standard --output-format zerv | \
  zerv version --source stdin --schema zerv-calver --output-format pep440
```

### Custom Templates

```bash
# Dynamic version based on branch
zerv version --output-template "{{major}}.{{minor}}.{{patch}}-{{bumped_branch}}"

# Include commit hash
zerv version --output-template "v{{major}}.{{minor}}.{{patch}}+{{bumped_commit_hash_short}}"

# Custom format with build metadata
zerv version --custom '{"build": "123"}' \
  --output-template "{{major}}.{{minor}}.{{patch}}-build.{{custom.build}}"
```

## Zerv RON Format

The Zerv RON (Rusty Object Notation) format provides complete version data for piping and debugging:

### Structure

```ron
(
    schema: (
        core: [var(Major), var(Minor), var(Patch)],
        extra_core: [var(Epoch), var(PreRelease), var(Post)],
        build: [var(BumpedBranch), var(BumpedCommitHashShort)],
        precedence_order: [Epoch, Major, Minor, Patch, Core, PreReleaseLabel, PreReleaseNum, Post, Dev, ExtraCore, Build],
    ),
    vars: (
        major: Some(0),
        minor: Some(7),
        patch: Some(72),
        epoch: None,
        pre_release: None,
        post: Some(4),
        dev: None,
        distance: Some(4),
        dirty: Some(false),
        bumped_branch: Some("dev"),
        bumped_commit_hash: Some("b3dd4923dd7f0a06520761b59139924e283b9a19"),
        bumped_timestamp: None,
        last_branch: None,
        last_commit_hash: None,
        last_timestamp: Some(1760630416),
        custom: (),
    ),
)
```

### Schema Components

#### Component Types

- `var(FieldName)` - Variable references (e.g., `var(Major)`, `var(BumpedBranch)`)
- `ts(Pattern)` - Timestamp patterns (e.g., `ts(YYYY)`, `ts(compact_date)`)
- `str("literal")` - String literals
- `int(123)` - Integer literals

#### Available Variables

- **Core Version**: `Major`, `Minor`, `Patch`, `Epoch`
- **Pre-release**: `PreRelease`
- **Post-release**: `Post`, `Dev`
- **VCS State**: `Distance`, `Dirty`, `BumpedBranch`, `BumpedCommitHashShort`
- **Timestamps**: `BumpedTimestamp`, `LastTimestamp`
- **Custom**: Access via nested paths in custom object

### Schema Sections

#### Core

Primary version components that determine version precedence.

#### Extra Core

Additional version metadata (epoch, pre-release, post-release).

#### Build

Build and VCS metadata that doesn't affect version precedence.

## Template Variables

When using `--output-template`, the following variables are available:

### Version Components

- `{{major}}`, `{{minor}}`, `{{patch}}` - Core version numbers
- `{{epoch}}` - PEP440 epoch number
- `{{post}}` - Post-release number
- `{{dev}}` - Development number

### Pre-release Components

- `{{pre_release.label}}` - Pre-release label (alpha, beta, rc)
- `{{pre_release.num}}` - Pre-release number

### VCS Data

- `{{distance}}` - Commits since tag
- `{{dirty}}` - Working tree state (true/false)
- `{{bumped_branch}}` - Current branch name
- `{{bumped_commit_hash}}` - Full commit hash
- `{{bumped_commit_hash_short}}` - Short commit hash (7 chars)
- `{{bumped_timestamp}}` - Commit timestamp
- `{{last_commit_hash}}` - Last version commit hash
- `{{last_branch}}` - Branch where last version was created
- `{{last_timestamp}}` - Last version creation timestamp

### Custom Variables

- `{{custom.*}}` - Any custom variables (e.g., `{{custom.build_id}}`)

## Error Handling

### Common Errors

**Not in a Git Repository**

```bash
Error: Not in a git directory. Use -C <dir> to specify directory or --source stdin to parse version string
```

**Solution**: Use `-C <dir>` or `--source stdin`

**Invalid Schema**

```bash
Error: Failed to parse RON schema: expected ')' at line 3, column 15
```

**Solution**: Validate RON syntax

**Conflicting Options**

```bash
Error: Cannot use --dirty and --no-dirty together
Error: Cannot use --clean with --distance (conflicting options)
```

**Solution**: Use only one conflicting option

### Exit Codes

- `0` - Success
- `1` - Error (invalid input, not a git repo, etc.)

## Version State Tiers

Zerv automatically determines version state based on VCS conditions:

### Tier 1: Clean Tagged Release

- **Condition**: `distance = 0`, `dirty = false`
- **Example**: `1.2.3`

### Tier 2: Post-Release (Distance)

- **Condition**: `distance > 0`, `dirty = false`
- **Example**: `1.2.3-post.4+dev.b3dd492`

### Tier 3: Development (Dirty)

- **Condition**: `dirty = true`
- **Example**: `1.2.3+dev.5.b3dd492`

## Best Practices

### CI/CD Integration

1. **Use specific output formats** for different package managers
2. **Override VCS state** for testing scenarios
3. **Use piping** for complex version transformations
4. **Validate versions** with `zerv check` before publishing

### Version Bumping

1. **Follow semantic versioning rules** - higher-level bumps reset lower components
2. **Use context control** to include/exclude VCS metadata
3. **Test with overrides** before applying to real repositories

### Schema Design

1. **Use preset schemas** for standard cases
2. **Create custom schemas** for specialized versioning needs
3. **Test schema behavior** with different VCS states

## Advanced Features

### Handlebars Templating

Zerv supports Handlebars templating in:

- Output templates (`--output-template`)
- Override values (most override options)
- Bump values (most bump options)

### Schema-Based Operations

- Override schema components by index
- Bump schema components independently
- Custom precedence ordering

### Piping Support

- Full data preservation through Zerv RON format
- Format conversion between SemVer, PEP440, and Zerv
- Multi-step processing pipelines

## Migration and Compatibility

### From Other Tools

Zerv can parse existing version strings in multiple formats:

- Semantic Versioning (SemVer)
- Python PEP440
- Auto-detection for mixed environments

### Integration Points

- **Git repositories** - Primary VCS integration
- **CI/CD pipelines** - Override capabilities for testing
- **Package managers** - Multiple output format support
- **Build systems** - Template-based custom formatting
