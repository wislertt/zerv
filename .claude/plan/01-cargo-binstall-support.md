# Plan: Add cargo binstall Support

**Status**: Completed
**Priority**: High
**Created**: 2026-02-15
**Completed**: 2026-02-15

---

## Context

Currently, users install Zerv via `cargo install zerv`, which compiles from source and is slow. `cargo binstall` can download pre-built binaries from GitHub releases, making installation much faster.

The repository already builds and publishes binaries to GitHub releases, but the asset naming doesn't follow cargo-binstall conventions, so binstall cannot find them automatically.

## Current State

**Release Assets** (v0.8.3):

- `zerv-linux-x86_64`
- `zerv-linux-arm64`
- `zerv-macos-x86_64`
- `zerv-macos-arm64`
- `zerv-windows-x86_64.exe`
- `zerv-windows-arm64.exe`

**Problem**: These names don't match cargo-binstall's expected format, which uses Rust target triples like `x86_64-unknown-linux-gnu`.

## Options Considered

### Option A: Change Asset Names to Match binstall Conventions (Recommended)

Rename release assets to include the full Rust target triple.

**Pros**:

- Follows standard conventions
- Simpler Cargo.toml configuration
- Works with more tools

**Cons**:

- Changes existing release asset naming

### Option B: Keep Current Names, Use Custom pkg-url Overrides

Configure Cargo.toml with target-specific overrides to map to current naming.

**Pros**:

- No workflow changes

**Cons**:

- More complex Cargo.toml
- Non-standard naming

## Recommended Approach: Option A

Change asset naming to `{name}-{target}` format. Cargo-binstall should auto-detect binaries with standard naming.

---

## Implementation Plan

### Step 1: Update `build-bin.yml` Workflow

Change `asset_name` in the matrix to use Rust target triples:

| Current                   | New                                |
| ------------------------- | ---------------------------------- |
| `zerv-linux-x86_64`       | `zerv-x86_64-unknown-linux-gnu`    |
| `zerv-linux-arm64`        | `zerv-aarch64-unknown-linux-gnu`   |
| `zerv-macos-x86_64`       | `zerv-x86_64-apple-darwin`         |
| `zerv-macos-arm64`        | `zerv-aarch64-apple-darwin`        |
| `zerv-windows-x86_64.exe` | `zerv-x86_64-pc-windows-msvc.exe`  |
| `zerv-windows-arm64.exe`  | `zerv-aarch64-pc-windows-msvc.exe` |

### Step 2: Update `scripts/install.sh`

Update the install script to use the new naming convention with target triples:

```bash
# Change platform/arch detection to output full target triples:
detect_target() {
    local platform=$(uname -s)
    local arch=$(uname -m)

    case "$platform" in
        Linux*)
            case "$arch" in
                x86_64|amd64) echo "x86_64-unknown-linux-gnu" ;;
                aarch64|arm64) echo "aarch64-unknown-linux-gnu" ;;
            esac ;;
        Darwin*)
            case "$arch" in
                x86_64|amd64) echo "x86_64-apple-darwin" ;;
                aarch64|arm64) echo "aarch64-apple-darwin" ;;
            esac ;;
        CYGWIN*|MINGW*|MSYS*)
            case "$arch" in
                x86_64|amd64) echo "x86_64-pc-windows-msvc" ;;
                aarch64|arm64) echo "aarch64-pc-windows-msvc" ;;
            esac ;;
        *)
            echo "Unsupported platform: $platform" >&2; exit 1 ;;
    esac
}
```

Update asset name construction to use target:

```bash
local target=$(detect_target)
local asset_name="${BINARY_NAME}-${target}"
```

### Step 3: Verify Installation

After the next release, test:

```bash
cargo binstall zerv
```

---

## Testing Strategy

1. **Local verification**: Use `cargo binstall --dry-run zerv` to verify URL resolution
2. **Post-release verification**: After merging and releasing, test actual installation
3. **Install script verification**: Test `curl -fsSL https://raw.githubusercontent.com/wislertt/zerv/main/scripts/install.sh | bash`

## Success Criteria

- `cargo binstall zerv` successfully downloads and installs the binary
- Installation completes in seconds (not minutes)
- All 6 target platforms work correctly
- Install script continues to work with new naming

## Files to Modify

1. `.github/workflows/build-bin.yml` - Update `asset_name` values in matrix to use Rust target triples
2. `scripts/install.sh` - Update to use target triple naming

## Notes

- Existing releases (v0.8.3 and earlier) will continue to work with `cargo install`
- Only new releases after this change will be binstall-compatible
- No breaking changes to the CLI or functionality
- If auto-detection fails, we can add explicit `[package.metadata.binstall]` metadata to Cargo.toml as a fallback
