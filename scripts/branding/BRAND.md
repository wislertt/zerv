# zerv brand

Source of truth for design decisions: color system, mark geometry, tagline, design constants. Generator tooling lives in [`README.md`](./README.md). Change a value here first, then regenerate.

Locked 2026-08-20 through companion-screen review (screens v17–v29).

## Mark

Lightning-Z — a `Z` drawn as a single fast stroke, leaning left. Source of truth is [`gen_mark.py`](./gen_mark.py): a parametric edge model (12 vertex dots on a 100×100 grid, one fillet radius per dot). Output lands in `.cache/zerv-mark.svg` (committed copy: `docs/img/brand/zerv-mark.svg`); downstream assets derive from it. Never edit the SVG by hand — change a knob, regenerate.

- Procedural geometry: straight edges + tangent arcs only, one continuous outline (no Beziers, no width profile). Bottom half = top half rotated exactly 180° about the grid center, so central symmetry and end-to-end connectedness hold by construction; both bar edges stay one straight line each through the mirror seam
- Sharp corners at the bolt and bar top; peaky compound ends (two internally tangent arcs, gentle blend into tight peak — curvature ramps to a point) at the flick tip and both bar ends; small tangent fillet-cap at the root
- Lean: −8°, baked into the generated path coordinates (the grid frame maps over the leaned design space)
- Geometry: viewBox `56 69 418 414` — **cropped tight to ink** (was `0 0 571 536` on the original trace; ~10–18% empty margin per side shrank the mark inside lockups, screens v27–v29). Ratios below are honest ink-height ratios

## Color system

One accent family + warm neutrals. No second hue.

### Accent — fixed, never changes

- `gold #FBBF24` — pale gold. The mark, favicon mark, interactive accents. **Single hex in both light and dark mode** (9.9:1 on `dark`, deliberate: one color = simpler brand; the glare of gold-on-dark is acceptable)

### Neutrals

- `ink #16232B` — ink on light: wordmark, body text on light surfaces
- `ink-dark #F2F5F7` — ink on dark: wordmark, body text on dark surfaces
- `paper #F7F5F0` — light mode surface
- `dark #12212A` — dark mode surface, favicon square
- `glow-light #FFFFFF` / `glow-dark #1B3242` — radial glow: OG card center wash
- `tag-ink #5C6B74` on light / `#A8B7C0` on dark — OG tagline, secondary text

### Rules

- Gold = brand + interactive. Neutrals = words and surfaces
- Gold on `paper` is ~1.7:1 — fine for the mark (large shape), never for text. Text stays `ink`/`ink-dark`
- Surfaces are tinted near-neutrals, never pure `#000000`/`#FFFFFF`
- Mark renders in `gold` unchanged on any surface, no variants

## Typography

- Wordmark font: **Sora, wght 600**, lowercase `zerv` — geometric, slightly futuristic; pairs with the lightning mark. Bundled as `fonts/Sora.ttf` (variable), instantiated at 600, outlines only in committed assets
- OG tagline: Sora 500
- Docs/UI body font: decided in Phase 2 (docs site), not here

## Tagline

Two versions, one voice. Locked 2026-08-20 (screens v30–v31, category survey: dunamai/GitVersion sell derivation, "automatically" is semantic-release's claim — never use it). Never paraphrase, never add a third.

**Card** (OG card, README subtitle, anywhere the logo appears):

```
Dynamic versioning from git. Every commit gets its version.
```

**Metadata** (`Cargo.toml`/`pyproject.toml` description, GitHub About):

```
Dynamic versioning from git. Every commit gets its version. Keep semantic-release on main. Let zerv prerelease every other branch.
```

Card stays tool-rival-free; metadata carries the semantic-release keyword as partner, not rival ("Keep… on main" presumes the reader already runs it).

## Design constants

- Lockup: mark ink height = 1.65 × text ink height, gap = 0.22 × mark height, mark optically centered on x-height midline (ratio picked on cropped seed, screen v29)
- Favicon: 512 grid, dark `#12212A` rounded square (rx 115), ink-tight mark scaled 0.85 centered (fills canvas, picked screen v32) — reads at 16px
- Apple-touch: same square, mark scaled 0.85 (rx 115 ≈ iOS corner mask, no extra headroom needed)
- OG card: lockup 640w, tagline one line fit to 960w, gap 56 (tagline size picked screen v33)
- Raster legibility (vision-checked): square favicon crisp at 16px; raw mark honest at 32px+, murky at 16px — never ship raw mark as favicon

## Asset usage

README logo URLs will point at `cdn.jsdelivr.net` (raw GitHub SVGs are blocked in `<img>` on PyPI — same convention as bakefile). Applied in Phase 3 (README shrink).
