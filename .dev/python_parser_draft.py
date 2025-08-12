import re
from pprint import pprint
from enum import Enum
from dataclasses import dataclass, field
from typing import List, Optional, Union

BASE_PATTERN = r"(?P<base>[0-9]+(?:\.[0-9]+)*)"


# pre enum
# - a --> alpha, a
# - b --> beta, b
# - rc --> rc, c, preview, pre

pep440_pre_map = {
    "alpha": "a",
    "a": "a",
    "beta": "b",
    "b": "b",
    "rc": "rc",
    "c": "rc",
    "preview": "rc",
    "pre": "rc",
}

# post
# - post --> rev, r, post

pep440_post_map = {"post": "post", "rev": "post", "r": "post"}


class PreReleaseLabel(Enum):
    ALPHA = "a"
    BETA = "b"
    RC = "rc"


@dataclass
class PEP440Version:
    epoch: int = 0
    release: list[int] = field(default_factory=list)
    pre_type: PreReleaseLabel | None = None
    pre_number: int | None = None
    post_number: int | None = None
    dev_number: int | None = None
    local: list[str | int] | None = None


PEP440_PATTERN = r"""
    v?
    (?:
        (?:(?P<epoch>[0-9]+)!)?                           # epoch
        (?P<release>[0-9]+(?:\.[0-9]+)*)                  # release segment
        (?P<pre>                                          # pre-release
            [-_\.]?
            (?P<pre_l>alpha|a|beta|b|preview|pre|c|rc)
            [-_\.]?
            (?P<pre_n>[0-9]+)?
        )?
        (?P<post>                                         # post release
            (?:-(?P<post_n1>[0-9]+))
            |
            (?:
                [-_\.]?
                (?P<post_l>post|rev|r)
                [-_\.]?
                (?P<post_n2>[0-9]+)?
            )
        )?
        (?P<dev>                                          # dev release
            [-_\.]?
            (?P<dev_l>dev)
            [-_\.]?
            (?P<dev_n>[0-9]+)?
        )?
    )
    (?:\+(?P<local>[a-z0-9]+(?:[-_\.][a-z0-9]+)*))?       # local version
"""

SEMVER_PATTERN = r"""
    ^v?(?P<major>0|[1-9]\d*)                            # major version
    \.(?P<minor>0|[1-9]\d*)                            # minor version
    \.(?P<patch>0|[1-9]\d*)                            # patch version
    (?:                                                 # optional prerelease
        -(?P<prerelease>
            (?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)    # prerelease identifier
            (?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))* # additional identifiers
        )
    )?
    (?:                                                 # optional build metadata
        \+(?P<buildmetadata>
            [0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*           # build metadata
        )
    )?$
"""

PVP_PATTERN = r"""
    ^v?(?P<version>[0-9]+(?:\.[0-9]+)+)                 # numeric version part
    (?P<tags>-[a-zA-Z0-9]+(?:-[a-zA-Z0-9]+)*)?         # deprecated tags
    $
"""

_pep440_regex = re.compile(PEP440_PATTERN, re.VERBOSE | re.IGNORECASE)
_semver_regex = re.compile(SEMVER_PATTERN, re.VERBOSE | re.IGNORECASE)
_pvp_regex = re.compile(PVP_PATTERN, re.VERBOSE | re.IGNORECASE)

is_valid_pep440 = lambda v: _pep440_regex.match(v) is not None
is_valid_semver = lambda v: _semver_regex.match(v) is not None
is_valid_pvp = lambda v: _pvp_regex.match(v) is not None


def parse_pep440_release(version):
    match = _pep440_regex.match(version)
    if match and match.group("release"):
        return [int(x) for x in match.group("release").split(".")]
    return None


pep_versions = [
    "1",
    "1.2",
    "42!2025.12.31a99.post123.dev456+deadbeef.abc123-zzz",
    "1!2.3.4rc5.post6.dev7+local.build.123",
    "v3.14.159a2.post1+git.abc123def",
    "0.1.0dev42+ubuntu.20.04",
    "0.1.0dev42+ubuntu.20-04",
    "1.2.3rc1",
    "1.2.3-c",
    "1.2.3.rc1",
    "1.2.3.rc1.post1",
    "1.2.3.rc1post1",
    "1.2.3.rc1-post1",
    "1.2.3.c-rev",
    "1.2.3.dev",
]

semver_versions = [
    "1.2.3-alpha.1.beta.2+build.123.sha.abc123",
    "10.20.30-rc.1+build.456.commit.def789",
    "2.0.0-x.7.z.92+build.2023.12.31",
    "1.0.0-alpha-beta+beta.exp.sha.5114f85",
    "v1.2.3-alpha.1.beta.2+build.123.sha.abc123",
    "V1.2.3-alpha.1.beta.2+build.123.sha.abc123",
    "V1.2.3-alpha.1.beta.2----+build.123.sha.abc123",
]

pvp_versions = [
    "1",
    "1.2",
    "1.2.3",
    "1.2.3.4",
    "1.2.3.4.5.6.7",
    "0.1",
    "10.20.30.40",
    "1.0-beta",
    "1.2.3-alpha",
    "1.0.2014-01-27",
    "2.1-rc-final",
    "v1.2.3",
    "v1.0-beta",
]


def test_versions(title, versions, validator, regex=None):
    print(f"=== {title} ===")
    for version in versions:
        valid = validator(version)
        print(f"'{version}' -> {valid}")
        if valid and regex:
            match = regex.match(version)
            components = {k: v for k, v in match.groupdict().items() if v}
            if regex == _pep440_regex:
                if "release" in components:
                    components["release"] = [
                        int(x) for x in components["release"].split(".")
                    ]

                # Normalize pre-release identifier
                if "pre_l" in components:
                    components["pre_l"] = pep440_pre_map.get(
                        components["pre_l"].lower(), components["pre_l"]
                    )

                # Normalize post-release identifier
                if "post_l" in components:
                    components["post_l"] = pep440_post_map.get(
                        components["post_l"].lower(), components["post_l"]
                    )

                # Parse local version
                if "local" in components:
                    local_parts = components["local"].split(".")
                    components["local"] = [
                        int(part) if part.isdigit() else part for part in local_parts
                    ]

            print("  Components:")
            pprint(components, indent=4, sort_dicts=False)
        print()


test_versions("PEP 440 Validation", pep_versions, is_valid_pep440, _pep440_regex)
# test_versions("SemVer Validation", semver_versions, is_valid_semver, _semver_regex)
# test_versions("PVP Validation", pvp_versions, is_valid_pvp, _pvp_regex)
