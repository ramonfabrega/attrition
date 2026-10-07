#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_farms_snip.py — `Farms::snip@008d9240` under unicorn.

    uv run tools/emu/sweep_farms_snip.py <install>/riseofnations.exe

`__thiscall`, `this` unused, three stack args `(farm, x, y)`, `ret 0xc`; void.
The farms array is reached through the dword at 0xc0a914 (`farms._100_4_`);
the byte it tests and writes is `base + (farm*0x30 + y)*4 + 0xac + x`, i.e.
farm records 0xc0 bytes apart with a 4x4 status grid at +0xac, row `y`,
column `x`. The two guards `x < 4`, `y < 4` are SIGNED (`jge`), so negatives
pass and index backwards.

Four farm records are laid out in a mapped page, each grid filled with a seeded
mix of states 0..5 (2 over-weighted: it is the only state that changes). Every
byte of the table outside the four grids is 0x7f (stood in for: never 2, so a
backward write that lands there is a no-op; the `which=9` row asserts that
nothing outside the grids changed). Page padding of 0x1000 either side keeps
negative indexes mapped.

One input yields five rows, `<farm> <x> <y> <g0> <g1> <g2> <g3> <which> -> <r>`:

    g0..g3   each farm's grid BEFORE the call, packed base 6, cell `y*4+x`
             least significant first (cell c contributes state * 6**c)
    which    0..3  r = that farm's grid AFTER the call, packed the same way
             9     r = how many bytes outside the four grids changed (stood-in
                   bytes are 0x7f, so always 0)
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine  # noqa: E402

ENTRY = 0x008D9240
G_FARMS = 0x00C0A914
LO, SIZE = 0x10000000, 0x4000
BASE = 0x10002000          # farms._100_4_
STRIDE, GRID, NF = 0xC0, 0xAC, 4


def pack(cells):
    return sum(c * 6**i for i, c in enumerate(cells))


def inputs():
    out = []
    # the whole -1..5 square on every farm
    for f in range(NF):
        for y in range(-1, 6):
            for x in range(-1, 6):
                out.append((f, x, y))
    return out


def more(rng):
    out = []
    # in range, random
    for _ in range(80):
        out.append((rng.randrange(NF), rng.randrange(4), rng.randrange(4)))
    # negative columns alias into the previous row / before the grid
    for _ in range(60):
        out.append((rng.randrange(NF), rng.randint(-20, -1), rng.randrange(4)))
    # negative rows: y = -48 + k lands in the previous farm's row k
    for f in range(NF):
        for y in range(-50, -45):
            for x in range(4):
                out.append((f, x, y))
    # boundaries of the signed compare and large values (rejected)
    for f in (0, 3):
        for x, y in ((4, 0), (0, 4), (0x7FFFFFFF, 0), (0, 0x7FFFFFFF), (100, 100)):
            out.append((f, x, y))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    m.uc.mem_map(LO, SIZE)
    m.uc.mem_write(G_FARMS, struct.pack("<I", BASE))
    rng = random.Random(1619)
    grid_ix = {BASE - LO + k * STRIDE + GRID + c for k in range(NF) for c in range(16)}
    for f, x, y in inputs() + more(rng):
        m.uc.mem_write(LO, bytes([0x7F]) * SIZE)
        grids = []
        for k in range(NF):
            cells = [rng.choice((0, 1, 2, 2, 2, 3, 4, 5)) for _ in range(16)]
            grids.append(cells)
            m.uc.mem_write(BASE + k * STRIDE + GRID, bytes(cells))
        before = bytes(m.uc.mem_read(LO, SIZE))
        m.call(ENTRY, (f, x, y), ecx=0x20000000)
        after = bytes(m.uc.mem_read(LO, SIZE))
        head = f"{f} {x} {y} " + " ".join(str(pack(g)) for g in grids)
        for k in range(NF):
            a = BASE + k * STRIDE + GRID - LO
            print(f"{head} {k} -> {pack(list(after[a:a + 16]))}")
        diff = [i for i in range(SIZE) if before[i] != after[i]]
        print(f"{head} 9 -> {sum(1 for i in diff if i not in grid_ix)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
