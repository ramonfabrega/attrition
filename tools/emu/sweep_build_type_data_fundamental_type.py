#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_type_data_fundamental_type.py — `BuildTypeData::fundamental_type@00639950` under unicorn (item 2072, hand-written on
tools/emu/hooks.py and tools/emu/buildtypes.py).

    uv run tools/emu/sweep_build_type_data_fundamental_type.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`. It walks `+0x3c` through the table until a negative `FROM`
and answers that root's `+0x4`. No slot is asked (mode is always 1).
Row: `<mode> <this's index> <the overriding slot's answer> <calls ok> <ten FROMs> <ten flag words> -> <eax>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from buildtypes import N, SLOT, World  # noqa: E402
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00639950


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2157)
    w = World(h, rng)
    over = h.vtable({SLOT: ("basic", 0, 0)})
    for i in range(300):
        froms, flags = w.lay()
        s = rng.randrange(N)
        mode = 1
        r = rng.randrange(N)
        h.answers["basic"] = r
        if not mode:
            h.w32(w.recs[s], over)
            w.before[s] = bytes(uc.mem_read(w.recs[s], 0x300))
        h.calls.clear()
        res = h.call(ENTRY, (), ecx=w.recs[s])
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        w.check()
        ok = int(len(h.calls) == (0 if mode else 1) and all(c[1] == w.recs[s] for c in h.calls))
        print(" ".join(str(x) for x in (mode, s, r, ok, *froms, *flags)) + f" -> {res}")
    w.restore()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
