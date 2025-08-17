# New CLI Design: Pipeline Architecture

## Overview

The new CLI design uses a unified pipeline architecture that processes versions through distinct stages:

```
Input → Version Object → Zerv Object → Transform → Output Version Object → Display
```

## Core Commands

### `zerv version [OPTIONS]`

Main version processing pipeline with composable operations.

### `zerv check <version> [OPTIONS]`

Validation-only command for version strings.

## Pipeline Stages

### 1. Input Sources (`--source`)

```bash
--source git                        # Auto-detect Git (default)
--source stdin                      # Read from stdin
--source tag <tag>                  # Parse specific tag
--source string <version>           # Parse version string
```

### 2. Schema Control (Internal Transform)

```bash
--schema zerv-default               # Zerv's opinionated schema (default)
--schema-ron <ron>                  # Custom RON schema
--schema-file <path>                # RON schema from file
```

### 3. Transformations

```bash
--bump major|minor|patch|alpha|beta|rc
--set-distance <n>
--set-dirty
--set-branch <name>
```

### 4. Output Control

```bash
--output-format <format>            # Target format: pep440, semver, pvp
--output-template <template>        # Custom template string
--output-prefix [prefix]            # Add prefix to output (defaults to "v")
```

**Note**: `--output-format` and `--output-template` are mutually exclusive (error if both used). `--output-prefix` can combine with either.

## Usage Examples

### Simple Cases

```bash
# Basic usage (Git → auto-format)
zerv version

# Validate version
zerv check 1.2.3

# Standard format output
zerv version --output-format pep440

# Standard format with "v" prefix (most common)
zerv version --output-format semver --output-prefix

# Custom prefix
zerv version --output-format semver --output-prefix "release-"

# Custom template output
zerv version --output-template "v{{major}}.{{minor}}.{{patch}}"
```

### Advanced Pipeline

```bash
# Full pipeline with custom schema
zerv version \
  --source git \
  --bump minor \
  --schema-ron '(core: [VarField("major"), VarField("minor"), VarField("patch")], extra_core: [VarField("pre_release")], build: [VarField("commit")])' \
  --output-format pep440

# Template-based output
zerv version \
  --source git \
  --bump patch \
  --output-template "v{{major}}.{{minor}}.{{patch}}-{{commit}}"

# State overrides with default schema
zerv version \
  --source string "1.2.3" \
  --set-distance 5 \
  --set-dirty \
  --output-format semver
```

### Schema Examples

```bash
# Default Zerv schema (tier-aware)
--schema zerv-default

# Simple schema (major.minor only)
--schema-ron '(core: [VarField("major"), VarField("minor")], extra_core: [], build: [])'

# CalVer schema (YYYY.MM.DD.patch)
--schema-ron '(core: [VarTimestamp("YYYY"), VarTimestamp("MM"), VarTimestamp("DD"), VarField("patch")], extra_core: [], build: [])'

# Custom Git schema (major.minor.branch.distance.commit)
--schema-ron '(core: [VarField("major"), VarField("minor"), VarField("current_branch"), VarField("distance"), VarField("current_commit_hash")], extra_core: [], build: [])'
```

### Zerv Default Schema

The `--schema zerv-default` preset provides a tier-aware schema:

```ron
(core: [VarField("major"), VarField("minor"), VarField("patch")], extra_core: [VarField("epoch"), VarField("pre_release"), VarField("post"), VarField("dev")], build: [VarField("current_branch"), VarField("distance"), VarField("current_commit_hash")])
```

**Tier-based component population:**

- **Tier 1** (Tagged, clean): `major`, `minor`, `patch`, `epoch`, `pre_release`
- **Tier 2** (Distance, clean): Above + `post`, `current_branch`, `distance`, `current_commit_hash`
- **Tier 3** (Dirty): All components including `dev` (timestamp)

## Usage Patterns

### Pattern 1: Custom Schema + Standard Format

