# Branding generators

Tooling that generates the brand assets (mark, wordmark, lockup, favicons, OG card). Design decisions — colors, mark geometry, tagline, constants — live in [`BRAND.md`](./BRAND.md). Outputs are committed to `docs/img/brand/`. Regenerate only when a design constant changes, then copy the files over.

The mark is generated: [`gen_mark.py`](./gen_mark.py) is a parametric edge model and the single source of truth for the mark's geometry. Its output, `.cache/zerv-mark.svg`, is what every other generator reads; the committed copy lives at `docs/img/brand/zerv-mark.svg`.

## Files

| File              | Produces                                                                                                  |
| ----------------- | --------------------------------------------------------------------------------------------------------- |
| `gen_mark.py`     | `zerv-mark.svg` (parametric mark; stdlib only)                                                            |
| `gen_wordmark.py` | `zerv-wordmark.svg`, `zerv-wordmark-dark.svg`                                                             |
| `gen_lockup.py`   | `zerv-lockup.svg`, `zerv-lockup-dark.svg` (reads the seed mark)                                           |
| `gen_favicons.py` | `favicon.ico` + `favicon-{16,32,48}.png`, `apple-touch-icon.png` (dark square), `zerv-mark-{256,512}.png` |
| `gen_og.py`       | `og-card-light.png`, `og-card-dark.png` (1200×630, brand surfaces + lockup + Sora tagline)                |

## Regenerate

Run from this directory. Python deps are pulled on the fly by `uv`; PNG rendering uses system `rsvg-convert` + `magick` (`brew install librsvg imagemagick`).

```bash
cd scripts/branding
python3 gen_mark.py
uv run python3 gen_wordmark.py
uv run python3 gen_lockup.py
python3 gen_favicons.py
uv run python3 gen_og.py
cp .cache/zerv-*.svg .cache/zerv-mark-*.png .cache/favicon* \
  .cache/apple-touch-icon.png .cache/og-card-*.png ../../docs/img/brand/
```

Generated files land in `.cache/` (gitignored). Dependency chain: `gen_mark.py` writes the mark SVG, lockup imports wordmark parts and reads the mark, `gen_og.py` reads the generated lockups. Run in the order above.

`preview.html` renders the `.cache/` outputs for side-by-side review. Serve: `python3 -m http.server 8743` from this directory.

## Font license

`fonts/Sora.ttf` is [Sora](https://fonts.google.com/specimen/Sora) by Sora Type Foundry / Jonathan Barnbrook, licensed under the [SIL Open Font License 1.1](./OFL.txt). Bundling the font with its license file satisfies the license terms. The committed assets contain outlined paths only.
