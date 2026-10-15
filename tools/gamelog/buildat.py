#!/usr/bin/env python3
"""buildat.py — a building's queue across frames, out of a dump: one building
by name, or every trainer of a player, printed on the first frame and then
only when its queue changed.

    python3 tools/gamelog/buildat.py <dump> <frame | lo-hi> <who>[/<o>] [--enums DIR]

Loop 2459, three reaches (2412's scratch script; 2491's and 2526's hand reads
of `BUILDQUEUE`'s counters across every trainer of one player, which named
the President's and the Space Program's arms): `unitat.py` names a unit, and
a building's queue — its type, `queued`, the head `job_counter`, the slots —
took a throwaway each time. A queue clock that never moves (run853's `1/2018`,
its second ICBM at 71400 against 0) is invisible to every other tool on disk.

The dump prints all twenty slots and the ones past `queued` keep their last
entry (Loop 2407: 2357 nearly booked a refutation on four stale Bombers), so
the slots are cut at `queued`. A dump with CRLF line ends reads the same
(2526). `--enums DIR` names the enum export as `unitat.py` does; without it
the type number stands alone. Exits 2 when no named building is on any
frame asked.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from unitat import DEFAULT_ENUMS, deep_field, field, parse_block, type_names  # noqa: E402


def buildings(path, lo, hi):
    """Yield `(frame, block lines)` for every BUILDDATA block on frames in `[lo, hi]`."""
    frame = None
    block = None
    depth = None
    with open(path, encoding='utf-8', errors='replace') as fh:
        for line in fh:
            s = line.rstrip('\r\n')
            stripped = s.lstrip(' ')
            indent = len(s) - len(stripped)
            if stripped.startswith('BEGIN FRAME '):
                if block:
                    yield frame, block
                    block = None
                frame = int(stripped.split()[2])
                continue
            if frame is None or not (lo <= frame <= hi):
                continue
            if block is not None:
                if stripped and indent <= depth:
                    yield frame, block
                    block = None
                else:
                    block.append(s)
                    continue
            if stripped == 'BEGIN BUILDDATA':
                block = [s]
                depth = indent
        if block:
            yield frame, block


def queue_of(node):
    """`(type, queued, [(slot type, slot job_counter) …])`, the slots cut at
    `queued`; the head's counter is slot 0's (the object's own `job_counter`
    is the building's, not the queue's)."""
    ty = int(field(node, 'orig_type', -1))
    queued = int(field(node, 'queued', 0))
    slots = []
    for name, child in node['children']:
        if name == 'BUILDQUEUE':
            types = [int(v) for k, v in child['fields'] if k == 'queue[scan].type']
            counters = [int(v) for k, v in child['fields'] if k == 'queue[scan].job_counter']
            slots = list(zip(types, counters))[:queued]
    return ty, queued, slots


def describe(who, o, ty, queued, slots, names):
    def named(n):
        return f'{n} {names[n]}' if n in names else str(n)
    inside = ', '.join(f'{named(t)}:{c}' for t, c in slots)
    return f'{who}/{o} {named(ty)} queued {queued} [{inside}]'


def parse_args(argv):
    import argparse
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('dump')
    ap.add_argument('frames', help='one frame, or lo-hi')
    ap.add_argument('name', help='<who> for every trainer of a player, or <who>/<o> for one building')
    ap.add_argument('--enums', default=DEFAULT_ENUMS)
    a = ap.parse_args(argv)
    lo, _, hi = a.frames.partition('-')
    a.lo, a.hi = int(lo), int(hi or lo)
    who, _, o = a.name.partition('/')
    a.who, a.o = int(who), (int(o) if o else None)
    return a


def main(argv=None):
    a = parse_args(argv if argv is not None else sys.argv[1:])
    names = type_names(a.enums)
    last = {}
    seen = False
    for frame, lines in buildings(a.dump, a.lo, a.hi):
        node = parse_block(lines)
        who, o = int(deep_field(node, 'who', -1)), int(deep_field(node, 'o', -1))
        if who != a.who or (a.o is not None and o != a.o):
            continue
        ty, queued, slots = queue_of(node)
        state = (ty, queued, tuple(slots))
        # A player's trainers: a building whose queue was never non-empty in
        # the window is not one.
        if a.o is None and queued == 0 and (who, o) not in last:
            continue
        seen = True
        if last.get((who, o)) == state:
            continue
        last[(who, o)] = state
        print(f'f{frame} {describe(who, o, ty, queued, slots, names)}')
    return 0 if seen else 2


if __name__ == '__main__':
    sys.exit(main())
