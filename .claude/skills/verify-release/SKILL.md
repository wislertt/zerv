---
name: verify-release
description: Verify a zerv release across all distribution channels. Use after semantic-release cuts a version on main, after merging a PR that touches release workflows/assets, or when asked to check release health. Checks GitHub release assets, PyPI files, crates.io, downloads and audits binaries, smoke-tests install.sh and pip wheel installs in containers, and installs via cargo-binstall.
---

# Verify zerv release

Complete state every release must reach. Run each check; anything off = investigate before closing.

## Inputs

- `$VER` — released version, bare semver (e.g. `0.8.29`). Default: `gh release list --limit 1`.
- Tag = `v$VER`. PyPI package = `zerv-version`. crates.io crate = `zerv`. Repo = `wislertt/zerv`.

## 1. Channel presence

```bash
TAG="v$VER"
gh run list --commit "$(git rev-parse HEAD)" --json workflowName,conclusion   # cd + security green
gh release view "$TAG" -R wislertt/zerv --json assets -q '.assets[].name'
curl -s https://pypi.org/pypi/zerv-version/json | python3 -c "
import json,sys
d=json.load(sys.stdin)
print(d['info']['version'])
print('\n'.join(sorted(f['filename'] for f in d['releases']['$VER'])))"
curl -s -A "zerv-verify" https://crates.io/api/v1/crates/zerv | python3 -c "import json,sys; print(json.load(sys.stdin)['crate']['max_stable_version'])"
```

Expected:

- GH release: **8 assets**, exactly `zerv-<triple>` for x86_64/aarch64 × unknown-linux-gnu/unknown-linux-musl/apple-darwin + `zerv-x86_64-pc-windows-msvc.exe` + `zerv-aarch64-pc-windows-msvc.exe`
- PyPI `$VER`: **9 files** — 8 wheels (manylinux_2_17 ×2 dual-tagged, musllinux_1_2 ×2, macosx ×2, win ×2) + 1 sdist
- crates.io `max_stable_version` = `$VER`
- crates.io needs the `-A` user agent or returns empty

## 2. Binary audit (linux assets)

```bash
rm -rf "/tmp/zerv_$VER" && mkdir "/tmp/zerv_$VER" && cd "/tmp/zerv_$VER"
gh release download "$TAG" -R wislertt/zerv -p 'zerv-*-unknown-linux-*'
for f in *; do
  echo "== $f"
  file "$f"
  grep -aoE 'GLIBC_[0-9.]+' "$f" | sort -uV | tail -1 || echo "static (no glibc refs)"
done
```

Expected:

- `*-gnu`: dynamically linked, stripped, max `GLIBC_2.17` — higher = compatibility regression (the whole point of the maturin `--zig` floor)
- `*-musl`: `statically linked`, stripped, **zero** GLIBC refs
- `.exe` assets: `PE32+ executable`, correct arch via `file`

## 3. install.sh smoke (containers)

Musl leg (alpine) — binary executing at all on musl is the proof; a gnu asset would fail exec:

```bash
docker run --rm alpine:3.20 sh -c '
  apk add -q curl git >/dev/null 2>&1 &&
  curl -fsSL https://raw.githubusercontent.com/wislertt/zerv/main/scripts/install.sh | sh &&
  git config --global user.email t@t && git config --global user.name t &&
  mkdir /r && cd /r && git init -q && git commit -q --allow-empty -m x && git tag v1.2.3 &&
  $HOME/.local/bin/zerv version'
```

Gnu leg (rockylinux 9, glibc 2.34 — the motivating case for the 2.17 floor):

```bash
docker run --rm rockylinux:9-minimal bash -c '
  microdnf install -y git tar gzip >/dev/null 2>&1;
  curl -fsSL https://raw.githubusercontent.com/wislertt/zerv/main/scripts/install.sh | sh &&
  git config --global user.email t@t && git config --global user.name t &&
  mkdir /r && cd /r && git init -q && git commit -q --allow-empty -m x && git tag v1.2.3 &&
  $HOME/.local/bin/zerv version'
```

Floor payoff leg (amazonlinux 2, glibc 2.26 — oldest glibc the 2.17 claim buys):

```bash
docker run --rm amazonlinux:2 bash -c '
  yum install -y -q git >/dev/null 2>&1;
  curl -fsSL https://raw.githubusercontent.com/wislertt/zerv/main/scripts/install.sh | sh &&
  git config --global user.email t@t && git config --global user.name t &&
  mkdir /r && cd /r && git init -q && git commit -q --allow-empty -m x && git tag v1.2.3 &&
  $HOME/.local/bin/zerv version'
```

Expected: all three print `1.2.3`.

Gotchas:

