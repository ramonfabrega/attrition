#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_vector_dist_coords.py — `vector_dist@0046d060` under unicorn (item 1998, hand-written on tools/emu/callfn.py).

    uv run tools/emu/sweep_vector_dist_coords.py <install>/riseofnations.exe

The four-`Coord*` overload: `&x0` in `ecx`, `&y0` in `edx`, `&x1`, `&y1` on the stack (`__cdecl`, the caller
pops). It reads the four words, takes `|x0 - x1|` and `|y0 - y1|` and jumps into `vector_dist@0046cff0`, which
runs unhooked. The four words sit in a page of their own, each at its own offset, and the rows keep each
difference inside ±2^30, as `sweep_vector_dist_ints.py` does.
Row: `<x0> <y0> <x1> <y1> -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE, Image, Machine, signed  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0046D060
PAGE = 0x2000_0000
EDGES = (0, 1, -1, 2, 59_999, 60_000, 60_001, -60_000, 0x100, 0x3000, 0x7FFF, 0x1FFFFFFF, -0x1FFFFFFF)


def rows():
    rng = random.Random(19980)
    out = [(a, b, c, d) for a in (0, 60_000, -1) for b in (0, 1, 59_999) for c in (0, 60_001) for d in (0, -2)]
    for e in EDGES:
        out.append((e, 0, 0, 0))
        out.append((0, e, 0, 0))
        out.append((0, 0, e, e))
    for i in range(400):
        span = rng.choice((0x100, 0x3000, 60_000, 200_000, 0x1FFFFFFF))
        out.append(tuple(rng.randint(-span // 2, span // 2) for _ in range(4)))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    m.uc.mem_map(PAGE, 0x1000)
    px0, py0, px1, py1 = PAGE + 0x10, PAGE + 0x84, PAGE + 0x200, PAGE + 0x3F0
    for x0, y0, x1, y1 in rows():
        for p, v in ((px0, x0), (py0, y0), (px1, x1), (py1, y1)):
            m.uc.mem_write(p, struct.pack("<i", v))
        r = signed(m.call(ENTRY, (px1, py1), ecx=px0, edx=py0))
        esp = m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100 - 8, f"esp {esp:x}: a cdecl return pops nothing"
        after = struct.unpack("<4i", b"".join(bytes(m.uc.mem_read(p, 4)) for p in (px0, py0, px1, py1)))
        assert after == (x0, y0, x1, y1), "the coordinates are read, never written"
        print(f"{x0} {y0} {x1} {y1} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
