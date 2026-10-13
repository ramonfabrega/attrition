#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_type_data_get_kill_value.py — `TypeData::get_kill_value@00663a00` under unicorn (item 1954, tools/sweep_gen.py).

    uv run tools/emu/sweep_type_data_get_kill_value.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object; calls slot +0x7c of its vtable with 1 stack argument(s).
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402
from sweeplib import OBJ, TYPE, REC, OUT, cell, setup  # noqa: E402,F401
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402,F401

ENTRY = 0x00663a00
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rows():
    rng = random.Random(1954)
    out = [(v,) for v in INT_EDGES]
    out += [(rng.randint(-0x80000000, 0x7FFFFFFF),) for _ in range(200)]
    return out


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    for row in rows():
        h.calls.clear()
        vt = h.vtable({0x7c: ("slot", 1, row[0], False)})
        obj = h.alloc(0x400)
        h.w32(obj, vt)
        r = h.call(ENTRY, (), ecx=obj)
        handed = ",".join(str(a) for name, ecx, args, _ in h.calls for a in args)
        print(" ".join(str(x) for x in row) + f" -> {r}" + (f"  # slot handed {handed}" if handed else ""))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
