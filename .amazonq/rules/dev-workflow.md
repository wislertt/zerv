# Development Workflow Rules

## MANDATORY: Always Check .dev First

Before performing ANY coding task, **read `.dev/00-README.md`** for complete project context and workflow.

## Essential Commands

✅ Use `make` commands (defined in `.dev/00-README.md`)
⚠️ Ensure `make lint` and `make test` always pass before committing

## Testing Strategy

- Use `make test_fast` for quick tests (no Docker needed)
- Use `make test` for full validation (requires Docker)
- If Docker isn't running, stick with `make test_fast`

## Error Handling Standards

- Use `zerv::error::ZervError` for all custom errors
- Implement proper error propagation with `?` operator
- Include context in error messages for debugging
