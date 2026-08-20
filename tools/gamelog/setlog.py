#!/usr/bin/env python3
"""Rewrite gamelog.ini.

usage: setlog.py DUMP_ALL end:CAT,CAT,... [start:CAT,CAT,...]
  every category under [End Frame] (and, if given, [Start Game]) not listed -> 0
"""
import sys, os
D = os.path.expanduser("~/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/gamelog.ini")
dump_all = sys.argv[1]
sections = {}
for a in sys.argv[2:]:
    name, cats = a.split(":", 1)
    sections[{"end": "[End Frame]", "start": "[Start Game]", "misc": "[Misc Logging]", "endgame": "[End Game]", "startframe": "[Start Frame]"}[name]] = set(c for c in cats.split(",") if c)
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
        out.append(k + "=" + ("1" if k in sections[section] else "0")); continue
    out.append(line)
open(D, "w", encoding="utf-8").write("\n".join(out) + "\n")
print("DUMP_ALL=%s; %s" % (dump_all, {k: sorted(v) for k, v in sections.items()}))
