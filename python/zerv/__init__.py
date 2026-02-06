from __future__ import annotations

import subprocess
from typing import Any, Literal

from zerv._find_zerv import find_zerv_bin
from importlib.metadata import PackageNotFoundError, version as _version


def _get_version() -> str:
    try:
        return _version("zerv-version")
    except PackageNotFoundError:
        return "0.0.0"


__version__ = _get_version()
__all__ = ["find_zerv_bin", "version", "flow", "check"]

# Format literals
Format = Literal["auto", "semver", "pep440", "zerv"]

# Source literals
Source = Literal["git", "stdin"]

# Bump type literals
BumpType = Literal[
    "epoch",
    "major",
    "minor",
    "patch",
    "pre_release_label",
    "pre_release_num",
    "post",
    "dev",
]

# Schema preset literals
StandardSchema = Literal[
    "standard",
    "standard-no-context",
    "standard-base",
    "standard-base-prerelease",
    "standard-base-prerelease-post",
    "standard-base-prerelease-post-dev",
    "standard-base-context",
    "standard-base-prerelease-context",
    "standard-base-prerelease-post-context",
    "standard-base-prerelease-post-dev-context",
    "standard-context",
]

CalverSchema = Literal[
    "calver",
    "calver-no-context",
    "calver-base",
    "calver-base-prerelease",
    "calver-base-prerelease-post",
    "calver-base-prerelease-post-dev",
    "calver-base-context",
    "calver-base-prerelease-context",
    "calver-base-prerelease-post-context",
    "calver-base-prerelease-post-dev-context",
    "calver-context",
]

# Combined schema preset for version command
SchemaPreset = StandardSchema | CalverSchema

# Check format literals
CheckFormat = Literal["semver", "pep440"]

# Flow-specific literals
FlowInputFormat = Literal["auto", "semver", "pep440"]
FlowOutputFormat = Literal["semver", "pep440", "zerv"]
FlowPreReleaseLabel = Literal["alpha", "beta", "rc"]
FlowPostMode = Literal["tag", "commit"]


def _extend_args(args: list[str], flags: list[tuple[str, Any]]) -> list[str]:
    for flag, value in flags:
        if value is None or value is False:
            continue

        args.append(flag)
        if not isinstance(value, bool):
            args.append(str(value))

    return args


