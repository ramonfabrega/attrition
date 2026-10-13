#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_group_report_speed.py — `Group::report_speed@00713bb0` under unicorn (item 1932).

    uv run tools/emu/sweep_group_report_speed.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object, one stack argument (`ret 4`): the speed. When the speed is below the (signed) word at `this+0x40`, both `this+0x40` and `this+0x3c` take it; otherwise nothing moves.
One row per input, `<speed> <old 0x3c> <old 0x40> -> <new 0x3c + (new 0x40 << 32)>`; the script asserts every other byte is untouched
(the byte at 0x4b is checked to be 0 after) and `esp` pops the argument.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, STACK_BASE, STACK_SIZE, signed  # noqa: E402
from sweeplib import OBJ, ARG, setup, xmm_bits  # noqa: E402,F401
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

def signed64(v):
    return v - (1 << 64) if v >> 63 else v


ENTRY = 0x00713bb0
EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x3F800000, 0x7FC00000)


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    setup(uc)
    rng = random.Random(1932)
    rows = [(rng.choice(EDGES), rng.choice(EDGES), rng.choice(EDGES)) for _ in range(60)]
    rows += [tuple(rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(3)) for _ in range(100)]
    # speeds near the stored word, both sides of it
    for _ in range(60):
        w = rng.randint(-0x7FFFFFF0, 0x7FFFFFF0)
        rows.append((w + rng.choice((-1, 0, 1)), rng.randint(-0x80000000, 0x7FFFFFFF), w))
    for sp, o3c, o40 in rows:
        fill = bytearray(rng.randrange(256) for _ in range(0x100))
        fill[0x3c:0x40] = struct.pack("<i", o3c)
        fill[0x40:0x44] = struct.pack("<i", o40)
        uc.mem_write(OBJ, bytes(fill))
        m.call(ENTRY, (sp,), ecx=OBJ)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        after = bytes(uc.mem_read(OBJ, 0x100))
        keep = after[:0x3c] == fill[:0x3c] and after[0x44:] == fill[0x44:]
        assert keep, "touched another byte"
        n3c = struct.unpack("<I", after[0x3c:0x40])[0]
        n40 = struct.unpack("<I", after[0x40:0x44])[0]
        print(f"{sp} {o3c} {o40} -> {signed64((n40 << 32) | n3c)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
