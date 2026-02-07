from __future__ import annotations

import pytest

from zerv import check


@pytest.mark.parametrize(
    "kwargs",
    [
        # Basic
        {"version": "1.0.0"},
        # Format options
        {"version": "1.0.0", "format": "semver"},
        {"version": "1.0.0", "format": "pep440"},
        # Verbose
        {"version": "1.0.0", "verbose": True},
    ],
)
def test_check_all_args(kwargs):
    """Test all check() arguments work without error."""
    result = check(**kwargs)
    assert result
