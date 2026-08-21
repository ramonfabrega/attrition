#!/usr/bin/env python3
"""One object's fields out of a single frame.

one.py KIND WHO O [field ...] [--file frame.txt]

KIND is the record's BEGIN name (UNITDATA, BUILDDATA, CITYDATA, LEADERDATA).
With no fields the whole record is printed. Exits non-zero when the object is
absent, which is itself the answer to "was the site disbanded?".
"""
import os
import sys

DEFAULT = os.path.join(os.environ.get("TMPDIR", "/tmp"), "ron-last.txt")
RECORDS = ("UNITDATA", "BUILDDATA", "ANIMALDATA", "CITYDATA", "LEADERDATA")

argv = sys.argv[1:]
path = DEFAULT
if "--file" in argv:
    i = argv.index("--file")
    path = argv[i + 1]
    argv = argv[:i] + argv[i + 2:]
kind, who, o = argv[0], argv[1], argv[2]
want = argv[3:]

cur = None
recs = []
for line in open(path):
    s = line.strip()
    if s.startswith("BEGIN "):
        if s[6:] in RECORDS:
            cur = {"kind": s[6:]}
            recs.append(cur)
        continue
    if cur is None:
        continue
    parts = s.split(" ")
    if len(parts) >= 2:
        cur.setdefault(" ".join(parts[:-1]), parts[-1])

hit = [r for r in recs if r["kind"] == kind and r.get("who") == who and r.get("o") == o]
if not hit:
    print("no record")
    sys.exit(1)
r = hit[-1]
if want:
    print(" ".join("%s=%s" % (k, r.get(k)) for k in want))
else:
    for k, v in r.items():
        print(k, v)
