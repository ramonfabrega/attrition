#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_greatest_common_factor.py — `greatest_common_factor@0092cee0` under unicorn (item 1993, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_greatest_common_factor.py <install>/riseofnations.exe

`__cdecl`-looking, but the arguments ride `ecx` and `edx` (the decompile's `in_ECX`, `in_EDX`); nothing on the stack, plain `ret`.
Counts down from `min(ecx, edx)` (signed) to the first value >= 2 that divides both; answers 1 when it reaches 1.
Row: `<ecx> <edx> -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0092cee0
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def pop_ok(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def main(argv):
    from callfn import Image, Machine, signed
    m = Machine(Image(argv[1]))
    rng = random.Random(1993)
    rows = [(0, 0), (1, 1), (2, 2), (12, 18), (-4, 8), (8, -4), (7, 13), (100, 75), (0x7FFFFFFF, 0), (0, 0x7FFFFFFF),
            (-0x80000000, 5), (5, -0x80000000), (-1, -1), (2, 4), (4, 2), (3, 0), (0, 3)]
    for _ in range(400):
        g = rng.randint(1, 60)
        a, b = g * rng.randint(1, 60), g * rng.randint(1, 60)
        if rng.random() < 0.3:
            a = rng.randint(-50, 3000)
        rows.append((a, b))
    for a, b in rows:
        r = signed(m.call(ENTRY, (), ecx=a, edx=b))
        esp = m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        print(f"{a} {b} -> {r}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
