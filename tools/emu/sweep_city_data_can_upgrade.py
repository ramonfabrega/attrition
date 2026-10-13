#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_city_data_can_upgrade.py — `CityData::can_upgrade@007383c0` under unicorn (item 2072, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_city_data_can_upgrade.py <install>/riseofnations.exe

`__thiscall`, one stack argument (the type to become), `ret 4`. `VILLAGE` (0x19e) answers 1 and asks nothing;
`TOWN` (0x19f) asks the direct callee 0x736540 (hooked, `ecx` this) of `CITY_BUILDINGS + 1` (the constants
through the cell `0xc061e4`, `+0x14c`), anything else of `METRO_BUILDINGS + 1` (`+0x150`), and answers whether
the callee's answer is non-zero. A row draws the type (the three cities, their neighbours, random), the two
constants and the callee's answer.
Row: `<type> <CITY_BUILDINGS> <METRO_BUILDINGS> <answer> <calls> <ecx is this> <handed or -1> -> <eax>`.
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


ENTRY = 0x007383C0
CONSTS = 0x00C061E4
KINDS = 0x00736540


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {KINDS: ("kinds", 1)})
    uc = h.m.uc
    rng = random.Random(2072)
    consts, this = h.alloc(0x400), h.alloc(0x100)
    saved = h.r32(CONSTS)
    h.w32(CONSTS, consts)
    for i in range(300):
        ty = rng.choice((0x19E, 0x19F, 0x1A0, 0x213, 0x19D, 0x1A1, 0, -1)) if rng.random() < 0.8 else rand32(rng)
        cb = rng.choice((0, 1, 3, 4, 6, -1)) if rng.random() < 0.6 else rng.randint(-0x10000, 0x10000)
        mb = rng.choice((0, 1, 6, 8, 10, -1)) if rng.random() < 0.6 else rng.randint(-0x10000, 0x10000)
        ans = rand32(rng) if rng.random() < 0.6 else 0
        h.answers["kinds"] = ans
        uc.mem_write(consts, bytes(rng.randrange(256) for _ in range(0x400)))
        h.w32(consts + 0x14C, cb & 0xFFFFFFFF)
        h.w32(consts + 0x150, mb & 0xFFFFFFFF)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x100)))
        wins = ((this, 0x100), (consts, 0x400))
        before = [bytes(uc.mem_read(a, n)) for a, n in wins]
        h.calls.clear()
        res = h.call(ENTRY, (ty,), ecx=this)
        popped(uc)
        assert [bytes(uc.mem_read(a, n)) for a, n in wins] == before, "wrote"
        ok = int(all(c[1] == this for c in h.calls))
        handed = h.calls[0][2][0] if h.calls else -1
        print(f"{ty} {cb} {mb} {ans} {len(h.calls)} {ok} {handed} -> {res}")
    h.w32(CONSTS, saved)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
