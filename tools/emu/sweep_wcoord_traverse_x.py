#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_wcoord_traverse_x.py — `WCoord::traverse_x@0046eef0` under unicorn (item 1971, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_wcoord_traverse_x.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the WCoord (an int at `+0`); stack arguments the out-pointer and an int `p`; stores
`value*4 + (p % 4)` (C remainder: `p & 3`, or `-((-p) & 3)` for a negative `p`) through the out-pointer and answers it.
Row: `<value> <p> -> <word stored>`; the script asserts `eax` is the out-pointer.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0046eef0
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def pop_ok(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def main(argv):
    from callfn import Image, Machine
    from sweeplib import OBJ, OUT, setup
    m = Machine(Image(argv[1]))
    uc = m.uc
    setup(uc)
    rng = random.Random(1971)
    for i in range(400):
        v = rand32(rng)
        p = rand32(rng) if i % 3 else rng.randint(-9, 9)
        uc.mem_write(OBJ, struct.pack("<i", v))
        uc.mem_write(OUT, struct.pack("<I", 0x55AA55AA))
        r = m.call(ENTRY, (OUT, p), ecx=OBJ)
        pop_ok(uc)
        assert r & 0xFFFFFFFF == OUT, "eax is not the out-pointer"
        assert bytes(uc.mem_read(OBJ, 4)) == struct.pack("<i", v), "the coord moved"
        print(f"{v} {p} -> {struct.unpack('<i', uc.mem_read(OUT, 4))[0]}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
