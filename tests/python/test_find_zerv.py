"""Tests for zerv._find_zerv module."""

from __future__ import annotations

import os
import sysconfig
from pathlib import Path
from unittest.mock import patch

import pytest

from zerv._find_zerv import find_zerv_bin


def test_find_zerv_bin_returns_string():
    result = find_zerv_bin()
    assert isinstance(result, str)


def test_find_zerv_bin_path_exists():
    result = find_zerv_bin()
    assert os.path.isfile(result), f"Path does not exist: {result}"


def test_find_zerv_bin_has_correct_name():
    result = find_zerv_bin()
    exe = sysconfig.get_config_var("EXE") or ""
    assert result.endswith(f"zerv{exe}"), (
        f"Expected path to end with 'zerv{exe}', got: {result}"
    )


def test_find_zerv_bin_not_found_raises_file_not_found():
    with patch("zerv._find_zerv.sysconfig.get_path") as mock_get_path:
        # Make all paths return a non-existent directory
        mock_get_path.return_value = "/nonexistent/path"

        with patch.object(Path, "is_file", return_value=False):
            with pytest.raises(FileNotFoundError):
                find_zerv_bin()


def test_find_zerv_bin_exe_suffix():
    exe = sysconfig.get_config_var("EXE") or ""
    expected_name = f"zerv{exe}"
    result = find_zerv_bin()
    assert os.path.basename(result) == expected_name
