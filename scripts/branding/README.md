# Branding generators

Tooling that generates the brand assets (mark, wordmark, lockup, favicons, OG card). Design decisions — colors, mark geometry, tagline, constants — live in [`BRAND.md`](./BRAND.md). Outputs are committed to `docs/img/brand/`. Regenerate only when a design constant changes, then copy the files over.

The mark itself is not generated: [`seed/zerv-mark.svg`](./seed/zerv-mark.svg) is hand-locked traced artwork and the single source of truth for the mark. Every generator reads it.

## Files

| File                 | Produces                                                                                                  |
| -------------------- | --------------------------------------------------------------------------------------------------------- |
| `seed/zerv-mark.svg` | mark source of truth (input, not generated)                                                               |
| `gen_wordmark.py`    | `zerv-wordmark.svg`, `zerv-wordmark-dark.svg`                                                             |
| `gen_lockup.py`      | `zerv-lockup.svg`, `zerv-lockup-dark.svg` (reads the seed mark)                                           |
| `gen_favicons.py`    | `favicon.ico` + `favicon-{16,32,48}.png`, `apple-touch-icon.png` (dark square), `zerv-mark-{256,512}.png` |
| `gen_og.py`          | `og-card-light.png`, `og-card-dark.png` (1200×630, brand surfaces + lockup + Sora tagline)                |

## Regenerate

Run from this directory. Python deps are pulled on the fly by `uv`; PNG rendering uses system `rsvg-convert` + `magick` (`brew install librsvg imagemagick`).

```bash
cd scripts/branding
uv run python3 gen_wordmark.py
uv run python3 gen_lockup.py
python3 gen_favicons.py
uv run python3 gen_og.py
cp .cache/zerv-*.svg .cache/favicon* .cache/apple-touch-icon.png \
  .cache/og-card-*.png ../../docs/img/brand/
cp seed/zerv-mark.svg ../../docs/img/brand/zerv-mark.svg
```

Generated files land in `.cache/` (gitignored). Dependency chain: lockup imports wordmark parts and reads the seed mark, `gen_og.py` reads the generated lockups. Run in the order above.

`preview.html` renders the `.cache/` outputs for side-by-side review. Serve: `python3 -m http.server 8743` from this directory.

## Font license

`fonts/Sora.ttf` is [Sora](https://fonts.google.com/specimen/Sora) by Sora Type Foundry / Jonathan Barnbrook, licensed under the [SIL Open Font License 1.1](./OFL.txt). Bundling the font with its license file satisfies the license terms. The committed assets contain outlined paths only.
