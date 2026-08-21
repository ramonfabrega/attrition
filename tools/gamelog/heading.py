#!/usr/bin/env python3
"""Is UnitData.angle the direction the unit is travelling?

heading.py frames.txt who o

Compares the logged `angle` against find_angle of the per-frame position
delta. Binary angles: full circle 2**32, north 0, east 0x40000000, y southward.
"""
import math
import sys

FULL = 1 << 32
path, who, o = sys.argv[1], sys.argv[2], sys.argv[3]


def deg(a):
    a &= FULL - 1
    a = a - FULL if a >= FULL // 2 else a
    return a * 360.0 / FULL


cur = None
frame = None
rows = []
for line in open(path):
    s = line.strip()
    if s.startswith("BEGIN "):
        name = s[6:]
        if name.startswith("FRAME"):
            frame = int(name.split()[1])
        elif name == "UNITDATA":
            cur = {}
        continue
    if cur is None:
        continue
    parts = s.split(" ")
    if len(parts) >= 2:
        cur.setdefault(" ".join(parts[:-1]), parts[-1])
    if s.startswith("good_obj "):
        if cur.get("who") == who and cur.get("o") == o:
            rows.append((frame, int(cur["x_internal"]), int(cur["y_internal"]),
                         int(cur["angle"]), int(cur.get("myspeed", 0))))
        cur = None

print("%-8s %-8s %-9s %-9s %s" % ("frame", "step", "angle", "heading", "angle-heading"))
n = 0
for (f0, x0, y0, a0, _), (f1, x1, y1, a1, _) in zip(rows, rows[1:]):
    dx, dy = x1 - x0, y1 - y0
    if dx == 0 and dy == 0:
        continue
    head = int(round(math.atan2(dx, -dy) / (2 * math.pi) * FULL))
    print("%-8d %-8.1f %9.2f %9.2f %9.2f" % (
        f1, math.hypot(dx, dy), deg(a1), deg(head), deg(a1 - head)))
    n += 1
    if n >= 25:
        break
