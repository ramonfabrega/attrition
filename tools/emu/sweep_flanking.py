#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_flanking.py — `flanking@0092cfe0` under unicorn.

    uv run tools/emu/sweep_flanking.py <install>/riseofnations.exe

The bias `e` rides in `ecx`; a plain `ret`, so nothing on the stack. Prints one
row per input, `<e, unsigned> -> <eax>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine  # noqa: E402

ENTRY = 0x0092CFE0


def inputs(seed=1575, n=400):
    edges = [0, 1, 0x2AAAAAAA - 1, 0x2AAAAAAA, 0x2AAAAAAA + 1,
             0x5FFFFFFF, 0x60000000, 0x60000001,
             0x7FFFFFFF, 0x80000000, 0x80000001,
             0xA0000000, 0xA0000001, 0xA0000000 - 1,
             0xD5555554, 0xD5555555, 0xD5555556, 0xD5555557,
             0xFFFFFFFE, 0xFFFFFFFF]
    # edges of the 1/2 split: e - 0x60000000 vs 0x40000000, i.e. e = 0xA0000000
    # and its neighbours, plus the wrap at 0x60000000.
    rng = random.Random(seed)
    out = list(edges)
    for _ in range(n):
        out.append(rng.getrandbits(32))
    # a band round each boundary
    for c in (0x2AAAAAAA, 0x60000000, 0xA0000000, 0xD5555555):
        for d in range(-8, 9):
            out.append((c + d) & 0xFFFFFFFF)
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    for e in inputs():
        print(f"{e} -> {m.call(ENTRY, (), ecx=e)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
