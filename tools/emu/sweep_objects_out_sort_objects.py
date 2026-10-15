#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_objects_out_sort_objects.py — `ObjectsOut::sort_objects@00654a30` under unicorn (item 2672, tools/sweep_gen.py).

    uv run tools/emu/sweep_objects_out_sort_objects.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object; calls 5 direct callee(s) and returns none.
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

ENTRY = 0x00654a30
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)




CALLS = [(6646896, 0, 'this', ''), (6648608, 0, 'this', ''), (6646176, 0, 'this', ''), (6645760, 0, 'this', ''), (6646528, 0, 'this', '')]
OWN = 0
# A `vtordisp` thunk (`sub ecx, [ecx-4]; jmp target`, item 2362) reads the word
# before `this`: `this` is laid in the heap with a drawn displacement there.
VTORDISP = False
# `--record-order` (item 2672, parked 2666): each call's token is headed `sK:`, K the
# callee's place in CALLS, so two callees on one `this` are ordered.
ORDER = True
DISPS = (0, 4, 8, 0x10, 0x1c, 0x58, 0x60, 0x100, -4, -0x60)


def rows():
    rng = random.Random(2672)
    return [(rng.randrange(0x1000, 0x7FFF0000) & ~3, rng.randint(-0x80000000, 0x7FFFFFFF),
             rng.randint(-0x80000000, 0x7FFFFFFF), rng.randint(-0x80000000, 0x7FFFFFFF),
             rng.randint(-0x80000000, 0x7FFFFFFF)) for _ in range(140)]


def main(argv):
    from hooks import Harness
    from callfn import STACK_BASE, STACK_SIZE
    h = Harness(argv[1], {a: (f"{a:x}", n) for a, n, _, _ in CALLS})
    drng = random.Random(2672 + 1)
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
        idx = {f"{a:x}": k for k, (a, _, _, _) in enumerate(CALLS)}
        seen = ";".join((f"s{idx[nm]}:" if ORDER else "") + ",".join(str(x) for x in (ecx, *args))
                        for nm, ecx, args, _ in h.calls)
        print(f"{this} {p0} {ans} {p1} {p2}{tail} -> {r} | {seen}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

# sweep_gen: ["fwd", "ObjectsOut::sort_objects@00654a30", "--item", "2672", "--call", "0x656c70:0:this:", "--call", "0x657320:0:this:", "--call", "0x6569a0:0:this:", "--call", "0x656800:0:this:", "--call", "0x656b00:0:this:", "--ret", "none", "--record-order"]
