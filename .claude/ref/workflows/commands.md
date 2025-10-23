# Essential Commands

## Development Setup

```bash
make setup_dev  # Install pre-commit hooks and cargo-tarpaulin
```

## Testing

```bash
make test       # Full test suite: Docker Git + Docker tests enabled (full coverage)
make test_flaky # Run 5 iterations to detect flaky tests
```

## Code Quality

```bash
make lint       # Check code formatting and clippy warnings
make update     # Update Rust toolchain and dependencies
```

## Slash Commands

```bash
/audit-all      # Run comprehensive code quality audit (comments, imports, constants, etc.)
```
