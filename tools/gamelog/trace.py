#!/usr/bin/env python3
"""Raw per-frame trace of one unit: damage, frac, hits, angle, damage_* fields."""
import sys

path, who, o = sys.argv[1], sys.argv[2], sys.argv[3]
FULL = 1 << 32


def deg(a):
    a &= FULL - 1
    a = a - FULL if a >= FULL // 2 else a
    return a * 360.0 / FULL


cur = None
frame = None
last = None
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
            row = (cur.get("damage"), cur.get("damage_frac"), cur.get("myhits"),
                   cur.get("damage_o"), cur.get("damage_who"), cur.get("damage_frame"),
                   cur.get("angle"), cur.get("recharging"))
            if row[:6] != (last[:6] if last else None):
                print("f%-7d dmg=%-4s frac=%-3s hits=%-4s from=%s/%-4s dframe=%-7s ang=%7.1f rech=%s" % (
                    frame, row[0], row[1], row[2], row[4], row[3], row[5],
                    deg(int(row[6])), row[7]))
                last = row
        cur = None
