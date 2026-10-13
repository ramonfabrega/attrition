#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_scale_by_axis.py — `World::scale_by_axis@006b76b0` under unicorn (item 1976, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_world_scale_by_axis.py <install>/riseofnations.exe

`__thiscall`, `ecx` is the value itself (an int, not a pointer); no stack argument. The world width is the int at `+0` of the pointer in the global cell `0xc06188`; the divisor is the int at `+0x144` of the record the cell `0xe7fca8` points at.
Answers `(div/2 + width*ecx) / div` (C division, 32-bit wrapping product), at least 1.
Row: `<ecx> <width> <div> -> <eax>`; a zero divisor, and `-1` where the quotient would overflow, are not drawn.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x006b76b0
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
    for i in range(400):
        world, ms = h.alloc(0x40), h.alloc(0x200)
        v, w = rand32(rng), rand32(rng)
        if i % 2:
            v, w = rng.randint(-3000, 3000), rng.randint(1, 600)
        d = rng.choice((1, 2, 3, 7, 64, 128, 512, -2, -7, 100)) if i % 2 else rng.randint(-1000, 1000)
        if d in (0, -1):
            d = 5
        uc.mem_write(0x00C06188, struct.pack("<I", world))
        uc.mem_write(0x00E7FCA8, struct.pack("<I", ms))
        h.w32(world, w)
        h.w32(ms + 0x144, d)
        r = h.call(ENTRY, (), ecx=v & 0xFFFFFFFF)
        print(f"{v} {w} {d} -> {r}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
