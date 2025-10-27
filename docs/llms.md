# Zerv CLI Documentation

> Comprehensive CLI manual for Zerv dynamic versioning tool, covering all commands, options, and usage patterns for both humans and AI assistants.

<!--
This file follows the llms.txt standard: https://llmstxt.org/
-->

## Quick Start

Zerv is a dynamic versioning CLI tool that generates version strings from version control system (VCS) data using configurable schemas. It works with Git repositories and supports multiple version formats like SemVer (e.g., `1.2.3`) and PEP440 (Python, e.g., `1.2.0.post1.dev2`).

### Installation Check

Verify Zerv is installed and working:

```bash
zerv --version
```

### Basic Usage

Generate a basic semantic version from your Git repository:

```bash
# Basic version generation (defaults to SemVer format)
zerv version

# Generate PEP440 format for Python packages
zerv version --output-format pep440

# Use different directory
zerv version -C /path/to/repo
```

### Common Patterns

```bash
# Force clean release version (no distance, no dirty)
zerv version --clean

# Override VCS data for testing
zerv version --tag-version v1.2.0 --distance 5 --dirty

# Use preset schema for calendar versioning
zerv version --schema zerv-calver
```

## Core Concepts

### What is Dynamic Versioning?

Dynamic versioning generates version strings automatically from version control system state, rather than requiring manual version number management. Zerv reads Git repository information (tags, commits, branch state) and applies configurable schemas to generate consistent version strings.

### When to Use Zerv vs Manual Versions

**Use Zerv when:**

- Building CI/CD pipelines that need automatic versioning
- Managing multiple related projects with consistent versioning
- Following GitFlow or similar branching strategies
- Needing reproducible builds from specific commits

**Use manual versions when:**

- Following strict semantic versioning with manual release management
- Versioning doesn't correlate with VCS state
- Working with legacy systems requiring fixed version strings

### Version Sources

Zerv supports two primary input sources:

**Git Source (default)**: Extracts version information directly from Git repository state, including:

- Latest version tag (e.g., `v1.2.0`)
- Distance from tag (number of commits since tag)
- Working directory dirty state
- Current branch name and commit hash

**Stdin Source**: Reads version data in Zerv RON format from stdin, enabling command piping:

```bash
zerv version --output-format zerv | zerv version --source stdin --schema calver
```

## Commands Reference

### zerv version

Generate version strings from VCS data using configurable schemas and transformations.

#### Main Configuration

**Input Control:**

- `--source <SOURCE>`: Input source (`git` or `stdin`, default: `git`)
    - `'git'`: Extract from repository
    - `'stdin'`: Read Zerv RON format from stdin
- `--input-format <FORMAT>`: Input format for parsing (`auto`, `semver`, `pep440`, default: `auto`)
- `-C <DIR>`: Change to directory before running command

**Schema Configuration:**

- `--schema <NAME>`: Schema preset name (`zerv-standard`, `zerv-calver`, etc.)
- `--schema-ron <RON>`: Custom schema in RON format

**Output Control:**

- `--output-format <FORMAT>`: Output format (`semver` (default), `pep440`, `zerv` (RON format for piping))
- `--output-template <TEMPLATE>`: Output template for custom formatting (Handlebars syntax)
- `--output-prefix <PREFIX>`: Prefix to add to version output (e.g., `'v'` for `'v1.0.0'`)

#### Override Configuration

**VCS Override Options:**

- `--tag-version <VERSION>`: Override detected tag version (e.g., `'v2.0.0'`, `'1.5.0-beta.1'`)
- `--distance <NUM>`: Override distance from tag (number of commits since tag)
- `--dirty`: Override dirty state to true (sets dirty=true)
- `--no-dirty`: Override dirty state to false (sets dirty=false)
- `--clean`: Force clean release state (sets distance=0, dirty=false). Conflicts with --distance and --dirty
- `--bumped-branch <NAME>`: Override current branch name
- `--bumped-commit-hash <HASH>`: Override commit hash (full or short form)
- `--bumped-timestamp <TIMESTAMP>`: Override commit timestamp (Unix timestamp)

