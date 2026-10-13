#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_region_release.py — `Region::release@00680560` under unicorn (item 2044, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_region_release.py <install>/riseofnations.exe

`__thiscall`, no stack argument, `ret`. For each of the region's `size` (`+0x14`, signed) coordinates — pairs of
words `x`, `y` in the list `+0x7c` names — it zeroes the short at `+4` of the world tile `x + width*y` (the
world through the cell `0xc06188`: width at `+0`, tiles of `0x1c` bytes at `[+0x134]`); then `size` and the
word at `+0x70` are 0. A row draws a small world, `size` -2..24 and coordinates inside it, repeats allowed.
Row: `<width> <size> <x,y;…> | <this hex> <tiles hex> -> <eax> | <this hex> <tiles hex>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402


def popped(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


ENTRY = 0x00680560
WORLD = 0x00C06188
H = 8
WIN = 0x80


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2044)
    saved = bytes(uc.mem_read(WORLD, 4))
    world = h.alloc(0x140)
    tiles = h.alloc(8 * H * 0x1C + 0x20)
    coords = h.alloc(8 * 24)
    this = h.alloc(WIN)
    h.w32(WORLD, world)
    for _ in range(300):
        w = rng.randrange(1, 9)
        uc.mem_write(world, bytes(rng.randrange(256) for _ in range(0x140)))
        h.w32(world, w)
        h.w32(world + 0x134, tiles)
        n = w * H * 0x1C
        uc.mem_write(tiles, bytes(rng.randrange(256) for _ in range(n + 0x20)))
        size = rng.choice((-2, -1, 0)) if rng.random() < 0.15 else rng.randrange(1, 25)
        pts = [(rng.randrange(w), rng.randrange(H)) for _ in range(max(size, 0))]
        for k, (x, y) in enumerate(pts):
            h.w32(coords + 8 * k, x)
            h.w32(coords + 8 * k + 4, y)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(WIN)))
        h.w32(this + 0x14, size)
        h.w32(this + 0x7C, coords)
        before = bytes(uc.mem_read(this, WIN)).hex(), bytes(uc.mem_read(tiles, n)).hex()
        guard = bytes(uc.mem_read(tiles + n, 0x20))
        res = h.call(ENTRY, (), ecx=this)
        popped(uc)
        assert bytes(uc.mem_read(tiles + n, 0x20)) == guard, "wrote past the tiles"
        after = bytes(uc.mem_read(this, WIN)).hex(), bytes(uc.mem_read(tiles, n)).hex()
        listed = ";".join(f"{x},{y}" for x, y in pts) or "-"
        print(f"{w} {size} {listed} | {before[0]} {before[1]} -> {res} | {after[0]} {after[1]}")
    uc.mem_write(WORLD, saved)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
