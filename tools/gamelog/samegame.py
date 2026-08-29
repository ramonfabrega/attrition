#!/usr/bin/env python3
"""samegame.py A B — are two dumps of the same game, frame for frame?

    samegame.py gamelog-run10-world6-long.txt gamelog-run33-longtrace.txt

Both names are taken relative to the archive directory (`RON_GAMELOG_DIR`,
or the bottle's `Logs`). Each `BEGIN FRAME n` block is reduced to a digest
of its **indented** lines — the dump proper, with the ambient `[Misc
Logging]` chatter that sits at column 0 (and carries wall-clock stamps)
left out — and the two digests are compared frame by frame.

Why it exists: a capture taken with the traced executable, or with
`!ffwd`, is only useful as a *longer sibling* of an existing dump if the
game it played was the same one. run18a checked that over four frames by
hand; this checks it over every frame both files hold, which is what
lets a new capture inherit an old one's siblings and retire its tests.

Output is the first differing frame and how many matched, so a `0` in the
"differ" column is the claim "these are the same game".

Each file's **last** block is dropped before comparing: a run ends by
quitting, and the block the quit interrupts is written short. run10 against
run14 is the calibration — 284 blocks identical, and the only difference in
the file was run14's truncated 285th.
"""
import hashlib
import os
import sys

ARCHIVE = os.environ.get(
    "RON_GAMELOG_DIR",
    os.path.expanduser(
        "~/Library/Application Support/CrossOver/Bottles/ron/drive_c/users"
        "/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"))


def digests(path):
    """{frame: sha1 of the block's indented lines}."""
    out, frame, h = {}, None, None
    with open(path, "rb") as f:
        for line in f:
            s = line.rstrip(b"\r\n")
            t = s.lstrip(b" ")
            if t.startswith(b"BEGIN FRAME "):
                if frame is not None:
                    out[frame] = h.hexdigest()
                try:
                    frame = int(t.split()[2])
                except (IndexError, ValueError):
                    frame = None
                h = hashlib.sha1()
                continue
            if frame is not None and s.startswith(b" "):
                h.update(s + b"\n")
    if frame is not None:
        out[frame] = h.hexdigest()
    return out


def main():
    a, b = (p if os.path.isabs(p) else os.path.join(ARCHIVE, p)
            for p in sys.argv[1:3])
    da, db = digests(a), digests(b)
    for d in (da, db):
        if d:
            del d[max(d)]
    common = sorted(set(da) & set(db))
    print("frames: %d and %d, %d in common (%d..%d)"
          % (len(da), len(db), len(common),
             common[0] if common else -1, common[-1] if common else -1))
    differ = [n for n in common if da[n] != db[n]]
    print("differ: %d; first: %s" % (len(differ), differ[:8]))
    if differ:
        return 1
    print("same game over every common frame")
    return 0


if __name__ == "__main__":
    sys.exit(main())
