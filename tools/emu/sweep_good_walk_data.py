#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_good_walk_data.py — `Good::walk_data@0066e5d0` under unicorn (item 1921, tools/sweep_gen.py, mix).

    uv run tools/emu/sweep_good_walk_data.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object, one stack argument: the walker. The function makes
3 step(s) in order — direct callees (hooked), the walker's callback (slot 0, two
arguments) and its slot 4 (one argument, from the global cell).
One row per input, `<this> <walker> <g> <gate> -> <result> | <ecx,args…>;…`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0066e5d0
CALLS = [(6693328, 1, 'this', 'p0')]
GLOBAL = 0xc06378
GATE = None


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {a: (f"{a:x}", n) for a, n, _, _ in CALLS})
    rng = random.Random(1921)
    vt = h.vtable({0: ("walk", 2, 7), 4: ("walk4", 1, 9)})
    for i in range(140):
        this = rng.randrange(0x1000, 0x7FFF0000) & ~3
        g = rng.randrange(0x1000, 0x7FFF0000) & ~3
        gv = 0 if i % 2 == 0 else rng.randint(1, 0x7FFFFFFF)
        walker = h.alloc(32)
        h.w32(walker, vt)
        if GATE is not None:
            h.w32(walker + GATE, gv)
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
