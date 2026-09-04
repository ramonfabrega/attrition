#!/usr/bin/env python3
"""turnanim.py DUMP — did any guy ever play a turn animation?

    turnanim.py gamelog-run44-islands-turners.txt

`Guy::do_turn@005d97a0:15` replaces the standing walk with `CHAR_TURN_LEFT`
(slot 21) or `CHAR_TURN_RIGHT` (22) for a guy whose piece names one —
`guy_flags & 8`, which `Guy::init_real@005db6b0:179` sets. 273 of the
install's unit pieces name a turn and **none of the eight any traced guy
carries do**, so the override has never executed in any capture on disk and
`docs/ANIM.md` §4.6 leaves it unmodelled deliberately (queue item 96).

This is the one-line verdict on a capture staged to fire it: every guy whose
`cur_anim` is 21 or 22, with the frame and the unit it belongs to. Exit 1
when there are none, so a stanza's `check:` line fails loudly on a capture
that did not reach what it was staged for.

`cur_anim` needs **`GUYS=4`**: `GuyData::log_data@005de6c0` announces levels
2, 3 and 4 through the Log vtable's `+0x28`, and the clock fields sit past
the last of them. At `GUYS=2` — run39's setting — a `GUY` block stops after
`ox` and the question cannot be asked of the file at all.
"""
import os
import sys

TURN = {21: "CHAR_TURN_LEFT", 22: "CHAR_TURN_RIGHT"}
ARCHIVE = os.environ.get(
    "RON_GAMELOG_DIR",
    os.path.expanduser(
        "~/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"))


def main():
    path = sys.argv[1]
    if not os.path.isabs(path):
        path = os.path.join(ARCHIVE, path)

    frame = None
    unit = {}
    guy = {}
    saw_curanim = False
    hits = []
    in_guy = False

    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            s = line.strip()
            if s.startswith("BEGIN FRAME "):
                try:
                    frame = int(s.split()[2])
                except (IndexError, ValueError):
                    frame = None
                continue
            if s.startswith("BEGIN UNITDATA") or s.startswith("BEGIN ANIMALDATA"):
                unit = {}
                in_guy = False
                continue
            if s.startswith("BEGIN GUY"):
                guy = {}
                in_guy = True
                continue
            if s.startswith("BEGIN "):
                in_guy = False
                continue
            parts = s.split()
            if len(parts) != 2:
                continue
            k, v = parts
            try:
                v = int(v)
            except ValueError:
                continue
            if in_guy:
                guy[k] = v
                if k == "cur_anim":
                    saw_curanim = True
                    if v in TURN:
                        hits.append((frame, guy.get("who", unit.get("who")),
                                     guy.get("o", unit.get("o")),
                                     guy.get("type"), v))
            else:
                unit[k] = v

    if not saw_curanim:
        print("no `cur_anim` in this dump at all — it was not taken at GUYS=4,"
              " so the question cannot be asked of it")
        return 1

    if not hits:
        print("no guy played a turn animation: 0 of slots 21/22")
        return 1

    print("%d guy-frames played a turn animation" % len(hits))
    seen = set()
    for frame, who, o, typ, slot in hits:
        key = (who, o, slot)
        if key in seen:
            continue
        seen.add(key)
        print("  frame %-6s who %-3s o %-5s piece type %-5s %s"
              % (frame, who, o, typ, TURN[slot]))
    print("%d distinct (who, o, slot) combinations" % len(seen))
    return 0


if __name__ == "__main__":
    sys.exit(main())
