"""Generate the lightning-Z mark SVG from a parametric edge model.

The mark is ONE continuous outline: straight edges + tangent circular arcs,
no skeleton, no width profile, no Beziers. The top half is a 12-dot vertex
ring on a 100x100 grid frame (y down); the bottom half is that ring rotated
exactly 180 deg about the grid center (50, 50), so central mirror symmetry
and end-to-end connectedness hold by construction. Both subpaths wind the
same way; nonzero fill unions them across the shared bar overlap.

Each vertex carries a fillet radius knob (all zero in the locked design --
the silhouette's character lives in the two compound ends). The flick tip
and each bar end are TWO internally tangent arcs, a gentle blend arc into a
tight peak arc, so curvature ramps 0 -> 1/rA -> 1/rT -> 0: peaky ends that
feather into the straight edges, G1 everywhere by construction. The seam
rule (V9 = rot180(V8), V10 derived collinear through V9) keeps both bar
edges dead straight across the mirror line -- one straight line each.

Output format (svg > g > path) stays compatible with
gen_lockup/gen_favicons/gen_og. Design decisions live in BRAND.md.

Locked by the 2026-08-23 edge-model campaign (round r6, user-picked r7e
apex variant); the knobs below carry the winner at full precision.

Run: python3 gen_mark.py  (stdlib only)
"""

import math
from itertools import pairwise
from pathlib import Path

FILL = "#FBBF24"
OUT = ".cache/zerv-mark.svg"

# Grid frame: 100x100 user-facing coordinates over the mark's design space.
FRAME_X, FRAME_Y, FRAME_SIDE = 56.0, 67.0, 418.0
GRID_TO_DESIGN = FRAME_SIDE / 100.0
CENTER = (50.0, 50.0)  # 180-deg mirror center in grid coords
CLAMP_FRAC = 0.49  # fillet tangent cap as a fraction of adjacent edges
SAG = 0.02  # arc flattening sagitta in grid units (bounds scan only)

# Vertex ring (grid coords, y down), travel order: flick tip shoulder,
# flick/bottom edge, bolt corner, root-cap dots, notch edge, bar top corner,
# seam dot, then the bar end + V11' that define the bar's two edge lines.
# Positions 8/9 are derived by the seam rule; stored values are placeholders.
VERTS = [
    (3.997, 0.163),  # 0  flick tip shoulder (dead: tip is compound)
    (-0.045, 3.995),  # 1  flick edge / bottom-edge virtual corner
    (25.179, 22.844),  # 2  bolt corner
    (53.236, 42.404),  # 3  root cap dot A (pass-through)
    (49.1169, 36.0742),  # 4  root cap dot B (pass-through)
    (28.189, 19.034),  # 5  notch edge end (sharp)
    (73.186, 41.907),  # 6  bar top corner (sharp)
    (38.203, 46.9095),  # 7  seam cut point V8 (free)
    (61.797, 53.0905),  # 8  V9 = rot180(V8)          (derived)
    (65.5469, 52.5543),  # 9  V10 = V9 - lam*u(V8-V7)  (derived)
    (106.68240420008644, 46.67197573075686),  # 10 W: virtual bar-end corner (off-canvas)
    (6.518, 1.855),  # 11 V11': bar top edge line definer (dead dot)
]
RADII = [0.0] * 12  # fillet knobs; locked design is fully sharp here

CAP_TL = 4.5  # root cap tangent length; cap edge = ring[3] -> ring[4]
PASS_THROUGH = (0, 1, 3, 4)  # dots with no corner of their own
SEAM_FREE, SEAM_ROT, SEAM_BAR = 7, 8, 9
SEAM_LAM = 3.788  # bar bottom edge length V9 -> V10

# Compound flick tip (replaces the vertex-11 corner): tight arc pinned by
# reach to the locked apex, blend arc tangent to the bar top edge line.
TIP_RT, TIP_RA, TIP_EPS = 2.3, 6.0, 0.0
TIP_ANCHOR = (0.9871427470857466, 1.0562480504005478)  # locked apex, grid coords

# Compound bar end (replaces the vertex-10 corner): tight arc placed by
# bisecting q until the arc junction M1 sits at BAR_APEX_Y; blend arc
# tangent to the bar bottom edge.
BAR_RT, BAR_RA, BAR_APEX_Y = 2.9, 8.0, 47.2

TURN_BAR = 2.579  # full corner turn at W, rad (sweep-sum gate)
TURN_TIP = 2.921  # full corner turn at the tip, rad (sweep-sum gate)


