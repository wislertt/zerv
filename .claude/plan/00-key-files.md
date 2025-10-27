# Key Files for Development

## Core Implementation

- `src/cli/app.rs` - CLI interface (needs implementation)
- `src/pipeline/mod.rs` - Data transformation pipeline
- `src/vcs/git.rs` - Git VCS implementation
- `src/version/mod.rs` - Version format support

## Configuration & Testing

- `src/config.rs` - Environment variable management
- `src/test_utils/git/` - Docker/Native Git testing
- `Makefile` - Build commands (`test_easy`, `test`, `lint`)
- `.github/workflows/ci-test.yml` - Multi-platform CI

## Critical Patterns

- **Git Testing**: Use `get_git_impl()` for environment-aware Git ops
- **Docker Tests**: Always check `should_run_docker_tests()` first
- **Error Handling**: Use `ZervError` with `io::Error::other()` pattern
- **Config Loading**: Centralized in `ZervConfig::load()`

## Make Commands

```bash
make test_easy  # Quick (no Docker tests)
make test       # Full (with Docker tests)
make lint       # All checks
```
