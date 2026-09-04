#!/usr/bin/env python3
"""Rewrite gamelog.ini.

usage: setlog.py DUMP_ALL end:CAT[=N],... [start:CAT[=N],...]
  every category under [End Frame] (and, if given, [Start Game]) not listed -> 0

The ini value is a **detail threshold**, not a flag (docs/ORACLE.md, "The
detail level is the knob"), so `UNITS=3` and a bare `UNITS` are different
settings; a bare name means 1. The useful levels are UNITS=3, BUILDS=7,
CITIES=5, GUYS=2, LEADERS=9, WORLD=6, GOODS=3, TERRAIN=2 — and GROUPS=1,
which `GameLog::full_dump` gates `dump_groups` on, so the whole 512-slot
group pool is a per-frame record without DUMP_ALL.
"""
import sys, os
D = os.path.expanduser("~/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/gamelog.ini")
dump_all = sys.argv[1]
SECTIONS = {"end": "[End Frame]", "start": "[Start Game]", "misc": "[Misc Logging]",
            "endgame": "[End Game]", "startframe": "[Start Frame]"}
sections = {}
for a in sys.argv[2:]:
    name, cats = a.split(":", 1)
    want = {}
    for c in cats.split(","):
        if not c:
            continue
        k, _, v = c.partition("=")
        want[k] = v or "1"
    sections[SECTIONS[name]] = want
out = []
section = None
for line in open(D, encoding="utf-8", errors="replace").read().splitlines():
    s = line.strip()
    if s.startswith("[") and s.endswith("]"):
        section = s
        out.append(line); continue
    if s.startswith("DUMP_ALL="):
        out.append("DUMP_ALL=" + dump_all); continue
    if section in sections and "=" in s and not s.startswith("DumpFileName") and not s.startswith("LogFile"):
        k = s.split("=")[0]
        out.append(k + "=" + sections[section].get(k, "0")); continue
    out.append(line)
open(D, "w", encoding="utf-8").write("\n".join(out) + "\n")
print("DUMP_ALL=%s; %s" % (dump_all, {k: sorted("%s=%s" % kv for kv in v.items())
                                      for k, v in sections.items()}))
