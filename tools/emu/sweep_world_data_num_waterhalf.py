#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_data_num_waterhalf.py — `WorldData::num_waterhalf@006b4db0` under unicorn (item 2061, hand-written
on tools/emu/hooks.py).

    uv run tools/emu/sweep_world_data_num_waterhalf.py <install>/riseofnations.exe

`__thiscall`, two arguments (pointers to the cell's `x` and `y`), `ret 8`, `this` unread. The world is the
record in the cell `0xc06188`: `+0` its width in cells, `+0x18` the tile row stride, `+0x134` the cell array
(`0x1c` a cell, the flags word first), `+0x138` the tile masks (a short a tile). A cell without `0x100` answers
0; else the count of its sixteen tiles (`x·4 + i&3`, `y·4 + i>>2`) whose mask `& 0x30` is `0x20`.
A row's world is random throughout — every cell, every tile — and prints only the cell and the sixteen tiles
the port reads, so a read anywhere else parts.
Row: `<width> <height> <x> <y> <flags> <mask0> … <mask15> -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x006B4DB0
WORLD = 0x00C06188


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2061)
    world = h.alloc(0x200)
    cells = h.alloc(8 * 8 * 0x1C)
    tiles = h.alloc(32 * 32 * 2)
    px, py = h.alloc(4), h.alloc(4)
    this = h.alloc(0x10)
    h.w32(WORLD, world)
    for _ in range(400):
        w, ht = rng.randint(1, 8), rng.randint(1, 8)
        uc.mem_write(world, bytes(rng.randrange(256) for _ in range(0x200)))
        h.w32(world, w)
        h.w32(world + 0x18, w * 4)
        h.w32(world + 0x134, cells)
        h.w32(world + 0x138, tiles)
        uc.mem_write(cells, bytes(rng.randrange(256) for _ in range(8 * 8 * 0x1C)))
        uc.mem_write(tiles, b"".join(struct.pack("<H", rng.getrandbits(16) if rng.random() < 0.5
                                                 else (rng.getrandbits(16) & ~0x30) | rng.choice((0, 0x10, 0x20, 0x30)))
                                     for _ in range(32 * 32)))
        x, y = rng.randrange(w), rng.randrange(ht)
        c = cells + (y * w + x) * 0x1C
        flags = struct.unpack("<H", uc.mem_read(c, 2))[0]
        if rng.random() < 0.7:
            flags |= 0x100
        h.w16(c, flags - (1 << 16) if flags & 0x8000 else flags)
        masks = [struct.unpack("<H", uc.mem_read(tiles + 2 * ((y * 4 + i // 4) * w * 4 + x * 4 + i % 4), 2))[0]
                 for i in range(16)]
        h.w32(px, x)
        h.w32(py, y)
        before = bytes(uc.mem_read(cells, 8 * 8 * 0x1C)), bytes(uc.mem_read(tiles, 32 * 32 * 2))
        res = h.call(ENTRY, (px, py), ecx=this)
        assert uc.reg_read(UC_X86_REG_ESP) == STACK_BASE + STACK_SIZE - 0x100, "wrong pop"
        assert (bytes(uc.mem_read(cells, 8 * 8 * 0x1C)), bytes(uc.mem_read(tiles, 32 * 32 * 2))) == before, "wrote"
        assert (h.r32(px), h.r32(py)) == (x, y), "wrote an argument"
        print(f"{w} {ht} {x} {y} {flags} " + " ".join(map(str, masks)) + f" -> {res}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
