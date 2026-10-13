#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_guy_turn_towards.py — `Guy::turn_towards@005d9720` under unicorn (item 2061, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_guy_turn_towards.py <install>/riseofnations.exe

`__thiscall`, three arguments `(to, unused, p3)`, `ret 0xc`. The heading is `this+0x18`; the turn owed is the
delta `to − heading`, its anticlockwise magnitude by `not`. Under three degrees (`0x2222220`) the turn snaps,
unasked; otherwise the guy's turn rate is asked (the direct callee at `0x5de340`, hooked, one argument, 1) and a
rate that covers the turn snaps too, else the heading steps by the rate the short way round. Either way the
direct callee at `0x5d97a0` (hooked, four arguments) is handed `(to, new heading, 1, p3)`; the answer is the turn
still owed.
Row: `<heading> <to> <rate> <p3> -> <eax> | <calls>`, each call `<name>:<ecx is this>:<arguments, unsigned>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x005D9720
RATE = 0x005DE340
TURN = 0x005D97A0
TOL = 0x2222220


def u32(v):
    return v & 0xFFFFFFFF


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {RATE: ("rate", 1), TURN: ("turn", 4)})
    uc = h.m.uc
    rng = random.Random(2061)
    this = h.alloc(0x40)
    for _ in range(500):
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x40)))
        frm = rng.getrandbits(32)
        r = rng.random()
        if r < 0.2:
            delta = rng.choice((0, 1, TOL - 1, TOL, TOL + 1, 0x80000000, 0x7FFFFFFF, 0x80000001,
                                u32(-TOL), u32(-TOL + 1), u32(-TOL - 1), u32(-1)))
        else:
            delta = rng.getrandbits(32)
        to = u32(frm + delta)
        owed = (~delta & 0xFFFFFFFF) if delta > 0x80000000 else delta
        r = rng.random()
        rate = (u32(owed + rng.choice((-1, 0, 1)))) if r < 0.3 else rng.getrandbits(32) if r < 0.5 else rng.randrange(owed + 1)
        h.answers["rate"] = rate
        h.w32(this + 0x18, frm)
        p2, p3 = rng.getrandbits(31), rng.getrandbits(31)
        before = bytes(uc.mem_read(this, 0x40))
        h.calls.clear()
        res = h.call(ENTRY, (to - (1 << 32) if to & 0x80000000 else to, p2, p3), ecx=this)
        assert uc.reg_read(UC_X86_REG_ESP) == STACK_BASE + STACK_SIZE - 0x100, "wrong pop"
        assert bytes(uc.mem_read(this, 0x40)) == before, "wrote"
        calls = ",".join(f"{name}:{int(ecx == this)}:" + "/".join(str(u32(a)) for a in args)
                         for name, ecx, args, _ in h.calls)
        print(f"{frm} {to} {rate} {p3} -> {u32(res)} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
