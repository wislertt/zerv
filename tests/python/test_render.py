from __future__ import annotations

import pytest
from zerv import render


@pytest.mark.parametrize(
    "kwargs,expected",
    [
        # Basic
        ({"version": "1.2.3"}, "1.2.3"),
        # Input format options
        ({"version": "1.2.3", "input_format": "auto"}, "1.2.3"),
        ({"version": "1.2.3", "input_format": "semver"}, "1.2.3"),
        ({"version": "1.2.3", "input_format": "pep440"}, "1.2.3"),
        # Output format options
        ({"version": "1.2.3", "output_format": "semver"}, "1.2.3"),
        ({"version": "1.2.3", "output_format": "pep440"}, "1.2.3"),
        # Output prefix
        ({"version": "1.2.3", "output_prefix": "v"}, "v1.2.3"),
        ({"version": "1.2.3", "output_prefix": "release-"}, "release-1.2.3"),
        # Output template
        ({"version": "1.2.3", "output_template": "v{{major}}"}, "v1"),
        ({"version": "1.2.3", "output_template": "{{major}}.{{minor}}"}, "1.2"),
    ],
)
def test_render_all_args(kwargs, expected):
    """Test all render() arguments work without error."""
    result = render(**kwargs)
    assert result == expected


@pytest.mark.parametrize(
    "version,output_format,expected",
    [
        # SemVer to PEP440
        ("1.2.3", "pep440", "1.2.3"),
        ("1.2.3-alpha.1", "pep440", "1.2.3a1"),
        ("1.2.3-beta.2", "pep440", "1.2.3b2"),
        ("1.2.3-rc.3", "pep440", "1.2.3rc3"),
        ("1.2.3-post.1", "pep440", "1.2.3.post1"),
        # PEP440 to SemVer
        ("1.2.3a1", "semver", "1.2.3-alpha.1"),
        ("1.2.3b2", "semver", "1.2.3-beta.2"),
        ("1.2.3rc3", "semver", "1.2.3-rc.3"),
        ("1.2.3.post1", "semver", "1.2.3-post.1"),
    ],
)
def test_render_format_conversion(version, output_format, expected):
    """Test format conversion between SemVer and PEP440."""
    result = render(version, output_format=output_format)
    assert result == expected
