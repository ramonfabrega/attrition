#!/usr/bin/env python3
"""unitaddr.py — a unit's address in a packet, from its position.

    python3 tools/recomp/unitaddr.py <frame-snapshot.bin> <x> <y>

`step4.py` on a unit needs the unit's address, and three lanes found it by
scanning the packet for the masked position `(x ^ 0x63637, y ^ 0x63637)`
at `+0x10` of the unit (parked 1953: 1845 read `units.lists` by hand, 1868
and 1948 scanned). Every hit is printed, one address a line, `x` and `y`
the unit's internal coordinates as the dump prints them (`x_internal`,
`y_internal`). Exits 1 when nothing matches. A reading instrument: nothing
here is a source, and the packet never enters the repository.
"""
import struct
import sys

sys.path.insert(0, __file__.rsplit('/', 1)[0])

MASK = 0x63637
AT = 0x10


def find(ranges, x, y):
    """Every address whose `+AT` holds the masked pair, over `(base, bytes)`."""
    pair = struct.pack('<II', (x ^ MASK) & 0xFFFFFFFF, (y ^ MASK) & 0xFFFFFFFF)
    hits = []
    for base, data in ranges:
        at = data.find(pair)
        while at >= 0:
            if at >= AT:
                hits.append(base + at - AT)
            at = data.find(pair, at + 4)
    return hits


def parse(argv):
    if len(argv) != 3:
        print(__doc__.strip().splitlines()[2].strip(), file=sys.stderr)
        raise SystemExit(64)
    return argv[0], int(argv[1], 0), int(argv[2], 0)


def main(argv=None):
    path, x, y = parse(sys.argv[1:] if argv is None else argv)
    from snapshot import Snapshot
    snap = Snapshot(path)

    def ranges():
        for base, size, off in snap.ranges:
            snap.f.seek(off)
            yield base, snap.f.read(size)

    hits = find(ranges(), x, y)
    for a in hits:
        print(f'{a:08x}')
    return 0 if hits else 1


if __name__ == '__main__':
    sys.exit(main())
