#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_type_get_good.py — `BuildTypeData::get_good@0063bd50` under unicorn.

    uv run tools/emu/sweep_build_type_get_good.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, a plain `ret`: `ecx` is the BuildTypeData and
the function reads one dword, the `TypeIndex` at `this+4`, subtracts 0x1a1 and
indexes a six-entry jump table (`0x63bd84`), anything else answering -1.
Prints one row per input, `<type_index, signed> -> <eax, signed>`:

    type_index   `this+4`, every value 0..0x2ff, then edges round the table,
                 negatives, 0x7fffffff, -0x80000000, and 200 seeded (1619)
                 32-bit values.

Stood in for: only a page for `this`; nothing else is read, so nothing else
is laid out.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0063BD50
THIS = 0x10000000


def inputs():
    out = list(range(0x300))
    out += [-1, -2, -0x1A1, -0x1A2, -0x100, -0x8000, 0x7FFFFFFF, -0x80000000, 0x80000001,
            0xFFFF, 0x10000, 0x1A1 + 0x100000000 - 0x100000000, 0x1A1 + 6, 0x1A1 - 1,
            0x100 + 0x1A1, 0xFFFFFFFF - 0x1A1, 0x7FFFFE5F, 0x80000000 + 0x1A1]
    rng = random.Random(1619)
    out += [rng.getrandbits(32) for _ in range(200)]
    return [signed(v & 0xFFFFFFFF) for v in out]


def main(argv):
    m = Machine(Image(argv[1]))
    m.uc.mem_map(THIS, 0x1000)
    for t in inputs():
        m.uc.mem_write(THIS + 4, struct.pack("<i", t))
        print(f"{t} -> {signed(m.call(ENTRY, (), ecx=THIS))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
