# docs/reference/ Reference Documentation

This directory contains detailed reference documentation for the Zerv project, organized by category.

## Directory Structure

```
docs/reference/
├── standards/          # Code quality and style standards
│   ├── code-style.md        # Comments, imports, test organization, line length
│   ├── constants.md         # Constants usage (MANDATORY)
│   ├── error-handling.md    # ZervError, error context standards
│   └── logging.md           # Tracing/logging standards
├── testing/            # Testing patterns and infrastructure
│   ├── overview.md          # Environment variables, running tests
│   ├── unit-tests.md        # Git testing, fixtures, flaky test prevention
│   └── integration-tests.md # TestCommand patterns, rstest usage
├── architecture/       # System architecture documentation
│   ├── pipeline.md          # Pipeline flow, versioning tiers
│   ├── modules.md           # Key modules overview
│   └── cli.md               # CLI commands and options
└── workflows/          # Development workflows
    ├── commands.md          # Bake commands
    ├── cicd.md              # CI/CD configuration
    └── github-actions-cache.md # GitHub Actions cache key patterns
```

## Usage

These reference files are imported into the main `AGENTS.md` using the `@docs/reference/...` syntax. This allows:

- **Concise main file**: AGENTS.md stays scannable (under 200 lines)
- **Organized details**: Deep documentation split by topic
- **Easy maintenance**: Update individual files without navigating huge document
- **All examples preserved**: Code examples kept for learning

## When to Update

Update these files when:

- Adding new coding standards or patterns
- Changing testing infrastructure
- Updating architecture decisions
- Modifying CI/CD workflows
- Adding new best practices

Always ensure changes are reflected in the appropriate category.

## Reference Direction Rules

**IMPORTANT**: Maintain one-way dependency to keep references clean:

✅ **Allowed**:

- `AGENTS.md` → `docs/reference/` (via imports)

❌ **NEVER**:

- `docs/reference/` → `docs/reference/` (avoid cross-references between reference docs)

**Why**: Reference docs in `docs/reference/` are permanent standards that should remain stable.
