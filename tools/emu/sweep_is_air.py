#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_is_air.py — `is_air@0046f000` under unicorn.

    uv run tools/emu/sweep_is_air.py <install>/riseofnations.exe

`cdecl` with the order index in `ecx` (the decompile's `in_ECX`), answering 1 for the air family (0x10, 0x11, 0x18), else 0.
One row per input, `<kind> -> <result>`: every index 0..=40 and the 8-bit edges.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046f000


def main(argv):
    m = Machine(Image(argv[1]))
    for k in list(range(0, 41)) + [0x7F, 0x80, 0xFF]:
        print(f"{k} -> {signed(m.call(ENTRY, (), ecx=k))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
