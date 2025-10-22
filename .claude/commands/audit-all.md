# Code Quality Audit

Run a comprehensive code quality audit checking for violations of project standards defined in CLAUDE.md.

## Audit Checklist

### 1. Comment Policy Violations

Search for bad comment patterns and report violations with file:line_number:

**Function name restatements** (search for these patterns in doc comments):

- `/// Converts` (if function name contains "convert")
- `/// Processes` (if function name contains "process")
- `/// Returns` (if function name is getter-like)
- `/// Checks if` (if function name starts with "is*" or "has*")
- `/// Gets` or `/// Sets` (if function name starts with "get*" or "set*")

**Inline obvious comments** (search for):

- `// Initialize`
- `// Create`
- `// Return`
- `// Calculate the`
- `// Format the`
- Inline comments that just repeat the line of code

**Section divider comments** (search for):

- `// ====`
- `// ----`
- `// ************`
- Pattern: `// [repeated characters]`

### 2. Import Statement Violations

Search for `use` statements inside function bodies (bad pattern):

- Pattern: `fn.*\{[^}]*use\s+.*::`
- Look for test functions with inline imports
- Exclude the exception case: `use X as Y` for intentional scope limitation

Report any inline imports found and suggest moving them to the top of the file or module.

### 3. Test Organization Violations

Search for comment-based test grouping instead of mod blocks:

- Pattern: `// ={10,}` followed by test description
- Pattern: `// -+.*Test`
- Look for tests grouped by comments rather than mod blocks

Suggest converting to module-based organization with `mod test_group { use super::*; ... }`

### 4. Constants Usage Violations

Search for bare string literals that should use constants:

**Field names** (search in match statements and comparisons):

- `"major"`, `"minor"`, `"patch"` (should use `fields::MAJOR`, etc.)
- `"epoch"`, `"pre_release"`, `"post"`, `"dev"`
- `"distance"`, `"dirty"`
- `"bumped_branch"`, `"last_commit_hash"`, etc.

**Format names**:

- `"semver"`, `"pep440"`, `"zerv"`, `"auto"` (should use `formats::*`)

**Source names**:

- `"git"`, `"stdin"` (should use `sources::*`)

**Schema names**:

- `"zerv-standard"`, `"zerv-calver"` (should use `schema_names::*`)

### 5. Error Handling Violations

Search for old error patterns:

- `io::Error::new(io::ErrorKind::Other` (should use `io::Error::other()`)
- Production code with `unwrap()` (exclude test files)
- Production code with `expect()` (exclude test files)
- Generic error messages without context

### 6. Code Reuse Violations

Search for duplicated patterns that should use existing utilities:

- Direct `DockerGit::new()` or `NativeGit::new()` usage (should use `get_git_impl()`)
- Custom test directory logic (should use `TestDir`)
- Custom Git fixture setup (should use `GitRepoFixture`)
- Duplicated test setup code across files

## Reporting Format

For each violation found, report:

```
[CATEGORY] file_path:line_number
Found: <actual code>
Should be: <suggested fix>
```

Group violations by category and provide a summary count at the end:

```
Summary:
- Comment violations: X
- Import violations: X
- Test organization: X
- Constants violations: X
- Error handling: X
- Code reuse: X

Total violations: X
```

## Instructions

1. Use Grep tool with appropriate patterns for each check
2. Read relevant files to confirm violations (avoid false positives)
3. Provide specific line numbers and context
4. Suggest concrete fixes for each violation
5. Prioritize violations by severity (error handling > constants > comments)

Focus on src/ directory and tests/ directory. Skip target/, .git/, and other build artifacts.
