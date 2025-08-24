# Development Workflow Rules

## MANDATORY: Always Check .dev First

Before performing ANY coding task, **read `.dev/00-README.md`** for complete project context and workflow.

## .dev Document Numbering

**Rule**: All .dev documents use sequential numbering to indicate creation order:

- `00-***.md`: Created at same point in time (current state)
- `01-***.md`: Next development phase
- `02-***.md`: Following phase
- etc.

**Higher numbers = more recent/updated plans**

Always verify against actual codebase - higher numbered docs are more likely to be current.

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
- Use `io::Error::other()` instead of `io::Error::new(io::ErrorKind::Other, ...)`

**Error Standard Violations Check:**

When user mentions:

- "check error standards"
- "find error violations"
- "audit error handling"
- "error compliance check"

→ Search codebase for violations:

- `io::Error::new(io::ErrorKind::Other` patterns
- Missing `ZervError` usage in custom error cases
- Direct `unwrap()` or `expect()` in production code
- Error messages without context

## Code Reuse Standards

**ALWAYS check existing utilities first:**

- Check `src/test_utils/` before creating new test utilities
- Reuse `TestDir`, `GitOperations` trait, and other existing infrastructure
- Use `get_git_impl()` for environment-aware Git operations
- Prefer `GitOperations` trait methods over direct Docker/Native calls
- Avoid duplicating code across different files
- Look for existing helper functions before implementing new ones

**Code Reuse Violations Check:**

When user mentions:

- "check code reuse"
- "find duplicated code"
- "audit code duplication"
- "redundant code check"

→ Search codebase for violations:

- Duplicated test setup patterns
- Direct `DockerGit`/`NativeGit` usage instead of `get_git_impl()`
- Reimplemented Git operations instead of using `GitOperations` trait
- Similar helper functions across files
- Unused existing utilities in `src/test_utils/git/`