def unit(d):
    dlen = math.hypot(d[0], d[1])
    return (d[0] / dlen, d[1] / dlen)


def dot(a, b):
    return a[0] * b[0] + a[1] * b[1]


def rot(p):
    """180-degree mirror about CENTER, in grid coords."""
    return (2 * CENTER[0] - p[0], 2 * CENTER[1] - p[1])


def inward(u):
    """Inward normal of a travel direction: the ring winds so that the
    interior of the half sits at (u.y, -u.x) for every edge's travel dir."""
    return (u[1], -u[0])


def resolved():
    """Full vert list with V9/V10 derived from the seam collinearity rule."""
    vs = list(VERTS)
    v7, v8 = vs[SEAM_FREE - 1], vs[SEAM_FREE]
    u = unit((v8[0] - v7[0], v8[1] - v7[1]))
    v9 = rot(v8)
    vs[SEAM_ROT] = v9
    vs[SEAM_BAR] = (v9[0] - SEAM_LAM * u[0], v9[1] - SEAM_LAM * u[1])
    return vs


def sweep(a, b, c):
    """(span, side) of the short way from radial a to radial b about c."""
    a1 = math.atan2(a[1] - c[1], a[0] - c[0])
    a2 = math.atan2(b[1] - c[1], b[0] - c[0])
    dp = (a2 - a1) % (2 * math.pi)
    dm = (a1 - a2) % (2 * math.pi)
    return (dp, 1) if dp < dm else (dm, -1)


def arc_points(a, b, c, r, side):
    """Flatten an arc to within SAG (grid units); bounds scan only."""
    if r <= 1e-9:
        return []
    a1 = math.atan2(a[1] - c[1], a[0] - c[0])
    a2 = math.atan2(b[1] - c[1], b[0] - c[0])
    if side > 0:
        while a2 < a1:
            a2 += 2 * math.pi
    else:
        while a2 > a1:
            a2 -= 2 * math.pi
    span = abs(a2 - a1)
    a_seg = 2 * math.acos(max(0.0, 1 - SAG / r)) if r > SAG else 0.5
    n = max(1, math.ceil(span / max(a_seg, 1e-6)))
    return [
        (c[0] + r * math.cos(a1 + (a2 - a1) * k / n), c[1] + r * math.sin(a1 + (a2 - a1) * k / n))
        for k in range(1, n + 1)
    ]


def corner(vs, i):
    """Fillet geometry at vertex i: (tangent-a, tangent-b, center, r, side)."""
    n = len(vs)
    p, p0, p1 = vs[i], vs[i - 1], vs[(i + 1) % n]
    u1 = unit((p[0] - p0[0], p[1] - p0[1]))
    u2 = unit((p1[0] - p[0], p1[1] - p[1]))
    cos_t = max(-1.0, min(1.0, u1[0] * u2[0] + u1[1] * u2[1]))
    tau = math.acos(cos_t)
    if tau < 1e-6:
        return None
    t = RADII[i] * math.tan(tau / 2)
    t = min(t, CLAMP_FRAC * math.dist(p, p0), CLAMP_FRAC * math.dist(p1, p))
    r = t / math.tan(tau / 2)
    side = 1 if (u1[0] * u2[1] - u1[1] * u2[0]) >= 0 else -1
    a = (p[0] - t * u1[0], p[1] - t * u1[1])
    b = (p[0] + t * u2[0], p[1] + t * u2[1])
    c = (a[0] - side * r * u1[1], a[1] + side * r * u1[0])
    return a, b, c, r, side


def cap_geom(vs):
    """Root cap: one arc tangent to the edges around the pass-through dots."""
    w2, w3, w4, w5 = vs[2], vs[3], vs[4], vs[5]
    u1 = unit((w3[0] - w2[0], w3[1] - w2[1]))
    u2 = unit((w5[0] - w4[0], w5[1] - w4[1]))
    n1 = (u1[1], -u1[0])
    n2 = (u2[1], -u2[0])
    t1 = (w3[0] - CAP_TL * u1[0], w3[1] - CAP_TL * u1[1])
    denom = 1.0 - dot(n2, n1)
    if abs(denom) < 1e-9:
        return None
    r = dot(n2, (t1[0] - w4[0], t1[1] - w4[1])) / denom
    if r <= 0.05:
        return None
    c = (t1[0] + r * n1[0], t1[1] + r * n1[1])
    t2 = (c[0] - r * n2[0], c[1] - r * n2[1])
    return t1, t2, c, r, u1, u2


