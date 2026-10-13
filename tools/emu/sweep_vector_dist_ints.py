#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_vector_dist_ints.py — `vector_dist@0046f020` under unicorn (item 1998, hand-written on tools/emu/callfn.py).

    uv run tools/emu/sweep_vector_dist_ints.py <install>/riseofnations.exe

The four-int overload: `x0` in `ecx`, `y0` in `edx`, `x1`, `y1` on the stack (`__cdecl`, the caller pops). It
takes `|x0 - x1|` and `|y0 - y1|` and jumps into `vector_dist@0046cff0` (`ecx`, `edx`), which runs unhooked:
the whole octagonal distance is the original's. The rows keep each difference inside ±2^30 (game coordinates
are far smaller), so neither side's `abs` nor `2 * hi` wraps; the edges sit at the 60,000 branch.
Row: `<x0> <y0> <x1> <y1> -> <eax>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE, Image, Machine, signed  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0046F020
EDGES = (0, 1, -1, 2, 59_999, 60_000, 60_001, -60_000, 0x100, 0x3000, 0x7FFF, 0x1FFFFFFF, -0x1FFFFFFF)


def rows():
    rng = random.Random(1998)
    out = [(a, b, c, d) for a in (0, 60_000, -1) for b in (0, 1, 59_999) for c in (0, 60_001) for d in (0, -2)]
    for e in EDGES:
        out.append((e, 0, 0, 0))
        out.append((0, e, 0, 0))
        out.append((e, e, 0, 0))
    for i in range(400):
        span = rng.choice((0x100, 0x3000, 60_000, 200_000, 0x1FFFFFFF))
        out.append(tuple(rng.randint(-span // 2, span // 2) for _ in range(4)))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    for x0, y0, x1, y1 in rows():
        r = signed(m.call(ENTRY, (x1, y1), ecx=x0, edx=y0))
        esp = m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100 - 8, f"esp {esp:x}: a cdecl return pops nothing"
        print(f"{x0} {y0} {x1} {y1} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