**Version Component Override Options:**

- `--major <NUM>`: Override major version number
- `--minor <NUM>`: Override minor version number
- `--patch <NUM>`: Override patch version number
- `--epoch <NUM>`: Override epoch number
- `--post <NUM>`: Override post number
- `--dev <NUM>`: Override dev number
- `--pre-release-label <LABEL>`: Override pre-release label (`alpha`, `beta`, `rc`)
- `--pre-release-num <NUM>`: Override pre-release number
- `--custom <JSON>`: Override custom variables in JSON format

**Schema Component Override Options:**

- `--core <INDEX=VALUE>`: Override core schema component by index
    - Positive index: `--core 0=5` (first component from start)
    - Tilde index: `--core ~1=2024` (last component from end)
    - Template: `--core 1={{major}}` (use template variables)
- `--extra-core <INDEX=VALUE>`: Override extra-core schema component by index
    - Positive index: `--extra-core 0=5` (first component from start)
    - Tilde index: `--extra-core ~1=beta` (last component from end)
    - Template: `--extra-core 1={{branch}}` (use template variables)
- `--build <INDEX=VALUE>`: Override build schema component by index
    - Positive index: `--build 0=5` (first component from start)
    - Tilde index: `--build ~1=release` (last component from end)
    - Template: `--build 1={{commit_short}}` (use template variables)

_Note: Use `~1` for last element, `~2` for second-to-last, etc._

#### Bump Configuration

**Field-Based Bump Options:**

- `--bump-major[=NUM]`: Add to major version (default: 1)
- `--bump-minor[=NUM]`: Add to minor version (default: 1)
- `--bump-patch[=NUM]`: Add to patch version (default: 1)
- `--bump-post[=NUM]`: Add to post number (default: 1)
- `--bump-dev[=NUM]`: Add to dev number (default: 1)
- `--bump-pre-release-num[=NUM]`: Add to pre-release number (default: 1)
- `--bump-epoch[=NUM]`: Add to epoch number (default: 1)
- `--bump-pre-release-label <LABEL>`: Bump pre-release label (`alpha`, `beta`, `rc`) and reset number to 0

**Schema-Based Bump Options:**

- `--bump-core <INDEX[=VALUE]>`: Bump core schema component by index (e.g., `--bump-core 0={{distance}}` or `--bump-core 0`)
- `--bump-extra-core <INDEX[=VALUE]>`: Bump extra-core schema component by index (e.g., `--bump-extra-core 0={{distance}}` or `--bump-extra-core 0`)
- `--bump-build <INDEX[=VALUE]>`: Bump build schema component by index (e.g., `--bump-build 0={{distance}}` or `--bump-build 0`)

**Context Control Options:**

- `--bump-context`: Include VCS context qualifiers (default behavior)
- `--no-bump-context`: Pure tag version, no VCS context

#### Practical Examples

_Repository state for examples: Latest tag `v0.7.73` with 5 commits since tag, working directory has uncommitted changes, on branch `some-branch`._

```bash
$ git describe --tags --abbrev=0
v0.7.73
$ git branch --show-current
some-branch
```

- **Distance**: Number of commits since the last version tag (5)
- **Dirty**: Working directory has uncommitted changes (`true`)
- **Branch**: Current git branch (`some-branch`)

**Basic Version Generation:**

```bash
# Generate SemVer from current Git state
zerv version
# Output: 0.7.74-dev.5+89e5a35

# Generate PEP440 format for Python
zerv version --output-format pep440
# Output: 0.7.74.dev5+89e5a35

# Use calendar versioning schema
zerv version --schema zerv-calver
# Output: 2024.1026.5+89e5a35

# Add prefix for Go modules
zerv version --output-prefix v
# Output: v0.7.74-dev.5+89e5a35
```

**Clean Release Examples:**

```bash
# Force clean release version (simulates tagged commit)
zerv version --clean
# Output: 0.7.73

# Simulate new tag release
zerv version --tag-version v1.0.0 --clean
# Output: 1.0.0
```

**VCS Override Testing:**

