#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_type_data_has_objmask.py — `ObjectTypeData::has_objmask@00470860` under unicorn (item 1814).

    uv run tools/emu/sweep_object_type_data_has_objmask.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the type record; returns `obj_masks` (+0x1e4) AND the argument — every pair of seeded words, against `masks & arg`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00470860
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
    ws = words()
    for i, mask in enumerate(ws):
        for arg in ws[i % 7::7]:
            uc.mem_write(OBJ + 0x1E4, struct.pack("<i", mask))
            r = signed(m.call(ENTRY, (arg,), ecx=OBJ))
            print(f"{mask} {arg} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
