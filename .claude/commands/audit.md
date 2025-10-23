# Code Quality Audit & Fix

Fast, reliable audit and fix workflow for uncommitted Rust files.

## Usage

```bash
/audit            # Detect violations and fix them manually
```

## Scripts

**Primary:**

- `audit.sh` - Complete audit with colored output and exit codes

**Quick utilities:**

- `quick.sh` - Just show long lines (no colors, minimal output)
- `summary.sh` - File and violation counts

## Daily Workflow

```bash
# 1. Pre-flight checks (REQUIRED)
make lint test            # Ensure code builds and tests pass

# 2. Detect violations
./audit/quick.sh          # See what needs fixing
./audit/audit.sh          # Full audit with fix suggestions

# 3. Fix violations manually
# - Break long rstest attributes across lines
# - Split long command strings (NOT raw strings)
# - Extract complex strings to variables

# 4. Verify fixes
./audit/summary.sh        # Check remaining violations
make lint test            # Ensure fixes don't break functionality
```

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
