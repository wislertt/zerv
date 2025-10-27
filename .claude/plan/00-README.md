# Zerv Development Guide

## Quick Start

1. **Check current state**: Read `00-current-state.md`
2. **Key files**: Reference `00-key-files.md`
3. **Run tests**: `make test_easy` (quick) or `make test` (full)
4. **Check rules**: `.amazonq/rules/` for coding standards

## Current Priority

**CLI Implementation** - Core functionality exists, need CLI interface.

## Architecture Overview

```
CLI → VCS Detection → Tag Parsing → Format Output
```

**Key Components:**

- **VCS Layer**: Extract git metadata (`src/vcs/`)
- **Version System**: Parse/convert formats (`src/version/`)
- **Pipeline**: Transform data (`src/pipeline/`)
- **CLI**: User interface (`src/cli/` - needs work)

## Testing Strategy

- **Local**: Docker Git for isolation
- **CI**: Native Git for platform testing
- **237 tests** with multi-platform coverage

## Environment Variables

- `ZERV_TEST_NATIVE_GIT=true` - Use native Git (CI only)
- `ZERV_TEST_DOCKER=true` - Enable Docker tests (Linux only)

## Next Steps

Focus on `src/cli/app.rs` implementation to wire existing components together.
