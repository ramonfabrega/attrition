#!/usr/bin/env python3
"""One line per object in a single frame — the "what is on the map" view.

objs.py [frame.txt]

Input is one frame, as written by lastframe.py. Prints kind, object number,
owner, tile, hit points, building type and flags. Object numbers are **per
player**: units count from 0 and buildings from 2000, so a line is identified
by (kind, who, o) and not by o alone. `guy` is the unit's TypeIndex, and is
only filled in when gamelog.ini has GUYS at 1 or more under the phase.
"""
import os
import sys

DEFAULT = os.path.join(os.environ.get("TMPDIR", "/tmp"), "ron-last.txt")
RECORDS = ("UNITDATA", "BUILDDATA", "ANIMALDATA", "CITYDATA", "LEADERDATA")

path = sys.argv[1] if len(sys.argv) > 1 else DEFAULT
cur = None
recs = []
for line in open(path):
    s = line.strip()
    if s.startswith("BEGIN "):
        name = s[6:]
        if name in RECORDS:
            cur = {"kind": name}
            recs.append(cur)
        if name == "GUY" and cur is not None:
            cur.setdefault("guys", []).append({})
        continue
    if cur is None:
        continue
    parts = s.split(" ")
    if len(parts) >= 2:
        k, v = " ".join(parts[:-1]), parts[-1]
        if "guys" in cur and k == "type" and cur["guys"] and "type" not in cur["guys"][-1]:
            cur["guys"][-1]["type"] = v
        cur.setdefault(k, v)

for r in recs:
    if "x_internal" not in r:
        continue
    tx = int(r["x_internal"]) / 192.0
    ty = int(r.get("y_internal", 0)) / 192.0
    guy = ",".join(sorted({g.get("type", "?") for g in r.get("guys", [])})) or "-"
    print("%-10s o=%5s who=%s tile=(%6.1f,%6.1f) hits=%6s otype=%4s flags=%s guy=%s" % (
        r["kind"], r.get("o"), r.get("who"), tx, ty, r.get("myhits"),
        r.get("orig_type", "-"), r.get("flags"), guy))
