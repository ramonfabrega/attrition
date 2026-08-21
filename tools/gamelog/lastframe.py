#!/usr/bin/env python3
"""Cut the last complete BEGIN FRAME block out of a growing gamelog.txt.

lastframe.py [out.txt] [n]

Reads only the tail, so it costs nothing on a multi-gigabyte log. `n` is how
many frames back to start (default 1, i.e. the last complete frame). The
result is small enough to grep or hand to objs.py / one.py.
"""
import os
import sys

LOGS = os.path.expanduser(
    "~/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/"
    "crossover/AppData/Roaming/Microsoft Games/Rise of Nations/Logs")
DEFAULT_OUT = os.path.join(os.environ.get("TMPDIR", "/tmp"), "ron-last.txt")

src = os.path.join(LOGS, "gamelog.txt")
out = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_OUT
n = int(sys.argv[2]) if len(sys.argv) > 2 else 1

size = os.path.getsize(src)
back = min(size, 400000 * (n + 1))
with open(src, "rb") as f:
    f.seek(size - back)
    data = f.read().decode("utf-8", "replace")

idx = [i for i in range(len(data)) if data.startswith("BEGIN FRAME", i)]
if len(idx) < n + 1:
    print("not enough frames in the tail (%d found); raise the read-back size"
          % len(idx))
    sys.exit(1)

chunk = data[idx[-(n + 1)]:idx[-1]]
with open(out, "w") as f:
    f.write(chunk)
print(out, len(chunk), chunk.split("\n")[0])
