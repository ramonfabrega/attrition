#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_set_tocean.py — `World::set_tocean@006b1c60` under unicorn (item 1976, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_world_set_tocean.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the world: the width int at `+0x18`, the u16 cell array pointer at `+0x138`; stack arguments two TCoord pointers (x, y) and a third int it never reads.
Cell `y*width + x` is and-ed with `0xffef` and or-ed with `0x20`; nothing else of the array moves.
Row: `<width> <x> <y> <cell before> -> <cell after>`; the script asserts every other byte of the array is as it was.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x006b1c60
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def pop_ok(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(1976)
    for i in range(300):
        w = rng.randint(1, 24)
        x, y = rng.randrange(w), rng.randrange(24)
        this, arr, xy = h.alloc(0x200), h.alloc(2 * 24 * 24), h.alloc(8)
        h.w32(this + 0x18, w)
        h.w32(this + 0x138, arr)
        h.w32(xy, x)
        h.w32(xy + 4, y)
        grid = bytes(rng.randrange(256) for _ in range(2 * 24 * 24))
        uc.mem_write(arr, grid)
        h.call(ENTRY, (xy, xy + 4, rand32(rng)), ecx=this)
        pop_ok(uc)
        after = bytes(uc.mem_read(arr, len(grid)))
        o = 2 * (y * w + x)
        assert after[:o] == grid[:o] and after[o + 2:] == grid[o + 2:], f"row {i}: more than the one cell moved"
        print(f"{w} {x} {y} {struct.unpack('<H', grid[o:o + 2])[0]} -> {struct.unpack('<H', after[o:o + 2])[0]}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