def cap_sweep(t1, t2, c, u1, u2):
    a1 = math.atan2(t1[1] - c[1], t1[0] - c[0])
    a2 = math.atan2(t2[1] - c[1], t2[0] - c[0])
    d_out = unit((u1[0] - u2[0], u1[1] - u2[1]))
    for side in (1, -1):
        b1, b2 = a1, a2
        if side > 0:
            while b2 < b1:
                b2 += 2 * math.pi
        else:
            while b2 > b1:
                b2 -= 2 * math.pi
        mid = (b1 + b2) / 2
        m = (math.cos(mid), math.sin(mid))
        if dot(m, d_out) > 0:
            return side, b1, b2
    return 1, a1, a2


def tip_geom(vs):
    """Compound flick tip: bar top edge -arcA- M1 -arcT- flick edge.

    The tight arc's center sits on the flick edge's interior offset at
    distance TIP_RT and lands TIP_EPS inside TIP_ANCHOR; the blend arc is
    tangent to the bar top edge line and internally tangent to the tight
    arc (|c_a - c_t| = rA - rT).
    """
    p, q = vs[11], vs[10]  # V11', W
    v1, v2 = vs[1], vs[2]
    rt, ra, eps = TIP_RT, TIP_RA, TIP_EPS
    u1 = unit((p[0] - q[0], p[1] - q[1]))  # travel toward V11
    u2f = unit((v2[0] - v1[0], v2[1] - v1[1]))
    n1, n2 = inward(u1), inward(u2f)

    bx = v1[0] + rt * n2[0] - TIP_ANCHOR[0]
    by = v1[1] + rt * n2[1] - TIP_ANCHOR[1]
    bq = 2 * (bx * u2f[0] + by * u2f[1])
    cq = bx * bx + by * by - (rt + eps) ** 2
    disc = bq * bq - 4 * cq
    if disc < 0:
        return None
    best = None  # (cand, s): prefer the solution with minimal |s|
    for s in ((-bq + math.sqrt(disc)) / 2, (-bq - math.sqrt(disc)) / 2):
        if not (0.3 <= s <= 6.0):
            continue
        c_t = (v1[0] + s * u2f[0] + rt * n2[0], v1[1] + s * u2f[1] + rt * n2[1])
        ex = p[0] + ra * n1[0] - c_t[0]
        ey = p[1] + ra * n1[1] - c_t[1]
        bq2 = -2 * (ex * u1[0] + ey * u1[1])
        cq2 = ex * ex + ey * ey - (ra - rt) ** 2
        disc2 = bq2 * bq2 - 4 * cq2
        if disc2 < 0:
            continue
        for t in ((-bq2 + math.sqrt(disc2)) / 2, (-bq2 - math.sqrt(disc2)) / 2):
            if not (-2.5 <= t <= 8.0):
                continue
            t_a = (p[0] - t * u1[0], p[1] - t * u1[1])
            c_a = (t_a[0] + ra * n1[0], t_a[1] + ra * n1[1])
            d = math.hypot(c_t[0] - c_a[0], c_t[1] - c_a[1])
            if abs(d - (ra - rt)) > 1e-6 or d < 1e-6:
                continue
            m1 = (c_a[0] + ra * (c_t[0] - c_a[0]) / d, c_a[1] + ra * (c_t[1] - c_a[1]) / d)
            t2p = (c_t[0] - rt * n2[0], c_t[1] - rt * n2[1])
            sa, side_a = sweep(t_a, m1, c_a)
            st, side_t = sweep(m1, t2p, c_t)
            if side_a != side_t:
                continue
            if not (0.1 < sa < 1.9 and 1.2 < st < 3.0):
                continue
            if not abs(sa + st - TURN_TIP) < 0.5:
                continue
            cand = (t_a, c_a, side_a, m1, c_t, side_t, t2p)
            if best is None or abs(s) < abs(best[1]):
                best = (cand, s)
    return None if best is None else best[0]


