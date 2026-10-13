#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_guy_set_elev_angle.py — `Guy::set_elev_angle@005d8fb0` under unicorn (item 1921).

    uv run tools/emu/sweep_guy_set_elev_angle.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object, the angle a float in `xmm1` (the decompile's
`in_XMM1_Da`), one unread stack argument (`ret 4`). The function stores the float's
bits at `this+0x40`. One row per input, `<bits> -> <word at this+0x40>`; the
script asserts `esp` is popped by 4 and every other byte of the object is untouched.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, STACK_BASE, STACK_SIZE, signed  # noqa: E402
from sweeplib import OBJ, setup, xmm_bits  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x005D8FB0
EDGES = (0, 1, -1, 0x3F800000, -0x40800000, 0x7F800000, -0x800000, 0x7FC00000, 0x00800000, 0x7FFFFFFF, -0x80000000)


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    setup(uc)
    rng = random.Random(1921)
    bits = list(EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]
    fill = bytes(rng.randrange(256) for _ in range(0x100))
    for b in bits:
        uc.mem_write(OBJ, fill)
        m.call(ENTRY, (rng.randint(-0x80000000, 0x7FFFFFFF),), ecx=OBJ, xmm={1: xmm_bits(b & 0xFFFFFFFF, raw=True)})
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        after = bytes(uc.mem_read(OBJ, 0x100))
        assert after[:0x40] == fill[:0x40] and after[0x44:] == fill[0x44:], "touched another byte"
        print(f"{b} -> {signed(struct.unpack('<I', after[0x40:0x44])[0])}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
