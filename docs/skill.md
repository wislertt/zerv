---
name: zerv
description: Generate version strings from Git state for CI/CD. Use when any build needs a version, even if zerv is not named: producing SemVer, PEP440, or CalVer versions in a pipeline, versioning Docker image tags, replacing git describe, versioning pre-release branches, converting between version formats, or validating version strings. Covers zerv flow, zerv version, zerv check, and zerv render.
license: Apache-2.0
compatibility: Standalone binary on Linux, macOS, and Windows. Requires a Git repository for VCS-derived input; the none source and overrides work without one.
metadata:
  version: "1.0"
  docs: https://zerv.wisl.dev
  repository: https://github.com/wislertt/zerv
---

# zerv

zerv is a dynamic versioning CLI. It reads Git state (latest reachable tag, distance from it, current branch, commit hash, dirty flag) and renders a version string through a configurable schema. Every commit on every branch gets a version, including dirty working directories. It runs next to semantic-release: semantic-release owns releases on main, zerv versions every other build.

## Commands

| Command                 | Purpose                                                                                     |
| ----------------------- | ------------------------------------------------------------------------------------------- |
| `zerv flow`             | Automated pre-release management from branch patterns. No flags needed for GitFlow layouts. |
| `zerv version`          | Manual version generation with full override control.                                       |
| `zerv check <VERSION>`  | Validate a version string. Exits non-zero on invalid input.                                 |
| `zerv render <VERSION>` | Convert a version between formats without touching Git.                                     |

## Choosing between flow and version

Use `zerv flow` when branch names should decide the pre-release channel. Default GitFlow rules: `develop` and `beta/*` branches get beta pre-releases, `release/*` branches get rc with the number extracted from the branch name, everything else gets alpha.

Use `zerv version` when CI metadata should decide, or when rendering from a known version. Every detected VCS value can be overridden: `--tag-version`, `--distance`, `--dirty`, `--bumped-branch`, `--bumped-commit-hash`, `--bumped-timestamp`, plus version components `--major`, `--minor`, `--patch`, `--epoch`, `--post`, `--dev`, `--pre-release-label`, `--pre-release-num`.

## Examples

Branch decides the channel (repo tagged `v1.0.0`):

```bash
zerv flow          # on main
# → 1.0.0

zerv flow          # on develop, 3 commits past the tag
# → 1.0.1-beta.1.post.3+develop.3.gf297dd0

zerv flow          # on feature/new-auth, 1 commit past the tag
# → 1.0.1-alpha.59394.post.1+feature.new.auth.1.g4e9af24
```

Pin a CI build to event metadata instead of the checked-out ref:

```bash
zerv flow --bumped-branch "$BRANCH_NAME" --bumped-commit-hash "g$COMMIT_HASH"
```

Fan one resolved version out to every format a pipeline needs:

```bash
zerv version --output-format zerv | zerv version --source stdin --output-format pep440
```

Validate before feeding a strict pipeline:

```bash
zerv check 1.2.3-alpha.1
```

## Output

Three output formats: `semver`, `pep440`, and `zerv` (RON representation of the resolved version). Select with `--output-format`. Add a prefix with `--output-prefix v`, or render arbitrary strings with a Tera template: `--output-template "{{ semver_obj.docker }}"` produces a Docker-tag-safe version (`+` becomes `-`).

22 schema presets ship (11 `standard-*`, 11 `calver-*`), selected with `--schema`. Custom schemas are RON, passed with `--schema-ron`.

## Config file

Repo policy can live in `zerv.toml` (discovered upward to the repository root). CLI flags override file values. `--config-file PATH` pins a specific file; passing the null device (`/dev/null`, `NUL` on Windows) disables config discovery entirely.

## Common gotchas

- The template flag is `--output-template`. There is no `--template`.
- `--dirty` is a flag, not a value: write `zerv version --dirty`, never `--dirty true`.
- Bare `zerv` with no subcommand prints help and exits with code 2.
- Version component overrides replace values; bump flags (`--bump-major`, `--bump-patch`, `--bump-core <index>`) increment them.

## Machine-readable docs

- Full docs, single file: https://zerv.wisl.dev/llms-full.txt
- Docs index: https://zerv.wisl.dev/llms.txt
- Search MCP server: https://zerv.wisl.dev/mcp
- Install this skill into agent context: `npx skills add https://zerv.wisl.dev`
