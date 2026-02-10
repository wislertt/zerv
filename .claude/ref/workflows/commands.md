# Essential Commands

## Development Setup

```bash
bake setup-dev  # Install pre-commit hooks and cargo-tarpaulin
```

## Testing

```bash
bake test       # Full test suite: Docker Git + Docker tests enabled (full coverage)
```

**Coverage Analysis**: See @.claude/ref/workflows/coverage.md for detailed coverage workflow

## Code Quality

```bash
bake lint       # Check code formatting and clippy warnings
bake update     # Update Rust toolchain and dependencies
```

## Slash Commands

```bash
/audit          # Run code quality audit and fix violations for uncommitted files
```
