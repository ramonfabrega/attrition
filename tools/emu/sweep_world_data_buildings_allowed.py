#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_data_buildings_allowed.py — `WorldData::buildings_allowed@006b2340` under unicorn (item 1993, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_world_data_buildings_allowed.py <install>/riseofnations.exe

`__thiscall` (`ecx` unread); stack arguments two WCoord pointers (x, y). The world is the pointer in the global cell `0xc06188`: width int at `+0`, cell array pointer at `+0x134`.
Answers 1 when the 16-bit word at the start of cell `(y*width + x)` of 0x1c bytes has none of `0x78` set.
Row: `<width> <x> <y> <cell word> -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x006b2340
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
        uc.mem_write(0x00C06188, struct.pack("<I", world))
        h.w32(world, w)
        h.w32(world + 0x134, arr)
        h.w32(xy, x)
        h.w32(xy + 4, y)
        uc.mem_write(arr, bytes(rng.randrange(256) for _ in range(0x1c * 24 * 24)))
        v = rng.choice((0, 0x78, 0x08, 0x10, 0x20, 0x40, 0xFF87, 0x87, 0x7F)) if i % 3 else rng.randrange(0x10000)
        h.w16(arr + 0x1c * (y * w + x), v if v < 0x8000 else v - 0x10000)
        r = h.call(ENTRY, (xy, xy + 4), ecx=rng.randrange(1 << 32))
        pop_ok(uc)
        print(f"{w} {x} {y} {v} -> {r}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
