# Command-Line Help for `zerv`

This document contains the help content for the `zerv` command-line program.

**Command Overview:**

- [`zerv`↴](#zerv)
- [`zerv version`↴](#zerv-version)
- [`zerv flow`↴](#zerv-flow)
- [`zerv check`↴](#zerv-check)

## `zerv`

Zerv is a dynamic versioning tool that generates version strings from version control system (VCS) data using configurable schemas. It supports multiple input sources, output formats, and advanced override capabilities for CI/CD workflows.

Use --llm-help to display the comprehensive CLI manual with detailed examples and guidance.

EXAMPLES:

# Basic version generation from git

zerv version

# Generate PEP440 format with custom schema

zerv version --output-format pep440 --schema calver

# Override VCS values for testing

zerv version --tag-version v2.0.0 --distance 5 --dirty true

# Force clean release state

zerv version --clean

# Pipe Zerv RON between commands

zerv version --output-format zerv | zerv version --source stdin --schema calver

# Use in different directory

zerv version -C /path/to/repo

**Usage:** `zerv [OPTIONS] [COMMAND]`

###### **Subcommands:**

- `version` — Generate version from VCS data with configurable schemas and overrides
- `flow` — Generate version with intelligent pre-release management based on Git branch patterns
- `check` — Validate version string format compliance

###### **Options:**

- `-v`, `--verbose` — Use verbose output (enables debug-level logs to stderr). Use RUST_LOG for fine-grained control (e.g., RUST_LOG=zerv::vcs=debug)
- `--llm-help` — Display comprehensive CLI manual

## `zerv version`

Generate version strings from version control system data using configurable schemas.
Supports multiple input sources (git, stdin), output formats (semver, pep440, zerv), and VCS overrides
for testing and CI/CD workflows.

**Usage:** `zerv version [OPTIONS]`

###### **Options:**

- `-s`, `--source <SOURCE>` — Input source: 'git' (extract from repository) or 'stdin' (read Zerv RON format)

    Default value: `git`

    Possible values: `git`, `stdin`

- `-f`, `--input-format <INPUT_FORMAT>` — Input format: 'auto' (detect), 'semver', or 'pep440'

    Default value: `auto`

    Possible values: `auto`, `semver`, `pep440`

- `-C`, `--directory <DIR>` — Working directory (default: current directory)
- `--output-format <OUTPUT_FORMAT>` — Output format: 'semver' (default), 'pep440', or 'zerv' (RON format for piping)

    Default value: `semver`

    Possible values: `semver`, `pep440`, `zerv`

- `--output-template <OUTPUT_TEMPLATE>` — Output template for custom formatting (Tera syntax: {{ variable }})
- `--output-prefix <OUTPUT_PREFIX>` — Prefix to add to version output (e.g., 'v' for 'v1.0.0')
- `--schema <SCHEMA>` — Schema preset name

Standard Schema Family (SemVer):
standard - Smart auto-detection based on repository state (clean/dirty/distance)
standard-base - 1.1.0
standard-base-prerelease - 1.1.0-alpha.1
standard-base-prerelease-post - 1.1.0-alpha.1.post.2
standard-base-prerelease-post-dev - 1.1.0-alpha.1.post.2.dev.1729924622
standard-base-context - 1.1.0+main.2.a1b2c3d
standard-base-prerelease-context - 1.1.0-alpha.1+main.2.a1b2c3d
standard-base-prerelease-post-context - 1.1.0-alpha.1.post.2+main.2.a1b2c3d
standard-base-prerelease-post-dev-context - 1.1.0-alpha.1.post.2.dev.1729924622+main.2.a1b2c3d
standard-context - Smart auto-detection with build context

CalVer Schema Family:
calver - Smart auto-detection based on repository state (clean/dirty/distance)
calver-base - 2024.11.03
calver-base-prerelease - 2024.11.03-alpha.1
calver-base-prerelease-post - 2024.11.03-alpha.1.post.2
calver-base-prerelease-post-dev - 2024.11.03-alpha.1.post.2.dev.1729924622
calver-base-context - 2024.11.03+main.2.a1b2c3d
calver-base-prerelease-context - 2024.11.03-alpha.1+main.2.a1b2c3d
calver-base-prerelease-post-context - 2024.11.03-alpha.1.post.2+main.2.a1b2c3d
calver-base-prerelease-post-dev-context - 2024.11.03-alpha.1.post.2.dev.1729924622+main.2.a1b2c3d
calver-context - Smart auto-detection with build context

- `--schema-ron <SCHEMA_RON>` — Custom schema in RON format
- `--tag-version <TAG_VERSION>` — Override detected tag version (e.g., 'v2.0.0', '1.5.0-beta.1')
- `--distance <DISTANCE>` — Override distance from tag (number of commits since tag)
- `--dirty` — Override dirty state to true (sets dirty=true)
- `--no-dirty` — Override dirty state to false (sets dirty=false)
- `--clean` — Force clean release state (sets distance=0, dirty=false). Conflicts with --distance and --dirty
- `--bumped-branch <BUMPED_BRANCH>` — Override current branch name
- `--bumped-commit-hash <BUMPED_COMMIT_HASH>` — Override commit hash (full or short form)
- `--bumped-timestamp <BUMPED_TIMESTAMP>` — Override commit timestamp (Unix timestamp)
- `--major <MAJOR>` — Override major version number
- `--minor <MINOR>` — Override minor version number
- `--patch <PATCH>` — Override patch version number
- `--epoch <EPOCH>` — Override epoch number
- `--post <POST>` — Override post number
- `--dev <DEV>` — Override dev number
- `--pre-release-label <PRE_RELEASE_LABEL>` — Override pre-release label (alpha, beta, rc, none, null). Supports templates like '{{{{#if dirty}}}}dev{{{{else}}}}beta{{{{/if}}}}'
- `--pre-release-num <PRE_RELEASE_NUM>` — Override pre-release number
- `--custom <CUSTOM>` — Override custom variables in JSON format
- `--core <INDEX=VALUE>` — Override core schema component by index=value (e.g., --core 0=5, --core ~1=2024, --core 1={{major}})
- `--extra-core <INDEX=VALUE>` — Override extra-core schema component by index=value (e.g., --extra-core 0=5, --extra-core ~1=beta, --extra-core 1={{branch}})
- `--build <INDEX=VALUE>` — Override build schema component by index=value (e.g., --build 0=5, --build ~1=release, --build 1={{commit_short}})
- `--bump-major <BUMP_MAJOR>` — Add to major version (default: 1)
- `--bump-minor <BUMP_MINOR>` — Add to minor version (default: 1)
- `--bump-patch <BUMP_PATCH>` — Add to patch version (default: 1)
- `--bump-post <BUMP_POST>` — Add to post number (default: 1)
- `--bump-dev <BUMP_DEV>` — Add to dev number (default: 1)
- `--bump-pre-release-num <BUMP_PRE_RELEASE_NUM>` — Add to pre-release number (default: 1)
- `--bump-epoch <BUMP_EPOCH>` — Add to epoch number (default: 1)
- `--bump-pre-release-label <BUMP_PRE_RELEASE_LABEL>` — Bump pre-release label (alpha, beta, rc, none, null) and reset number to 0. Supports templates like '{{{{#if (eq bumped_branch "release")}}}}rc{{{{else}}alpha{{{{/if}}}}'
- `--bump-core <INDEX[=VALUE]>` — Bump core schema component by index[=value] (e.g., --bump-core 0={{distance}} or --bump-core 0)
- `--bump-extra-core <INDEX[=VALUE]>` — Bump extra-core schema component by index[=value] (e.g., --bump-extra-core 0={{distance}} or --bump-extra-core 0)
- `--bump-build <INDEX[=VALUE]>` — Bump build schema component by index[=value] (e.g., --bump-build 0={{distance}} or --bump-build 0)
- `--bump-context` — Include VCS context qualifiers (default behavior)
- `--no-bump-context` — Pure tag version, no VCS context

## `zerv flow`

Generate version strings with automatic pre-release detection based on Git branch patterns.
This command acts as an intelligent wrapper around 'zerv version' that automatically determines
pre-release information from the current Git branch using configurable pattern matching.

**Usage:** `zerv flow [OPTIONS]`

###### **Options:**

- `-s`, `--source <SOURCE>` — Input source: 'git' (extract from repository) or 'stdin' (read Zerv RON format)

    Default value: `git`

    Possible values: `git`, `stdin`

- `-f`, `--input-format <INPUT_FORMAT>` — Input format: 'auto' (detect), 'semver', or 'pep440'

    Default value: `auto`

    Possible values: `auto`, `semver`, `pep440`

- `-C`, `--directory <DIR>` — Working directory (default: current directory)
- `--output-format <OUTPUT_FORMAT>` — Output format: 'semver' (default), 'pep440', or 'zerv' (RON format for piping)

    Default value: `semver`

    Possible values: `semver`, `pep440`, `zerv`

- `--output-template <OUTPUT_TEMPLATE>` — Output template for custom formatting (Tera syntax: {{ variable }})
- `--output-prefix <OUTPUT_PREFIX>` — Prefix to add to version output (e.g., 'v' for 'v1.0.0')
- `--pre-release-label <PRE_RELEASE_LABEL>` — Pre-release label for flow versions (alpha, beta, rc)

    Possible values: `alpha`, `beta`, `rc`

- `--pre-release-num <PRE_RELEASE_NUM>` — Pre-release number for flow versions (integer, default: {{ hash_int(value=bumped_branch, length=HASH_BRANCH_LEN) }})
- `--post-mode <POST_MODE>` — Post calculation mode (commit, tag)

    Possible values: `tag`, `commit`

- `--branch-rules <BRANCH_RULES>` — Branch rules in RON format (default: GitFlow rules)

    Default value: `[(pattern:"develop",pre_release_label:beta,pre_release_num:Some(1),post_mode:commit),(pattern:"release/*",pre_release_label:rc,pre_release_num:None,post_mode:tag)]`

- `--tag-version <TAG_VERSION>` — Override detected tag version (e.g., 'v2.0.0', '1.5.0-beta.1')
- `--distance <DISTANCE>` — Override distance from tag (number of commits since tag)
- `--dirty` — Override dirty state to true (sets dirty=true)
- `--no-dirty` — Override dirty state to false (sets dirty=false)
- `--clean` — Force clean release state (sets distance=0, dirty=false). Conflicts with --distance and --dirty
- `--bumped-branch <BUMPED_BRANCH>` — Override current branch name
- `--bumped-commit-hash <BUMPED_COMMIT_HASH>` — Override commit hash (full or short form)
- `--bumped-timestamp <BUMPED_TIMESTAMP>` — Override commit timestamp (Unix timestamp)
- `--major <MAJOR>` — Override major version number
- `--minor <MINOR>` — Override minor version number
- `--patch <PATCH>` — Override patch version number
- `--epoch <EPOCH>` — Override epoch number
- `--post <POST>` — Override post number
- `--hash-branch-len <HASH_BRANCH_LEN>` — Hash length for bumped branch hash (1-10, default: 5)

    Default value: `5`

- `--schema <SCHEMA>` — Schema preset name

Standard Schema Family (SemVer):
standard - Smart auto-detection based on repository state (clean/dirty/distance)
standard-base - 1.1.0
standard-base-prerelease - 1.1.0-alpha.1
standard-base-prerelease-post - 1.1.0-alpha.1.post.2
standard-base-prerelease-post-dev - 1.1.0-alpha.1.post.2.dev.1729924622
standard-base-context - 1.1.0+main.2.a1b2c3d
standard-base-prerelease-context - 1.1.0-alpha.1+main.2.a1b2c3d
standard-base-prerelease-post-context - 1.1.0-alpha.1.post.2+main.2.a1b2c3d
standard-base-prerelease-post-dev-context - 1.1.0-alpha.1.post.2.dev.1729924622+main.2.a1b2c3d
standard-context - Smart auto-detection with build context

CalVer Schema Family:
calver - Smart auto-detection based on repository state (clean/dirty/distance)
calver-base - 2024.11.03
calver-base-prerelease - 2024.11.03-alpha.1
calver-base-prerelease-post - 2024.11.03-alpha.1.post.2
calver-base-prerelease-post-dev - 2024.11.03-alpha.1.post.2.dev.1729924622
calver-base-context - 2024.11.03+main.2.a1b2c3d
calver-base-prerelease-context - 2024.11.03-alpha.1+main.2.a1b2c3d
calver-base-prerelease-post-context - 2024.11.03-alpha.1.post.2+main.2.a1b2c3d
calver-base-prerelease-post-dev-context - 2024.11.03-alpha.1.post.2.dev.1729924622+main.2.a1b2c3d
calver-context - Smart auto-detection with build context

## `zerv check`

Validate that version strings conform to specific format requirements.
Supports SemVer, PEP440, and other version format validation.

**Usage:** `zerv check [OPTIONS] <VERSION>`

###### **Arguments:**

- `<VERSION>` — Version string to validate

###### **Options:**

- `-f`, `--format <FORMAT>` — Format to validate against

<hr/>

<small><i>
This document was generated automatically by
<a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
