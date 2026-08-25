#!/usr/bin/env python3
"""The animation clock out of a DUMP_ALL dump: every guy's cur_time /
end_time / cur_anim / gpiece per frame block, and what that says about the
art data — the length of each (gpiece, anim) and whether it loops.

    anims.py <dump> [who/o ...]      the per-frame table for those units
    anims.py <dump> --lengths        the (gpiece, anim) -> end_time table
    anims.py <dump> --wraps          every guy whose clock reset between two
                                     passes: same anim (a loop) or a new one

Frame blocks are `BEGIN FRAME n`; the start-of-game dump is frame 0 here.
"""
import re
import sys
from collections import defaultdict

path = sys.argv[1]
args = sys.argv[2:]
want = {a for a in args if not a.startswith("--")}
mode = next((a for a in args if a.startswith("--")), None)

# pass -> [guy dicts]; a pass is one dump of the whole state (the start dump,
# then `begin_frame` and `end_frame` per logged frame), and `passes[i]` is
# labelled by the frame number it was printed under.
frames = defaultdict(list)
labels = {}
seeds = {}
frame = 0
npass = -1
guy = None
in_guy = False
keys = {
    "type", "who", "o", "guy_num", "gpiece", "cur_anim", "cur_time",
    "end_time", "guy_flags", "stopped", "last_time",
}
with open(path, encoding="latin-1") as f:
    for line in f:
        s = line.lstrip()
        if s.startswith("BEGIN "):
            if in_guy and guy and "who" in guy:
                frames[npass].append(guy)
            in_guy = s.startswith("BEGIN GUY")
            guy = {} if in_guy else None
            m = re.match(r"BEGIN FRAME (\d+)", s)
            if m:
                frame = int(m.group(1))
            # A pass is one `GameLog::full_dump` — the start-of-game dump,
            # then `begin_frame`'s and `end_frame`'s under each FRAME.
            if s.startswith("BEGIN FULL DUMP"):
                npass += 1
                labels[npass] = frame
            continue
        if s.startswith("game_random seed "):
            seeds[npass] = int(s.split()[2])
            continue
        if in_guy:
            k, _, v = s.partition(" ")
            if k in keys:
                try:
                    guy[k] = int(v)
                except ValueError:
                    pass
    if in_guy and guy and "who" in guy:
        frames[npass].append(guy)


def label(p):
    return f"F{labels.get(p, '?')}p{p}"


if mode is None:
    for p in sorted(frames):
        print(f"{label(p):>8} seed {seeds.get(p, 0):#010x}")


def key(g):
    return (g["who"], g["o"], g.get("guy_num", 0))


if mode == "--lengths":
    table = defaultdict(set)
    for fr in frames.values():
        for g in fr:
            if g.get("end_time", 0) > 0 and "gpiece" in g:
                table[(g["gpiece"], g["cur_anim"], g["type"])].add(g["end_time"])
    for (gp, an, ty), ends in sorted(table.items()):
        print(f"gpiece {gp:4d} type {ty:3d} anim {an:3d} end_time {sorted(ends)}")
elif mode == "--wraps":
    order = sorted(frames)
    for a, b in zip(order, order[1:]):
        prev = {key(g): g for g in frames[a]}
        for g in frames[b]:
            p = prev.get(key(g))
            if not p or "cur_time" not in g or "cur_time" not in p:
                continue
            if g["cur_time"] < p["cur_time"] or g["cur_anim"] != p["cur_anim"]:
                who, o, n = key(g)
                print(
                    f"{label(a)}->{label(b)} {who}/{o}#{n} type {g['type']} gpiece {g['gpiece']}: "
                    f"anim {p['cur_anim']} {p['cur_time']}/{p['end_time']} -> "
                    f"anim {g['cur_anim']} {g['cur_time']}/{g['end_time']}"
                    + ("  LOOP" if g["cur_anim"] == p["cur_anim"] else "  NEW")
                )
else:
    for fr in sorted(frames):
        for g in frames[fr]:
            who, o, n = key(g)
            if want and f"{who}/{o}" not in want:
                continue
            print(
                f"{label(fr):>8} {who}/{o}#{n} type {g.get('type')} gpiece {g.get('gpiece')} "
                f"anim {g.get('cur_anim')} {g.get('cur_time')}/{g.get('end_time')} "
                f"last {g.get('last_time')} flags {g.get('guy_flags')} stopped {g.get('stopped')}"
            )
