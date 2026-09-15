# Semantic-release: untagged non-release push must not fail

**Date:** 2026-09-15
**Status:** implementation complete (uncommitted), awaiting fixture-repo setup + CI verification
**Trigger bug:** https://github.com/scbabacus/gcp-landing-zone/actions/runs/34922524139/job/104235008661

## Root cause of the original failure

Commit `revert: restore wiz org-wide scanning module (#231)` (bc4dc24) landed on
gcp-landing-zone `main`. semantic-release `commit-analyzer` has no release rule
for type `revert` → no release published. The `determine-version-from-release-or-tag`
step in `shared-semantic-release.yml` then required HEAD to carry a `v*` tag;
bc4dc24 had none → `exit 1`.

Tag evidence at time of failure:

```
bc4dc24 (revert)  → no tags          ← failed
a8160d0 (chore)   → v1 v1.64 v1.64.11 ← passed (chore releases patch in that repo)
```

Why PR validation passed: `.github/workflows/semantic-pull-request.yml` uses
`amannn/action-semantic-pull-request`, whose default types come from commitizen
`conventional-commit-types` — which INCLUDES `revert`. So the PR linter accepts
`revert:` but the analyzer never releases on it. Two tools, divergent semantics.

## What was changed (all in this repo, all UNCOMMITTED)

### 1. `.github/workflows/shared-semantic-release.yml`

- `determine-version-from-release-or-tag` step rewritten:
    - Refactored to use step-level `env:` (`RELEASE_PUBLISHED`, `RELEASE_VERSION`)
      instead of inline `${{ }}` expressions inside bash. Script is now plain
      testable bash and has no expression-injection surface.
    - **Behavior change (the fix):** when `published=false` and HEAD has no tag,
      the step now emits `new_release_version=""` + `::notice::` and exits 0.
      Previously: `::error::` + exit 1.
    - Fallback tag selection improved: prefers exact semver tag
      (`^v\d+\.\d+\.\d+$`), falls back to moveable tags (`^v\d+(\.\d+)?$`,
      lexical order → `v1` before `v1.64`).
