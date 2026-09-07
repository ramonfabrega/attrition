#!/usr/bin/env python3
"""The danger map out of a dumped frame block — `docs/DANGER.md`'s own oracle.

    danger.py map   FILE FRAME [--differ | --same]
    danger.py units FILE FRAME [--types FILE] [--dump N] [--least N]

`WorldData::log_data@006b6080` prints `danger[who][scan]` — eight rows of
`reg_size` ints — inside every `BEGIN WORLD`, and a `DUMP_ALL` frame block
carries **six** copies: three per `FULL DUMP` (byte-identical, checked on
run29, run65 and run72), and one `FULL DUMP` apiece from `GameLog::begin_frame`
and `GameLog::end_frame`. So one block holds the map **before** and **after**
everything sim-frame N did, which is why a rebuild frame is worth capturing:

    Game::do_frame@00591ef0 — begin_frame(N) ... GameDaemon::process_all
    (calc_danger when N % 200 == 0) ... Objects::process_all ... frame += 1 ...
    end_frame(N)

`calc_danger` runs **before** any object moves, so dump 1's objects are exactly
what it read and dump 2's map is exactly what it wrote.

`map` prints one line per `FULL DUMP`; `--differ` exits non-zero unless the two
maps differ (something the rebuild saw had changed) and `--same` is its
inverse. `units` lists the units whose type carries the military bit
`role & 0x10000` (`UnitTypeData+0x2c8`), the half-cell each indexes and what
every leader's row holds there; `--least N` exits non-zero below N of them.
`--types` names an archive whose dump has the `UNITTYPE` records — a cheap
window has none and must borrow one.

The half-cell is `danger[who][reg_xs * div3(y >> 9) + div3(x >> 9)]` with
`reg_xs = xs * 4 >> 3`, one entry per two cells each way, so 30 x 30 on a
60 x 60 map. `docs/DANGER.md` sections 1 and 2.
"""
import hashlib
import os
import sys

ARCHIVE = os.environ.get(
    "RON_GAMELOG_DIR",
    os.path.expanduser(
        "~/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"))

RECORDS = ("UNITDATA", "BUILDDATA", "ANIMALDATA", "CITYDATA", "LEADERDATA")


def resolve(name):
    return name if os.path.exists(name) else os.path.join(ARCHIVE, name)


def indent(line):
    return len(line) - len(line.lstrip(" "))


def block(path, frame):
    """The `BEGIN FRAME frame` block's lines, **indentation intact**.

    The dump nests by indentation and a record ends at the next `BEGIN` no
    deeper than its own (`samegame.py`'s rule). Stripping first — which is what
    `frame.py` writes — loses that, and then the last `UNITDATA` of a dump
    collects the `BEGIN GUY`s of all 674 `GROUPDATA` records after it. The
    frame line itself carries a leading space and the file is CRLF, so the
    match is on the stripped text while the line is kept whole.
    """
    want, out, on = "BEGIN FRAME %d" % frame, [], False
    with open(path, errors="replace") as f:
        for line in f:
            line = line.rstrip("\r\n")
            s = line.strip()
            if s.startswith("BEGIN FRAME"):
                if on:
                    return out
                on = s == want
            if on:
                out.append(line)
    if not out:
        sys.exit("no BEGIN FRAME %d in %s" % (frame, path))
    return out


def maps(lines):
    """([danger row per FULL DUMP], xs, ys, [(lo, hi) line span per dump])."""
    out, spans, xs, ys = [], [], None, None
    cur, world, wd = None, None, -1
    for i, line in enumerate(lines):
        s = line.strip()
        if s.startswith("BEGIN ") or s.startswith("END "):
            name = s[6:] if s.startswith("BEGIN ") else None
            if world is not None and (name is None or indent(line) <= wd):
                world = None
            if s == "BEGIN FULL DUMP":
                out.append([])
                if spans:
                    spans[-1][1] = i
                spans.append([i, len(lines)])
                cur = out[-1]
            elif name == "WORLD" and cur is not None:
                world, wd = [], indent(line)
                cur.append(world)
            continue
        p = s.split()
        if len(p) != 2 or world is None:
            continue
        k, v = p
        if k == "danger[who][scan]":
            world.append(int(v))
        elif k == "xs":
            xs = int(v)
        elif k == "ys":
            ys = int(v)
    if not out:                          # a cheap block: one implicit dump
        out, spans = [[]], [[0, len(lines)]]
    return out, xs, ys, spans


