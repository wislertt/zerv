# Code Quality Audit & Fix

Fast, reliable audit and fix workflow for any code files.

## Usage

```bash
/audit                           # Audit uncommitted files only (default)
/audit check the cli module      # NLP: Audit src/cli/ directory
/audit review main file          # NLP: Audit src/main.rs or src/lib.rs
/audit look at tests and cli     # NLP: Audit tests/ and src/cli/
```

## Scripts

**Primary:**

- `audit.sh` - Complete audit with colored output and exit codes

**Quick utilities:**

- `quick.sh` - Just show long lines (no colors, minimal output)
- `summary.sh` - File and violation counts

## Audit Workflow

**When you run `/audit` command:**

### 1. Pre-flight checks (REQUIRED) 🔴

```bash
make lint test            # Ensure code builds and tests pass
```

⚠️ **If this fails, stop here and fix lint/test errors first!**
The audit cannot proceed if the code doesn't build or tests don't pass.

### 2. Extract target paths 🎯

**From user request (NLP):**

- "cli" → `src/cli/` directory
- "tests" → `tests/` directory
- "main" → `src/main.*` files
- "config" → files with "config" in name
- "docs" → `*.md`, `docs/` directory
- "scripts" → `*.sh`, `scripts/` directory
- No paths specified → Use uncommitted files:

```bash
# Get all uncommitted files
git status --porcelain | sed 's/^[[:space:]]*[AMD?]//'
```

### 3. Detect violations 📋

**Auto-detected by scripts:**

- Long lines (>100 chars)
- Bad comment patterns
- Inline imports in functions

```bash
./audit/quick.sh          # Fast: Show long lines
./audit/audit.sh          # Full: All violations + suggestions
```

**Manual checks required:**

- Constants usage (bare strings vs constants)
- Error handling patterns
- Code reuse violations
- Test organization

### 4. Fix violations manually 🔧

- Break long rstest attributes across lines
- Split long command strings (NOT raw strings)
- Extract complex strings to variables
- Check constants usage in related files
- Verify error handling patterns

### 5. Verify fixes ✅

```bash
./audit/summary.sh        # Check remaining violations
make lint test            # Ensure fixes don't break functionality
```

### NLP Path Examples 🗣️

**Natural language requests:**

- "audit the cli code" → `src/cli/`
- "check documentation" → `*.md`, `docs/`
- "review config files" → Files matching `*config*`
- "audit scripts" → `*.sh`, `scripts/`
- "check workflows" → `.github/workflows/`
- "audit everything" → Entire repository
- No specific mention → Uncommitted files only

## Common Fixes

**Long rstest attributes:**

```rust
// ❌ BAD (101+ chars)
#[case::template_basic(1672531200, "--output-template {{bumped_timestamp}}", |output: &str, timestamp: i64| {

// ✅ GOOD (break across lines)
#[case::template_basic(
    1672531200,
    "--output-template {{bumped_timestamp}}",
    |output: &str, timestamp: i64| {
        output == timestamp.to_string()
    }
)]
```

**Long command strings:**

```rust
// ❌ BAD
"version --source stdin --tag-version 5.0.0 --input-format semver --output-format semver"

// ✅ GOOD
"version --source stdin --tag-version 5.0.0 \
 --input-format semver --output-format semver"
```

## Manual Audit Checklist

### 1. Comment Policy Violations

- Function name restatements
- Inline obvious comments
- Section divider comments

### 2. Import Statement Violations

- Inline imports in functions

### 3. Test Organization Violations

- Comment-based grouping instead of mod blocks

### 4. Constants Usage Violations

- Bare string literals for fields/formats/sources

### 5. Error Handling Violations

- unwrap()/expect() in production
- Generic error messages

### 6. Code Reuse Violations

- Direct Git implementation usage
- Custom test utilities

### 7. Long Line Violations (>100 chars)

**Common Patterns & Fixes:**

**Long rstest attributes:**

```rust
// ❌ BAD (101+ chars)
#[case::template_basic(1672531200, "--output-template {{bumped_timestamp}}", |output: &str, timestamp: i64| {

// ✅ GOOD (break across lines)
#[case::template_basic(
    1672531200,
    "--output-template {{bumped_timestamp}}",
    |output: &str, timestamp: i64| {
        output == timestamp.to_string()
    }
)]
```

**Long command strings:**

```rust
// ❌ BAD (101+ chars)
"version --source stdin --tag-version 5.0.0 --input-format semver --output-format semver"

// ✅ GOOD (use format! or break across lines)
let cmd = format!(
    "version --source stdin --tag-version 5.0.0 \
     --input-format semver --output-format semver"
);
// OR
"version --source stdin --tag-version 5.0.0 \
 --input-format semver --output-format semver"
```

**Complex template strings:**

```rust
// ❌ BAD (155 chars)
r#"--output-template "{{format_timestamp bumped_timestamp format=\"compact_date\"}}""#

// ✅ GOOD (extract to variable)
let template = r#"--output-template "{{format_timestamp bumped_timestamp format=\"compact_date\"}}""#;
```
