from __future__ import annotations

import pytest

from zerv import flow


@pytest.mark.parametrize(
    "kwargs",
    [
        # Basic
        {},
        # Input options
        {"repo_path": "."},
        {"source": "git"},
        {"verbose": True},
        {"input_format": "semver"},
        # Output options
        {"output_format": "semver"},
        {"output_template": "{version}"},
        {"output_prefix": "v"},
        # Pre-release options
        {"pre_release_label": "alpha"},
        {"pre_release_num": 1},
        {"post_mode": "tag"},
        # VCS override options
        # branch_rules needs JSON array - skip
        {"tag_version": "v1.0.0"},
        {"distance": 5},
        {"dirty": True},
        {"no_dirty": True},
        {"clean": True},
        # bumped_* need context
        {"tag_version": "v1.0.0", "bumped_branch": "main"},
        {"tag_version": "v1.0.0", "bumped_commit_hash": "abc123"},
        {"tag_version": "v1.0.0", "bumped_timestamp": 1234567890},
        # Version component overrides
        {"major": 1},
        {"minor": 1},
        {"patch": 1},
        {"epoch": 1},
        {"post": 1},
        {"hash_branch_len": 7},
        # Schema options
        {"schema": "standard"},
    ],
)
def test_flow_all_args(kwargs):
    result = flow(**kwargs)
    assert result