def records(lines):
    """Top-level object records of a run of dump lines, closed by indentation."""
    out, rec, rd = [], None, -1
    for line in lines:
        s = line.strip()
        if s.startswith("BEGIN "):
            d, name = indent(line), s[6:]
            # A `GUY` deeper than the open record is that unit's; the dump
            # also writes a **top-level guy pool** at the same depth as the
            # records, and one of those closes the record like any other BEGIN.
            if rec is not None and d <= rd:
                rec = None
            if name in RECORDS:
                rec, rd = {"kind": name}, d
                out.append(rec)
            elif name == "GUY" and rec is not None:
                rec.setdefault("guys", []).append({})
            continue
        p = s.split()
        if len(p) == 2 and rec is not None:
            k, v = p
            if "guys" in rec and k == "type" and rec["guys"] and "type" not in rec["guys"][-1]:
                rec["guys"][-1]["type"] = v
            rec.setdefault(k, v)
    return out


def military_types(path):
    """Types whose `role` has the military bit, from the first `UNITTYPE` table."""
    mil, cur, seen = set(), None, False
    with open(path, errors="replace") as f:
        for line in f:
            s = line.strip()
            if s.startswith("BEGIN "):
                name = s[6:]
                if name == "UNITTYPE":
                    cur, seen = {}, True
                elif name in ("BUILDTYPE", "GOODTYPE", "TECHTYPE", "SPELLTYPE"):
                    cur = None
                elif seen and name in RECORDS:
                    break                # the type table is over; stop reading
                continue
            p = s.split()
            if len(p) == 2 and cur is not None and p[0] in ("type", "role") and p[0] not in cur:
                try:
                    cur[p[0]] = int(p[1])
                except ValueError:
                    continue
                if "type" in cur and "role" in cur:
                    if cur["role"] & 0x10000:
                        mil.add(cur["type"])
                    cur = None
    if not mil:
        sys.exit("no UNITTYPE records in %s — pass --types with a DUMP_ALL archive" % path)
    return mil


def do_map(frame, worlds, argv):
    if not worlds:
        print("frame %d: no WORLD block — nothing to compare" % frame)
        return 1
    for i, dump in enumerate(worlds):
        copies = sorted({hashlib.sha1(str(w).encode()).hexdigest()[:10] for w in dump})
        w = dump[0]
        print("frame %d dump%d: %d WORLD copies %s  n=%d nonzero=%d [%d, %d]"
              % (frame, i + 1, len(dump), copies, len(w),
                 sum(1 for v in w if v), min(w), max(w)))
    if "--differ" not in argv and "--same" not in argv:
        return 0
    if len(worlds) < 2:
        print("only %d dump carries a map — cannot compare" % len(worlds))
        return 1
    a, b = worlds[0][0], worlds[1][0]
    n = sum(1 for x, y in zip(a, b) if x != y)
    print("dump1 vs dump2: %s"
          % ("IDENTICAL" if n == 0 else "DIFFER in %d of %d" % (n, len(a))))
    return 0 if ((n > 0) == ("--differ" in argv)) else 1


def do_units(path, frame, lines, worlds, xs, ys, spans, argv):
    tsrc = resolve(argv[argv.index("--types") + 1]) if "--types" in argv else None
    mil = military_types(tsrc or path)
    which = int(argv[argv.index("--dump") + 1]) if "--dump" in argv else 1
    lo, hi = spans[which - 1]
    reg_xs = (xs * 4) >> 3 if xs else 0
    reg_size = reg_xs * (((ys * 4) >> 3) if ys else 0)
    found = 0
    for r in records(lines[lo:hi]):
        if r["kind"] != "UNITDATA" or "x_internal" not in r:
            continue
        types = {int(g["type"]) for g in r.get("guys", []) if "type" in g}
        if not types & mil:
            continue
        found += 1
        x, y = int(r["x_internal"]), int(r["y_internal"])
        hx, hy = (x >> 9) // 3, (y >> 9) // 3
        where = ""
        if worlds and reg_size:
            idx = reg_xs * hy + hx
            for i, dump in enumerate(worlds):
                row = [dump[0][w * reg_size + idx] for w in range(8)]
                where += "  dump%d[%d]=%s" % (i + 1, idx, row[:2])
        print("who=%s o=%s guy=%s pos=(%d,%d) half=(%d,%d)%s flags=%s"
              % (r.get("who"), r.get("o"), sorted(types), x, y, hx, hy, where, r.get("flags")))
    print("frame %d dump%d: %d unit(s) with role & 0x10000 on the map"
          % (frame, which, found))
    need = int(argv[argv.index("--least") + 1]) if "--least" in argv else 0
    return 0 if found >= need else 1


def main():
    if len(sys.argv) < 4:
        sys.exit(__doc__)
    mode, path, frame = sys.argv[1], resolve(sys.argv[2]), int(sys.argv[3])
    argv = sys.argv[4:]
    lines = block(path, frame)
    dumps, xs, ys, spans = maps(lines)
    worlds = [d for d in dumps if d and d[0]]
    if mode == "map":
        return do_map(frame, worlds, argv)
    if mode == "units":
        return do_units(path, frame, lines, worlds, xs, ys, spans, argv)
    sys.exit(__doc__)


if __name__ == "__main__":
    sys.exit(main())
