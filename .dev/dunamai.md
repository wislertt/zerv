# Dunamai Analysis for Zerv Development

## Core Architecture

### Version Class Structure

- **Base fields**: `base`, `stage`, `revision`, `distance`, `commit`, `dirty`, `tagged_metadata`, `epoch`, `branch`, `timestamp`
- **Internal fields**: `_matched_tag`, `_newer_unmatched_tags`, `_smart_bumped`
- **Metadata**: `concerns` (Set[Concern]), `vcs` (Vcs enum)

### Key Patterns

- **VERSION_SOURCE_PATTERN**: Main regex for parsing version tags
- **Pattern enum**: Default, DefaultUnprefixed with regex() method
- **Style enum**: Pep440, SemVer, Pvp for serialization formats

## VCS Implementation Strategy

### Common VCS Interface

Each VCS implements similar methods:

- Tag discovery and sorting
- Distance calculation from tag to HEAD
- Commit hash extraction (short/full)
- Dirty state detection
- Branch name extraction
- Timestamp parsing

### Git Implementation Highlights

- Archive support via `.git_archival.json`
- Git version detection for feature compatibility
- Shallow repository detection and warnings
- Tag topology ordering for proper tag selection
- Untracked file handling for dirty detection

### Archive Support

- Git: `.git_archival.json` with hash, timestamp, refs, describe
- Mercurial: `.hg_archival.txt` with latesttag, latesttagdistance, node, branch

## Version Serialization

### PEP 440 Format

- Epoch support: `1!2.0.0`
- Pre-release: `a`, `b`, `rc` with revision numbers
- Post-release: `.post1`
- Dev release: `.dev1`
- Local version: `+metadata.segments`

### SemVer Format

- Core: `major.minor.patch`
- Pre-release: `-alpha.1`
- Build metadata: `+build.metadata`

### PVP Format

- Base with dashes: `1.2.3-alpha-1-metadata`

## Pattern Matching System

### Default Pattern Features

- Optional `v` prefix
- Epoch support: `1!`
- Base version: `1.2.3`
- Stage/revision: `alpha1`, `beta.2`, `rc-3`
- Tagged metadata: `+linux.x86_64`

### Custom Pattern Support

- Must contain `?P<base>` named group
- Optional groups: `stage`, `revision`, `tagged_metadata`, `epoch`
- Validation against known presets

## Error Handling

### Concern System

- `ShallowRepository`: Warning for incomplete history
- Extensible enum for future warnings
- Strict mode converts warnings to errors

### Fallback Strategy

- Default to `0.0.0` when no tags found
- Preserve commit info and dirty state
- Configurable via strict mode

## Command Execution

### Security Considerations

- Uses `shlex.split()` for safe command parsing
- Environment variable filtering (removes GIT_TRACE\*)
- Proper error code handling
- Output sanitization

### Cross-Platform Support

- Path handling with `pathlib.Path`
- Shell detection and compatibility
- Program availability checking with `shutil.which()`

## Key Algorithms

### Tag Selection Logic

1. Get all tags from VCS
2. Sort by creation date/topology
3. Try pattern matching in order
4. Calculate distance from matched tag
5. Handle unmatched newer tags

### Distance Calculation

- Git: `git rev-list --count tag..HEAD`
- Mercurial: Count commits between tag and current
- Others: Similar commit counting approaches

### Dirty Detection

- Git: `git describe --dirty` + optional untracked check
- Mercurial: Parse `hg summary` output
- Others: VCS-specific status commands

## Template System

### Variable Extraction

- Base components: `major`, `minor`, `patch`
- Metadata: `commit`, `branch`, `timestamp`
- State: `dirty`, `distance`
- Computed: `branch_escaped`

### Format Validation

- Style-specific regex validation
- SemVer leading zero prohibition
- PEP 440 compliance checking

## Performance Considerations

### Command Optimization

- Minimal VCS command calls
- Efficient tag filtering
- Lazy evaluation where possible
- Git version-specific optimizations

### Memory Management

- Streaming command output
- Minimal object creation
- Efficient string operations

## Testing Strategy

### Archive Testing

- Pre-built test archives for each VCS
- Consistent test scenarios
- Edge case coverage

### Integration Tests

- Real VCS repository testing
- Cross-platform validation
- Version comparison testing

## Migration Notes for Zerv

### Rust Equivalents Needed

- Regex crate for pattern matching
- Chrono for timestamp handling
- Subprocess execution with proper error handling
- JSON parsing for archive metadata
- TOML for configuration files

### Performance Opportunities

- Parallel VCS detection
- Compiled regex patterns
- Zero-copy string operations
- Efficient command execution

### Safety Improvements

- Type-safe VCS enum dispatch
- Compile-time pattern validation
- Memory-safe command execution
- Structured error handling
