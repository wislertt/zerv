[![tests](https://img.shields.io/github/actions/workflow/status/wislertt/zerv/ci-test.yml?branch=main&label=tests&logo=github)](https://github.com/wislertt/zerv/actions/workflows/ci-test.yml)
[![release](https://img.shields.io/github/actions/workflow/status/wislertt/zerv/cd.yml?branch=main&label=release&logo=github)](https://github.com/wislertt/zerv/actions/workflows/cd.yml)
[![quality gate status](https://sonarcloud.io/api/project_badges/measure?project=wislertt_zerv&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=wislertt_zerv)
[![security rating](https://sonarcloud.io/api/project_badges/measure?project=wislertt_zerv&metric=security_rating)](https://sonarcloud.io/summary/new_code?id=wislertt_zerv)
[![vulnerabilities](https://sonarcloud.io/api/project_badges/measure?project=wislertt_zerv&metric=vulnerabilities)](https://sonarcloud.io/summary/new_code?id=wislertt_zerv)
[![codecov](https://img.shields.io/codecov/c/github/wislertt/zerv?token=549GL6LQBX&label=codecov&logo=codecov)](https://codecov.io/gh/wislertt/zerv)
[![crates.io](https://img.shields.io/crates/v/zerv?color=green)](https://crates.io/crates/zerv)
[![downloads](https://img.shields.io/crates/d/zerv?label=downloads&color=green)](https://crates.io/crates/zerv)

# zerv

Dynamic Versioning Cli

## 🚧 Under Active Development 🚧

**⚠️ This project is currently in active development and has not even reached alpha state. It is not usable right now and not recommended for any use.**

## Installation

### Quick Install (Recommended)

```bash
# Install latest version
curl -sSL https://raw.githubusercontent.com/wislertt/zerv/main/scripts/install.sh | bash

# Install specific version
curl -sSL https://raw.githubusercontent.com/wislertt/zerv/main/scripts/install.sh | bash -s v0.4.3

# Or using environment variable
curl -sSL https://raw.githubusercontent.com/wislertt/zerv/main/scripts/install.sh | ZERV_VERSION=v0.4.3 bash
```

### Manual Download

Download pre-built binaries from [GitHub Releases](https://github.com/wislertt/zerv/releases)

### From Source (Cargo)

```bash
cargo install zerv
```

## Uninstall

For Quick Install and Manual Download:

```bash
rm ~/.local/bin/zerv
```
