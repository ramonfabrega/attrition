#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_air_turn_speed.py — `Unit::air_turn_speed@005ea390` under unicorn (item 2061, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_unit_air_turn_speed.py <install>/riseofnations.exe

`__thiscall`, two arguments `(sign, flag)`, `ret 8`, no callee. The rate is `(type +0x2c4 >> 8) × unit_turn_speed`
(the constants' `+8`, the cell `0xc061f0`; `imul`, wrapping); with `flag` 0 the guy's stored bank — the float
at `+0x44` of `**(this+0xf4)` — is truncated (`cvttss2si`) and negated, a sign that disagrees with `sign` answers
the floor `0x5b05b0`, else the rate `/ 0x37` (unsigned) times the bank's magnitude, floored.
The rows are the port's arm: `flag` 0, the type's `+0x2b4 & 0x20` clear (the other bits random), and a bank
inside `cvttss2si`'s range (the port saturates past it, single.rs).
Row: `<turn word> <unit_turn_speed> <bank bits> <sign> -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x005EA390
CONSTS = 0x00C061F0
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 0x37, 0x36, 255)


def bank(rng):
    r = rng.random()
    if r < 0.15:
        v = float(rng.choice((0, 1, -1, 2, -2, 0.5, -0.5, 0.99, -0.99, 60, -60, 1e-30, -0.0)))
    elif r < 0.3:
        v = float(rng.randint(-200, 200))
    else:
        v = rng.choice((1, -1)) * 2 ** rng.uniform(-8, 30.99)
    return struct.unpack("<I", struct.pack("<f", v))[0]


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2061)
    this = h.alloc(0x100)
    ty = h.alloc(0x300)
    rec = h.alloc(0x40)
    gp = h.alloc(0x10)
    guy = h.alloc(0x80)
    h.w32(CONSTS, rec)
    for _ in range(400):
        for a, n in ((this, 0x100), (ty, 0x300), (rec, 0x40), (guy, 0x80)):
            uc.mem_write(a, bytes(rng.randrange(256) for _ in range(n)))
        h.w32(this + 0x18, ty)
        h.w32(this + 0xF4, gp)
        h.w32(gp, guy)
        turn = rng.choice(I32) if rng.random() < 0.2 else rng.randint(-0x80000000, 0x7FFFFFFF)
        h.w32(ty + 0x2C4, turn)
        h.w8(ty + 0x2B4, uc.mem_read(ty + 0x2B4, 1)[0] & ~0x20)
        uts = rng.choice(I32) if rng.random() < 0.2 else rng.randint(-0x80000000, 0x7FFFFFFF)
        if rng.random() < 0.5:
            uts = rng.randrange(0x400)
            h.w32(ty + 0x2C4, rng.randrange(0x40000))
            turn = h.r32(ty + 0x2C4)
        h.w32(rec + 8, uts)
        bits = bank(rng)
        h.w32(guy + 0x44, bits)
        sign = rng.choice(I32) if rng.random() < 0.3 else rng.randint(-0x80000000, 0x7FFFFFFF)
        before = bytes(uc.mem_read(this, 0x100)), bytes(uc.mem_read(ty, 0x300)), bytes(uc.mem_read(guy, 0x80))
        res = h.call(ENTRY, (sign, 0), ecx=this)
        assert uc.reg_read(UC_X86_REG_ESP) == STACK_BASE + STACK_SIZE - 0x100, "wrong pop"
        assert (bytes(uc.mem_read(this, 0x100)), bytes(uc.mem_read(ty, 0x300)),
                bytes(uc.mem_read(guy, 0x80))) == before, "wrote"
        print(f"{turn} {uts} {bits} {sign} -> {res}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
