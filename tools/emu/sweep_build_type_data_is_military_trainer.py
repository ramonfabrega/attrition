#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_type_data_is_military_trainer.py — `BuildTypeData::is_military_trainer@0063bcf0` under unicorn (item 2072, hand-written on
tools/emu/hooks.py and tools/emu/buildtypes.py).

    uv run tools/emu/sweep_build_type_data_is_military_trainer.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`. It compares its own slot `+0xe4` with `basic_type`'s address
(0x639970): equal, a root answers its own table entry's `+0x2c0 & 0x40000000` and a child asks its parent's
slot (the real `basic_type`, recursing); different, it asks its own slot. The answer indexes the table, whose
record's `+0x2c0 & 0x40000000` it answers. Mode 1 is the base, mode 0 a stub answering an index.
Row: `<mode> <this's index> <the overriding slot's answer> <calls ok> <ten FROMs> <ten flag words> -> <eax>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from buildtypes import N, SLOT, World  # noqa: E402
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0063BCF0


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2159)
    w = World(h, rng)
    over = h.vtable({SLOT: ("basic", 0, 0)})
    for i in range(300):
        froms, flags = w.lay()
        s = rng.randrange(N)
        mode = i % 2
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
