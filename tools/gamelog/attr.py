#!/usr/bin/env python3
"""The attrition instrument: every unit's period, its ticks, and the phase lock.

attr.py [gamelog] [--who N] [--o N] [--from F] [--to F]

For each unit (who, o) whose `attrition`, `supply`, `damage` or `damage_frac`
ever changes, prints

  * the period timeline — every frame the logged `attrition` value changed;
  * every tick — a frame where damage*16 + damage_frac grew, with the delta in
    sixteenths and the period in force;
  * the phase check — `docs/ATTRITION.md`, "The cadence": a tick lands on
    sim-frame f only when (f + o) % period == 0, and the period refreshes on
    (f + o) % 32 == 0. The log's `FRAME n` label is one ahead of the sim-frame
    (`docs/ORACLE.md`, "The frame label, settled"), so the check is run with
    the label offset -1; the offset 0 fit is printed beside it as the control.

Reads `UNITS=3` records (attrition, supply, damage, damage_frac, myhits are
level-3 fields). Frames are the log's labels; `sim = label - 1`.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from track import records, DEFAULT  # noqa: E402

FIELDS = ("attrition", "supply", "damage", "damage_frac", "myhits", "x_internal", "y_internal",
          "damage_o", "damage_who", "damage_frame")

# An attrition tick is +6/16 for a figure of a 3-squad, +16/16 (one whole point)
# for a lone figure or a wagon; anything else is combat (`docs/ATTRITION.md`,
# "The damage"). Sizes 2 and 4 would be 8 and 4.
ATTRITION_DELTAS = {4, 6, 8, 16}


def main(argv):
    path = DEFAULT
    who = o_sel = lo = hi = None
    i = 0
    while i < len(argv):
        a = argv[i]
        if a == "--who":
            who = argv[i + 1]; i += 2
        elif a == "--o":
            o_sel = argv[i + 1]; i += 2
        elif a == "--from":
            lo = int(argv[i + 1]); i += 2
        elif a == "--to":
            hi = int(argv[i + 1]); i += 2
        else:
            path = a if os.path.exists(a) else os.path.join(os.path.dirname(DEFAULT), a); i += 1

    # per unit: list of (frame, dict)
    hist = {}
    with open(path, encoding="utf-8", errors="replace") as fh:
        for frame, r in records(fh, "UNITDATA"):
            if lo is not None and frame < lo:
                continue
            if hi is not None and frame > hi:
                break
            if who is not None and r.get("who") != who:
                continue
            if o_sel is not None and r.get("o") != o_sel:
                continue
            key = (int(r.get("who", -1)), int(r.get("o", -1)))
            vals = {k: r.get(k) for k in FIELDS}
            if vals["attrition"] is None or vals["damage"] is None:
                continue  # a block without the level-3 fields (the end-of-game dump)
            hist.setdefault(key, []).append((frame, vals))

    for key in sorted(hist):
        seq = hist[key]
        w, o = key
        first = seq[0][1]
        # does anything move?
        moving = any(s[1]["attrition"] != first["attrition"] or s[1]["supply"] != first["supply"]
                     or s[1]["damage"] != first["damage"] or s[1]["damage_frac"] != first["damage_frac"]
                     for s in seq)
        if not moving and (first["attrition"] in (None, "0")):
            continue
        print("== who %d o %d  hits %s  first frame %d tile (%.1f,%.1f)  last frame %d" % (
            w, o, first["myhits"], seq[0][0],
            int(first["x_internal"] or 0) / 192.0, int(first["y_internal"] or 0) / 192.0, seq[-1][0]))
        # period timeline
        last_att = None
        last_sup = None
        for frame, v in seq:
            if v["attrition"] != last_att or v["supply"] != last_sup:
                sim = frame - 1
                print("   frame %6d (sim %6d, (sim+o)%%32=%2d)  attrition %s  supply %s" % (
                    frame, sim, (sim + o) % 32, v["attrition"], v["supply"]))
                last_att, last_sup = v["attrition"], v["supply"]
        # ticks — every rise of damage*16 + damage_frac. A drop (or a change of
        # myhits) is an object number being recycled by a new unit: the history
        # restarts there. A rise that is not an attrition-sized delta, or that
        # comes with a new damage_frame, is combat and is listed separately.
        prev_total = None
        prev_att = None
        prev_dframe = None
        ticks = []
        combat = []
        for frame, v in seq:
            try:
                total = int(v["damage"] or 0) * 16 + int(v["damage_frac"] or 0)
            except ValueError:
                continue
            if prev_total is not None and total < prev_total:
                prev_total = None  # recycled
            if prev_total is not None and total != prev_total:
                delta = total - prev_total
                per = int(v["attrition"] or 0)
                if delta in ATTRITION_DELTAS and per > 0 and v["damage_frame"] == prev_dframe:
                    ticks.append((frame, delta, v["attrition"], prev_att))
                else:
                    combat.append((frame, delta, v["damage_o"], v["damage_who"]))
            prev_total = total
            prev_att = v["attrition"]
            prev_dframe = v["damage_frame"]
        if ticks:
            print("   attrition ticks: %d" % len(ticks))
            fit = {0: 0, -1: 0}
            for frame, delta, att, att_before in ticks:
                per = int(att or 0)
                marks = []
                for off in (0, -1):
                    sim = frame + off
                    ok = per > 0 and (sim + o) % per == 0
                    fit[off] += ok
                    marks.append("%s%d" % ("ok" if ok else "NO", off))
                print("   frame %6d  +%3d/16  period %s (was %s)  %s" % (frame, delta, att, att_before, " ".join(marks)))
            print("   phase fit: offset -1 %d/%d, offset 0 %d/%d" % (fit[-1], len(ticks), fit[0], len(ticks)))
        if combat:
            print("   other damage: %d rises, first %s, last %s" % (
                len(combat), combat[0], combat[-1]))


if __name__ == "__main__":
    main(sys.argv[1:])
