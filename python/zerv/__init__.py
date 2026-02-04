"""Zerv Python API.

This module provides a Python interface to the zerv CLI tool.
The zerv binary is automatically located and called via subprocess.
"""

from __future__ import annotations

import subprocess
from typing import Literal

from zerv._find_zerv import find_zerv_bin

__version__ = "0.0.0"
__all__ = [
    "find_zerv_bin",
    "version",
    "flow",
    "check",
]

# Format literals
Format = Literal["auto", "semver", "pep440", "zerv", "json", "raw"]

# Source literals
Source = Literal["git", "stdin", ""]

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
    "",
]

# Schema preset literals
SchemaPreset = Literal[
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
    "",
]

# Check format literals
CheckFormat = Literal["semver", "pep440", "auto", ""]


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
    repo_path: str = ".",
    format: Format = "semver",
    prefix: str = "",
    commit: str = "",
    tag: str = "",
    branch: str = "",
    bump: BumpType = "",
    default: str = "",
    source: Source = "",
    schema: SchemaPreset = "",
    fields: str = "",
    remote: str = "origin",
) -> str:
    """Generate version information for a git repository.

    Args:
        repo_path: Path to git repository (default: ".")
        format: Output format - semver, pep440, zerv, auto (default: "semver")
        prefix: Version prefix to strip/add (default: "")
        commit: Specific commit hash (default: current HEAD)
        tag: Specific tag to use as base (default: auto-detect)
        branch: Branch name for flow versioning (default: current branch)
        bump: Version part to bump - major, minor, patch, etc. (default: "")
        default: Default version if no tags found (default: "")
        source: VCS source - git, stdin (default: auto-detect)
        schema: Schema format for zerv versions (default: auto-detect)
        fields: Custom fields for template (default: "")
        remote: Git remote for flow operations (default: "origin")

    Returns:
        Version string
    """
    args = ["version", "-C", repo_path]

    if format != "semver":
        args.extend(["--output-format", format])
    if prefix:
        args.extend(["--prefix", prefix])
    if commit:
        args.extend(["--commit", commit])
    if tag:
        args.extend(["--tag", tag])
    if branch:
        args.extend(["--branch", branch])
    if bump:
        args.extend(["--bump", bump])
    if default:
        args.extend(["--default", default])
    if source:
        args.extend(["--source", source])
    if schema:
        args.extend(["--schema", schema])
    if fields:
        args.extend(["--fields", fields])
    if remote != "origin":
        args.extend(["--remote", remote])

    return _run_zerv_command(args)


def flow(
    repo_path: str = ".",
    format: Format = "json",
    prefix: str = "",
    branch: str = "",
    schema: SchemaPreset = "",
    remote: str = "origin",
) -> str:
    """Generate flow-based version information.

    Args:
        repo_path: Path to git repository (default: ".")
        format: Output format (default: "json")
        prefix: Version prefix (default: "")
        branch: Branch name (default: current branch)
        schema: Schema format (default: auto-detect)
        remote: Git remote (default: "origin")

    Returns:
        Formatted flow information as string
    """
    args = ["flow", "-C", repo_path]

    if format != "json":
        args.extend(["--output-format", format])
    if prefix:
        args.extend(["--prefix", prefix])
    if branch:
        args.extend(["--branch", branch])
    if schema:
        args.extend(["--schema", schema])
    if remote != "origin":
        args.extend(["--remote", remote])

    return _run_zerv_command(args)


def check(
    version: str,
    format: CheckFormat = "auto",
) -> str:
    """Validate a version string.

    Args:
        version: Version string to validate
        format: Format to validate against - semver, pep440, auto (default: "auto")

    Returns:
        Validation result as string
    """
    args = ["check", version]
    if format != "auto":
        args.extend(["--format", format])

    return _run_zerv_command(args)
