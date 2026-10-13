#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_data_get_coll_block.py — `WorldData::get_coll_block@006b5350` under unicorn (item 1993, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_world_data_get_coll_block.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the world: width int at `+0`, cell array pointer at `+0x134`, cells 0x1c bytes. Stack arguments two WCoord pointers (x, y) and a region `p3`.
The cell `(y*width + x)` holds a signed 16-bit region at `+4` and a block word at `+0x18`. With `p3 >= 0` and a different region the answer is -1, else the block word.
Row: `<width> <x> <y> <p3> <cell region> <block> -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x006b5350
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
    rng = random.Random(1993)
    world, arr, xy = h.alloc(0x200), h.alloc(0x1c * 24 * 24), h.alloc(8)
    for i in range(400):
        w = rng.randint(1, 24)
        x, y = rng.randrange(w), rng.randrange(24)
        h.w32(world, w)
        h.w32(world + 0x134, arr)
        h.w32(xy, x)
        h.w32(xy + 4, y)
        uc.mem_write(arr, bytes(rng.randrange(256) for _ in range(0x1c * 24 * 24)))
        reg = rng.choice((0, 1, -1, 5, 0x7FFF, -0x8000, 3))
        p3 = rng.choice((reg, reg, -1, -2, 0, 5, 0x10000, -0x8000, 0x7FFF, 0x8000, -0x80000000, 0x7FFFFFFF, rand32(rng)))
        blk = rand32(rng)
        c = arr + 0x1c * (y * w + x)
        h.w16(c + 4, reg)
        h.w32(c + 0x18, blk)
        r = h.call(ENTRY, (xy, xy + 4, p3), ecx=world)
        pop_ok(uc)
        print(f"{w} {x} {y} {p3} {reg} {blk} -> {r}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
