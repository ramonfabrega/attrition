#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_armies_find_city.py — `Armies::find_city@006f3160` under unicorn (item 2044, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_armies_find_city.py <install>/riseofnations.exe

`__thiscall` with three stack arguments (`who`, the city, a third never read), `ret 0xc`; `this` is never read.
It walks the player's sixteen army slots (`[0xc09710 + who*0x1c]`, an array of record pointers) and answers the
first whose `valid` short (`+0x0`) is nonzero, whose target word (`+0x20`) is the city and whose status byte
(`+0x4`) has bit 0; else -1. A row draws the city from a few values and each slot's three fields so that some
rows match early, some late and some never.
Row: `<who> <city> <p3> (<valid> <status> <target>)×16 -> <eax>`.
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


ENTRY = 0x006F3160
TABLE = 0x00C09710
REC = 0x28


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2044)
    slots = h.alloc(16 * 4)
    recs = [h.alloc(REC) for _ in range(16)]
    for k, r in enumerate(recs):
        h.w32(slots + 4 * k, r)
    for _ in range(300):
        who = rng.randrange(9)
        table = bytes(uc.mem_read(TABLE, 9 * 0x1C))
        h.w32(TABLE + who * 0x1C, slots)
        city = rng.choice((0, 1, 2, 7, -1)) if rng.random() < 0.85 else rand32(rng)
        p3 = rand32(rng)
        cells = []
        for r in recs:
            valid = rng.choice((0, 1, -1, 0x100)) if rng.random() < 0.7 else rng.randint(-0x8000, 0x7FFF)
            status = rng.choice((0, 1, 2, 3, 0x41, 0x100, 0x101)) if rng.random() < 0.8 else rand32(rng)
            target = city if rng.random() < 0.25 else (rng.choice((0, 1, 2, 7, -1)) if rng.random() < 0.7 else rand32(rng))
            uc.mem_write(r, bytes(rng.randrange(256) for _ in range(REC)))
            h.w16(r, valid)
            h.w32(r + 4, status)
            h.w32(r + 0x20, target)
            cells += [valid, status, target]
        before = [bytes(uc.mem_read(r, REC)) for r in recs]
        res = h.call(ENTRY, (who, city, p3), ecx=rand32(rng) & 0xFFFFFFFF)
        popped(uc)
        assert [bytes(uc.mem_read(r, REC)) for r in recs] == before, "wrote a record"
        h.w32(TABLE + who * 0x1C, struct.unpack_from("<I", table, who * 0x1C)[0])
        print(" ".join(str(x) for x in (who, city, p3, *cells)) + f" -> {res}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
