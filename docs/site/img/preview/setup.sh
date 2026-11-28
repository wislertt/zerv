#!/usr/bin/env bash
# Recreates the demo repo and renders zerv-preview.gif.
#
# Usage: ./setup.sh [target-dir]
#
# The demo repo is rebuilt from scratch on every run, so the output is
# deterministic. Requirements: git, vhs (0.12.1 or later, 0.12.0 writes no
# output), zerv on PATH.

set -euo pipefail

DIR="${1:-zerv-demo-render}"
SRC="$(cd "$(dirname "$0")" && pwd)"

for cmd in git vhs zerv; do
    command -v "$cmd" >/dev/null || { echo "error: $cmd not on PATH" >&2; exit 1; }
done

VHS_VER="$(vhs --version | awk '{print $NF}')"
if [ "$VHS_VER" = "0.12.0" ] || [ "$VHS_VER" = "v0.12.0" ]; then
    echo "error: vhs 0.12.0 writes no output, install 0.12.1 or later" >&2
    exit 1
fi

rm -rf "$DIR"
mkdir -p "$DIR"
cd "$DIR"

git init -q repo
cd repo
git config user.email demo@demo.dev
git config user.name Demo

# .gitignore keeps the tape and gif untracked, otherwise zerv sees the repo
# as dirty and every version gains a .dev suffix.
printf 'zerv.tape\nzerv-preview.gif\n' > .gitignore
# standard-no-context strips the build context (+branch.N.g<hash>) so the
# GIF only shows the version part.
printf 'schema = "standard-no-context"\n' > zerv.toml
printf 'fn main() { println!("hello"); }\n' > main.rs
git add -A
git commit -qm "init"
git tag v1.2.3

# One worktree per git state, so the tape needs no hidden setup commands.
git worktree add -q ../wt-develop -b develop
printf '// dev work\n' >> ../wt-develop/main.rs
git -C ../wt-develop commit -qam "dev work"

git worktree add -q ../wt-feature -b feature/otp develop
printf '// otp work\n' >> ../wt-feature/main.rs
git -C ../wt-feature commit -qam "add otp"

cp "$SRC/zerv.tape" .
vhs zerv.tape

echo
echo "done: $DIR/repo/zerv-preview.gif"
echo "re-render: git -C '$DIR/wt-feature' checkout -q -- main.rs && cd '$DIR/repo' && vhs zerv.tape"
