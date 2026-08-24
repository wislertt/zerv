[![tests](https://img.shields.io/github/actions/workflow/status/wislertt/zerv/cd.yml?branch=main&label=tests&logo=github)](https://github.com/wislertt/zerv/actions/workflows/cd.yml)
[![release](https://img.shields.io/github/actions/workflow/status/wislertt/zerv/cd.yml?branch=main&label=release&logo=github)](https://github.com/wislertt/zerv/actions/workflows/cd.yml)
[![quality-gate-status](https://sonarcloud.io/api/project_badges/measure?project=wislertt_zerv&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=wislertt_zerv)
[![security-rating](https://sonarcloud.io/api/project_badges/measure?project=wislertt_zerv&metric=security_rating)](https://sonarcloud.io/summary/new_code?id=wislertt_zerv)
[![vulnerabilities](https://sonarcloud.io/api/project_badges/measure?project=wislertt_zerv&metric=vulnerabilities)](https://sonarcloud.io/summary/new_code?id=wislertt_zerv)
[![codecov](https://codecov.io/gh/wislertt/zerv/graph/badge.svg?token=549GL6LQBX)](https://codecov.io/gh/wislertt/zerv)
[![ruff](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/ruff/main/assets/badge/v2.json&color=green)](https://github.com/astral-sh/ruff)
[![ty](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/ty/main/assets/badge/v0.json&color=green)](https://github.com/astral-sh/ty)
[![crates.io](https://img.shields.io/crates/v/zerv?color=green)](https://crates.io/crates/zerv)
[![pypi](https://img.shields.io/pypi/v/zerv-version.svg?color=blue)](https://pypi.python.org/pypi/zerv-version)
[![status](https://img.shields.io/pypi/status/zerv-version)](https://pypi.python.org/pypi/zerv-version)
[![license](https://img.shields.io/pypi/l/zerv-version)](https://pypi.python.org/pypi/zerv-version)
[![downloads](https://static.pepy.tech/personalized-badge/zerv-version?period=total&units=international_system&left_color=grey&right_color=blue&left_text=pypi%20downloads)](https://pepy.tech/projects/zerv-version)
[![python](https://img.shields.io/badge/python-3.10%20%7C%203.11%20%7C%203.12%20%7C%203.13%20%7C%203.14-blue?logo=python)](https://github.com/wislertt/zerv/)

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://cdn.jsdelivr.net/gh/wislertt/zerv@main/docs/img/brand/zerv-lockup-dark.svg">
    <img src="https://cdn.jsdelivr.net/gh/wislertt/zerv@main/docs/img/brand/zerv-lockup.svg" width="360" alt="zerv logo">
  </picture>
</p>

# zerv

Dynamic versioning from git. Every commit gets its version. Keep semantic-release on main. Let zerv prerelease every other branch.

Documentation: **[zerv.wisl.dev](https://zerv.wisl.dev)**

## Why zerv?

- **Runs next to [semantic-release](https://zerv.wisl.dev/cicd/semantic-release)** - semantic-release decides releases on main; zerv versions every other build: any commit on any branch, even with uncommitted changes. No build is ever unversioned.
- **Two modes** - [`zerv flow`](https://zerv.wisl.dev/concepts/flow) automates pre-release management from Git branch patterns; [`zerv version`](https://zerv.wisl.dev/concepts/version) gives full manual control with schemas, overrides, and templates.
- **Any output format** - SemVer, PEP440, CalVer, or [Tera templates](https://zerv.wisl.dev/concepts/formats-and-templates). Generate every format your pipelines need from a single ZERV RON payload.

See how zerv compares to semantic-release, setuptools-scm, and `git describe` in [Why zerv](https://zerv.wisl.dev/getting-started/why-zerv).

## Installation

```bash
# Python (uv) - Recommended
uv tool install zerv-version

# Python (pip)
pip install zerv-version

# Rust (cargo)
cargo install zerv

# Installation script (latest, or `bash -s vX.X.X` for a specific version)
curl -sSL https://raw.githubusercontent.com/wislertt/zerv/main/scripts/install.sh | bash

# Pre-built binaries
# https://github.com/wislertt/zerv/releases
```

The PyPI wheel also exposes a [Python binding](https://zerv.wisl.dev/getting-started/installation#use-from-python): `import zerv`.

## Quick Start

**Version any Git state**: `zerv flow` generates meaningful SemVer versions from any Git state, no manual configuration required.

```bash
# The current branch determines the output
zerv flow
# → 1.0.0 (on main branch with tag v1.0.0)
# → 1.0.1-rc.1.post.3 (on release branch with pre-release tag)
# → 1.0.1-beta.1.post.3+develop.3.gf297dd0 (on develop branch)
# → 1.0.1-alpha.59394.post.1+feature.new.auth.1.g4e9af24 (on feature branch)
# → 1.0.1-alpha.17015.post.1.dev.1764382150+feature.dirty.work.1.g54c499a (on dirty feature branch)
```

<!-- Corresponding test: tests/integration_tests/flow/docs/quick_start.rs:test_quick_start_documentation_examples -->

Fan one ZERV RON payload out to every format your pipelines need in the [quickstart](https://zerv.wisl.dev/getting-started/quickstart).

## Documentation

Full documentation lives at [zerv.wisl.dev](https://zerv.wisl.dev):

- [Why zerv](https://zerv.wisl.dev/getting-started/why-zerv) - positioning vs semantic-release, setuptools-scm, `git describe`
- [Quickstart](https://zerv.wisl.dev/getting-started/quickstart) - one ZERV RON payload to every format
- [Concepts](https://zerv.wisl.dev/concepts/flow) - flow, version, schema system, formats and templates, config file
- [CI/CD](https://zerv.wisl.dev/cicd/github-actions) - GitHub Actions usage, coexisting with semantic-release
- [CLI reference](https://zerv.wisl.dev/cli/zerv) - `zerv version`, `zerv flow`, `zerv check`, `zerv render`
- [Python binding](https://zerv.wisl.dev/getting-started/installation#use-from-python) - `import zerv` from the PyPI wheel
- [Troubleshooting](https://zerv.wisl.dev/troubleshooting)

## Development

```bash
git clone https://github.com/wislertt/zerv.git
cd zerv

uv tool install bakefile

bake setup-dev        # Install pre-commit hooks and cargo-tarpaulin

bake test             # Full test suite (Docker Git + Docker tests enabled)
bake test-rust        # Rust tests only
bake test-python      # Python binding tests only
bake lint             # Formatting and clippy checks
bake docs             # Local Mintlify docs dev server
```

The project uses [uv](https://github.com/astral-sh/uv) and [mise](https://github.com/jdx/mise) for dependency and tool management.

## Contributing

Contributions are welcome. See [CLAUDE.md](/.claude/CLAUDE.md) for development guidelines, including project structure, testing conventions, and the development workflow.

## Author

Wisaroot Lertthaweedech – [wisl.dev](https://wisl.dev)

## License

Licensed under the Apache License 2.0. See [LICENSE](/LICENSE) for the full text.

The wordmark in `docs/img/brand/` uses outlined paths from [Sora](https://fonts.google.com/specimen/Sora), licensed under the [SIL Open Font License 1.1](https://openfontlicense.org/open-font-license-official-text/).