```bash
# Simulate tagged release with version bump
zerv version --tag-version v2.1.0 --clean
# Output: 2.1.0

# Simulate development build with specific distance
zerv version --tag-version v2.1.0 --distance 15 --dirty
# Output: 2.1.1-dev.15

# Test specific branch state
zerv version --bumped-branch feature/new-api --bumped-commit-hash abc123def
# Output: 0.7.74-dev.5+abc123def
```

**Component Override Examples:**

```bash
# Force specific version components
zerv version --major 2 --minor 1 --patch 0
# Output: 2.1.0-dev.5+89e5a35

# Set pre-release state
zerv version --pre-release-label beta --pre-release-num 3
# Output: 0.7.74-beta.3.dev.5+89e5a35

# Use custom variables
zerv version --custom '{"build_id":"123","metadata":"test"}'
# Output: 0.7.74-dev.5+89e5a35
```

**Schema Override Examples:**

```bash
# Override core schema components by index
zerv version --core 0={{major}} --core 1={{minor}} --core 2={{patch}}
# Output: 0.7.74-dev.5+89e5a35 (same as default, but explicit)

# Custom build metadata
zerv version --build 0={{bumped_commit_hash_short}} --build 1={{distance}}
# Output: 0.7.74-dev.5+89e5a35.89e5a35.5
```

**Pipeline and Integration Examples:**

```bash
# Pipe between commands
zerv version --output-format zerv | zerv version --source stdin --schema zerv-calver

# Generate with custom template
zerv version --output-template "{{major}}.{{minor}}.{{patch}}+{{distance}}.{{bumped_commit_hash_short}}"

# Use in CI/CD script
export VERSION=$(zerv version --output-format pep440 --clean)
echo "Generated version: $VERSION"
```

**Advanced Schema Examples:**

```bash
# Custom RON schema for complex versioning
zerv version --schema-ron '
(
  core: [
    Var(Major),
    Var(Minor),
    Var(Patch)
  ],
  extra_core: [
    Var(PreRelease)
  ],
  build: [
    Var(Distance),
    Var(BumpedCommitHashShort)
  ]
)
'
```

**Workflow Integration Examples:**

```bash
# Development workflow
zerv version --tag-version v1.2.0 --distance 5 --dirty
# Output: 1.2.1-dev.5+abcdef

# Release candidate workflow
zerv version --tag-version v1.2.0 --pre-release-label rc
# Output: 1.2.0-rc.1

# Production release workflow
zerv version --tag-version v1.2.0 --clean
# Output: 1.2.0
```

**Debug and Troubleshooting Examples:**

```bash
# Verbose output to see version calculation details
zerv version --verbose

# Test specific timestamp
zerv version --bumped-timestamp 1640995200

# Force dirty state for testing
zerv version --dirty --tag-version v1.0.0 --distance 0
```

**Different Working Directory:**

```bash
# Generate version for different repository
zerv version -C /path/to/other/repo

# Generate version for parent directory
zerv version -C ..
```

### zerv check

Validate version string format compliance for SemVer, PEP440, and other formats.

#### Options

- `--format <FORMAT>`: Format to validate against (`semver`, `pep440`), auto-detects if not specified

#### Examples

```bash
# Auto-detect and validate format
zerv check 1.2.3

# Validate specific format
zerv check --format semver 1.2.3
zerv check --format pep440 1.2.0.post1.dev2

# Invalid version examples (will show errors)
zerv check invalid.version.string
zerv check --format semver 1.2  # Missing patch version
```

## Troubleshooting

### Common Issues

**Issue: "No version tags found" error**

```bash
# Verify repository has version tags
git tag --list 'v*'

# Solution: Create initial version tag
git tag v1.0.0
git push origin v1.0.0
```

**Issue: Repository dirty when it should be clean**

```bash
# Check repository status
git status

# Solution: Clean working directory or use --clean flag
zerv version --clean
```

**Issue: Invalid schema RON format**

```bash
# Validate RON syntax with proper structure
zerv version --schema-on "(core: [Var(Major)])"
# Error: "Invalid schema RON format"

# Solution: Use proper RON syntax
zerv version --schema-ron "(core: [Var(Major)], extra_core: [], build: [])"
```

