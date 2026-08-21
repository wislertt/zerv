"""Generate lockup: mark left + wordmark right, one SVG (light + dark).

Mark embeds as nested <svg> scaled by viewBox; text reuses wordmark parts.
Run: uv run python3 gen_lockup.py
"""

import math
import re
from pathlib import Path

from gen_wordmark import FONT, INK, INK_DARK, load, wordmark_parts

MARK = "seed/zerv-mark.svg"
MARK_TEXT_RATIO = 1.65  # mark ink height vs text ink height (seed viewBox cropped to ink)
GAP_MARK_RATIO = 0.22  # gap vs mark height
PAD = 2


def mark_inner():
    with open(MARK) as f:
        src = f.read()
    m = re.search(r'viewBox="([-\d. ]+)"', src)
    if m is None:
        raise ValueError(f"viewBox not found in {MARK}")
    vb = m.group(1)
    vw, vh = float(vb.split()[2]), float(vb.split()[3])
    inner = src[src.index(">") + 1 : src.rindex("</svg>")].rstrip()
    return vb, vw, vh, inner


def build(ink):
    font = load(*FONT)
    ds, boxes, xh = wordmark_parts(font)
    tx0 = min(b[0] for b in boxes)
    tx1 = max(b[2] for b in boxes)
    ty0 = min(b[1] for b in boxes)
    ty1 = max(b[3] for b in boxes)

    text_h = ty1 - ty0
    mark_h = MARK_TEXT_RATIO * text_h
    gap = GAP_MARK_RATIO * mark_h
    vb, mvw, mvh, inner = mark_inner()
    mark_w = mvw * mark_h / mvh

    cy = -xh / 2  # optical center: x-height midline, baseline at y=0
    mark_top = cy - mark_h / 2
    text_dx = mark_w + gap - tx0

    x0 = min(0.0, text_dx + tx0)
    x1 = max(mark_w, text_dx + tx1)
    y0 = min(mark_top, ty0)
    y1 = max(mark_top + mark_h, ty1)
    vx0, vy0 = math.floor(x0 - PAD), math.floor(y0 - PAD)
    vx1, vy1 = math.ceil(x1 + PAD), math.ceil(y1 + PAD)
    w, h = vx1 - vx0, vy1 - vy0
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vx0} {vy0} {w} {h}" '
        f'width="{w}" height="{h}" role="img" aria-label="zerv logo">\n'
        f'  <svg x="0" y="{mark_top:.1f}" width="{mark_w:.1f}" '
        f'height="{mark_h:.1f}" viewBox="{vb}">\n'
        f"{inner}\n"
        "  </svg>\n"
        f'  <g transform="translate({text_dx:.1f} 0)">\n'
        f'    <path d="{" ".join(ds)}" fill="{ink}"/>\n'
        "  </g>\n"
        "</svg>"
    )


def main():
    for suffix, ink in (("", INK), ("-dark", INK_DARK)):
        svg = build(ink)
        out = Path(f".cache/zerv-lockup{suffix}.svg")
        out.write_text(svg + "\n")
        print(f"wrote {out} ({len(svg)} bytes)")


if __name__ == "__main__":
    main()
