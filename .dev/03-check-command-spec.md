# Zerv Check Command Specification

## Command Syntax

```bash
zerv check <version> [OPTIONS]
```

## Arguments

### Required Arguments

- `<version>` - Version string to validate

### Optional Arguments

- `-f, --format <FORMAT>` - Format to validate against
    - Supported values: `pep440`, `semver`
    - Default: Auto-detect (validates against both formats)

## Output Format

### Specific Format

```bash
$ zerv check 1.2.3 --format pep440
Version: 1.2.3
✓ Valid PEP440 format

$ zerv check 01.02.03 --format pep440
Version: 01.02.03
✓ Valid PEP440 format (normalized: 1.2.3)
```

### Auto-detect (Both Formats Valid)

```bash
$ zerv check 1.2.3
Version: 1.2.3
✓ Valid PEP440 format
✓ Valid SemVer format

$ zerv check 01.02.03
Version: 01.02.03
✓ Valid PEP440 format (normalized: 1.2.3)
✓ Valid SemVer format (normalized: 1.2.3)
```

### Auto-detect (Single Format Valid)

```bash
$ zerv check 1.2.3.dev0
Version: 1.2.3.dev0
✓ Valid PEP440 format

$ zerv check 1.2.3-alpha
Version: 1.2.3-alpha
✓ Valid SemVer format
```

### Error Cases

#### Invalid Version

```bash
$ zerv check invalid
✗ Invalid version: invalid
```

#### Unknown Format

```bash
$ zerv check 1.2.3 --format unknown
✗ Unknown format: unknown
Supported formats: pep440, semver
```

## Behavior

1. **Shows version once** at the top
2. **Auto-detection by default** - validates against all supported formats
3. **Shows normalized form per format** when versions differ from input
4. **Only displays valid formats** in auto-detect mode
5. **Consistent symbols**: ✓ for success, ✗ for errors
6. **Clear error messages** with supported format list

## Exit Codes

- `0` - Version is valid in at least one format
- `1` - Version is invalid or unknown format specified