**Issue: Template variable not found**

```bash
# This will fail with undefined variable error
zerv version --output-template "{{undefined_var}}"

# Solution: Use available template variables
zerv version --output-template "{{major}}.{{minor}}.{{patch}}"
```

### Debugging with --verbose

Use verbose mode to see detailed version calculation information:

```bash
zerv version --verbose
# DEBUG: Detected Git repository state
# DEBUG: Found version tag: v1.2.0
# DEBUG: Distance from tag: 15 commits
# DEBUG: Working directory dirty: true
# DEBUG: Applying zerv-standard schema
# 1.2.1-dev.15+abcdef123
```

### Error Scenarios

**Missing schema file:**

```bash
zerv version --schema unknown-schema
# ERROR: Unknown preset schema name: unknown-schema
```

**Invalid version component:**

```bash
zerv version --major not_a_number
# ERROR: Invalid number format for major component
```

**Conflicting override options:**

```bash
zerv version --clean --distance 5
# ERROR: Cannot use --clean with --distance
```

**Invalid pre-release label:**

```bash
zerv version --pre-release-label invalid
# ERROR: Invalid pre-release label. Valid: alpha, beta, rc
```

### Getting Help

- Use `zerv --help` for basic command reference
- Use `zerv version --help` for detailed version command options
- Use `zerv check --help` for check command options
- Use `--verbose` flag to see detailed calculation information

## Handlebars Template System

Zerv uses Handlebars templating for advanced output customization and variable substitution in override values.

### Available Template Variables

**Core Version Components:**

- `{{major}}` - Major version number
- `{{minor}}` - Minor version number
- `{{patch}}` - Patch version number
- `{{epoch}}` - Epoch number (PEP440)

**Pre-release Components:**

- `{{pre_release.label}}` - Pre-release label (alpha, beta, rc)
- `{{pre_release.num}}` - Pre-release number

**Post-release Components:**

- `{{post}}` - Post-release number
- `{{dev}}` - Development number

**VCS State:**

- `{{distance}}` - Commits since last tag
- `{{dirty}}` - Working directory state (true/false)
- `{{bumped_branch}}` - Current branch name
- `{{bumped_commit_hash}}` - Full commit hash
- `{{bumped_commit_hash_short}}` - Short commit hash (7 chars)
- `{{bumped_timestamp}}` - Commit timestamp
- `{{last_branch}}` - Branch where last version was created
- `{{last_commit_hash}}` - Last version commit hash
- `{{last_commit_hash_short}}` - Last version short commit hash
- `{{last_timestamp}}` - Last version creation timestamp

**Custom Variables:**

- `{{custom.field_name}}` - Any custom variables from `--custom` JSON

**Formatted Versions:**

- `{{semver}}` - Complete SemVer format version
- `{{pep440}}` - Complete PEP440 format version

### Handlebars Helper Functions

Zerv provides custom Handlebars helpers for advanced templating:

**String Processing:**

```handlebars
{{sanitize "Feature/API-v2" preset="dotted"}}
# Feature.API.v2
{{sanitize "Feature-API-v2" preset="pep440"}}
# feature.api.v2
{{sanitize "Feature-API" separator="_"}}
# Feature_API
{{sanitize "feature/test" separator="-"}}
# feature-test
{{prefix "commit-hash" 7}}
# commit-h (first 7 chars)
```

**Hash Generation:**

```handlebars
{{hash "input-string"}}
# c7dedb4 (7 chars, default)
{{hash "input-string" 10}}
# c7dedb4632 (10 chars)
{{hash_int "input-string"}}
# 7126668 (no leading zeros)
{{hash_int "input-string" 5 allow_leading_zero=true}}
# 14402 (may start with zero)
```

**Timestamp Formatting:**

```handlebars
{{format_timestamp 1703123456}}
# 2023-12-21 (default)
{{format_timestamp 1703123456 format="%Y-%m-%d"}}
# 2023-12-21
{{format_timestamp 1703123456 format="compact_date"}}
# 20231221
{{format_timestamp 1703123456 format="compact_datetime"}}
# 2023122115056
```

