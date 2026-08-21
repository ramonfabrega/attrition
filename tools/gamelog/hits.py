#!/usr/bin/env python3
"""Every damage increment in a window of frames, with the flank geometry.

hits.py frames.txt [who]

For each frame in which a unit's (damage*16 + damage_frac) rises, print the
increment in sixteenths, the victim's angle at that frame, the attacker named
by damage_o/damage_who, and

    d = victim.angle - find_angle(victim - attacker)

normalised to a signed binary angle and shown in degrees. Angles are binary:
the full circle is 2**32, north is 0, east is 0x40000000, and y increases
southward (docs/MOVEMENT.md).
"""
import math
import sys

RECORDS = ("UNITDATA",)
FULL = 1 << 32


def frames(path):
    cur = None
    frame = None
    recs = []
    for line in open(path):
        s = line.strip()
        if s.startswith("BEGIN "):
            name = s[6:]
            if name.startswith("FRAME"):
                if frame is not None:
                    yield frame, recs
                frame = int(name.split()[1])
                recs = []
                cur = None
            elif name in RECORDS:
                cur = {}
                recs.append(cur)
            continue
        if cur is None:
            continue
        parts = s.split(" ")
        if len(parts) >= 2:
            cur.setdefault(" ".join(parts[:-1]), parts[-1])
    if frame is not None:
        yield frame, recs


def signed(a):
    a &= FULL - 1
    return a - FULL if a >= FULL // 2 else a


def deg(a):
    return signed(a) * 360.0 / FULL


def find_angle(dx, dy):
    # north is 0, east is +90 degrees, y increases southward
    return int(round(math.atan2(dx, -dy) / (2 * math.pi) * FULL))


path = sys.argv[1]
prev = {}
prevpos = {}
for fr, recs in frames(path):
    pos = {}
    for r in recs:
        if "o" not in r or "who" not in r:
            continue
        key = (r["who"], r["o"])
        pos[key] = r
    for key, r in pos.items():
        tot = int(r.get("damage", 0)) * 16 + int(r.get("damage_frac", 0))
        was = prev.get(key)
        prev[key] = tot
        if was is None or tot <= was:
            continue
        inc = tot - was
        akey = (r.get("damage_who"), r.get("damage_o"))
        a = prevpos.get(akey) or pos.get(akey)
        line = "f%-7d victim %s/%-3s +%-4d sixteenths  angle %7.1f" % (
            fr, r["who"], r["o"], inc, deg(int(r["angle"])))
        if a and a.get("x_internal"):
            dx = int(r["x_internal"]) - int(a["x_internal"])
            dy = int(r["y_internal"]) - int(a["y_internal"])
            bearing = find_angle(dx, dy)
            d = deg(int(r["angle"]) - bearing)
            line += "  attacker %s/%-3s bearing %7.1f  d %7.1f" % (
                akey[0], akey[1], deg(bearing), d)
        else:
            line += "  attacker %s/%s (not in frame)" % akey
        print(line)
    prevpos = pos
