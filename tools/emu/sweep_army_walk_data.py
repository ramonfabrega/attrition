#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_army_walk_data.py — `Army::walk_data@006f9850` under unicorn (item 2547, tools/sweep_gen.py, mix).

    uv run tools/emu/sweep_army_walk_data.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object, one stack argument: the walker. The function makes
3 step(s) in order — direct callees (hooked), the walker's callback (slot 0, two
arguments) and its slot 4 (one argument, from the global cell).
One row per input, `<this> <walker> <g> <gate> -> <result> | <ecx,args…>;…` (the gate column is
the slot-4 callee's leftover `ecx` when a step names `stale`).
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x006f9850
CALLS = []
GLOBAL = 0xc06378
GATE = None
GATE_THIS = True
STALE = False


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {a: (f"{a:x}", n) for a, n, _, _ in CALLS})
    rng = random.Random(2547)
    vt = h.vtable({0: ("walk", 2, 7), 4: ("walk4", 1, 9)})
    for i in range(140):
        this = rng.randrange(0x1000, 0x7FFF0000) & ~3
        if GATE_THIS:
            this = h.alloc(0x100)  # a written object: the gate is the short at [this]
        g = rng.randrange(0x1000, 0x7FFF0000) & ~3
        gv = 0 if i % 2 == 0 else rng.randint(1, 0x7FFFFFFF)
        if STALE:
            # the `ecx` the slot-4 callee leaves (parked 1922, 2498), printed in the gate column
            gv = rng.randrange(0x1000, 0x7FFF0000) & ~3
            h.answers["walk4"] = (9, gv)
        walker = h.alloc(32)
        h.w32(walker, vt)
        if GATE is not None:
            h.w32(walker + GATE, gv)
        if GATE_THIS:
            gv = gv & 0xFFFF if i % 4 != 3 else 0
            h.w16(this, gv - 0x10000 if gv & 0x8000 else gv)
        if GLOBAL is not None:
            rec = h.alloc(0x40)
            h.w32(rec + 0x10, g)
            h.m.uc.mem_write(GLOBAL, struct.pack("<I", rec))
        h.calls.clear()
        r = h.call(ENTRY, (walker,), ecx=this)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        seen = ";".join(",".join(str(x) for x in (ecx, *args)) for _, ecx, args, _ in h.calls)
        print(f"{this} {walker} {g} {gv} -> {r} | {seen}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

# sweep_gen: ["mix", "Army::walk_data@006f9850", "--item", "2547", "--glob", "0xc06378", "--gate-this", "--step", "walk4:0xa78", "--step", "walk:this:this+2", "--step", "gated/walk:this+2:this+0x98"]