**Math Operations:**

```handlebars
{{add 5 3}}
# 8
{{subtract 10 4}}
# 6
{{multiply 7 6}}
# 42
```

### Template Examples

```bash
# Dynamic version based on branch
zerv version --output-template "{{major}}.{{minor}}.{{patch}}-{{sanitize bumped_branch preset='dotted'}}"

# Include commit hash and distance
zerv version --output-template "v{{major}}.{{minor}}.{{patch}}+{{distance}}.{{bumped_commit_hash_short}}"

# Custom format with build metadata
zerv version --custom '{"build_id":"123","environment":"prod"}' \
  --output-template "{{major}}.{{minor}}.{{patch}}-build.{{custom.build_id}}"

# Calendar versioning with timestamp
zerv version --output-template "{{format_timestamp bumped_timestamp format='%Y'}}.{{format_timestamp bumped_timestamp format='%m'}}.{{distance}}"

# Sanitized branch name for package naming
zerv version --output-template "myapp-{{sanitize bumped_branch preset='pep440'}}-{{major}}.{{minor}}"

# Generate unique build identifier
zerv version --output-template "{{hash_int bumped_timestamp 5}}-{{prefix bumped_commit_hash_short 6}}"
```

## Schema System and Tiers

Zerv automatically determines version state tiers based on VCS conditions and applies appropriate schema components.

### Schema Tiers

**Tier 1: Clean Tagged Release**

- **Condition**: `distance = 0`, `dirty = false`
- **Schema**: Core version only (Major.Minor.Patch)
- **Example**: `1.2.3`
- **Use case**: Official releases, production deployments

**Tier 2: Post-Release (Clean with Distance)**

- **Condition**: `distance > 0`, `dirty = false`
- **Schema**: Core + Distance + Branch + Commit Hash
- **Example**: `1.2.3-post.5+dev.main.abc123`
- **Use case**: Development builds between releases

**Tier 3: Development (Dirty Working Directory)**

- **Condition**: `dirty = true` (regardless of distance)
- **Schema**: Core + Distance + Branch + Commit Hash + Dev
- **Example**: `1.2.3-dev.5+dev.main.abc123`
- **Use case**: Local development, feature branches

### Built-in Schema Presets

**Standard Schema (`zerv-standard`)**

```ron
(
    core: [Var(Major), Var(Minor), Var(Patch)],
    extra_core: [Var(Epoch), Var(PreRelease), Var(Post)],
    build: [Var(BumpedBranch), Var(Distance), Var(BumpedCommitHashShort)],
    precedence_order: [
        Epoch, Major, Minor, Patch, Core,
        PreReleaseLabel, PreReleaseNum, Post, Dev,
        ExtraCore, Build
    ],
)
```

**Calendar Versioning Schema (`zerv-calver`)**

```ron
(
    core: [
        Str("2024"),
        Str("12"),
        Str("26")
    ],
    extra_core: [Var(Major), Var(Minor), Var(Patch)],
    build: [Var(BumpedBranch), Var(Distance), Var(BumpedCommitHashShort)],
)
```

### Custom Schema Syntax

Schema components use RON (Rusty Object Notation) format:

**Component Types:**

- `Var(FieldName)` - Variable references (e.g., `Var(Major)`, `Var(BumpedBranch)`)
- `Str("literal")` - String literals (e.g., `Str("release")`, `Str("v1.0.0")`, `Str("alpha")`)
- `UInt(123)` - Unsigned integer literals (e.g., `UInt(0)`, `UInt(42)`)

**Available Variables:**

- **Core**: `Major`, `Minor`, `Patch`
- **Metadata**: `Epoch`, `PreRelease`, `Post`, `Dev`
- **VCS State**: `Distance`, `Dirty`, `BumpedBranch`, `BumpedCommitHash`, `BumpedCommitHashShort`, `BumpedTimestamp`
- **Last Version**: `LastBranch`, `LastCommitHash`, `LastCommitHashShort`, `LastTimestamp`
- **Custom**: `Custom("field_name")` for custom JSON fields

**Schema Sections:**

