#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_angle_diff.py — `angle_diff@0092d0b0` under unicorn (item 1814).

    uv run tools/emu/sweep_angle_diff.py <install>/riseofnations.exe

`ecx`/`edx` arguments, no frame — `a - b` wrapping, complemented above 0x80000000; the edge words crossed and four hundred random pairs, against that.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0092d0b0
OBJ, ARGS = 0x10000000, 0x10100000
EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def words(seed=1814, n=150):
    rng = random.Random(seed)
    return list(EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(n)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARGS, 0x1000)
    rng = random.Random(1814)
    ws = [w & 0xFFFFFFFF for w in EDGES] + [0x80000000, 0x80000001, 0x7FFFFFFF]
    rows = [(a, b) for a in ws for b in ws]
    rows += [(rng.getrandbits(32), rng.getrandbits(32)) for _ in range(400)]
    for a, b in rows:
        r = m.call(ENTRY, (), ecx=a, edx=b)
        print(f"{a} {b} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
