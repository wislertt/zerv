# Python API & Pip Distribution

**Status:** Planned
**Priority:** Medium
**Created:** 2025-02-03

---

## Context

Zerv is currently a Rust CLI tool distributed via `cargo install`. Users have requested Python API access:

```python
import zerv

zerv.version(repo_path=".")
zerv.flow(repo_path=".")
```

Additionally, users want `pip install zerv` as an alternative installation method.

---

## Goals

1. Provide Python API for programmatic access
2. Enable `pip install zerv` distribution
3. Maintain `cargo install zerv` functionality
4. Keep CLI behavior identical between installation methods
5. Zero code duplication between Rust and Python CLIs

---

## Architecture

### Distribution Channels

| Method               | Installs                   | CLI Source       | Python API    |
| -------------------- | -------------------------- | ---------------- | ------------- |
| `cargo install zerv` | Rust binary                | Rust (clap)      | N/A           |
| `pip install zerv`   | Python wheel + Rust binary | Same Rust binary | PyO3 bindings |

### Project Structure

```
zerv/
├─ Cargo.toml              # Core library + binary (cargo install)
├─ pyproject.toml          # Python package (pip install)
├─ src/
│  ├─ lib.rs               # Core library (no PyO3 concerns)
│  ├─ cli/                 # CLI logic (pure Rust)
│  └─ main.rs              # Binary entry point
└─ python/
   └─ zerv/
      ├─ __init__.py       # Python module exports
      └─ _core.pyi         # Optional typing hints
```

### Key Design Points

1. **Core library remains PyO3-free** - `src/lib.rs` and all modules have zero PyO3 dependencies

2. **Error handling at boundary** - PyO3 layer converts `Result<T, ZervError>` → `PyResult<T>`:

    ```rust
    // python/src/lib.rs
    fn version(path: &str) -> PyResult<String> {
        zerv_core::version(path)
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }
    ```

3. **CLI is pure Rust** - The `zerv` command from pip installs is the **same native Rust binary** as cargo install. Zero duplication.

4. **Python API via PyO3** - Thin boundary layer exposing core functions to Python

### Performance Comparison

| Install method | Startup path     | Overhead |
| -------------- | ---------------- | -------- |
| Cargo install  | OS → Rust binary | ~0ms     |
| Pip install    | OS → Rust binary | ~0ms     |

**Same binary, identical performance.**

---

## Cargo.toml Configuration

```toml
[package]
name = "zerv"
version = "0.1.0"
edition = "2021"

[lib]
name = "zerv_core"
crate-type = ["cdylib", "rlib"]

[[bin]]
name = "zerv"
path = "src/main.rs"

[dependencies]
pyo3 = { version = "0.20", features = ["extension-module"] }
```

---

## Python API Design (Draft)

```python
import zerv

# Version information
zerv.version(
    repo_path=".",           # Path to git repository
    format="semver",         # Version format
    prefix="",               # Version prefix
    # ... other options from CLI args
)
# Returns: str (e.g., "1.2.3")

# Flow information
zerv.flow(
    repo_path=".",
    format="json",
    # ... other options
)
# Returns: str (formatted output)

# Check configuration
zerv.check(
    repo_path=".",
    schema="zerv",
)
# Returns: str (validation result)
```

---

## Technology Stack

- **PyO3** - Rust bindings for Python
- **Maturin** - Build tool for Rust-based Python packages
- **pyproject.toml** - Modern Python packaging

Reference implementations:

- `uv` - CLI + Python API, pure Rust CLI
- `ruff` - Linter as both CLI and library

---

## pyproject.toml Configuration

```toml
[build-system]
requires = ["maturin>=1.4"]
build-backend = "maturin"

[project]
name = "zerv"
version = "0.1.0"
description = "Dynamic versioning CLI tool"
requires-python = ">=3.8"

[tool.maturin]
bindings = "pyo3"
```

---

## Installation Behavior

### Cargo Installation

```bash
cargo install zerv
```

Produces: Native Rust binary

### Pip Installation

```bash
pip install zerv
```

Produces:

- Native Rust CLI binary (identical to cargo install)
- Python extension module
- Python import support

---

## Build & Packaging

### Local Development

```bash
maturin develop
```

### Build Wheels

```bash
maturin build --release
```

---

## Distribution

### Publish to crates.io

```bash
cargo publish
```

### Publish to PyPI

```bash
twine upload dist/*
```

---

## Open Questions for Planning Phase

1. **Which specific functions from `src/` should be exposed to Python?**
    - High-level only (equivalent to CLI commands)?
    - Lower-level functions for advanced use cases?

2. **Python version support?**
    - Minimum Python version (3.9+? 3.10+?)
    - Which versions to test in CI?

3. **Error mapping strategy?**
    - Simple: all errors → `PyValueError` with message
    - Detailed: map specific `ZervError` variants to appropriate Python exceptions

4. **Type handling for complex returns?**
    - Serialize to JSON/string at boundary
    - Implement `#[pyclass]` for custom types (more complex)

5. **Build and release process?**
    - How to build wheels for multiple platforms?
    - CI/CD changes needed?
    - Publish to PyPI workflow?

---

## Success Criteria

- [ ] `pip install zerv` works
- [ ] `import zerv; zerv.version()` works
- [ ] `zerv --help` identical output from both installs
- [ ] All existing tests pass
- [ ] Python API tests added
- [ ] Documentation updated

---

## Responsibilities Breakdown

| Layer      | Responsibility           |
| ---------- | ------------------------ |
| Rust Core  | Business logic           |
| CLI Layer  | Argument parsing and UX  |
| PyO3 Layer | Python bindings          |
| Maturin    | Wheel + binary packaging |

---

## Recommended Development Workflow

1. Implement logic in Rust core
2. Use core from CLI
3. Expose selected functions via PyO3
4. Build wheels using maturin
5. Publish to both ecosystems

---

## Design Advantages

- High performance (Rust execution)
- Single implementation
- Multi-language support
- Minimal Python overhead
- Compatible with existing Rust tooling
- Zero CLI overhead for pip installs

---

## References

- Existing CLI entry: `src/cli/app.rs:run()`, `src/main.rs`
- Existing pipeline functions: `run_version_pipeline()`, `run_flow_pipeline()`, `run_check_command()`
- PyO3 guide: https://pyo3.rs/
- Maturin guide: https://maturin.rs/
