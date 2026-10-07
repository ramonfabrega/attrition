#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_reversing.py — `reversing@0092cf20` under unicorn (item 1575's sweep).

    uv run tools/emu/sweep_reversing.py <install>/riseofnations.exe

`__cdecl`-looking in the decompile, but the listing reads only `ecx` (the
wrapped angular difference) and ends in a plain `ret`. One row per input,
`<angle as i32> -> <0|1>`: the edges the two compares name (0x40000000 and
0xc0000000, both neighbours) and a seeded random set over the whole u32 range.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine  # noqa: E402

ENTRY = 0x0092CF20


def signed(v):
    v &= 0xFFFF_FFFF
    return v - (1 << 32) if v & 0x8000_0000 else v


def inputs(seed=1575, n=400):
    edges = set()
    for c in (0, 0x4000_0000, 0xC000_0000, 0x8000_0000, 0xFFFF_FFFF, 0x7FFF_FFFF):
        for d in (-2, -1, 0, 1, 2):
            edges.add((c + d) & 0xFFFF_FFFF)
    out = sorted(edges)
    rng = random.Random(seed)
    out += [rng.getrandbits(32) for _ in range(n)]
    # a band around each bound too, where random u32 rarely lands
    for c in (0x4000_0000, 0xC000_0000):
        out += [(c + rng.randint(-0x10000, 0x10000)) & 0xFFFF_FFFF for _ in range(50)]
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    for v in inputs():
        print(f"{signed(v)} -> {m.call(ENTRY, (), ecx=v)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
