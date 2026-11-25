#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4", "capstone==5.0.3"]
# ///
"""sweep_keys.py — `get_position.py … sweep` for a list of keys, one oracle.

    uv run tools/recomp/sweep_keys.py <exe> <packet> <keys.txt> <outdir>

`keys.txt` holds one `piece node anim time` a line (the bays' lister's
`KEY` lines, item 4651, with the class columns cut). For each piece the
keys its packet does not already store are first written by
`GraphicPieces::init_position@00901820` (axis 3, as every shipped
`<RELEASEEVENT>` is), exactly as `get_position.py --init` does, and each
key is then swept at every whole degree `0..=360` into
`<outdir>/sweep-<piece>-<node>-<anim>-<time>.txt`, the same columns
`get_position.py … sweep` prints (`deg ret x y z fx fy fz`), so
`fit_bay.py` and the crate's sweep tests read it unchanged.

It is `get_position.py`'s `Oracle` driven over many pieces in one process:
the packet is loaded once a piece, where the single-key form reloads it for
every key (item 4618 ran 75 keys that way; the lister names hundreds). A key
`init_position` refuses, or faults on (a read the packet does not hold), is
printed and skipped. Nothing it writes enters
git.
"""
import os
import sys
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import get_position  # noqa: E402


def main():
    if len(sys.argv) != 5:
        sys.exit(__doc__)
    exe, packet, keys, out = sys.argv[1:]
    os.makedirs(out, exist_ok=True)
    by_piece = defaultdict(list)
    for line in open(keys):
        f = line.split()
        if len(f) >= 4:
            by_piece[int(f[0])].append(tuple(int(x) for x in f[1:4]))
    done = refused = 0
    for piece, ks in sorted(by_piece.items()):
        # A fresh machine a piece: `init_position` grows the engine's
        # arrays through the heap, and one piece's growth must not be the
        # next one's state.
        o = get_position.Oracle(exe, packet)
        _, _, _, _, rows = o.entries(piece)
        stored = {(n, a, t) for n, a, t, *_ in rows}
        for node, anim, time in ks:
            if (node, anim, time) in stored:
                continue
            try:
                r = o.init_position(piece, node, anim, time, 3)
            except Exception as e:  # unicorn's UcError: a load the packet lacks
                r = f"{type(e).__name__}: {e}"
            if r != 0:
                print(f"REFUSED init_position({piece}, {node}, {anim}, {time}, 3) = {r}")
                refused += 1
                continue
            stored.add((node, anim, time))
        for node, anim, time in ks:
            if (node, anim, time) not in stored:
                continue
            path = os.path.join(out, f"sweep-{piece}-{node}-{anim}-{time}.txt")
            with open(path, "w") as w:
                for deg in range(0, 361):
                    r, v = o.get(piece, node, anim, time, deg)
                    w.write(
                        f"{deg} {r} {int(v[0])} {int(v[1])} {int(v[2])} "
                        f"{v[0]!r} {v[1]!r} {v[2]!r}\n"
                    )
            done += 1
        print(f"piece {piece}: {len(ks)} keys", flush=True)
    print(f"swept {done}, refused {refused}")


if __name__ == "__main__":
    main()
