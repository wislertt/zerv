# CI/CD

## Multi-Platform Testing

- **Linux**: Native Git + Docker tests enabled
- **macOS**: Native Git + Docker tests skipped
- **Windows**: Native Git + Docker tests skipped

## Pre-commit Hooks

- Code formatting (rustfmt)
- Linting (clippy)
- Running tests

**IMPORTANT**: Ensure `make lint` and `make test` always pass before committing.

## GitHub Actions

- `ci-test.yml`: Runs tests across Linux, macOS, Windows
- `ci-pre-commit.yml`: Validates formatting and linting
- `cd.yml`: Release automation
- `security.yml`: Security scanning with SonarCloud
