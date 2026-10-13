#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_armies_num_armies.py — `Armies::num_armies@006f3200` under unicorn (item 2038, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_armies_num_armies.py <install>/riseofnations.exe

`__thiscall` with three stack arguments `(who, mask, reg)`, `ret 0xc`; `this` is never read — the list is the
global `armies` singleton's, its per-player pointer table at `0xc09710 + who*0x1c`. The function counts the
sixteen army slots whose record has a nonzero `valid` short at `+0`, a status word at `+4` sharing a bit with
`mask`, and — unless `reg < 0` — the region `+8` equal to `reg`. A row draws `who` in 0..9, the records (valid,
status, region) and the arguments; `this` is a random word.
Row: `<who> <mask> <reg> <valid status region>×16 -> <eax>`.
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


ENTRY = 0x006F3200
TABLE = 0x00C09710


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2038)
    slots = h.alloc(16 * 4)
    recs = [h.alloc(0x10) for _ in range(16)]
    for k, r in enumerate(recs):
        h.w32(slots + 4 * k, r)
    for i in range(300):
        who = rng.randrange(9)
        table = bytes(uc.mem_read(TABLE, 9 * 0x1C))
        h.w32(TABLE + who * 0x1C, slots)
        bits = rng.choice((1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80))
        mask = rand32(rng) if rng.random() < 0.3 else bits | (rng.choice((0, 2, 0x40)) if rng.random() < 0.3 else 0)
        reg = rng.choice((-1, -2, 0, 1, 2, 3, 0x40, 0x41)) if rng.random() < 0.8 else rand32(rng)
        cells = []
        for r in recs:
            valid = rng.choice((0, 1, -1, 0x100)) if rng.random() < 0.7 else rng.randint(-0x8000, 0x7FFF)
            status = rng.choice((0, 1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80, 0x41, 3)) if rng.random() < 0.8 else rand32(rng)
            region = rng.choice((0, 1, 2, 3, 0x40, 0x41, -1)) if rng.random() < 0.8 else rand32(rng)
            uc.mem_write(r, bytes(rng.randrange(256) for _ in range(0x10)))
            h.w16(r, valid)
            h.w32(r + 4, status)
            h.w32(r + 8, region)
            cells += [valid, status, region]
        before = [bytes(uc.mem_read(r, 0x10)) for r in recs]
        res = h.call(ENTRY, (who, mask, reg), ecx=rand32(rng) & 0xFFFFFFFF)
        popped(uc)
        assert [bytes(uc.mem_read(r, 0x10)) for r in recs] == before, "wrote a record"
        h.w32(TABLE + who * 0x1C, struct.unpack_from("<I", table, who * 0x1C)[0])
        print(" ".join(str(x) for x in (who, mask, reg, *cells)) + f" -> {res}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
