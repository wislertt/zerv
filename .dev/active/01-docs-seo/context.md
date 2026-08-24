# Context: Docs Site SEO — zerv

Companion to a future `tasks.md` (checklist). This file holds everything an agent needs to plan SEO work: verified live findings, known bug classes, transferable decisions from the bakefile repo's completed SEO pass, and the opportunities specific to this repo. Dates are absolute. Verify anything marked "verify" against the live site before acting — this reflects 2026-08-24 state.

## Project facts

- Repo: `github.com/wislertt/zerv`, main branch `main`.
- Product: CLI that generates version strings from Git state (tags, distance, branch, dirtiness). Runs next to semantic-release: semantic-release owns releases on main, zerv versions every other branch/build/dirty tree.
- Tagline: "Dynamic versioning from git. Every commit gets its version."
- **Dual distribution, two different package names**: Rust crate `zerv` on crates.io, Python package `zerv-version` on PyPI (wheel ships a Rust extension + `import zerv` binding). Also install script and pre-built binaries on GitHub Releases.
- CLI: `zerv` with subcommands `flow`, `version`, `check`, `render`.
- Author: Wisaroot Lertthaweedech (`wisl.dev`). Same author as bakefile and leetcode-py.
- Docs: Mintlify site at `https://zerv.wisl.dev` (custom domain; default deployment host `zerv.mintlify.app`).
- Separate demo repo exists: `github.com/wislertt/zerv-flow` (linked from docs footer) — an additional backlink/landing surface.

## Docs stack and conventions

- Mintlify, `docs/docs.json`. Theme `mint`. Site name `zerv` (rendered title suffix ` - zerv`, 7 chars, so frontmatter titles can run up to ~53 chars).
- 17 pages: index, 3 getting-started, 5 concepts (flow, version, schema-system, formats-and-templates, config-file), 2 cicd (github-actions, semantic-release), 5 cli reference, 1 troubleshooting hub.
- `docs/skill.md` hand-written, served at `/skill.md`, overrides Mintlify's auto-generated file. Do not delete, no MDX-only components.
- `markdown.instructions` in docs.json injected into `llms.txt` / exports. Notable instruction: "Examples on this site are backed by integration tests; copy them verbatim" and exact flag names (`--output-template`, `--dirty` takes no value).
- Docs verification: `bake docs-check` (bakefile.py defines `docs_check` → runs `mintlify broken-links` in `docs/`). `bake docs` runs the dev server.

## Verified live state (2026-08-24)

Checked directly against production:

- `robots.txt`: Mintlify default, clean. AI bots allowed, `Content-Signal: ai-train=yes, search=yes, ai-input=yes`. Do not replace.
- `sitemap.xml`: auto-generated, 17 pages, fresh `lastmod`.
- `llms.txt`: HTTP 200.
- Canonical set globally (`https://zerv.wisl.dev`).
- Organization schema `sameAs` includes GitHub, PyPI, crates.io, wisl.dev (four sources — better than most).
- Custom domain serves `/img/brand/*` (og card asset returns 200).
- README links deeply into docs (flow, version, formats, semantic-release, why-zerv, installation).

## Known bugs (same class as bakefile + leetcode-py, verified live 2026-08-24)

1. **Duplicated title tag.** Homepage renders `<title>zerv - zerv</title>` (page title + site name identical). Fix: keyword title in `docs/index.mdx`, suffix carries brand. Candidate direction: "Git-based dynamic versioning CLI" phrasing — planning agent picks final wording.
2. **og:image resolves to wrong host.** Meta emits `https://zerv.mintlify.app/img/brand/og-card-light.png` because `seo.metatags.og:image` is a relative path. Fix: absolute URL `https://zerv.wisl.dev/img/brand/og-card-light.png` (asset verified serving on custom domain).
3. **Duplicated title on `cli/zerv`.** Title "zerv" renders `zerv - zerv`. Same fix bakefile used for its CLI page ("bakefile CLI").
4. **Product-label titles.** "Installation", "Quickstart", "Troubleshooting", "Flow", "Version", "Config file" are labels, not queries. Same fix pattern: query-shaped titles + `sidebarTitle` where long.

