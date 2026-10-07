#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_find_angle.py — `find_angle@0092d130` under unicorn.

    uv run tools/emu/sweep_find_angle.py <install>/riseofnations.exe

A free function on a register pair (the listing: `ecx` = dx, `edx` = dy, a
plain `ret` with nothing popped; the decompile's two stack parameters do not
exist). It negates dy itself. Prints one row per input,
`<dx> <dy> -> <eax, as a signed 32-bit>`.

The domain is |dx|, |dy| <= 131071 (2^17 - 1): the original scales the shorter
leg by 0x4000 into a signed 32-bit and `idiv`s, so beyond that the product
wraps and at INT_MIN the `idiv` or the negation traps. A game's coordinates
(map units, a few thousand) never reach it. Nothing is stood in for: the
function reads no memory.

Inputs: the axes and their neighbours, every octant's diagonal and the
ratio boundaries round 0x1333/0x4000 (the correction's fold), a grid, and a
seeded (1619) random draw at several magnitudes.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine  # noqa: E402

ENTRY = 0x0092D130
LIM = 131071


def inputs(seed=1619):
    out = []
    vals = [0, 1, -1, 2, -2, 3, -3, 7, -7, 100, -100, 4095, -4095, 4096, -4096,
            LIM, -LIM, LIM - 1, -LIM + 1, 65536, -65536]
    for a in vals:
        for b in vals:
            out.append((a, b))
    # ratio boundaries: lo/hi round 0x1333/0x4000 (~0.2998) and 1, and the
    # exact diagonal; all eight sign/dominance combinations
    for hi in (1000, 4096, 16384, 100000, LIM):
        for num in range(0, 0x4000 + 1, 0x4000 // 64):
            lo = hi * num // 0x4000
            fold = hi * 0x1333 // 0x4000
            for lo2 in {lo - 1, lo, lo + 1, hi - 1, hi, fold - 1, fold, fold + 1}:
                if not 0 <= lo2 <= LIM:
                    continue
                for sx in (1, -1):
                    for sy in (1, -1):
                        out.append((sx * hi, sy * lo2))
                        out.append((sx * lo2, sy * hi))
    rng = random.Random(seed)
    for bits in (4, 8, 12, 16, 17):
        m = min((1 << bits) - 1, LIM)
        for _ in range(150):
            out.append((rng.randint(-m, m), rng.randint(-m, m)))
    seen, uniq = set(), []
    for p in out:
        if p not in seen:
            seen.add(p)
            uniq.append(p)
    return uniq


def main(argv):
    m = Machine(Image(argv[1]))
    for dx, dy in inputs():
        r = m.call(ENTRY, (), ecx=dx, edx=dy)
        r = r - (1 << 32) if r >= 1 << 31 else r
        print(f"{dx} {dy} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
