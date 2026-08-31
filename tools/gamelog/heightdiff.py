#!/usr/bin/env python3
"""heightdiff.py DUMP FRAME_A FRAME_B [TILE ...] — which corners the terraform moved.

    heightdiff.py gamelog-run43-roadterraform.txt 100 106 6,171 33,161

`docs/ROADS.md` §7.1 reads the road search's grid as the **pre**-terraform
one, and says `terraform_for_building@00875210` runs after `place_roads`,
moving "the 128 corners ... the two footprints' boxes and nothing else".
That claim was established across **two different games** — run13's `FRAME
100`, in which nothing is ever placed, against run32's 104 — so it carried
an assumption it could not check: that the two games' terrain agrees up to
the placement. A window that holds both frames of *one* game turns the
comparison into a difference (queue item 57).

The heights are `terrain->master_land_heights`, `(4·xs + 1) × (4·ys + 1)`
corners row-major in millionths, printed by `GameLog::dump_all@0092f2d0` as
a bare `SimpleArray<float>` with no block of its own — so they land on the
preceding `UnbuiltForts` block, whose own `length` is 0 and whose second
`length` is the array's (`crates/rondata/src/gamelog.rs`, `heights_in`).
Only a `DUMP_ALL` frame carries them.

A block `FRAME n` is the end of sim-frame `n − 1`, so `FRAME 100` is before
the cheats at the top of sim-frame 100 and `FRAME 106` is after the
Smelter's replan on 104 and the Granary's on 105.

Given tiles, it reports whether every moved corner lies in one of their
boxes — which is the half of §7.1's claim that a single frame pair can
falsify.
"""
import os
import sys

ARCHIVE = os.environ.get(
    "RON_GAMELOG_DIR",
    os.path.expanduser(
        "~/Library/Application Support/CrossOver/Bottles/ron/drive_c/users"
        "/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"))


def micro(v):
    """The log prints a float; keep it in millionths so nothing is lost."""
    v = v.strip()
    try:
        return int(round(float(v) * 1_000_000))
    except ValueError:
        return None


def heights_of(path, want):
    """{frame: [corner heights]} for the frames in `want`."""
    out = {}
    frame = None
    block = None
    pending = None      # values being collected
    need = 0
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            s = line.strip()
            if s.startswith("BEGIN FRAME "):
                try:
                    frame = int(s.split()[2])
                except (IndexError, ValueError):
                    frame = None
                block, pending, need = None, None, 0
                continue
            if frame not in want:
                continue
            if s.startswith("BEGIN "):
                # The heights are printed with no block of their own and land
                # on `UnbuiltForts`; any other block's big array is not them.
                block = s[6:].strip()
                if pending is not None and len(pending) < need:
                    pending, need = None, 0
                continue
            if block != "UnbuiltForts":
                continue
            if s.startswith("length "):
                try:
                    n = int(s.split()[1])
                except (IndexError, ValueError):
                    continue
                # The forts list itself is `length 0`; the heights are the
                # one over a thousand.
                if n > 1000 and frame not in out:
                    pending, need = [], n
                continue
            if pending is not None and s.startswith("list[scan] "):
                v = micro(s.split(None, 1)[1])
                if v is not None:
                    pending.append(v)
                    if len(pending) == need:
                        out[frame] = pending
                        pending, need = None, 0
    return out


def main():
    path = sys.argv[1]
    if not os.path.isabs(path):
        path = os.path.join(ARCHIVE, path)
    a, b = int(sys.argv[2]), int(sys.argv[3])
    tiles = []
    for t in sys.argv[4:]:
        x, y = t.split(",")
        tiles.append((int(x), int(y)))

    h = heights_of(path, {a, b})
    for n in (a, b):
        if n not in h:
            sys.exit("FRAME %d carries no master_land_heights — is it inside "
                     "the DUMP_ALL window?" % n)
    ha, hb = h[a], h[b]
    print("FRAME %d: %d corners; FRAME %d: %d corners" % (a, len(ha), b, len(hb)))
    if len(ha) != len(hb):
        sys.exit("the two frames disagree on the grid size")

    side = int(round(len(ha) ** 0.5))
    if side * side != len(ha):
        print("note: %d corners is not square; row/col mapping skipped" % len(ha))
        side = None

    moved = [i for i, (x, y) in enumerate(zip(ha, hb)) if x != y]
    print("corners moved: %d" % len(moved))
    if not moved:
        return 0

    if side is None:
        return 0

    # The corner-to-tile scale is **measured, not assumed**: this grid is
    # 241 a side where the lobby's map is 180 tiles, which is not the 4:1 the
    # field's name suggests, so the tool reports where the moved corners
    # actually are and lets the clusters say what the mapping is.
    coords = [(i % side, i // side) for i in moved]
    print("grid %d x %d corners" % (side, side))
    print("corner column range %d..%d, row range %d..%d"
          % (min(c for c, _ in coords), max(c for c, _ in coords),
             min(r for _, r in coords), max(r for _, r in coords)))
    print("i.e. tiles x %d..%d, y %d..%d"
          % (min(c for c, _ in coords) // 4, max(c for c, _ in coords) // 4,
             min(r for _, r in coords) // 4, max(r for _, r in coords) // 4))

    # Cluster the moved corners by proximity, so "the two footprints' boxes
    # and nothing else" is a count of clusters and their extents rather than
    # a test against a scale this tool has not earned.
    clusters = []
    for c, r in sorted(coords):
        for cl in clusters:
            if any(abs(c - cc) <= 6 and abs(r - rr) <= 6 for cc, rr in cl):
                cl.append((c, r))
                break
        else:
            clusters.append([(c, r)])
    print("\nclusters of moved corners: %d" % len(clusters))
    for cl in clusters:
        cs = [c for c, _ in cl]
        rs = [r for _, r in cl]
        print("  %4d corners, columns %d..%d, rows %d..%d, centre (%.1f, %.1f)"
              % (len(cl), min(cs), max(cs), min(rs), max(rs),
                 sum(cs) / len(cs), sum(rs) / len(rs)))
    if tiles:
        print("\nthe stanza placed at tiles: %s"
              % ", ".join("(%d, %d)" % t for t in tiles))
        for cl in clusters:
            cs = [c for c, _ in cl]
            rs = [r for _, r in cl]
            cc, rr = sum(cs) / len(cs), sum(rs) / len(rs)
            for tx, ty in tiles:
                if tx:
                    print("  cluster centre (%.1f, %.1f) / tile (%d, %d)"
                          " = %.3f, %.3f corners a tile"
                          % (cc, rr, tx, ty, cc / tx, rr / ty if ty else 0))
    return 0


if __name__ == "__main__":
    sys.exit(main())