## Keyword space specific to this repo

zerv competes in the "version from git" niche. Real query targets and who owns them now:

- "semantic-release alternative" / running versioning beside semantic-release — `getting-started/why-zerv` + `cicd/semantic-release` already target this. Strongest existing surface.
- "setuptools-scm alternative", "git describe alternative" — why-zerv comparison covers semantic-release, setuptools-scm, git describe (per README). Verify the page sections match all three.
- **dunamai is not mentioned** in README's comparison list. dunamai is the most-used Python "version from git" library and likely owns the head terms ("python version from git", "pep440 from git"). Planning agent should evaluate adding it to the comparison (honest content only).
- Long-tail: "version every commit", "version ci artifacts", "calver from git", "pep440 from git tags", "prerelease versioning feature branches".
- Format keywords already have concept pages: SemVer, PEP440, CalVer, Tera templates (`concepts/formats-and-templates`).

## Troubleshooting split opportunity

`troubleshooting/index.mdx` has 9 H2 sections on one page — same shape bakefile had before its split. Errors from a versioning CLI are exact strings devs paste into Google (schema errors, config errors, git state errors). Same treatment: per-error pages with the exact error string as title, `sidebarTitle` short, hub links all. Thin/behavioral sections stay on the hub. (Read the actual sections before splitting — count of error-string sections vs behavioral ones determines page count.)

## Measurement status (head start — mostly done)

- GSC **Domain property `wisl.dev` verified 2026-08-24** via DNS TXT at Spaceship registrar. Domain property covers ALL subdomains — `zerv.wisl.dev` already verified. No new verification needed.
- Bing Webmaster imported from GSC same day (covers Bing + DuckDuckGo + Yahoo).
- **Pending:** submit `https://zerv.wisl.dev/sitemap.xml` in GSC (Sitemaps page); confirm Bing copied it via import.
- **Pending:** GSC baseline (~2 weeks after sitemap submit): indexed pages, impressions, queries.

## Transferable decisions from the bakefile SEO pass (2026-08-24, same author, same Mintlify setup)

Applied and verified on bakefile.wisl.dev — reuse the patterns:

- Homepage/landing `title` frontmatter = keywords, site-name suffix carries brand. Never put the brand word in the title when the suffix already appends it.
- `sidebarTitle` frontmatter decouples long SEO titles from sidebar labels. Mintlify supports it natively.
- `og:image` must be absolute; relative resolves against the mintlify.app host.
- Query-shaped H2s on comparison pages ("X vs Y") match comparison queries. FAQ sections only with genuinely matching Q&A.
- Troubleshooting: per-error pages, exact error string as title, hub links with error strings as anchors.
- Internal linking: no orphan pages (footer/sidebar don't count — content links only), first-mention descriptive anchors, both directions.
- Titles: unique, 50–60 chars rendered (including the ` - zerv` suffix).
- Verification: `bake docs-check` after every docs edit; `curl -s <url> | grep '<title>'` after deploy.

## Off-repo SEO context (separate track, informational)

- No blog on the docs site (author decision). Articles live on wisl.dev (planned Astro blog), link into all docs sites. Cross-posting to Medium/dev.to only with canonical → wisl.dev.
- Backlink surfaces this repo uniquely has: crates.io page, PyPI `zerv-version` page, the separate `zerv-flow` demo repo, GitHub Releases. Launch venues for this niche: r/rust, r/python HN threads on release tooling, semantic-release discussions.
- Both competitor keywords (setuptools-scm, semantic-release ecosystems) have active communities where honest answers earn links.

## Verification commands

```bash
bake docs-check                                            # broken links
curl -s https://zerv.wisl.dev/ | grep '<title>'           # rendered title
curl -s https://zerv.wisl.dev/ | grep 'og:image'          # og image host
curl -s https://zerv.wisl.dev/sitemap.xml | grep -c '<loc>'  # page count
```

## Non-goals (decided, do not relitigate)

- No blog section on the docs site.
- No custom `robots.txt` / `sitemap.xml` (Mintlify defaults correct).
- No structured-data changes beyond Mintlify's automatic JSON-LD + existing organization block.
- Not moving docs off the subdomain.
