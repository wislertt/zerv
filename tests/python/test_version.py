from __future__ import annotations

import pytest

from zerv import version


@pytest.mark.parametrize(
    "kwargs",
    [
        # Basic
        {},
        # Input options
        {"source": "git"},
        {"input_format": "semver"},
        {"repo_path": "."},
        # Output options
        {"output_format": "semver"},
        {"output_template": "{version}"},
        {"output_prefix": "v"},
        # Schema options
        {"schema": "standard"},
        # VCS override options
        {"tag_version": "v1.0.0"},
        {"distance": 5},
        {"dirty": True},
        {"no_dirty": True},
        {"clean": True},
        # bumped_* need context
        {"tag_version": "v1.0.0", "bumped_branch": "main"},
        {"tag_version": "v1.0.0", "bumped_commit_hash": "abc123"},
        {"tag_version": "v1.0.0", "bumped_timestamp": 1234567890},
        # Version component override options
        {"major": 1},
        {"minor": 1},
        {"patch": 1},
        {"epoch": 1},
        {"post": 1},
        # Version-specific override options
        {"dev": 1},
        {"pre_release_label": "alpha"},
        {"pre_release_num": 1},
        # Schema component override options (need index=value format)
        {"core": "0=1"},
        {"extra_core": "0=0"},
        # build override requires stdin with actual content - skip
        # Field-based bump options
        {"bump_major": 1},
        {"bump_minor": 1},
        {"bump_patch": 1},
        {"bump_post": 1},
        {"bump_dev": 1},
        {"bump_pre_release_num": 1},
        {"bump_epoch": 1},
        # bump_pre_release_label uses label name directly
        {"bump_pre_release_label": "beta"},
        # Context control options
        {"bump_context": True},
        {"no_bump_context": True},
    ],
)
def test_version_all_args(kwargs):
    """Test all version() arguments work without error."""
    result = version(**kwargs)
    assert result
