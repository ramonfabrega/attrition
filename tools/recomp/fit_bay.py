#!/usr/bin/env python3
"""fit_bay.py — the `sim::launch::Bay` rows a `get_position.py … sweep` admits.

    python3 tools/recomp/fit_bay.py <sweep.txt> [<sweep.txt> …] [--check RIGHT,FWD]

A sweep (`get_position.py <exe> <packet> sweep <piece> <node> <anim> <time>`)
prints the original's truncated launch offset `(x, y, z)` at every whole
degree `0..=360`. `Bay::point` (`crates/sim/src/launch.rs`) turns an integer
`(right, fwd)` in thousandths by the whole degree `(deg + 180) % 360` with
`COS_DEG` and truncates each axis toward zero. This walks `right` around the
float estimate, intersects the `fwd` interval each of the 361 headings allows
on each axis, and prints the feasible lattice region: its size, its bounds,
and its centroid rounded — the row items 3590 and 3591 entered by hand with a
scratch fitter each (the third use graduated it here, item 3592). `dz` is the
sweep's `z`, printed when it is one number. `--check` says whether a given
`right,fwd` reproduces every heading. `COS_DEG` is read from `launch.rs`, so
the fit is the crate's own arithmetic, never a copy of it.
"""
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
LAUNCH = os.path.join(HERE, "..", "..", "crates", "sim", "src", "launch.rs")
DEN = 1000 << 30


def cos_table():
    src = open(LAUNCH).read()
    m = re.search(r"const COS_DEG: \[i64; 91\] = \[(.*?)\];", src, re.S)
    vals = [int(v.replace("_", "")) for v in re.findall(r"-?[\d_]+", m.group(1))]
    assert len(vals) == 91, len(vals)
    return vals


COS = cos_table()


def cos_sin(d):
    def c(d):
        d %= 360
        if d <= 90:
            return COS[d]
        if d <= 180:
            return -COS[180 - d]
        if d <= 270:
            return -COS[d - 180]
        return COS[360 - d]

    return c(d), c(d + 270)


def tdiv(n, d):
    """Rust's `/` on i64: truncation toward zero."""
    q = abs(n) // abs(d)
    return q if (n >= 0) == (d > 0) else -q


def window(x):
    """The numerators `N` with `tdiv(N, DEN) == x`."""
    if x > 0:
        return x * DEN, (x + 1) * DEN - 1
    if x < 0:
        return (x - 1) * DEN + 1, x * DEN
    return -(DEN - 1), DEN - 1


def ceil_div(a, b):
    return -((-a) // b)


def f_range(lo, hi, k):
    """The integer `f` with `lo <= f * k <= hi`, as `(flo, fhi)` or None for all."""
    if k == 0:
        return None if lo <= 0 <= hi else (1, 0)
    if k > 0:
        return ceil_div(lo, k), hi // k
    return ceil_div(hi, k), lo // k


def point(r, f, deg):
    c, s = cos_sin((deg + 180) % 360)
    return tdiv(r * c + f * s, DEN), tdiv(r * s - f * c, DEN)


def read(path):
    rows = []
    for line in open(path):
        p = line.split()
        if len(p) >= 8 and p[0].isdigit():
            rows.append((int(p[0]), int(p[2]), int(p[3]), int(p[4]), float(p[5]), float(p[6])))
    return rows


def fit(rows, span=4000):
    d0 = next(r for r in rows if r[0] == 0)
    # At sweep degree 0 the turn is 180: x = -right, y = fwd (thousandths).
    r0, f0 = round(-1000 * d0[4]), round(1000 * d0[5])
    region = []
    for r in range(r0 - span, r0 + span + 1):
        flo, fhi = f0 - 1_000_000, f0 + 1_000_000
        for deg, x, y, _z, _fx, _fy in rows:
            c, s = cos_sin((deg + 180) % 360)
            lo, hi = window(x)
            g = f_range(lo - r * c, hi - r * c, s)
            if g is not None:
                flo, fhi = max(flo, g[0]), min(fhi, g[1])
            lo, hi = window(y)
            g = f_range(lo - r * s, hi - r * s, -c)
            if g is not None:
                flo, fhi = max(flo, g[0]), min(fhi, g[1])
            if flo > fhi:
                break
        else:
            region.extend((r, f) for f in range(flo, fhi + 1))
    return (r0, f0), region


def main(argv):
    check = None
    if "--check" in argv:
        i = argv.index("--check")
        check = tuple(int(v) for v in argv[i + 1].split(","))
        argv = argv[:i] + argv[i + 2:]
    for path in argv:
        rows = read(path)
        zs = sorted({z for _, _, _, z, _, _ in rows})
        est, region = fit(rows)
        print(f"{os.path.basename(path)}: {len(rows)} headings, z {zs}, estimate {est}")
        if not region:
            print("  no feasible (right, fwd)")
        else:
            rs = [r for r, _ in region]
            fs = [f for _, f in region]
            cr, cf = round(sum(rs) / len(rs)), round(sum(fs) / len(fs))
            inside = (cr, cf) in set(region)
            print(f"  region {len(region)} points, right {min(rs)}..{max(rs)}, fwd {min(fs)}..{max(fs)}")
            print(f"  centroid right {cr} fwd {cf} ({'inside' if inside else 'OUTSIDE'} the region)")
        if check:
            bad = [(d, x, y, point(*check, d)) for d, x, y, *_ in rows if point(*check, d) != (x, y)]
            print(f"  check {check}: {len(rows) - len(bad)} of {len(rows)} headings" + (f", first miss {bad[0]}" if bad else ""))


if __name__ == "__main__":
    main(sys.argv[1:])