def _run_zerv_command(args: list[str]) -> str:
    """Run the zerv CLI binary and return stdout."""
    zerv_bin = find_zerv_bin()
    result = subprocess.run(
        [zerv_bin, *args],
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(f"zerv command failed: {result.stderr}")
    return result.stdout.strip()


def version(
    *,
    # Input options
    source: Source | None = None,
    input_format: Format | None = None,
    repo_path: str | None = None,
    # Output options
    output_format: Format | None = None,
    output_template: str | None = None,
    output_prefix: str | None = None,
    # Schema options
    schema: SchemaPreset | None = None,
    schema_ron: str | None = None,
    # VCS override options
    tag_version: str | None = None,
    distance: int | None = None,
    dirty: bool | None = None,
    no_dirty: bool | None = None,
    clean: bool | None = None,
    bumped_branch: str | None = None,
    bumped_commit_hash: str | None = None,
    bumped_timestamp: int | None = None,
    # Version component override options
    major: int | None = None,
    minor: int | None = None,
    patch: int | None = None,
    epoch: int | None = None,
    post: int | None = None,
    # Version-specific override options
    dev: int | None = None,
    pre_release_label: str | None = None,
    pre_release_num: int | None = None,
    custom: str | None = None,
    # Schema component override options
    core: str | None = None,
    extra_core: str | None = None,
    build: str | None = None,
    # Field-based bump options
    bump_major: int | None = None,
    bump_minor: int | None = None,
    bump_patch: int | None = None,
    bump_post: int | None = None,
    bump_dev: int | None = None,
    bump_pre_release_num: int | None = None,
    bump_epoch: int | None = None,
    bump_pre_release_label: str | None = None,
    # Schema-based bump options
    bump_core: str | None = None,
    bump_extra_core: str | None = None,
    bump_build: str | None = None,
    # Context control options
    bump_context: bool | None = None,
    no_bump_context: bool | None = None,
) -> str:
    return _run_zerv_command(
        args=_extend_args(
            args=["version"],
            flags=[
                # Input options
                ("-s", source),
                ("-f", input_format),
                ("-C", repo_path),
                # Output options
                ("--output-format", output_format),
                ("--output-template", output_template),
                ("--output-prefix", output_prefix),
                # Schema options
                ("--schema", schema),
                ("--schema-ron", schema_ron),
                # VCS override options
                ("--tag-version", tag_version),
                ("--distance", distance),
                ("--dirty", dirty),
                ("--no-dirty", no_dirty),
                ("--clean", clean),
                ("--bumped-branch", bumped_branch),
                ("--bumped-commit-hash", bumped_commit_hash),
                ("--bumped-timestamp", bumped_timestamp),
                # Version component override options
                ("--major", major),
                ("--minor", minor),
                ("--patch", patch),
                ("--epoch", epoch),
                ("--post", post),
                # Version-specific override options
                ("--dev", dev),
                ("--pre-release-label", pre_release_label),
                ("--pre-release-num", pre_release_num),
                ("--custom", custom),
                # Schema component override options
                ("--core", core),
                ("--extra-core", extra_core),
                ("--build", build),
                # Field-based bump options
                ("--bump-major", bump_major),
                ("--bump-minor", bump_minor),
                ("--bump-patch", bump_patch),
                ("--bump-post", bump_post),
                ("--bump-dev", bump_dev),
                ("--bump-pre-release-num", bump_pre_release_num),
                ("--bump-epoch", bump_epoch),
                ("--bump-pre-release-label", bump_pre_release_label),
                # Schema-based bump options
                ("--bump-core", bump_core),
                ("--bump-extra-core", bump_extra_core),
                ("--bump-build", bump_build),
                # Context control options
                ("--bump-context", bump_context),
                ("--no-bump-context", no_bump_context),
            ],
        )
    )


def flow(
    *,
    repo_path: str | None = None,
    source: Source | None = None,
    verbose: bool | None = None,
    input_format: FlowInputFormat | None = None,
    output_format: FlowOutputFormat | None = None,
    output_template: str | None = None,
    output_prefix: str | None = None,
    pre_release_label: FlowPreReleaseLabel | None = None,
    pre_release_num: int | None = None,
    post_mode: FlowPostMode | None = None,
    branch_rules: str | None = None,
    tag_version: str | None = None,
    distance: int | None = None,
    dirty: bool | None = None,
    no_dirty: bool | None = None,
    clean: bool | None = None,
    bumped_branch: str | None = None,
    bumped_commit_hash: str | None = None,
    bumped_timestamp: int | None = None,
    major: int | None = None,
    minor: int | None = None,
    patch: int | None = None,
    epoch: int | None = None,
    post: int | None = None,
    hash_branch_len: int | None = None,
    schema: StandardSchema | None = None,
    schema_ron: str | None = None,
) -> str:
    return _run_zerv_command(
        args=_extend_args(
            args=["flow"],
            flags=[
                ("-C", repo_path),
                ("-s", source),
                ("-v", verbose),
                ("-f", input_format),
                ("--output-format", output_format),
                ("--output-template", output_template),
                ("--output-prefix", output_prefix),
                ("--pre-release-label", pre_release_label),
                ("--pre-release-num", pre_release_num),
                ("--post-mode", post_mode),
                ("--branch-rules", branch_rules),
                ("--tag-version", tag_version),
                ("--distance", distance),
                ("--dirty", dirty),
                ("--no-dirty", no_dirty),
                ("--clean", clean),
                ("--bumped-branch", bumped_branch),
                ("--bumped-commit-hash", bumped_commit_hash),
                ("--bumped-timestamp", bumped_timestamp),
                ("--major", major),
                ("--minor", minor),
                ("--patch", patch),
                ("--epoch", epoch),
                ("--post", post),
                ("--hash-branch-len", hash_branch_len),
                ("--schema", schema),
                ("--schema-ron", schema_ron),
            ],
        )
    )


def check(
    version: str,
    *,
    format: CheckFormat | None = None,
    verbose: bool | None = None,
) -> str:
    return _run_zerv_command(
        args=_extend_args(
            args=["check", version],
            flags=[
                ("--format", format),
                ("-v", verbose),
            ],
        )
    )
