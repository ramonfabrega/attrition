#!/usr/bin/env python3
"""gamelog helpers.

gl.py first <BLOCK> [nlines] [file]   print the first 'BEGIN <BLOCK>' and nlines after it (flattened)
gl.py nth <BLOCK> <k> [nlines] [file] print the k-th (0-based) occurrence
gl.py frames [file]                   count BEGIN FRAME and print first/last frame numbers
gl.py grep <regex> [max] [file]       grep with line numbers
gl.py lines <a> <b> [file]            print lines a..b
"""
import sys, os, re
LOGS = os.path.expanduser("~/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs")
DEFAULT = os.path.join(LOGS, "gamelog.txt")

def path(a):
    return a if a and os.path.exists(a) else (os.path.join(LOGS, a) if a and os.path.exists(os.path.join(LOGS, a)) else DEFAULT)

def flat(lines):
    return " | ".join(l.strip() for l in lines)

cmd = sys.argv[1]
if cmd in ("first", "nth"):
    block = sys.argv[2]
    if cmd == "nth":
        k = int(sys.argv[3]); rest = sys.argv[4:]
    else:
        k = 0; rest = sys.argv[3:]
    n = int(rest[0]) if rest else 80
    f = path(rest[1] if len(rest) > 1 else None)
    pat = "BEGIN " + block
    buf = []; seen = -1; grab = 0
    with open(f, encoding="utf-8", errors="replace") as fh:
        for i, line in enumerate(fh):
            if grab:
                buf.append(line); grab -= 1
                if grab == 0: break
                continue
            if line.strip().startswith(pat):
                seen += 1
                if seen == k:
                    print("line", i + 1); buf.append(line); grab = n
    out = flat(buf)
    for i in range(0, len(out), 220):
        print(out[i:i+220])
elif cmd == "frames":
    f = path(sys.argv[2] if len(sys.argv) > 2 else None)
    first = last = None; c = 0
    with open(f, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            if line.lstrip().startswith("BEGIN FRAME"):
                c += 1; last = line.strip()
                if first is None: first = last
    print(c, first, last, os.path.getsize(f))
elif cmd == "grep":
    rx = re.compile(sys.argv[2]); mx = int(sys.argv[3]) if len(sys.argv) > 3 else 50
    f = path(sys.argv[4] if len(sys.argv) > 4 else None)
    c = 0
    with open(f, encoding="utf-8", errors="replace") as fh:
        for i, line in enumerate(fh):
            if rx.search(line):
                print(i + 1, line.rstrip()); c += 1
                if c >= mx: break
elif cmd == "lines":
    a, b = int(sys.argv[2]), int(sys.argv[3])
    f = path(sys.argv[4] if len(sys.argv) > 4 else None)
    with open(f, encoding="utf-8", errors="replace") as fh:
        for i, line in enumerate(fh):
            if a <= i + 1 <= b: sys.stdout.write(line)
            if i + 1 > b: break
