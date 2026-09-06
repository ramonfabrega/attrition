#!/usr/bin/env python3
"""Every object that appears or disappears across a dumped frame window.

births.py FILE [LO] [HI]

The fourth capture in a row has asked the same question of an archive — "on
which frame does the object census move, and what was born" — so the probe
graduates (`CLAUDE.md`, Conventions). run76 asked it of Great Lakes 6640,
run77 and run78 of two East Indies windows, run79 of Great Lakes 6910.
Checked against the two whose answers are on record: run76's window gives
its one unit at `FRAME 6763` and nothing else, and run78's gives `1/60`,
uid 91, `guy 353`, 109 hits at `(38040, 42408)` on `FRAME 15783` — the
comparison `docs/ORACLE.md` made by hand — plus a death at 15800 that the
hand pass missed. `guys=3` on that line is the fact that cost run78 a
capture to learn.

An object is keyed `(kind, who, o, uid)`, not `o` alone: `o` is per player
and per kind (units from 0, buildings from 2000) and a freed slot is reused,
so a birth into a dead unit's `o` would otherwise read as no change at all.
That is the comparison run78 made by hand.

Streaming, two frames of state at a time — an archive is hundreds of MB and
the memory ceiling applies to the capture lane's own tools too.

**Pass the capture's own window.** Outside it the dump reverts to the run's
`[End Frame]` detail, where `uid` and the `GUY` records are gone, and every
object on the map then reads as gained — which is what run76's archive does
at 6871, one frame past its window's close.

Prints, per frame that moved: the per-kind counts, then one line per object
gained (`+`) or lost (`-`) with the fields a birth is read from — position in
internal units and in tiles, `uid`, the guy types, `group`, `angle`, and for
a building its `orig_type`.
"""
import sys

RECORDS = ("UNITDATA", "BUILDDATA", "ANIMALDATA", "CITYDATA", "LEADERDATA")

if len(sys.argv) < 2:
    sys.exit(__doc__)
path = sys.argv[1]
lo = int(sys.argv[2]) if len(sys.argv) > 2 else None
hi = int(sys.argv[3]) if len(sys.argv) > 3 else None


def fmt(key, r):
    kind, who, o, uid = key
    x, y = int(r.get("x_internal", 0)), int(r.get("y_internal", 0))
    guy = ",".join(sorted({g.get("type", "?") for g in r.get("guys", [])})) or "-"
    return ("%-10s %s/%-5s uid=%-6s (%6d,%6d) tile=(%6.1f,%6.1f) hits=%-6s "
            "guys=%-2d guy=%-8s group=%-5s angle=%-12s otype=%-5s flags=%s" % (
                kind, who, o, uid, x, y, x / 192.0, y / 192.0,
                r.get("myhits", "-"), len(r.get("guys", [])), guy,
                r.get("group", "-"), r.get("angle", "-"),
                r.get("orig_type", "-"), r.get("flags", "-")))


def census(recs):
    out = {}
    for r in recs:
        if "x_internal" not in r or "o" not in r:
            continue
        out[(r["kind"], r.get("who"), r["o"], r.get("uid", "-"))] = r
    return out


def report(frame, prev, cur):
    gained = [k for k in cur if k not in prev]
    lost = [k for k in prev if k not in cur]
    if not gained and not lost:
        return
    counts = {}
    for k in cur:
        counts[k[0]] = counts.get(k[0], 0) + 1
    before = {}
    for k in prev:
        before[k[0]] = before.get(k[0], 0) + 1
    moved = ", ".join("%s %s->%s" % (n, before.get(n, 0), counts.get(n, 0))
                      for n in sorted(set(counts) | set(before))
                      if before.get(n, 0) != counts.get(n, 0))
    print("FRAME %s  %s" % (frame, moved or "slot reuse, no count change"))
    for k in sorted(gained):
        print("  + " + fmt(k, cur[k]))
    for k in sorted(lost):
        print("  - " + fmt(k, prev[k]))
    sys.stdout.flush()


frame = None
cur = None
recs = []
prev = {}
nframes = 0
with open(path, errors="replace") as fh:
    for line in fh:
        s = line.strip()
        if s.startswith("BEGIN FRAME "):
            n = int(s.split()[2])
            if frame is not None and (lo is None or lo <= frame) and (hi is None or frame < hi):
                c = census(recs)
                if prev:
                    report(frame, prev, c)
                prev = c
                nframes += 1
            frame, cur, recs = n, None, []
            continue
        if frame is None:
            continue
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

if frame is not None and (lo is None or lo <= frame) and (hi is None or frame < hi):
    c = census(recs)
    if prev:
        report(frame, prev, c)
    nframes += 1
print("compared %d frames" % nframes)
