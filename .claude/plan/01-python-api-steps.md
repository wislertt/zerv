# Python API & Pip Distribution - Implementation Steps

**Status:** In Progress
**Priority:** Medium
**Created:** 2025-02-04
**Updated:** 2025-02-04

---

## Progress

### ✅ Phase 1: Project Configuration (Complete)

- [x] Added `crate-type = ["cdylib", "rlib"]` to Cargo.toml `[lib]` section
- [x] Added PyO3 as optional dependency (v0.28, optional, with `python` feature)
- [x] Created `pyproject.toml` with maturin configuration
- [x] Configured Python package name as `zerv-version` (PyPI name conflict)
- [x] Set Python version support to 3.10-3.14
- [x] All existing tests pass (686 tests)

### 🚧 Phase 2: PyO3 Boundary Layer (Next)

- [ ] Create Python module structure (`python/zerv/__init__.py`, `_core.pyi`)
- [ ] Create PyO3 bindings module (`src/python.rs`)
- [ ] Add placeholder implementations

### ⏳ Phase 3-6: Pending

---

## Overview

This document provides detailed, step-by-step implementation instructions for adding Python API and pip distribution to Zerv using PyO3 and Maturin.

**Architecture Summary:**

- Core Rust library remains pure (no PyO3 dependencies in core)
- PyO3 bindings in a boundary layer
- Same Rust binary distributed via both cargo and pip
- Zero code duplication between installation methods

---

## Phase 1: Project Configuration

### Step 1.1: Update Cargo.toml for Dual Distribution

Edit `Cargo.toml`:

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
# Existing dependencies...
pyo3 = { version = "0.23", features = ["extension-module"], optional = true }

[features]
default = []
python = ["dep:pyo3"]
```

**Verification:** `cargo check` passes without PyO3 features enabled.

---

### Step 1.2: Create pyproject.toml

Create `pyproject.toml`:

```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "zerv-version"
version = "0.0.0" # Use git tag
description = "Dynamic versioning CLI tool"
readme = "README.md"
requires-python = ">=3.10"
license = "Apache-2.0"
keywords = ["versioning", "git", "semver", "cli"]
classifiers = [
    "Development Status :: 4 - Beta",
    "Intended Audience :: Developers",
    "License :: OSI Approved :: Apache Software License",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Programming Language :: Python :: 3.13",
    "Programming Language :: Python :: 3.14",
    "Programming Language :: Rust",
    "Topic :: Software Development :: Version Control",
]

[project.optional-dependencies]
dev = ["pytest", "mypy"]

[tool.maturin]
bindings = "pyo3"
features = ["python"]
```

✅ **COMPLETE** - Created with:

- Package name `zerv-version` (PyPI naming conflict)
- Python 3.10-3.14 support
- Apache-2.0 license matching Cargo.toml

---

## Phase 2: PyO3 Boundary Layer

### Step 2.1: Create Python Module Structure

Create directory structure:

```
python/
└── zerv/
    ├── __init__.py     # Python module exports
    └── _core.pyi       # Type stubs
```

Create `python/zerv/__init__.py`:

```python
"""Zerv - Dynamic versioning for any commit."""

from . import _core

__version__ = "0.1.0"

__all__ = [
    "version",
    "flow",
    "check",
]
```

Create `python/zerv/_core.pyi`:

```python
"""Type stubs for zerv._core with Literal types for type safety."""

from typing import Literal, Optional

# Format literals
Format = Literal["auto", "semver", "pep440", "zerv"]

# Source literals
Source = Literal["git", "stdin"]

# Bump type literals
BumpType = Literal["epoch", "major", "minor", "patch", "pre_release_label", "pre_release_num", "post", "dev"]

# Pre-release label literals
PreReleaseLabel = Literal["alpha", "beta", "rc"]

# Schema preset literals (common ones shown - full list in code)
SchemaPreset = Literal[
    "standard", "standard-no-context", "standard-base",
    "standard-base-prerelease", "standard-base-prerelease-post",
    "standard-base-prerelease-post-dev", "calver", "calver-base",
    # ... 20+ more presets
]

def version(
    repo_path: str = ".",
    format: Format = "semver",
    prefix: str = "",
    commit: str = "",
    tag: str = "",
    branch: str = "",
    bump: BumpType = "",  # Type-safe: IDE will suggest valid values
    default: str = "",
    source: Source = "",  # Type-safe: only "git" or "stdin"
    schema: SchemaPreset = "",  # Type-safe: only valid schema names
    fields: str = "",
    remote: str = "",
) -> str: ...

