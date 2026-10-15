#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_air_alt_speed.py — `Unit::air_alt_speed@005ea360` under unicorn (item 2513; hand-written, parked in the Loop).

    uv run tools/emu/sweep_unit_air_alt_speed.py <install>/riseofnations.exe

`ecx` = a word whose sign bit is compared with `xmm1 < 0`, the float in `xmm1`, a plain `ret`: when the two agree it
returns `(int)(xmm1 * 25.0f / 20.0f)` (`cvttss2si`), else 0. One row per input, `<ecx> <bits> -> <eax>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, STACK_BASE, STACK_SIZE, signed  # noqa: E402
from sweeplib import setup, xmm_bits  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x005EA360
EDGES = (0, 1, -1, 0x7FFFFFFF, -0x80000000, 0x3F800000, 0xBF800000, 0x80000000, 0x7F800000, 0xFF800000, 0x7FC00000,
         0x4F000000, 0xCF000000, 0x41A00000, 0xC1A00000)


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    setup(uc)
    rng = random.Random(2513)
    bits = [(rng.choice((0, 5, -5, 1 << 31, -(1 << 31))), b) for b in EDGES for _ in range(2)]
    bits += [(rng.randint(-0x80000000, 0x7FFFFFFF), rng.randint(0, 0xFFFFFFFF)) for _ in range(200)]
    for e, b in bits:
        r = m.call(ENTRY, (), ecx=e & 0xFFFFFFFF, xmm={1: xmm_bits(b & 0xFFFFFFFF, raw=True)})
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        print(f"{e} {b} -> {signed(r)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
