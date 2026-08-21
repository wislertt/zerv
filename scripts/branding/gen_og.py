"""Generate OG social card: brand surface bg, lockup, Sora tagline.

Run: uv run python3 gen_og.py  (SVG build)
then rsvg-convert + magick for the PNG render (no cairo dependency).
"""

import re
import subprocess
from pathlib import Path

from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.recordingPen import DecomposingRecordingPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from gen_wordmark import load

CACHE = Path(".cache")

W, H = 1200, 630
TAGLINE = "Dynamic versioning from git. Every commit gets its version."
TAG_FONT = ("fonts/Sora.ttf", {"wght": 500})
LOCKUP_W = 640
TAG_W = 960
GAP = 56  # lockup bottom -> tagline cap top
SS = 2  # supersample render, LANCZOS back down for crisp edges

MODES = {
    "dark": {
        "bg": "#12212A",
        "glow": "#1B3242",
        "lockup": "zerv-lockup-dark.svg",
        "tag_ink": "#A8B7C0",
    },
    "light": {
        "bg": "#F7F5F0",
        "glow": "#FFFFFF",
        "lockup": "zerv-lockup.svg",
        "tag_ink": "#5C6B74",
    },
}


def lockup_inner(name):
    src = (CACHE / name).read_text()
    m = re.search(r'viewBox="([-\d. ]+)"', src)
    if m is None:
        raise ValueError(f"viewBox not found in .cache/{name}")
    vb = m.group(1)
    vw, vh = float(vb.split()[2]), float(vb.split()[3])
    inner = src[src.index(">") + 1 : src.rindex("</svg>")].rstrip()
    return vb, vw, vh, inner


def text_paths(font, text, size):
    """Glyph outlines at `size`, flat baseline y=0. Returns (path d, bounds)."""
    scale = size / font["head"].unitsPerEm
    cmap = font.getBestCmap()
    hmtx = font["hmtx"]
    gs = font.getGlyphSet()
    ds = []
    x = 0.0
    bp = BoundsPen(None)
    for ch in text:
        gname = cmap[ord(ch)]
        rp = DecomposingRecordingPen(gs)
        gs[gname].draw(rp)
        t = (scale, 0, 0, -scale, x, 0)
        sp = SVGPathPen(None)
        for op in rp.value:
            getattr(TransformPen(sp, t), op[0])(*op[1])
        ds.append(sp.getCommands())
        for op in rp.value:
            getattr(TransformPen(bp, t), op[0])(*op[1])
        x += hmtx[gname][0] * scale
    return " ".join(ds), bp.bounds


def build(font, mode):
    m = MODES[mode]
    vb, vw, vh, inner = lockup_inner(m["lockup"])
    lw = LOCKUP_W
    lh = lw * vh / vw

    tag_d, (tx0, ty0, tx1, ty1) = text_paths(font, TAGLINE, 100)
    f = TAG_W / (tx1 - tx0)
    tag_h = f * (ty1 - ty0)

    block_h = lh + GAP + tag_h
    top = (H - block_h) / 2
    lx, ly = (W - lw) / 2, top
    tag_base = top + lh + GAP + f * ty1
    tag_tx = (W - TAG_W) / 2 - f * tx0

    return "\n".join(
        [
            f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}">',
            "  <defs>",
            '    <radialGradient id="glow" cx="0.5" cy="0.45" r="0.75">',
            f'      <stop offset="0%" stop-color="{m["glow"]}"/>',
            # plateau: flat bg from 55% out so card edges land exactly on the
            # page surface color (docs hero), glow stays a center-only wash.
            f'      <stop offset="55%" stop-color="{m["bg"]}"/>',
            f'      <stop offset="100%" stop-color="{m["bg"]}"/>',
            "    </radialGradient>",
            "  </defs>",
            f'  <rect width="{W}" height="{H}" fill="url(#glow)"/>',
            f'  <svg x="{lx:.1f}" y="{ly:.1f}" width="{lw:.1f}" height="{lh:.1f}" viewBox="{vb}">',
            inner,
            "  </svg>",
            f'  <g transform="translate({tag_tx:.1f} {tag_base:.1f}) scale({f:.4f})">',
            f'    <path d="{tag_d}" fill="{m["tag_ink"]}"/>',
            "  </g>",
            "</svg>",
        ]
    )


def render(svg, name):
    big = CACHE / f"{name}.ss.png"
    out = CACHE / name
    subprocess.run(
        ["rsvg-convert", "-w", str(W * SS), "-h", str(H * SS), "-o", str(big)],
        input=svg.encode(),
        check=True,
    )
    subprocess.run(
        ["magick", str(big), "-resize", f"{W}x{H}", "-filter", "LANCZOS", str(out)],
        check=True,
    )
    big.unlink()
    print(f"wrote {out} ({W}x{H}, {out.stat().st_size // 1024}K)")


def main():
    CACHE.mkdir(exist_ok=True)
    tag_font = load(*TAG_FONT)
    for mode in MODES:
        render(build(tag_font, mode), f"og-card-{mode}.png")


if __name__ == "__main__":
    main()
