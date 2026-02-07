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

    print("\n=== _find_zerv_binary ===")
    print(f"sys.platform: {sys.platform}")
    print(f"project_root: {project_root}")
    print(f"binary_name: {binary_name}")

    for path in [
        project_root / "target" / "release" / binary_name,
        project_root / "target" / "debug" / binary_name,
    ]:
        print(f"Checking: {path}")
        print(
            f"  exists: {path.exists()}, is_file: {path.is_file() if path.exists() else 'N/A'}"
        )
        if path.exists() and path.is_file():
            print(f"  Found! Returning: {path}")
            return path

    print("  NOT FOUND in any location!")
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
    print("\n=== conftest.py: symlink_zerv_to_venv_bin fixture starting ===")

    # Find the actual zerv binary
    zerv_bin = _find_zerv_binary()
    print(f"Found zerv binary at: {zerv_bin}")
    print(f"Binary exists: {zerv_bin.exists()}")
    print(f"Binary is file: {zerv_bin.is_file()}")

    # Get the venv's scripts/bin directory
    if sys.platform == "win32":
        venv_bin = Path(sys.prefix) / "Scripts"
    else:
        venv_bin = Path(sys.prefix) / "bin"

    print(f"sys.prefix: {sys.prefix}")
    print(f"venv_bin directory: {venv_bin}")
    print(f"venv_bin exists: {venv_bin.exists()}")
    print(f"venv_bin is_dir: {venv_bin.is_dir()}")

    # Create symlink
    symlink_path = venv_bin / "zerv"
    print(f"symlink_path target: {symlink_path}")

    # Remove existing symlink if it exists (from previous test runs)
    if symlink_path.exists() or symlink_path.is_symlink():
        print(f"Removing existing symlink/file at: {symlink_path}")
        symlink_path.unlink()

    # Create the symlink
    try:
        print(f"Attempting symlink: {symlink_path} -> {zerv_bin}")
        symlink_path.symlink_to(zerv_bin)
        print("Symlink created successfully")
    except OSError as e:
        # On Windows, symlinks may require developer mode or admin privileges
        # Fall back to copying if symlink fails
        import shutil

        print(f"Symlink failed: {e}")
        print("Falling back to shutil.copy2...")
        shutil.copy2(zerv_bin, symlink_path)
        print("Copy completed")

    # Verify the result
    print(f"symlink_path exists after operation: {symlink_path.exists()}")
    print(f"symlink_path is_file after operation: {symlink_path.is_file()}")
    print(f"symlink_path is_symlink after operation: {symlink_path.is_symlink()}")
    print("=== conftest.py: fixture setup complete ===\n")

    yield

    # Cleanup: remove the symlink after all tests complete
    if symlink_path.exists() or symlink_path.is_symlink():
        symlink_path.unlink()
