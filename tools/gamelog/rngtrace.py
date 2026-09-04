#!/usr/bin/env python3
"""rngtrace.py [gamelog] [cap]  -- the sync stream's state at every checkpoint

`GameLog::say_checksum@00930b30` prints, at `check_all_level >= 14` in
`rise.ini`, the `game_random seed` at every call site of the setup path
(`[Misc Logging] CHECKSUM=2` takes `Game::init`, `init_rules_and_teams`,
`Setup::build_game`, `Map::make`, `Terrain::init`, `Leader::init`) and, under
`DUMP_ALL=1`, at every `full_dump` — begin_game, begin_frame, end_frame. One
row per record: the record number, source file and line, the seed, and the
number of `Random::get` draws since the previous record (a forward walk of
the 32-bit LCG, capped). `docs/ORACLE.md`, "The setup path's checksum trace
is the RNG state".
"""
import os
import re
import sys

DEFAULT = os.path.expanduser(
    "~/ron-data/"
    "AppData/Roaming/Microsoft Games/Rise of Nations/Logs/gamelog.txt")
path = sys.argv[1] if len(sys.argv) > 1 else DEFAULT
CAP = int(sys.argv[2]) if len(sys.argv) > 2 else 3_000_000


def steps(a, b, cap=CAP):
    s = a
    for k in range(cap + 1):
        if s == b:
            return k
        s = (s * 0x19660D + 0x3C6EF35F) & 0xFFFFFFFF
    return None


rows = []
cur = None
frame = None
with open(path, encoding="latin-1") as f:
    for lineno, l in enumerate(f, 1):
        s = l.strip()
        if l.startswith("BEGIN FRAME") or l.startswith(" BEGIN FRAME"):
            frame = s.split()[2]
        m = re.match(r"CHECKSUM (\d+)$", s)
        if m:
            cur = {"n": int(m.group(1)), "at": lineno, "frame": frame, "file": "?", "line": 0}
            continue
        if cur is None:
            continue
        m = re.match(r"FILE (\S+)", s)
        if m:
            cur["file"] = m.group(1)
            continue
        m = re.match(r"LINE (\d+)", s)
        if m:
            cur["line"] = int(m.group(1))
            continue
        m = re.match(r"game_random seed (-?\d+)", s)
        if m:
            cur["seed"] = int(m.group(1)) & 0xFFFFFFFF
            rows.append(cur)
            cur = None

prev = None
for r in rows:
    d = "" if prev is None else steps(prev, r["seed"])
    if d is None:
        d = f">{CAP}"
    print(
        f"{r['n']:5d} {r['file']:<14} {r['line']:6d} 0x{r['seed']:08x} +{d:<6} "
        f"(log line {r['at']}, frame {r['frame']})"
    )
    prev = r["seed"]
