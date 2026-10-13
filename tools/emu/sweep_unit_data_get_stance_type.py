#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_get_stance_type.py — `UnitData::get_stance_type@0046cee0` under unicorn (item 1993, tools/sweep_gen.py).

    uv run tools/emu/sweep_unit_data_get_stance_type.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object, whose callee object is at this+0x18; calls slot +0x138 of its vtable with 0 stack argument(s).
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

ENTRY = 0x0046cee0
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


OWN = 0


def rows():
    rng = random.Random(1993)
    vs = list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]
    return [(v, *(rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(OWN))) for v in vs]


def main(argv):
    from hooks import Harness
    from callfn import STACK_BASE, STACK_SIZE
    h = Harness(argv[1], {})
    for row in rows():
        h.calls.clear()
        vt = h.vtable({0x138: ("slot", 0, row[0], False)})
        inner = h.alloc(0x400)
        h.w32(inner, vt)
        obj = h.alloc(0x400)
        h.w32(obj + 0x18, inner)
        r = h.call(ENTRY, row[1:], ecx=obj)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert [(c[1], c[2]) for c in h.calls] == [(inner, list(row[1:1]))], (row, h.calls)
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
