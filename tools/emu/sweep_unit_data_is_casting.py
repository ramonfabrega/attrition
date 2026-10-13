#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_is_casting.py — `UnitData::is_casting@0060a530` under unicorn (item 2038, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_unit_data_is_casting.py <install>/riseofnations.exe

`__thiscall` with one stack argument (the spell), `ret 4`. It reads the unit's own record through the owner's
table (`0xc0aec0 + who*0x1c`, `who` the byte at `+0x9`, the index the signed short at `+0xa`) and answers 1 for
spell `0x27c` when that record's word at `+0x68` has bit `0x8000`, for `0x27b` when it has `0x2000`, and 0 for
any other spell without reading it. A row draws the spell (the two, their neighbours, a random word), the flag
word, `who` and the index.
Row: `<who> <index> <spell> <flags> -> <eax>`.
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


ENTRY = 0x0060A530
TABLE = 0x00C0AEC0
N = 64


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2038)
    slots = h.alloc(N * 4)
    rec = h.alloc(0x80)
    this = h.alloc(0x40)
    for i in range(300):
        who = rng.randrange(9)
        saved = h.r32(TABLE + who * 0x1C)
        h.w32(TABLE + who * 0x1C, slots)
        idx = rng.randrange(N)
        uc.mem_write(slots, bytes(rng.randrange(256) for _ in range(N * 4)))
        h.w32(slots + 4 * idx, rec)
        spell = rng.choice((0x27C, 0x27B, 0x27A, 0x27D, 0, -1)) if rng.random() < 0.85 else rand32(rng)
        flags = rand32(rng)
        if rng.random() < 0.5:
            bit = rng.choice((0x8000, 0x2000, 0xA000))
            flags = flags | bit if rng.random() < 0.5 else flags & ~bit
        uc.mem_write(rec, bytes(rng.randrange(256) for _ in range(0x80)))
        h.w32(rec + 0x68, flags)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x40)))
        h.w8(this + 0x9, who)
        h.w16(this + 0xA, idx)
        before = bytes(uc.mem_read(this, 0x40)), bytes(uc.mem_read(rec, 0x80))
        res = h.call(ENTRY, (spell,), ecx=this)
        popped(uc)
        assert (bytes(uc.mem_read(this, 0x40)), bytes(uc.mem_read(rec, 0x80))) == before, "wrote"
        h.w32(TABLE + who * 0x1C, saved)
        print(f"{who} {idx} {spell} {flags} -> {res}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
