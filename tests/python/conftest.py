from __future__ import annotations

import pytest

from tests.python.utils import symlink_zerv_to_venv_bin as _symlink_zerv_to_venv_bin


@pytest.fixture(scope="session", autouse=True)
def symlink_zerv_to_venv_bin():
    _symlink_zerv_to_venv_bin()
    yield
