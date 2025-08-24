# CLI Design - Pipeline Architecture

## Core Commands

### `zerv version [OPTIONS]`

Main version processing pipeline with composable operations.

### `zerv check <version> [OPTIONS]`

Validation-only command for version strings.

## Pipeline Architecture

```
Input → Version Object → Zerv Object → Transform → Output Version Object → Display
```

## Key Options

### Input Sources

```bash
--source git                        # Auto-detect Git (default)
--source string <version>           # Parse version string
```

### Schema Control

```bash
--schema zerv-default               # Zerv's opinionated schema (default)
--schema-ron <ron>                  # Custom RON schema
```

### Output Control

```bash
--output-format <format>            # Target format: pep440, semver
--output-template <template>        # Custom template string
--output-prefix [prefix]            # Add prefix (defaults to "v")
```

## Usage Examples

### Basic Cases

```bash
# Basic usage (Git → auto-format)
zerv version

# Standard format output
zerv version --output-format pep440
zerv version --output-format semver --output-prefix

# Validate version
zerv check 1.2.3

# Convert formats
zerv version --source string "1.2.3a1" --output-format semver
```

### Advanced Pipeline

```bash
# Custom template output
zerv version --output-template "v{{major}}.{{minor}}.{{patch}}-{{commit}}"

# Custom schema (CalVer)
zerv version --schema-ron '(core: [VarTimestamp("YYYY"), VarTimestamp("MM"), VarField("patch")], ...)'
```

## Default Schema (Tier-Aware)

**Tier 1** (Tagged, clean): `major.minor.patch`
**Tier 2** (Distance, clean): `major.minor.patch.post<distance>+branch.<commit>`
**Tier 3** (Dirty): `major.minor.patch.dev<timestamp>+branch.<commit>`

## Implementation Priority

1. **Basic version generation**: `zerv version` from Git
2. **Format output**: `--output-format pep440|semver`
3. **Validation**: `zerv check <version>`
4. **String input**: `--source string`
5. **Custom templates**: `--output-template`

## Advanced Features (Future)

### Version Bumping

```bash
zerv version --bump major           # 1.2.3 → 2.0.0
zerv version --bump minor           # 1.2.3 → 1.3.0
zerv version --bump patch           # 1.2.3 → 1.2.4
zerv version --bump alpha           # 1.2.3 → 1.2.4-alpha.1
```

### State Overrides

```bash
zerv version --set-distance 10      # Override commit distance
zerv version --set-dirty            # Force dirty state
zerv version --set-branch main      # Override branch name
zerv version --set-commit abc123    # Override commit hash
```

### Version Comparison

```bash
zerv compare 1.2.3 gt 1.2.2 --format semver     # exit 0 (true)
zerv compare 2.0.0a1 lt 2.0.0 --format pep440   # exit 0 (true)
```
