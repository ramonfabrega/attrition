#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_type_corner_tile.py — `BuildTypeData::corner_tile@006364c0`
under unicorn.

    uv run tools/emu/sweep_build_type_corner_tile.py <install>/riseofnations.exe

`__thiscall`, `ret 0x10`: this in ecx, then `TCoord x`, `TCoord y`,
`Coord *out_x`, `Coord *out_y` on the stack. It reads `this+0x234` (x_size)
and `this+0x238` (y_size) and writes `(size + 2*corner) * 0x60` through each
out pointer. One input prints two rows, axis 0 (x) and axis 1 (y):

    <x_size> <y_size> <corner_x> <corner_y> <axis> -> <value>
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x006364C0
THIS, OUT = 0x1000_0000, 0x1000_0800


def main(argv):
    m = Machine(Image(argv[1]))
    m.uc.mem_map(THIS, 0x1000)
    rng = random.Random(1575)
    edges = [-1, 0, 1, 2, 3, 4, 5, 7, 8, 100, 255, 256, 1000, -1000]
    cases = [(a, b, c, d) for a in (0, 1, 2, 4, 7) for b in (0, 3, 5) for c in edges[:9] for d in edges[:9]]
    for _ in range(600):
        lim = rng.choice((16, 256, 5000, 1_000_000))
        cases.append((rng.randint(-lim, lim), rng.randint(-lim, lim),
                      rng.randint(-lim, lim), rng.randint(-lim, lim)))
    for xs, ys, cx, cy in cases:
        m.uc.mem_write(THIS + 0x234, (xs & 0xFFFFFFFF).to_bytes(4, "little"))
        m.uc.mem_write(THIS + 0x238, (ys & 0xFFFFFFFF).to_bytes(4, "little"))
        m.uc.mem_write(OUT, bytes(8))
        m.call(ENTRY, (cx, cy, OUT, OUT + 4), ecx=THIS)
        ox = signed(int.from_bytes(m.uc.mem_read(OUT, 4), "little"))
        oy = signed(int.from_bytes(m.uc.mem_read(OUT + 4, 4), "little"))
        print(f"{xs} {ys} {cx} {cy} 0 -> {ox}")
        print(f"{xs} {ys} {cx} {cy} 1 -> {oy}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