- New workflow inputs (backwards compatible, empty = old behavior):
    - `checkout_repository` (string, default "") — passed to actions/checkout
    - `checkout_ref` (string, default "") — passed to actions/checkout
    - actions/checkout treats empty string as "use default", so existing
      consumers (gcp-landing-zone, this repo's ci/cd) are unaffected.
- Output description updated: `new_release_version` is empty when no release
  published and no tag on HEAD.

### 2. `.github/workflows/test-shared-semantic-release.yml` (NEW)

Integration test for the shared workflow. Structure:

```
setup-fixtures  → crafts 5 orphan git histories in temp dirs, force-pushes as
                  branches zerv-ci/semantic-release-f{1..5}-* of the DEDICATED
                  fixture repo wislertt/zerv-sandbox (NOT this repo).
                  Each fixture tree carries its own .releaserc.json with
                  angular defaults (only feat/fix/perf release) so `revert:`
                  commits are guaranteed non-releasing.
f1-no-tags      revert commits, zero tags          → published=false, version=""
f2-semver-tag   HEAD: v2.3.4 + v2 + v2.3           → version=2.3.4 (semver preferred)
f3-moveable     HEAD: v3 + v3.1                    → version=3    (moveable, lexical)
f4-tag-behind   v1.2.3 behind, revert on HEAD      → version=""   ← exact prod bug E2E
f5-new-release  v1.0.0 + feat commit               → published=true, version=1.1.0 (dry-run)
verify          15 exact assertions (5 scenarios × published/version/valid)
cleanup-fixtures if: always() → deletes fixture branches + tags from fixture repo
```

Key design decisions (do not regress these):

- **Fixtures live in a dedicated separate repo** (`wislertt/zerv-sandbox`),
  NOT in this repo. Earlier iteration pushed fixture tags (v1.0.0, v2.3.4,
  v3.1...) into this repo — that pollutes the real tag namespace and can race
  with real releases (semantic-release reads last-release tags repo-globally).
  User explicitly rejected touching real tags. Do not move fixtures back.
- Scenario jobs pass `checkout_repository: wislertt/zerv-sandbox` +
  `checkout_ref: refs/heads/zerv-ci/...` to the shared workflow.
- The fixture repo may stay **private**: the shared workflow accepts an
  optional `checkout_token` secret (`${{ secrets.checkout_token || github.token }}`
  on its checkout step), and each scenario call passes
  `secrets: checkout_token: ${{ secrets.ZERV_SANDBOX_REPO_TOKEN }}` down to it.
  Originally the repo had to be public because the PAT could not reach
  actions/checkout; user chose the token-threading option instead.
- Fixture push/cleanup auth uses secret `ZERV_SANDBOX_REPO_TOKEN` (fine-grained
  PAT, Contents:RW on the fixture repo only). Setup fails with a clear error
  message if the secret is missing.
- Test wired into `ci.yml` AND `cd.yml`. Originally ci-only (lean release
  path), but cd runs on a push event, so f5's strict release-detection
  assertions (skipped under pull_request) execute automatically on every
  release — no manual dispatch needed.
- All scenario runs use `dry_run: true` → semantic-release publishes nothing.
- f5 strict assertions are event-gated: under a pull_request event
  semantic-release skips release detection entirely ("This run was triggered by
  a pull request...") and `GITHUB_*` env vars are reserved — they CANNOT be
  overridden at step level (attempted `GITHUB_EVENT_NAME: push` override was
  silently ignored by the runner). So verify asserts f5 strictly only on
  non-PR events (workflow_dispatch / push with the workflow on main); PR runs
  emit a notice and skip the two f5 assertions.
- f5 fixture .releaserc `branches` must name the branch semantic-release
  resolves from the CALLER's GITHUB_REF (`"main"` for dispatch/push on main),
  not the fixture branch name; fixture() takes an optional 2nd arg for it.
- Cleanup `if: always()`; deletes EVERY branch/tag in the sandbox (it is
  dedicated to CI) via ls-remote → push --delete loop, then re-lists and fails
  if anything remains. Earlier version ran `git push` with no git repo present
  (no checkout step) — every delete failed "not a git repository" and the
  `|| echo` guard masked it as "already absent", so refs piled up silently.

### 3. `.github/workflows/ci.yml`

Added `test-shared-semantic-release` job calling the new test workflow.

### 4. Python test — DELETED on purpose

`tests/python/test_shared_semantic_release_workflow.py` existed (extracted the
bash from YAML, ran it against git fixtures, 5 scenarios). User reviewed and
rejected the approach: workflow behavior should be tested by a workflow with
crafted dummy repos, not by pytest-extracted bash. The file was deleted;
scenario coverage moved into the workflow test. Do not recreate it.

### 5. NOT changed (unrelated pre-existing)

`docs/cicd/*.mdx`, `docs/docs.json`, `docs/getting-started/why-zerv.mdx` were
already modified in the worktree before this task. Leave them out of commits
for this work or ask the user.

## Current git state

Branch `main` (same as origin/main). Uncommitted:

```
 M .github/workflows/ci.yml
 M .github/workflows/shared-semantic-release.yml
?? .github/workflows/test-shared-semantic-release.yml
 M docs/... (pre-existing, unrelated)
```

Pre-commit (`pre-commit run --files <changed files>`) passes: exit 0, includes
actionlint + prettier + check-yaml.

## Verification status

- Local: fixture-creation bash logic dry-run tested in temp dir (histories,
  tags, .releaserc content all correct).
- Local: pre-commit + actionlint clean.
- NOT yet verified: actual workflow execution. `uses: ./.github/workflows/...`
  and `workflow_dispatch` require the workflow file on the DEFAULT branch, so
  the test can only run after these commits land on main.

## User's remaining setup (one-time, blocks CI verification)

1. Create repo `wislertt/zerv-sandbox` (empty, no README). May be private
   (see design decisions: checkout_token threading). DONE 2026-09-15.
2. Fine-grained PAT: only `zerv-sandbox`, Contents: Read/Write. DONE.
3. Secret `ZERV_SANDBOX_REPO_TOKEN` on wislertt/zerv. DONE.

## Open follow-ups

1. Commit + push these changes → ci.yml runs the test matrix → first real
   verification. If a scenario fails, the verify job names it exactly
   (f1..f5); check the semantic-release logs in that scenario job.
2. Bump pin in gcp-landing-zone `.github/workflows/cd.yml` (lines ~21, 29, 33
   reference `wislertt/zerv@2fcb93c... # v0.8.30`) after zerv releases.
3. `revert` release rule in `.releaserc.mjs` (THIS repo's own config): the
   `types` array lists feat fix chore docs style refactor perf test build ci —
   `revert` is missing, and the `types_glob` → patch release rule covers all
   listed types. Adding `"revert"` to the array = one-line fix so future
   reverts in repos using zerv's default config publish patch releases.
   NOT yet applied — user hasn't confirmed.
4. Known pre-existing quirk (documented, not fixed): when only moveable tags
   are on HEAD, the fallback returns the moveable version (e.g. `3` from
   `v3`), not full semver. Kept for backwards compatibility.

## Behavioral contract after the fix (for consumers like gcp-landing-zone)

```
published=true                  → new_release_version = released semver
published=false, tag on HEAD    → new_release_version = version from tag
published=false, no tag on HEAD → new_release_version = "" (empty), exit 0
is_valid_semantic_release       → true whenever the job ran successfully
```

Downstream note for gcp-landing-zone: `zerv-versioning` is gated on
`is_valid == 'true'`, so it now RUNS on non-release pushes and advances the
moveable tags v1/v1.64 to HEAD (self-healing: future non-release pushes find a
tag on HEAD and use the fallback path).

## CI debugging log (2026-09-15, branch fix-shared-semantic-release)

1. Run 1: setup failed — secret empty. Reusable-workflow calls don't inherit
   secrets; fixed with explicit `secrets:` passthrough in ci.yml + secret
   declaration in the test workflow's `workflow_call` block (actionlint).
2. Run 2: scenario checkouts failed "Repository not found" — sandbox private +
   default GITHUB_TOKEN can't read it. Fixed with `checkout_token` secret input
   on shared workflow; scenarios pass `ZERV_SANDBOX_REPO_TOKEN` down.
3. Run 3: verify failed only on f5 — semantic-release skips release detection
   under pull_request events. First fix attempt (`event_name_override: push`
   shared-workflow input setting the GITHUB_EVENT_NAME env) did nothing:
   GITHUB_* variables are reserved and the runner ignores overrides. Final
   fix: event-gated assertions + fixture .releaserc branches=["main"].
   Also discovered cleanup never actually deleted refs ("not a git repository":
   the job never checked out, and `git init` without `cd` didn't help — fixed
   with an explicit cd into the throwaway repo + ls-remote re-check).
4. Run 4: verify PASSED (f1-f4 + gates). Cleanup failed: GitHub refuses to
   delete the sandbox's CURRENT branch — the first ref pushed to the empty
   repo became its default branch.
5. Run 5: cleanup still failed — fix attempt PATCHed default_branch via REST
   API, but changing the default branch requires Administration:Write on the
   PAT (curl exit 22 = HTTP error; PAT has only Contents:RW). Final design:
   setup-fixtures pushes a permanent "main" FIRST (on an empty repo the first
   pushed ref becomes the default, so no API call needed); cleanup keeps
   "main" and deletes everything else. One-time manual step: switch the
   sandbox's default branch to "main" in repo settings (drifted from an
   older run).
