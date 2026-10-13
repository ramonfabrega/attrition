#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_type_data_get_city_level.py — `BuildTypeData::get_city_level@00639800` under unicorn (item 1971, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_build_type_data_get_city_level.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the type record; reads the index at `+0x4`: `0x19f` answers 2, `0x1a0` and `0x213` answer 3, else 1.
Row: `<index> -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00639800
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def pop_ok(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def main(argv):
    from callfn import Image, Machine
    from sweeplib import OBJ, setup
    m = Machine(Image(argv[1]))
    uc = m.uc
    setup(uc)
    rng = random.Random(1971)
    vals = list(range(-2, 0x340)) + [0x7FFFFFFF, -0x80000000]
    vals += [rand32(rng) for _ in range(100)]
    for v in vals:
        uc.mem_write(OBJ + 4, struct.pack("<i", v))
        r = m.call(ENTRY, (), ecx=OBJ)
        pop_ok(uc)
        print(f"{v} -> {r}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
