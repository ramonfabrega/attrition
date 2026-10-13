#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_caravan_link_walk_data.py — `CaravanLink::walk_data@007369c0` under unicorn (item 1916, tools/sweep_gen.py, walk).

    uv run tools/emu/sweep_caravan_link_walk_data.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object, one stack argument: the walker. The function calls
the walker's callback (the first word of the walker's vtable) with `ecx` = the walker
and two stack arguments, `this` and `this+0x8` of the object. One row per input,
`<this> <walker> -> <result> | <ecx,arg0,arg1>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x007369c0


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    rng = random.Random(1916)
    for _ in range(140):
        this = rng.randrange(0x1000, 0x7FFF0000) & ~3
        ans = rng.randint(-0x80000000, 0x7FFFFFFF)
        vt = h.vtable({0: ("walk", 2, ans & 0xFFFFFFFF)})
        walker = h.alloc(16)
        h.w32(walker, vt)
        h.calls.clear()
        r = h.call(ENTRY, (walker,), ecx=this)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        seen = ";".join(",".join(str(x) for x in (ecx, *args)) for _, ecx, args, _ in h.calls)
        print(f"{this} {walker} -> {r} | {seen}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
