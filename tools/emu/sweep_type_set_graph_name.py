#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_type_set_graph_name.py — `Type::set_graph_name@00663b60` under unicorn (item 1895, tools/sweep_gen.py).

    uv run tools/emu/sweep_type_set_graph_name.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object; calls 1 direct callee(s) and returns last.
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

ENTRY = 0x00663b60
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)




CALLS = [(10612400, 1, 'this+0x88', 'p0')]
OWN = 1


def rows():
    rng = random.Random(1895)
    return [(rng.randrange(0x1000, 0x7FFF0000) & ~3, rng.randint(-0x80000000, 0x7FFFFFFF),
             rng.randint(-0x80000000, 0x7FFFFFFF)) for _ in range(140)]


def main(argv):
    from hooks import Harness
    from callfn import STACK_BASE, STACK_SIZE
    h = Harness(argv[1], {a: (f"{a:x}", n) for a, n, _, _ in CALLS})
    for this, p0, ans in rows():
        h.calls.clear()
        for i, (a, n, _, _) in enumerate(CALLS):
            h.answers[f"{a:x}"] = (ans + i) & 0xFFFFFFFF
        r = h.call(ENTRY, (p0,)[:OWN], ecx=this)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        seen = ";".join(",".join(str(x) for x in (ecx, *args)) for _, ecx, args, _ in h.calls)
        print(f"{this} {p0} {ans} -> {r} | {seen}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