def flow(
    repo_path: str = ".",
    format: Format = "json",
    prefix: str = "",
    branch: str = "",
    schema: SchemaPreset = "",
    remote: str = "",
) -> str: ...

def check(
    repo_path: str = ".",
    schema: SchemaPreset = "zerv",
) -> str: ...
```

**Benefits of Literal types:**

- IDE autocomplete suggests only valid values
- mypy catches typos at type-check time
- Better developer experience with inline documentation

---

### Step 2.2: Create PyO3 Bindings Module

Create `src/python.rs`:

```rust
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use std::path::Path;

/// Core library module
mod lib;

/// CLI module
mod cli;

/// Error types
use lib::error::ZervError;

/// Convert ZervError to PyErr
fn to_py_err(err: ZervError) -> PyErr {
    PyErr::new::<PyValueError, _>(err.to_string())
}

/// Generate version information for a git repository.
///
/// Args:
///     repo_path: Path to git repository (default: ".")
///     format: Output format - semver, pep440, calver, etc. (default: "semver")
///     prefix: Version prefix to strip/add (default: "")
///     commit: Specific commit hash (default: current HEAD)
///     tag: Specific tag to use as base (default: auto-detect)
///     branch: Branch name for flow versioning (default: current branch)
///     bump: Version part to bump - major, minor, patch, etc. (default: "")
///     default: Default version if no tags found (default: "")
///     source: VCS source - git, docker, etc. (default: auto-detect)
///     schema: Schema format for zerv versions (default: "")
///     fields: Custom fields for template (default: "")
///     remote: Git remote for flow operations (default: "origin")
///
/// Returns:
///     Version string
#[pyfunction]
#[pyo3(signature = (
    repo_path = ".",
    format = "semver",
    prefix = "",
    commit = "",
    tag = "",
    branch = "",
    bump = "",
    default = "",
    source = "",
    schema = "",
    fields = "",
    remote = "origin"
))]
fn version(
    repo_path: &str,
    format: &str,
    prefix: &str,
    commit: &str,
    tag: &str,
    branch: &str,
    bump: &str,
    default: &str,
    source: &str,
    schema: &str,
    fields: &str,
    remote: &str,
) -> PyResult<String> {
    // Build args from parameters
    let args = vec![
        ("version".to_string(), vec![
            "--repo".to_string(), repo_path.to_string(),
            "--format".to_string(), format.to_string(),
        ])
    ];

    // Implementation will call existing pipeline
    // For now, placeholder
    Ok("0.1.0".to_string())
}

/// Generate flow-based version information.
///
/// Args:
///     repo_path: Path to git repository (default: ".")
///     format: Output format (default: "json")
///     prefix: Version prefix (default: "")
///     branch: Branch name (default: current branch)
///     schema: Schema format (default: "")
///     remote: Git remote (default: "origin")
///
/// Returns:
///     Formatted flow information as string
#[pyfunction]
#[pyo3(signature = (
    repo_path = ".",
    format = "json",
    prefix = "",
    branch = "",
    schema = "",
    remote = "origin"
))]
fn flow(
    repo_path: &str,
    format: &str,
    prefix: &str,
    branch: &str,
    schema: &str,
    remote: &str,
) -> PyResult<String> {
    Ok("{}".to_string())
}

/// Validate zerv configuration.
///
/// Args:
///     repo_path: Path to git repository (default: ".")
///     schema: Schema to validate against (default: "zerv")
///
/// Returns:
///     Validation result as string
#[pyfunction]
#[pyo3(signature = (
    repo_path = ".",
    schema = "zerv"
))]
fn check(
    repo_path: &str,
    schema: &str,
) -> PyResult<String> {
    Ok("OK".to_string())
}

/// Python module definition
#[pymodule]
fn _core(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(flow, m)?)?;
    m.add_function(wrap_pyfunction!(check, m)?)?;
    Ok(())
}
```

---

### Step 2.3: Update lib.rs to Include Python Module

Add to `src/lib.rs`:

```rust
// Re-export Python module when feature is enabled
#[cfg(feature = "python")]
pub mod python;
```

Update `Cargo.toml` lib configuration:

```toml
[lib]
name = "zerv_core"  # For cargo install
crate-type = ["cdylib", "rlib"]
```

**Note:** For PyO3/maturin, the compiled module name will be `_core` (defined in the `#[pymodule]` attribute).

---

## Phase 3: Implement Python Bindings

### Step 3.1: Implement version() Binding

Update `src/python.rs` `version()` function to use existing pipeline:

