#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_set_down.py — `World::set_down@0046ef50` under unicorn (item 1971, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_world_set_down.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the world: the width int at `+0`, the cell array pointer at `+0x134`; stack arguments two WCoord pointers (x, y) and two ints.
Cell `(y*width + x)` of 0x1c bytes gets the low 16 bits of the third argument at `+8` and of the fourth at `+10`; nothing else of the array moves.
Row: `<width> <x> <y> <p3> <p4> -> <u32 at cell+8>`; the script asserts every other byte of the array is as it was.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0046ef50
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def pop_ok(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def main(argv):
    from callfn import Image, Machine
    from sweeplib import ARG, OBJ, setup
    m = Machine(Image(argv[1]))
    uc = m.uc
    setup(uc)
    rng = random.Random(1971)
    base = OBJ + 0x1000
    size = 0x1c * 24 * 24
    for i in range(300):
        w = rng.randint(1, 24)
        x, y = rng.randrange(w), rng.randrange(24)
        p3, p4 = rand32(rng), rand32(rng)
        uc.mem_write(OBJ, struct.pack("<I", w))
        uc.mem_write(OBJ + 0x134, struct.pack("<I", base))
        uc.mem_write(ARG, struct.pack("<ii", x, y))
        grid = bytes(rng.randrange(256) for _ in range(size))
        uc.mem_write(base, grid)
        m.call(ENTRY, (ARG, ARG + 4, p3, p4), ecx=OBJ)
        pop_ok(uc)
        after = bytes(uc.mem_read(base, size))
        cell = (y * w + x) * 0x1c
        want = bytearray(grid)
        want[cell + 8:cell + 10] = struct.pack("<H", p3 & 0xFFFF)
        want[cell + 10:cell + 12] = struct.pack("<H", p4 & 0xFFFF)
        assert after == bytes(want), f"row {i}: more than the two words moved"
        print(f"{w} {x} {y} {p3} {p4} -> {struct.unpack('<I', after[cell + 8:cell + 12])[0]}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