**Use Case**: Different data structure, compliant output

```bash
# CalVer schema with PEP440 output
zerv version --schema-ron '(core: [VarTimestamp("YYYY"), VarTimestamp("MM"), VarField("patch")], ...)' --output-format pep440
```

### Pattern 2: Default Schema + Custom Template

**Use Case**: Standard data structure, custom output

```bash
# Default schema with custom output format
zerv version --output-template "v{{major}}.{{minor}}.{{patch}}-{{commit}}"

# Or use "v" prefix with standard format (shorthand)
zerv version --output-format semver --output-prefix
```

**Note**: Most users choose one approach, but both can be combined for advanced use cases.

## Common Use Cases

### Most Frequent Patterns

```bash
# Basic version generation (90% of users)
zerv version

# Git tags with "v" prefix (most common in Git workflows)
zerv version --output-format semver --output-prefix

# Python package versions (PEP440 compliant)
zerv version --output-format pep440

# Simple major.minor for documentation
zerv version --output-template "{{major}}.{{minor}}"

# Docker image tags
zerv version --output-template "{{major}}.{{minor}}.{{patch}}-{{commit}}"

# Release branches
zerv version --output-format semver --output-prefix "release-"
```

### CI/CD Integration

```bash
# GitHub Actions
VERSION=$(zerv version --output-format semver --output-prefix)
docker build -t myapp:$VERSION .

# GitLab CI
zerv version --output-format pep440 > VERSION.txt

# Jenkins
zerv version --output-template "{{major}}.{{minor}}.{{patch}}" > version.properties
```

## Benefits

### 1. **Unified Architecture**

Single command handles all version processing instead of multiple `from X` commands.

### 2. **Clear Usage Patterns**

Two main approaches: custom schemas for different data structures with standard output, or custom templates for different output formats with standard data.

### 3. **Maximum Flexibility**

RON schema support allows users to define any version structure they need, while `zerv-default` provides sensible defaults.

### 4. **Composable Operations**

Each pipeline stage can be controlled independently with clear, non-ambiguous flags.

### 5. **Clear Data Flow**

The pipeline makes it obvious what happens at each step: input → transform → output.

### 6. **Testable Design**

Each pipeline stage can be unit tested in isolation.

## Implementation Strategy

### Phase 1: Core Pipeline

- Implement basic `zerv version` with Git source
- Add RON schema parsing for `ZervFormat`
- Add `zerv-default` schema preset
- Support built-in output formats (pep440, semver, pvp)

### Phase 2: Rich Transforms

- Add version bumping operations
- Add state override flags
- Add template-based output (`--output-template`)

### Phase 3: Multiple Sources

- Add stdin input support
- Add direct string parsing
- Add specific tag parsing

## Error Handling

### Common Error Cases

```bash
# Conflicting output flags (exits with error)
zerv version --output-format pep440 --output-template "v{{major}}"
# Error: Cannot use both --output-format and --output-template
# Exit code: 1

# No Git repository found
zerv version --source git
# Error: Not a git repository (or any of the parent directories)
# Exit code: 1

# Invalid RON schema
zerv version --schema-ron '(invalid syntax)'
# Error: Invalid RON schema: expected '(' at line 1
# Exit code: 1

# Invalid template syntax
zerv version --output-template "{{invalid_var}}"
# Error: Unknown template variable: invalid_var
# Available: major, minor, patch, commit, branch, distance
# Exit code: 1

# No tags found in repository
zerv version
# Warning: No version tags found, using 0.0.0 as base
# Output: 0.0.0.post5+main.5.abc123
# Exit code: 0 (warning, not error)
```

### Error Handling Best Practices

- **Conflicting flags**: Immediate error with clear message
- **Invalid syntax**: Parse error with location information
- **Missing data**: Warnings with fallback behavior
- **System errors**: Clear error messages with suggested fixes

This design provides a clean, powerful interface that scales from simple usage to complex version manipulation workflows.
