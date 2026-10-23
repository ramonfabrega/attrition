#!/usr/bin/env python3
"""ammo.py — every live `Ammo` on a packet, read as `AmmoData`.

    python3 tools/recomp/ammo.py <frame-snapshot.bin> [--all]

No capture writes an `AMMO` record, so where a round leaves from, where it
lands and how long it flies are a packet's to say. 3518 and 3519 each wrote
a scratch scanner for this; 3523 was the third reach (CLAUDE.md, "a probe
shape reached for a third time graduates").

The scan finds `Ammo::vftable` (`0xb45418`) at every aligned word of the
packet's ranges and reads the object through `AmmoData`'s layout
(`types.txt`, `struct AmmoData`, size 0x6c): `flags` `+0x4`, `rolling`
`+0x5`, `accuracy` `+0x6`, the launch `sx/sy/sz` `+0xc`..`+0x14`, the
landing `ex/ey/ez` `+0x18`..`+0x20`, `cur_time`/`total_time` `+0x24`/`+0x28`,
`angle` `+0x2c`, `splash_area` `+0x30`, `index` `+0x34`, the shooter
`who/o` `+0x3c`/`+0x40`, `num_guys` `+0x44`, the target `whom/ox`
`+0x48`/`+0x4c`. A slot is live when `flags & 3` is set — the test
`Objects::add_ammo@00658b10` frees a slot by. `--all` prints the free ones
too. Original-derived output: it stays outside git.
"""
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from snapshot import Snapshot  # noqa: E402

VFTABLE = 0xB45418
SIZE = 0x6C


def rounds(snap):
    for base, size, off in snap.ranges:
        snap.f.seek(off)
        data = snap.f.read(size)
        at = 0
        needle = struct.pack("<I", VFTABLE)
        while True:
            at = data.find(needle, at)
            if at < 0 or at + SIZE > len(data):
                break
            if at % 4 == 0:
                yield base + at, data[at : at + SIZE]
            at += 4


def decode(b):
    i = lambda o: struct.unpack_from("<i", b, o)[0]  # noqa: E731
    return dict(
        flags=b[4], rolling=struct.unpack_from("<b", b, 5)[0], accuracy=struct.unpack_from("<h", b, 6)[0],
        gpiece=i(0x8), s=(i(0xC), i(0x10), i(0x14)), e=(i(0x18), i(0x1C), i(0x20)),
        cur=i(0x24), total=i(0x28), angle=i(0x2C), splash=i(0x30), index=i(0x34),
        who=i(0x3C), o=i(0x40), num_guys=i(0x44), whom=i(0x48), ox=i(0x4C),
    )


def main():
    snap = Snapshot(sys.argv[1])
    print(f"packet frame {snap.frame}, trace frame {snap.trace_frame}")
    for va, b in rounds(snap):
        d = decode(b)
        if not d["flags"] & 3 and "--all" not in sys.argv:
            continue
        print(f"{va:08x} slot {d['index']} {d['who']}/{d['o']} -> {d['whom']}/{d['ox']} "
              f"from {d['s']} to {d['e']} t {d['cur']}/{d['total']} flags {d['flags']:#x} "
              f"rolling {d['rolling']} acc {d['accuracy']} angle {d['angle']} splash {d['splash']} "
              f"guys {d['num_guys']} gpiece {d['gpiece']}")


if __name__ == "__main__":
    main()
