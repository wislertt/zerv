# Docs ↔ Test Sync Audit

Verify documentation code examples stay in sync with their backing tests.
Periodic sweep — run after test renames/refactors, before releases, or before
large docs edits.

**Tests are the source of truth.** When docs and tests disagree, update the
docs to match the tests.

## Usage

```bash
/docs-sync              # Full sweep: docs/ site + README.md
/docs-sync cli pages    # NLP: only docs/cli/*.mdx
/docs-sync readme       # NLP: only README.md
```

## Scope

| Docs surface                    | Reference comment format                |
| ------------------------------- | --------------------------------------- |
| `docs/**/*.mdx` (Mintlify site) | `{/* Corresponding test: path::fn */}`  |
| `README.md`                     | `<!-- Corresponding test: path::fn -->` |

Reference path resolves from repo root. Valid targets:

- `tests/integration_tests/**/*.rs` — Rust (`fn <name>`)
- `tests/python/*.py` — Python (`def <name>`)

## Workflow

### 1. Collect references

```bash
grep -rn "Corresponding test:" docs README.md --include="*.mdx" -r 2>/dev/null; \
grep -n "Corresponding test:" README.md
```

Build list of `(doc_file, line, test_path, test_fn)`.

### 2. Verify each reference exists

- Test file exists on disk
- File contains `fn <name>` (Rust) or `def <name>` (Python)

Report broken refs — stale after renames, moves, deletions.

### 3. Compare example content vs test

For each reference, read the doc's preceding code block and the test function:

- **Commands match**: flags, arguments, quoting exactly as the test invokes
  them (e.g. `--output-template`, NOT `--template`)
- **Expected output matches**: strings shown as output comments in docs must
  equal the test's asserted values
- **Meaning matches**: if the test was renamed/refactored to cover different
  behavior, the doc example no longer points at the right backing

### 4. Detect orphans

Code blocks in scope WITHOUT a `Corresponding test` comment. Markdown
reference comment placement: directly below the block it covers.

### 5. Fix

- Broken ref → repoint to the correct test (search by example content)
- Content drift → update doc block to match test
- Orphan block → either add a backing test or add a reference to an existing
  test that already covers the example
- MDX comments must be `{/* ... */}`; HTML `<!-- -->` breaks Mintlify builds

### 6. Report

Summary table: refs checked / broken refs fixed / content drift fixed /
orphans resolved. List anything intentionally left (e.g. illustrative blocks
that cannot be tested — flag for review instead of silently skipping).

## Verification

Docs-only fixes: none needed beyond `bake docs-check` if `docs/` changed.

If any test file was edited: run the touched tests targeted, e.g.

```bash
cargo +nightly test --test integration -- <filter>
```

Full `bake -c lint test` only if this sweep is the end of a larger task.
