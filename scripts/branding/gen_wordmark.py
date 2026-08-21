"""Generate zerv wordmark: Sora 600 "zerv" outlines -> SVG.

Run: uv run python3 gen_wordmark.py
"""

import math
from pathlib import Path

from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.recordingPen import DecomposingRecordingPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

TEXT = "zerv"
SIZE = 160
INK = "#16232B"
INK_DARK = "#F2F5F7"
PAD = 4
CACHE = Path(".cache")

FONT = ("fonts/Sora.ttf", {"wght": 600})


def load(path, overrides):
    font = TTFont(path)
    if "fvar" in font:
        axes = {a.axisTag: a.defaultValue for a in font["fvar"].axes}
        axes.update(overrides)
        font = instantiateVariableFont(font, axes, inplace=True)
    return font


def contours_of(glyph_set, gname):
    rp = DecomposingRecordingPen(glyph_set)
    glyph_set[gname].draw(rp)
    conts, cur = [], []
    for op in rp.value:
        cur.append(op)
        if op[0] == "closePath":
            conts.append(cur)
            cur = []
    if cur:
        conts.append(cur)
    return conts


def replay(cont, pen):
    for name, args in cont:
        getattr(pen, name)(*args)


def cont_bounds(cont, transform):
    bp = BoundsPen(None)
    replay(cont, TransformPen(bp, transform))
    return bp.bounds


def wordmark_parts(font):
    """Return (glyph path cmds, ink boxes, x-height) at SIZE, baseline y=0."""
    upem = font["head"].unitsPerEm
    scale = SIZE / upem
    cmap = font.getBestCmap()
    hmtx = font["hmtx"]
    gs = font.getGlyphSet()

    ds = []
    boxes = []
    x = 0.0
    for ch in TEXT:
        gname = cmap[ord(ch)]
        for cont in contours_of(gs, gname):
            t = (scale, 0, 0, -scale, x, 0)
            sp = SVGPathPen(None)
            replay(cont, TransformPen(sp, t))
            ds.append(sp.getCommands())
            boxes.append(cont_bounds(cont, t))
        x += hmtx[gname][0] * scale
    return ds, boxes, font["OS/2"].sxHeight * scale


def build(font, ink):
    ds, boxes, _xh = wordmark_parts(font)
    xs0 = min(b[0] for b in boxes)
    ys0 = min(b[1] for b in boxes)
    xs1 = max(b[2] for b in boxes)
    ys1 = max(b[3] for b in boxes)
    vx0, vy0 = math.floor(xs0 - PAD), math.floor(ys0 - PAD)
    vx1, vy1 = math.ceil(xs1 + PAD), math.ceil(ys1 + PAD)
    w, h = vx1 - vx0, vy1 - vy0
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vx0} {vy0} {w} {h}" '
        f'width="{w}" height="{h}" role="img" aria-label="zerv wordmark">\n'
        f'  <path d="{" ".join(ds)}" fill="{ink}"/>\n'
        "</svg>"
    )


def main():
    font = load(*FONT)
    CACHE.mkdir(exist_ok=True)
    for suffix, ink in (("", INK), ("-dark", INK_DARK)):
        svg = build(font, ink)
        out = CACHE / f"zerv-wordmark{suffix}.svg"
        out.write_text(svg + "\n")
        print(f"wrote {out} ({len(svg)} bytes)")


if __name__ == "__main__":
    main()
