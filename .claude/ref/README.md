# .claude/ref/ Reference Documentation

This directory contains detailed reference documentation for the Zerv project, organized by category.

## Directory Structure

```
.claude/ref/
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
    ├── commands.md          # Bake commands, slash commands
    └── cicd.md              # CI/CD configuration
```

## Usage

These reference files are imported into the main `CLAUDE.md` using the `@.claude/ref/...` syntax. This allows:

- **Concise main file**: CLAUDE.md stays scannable (under 200 lines)
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

- `CLAUDE.md` → `.claude/ref/` (via imports)
- `.claude/plan/` → `.claude/ref/` (plans can reference standards)

❌ **NEVER**:

- `.claude/ref/` → `.claude/plan/` (reference docs should NOT reference specific plans)
- `.claude/ref/` → `.claude/ref/` (avoid cross-references between reference docs)

**Why**: `.claude/plan/` contains temporary working documents with short lifecycle (created during planning, deleted when complete). Reference docs in `.claude/ref/` are permanent standards that should remain stable.