def bar_geom(vs):
    """Compound bar end: bar bottom edge -arcA- M1 -arcT- bar top edge.

    q slides the tight arc's center along the top edge's interior offset;
    t solves the blend arc's internal tangency to the bottom edge's offset.
    Bisection picks q so the junction M1 sits at BAR_APEX_Y.
    """
    p, a9, e2 = vs[10], vs[9], vs[11]  # W, V10, V11'
    rt, ra, ytar = BAR_RT, BAR_RA, BAR_APEX_Y
    u1 = unit((p[0] - a9[0], p[1] - a9[1]))
    u2 = unit((e2[0] - p[0], e2[1] - p[1]))
    n1, n2 = inward(u1), inward(u2)

    def geom_from_q(q):
        c_t = (p[0] + q * u2[0] + rt * n2[0], p[1] + q * u2[1] + rt * n2[1])
        t2p = (c_t[0] - rt * n2[0], c_t[1] - rt * n2[1])
        ex = p[0] + ra * n1[0] - c_t[0]
        ey = p[1] + ra * n1[1] - c_t[1]
        bq2 = -2 * (ex * u1[0] + ey * u1[1])
        cq2 = ex * ex + ey * ey - (ra - rt) ** 2
        disc2 = bq2 * bq2 - 4 * cq2
        if disc2 < 0:
            return None
        best = None  # (cand, t): prefer the minimal-t tangency root
        for t in ((-bq2 + math.sqrt(disc2)) / 2, (-bq2 - math.sqrt(disc2)) / 2):
            if not (0.5 <= t <= 34.0):
                continue
            t_a = (p[0] - t * u1[0], p[1] - t * u1[1])
            c_a = (t_a[0] + ra * n1[0], t_a[1] + ra * n1[1])
            d = math.hypot(c_t[0] - c_a[0], c_t[1] - c_a[1])
            if abs(d - (ra - rt)) > 1e-6 or d < 1e-6:
                continue
            m1 = (c_a[0] + ra * (c_t[0] - c_a[0]) / d, c_a[1] + ra * (c_t[1] - c_a[1]) / d)
            sa, side_a = sweep(t_a, m1, c_a)
            st, side_t = sweep(m1, t2p, c_t)
            if side_a != side_t:
                continue
            if not (0.1 < sa < 1.6 and 0.9 < st < 2.4):
                continue
            if not abs(sa + st - TURN_BAR) < 0.45:
                continue
            cand = (t_a, c_a, side_a, m1, c_t, side_t, t2p)
            if best is None or t < best[1]:
                best = (cand, t)
        return None if best is None else best[0]

    # m1.y falls monotonically as q grows; bracket then bisect
    qs = [0.5 + (25.0 - 0.5) * i / 60.0 for i in range(61)]
    feas = [(q, g) for q in qs if (g := geom_from_q(q)) is not None]
    if len(feas) < 2:
        return None
    bracket = None
    for (q1, g1), (q2, g2) in pairwise(feas):
        if (g1[3][1] - ytar) * (g2[3][1] - ytar) <= 0:
            bracket = (q1, q2, g1)
            break
    if bracket is None:
        return None
    q1, q2, g = bracket
    f_lo = g[3][1] - ytar
    for _ in range(60):
        mid = 0.5 * (q1 + q2)
        gm = geom_from_q(mid)
        if gm is None:
            break
        g = gm
        f = gm[3][1] - ytar
        if abs(f) < 1e-7:
            break
        if f * f_lo > 0:
            q1, f_lo = mid, f
        else:
            q2 = mid
    if abs(g[3][1] - ytar) > 0.02:
        return None
    return g


def half_features(vs):
    """Analytic feature list for the half outline, travel order:
    ("line", p0, p1) | ("arc", a, b, c, r, side, is_cap)."""
    feats = []
    prev = None
    i = 0
    while i < len(vs):
        if i == 10:  # compound bar end
            g = bar_geom(vs)
            if g is None:
                return None
            t_a, c_a, side_a, m1, c_t, side_t, t2p = g
            feats.append(("line", prev, t_a))
            feats.append(("arc", t_a, m1, c_a, BAR_RA, side_a, False))
            feats.append(("arc", m1, t2p, c_t, BAR_RT, side_t, False))
            prev = t2p
        elif i == 11:  # compound flick tip
            g = tip_geom(vs)
            if g is None:
                return None
            t_a, c_a, side_a, m1, c_t, side_t, t2p = g
            feats.append(("line", prev, t_a))
            feats.append(("arc", t_a, m1, c_a, TIP_RA, side_a, False))
            feats.append(("arc", m1, t2p, c_t, TIP_RT, side_t, False))
            prev = t2p
        elif i == 3:  # root cap edge over the pass-through dots
            g = cap_geom(vs)
            if g is None:
                return None
            t1, t2, c, r, u1, u2 = g
            side, _, _ = cap_sweep(t1, t2, c, u1, u2)
            feats.append(("line", prev, t1))
            feats.append(("arc", t1, t2, c, r, side, True))
            prev = t2
            i += 2
            continue
        elif i in PASS_THROUGH:
            i += 1
            continue
        else:
            fil = corner(vs, i)
            if fil is None:
                feats.append(("line", prev, vs[i]))
                prev = vs[i]
            else:
                a, b, c, r, s2 = fil
                feats.append(("line", prev, a))
                feats.append(("arc", a, b, c, r, s2, False))
                prev = b
        i += 1
    start = feats[0][2] if feats[0][0] == "line" else feats[0][1]
    feats.append(("line", prev, start))
    return [
        f
        for f in feats
        if f[0] == "arc"
        or (f[1] is not None and math.hypot(f[2][0] - f[1][0], f[2][1] - f[1][1]) > 1e-9)
    ]


