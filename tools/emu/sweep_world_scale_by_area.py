#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_scale_by_area.py — `World::scale_by_area@006b2b10` under unicorn (item 1976, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_world_scale_by_area.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the world: an int at `+8`; one stack argument. The divisor is the int at `+0x144` of the record the global cell `0xe7fca8` points at, squared (32-bit).
Answers `(sq/2 + field*p0) / sq` (C division, 32-bit wrapping product), at least 1.
Row: `<field> <p0> <div> -> <eax>`; a divisor whose square wraps to zero is not drawn.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x006b2b10
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
        this, ms = h.alloc(0x40), h.alloc(0x200)
        f, p0 = rand32(rng), rand32(rng)
        if i % 2:
            f, p0 = rng.randint(-3000, 3000), rng.randint(-3000, 3000)
        d = rng.choice((1, 2, 3, 7, 64, 128, 512, -2, -7, 100, 46341, 65535)) if i % 2 else rng.randint(-1000, 1000)
        if (d * d) & 0xFFFFFFFF == 0:
            d = 5
        uc.mem_write(0x00E7FCA8, struct.pack("<I", ms))
        h.w32(this + 8, f)
        h.w32(ms + 0x144, d)
        r = h.call(ENTRY, (p0,), ecx=this)
        pop_ok(uc)
        print(f"{f} {p0} {d} -> {r}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
