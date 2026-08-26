#!/usr/bin/env python3
"""The live GROUPDATA records of one frame — the group pool, folded.

groups.py [frame.txt]

Input is one frame, as written by lastframe.py. Prints one line per group
with `num > 0`, then its member table: the object, its slot offset `off`,
the rotated `curr`, and the packed facing byte `angles`. Empty pool slots
(512 of them, plus the `HOTKEYGROUPS` block's nested copies) are dropped.

The pool is a per-frame record without `DUMP_ALL`: `GameLog::full_dump`
gates `dump_groups` on `details[mode][0x12]`, so `[End Frame] GROUPS=1`
in gamelog.ini is enough (docs/ORACLE.md, "The group pool is a cheap
per-frame record"). `docs/GROUPS.md` §1 names every scalar.
"""
import os
import re
import sys

DEFAULT = os.path.join(os.environ.get("TMPDIR", "/tmp"), "ron-last.txt")
SCALARS = ("id", "who", "num", "army", "ox", "oy", "o_dist", "o_angle",
           "buildings", "disband", "order_num", "priority", "stamp", "role",
           "form", "think_frame", "facing", "new_speed", "speed", "form_num")
ARRAYS = ("list", "off_x", "off_y", "curr_x", "curr_y", "angles")

path = sys.argv[1] if len(sys.argv) > 1 else DEFAULT
groups = []
cur = None
frame = ""
for line in open(path, errors="replace"):
    s = line.strip()
    if s.startswith("BEGIN FRAME"):
        frame = s
        continue
    if s.startswith("BEGIN GROUPDATA"):
        cur = {"arr": {}}
        groups.append(cur)
        continue
    if cur is None:
        continue
    if s.startswith("BEGIN "):
        continue
    m = re.match(r"^(\w+) (-?\d+)$", s)
    if m and m.group(1) in SCALARS:
        cur.setdefault(m.group(1), int(m.group(2)))
        continue
    # `Array<T>::log_data` writes one `name[scan] v` line per element, not one
    # line with the whole row, so the values accumulate.
    m = re.match(r"^(\w+)\[scan\] (-?\d+)$", s)
    if m and m.group(1) in ARRAYS:
        cur["arr"].setdefault(m.group(1), []).append(int(m.group(2)))

print(frame, "-- %d records, %d live" % (len(groups), sum(1 for g in groups if g.get("num", 0) > 0)))
for g in groups:
    if g.get("num", 0) <= 0:
        continue
    print("\ngroup id %s who %s num %s army %s form %s form_num %s facing %s "
          "role %s order_num %s speed %s o (%s,%s) o_angle %s o_dist %s"
          % tuple(g.get(k) for k in ("id", "who", "num", "army", "form", "form_num",
                                     "facing", "role", "order_num", "speed",
                                     "ox", "oy", "o_angle", "o_dist")))
    a = g["arr"]
    n = max((len(v) for v in a.values()), default=0)
    print("   i  %6s %8s %8s %8s %8s %7s" % ("o", "off_x", "off_y", "curr_x", "curr_y", "angles"))
    for i in range(n):
        def col(k):
            v = a.get(k, [])
            return v[i] if i < len(v) else ""
        print("  %2d  %6s %8s %8s %8s %8s %7s"
              % (i, col("list"), col("off_x"), col("off_y"),
                 col("curr_x"), col("curr_y"), col("angles")))
