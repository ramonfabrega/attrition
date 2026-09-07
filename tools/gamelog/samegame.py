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

`--exclude NAME` drops every `BEGIN NAME` block from both digests, which is
what makes the comparison mean anything when the new capture was taken at a
*higher detail* than the old one. A capture that raises one category's
threshold to read a field — run40 taking `LEADERS=2` for the goody bucket
where run39 had 1 — adds lines to that category's blocks and to no others,
so every frame's digest differs and the same-game question goes unanswered
by default. Excluding the category that moved asks the question that is
still worth asking: is everything the two runs *do* record in common
identical, frame for frame. It is a weaker claim than a bare run and should
be read as one — the excluded record is unchecked, not checked and equal.

`--drop KEY` drops every line whose first token is KEY, wherever it sits.
It exists because `--exclude` is a **record** and the smallest thing a
detail level adds is sometimes a *field*: at `GUYS=4` an `ANIMALDATA`
prints three of its own — `ox`, `whom`, `aid` — after its nested
`UNITDATA` and outside any `GUY` block, so `--exclude GUY` alone leaves
120 lines a block (forty animals) between run79 and run87 and
`--exclude ANIMALDATA` would throw away all forty animals to remove them.
Dropping the three keys removes exactly those lines and keeps every
animal's record. Weaker than a bare run, and to be read as one: the
dropped field is unchecked, not checked and equal.

A block runs from its `BEGIN NAME` to the next line indented no deeper,
except that a line at the *same* depth which is not itself a `BEGIN` still
belongs to it: the dump indents a leader's `who` one deeper than its `BEGIN
LEADERDATA` but its `leader_flags` at the same depth, and both are the
leader's.
"""
import hashlib
import os
import sys

ARCHIVE = os.environ.get(
    "RON_GAMELOG_DIR",
    os.path.expanduser(
        "~/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"))


def digests(path, exclude=(), drop=()):
    """{frame: sha1 of the block's indented lines}, minus the excluded blocks."""
    out, frame, h = {}, None, None
    heads = tuple(b"BEGIN " + e.encode() for e in exclude)
    keys = tuple(k.encode() for k in drop)
    skip_depth = None
    with open(path, "rb") as f:
        for line in f:
            s = line.rstrip(b"\r\n")
            t = s.lstrip(b" ")
            depth = len(s) - len(t)
            if t.startswith(b"BEGIN FRAME "):
                if frame is not None:
                    out[frame] = h.hexdigest()
                try:
                    frame = int(t.split()[2])
                except (IndexError, ValueError):
                    frame = None
                h = hashlib.sha1()
                skip_depth = None
                continue
            if skip_depth is not None:
                # Deeper is inside; the same depth and not a BEGIN is still
                # the block's own (a leader's `leader_flags` sits at its
                # `BEGIN`'s depth); anything else has ended it.
                if depth > skip_depth:
                    continue
                if depth == skip_depth and not t.startswith(b"BEGIN "):
                    continue
                skip_depth = None
            if heads and any(t == e or t.startswith(e + b" ") for e in heads):
                skip_depth = depth
                continue
            if keys and t.split(b" ", 1)[0] in keys:
                continue
            if frame is not None and s.startswith(b" "):
                h.update(s + b"\n")
    if frame is not None:
        out[frame] = h.hexdigest()
    return out


def main():
    args, exclude, drop = [], [], []
    it = iter(sys.argv[1:])
    for a in it:
        if a == "--exclude":
            exclude.append(next(it))
        elif a.startswith("--exclude="):
            exclude.append(a.split("=", 1)[1])
        elif a == "--drop":
            drop.append(next(it))
        elif a.startswith("--drop="):
            drop.append(a.split("=", 1)[1])
        else:
            args.append(a)
    a, b = (p if os.path.isabs(p) else os.path.join(ARCHIVE, p)
            for p in args[:2])
    if exclude:
        print("excluding: %s" % ", ".join(exclude))
    if drop:
        print("dropping keys: %s" % ", ".join(drop))
    da, db = digests(a, exclude, drop), digests(b, exclude, drop)
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