- `zerv version` needs a git repo — without one it exits with `VCS not found`; that still proves the binary runs, but tag the repo for the real check
- minimal images lack `file` → install.sh prints `file: command not found` on stderr and proceeds; cosmetic, not a failure
- alpine/rocky-minimal ship no `git` (and rocky needs `tar`+`gzip`) — install those first or every step after fails confusingly

## 4. pip install (PyPI wheels, containers)

Bookworm (gnu wheel — manylinux_2_17):

```bash
docker run -e VER="$VER" --rm python:3.14-slim-bookworm bash -c '
  apt-get update -qq >/dev/null && apt-get install -y -qq git >/dev/null &&
  pip install -q --no-cache-dir "zerv-version==$VER" &&
  git config --global user.email t@t && git config --global user.name t &&
  mkdir /r && cd /r && git init -q && git commit -q --allow-empty -m x && git tag v1.2.3 &&
  zerv version'
```

Alpine (musl wheel — musllinux_1_2):

```bash
docker run -e VER="$VER" --rm python:3.14-alpine sh -c '
  apk add -q git >/dev/null 2>&1 &&
  pip install -q --no-cache-dir "zerv-version==$VER" &&
  git config --global user.email t@t && git config --global user.name t &&
  mkdir /r && cd /r && git init -q && git commit -q --allow-empty -m x && git tag v1.2.3 &&
  zerv version'
```

Expected: both print `1.2.3`. Verify the wheel tag actually used matches the container libc (`pip download` or pip's install log) — gnu container must not silently fall back to sdist.

## 5. cargo-binstall

Resolution (local, fast):

```bash
cargo binstall --dry-run --no-confirm "zerv@$VER"
```

Expected: resolves, downloads the right asset for the host triple from github.com, `Dry-run: Not proceeding`. Pure defaults, no flags — `[package.metadata.binstall] pkg-fmt = "bin"` in Cargo.toml + default pkg-url probing the versionless asset name do all the work.

Real install (container):

```bash
docker run -e VER="$VER" --rm rockylinux:9-minimal bash -c '
  microdnf install -y git tar gzip >/dev/null 2>&1;
  curl -LsSf https://github.com/cargo-bins/cargo-binstall/raw/main/install-from-binstall-release.sh | bash >/dev/null 2>&1 &&
  $HOME/.cargo/bin/cargo-binstall --no-confirm "zerv@$VER" &&
  git config --global user.email t@t && git config --global user.name t &&
  mkdir /r && cd /r && git init -q && git commit -q --allow-empty -m x && git tag v1.2.3 &&
  $HOME/.cargo/bin/zerv version'
```

Expected: `1.2.3`. The installer script ships the prebuilt cargo-binstall binary — no rust toolchain needed; invoke `cargo-binstall` directly, not `cargo binstall`.

Real install on the local machine (writes `~/.cargo/bin/zerv`) only on explicit request.

## Report

End every run with a status report. One line per check: ✅ pass / ❌ fail, followed by the evidence (value found, expected). Every check NOT run due to a platform limit gets its own ⏭️ line with the reason — the report must state explicitly what was tested and what was skipped. Shape:

```markdown
# Release v0.8.29 verification

- ✅ cd run on merge commit — success
- ✅ GH release — 8/8 assets (gnu+musl ×2, mac ×2, win ×2)
- ✅ PyPI — 9/9 files
- ✅ crates.io — 0.8.29
- ✅ gnu binaries — max GLIBC_2.17 (x64, arm64), stripped
- ✅ musl binaries — static, zero glibc refs (x64, arm64), stripped
- ✅ install.sh alpine — musl asset runs, `1.2.3`
- ✅ install.sh rocky9 — gnu asset (glibc 2.34) runs, `1.2.3`
- ✅ install.sh amazonlinux:2 — gnu asset (glibc 2.26, floor payoff) runs, `1.2.3`
- ✅ pip bookworm — manylinux_2_17 wheel installs, `1.2.3`
- ✅ pip alpine — musllinux_1_2 wheel installs, `1.2.3`
- ✅ cargo-binstall dry-run — resolves zerv@0.8.29 with pure defaults
- ✅ cargo-binstall container — real install from GH asset, `1.2.3`
- ⏭️ windows .exe binaries — not executed: no windows runner in docker; structure-only audit (PE32+ / arch via `file`) covered in section 2
- ⏭️ macOS darwin binaries — not executed: no macOS runner in docker; structure-only audit covered in section 2
- ⏭️ non-host-arch binaries — not executed: containers run host arch only
```

Any ❌ → investigation line under it with observed vs expected, then next action (rerun job, re-execute workflow, fix + new release). Do not fix silently inside the report — surface it.

## Pass criteria

All lines ✅ = release complete. Record the run (plan doc or memory) and note anything that needed a job rerun.
