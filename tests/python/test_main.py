from __future__ import annotations

import subprocess
import sys

import pytest


@pytest.mark.parametrize("args", [["--version"], ["--help"]])
def test_python_m_zerv_executes(args):
    result = subprocess.run(
        [sys.executable, "-m", "zerv", *args],
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, f"stdout: {result.stdout}\nstderr: {result.stderr}"
    assert len(result.stdout) > 0, "Expected output on stdout"


def test_python_m_zerv_version_output():
    result = subprocess.run(
        [sys.executable, "-m", "zerv", "--version"],
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0
    assert "zerv" in result.stdout.lower(), (
        f"Expected 'zerv' in output, got: {result.stdout}"
    )


def test_python_m_zerv_help_output():
    result = subprocess.run(
        [sys.executable, "-m", "zerv", "--help"],
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0
    # Typical help indicators
    output_lower = result.stdout.lower()
    assert "usage" in output_lower or "help" in output_lower, (
        f"Expected help text, got: {result.stdout}"
    )


def test_python_m_zerv_invalid_arg_returns_error():
    result = subprocess.run(
        [sys.executable, "-m", "zerv", "--nonexistent-flag-xyz"],
        capture_output=True,
        text=True,
    )
    # Should fail (non-zero exit code)
    assert result.returncode != 0
