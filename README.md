[![tests](https://img.shields.io/github/actions/workflow/status/wislertt/zerv/ci-test.yml?branch=main&label=tests&logo=github)](https://github.com/wislertt/zerv/actions/workflows/ci-test.yml)
[![release](https://img.shields.io/github/actions/workflow/status/wislertt/zerv/cd.yml?branch=main&label=release&logo=github)](https://github.com/wislertt/zerv/actions/workflows/cd.yml)
[![quality gate status](https://sonarcloud.io/api/project_badges/measure?project=wislertt_zerv&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=wislertt_zerv)
[![security rating](https://sonarcloud.io/api/project_badges/measure?project=wislertt_zerv&metric=security_rating)](https://sonarcloud.io/summary/new_code?id=wislertt_zerv)
[![vulnerabilities](https://sonarcloud.io/api/project_badges/measure?project=wislertt_zerv&metric=vulnerabilities)](https://sonarcloud.io/summary/new_code?id=wislertt_zerv)
[![codecov](https://img.shields.io/codecov/c/github/wislertt/zerv?token=549GL6LQBX&label=codecov&logo=codecov)](https://codecov.io/gh/wislertt/zerv)
[![crates.io](https://img.shields.io/crates/v/zerv?color=green)](https://crates.io/crates/zerv)
[![downloads](https://img.shields.io/crates/d/zerv?label=downloads&color=green)](https://crates.io/crates/zerv)

# zerv

**Automatic versioning for every commit** - Generate semantic versions from any commit across all branches, or dirty working directory, with seamless pre-release handling and flexible format support for any CI/CD workflow.

## Quick Start

```bash
# Install
cargo install zerv

# Try automated versioning (current branch determines output)
zerv flow
# → 1.0.0 (on main branch with tag v1.0.0)
# → 1.0.1-rc.1.post.3 (on release branch with pre-release tag)
# → 1.0.1-beta.1.post.3+develop.3.gf297dd0 (on develop branch)
# → 1.0.1-alpha.59394.post.1+feature.new.auth.1.g4e9af24 (on feature branch)
# → 1.0.1-alpha.17015.post.1.dev.1764382150+feature.dirty.work.1.g54c499a (on dirty feature branch)
```

<!-- Corresponding test: tests/integration_tests/flow/docs/quick_start.rs:test_quick_start_documentation_examples -->