def to_design_point(p):
    return (FRAME_X + p[0] * GRID_TO_DESIGN, FRAME_Y + p[1] * GRID_TO_DESIGN)


def path_cmds(feats, xf):
    """SVG path commands for one half ring (design coords, 2dp)."""

    def fmt(p):
        p = to_design_point(xf(p))
        return f"{p[0]:.2f} {p[1]:.2f}"

    cur = to_design_point(xf(feats[0][1]))
    out = [f"M {cur[0]:.2f} {cur[1]:.2f}"]

    def emit(txt, endp):
        nonlocal cur
        e = to_design_point(xf(endp))
        if abs(e[0] - cur[0]) <= 0.02 and abs(e[1] - cur[1]) <= 0.02:
            return  # degenerate command that would not move the pen
        out.append(txt)
        cur = e

    for f in feats:
        if f[0] == "line":
            emit(f"L {fmt(f[2])}", f[2])
            continue
        _, a, b, c, r, s2, is_cap = f
        rd = r * GRID_TO_DESIGN
        large = 0
        if is_cap:
            a1 = math.atan2(a[1] - c[1], a[0] - c[0])
            a2 = math.atan2(b[1] - c[1], b[0] - c[0])
            if s2 > 0:
                while a2 < a1:
                    a2 += 2 * math.pi
            else:
                while a2 > a1:
                    a2 -= 2 * math.pi
            large = 1 if math.degrees(abs(a2 - a1)) > 180.0 else 0
        if rd > 0.05:
            emit(f"A {rd:.2f} {rd:.2f} 0 {large} {1 if s2 > 0 else 0} {fmt(b)}", b)
        else:
            emit(f"L {fmt(b)}", b)
    out.append("Z")
    return out


def ring_bounds(feats):
    """Tight ink bounds of the FULL mark (half + mirror), grid coords."""
    pts = []
    for f in feats:
        if f[0] == "line":
            pts.extend(p for p in f[1:] if p is not None)
        else:
            _, a, b, c, r, s2, _ = f
            pts.extend([a, b])
            pts.extend(arc_points(a, b, c, r, s2))
    pts += [rot(p) for p in pts]
    return (
        min(p[0] for p in pts),
        min(p[1] for p in pts),
        max(p[0] for p in pts),
        max(p[1] for p in pts),
    )


def main():
    feats = half_features(resolved())
    if feats is None:
        raise SystemExit("infeasible knobs: no outline solves")
    d = " ".join(path_cmds(feats, lambda p: p) + path_cmds(feats, rot))

    # viewBox cropped tight to ink (whole design units), per BRAND.md
    x0, y0, x1, y1 = ring_bounds(feats)
    gx = to_design_point((x0, y0))
    gx1 = to_design_point((x1, y1))
    vx, vy = math.floor(gx[0]), math.floor(gx[1])
    vw, vh = math.ceil(gx1[0]) - vx, math.ceil(gx1[1]) - vy

    svg = (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vx} {vy} {vw} {vh}" '
        f'width="{vw}" height="{vh}">\n'
        "<g>\n"
        f'<path fill="{FILL}" fill-rule="nonzero" d="{d}"/>\n'
        "</g>\n"
        "</svg>\n"
    )
    Path(OUT).parent.mkdir(exist_ok=True)
    with open(OUT, "w") as f:
        f.write(svg)
    print(f"wrote {OUT} ({len(svg)} bytes, {d.count('M ')} subpaths, viewBox {vx} {vy} {vw} {vh})")


if __name__ == "__main__":
    main()
