# .dev/

This directory contains development planning and design documents for AI-assisted coding.

## Purpose

- **Planning documents** - Project roadmaps and feature specifications
- **Design documents** - Architecture and implementation designs
- **AI context** - Documents for AI agents to understand project requirements

## Usage

Reference this directory with `@.dev` in AI conversations to provide full project context.

## Development Workflow

Use these commands during development:

- `make test` - Run tests with coverage using cargo-tarpaulin (instead of `cargo test`)
- `make lint` - Run formatting and clippy checks (instead of `cargo check`)
- `make run` - Run the CLI binary (instead of `cargo run`)
- `make setup_dev` - Install development dependencies

## Source Code Reference

Dunamai source code is available at `.cache/repos/dunamai` for reference during development.

Ripgrep source code can be referenced at [https://github.com/BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep) for installation patterns and CLI best practices.

## Files

- `planning.md` - Project planning and roadmap
- `cli-design.md` - CLI design and architecture
- `dunamai.md` - Dunamai analysis and key insights for zerv development
