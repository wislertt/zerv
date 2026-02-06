from __future__ import annotations

from zerv import _extend_args


def test_extend_args_basic():
    result = _extend_args(
        ["base"],
        [
            ("--foo", "value"),
            ("--bar", None),  # skipped
            ("--baz", True),  # flag only
            ("--qux", False),  # skipped
        ],
    )
    assert result == ["base", "--foo", "value", "--baz"]


def test_extend_args_returns_list_for_chaining():
    result = _extend_args(["cmd"], [("--opt", "val")])
    assert result == ["cmd", "--opt", "val"]
