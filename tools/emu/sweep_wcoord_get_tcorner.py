#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_wcoord_get_tcorner.py — `WCoord::get_tcorner@0046f080` under unicorn (item 1814).

    uv run tools/emu/sweep_wcoord_get_tcorner.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the `WCoord`, an out pointer on the stack: writes `value << 2` and returns the pointer; the printed result is the word written.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046f080
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
    for w in words():
        uc.mem_write(OBJ, struct.pack("<i", w))
        uc.mem_write(ARGS, struct.pack("<I", 0xDEADBEEF))
        p = m.call(ENTRY, (ARGS,), ecx=OBJ)
        assert p == ARGS, hex(p)
        out = struct.unpack("<i", bytes(uc.mem_read(ARGS, 4)))[0]
        print(f"{w} -> {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
