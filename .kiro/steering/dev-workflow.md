# Development Workflow Rules

## MANDATORY: Always Check .dev First

Before performing ANY coding task, **read `.dev/00-README.md`** for complete project context and workflow.

Reference the project documentation: #[[file:../../.dev/00-README.md]]

## .dev Document Numbering

**Rule**: All .dev documents use sequential numbering to indicate creation order:

- `00-***.md`: Created at same point in time (current state)
- `01-***.md`: Next development phase
- `02-***.md`: Following phase

**Higher numbers = more recent/updated plans**

## Essential Commands

✅ Use `make` commands (defined in `.dev/00-README.md`)
⚠️ Ensure `make lint` and `make test` always pass before committing

## Error Handling Standards

- Use `zerv::error::ZervError` for all custom errors
- Implement proper error propagation with `?` operator
- Include context in error messages for debugging
- Use `io::Error::other()` instead of `io::Error::new(io::ErrorKind::Other, ...)`

## Code Reuse Standards

**ALWAYS check existing utilities first:**

- Check `src/test_utils/` before creating new test utilities
- Reuse `TestDir`, `GitOperations` trait, and other existing infrastructure
- Use `get_git_impl()` for environment-aware Git operations
- Prefer `GitOperations` trait methods over direct Docker/Native calls
