# Tasks: Docs Site SEO — zerv

Checklist companion to `context.md`. Decisions locked 2026-08-24. All repo work ships as **one PR**.

## Locked decisions

- Homepage title: `Dynamic versioning from git for every commit` (45 chars; 52 rendered with ` - zerv` suffix)
- dunamai: add honest comparison section to `getting-started/why-zerv`
- Troubleshooting: split 5 error-string sections into standalone pages; hub keeps 4 behavioral sections
- PR shape: single PR for all repo changes (matches docs-site build precedent)
- Non-goals: unchanged, see `context.md` (no blog, no custom robots/sitemap, no structured-data changes, subdomain stays)
- No further llms.txt/GEO work: Google officially ignores it (June 2026 guidance); `skill.md` is the one AI-facing asset worth keeping

## Phase A — meta fixes

- [x] A1. `docs/docs.json`: `seo.metatags.og:image` → `https://zerv.wisl.dev/img/brand/og-card-light.png` (absolute; relative resolves against mintlify.app host)
- [x] A2. `docs/index.mdx`: `title` → `Dynamic versioning from git for every commit`
- [x] A3. `docs/cli/zerv.mdx`: `title` → query-shaped (candidate: `zerv CLI commands and global flags`), keep short `sidebarTitle`
- [x] A4. Label titles → query-shaped; `sidebarTitle` keeps the short label. Candidates below — finalize wording at implementation, keep rendered title (incl. ` - zerv` suffix) between 50–60 chars where possible:

    Final wording (rendered chars incl. suffix): Installation → `Install zerv from PyPI, crates.io, or binaries` (53); Quickstart → `zerv quickstart: your first version from git` (50); Flow → `zerv flow: branch-based versioning automation` (53); Version → `zerv version: manual versioning pipeline` (47); Config file → `zerv.toml config file: discovery and precedence` (54, matches page's Discovery/Precedence H2s); Formats → `Version formats and Tera templates` (41); Schema system → `Version schema system and 22 presets` (43, 22 verified in two page descriptions); Troubleshooting hub → `zerv errors and troubleshooting` (38); GitHub Actions → `Version every GitHub Actions build` (41). All 11 changed pages got `sidebarTitle` = old label.

    | Page                  | Candidate title                                |
    | --------------------- | ---------------------------------------------- |
    | Installation          | Install zerv from PyPI, crates.io, or binaries |
    | Quickstart            | zerv quickstart: first version from git        |
    | Flow (concept)        | zerv flow: branch-based version automation     |
    | Version (concept)     | zerv version: manual version pipeline          |
    | Config file           | zerv.toml config file reference                |
    | Formats and templates | Version formats and Tera templates             |
    | Schema system         | Version schema system and 22 presets           |
    | Troubleshooting hub   | zerv errors and troubleshooting                |
    | GitHub Actions        | Version every GitHub Actions build             |

- [x] A5. Sweep: every rendered title unique; no page where title + suffix duplicates the brand word (`X - zerv - zerv` pattern). Verified 2026-08-24: all 17 rendered titles unique (min 15, max 54 chars); the two exact-brand duplicates (index, cli/zerv) fixed in A2/A3; `bake docs-check` clean.

## Phase B — troubleshooting split

- [x] B1. Read `docs/troubleshooting/index.mdx` in full; confirm the 5-error / 4-behavioral classification before splitting
- [x] B2. Create 5 per-error pages under `docs/troubleshooting/`:
    - `title` = exact error string as the CLI emits it (verbatim — these match paste-into-Google queries)
    - short `sidebarTitle`
    - body: exact error, cause, fix, code examples (test-backed per docs standards where examples exist)
    - Pages: `No version tags are reachable from HEAD`, `VCS not found: git`, `Unknown schema`, `Conflicting options`, `Config parse error`

    Done 2026-08-24. Deviation from the locked page list: the VCS page title is
    `VCS not found: Not in a git repository` (quoted, has `: `), not `VCS not found: git` — the
    hub's old string was wrong; every production construction site emits payload
    `Not in a git repository (--source git)` (`src/vcs/git.rs:89`, `src/vcs/mod.rs:44`,
    `mod.rs:99`). Verbatim rule wins. All 5 pages carry reproduce-blocks backed by existing
    tests: git.rs (`test_git_source_no_tag_version`, `test_git_source_not_a_git_repo`),
    none.rs (`test_none_source_with_distance`), docs/troubleshooting.rs (unknown schema,
    conflicting options), config_file/mod.rs (`unknown_field_errors_loud`,
    `malformed_config_errors_loud`). Frontmatter `description` values needed double quotes
    (all contain `zerv error: ...`).

- [x] B3. Hub keeps the 4 behavioral sections (help-and-exits, PEP440 vs SemVer output, Docker rejects `+`, stdin piping); links all 5 error pages with the error strings as anchor text
- [x] B4. Wire new pages into `docs.json` navigation (troubleshooting group)
- [x] B5. Check existing inbound links to the hub still resolve (README, other docs pages deep-linking `troubleshooting#anchors`) — only inbound link is `README.md:86` → `https://zerv.wisl.dev/troubleshooting` (no anchor); zero `troubleshooting#` anchor links repo-wide. Safe. Verified live: all 5 pages 200 + render titles, sidebar shows all 6 entries, hub errors index renders, `bake docs-check` clean.

## Phase C — comparison + internal links

- [x] C1. `getting-started/why-zerv.mdx`: verify comparison sections actually cover semantic-release, setuptools-scm, and git describe (README claims all three) — coverage was 1-bullet thin for setuptools-scm and git describe; now full per-tool H2 sections. README lines 35 + 80 updated to list dunamai too.
- [x] C2. Add dunamai section: honest table row + short paragraph (Python library, PEP440-centric, no CLI — state real differences, no strawman)

    Done 2026-08-24, with correction: the task note "no CLI" was wrong — dunamai ships a
    console script (`dunamai from git`, `dunamai check`; verified against
    github.com/mtkennerly/dunamai). Honest differences written: Python env vs static Rust
    binary; dunamai supports more VCSs (Mercurial, Subversion, Fossil, …); dunamai pre-release
    comes from the last tag (branch only as `{branch}` format substitution) vs `zerv flow`
    branch-pattern→label+number; ZERV RON payload re-render vs one string per invocation.
    Summary table added under "When to reach for something else" with all four rows.

- [x] C3. Query-shaped H2s on comparison sections (`zerv vs semantic-release`, etc.) — `zerv vs semantic-release` (renamed from "Both tools in one repository"), `zerv vs setuptools-scm`, `zerv vs dunamai`, `zerv vs git describe`. All verified rendering live.
- [x] C4. Internal-link audit: no orphan pages (content-body links only — footer/sidebar don't count), descriptive first-mention anchors, links in both directions — audit script (md links + Card hrefs, frontmatter/test-comments stripped) over 22 pages. Fixed: 5 error pages now link back to hub (bidirectional); quickstart "Where to go next" links `/cli/zerv`. Remaining orphan: homepage only (nav root — accepted exception). Anchors spot-checked descriptive. `bake docs-check` clean.

## Phase D — verify + ship

- [ ] D1. `bake docs-check`
- [ ] D2. Docs test sweep for touched examples: `ZERV_TEST_NATIVE_GIT=false ZERV_TEST_DOCKER=true cargo +nightly test --test integration -- docs`
- [ ] D3. Open single PR, review, merge
- [ ] D4. Post-deploy curl checks: homepage title, `og:image` host is `zerv.wisl.dev`, `cli/zerv` title, sitemap page count 17 → 22
- [ ] D5. GSC URL inspection on new error pages and `/skill.md` (`seo.indexing` defaults to `navigable` and `skill.md` is not in nav — if not indexed, decide between `seo.indexing: "all"` or accepting non-indexed)

## Off-repo (user actions, no PR)

- [ ] GSC: submit `https://zerv.wisl.dev/sitemap.xml` (Sitemaps page)
- [ ] Confirm Bing Webmaster copied the sitemap via GSC import
- [ ] ~2026-09-07: pull GSC baseline (indexed pages, impressions, queries)

## Verification commands

```bash
bake docs-check
curl -s https://zerv.wisl.dev/ | grep '<title>'              # rendered title
curl -s https://zerv.wisl.dev/ | grep 'og:image'             # og image host
curl -s https://zerv.wisl.dev/sitemap.xml | grep -c '<loc>'  # page count
```
