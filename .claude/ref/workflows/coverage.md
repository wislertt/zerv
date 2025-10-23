# Test Coverage Workflow

## Running Coverage Analysis

### Generate Coverage Report

```bash
make test       # Generates coverage reports in coverage/ directory
```

This command:

- Runs all tests with `cargo-tarpaulin`
- Generates reports in multiple formats:
    - `coverage/lcov.info` - LCOV format (programmatic analysis)
    - `coverage/index.html` - HTML report (visual inspection)
    - `coverage/cobertura.xml` - XML format (CI integration)
- Excludes test files from coverage (`--exclude-files '**/tests/**'`)
- Only includes `src/*` files

### Analyze Coverage Report

Use `lcov.info` for programmatic analysis:

```bash
# View overall coverage summary
grep -E "^SF:|^LH:|^LF:" coverage/lcov.info | awk '
  /^SF:/ { file=$0; sub(/^SF:/, "", file) }
  /^LH:/ { hit=$2 }
  /^LF:/ { total=$2; if(total>0) printf "%s: %.1f%% (%d/%d)\n", file, (hit/total)*100, hit, total }
'

# Find files with lowest coverage
grep -E "^SF:|^LH:|^LF:" coverage/lcov.info | awk '
  /^SF:/ { file=$0; sub(/^SF:/, "", file) }
  /^LH:/ { hit=$2 }
  /^LF:/ { total=$2; if(total>0) { pct=(hit/total)*100; printf "%.1f%% %s\n", pct, file } }
' | sort -n | head -20

# Find uncovered lines in a specific file
# Extract DA (line data) entries where execution count is 0
grep -A 1000 "^SF:src/version/semver/to_zerv.rs" coverage/lcov.info | \
  grep "^DA:" | grep ",0$" | cut -d: -f2 | cut -d, -f1
```

### Coverage Report Format (lcov.info)

The `lcov.info` file uses this format:

```
SF:<source file path>           # Source file
FN:<line>,<function name>       # Function definition
FNDA:<count>,<function name>    # Function execution count
FNF:<number>                    # Functions found
FNH:<number>                    # Functions hit
DA:<line>,<count>               # Line execution count
LH:<number>                     # Lines hit
LF:<number>                     # Lines found
end_of_record
```

Key markers for analysis:

- `SF:` - Start of file record
- `DA:` - Data for each line (line number, execution count)
    - `DA:42,0` means line 42 was NOT executed (0 times)
    - `DA:43,5` means line 43 was executed 5 times
- `LH:` - Lines hit (covered)
- `LF:` - Lines found (total executable lines)
- Coverage % = (LH / LF) \* 100

### Example Analysis

Find all uncovered lines in semver/to_zerv.rs:

```bash
# Extract file section and find lines with 0 executions
awk '
  /^SF:.*semver\/to_zerv.rs/ { in_file=1 }
  in_file && /^DA:/ && /,0$/ {
    split($0, a, ":");
    split(a[2], b, ",");
    print "Line " b[1] " is uncovered"
  }
  in_file && /^end_of_record/ { exit }
' coverage/lcov.info
```

## Coverage Goals

- **Target**: 95% overall coverage
- **Minimum**: 80% per file (exceptions for edge cases)
- **Priority**: Production code (`src/*`) over test utilities (`src/test_utils/*`)

## Best Practices

1. **Run coverage before adding tests** - Identify exact uncovered lines
2. **Focus on high-value gaps** - Complex logic > simple getters
3. **Use lcov.info for automation** - HTML for visual inspection
4. **Exclude test files** - Already done via `--exclude-files '**/tests/**'`
5. **Test edge cases** - Error paths, None/Some branches, early returns
