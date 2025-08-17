# .dev/

Development documentation for zerv - Dynamic Versioning CLI

## Project Status

**Current State**: Version parsing system implemented
**Next Step**: Building Git VCS integration
**Target**: Alpha release with Git support

## What's Implemented

✅ **Core Version System**

- Universal `Zerv` format with component-based structure
- PEP440 version parsing, display, and conversion
- SemVer version parsing, display, and conversion
- Comprehensive test coverage (91.40%)

✅ **Development Infrastructure**

- Docker-based testing for Git integration
- Fast local tests without external dependencies
- Makefile workflow with linting and formatting

## Development Workflow

```bash
# Fast development cycle (no Docker required)
make test_fast    # Run unit tests only
make lint         # Format and check code
make run          # Test CLI binary

# Full validation (includes Docker tests)
make test         # Run all tests including Docker integration
make setup_dev    # Install development dependencies
```

## Architecture

```
src/version/
├── zerv/         # Universal version format
├── pep440/       # PEP440 implementation
├── semver/       # SemVer implementation
└── mod.rs        # Public API
```

## Files

- `current-state.md` - Detailed implementation status
- `new-cli-design.md` - Pipeline CLI architecture design
- `roadmap.md` - Development roadmap to alpha release
- `archived-insights.md` - Valuable concepts from old documentation
- `.cache/` - Archived old documentation (preserved for reference)

## Next Phase

Git VCS integration + pipeline CLI (`zerv version` with `zerv-default` schema)
