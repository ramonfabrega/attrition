#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_type_data_research_time.py — `UnitTypeData::research_time@0061d760` under unicorn (item 1998,
hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_unit_type_data_research_time.py <install>/riseofnations.exe

`__thiscall`, one stack argument, `ret 4`. It hands the argument to `TypeData::research_time@006639a0` (hooked:
`ecx` = `this`, one argument, `ret 4`) and answers `(t * research_premium_time) / 256` — a 32-bit `imul` with
`research_premium_time` the int at `this+0x2e4`, then `cdq; and edx, 0xff; add; sar 8`: a signed division by 256
that truncates toward zero, not an arithmetic shift. The rows draw `t` and the premium from the edges, the shipped
8.8 values and random words, so both signs of the product are reached.
Row: `<p0> <t> <premium> <handed p0> <ecx is this> -> <eax>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0061D760
CALLEE = 0x006639A0
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 255, 256, 257, -255, -257, 512, 0x3C00, 60_000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    rng = random.Random(1998)
    cur = {}
    h = Harness(argv[1], {CALLEE: ("TypeData::research_time", 1)})
    h.answers["TypeData::research_time"] = lambda ecx, args: cur["t"]
    uc = h.m.uc
    this = h.alloc(0x400)
    for i in range(420):
        p0 = rand32(rng)
        t = rand32(rng) if i % 2 else rng.randint(0, 200_000)
        prem = rand32(rng) if i % 2 else rng.choice((0x100, 0x200, 0x180, 0x80))
        cur["t"] = t
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x400)))
        h.w32(this + 0x2E4, prem)
        h.calls.clear()
        r = h.call(ENTRY, (p0,), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert len(h.calls) == 1, h.calls
        _, ecx, args, _ = h.calls[0]
        print(f"{p0} {t} {prem} {args[0]} {int(ecx == this)} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
