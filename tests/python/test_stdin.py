from __future__ import annotations

from zerv import flow, version

SAMPLE_ZERV_INPUT = """(
    schema: (
        core: [
            var(Major),
            var(Minor),
            var(Patch),
        ],
        extra_core: [],
        build: [
            var(BumpedBranch),
            var(Distance),
            var(BumpedCommitHashShort),
        ],
        precedence_order: [
            Major,
            Minor,
            Patch,
            Build,
        ],
    ),
    vars: (
        major: Some(1),
        minor: Some(2),
        patch: Some(3),
        epoch: None,
        pre_release: None,
        post: None,
        dev: None,
        distance: Some(5),
        dirty: Some(true),
        bumped_branch: Some("feature/test"),
        bumped_commit_hash: Some("abc123def"),
        bumped_timestamp: Some(1234567890),
        last_branch: None,
        last_commit_hash: Some("abc123def456"),
        last_timestamp: Some(1234567890),
        last_tag_version: Some("v1.2.3"),
        custom: (),
    ),
)"""


def test_version_with_stdin_basic():
    result = version(
        source="stdin",
        stdin=SAMPLE_ZERV_INPUT,
    )
    assert result
    assert "1.2.3" in result


def test_version_with_stdin_and_output_format():
    result = version(
        source="stdin",
        stdin=SAMPLE_ZERV_INPUT,
        output_format="pep440",
    )
    assert result
    assert "1.2.3" in result


def test_version_with_stdin_and_schema():
    result = version(
        source="stdin",
        stdin=SAMPLE_ZERV_INPUT,
        schema="calver",
    )
    assert result


def test_version_pipe_between_commands():
    # First call: get version in zerv format
    zerv_output = version(tag_version="v1.2.3", clean=True, output_format="zerv")

    # Second call: use that output as stdin
    result = version(source="stdin", stdin=zerv_output, schema="standard")
    assert result
    assert "1.2.3" in result


def test_flow_with_stdin_basic():
    result = flow(source="stdin", stdin=SAMPLE_ZERV_INPUT)
    assert result


def test_flow_with_stdin_and_output_format():
    result = flow(source="stdin", stdin=SAMPLE_ZERV_INPUT, output_format="pep440")
    assert result
