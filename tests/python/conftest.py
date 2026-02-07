from __future__ import annotations

import sys
from pathlib import Path

import pytest


def _find_zerv_binary() -> Path:
    """Find the zerv binary in target/release or target/debug."""
    # Start from tests/python and go up to project root
    project_root = Path(__file__).parent.parent.parent

    # On Windows, the binary has .exe extension
    binary_name = "zerv.exe" if sys.platform == "win32" else "zerv"

    for path in [
        project_root / "target" / "release" / binary_name,
        project_root / "target" / "debug" / binary_name,
    ]:
        if path.exists() and path.is_file():
            return path

    raise FileNotFoundError(
        f"Could not find zerv binary (searched for '{binary_name}'). Searched in: "
        f"{project_root / 'target' / 'release'}, {project_root / 'target' / 'debug'}"
    )


@pytest.fixture(scope="session", autouse=True)
def symlink_zerv_to_venv_bin(tmp_path_factory):
    """
    Symlink zerv binary to venv bin directory so find_zerv_bin() can find it.

    With `bindings = "bin"`, maturin doesn't install the binary to .venv/bin/.
    This fixture creates a symlink from the actual binary location (target/release or target/debug)
    to the venv's bin directory where find_zerv_bin() expects it.

    This is only needed for development/testing. Production installs from PyPI
    will have the binary properly installed to .venv/bin/.
    """
    # Find the actual zerv binary
    zerv_bin = _find_zerv_binary()

    # Get the venv's scripts/bin directory
    if sys.platform == "win32":
        venv_bin = Path(sys.prefix) / "Scripts"
    else:
        venv_bin = Path(sys.prefix) / "bin"

    # On Windows, the symlink needs .exe extension to match what find_zerv_bin() expects
    symlink_name = "zerv.exe" if sys.platform == "win32" else "zerv"
    symlink_path = venv_bin / symlink_name

    # Remove existing symlink if it exists (from previous test runs)
    if symlink_path.exists() or symlink_path.is_symlink():
        symlink_path.unlink()

    # Create the symlink
    try:
        symlink_path.symlink_to(zerv_bin)
    except OSError:
        # On Windows, symlinks may require developer mode or admin privileges
        # Fall back to copying if symlink fails
        import shutil

        shutil.copy2(zerv_bin, symlink_path)

    yield

    # Cleanup: remove the symlink after all tests complete
    if symlink_path.exists() or symlink_path.is_symlink():
        symlink_path.unlink()
