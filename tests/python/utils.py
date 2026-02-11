from __future__ import annotations

import shutil
import sys
from pathlib import Path


def find_zerv_bin_from_target() -> Path:
    """Find the zerv binary in target/release or target/debug."""
    project_root = Path(__file__).parent.parent.parent

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


def symlink_zerv_to_venv_bin(zerv_bin: Path | None = None) -> Path:
    """
    Symlink zerv binary to venv bin directory.

    With `bindings = "bin"`, maturin doesn't install the binary to .venv/bin/.
    This creates a symlink from the actual binary location (target/release or target/debug)
    to the venv's bin directory.

    Args:
        zerv_bin: Path to zerv binary. If None, will auto-detect.

    Returns:
        Path to the created symlink/copied binary.
    """
    if zerv_bin is None:
        zerv_bin = find_zerv_bin_from_target()

    if sys.platform == "win32":
        venv_bin = Path(sys.prefix) / "Scripts"
    else:
        venv_bin = Path(sys.prefix) / "bin"

    symlink_name = "zerv.exe" if sys.platform == "win32" else "zerv"
    symlink_path = venv_bin / symlink_name

    if symlink_path.exists() or symlink_path.is_symlink():
        symlink_path.unlink()

    try:
        symlink_path.symlink_to(zerv_bin)
    except OSError:
        shutil.copy2(zerv_bin, symlink_path)

    return symlink_path
