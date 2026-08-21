"""Generate favicons + standalone mark PNGs from the seed mark.

favicon = dark rounded square + pale-gold mark (reads at 16px).
Run: plain python3 + rsvg-convert + magick. No font dependency.
"""

import re
import subprocess
from pathlib import Path

CACHE = Path(".cache")

DARK = "#12212A"
SIZES = (16, 32, 48)
ICO_SOURCES = ("favicon-16.png", "favicon-32.png", "favicon-48.png")
APPLE_TOUCH = 180
MARK_PNGS = (256, 512)

# 512 design grid: ink-tight mark scaled 0.85 centered, corners keep rx 115
MARK_SCALE = 0.85
CANVAS = 512
RX = 115
APPLE_SCALE = 0.85  # rx 115 matches iOS corner mask, no extra headroom needed


def seed_path():
    src = Path("seed/zerv-mark.svg").read_text()
    m = re.search(r'viewBox="([-\d. ]+)"', src)
    if m is None:
        raise ValueError("viewBox not found in seed/zerv-mark.svg")
    vb = m.group(1)
    vw, vh = float(vb.split()[2]), float(vb.split()[3])
    inner = src[src.index("<g") : src.rindex("</svg>")].rstrip()
    return vb, vw, vh, inner


def square_svg(mark_scale):
    vb, vw, vh, inner = seed_path()
    w = vw * mark_scale
    h = vh * mark_scale
    x = (CANVAS - w) / 2
    y = (CANVAS - h) / 2
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {CANVAS} {CANVAS}">\n'
        f'  <rect width="{CANVAS}" height="{CANVAS}" rx="{RX}" fill="{DARK}"/>\n'
        f'  <svg x="{x:.1f}" y="{y:.1f}" width="{w:.1f}" height="{h:.1f}" viewBox="{vb}">\n'
        f"{inner}\n"
        "  </svg>\n"
        "</svg>\n"
    )


def mark_only_svg(size):
    vb, vw, vh, inner = seed_path()
    side = max(vw, vh)
    x = (side - vw) / 2
    y = (side - vh) / 2
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {side} {side}" '
        f'width="{size}" height="{size}">\n'
        f'  <svg x="{x:.1f}" y="{y:.1f}" width="{vw:.1f}" height="{vh:.1f}" viewBox="{vb}">\n'
        f"{inner}\n"
        "  </svg>\n"
        "</svg>\n"
    )


def rsvg(svg, out, width):
    subprocess.run(
        ["rsvg-convert", "-w", str(width), "-h", str(width), "-o", str(out)],
        input=svg.encode(),
        check=True,
    )
    print(f"wrote {out} ({width}x{width})")


def main():
    CACHE.mkdir(exist_ok=True)
    (CACHE / "zerv-mark.svg").write_text(Path("seed/zerv-mark.svg").read_text())
    print("wrote .cache/zerv-mark.svg (seed copy)")
    sq = square_svg(MARK_SCALE)
    for s in SIZES:
        rsvg(sq, CACHE / f"favicon-{s}.png", s)
    rsvg(square_svg(APPLE_SCALE), CACHE / "apple-touch-icon.png", APPLE_TOUCH)
    subprocess.run(
        ["magick", *[str(CACHE / n) for n in ICO_SOURCES], str(CACHE / "favicon.ico")],
        check=True,
    )
    print(f"wrote {CACHE / 'favicon.ico'}")
    for s in MARK_PNGS:
        rsvg(mark_only_svg(s), CACHE / f"zerv-mark-{s}.png", s)


if __name__ == "__main__":
    main()
