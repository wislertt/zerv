# Development Workflow Rules

## MANDATORY: Always Check .dev First

Before performing ANY coding task, **read `.dev/00-README.md`** for complete project context and workflow.

## Essential Commands

✅ Use `make` commands (defined in `.dev/00-README.md`)
⚠️ Ensure `make lint` and `make test` always pass before committing

## Testing Strategy

**For Amazon Q (AI Assistant):**

- ALWAYS use `make lint` and `make test` for validation

## Error Handling Standards

- Use `zerv::error::ZervError` for all custom errors
- Implement proper error propagation with `?` operator
- Include context in error messages for debugging
