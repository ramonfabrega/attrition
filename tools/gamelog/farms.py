#!/usr/bin/env python3
"""farms.py <gamelog> block [block ...] -- the Farms list at the first pass of
each frame block: per farm (in list order) who/o, empties, and every
non-empty cell as (print-index, status, percent)."""
import sys

path = sys.argv[1]
blocks = [None if b == "init" else int(b) for b in sys.argv[2:]]


def farms_at(block):
    frame = None
    passes = 0
    out = []
    grab = False
    cur = None
    n = None
    with open(path, encoding="latin-1") as f:
        for line in f:
            s = line.strip()
            if s.startswith("BEGIN FRAME"):
                frame = int(s.split()[2])
                passes = 0
                continue
            if frame != block:
                continue
            if s == "BEGIN FULL DUMP":
                passes += 1
                continue
            if passes != 1:
                continue
            if s.startswith("wheat_min_height"):
                grab = True
                continue
            if not grab:
                continue
            parts = s.split(" ")
            k, v = " ".join(parts[:-1]), parts[-1]
            if k == "length" and n is None:
                n = int(v)
                continue
            if k == "who":
                cur = {"who": v, "cells": [], "extra": {}}
                out.append(cur)
                continue
            if cur is None:
                continue
            if k == "o":
                cur["o"] = v
            elif k.startswith("percent"):
                cur["cells"].append([float(v), None])
            elif k.startswith("status"):
                cur["cells"][-1][1] = int(v)
                if len(out) == n and len(cur["cells"]) == 16:
                    return out
            elif len(cur["cells"]) == 16:
                cur["extra"][k] = v
    return out


for b in blocks:
    print(f"=== block {b} (end of sim-frame {b - 1 if b is not None else 'setup'})")
    for i, fm in enumerate(farms_at(b)):
        cells = fm["cells"]
        empty = sum(1 for p, st in cells if st == 0)
        busy = [(j, st, p) for j, (p, st) in enumerate(cells) if st != 0]
        print(f"  farm {i}: who {fm.get('who')} o {fm.get('o')} empty {empty} cells {busy}")
