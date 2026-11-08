#!/usr/bin/env python3
"""One frame's block, cut out of any archived gamelog by number.

frame.py FILE FRAME [OUT]

`lastframe.py` cuts the *tail* of a growing `gamelog.txt`, which is what a
live capture needs. This cuts frame `n` out of a finished, archived dump —
`gamelog-run71-greatlakes-5k.txt` is 832 MB and streaming it costs a few
seconds, against loading it whole. The result is small enough to hand to
`objs.py` or `one.py`, which is the point:

    python3 tools/gamelog/frame.py "$LOGS/gamelog-run71-greatlakes-5k.txt" 4177
    python3 tools/gamelog/one.py UNITDATA 1 11 --file /tmp/ron-runs/f4177.txt

**Two traps, each of which reads as "the frame is not in the file".** The
dumps are CRLF, so a comparison against `"BEGIN FRAME 4177\\n"` never fires;
and the frame line carries a **leading space**, so even `rstrip()` is not
enough. Compare on the fully stripped line. Both cost an iteration on
2026-09-03, and `grep -a` finding the line while the reader does not is
exactly what they look like.

**The cut ends at the next `BEGIN FRAME`, whatever its number.** A capture
with a stride dumps a block every fifty frames, so the line after 651's
block is `BEGIN FRAME 701`; a cut that waited for `652` ran to the dump's
end — 2.0 M lines for 651 on run1542, forty frames, and `one.py --file`
read the last record in the file rather than the frame's (Loop 3977).
"""
import os
import sys

if len(sys.argv) < 3:
    sys.exit(__doc__)

path, frame = sys.argv[1], int(sys.argv[2])
out = sys.argv[3] if len(sys.argv) > 3 else os.path.join(
    os.environ.get("RON_TMP", "/tmp/ron-runs"), f"f{frame}.txt"
)
os.makedirs(os.path.dirname(out), exist_ok=True)

start = f"BEGIN FRAME {frame}"
keep, n = False, 0
with open(path, errors="replace") as fh, open(out, "w") as w:
    for line in fh:
        s = line.strip()
        if s.startswith("BEGIN FRAME"):
            if keep:
                break
            keep = s == start
        if keep:
            # Keep the indentation: `leader.py` cuts a block by it, and a
            # slice that flattened every line ended the LEADERDATA block on
            # its first line — "no LEADERDATA for who 1" on a frame that
            # held four (parked 755). `objs.py` and `one.py` strip per line.
            w.write(line.rstrip("\r\n") + "\n")
            n += 1

print(f"{out}: {n} lines")
if not n:
    sys.exit(f"frame {frame} is not in {path}")
