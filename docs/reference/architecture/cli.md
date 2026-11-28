# CLI Implementation Standards

## Core Commands

- `zerv version [OPTIONS]` - Main version processing pipeline
- `zerv check <version> [OPTIONS]` - Validation-only command

## Essential CLI Options

**Input Sources:**

- `--source git` (default) - Auto-detect Git
- `--source stdin` - Read version from stdin

**Schema Control:**

- `--schema zerv-default` (default) - Tier-aware schema
- `--schema-ron <ron>` - Custom RON schema

**Output Control:**

- `--output-format <format>` - Target format: pep440, semver
- `--output-template <template>` - Custom template string
- `--output-prefix [prefix]` - Add prefix (defaults to "v")