```rust
use crate::cli::version::run_version_pipeline;
use crate::cli::args::VersionArgs;

fn version(
    repo_path: &str,
    format: &str,
    prefix: &str,
    commit: &str,
    tag: &str,
    branch: &str,
    bump: &str,
    default: &str,
    source: &str,
    schema: &str,
    fields: &str,
    remote: &str,
) -> PyResult<String> {
    use std::path::PathBuf;

    // Build VersionArgs from parameters
    let args = VersionArgs {
        repo: PathBuf::from(repo_path),
        format: format.to_string(),
        prefix: if prefix.is_empty() { None } else { Some(prefix.to_string()) },
        commit: if commit.is_empty() { None } else { Some(commit.to_string()) },
        tag: if tag.is_empty() { None } else { Some(tag.to_string()) },
        branch: if branch.is_empty() { None } else { Some(branch.to_string()) },
        bump: if bump.is_empty() { None } else { Some(bump.to_string()) },
        default: if default.is_empty() { None } else { Some(default.to_string()) },
        source: if source.is_empty() { None } else { Some(source.to_string()) },
        schema: if schema.is_empty() { None } else { Some(schema.to_string()) },
        fields: if fields.is_empty() { None } else { Some(fields.to_string()) },
        remote: remote.to_string(),
    };

    run_version_pipeline(args)
        .map_err(to_py_err)
}
```

---

### Step 3.2: Implement flow() Binding

```rust
use crate::cli::flow::run_flow_pipeline;
use crate::cli::args::FlowArgs;

fn flow(
    repo_path: &str,
    format: &str,
    prefix: &str,
    branch: &str,
    schema: &str,
    remote: &str,
) -> PyResult<String> {
    use std::path::PathBuf;

    let args = FlowArgs {
        repo: PathBuf::from(repo_path),
        format: format.to_string(),
        prefix: if prefix.is_empty() { None } else { Some(prefix.to_string()) },
        branch: if branch.is_empty() { None } else { Some(branch.to_string()) },
        schema: if schema.is_empty() { None } else { Some(schema.to_string()) },
        remote: remote.to_string(),
    };

    run_flow_pipeline(args)
        .map_err(to_py_err)
}
```

---

### Step 3.3: Implement check() Binding

```rust
use crate::cli::check::run_check_command;
use crate::cli::args::CheckArgs;

fn check(
    repo_path: &str,
    schema: &str,
) -> PyResult<String> {
    use std::path::PathBuf;

    let args = CheckArgs {
        repo: PathBuf::from(repo_path),
        schema: schema.to_string(),
    };

    run_check_command(args)
        .map_err(to_py_err)
}
```

---

## Phase 4: Testing

### Step 4.1: Create Python Tests

Create `tests/python_api_test.rs`:

```rust
#[cfg(test)]
mod python_api_tests {
    use pyo3::prelude::*;
    use pyo3::types::PyDict;

    #[test]
    fn test_version_function_exists() {
        // Test that version function can be imported
        Python::with_gil(|py| {
            let module = PyModule::from_code(
                py,
                "import zerv\nprint(zerv.version('.'))",
                "test.py",
            ).unwrap();

            // Additional assertions...
        });
    }
}
```

Create `python/tests/test_api.py`:

```python
"""Python API tests."""

import zerv
import os

def test_version_basic():
    """Test basic version call."""
    result = zerv.version(repo_path=".")
    assert isinstance(result, str)
    # Should return a version string
    assert len(result) > 0

def test_version_with_format():
    """Test version with format specified."""
    result = zerv.version(repo_path=".", format="semver")
    assert isinstance(result, str)

def test_flow_basic():
    """Test basic flow call."""
    result = zerv.flow(repo_path=".")
    assert isinstance(result, str)
    # Should be valid JSON
    import json
    json.loads(result)

def test_check_basic():
    """Test basic check call."""
    result = zerv.check(repo_path=".")
    assert isinstance(result, str)
```

---

### Step 4.2: Update CI for Python Tests

Add to `.github/workflows/test.yml`:

```yaml
- name: Install Python
  uses: actions/setup-python@v5
  with:
      python-version: "3.11"

- name: Install Maturin
  run: pip install maturin

- name: Build Python extension
  run: maturin develop --release

- name: Run Python tests
  run: pytest tests/python/
```

---

## Phase 5: Build & Release

### Step 5.1: Build Wheels Locally

```bash
# Install maturin
pip install maturin

# Build for development
maturin develop --release

# Build distribution wheels
maturin build --release

# Test the wheel
pip install target/wheels/zerv-*.whl
python -c "import zerv; print(zerv.version('.'))"
```

