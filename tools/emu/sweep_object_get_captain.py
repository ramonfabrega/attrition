#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_get_captain.py — `ObjectData::get_captain@00472400` under unicorn.

    uv run tools/emu/sweep_object_get_captain.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object; answers the signed short at `+0xa` (`o`), sign-extended.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00472400
OBJ = 0x10000000
ARG = 0x20000000


def rows():
    rng = random.Random(1708)
    edges = [0, 1, -1, 2, 2000, 2999, 3000, 32767, -32768, -2, 255, 256, -256, 0x7F, -128, 0x4000, -0x4000]
    return [(v,) for v in edges] + [(1 << b,) for b in range(15)] + [(-(1 << b),) for b in range(16)] + [(rng.randint(-32768, 32767),) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARG, 0x1000)
    for row in rows():
        uc.mem_write(OBJ, b"\xA5" * 0x40)
        uc.mem_write(OBJ + 0xA, struct.pack("<h", row[0]))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
