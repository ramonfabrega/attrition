#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_follow_order_walk_data_vtordisp.py — `FollowOrder::walk_data_vtordisp@00486b94` under unicorn (item 2362, tools/sweep_gen.py).

    uv run tools/emu/sweep_follow_order_walk_data_vtordisp.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object; calls 1 direct callee(s) and returns none.
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

ENTRY = 0x00486b94
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)




CALLS = [(4746064, 1, 'vtordisp', 'p0')]
OWN = 1
# A `vtordisp` thunk (`sub ecx, [ecx-4]; jmp target`, item 2362) reads the word
# before `this`: `this` is laid in the heap with a drawn displacement there.
VTORDISP = True
DISPS = (0, 4, 8, 0x10, 0x1c, 0x58, 0x60, 0x100, -4, -0x60)


def rows():
    rng = random.Random(2362)
    return [(rng.randrange(0x1000, 0x7FFF0000) & ~3, rng.randint(-0x80000000, 0x7FFFFFFF),
             rng.randint(-0x80000000, 0x7FFFFFFF), rng.randint(-0x80000000, 0x7FFFFFFF),
             rng.randint(-0x80000000, 0x7FFFFFFF)) for _ in range(140)]


def main(argv):
    from hooks import Harness
    from callfn import STACK_BASE, STACK_SIZE
    h = Harness(argv[1], {a: (f"{a:x}", n) for a, n, _, _ in CALLS})
    drng = random.Random(2362 + 1)
    for this, p0, ans, p1, p2 in rows():
        tail = ""
        if VTORDISP:
            disp = drng.choice(DISPS) if drng.random() < 0.6 else drng.randint(-0x80000000, 0x7FFFFFFF)
            buf = h.alloc(8)
            h.w32(buf, disp)
            this, tail = buf + 4, f" {disp}"
        h.calls.clear()
        for i, (a, n, _, _) in enumerate(CALLS):
            h.answers[f"{a:x}"] = (ans + i) & 0xFFFFFFFF
        r = h.call(ENTRY, (p0, p1, p2)[:OWN], ecx=this)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        seen = ";".join(",".join(str(x) for x in (ecx, *args)) for _, ecx, args, _ in h.calls)
        print(f"{this} {p0} {ans} {p1} {p2}{tail} -> {r} | {seen}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

# sweep_gen: ["fwd", "FollowOrder::walk_data_vtordisp@00486b94", "--call", "0x486b50:1:vtordisp:p0", "--own", "1", "--item", "2362"]
