#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_decoy_time.py — `UnitData::decoy_time@00609d70` under unicorn (item 2072, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_unit_data_decoy_time.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`. It asks the leader `who` (the unsigned byte at `+0x9`) for its
general upgrade — the direct callee 0x6e0830, hooked, `ecx` the leaders' list (the cell `0xc061e0`) plus
`who * 0x6eec` — and answers `(upgrade + 2) * DECOY_TIME / 2 - mana_burn` (the constants through the cell
`0xc061e4`, `+0xcd0`; `mana_burn` the signed short at `+0x96`), a negative floored at 0. A row draws the
upgrade, the constant and the burn over the ranges a game holds and their edges.
Row: `<who> <upgrade> <DECOY_TIME> <mana_burn> <calls> <ecx is the leader> -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def popped(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


ENTRY = 0x00609D70
CONSTS = 0x00C061E4
LEADERS = 0x00C061E0
GENERAL = 0x006E0830


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {GENERAL: ("general", 0)})
    uc = h.m.uc
    rng = random.Random(2072)
    consts, this = h.alloc(0x1000), h.alloc(0x100)
    saved = (h.r32(CONSTS), h.r32(LEADERS))
    h.w32(CONSTS, consts)
    for i in range(300):
        base = rng.randrange(0x100000, 0x7FFF0000) & ~3
        h.w32(LEADERS, base)
        who = rng.randrange(256) if rng.random() < 0.2 else rng.randrange(10)
        g = rng.choice((-2, -1, 0, 1, 2, 3, 4)) if rng.random() < 0.8 else rng.randint(-1000, 1000)
        dt = rng.choice((0, 1, 2, 2500, 3000, 0x7FFF)) if rng.random() < 0.5 else rng.randint(-0x10000, 0x10000)
        burn = rng.choice((0, 1, -1, 0x7FFF, -0x8000, 2500)) if rng.random() < 0.4 else rng.randint(-0x8000, 0x7FFF)
        h.answers["general"] = g
        uc.mem_write(consts, bytes(rng.randrange(256) for _ in range(0x1000)))
        h.w32(consts + 0xCD0, dt & 0xFFFFFFFF)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x100)))
        h.w8(this + 0x9, who)
        h.w16(this + 0x96, burn)
        wins = ((this, 0x100), (consts, 0x1000))
        before = [bytes(uc.mem_read(a, n)) for a, n in wins]
        h.calls.clear()
        res = h.call(ENTRY, (), ecx=this)
        popped(uc)
        assert [bytes(uc.mem_read(a, n)) for a, n in wins] == before, "wrote"
        ok = int(all(c[1] == (base + who * 0x6EEC) & 0xFFFFFFFF for c in h.calls))
        print(f"{who} {g} {dt} {burn} {len(h.calls)} {ok} -> {res}")
    h.w32(CONSTS, saved[0])
    h.w32(LEADERS, saved[1])
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