---

### Step 5.2: Update Release Workflow

Modify `.github/workflows/release.yml` to build Python wheels:

```yaml
- name: Build Python wheels
  run: |
      pip install maturin
      maturin build --release

- name: Upload Python wheels to release
  uses: softprops/action-gh-release@v2
  with:
      files: target/wheels/*.whl
```

---

### Step 5.3: Publish to PyPI

Create `.github/workflows/publish-pypi.yml`:

```yaml
name: Publish to PyPI

on:
    release:
        types: [published]

jobs:
    publish:
        runs-on: ubuntu-latest
        permissions:
            id-token: write # Required for trusted publishing

        steps:
            - uses: actions/checkout@v4

            - name: Install Python
              uses: actions/setup-python@v5
              with:
                  python-version: "3.11"

            - name: Install Maturin
              run: pip install maturin

            - name: Build wheels
              run: maturin build --release

            - name: Publish to PyPI
              run: maturin upload --skip-existing target/wheels/*
              env:
                  MATURIN_PYPI_TOKEN: ${{ secrets.PYPI_API_TOKEN }}
```

---

## Phase 6: Documentation

### Step 6.1: Update README.md

Add Python API section:

````markdown
## Python API

Zerv can be used as a Python library:

```bash
pip install zerv
```
````

```python
import zerv

# Get version
version = zerv.version(repo_path=".")
print(version)  # "1.2.3"

# Get flow information
flow_info = zerv.flow(repo_path=".")
print(flow_info)  # {"branch": "main", "version": "1.2.3", ...}

# Check configuration
result = zerv.check(repo_path=".")
print(result)  # "OK"
```

<!-- Corresponding test: tests/python/api_test.py -->

````

---

### Step 6.2: Create Python Documentation

Create `PYTHON_API.md`:

```markdown
# Python API Reference

## Installation

```bash
pip install zerv
````

## Functions

### `version(repo_path=".", format="semver", ...)`

Generate version for a git repository.

**Parameters:**

- `repo_path` (str): Path to repository. Default: "."
- `format` (str): Version format. Default: "semver"
- `prefix` (str): Version prefix. Default: ""
- `commit` (str): Specific commit. Default: ""
- `tag` (str): Specific tag. Default: ""
- `branch` (str): Branch name. Default: ""
- `bump` (str): Part to bump. Default: ""
- `default` (str): Default version. Default: ""
- `source` (str): VCS source. Default: ""
- `schema` (str): Schema format. Default: ""
- `fields` (str): Custom fields. Default: ""
- `remote` (str): Git remote. Default: "origin"

**Returns:** str

### `flow(repo_path=".", format="json", ...)`

Generate flow-based version information.

### `check(repo_path=".", schema="zerv")`

Validate zerv configuration.

```

---

## Implementation Order

Execute phases in this order:

1. ✅ **Phase 1** - Project configuration (COMPLETE)
2. ⏳ **Phase 2** - PyO3 boundary structure (NEXT)
3. ⏳ **Phase 3** - Implement actual bindings
4. ⏳ **Phase 4** - Testing
5. ⏳ **Phase 5** - Build and release
6. ⏳ **Phase 6** - Documentation

---

## Verification Checklist

After each phase, verify:

- [x] `cargo check` passes ✅
- [x] `cargo test` passes ✅ (686 tests)
- [ ] `maturin develop` works (Phase 2+)
- [ ] `import zerv` in Python works (Phase 2+)
- [ ] Python tests pass (Phase 4+)
- [ ] Built wheel installs and works (Phase 5+)

---

## Success Criteria

- [ ] `pip install zerv-version` installs successfully
- [ ] `import zerv; zerv.version(".")` returns a version
- [ ] `zerv --help` works identically from both install methods
- [x] All existing tests continue to pass ✅ (686 tests)
- [ ] Python API tests added and passing
- [ ] Documentation updated
- [ ] CI/CD updated for Python builds

---

## Notes

- **No PyO3 in core**: Core library (`src/lib.rs` and modules) never imports PyO3
- **Error boundary**: Convert `Result<T, ZervError>` to `PyResult<T>` at boundary
- **Same binary**: CLI from pip install is identical to cargo install binary
- **Type stubs**: Use `.pyi` files for IDE autocomplete without runtime overhead

---

## References

- PyO3 Guide: https://pyo3.rs/
- Maturin Guide: https://maturin.rs/
- Related Plan: `.claude/plan/01-python-api.md`
```