- **Core**: Primary version components (affects version precedence)
- **Extra_core**: Additional metadata (epoch, pre-release, post-release)
- **Build**: Build metadata (doesn't affect precedence)

### Built-in Schema Presets (Recommended)

**For most use cases, use built-in presets instead of custom schemas:**

```bash
# Recommended: Use built-in standard schema
zerv version --schema zerv-standard

# Recommended: Use built-in calendar versioning
zerv version --schema zerv-calver
```

### Built-in Schema RON Equivalents (For Reference Only)

**Standard Schema (`zerv-standard`) Equivalent:**

```ron
(
    core: [Var(Major), Var(Minor), Var(Patch)],
    extra_core: [Var(Epoch), Var(PreRelease), Var(Post)],
    build: [Var(BumpedBranch), Var(Distance), Var(BumpedCommitHashShort)],
    precedence_order: [
        Epoch, Major, Minor, Patch, Core,
        PreReleaseLabel, PreReleaseNum, Post, Dev,
        ExtraCore, Build
    ],
)
```

**Calendar Versioning Schema (`zerv-calver`) Equivalent:**

```ron
(
    core: [
        Str("2024"),
        Str("12"),
        Str("26")
    ],
    extra_core: [Var(Major), Var(Minor), Var(Patch)],
    build: [Var(BumpedBranch), Var(Distance), Var(BumpedCommitHashShort)],
)
```

## Advanced Usage Patterns

### Multi-Language Project Versioning

Zerv supports multiple output formats, making it ideal for polyglot projects:

```bash
# Generate versions for different ecosystems in one pipeline
echo "=== Multi-Language Version Generation ==="

# Python (PEP440)
zerv version --output-format pep440 --output-prefix ""

# JavaScript/Node.js (SemVer with build metadata)
zerv version --output-template "{{major}}.{{minor}}.{{patch}}+{{distance}}.{{bumped_commit_hash_short}}"

# Go modules (with 'v' prefix)
zerv version --output-prefix v

# Docker images (with date and hash)
zerv version --output-template "{{major}}.{{minor}}.{{patch}}-{{bumped_timestamp|date_format:YYYYMMDD}}-{{bumped_commit_hash_short}}"
```

### Branch-Based Versioning Strategies

Implement different versioning strategies based on branch types:

```bash
# Main branch production builds
zerv version --clean --schema zerv-standard

# Feature branch development builds
zerv version --pre-release-label dev --output-template "{{major}}.{{minor}}.{{patch}}-dev.{{distance}}"

# Release candidate branches
zerv version --pre-release-label rc --output-template "{{major}}.{{minor}}.{{patch}}-rc.{{distance}}"

# Hotfix branches
zerv version --output-template "{{major}}.{{minor}}.{{patch}}-hotfix.{{distance}}"
```

### Template System Deep Dive

Zerv's Handlebars-based template system provides powerful customization:

```bash
# Basic template usage
zerv version --output-template "v{{major}}.{{minor}}.{{patch}}"

# Conditional template logic
zerv version --output-template "{{major}}.{{minor}}.{{patch}}{{#if dirty}}-dev{{/if}}"

# Template helpers and formatting
zerv version --output-template "{{major}}.{{minor}}.{{patch}}+{{bumped_timestamp|date_format:\"YYYY-MM-DD\"}}"

# Complex multi-component templates
zerv version --output-template "{{major}}.{{minor}}.{{patch}}-{{#if dirty}}dirty-{{/if}}{{distance}}-{{bumped_commit_hash_short}}"

# Integration with environment variables
export BUILD_ID=$(date +%s)
zerv version --output-template "{{major}}.{{minor}}.{{patch}}+${BUILD_ID}"
```

## CI/CD Integration

### Quick Integration

**Basic CI/CD Integration:**

```bash
# Production releases
VERSION=$(zerv version --clean)

# Development builds
VERSION=$(zerv version --output-template "{{major}}.{{minor}}.{{patch}}+{{bumped_commit_hash_short}}")
```

**Best Practices**

1. Use `zerv version --clean` for releases
2. Use preset schemas that match your versioning strategy
3. Keep working directories clean for predictable versions
