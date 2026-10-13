#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_type_data_is_spellcaster.py — `BuildTypeData::is_spellcaster@00639840` under unicorn (item 2083,
hand-written on tools/emu/hooks.py and tools/emu/buildtypes.py).

    uv run tools/emu/sweep_build_type_data_is_spellcaster.py <install>/riseofnations.exe

`__thiscall` with one stack argument it never reads (`ret 4`). It takes its own index (`+0x4`) into the
`buildtypes` table and walks the `FROM` chain (`+0x3c`) from that entry to a negative, answering 1 at the first
record whose `+0x2c0` has `0x20000000`, else 0 — every record of the chain, not the root alone, and no vtable
call. Each record's bit 29 is cleared 60 % of the time so a chain without one is common. The stack argument is a
random word.
Row: `<this's index> <arg> <ten FROMs> <ten flag words> -> <eax>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from buildtypes import N, REC, World  # noqa: E402
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00639840
CASTER = 0x20000000


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2083)
    w = World(h, rng)
    for _ in range(300):
        froms, flags = w.lay()
        for i, r in enumerate(w.recs):
            if rng.random() < 0.6:
                v = flags[i] & ~CASTER
                h.w32(r + 0x2C0, v)
                flags[i] = v
        w.before = [bytes(uc.mem_read(r, REC)) for r in w.recs]
        s = rng.randrange(N)
        arg = rng.randint(-0x80000000, 0x7FFFFFFF)
        h.calls.clear()
        res = h.call(ENTRY, (arg,), ecx=w.recs[s])
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert not h.calls, "called a slot"
        w.check()
        print(" ".join(str(x) for x in (s, arg, *froms, *flags)) + f" -> {res}")
    w.restore()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
