#!/usr/bin/env python3
"""Screen point for a tile, under the isometric projection.

aim.py A B C D tx,ty [tx,ty ...]        -- screen point per tile
aim.py A B C D --from fx,fy tx,ty ...   -- and the vector_dist from a tile

The projection is exactly linear in u = wx - wy and v = wx + wy:

    screen_x = A*u + C        screen_y = B*v + D

A and B depend on the zoom only, C and D on the camera. Derive A and B once
from two right-click anchors (`cheat select <o>` then a right-click on open
ground; the unit's logged orders_x/y is the world point under the cursor).
After a camera move, keep A and B and re-fit C and D from a single
`cheat add NEW tower` probe, which reads the cursor's tile back exactly.
See docs/ORACLE.md, "A scripted placement test".

Internal units are 192 to the tile; a tile's centre is tile*192 + 96.
"""
import sys

A, B, C, D = (float(v) for v in sys.argv[1:5])
rest = sys.argv[5:]

origin = None
if rest and rest[0] == "--from":
    origin = tuple(int(v) for v in rest[1].split(","))
    rest = rest[2:]


def screen(tx, ty):
    x, y = tx * 192 + 96, ty * 192 + 96
    return A * (x - y) + C, B * (x + y) + D


def vector_dist(dx, dy):
    dx, dy = abs(dx), abs(dy)
    hi, lo = max(dx, dy), min(dx, dy)
    return hi + (lo * lo) // (2 * hi) if hi else 0


for arg in rest:
    tx, ty = (int(v) for v in arg.split(","))
    sx, sy = screen(tx, ty)
    d = ""
    if origin:
        d = "  dist=%3d" % vector_dist(tx - origin[0], ty - origin[1])
    print("tile(%3d,%3d) screen=(%6.0f,%6.0f)%s" % (tx, ty, sx, sy, d))
