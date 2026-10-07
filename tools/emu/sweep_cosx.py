#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_cosx.py — `cosx@0092d0c0` under unicorn, one row per input.

    uv run tools/emu/sweep_cosx.py <install>/riseofnations.exe

`cosx(angle, distance)` takes both in registers (`ecx` the binary angle,
`edx` the distance; no stack argument, plain `ret`), adds a quarter turn
(`+0x40000000`), folds it and tail-jumps into `sin_table@00a46a00`. It reads
the executable's own `sine_table` (`.bss` at 0xe32f40), which the image
does not hold: `trig_init@00a46980` builds it at startup, so the script runs
that first under the same machine — the original's own initializer, its
`sin` included — and nothing is synthesized. Prints `<angle> <distance> -> <result>`, signed integers.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine  # noqa: E402

COSX = 0x0092D0C0
TRIG_INIT = 0x00A46980


def signed(v):
    return v - (1 << 32) if v & 0x8000_0000 else v


def calls(seed=1575, n=600):
    rng = random.Random(seed)
    angles = [0, 1, 0x3FFFFF, 0x400000, 0x3FFFFFFF, 0x40000000, 0x40000001, 0x55555555,
              0x7FFFFFFF, -0x80000000, -0x7FFFFFFF, -1, -0x40000000, -0x40000001, -0x3FFFFFFF,
              -0x41000000, 0x3FC00000, 0x3FBFFFFF, 0x3F800000, 0x7FC00000, -0x00400000,
              0x20000000, -0x20000000, 0x60000000, -0x60000000,
              0xB60B60, 0xB60B60 * 90, 0xB60B60 * 180, -0xB60B60 * 90]
    dists = [0, 1, -1, 2, 5, 100, -100, 0xFFFE, 0xFFFF, 0x10000, -0xFFFE, -0xFFFF, -0x10000,
             0xFFFFFD, 0xFFFFFE, 0xFFFFFF, 0x1000000, -0xFFFFFE, -0xFFFFFF, -0x1000000,
             0x7FFFFFFF, -0x7FFFFFFF, -0x80000000, 0x12345678]
    for a in angles:
        for d in dists:
            yield a, d
    for _ in range(n):
        r = rng.random()
        a = rng.randint(-(1 << 31), (1 << 31) - 1)
        lim = 3_000 if r < 0.4 else 0x20000 if r < 0.7 else 0x2000000 if r < 0.9 else (1 << 31) - 1
        yield a, rng.randint(-lim, lim)


def main(argv):
    m = Machine(Image(argv[1]))
    m.call(TRIG_INIT, max_insns=5_000_000)
    for a, d in calls():
        print(f"{a} {d} -> {signed(m.call(COSX, (), ecx=a, edx=d))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
