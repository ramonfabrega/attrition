#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_wall_set_type.py — `Wall::set_type@00640da0` under unicorn (item 2020, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_wall_set_type.py <install>/riseofnations.exe

`__thiscall`, two stack arguments (`TypeIndex`, `SetTypeIndex`), `ret 8`. Three direct callees, each hooked:
the wall's stats decrement (0x642da0, no arguments) when the kind is `SET_TYPE_NORMAL` (0); the base
`set_type` (0x662450, `ret 8`) handed both arguments unchanged, always; the stats increment (0x643270, no
arguments) unless the kind is `SET_TYPE_LOAD` (2). Every call's `ecx` must be `this`. The kind is drawn from
-1..4 and random words; the type is random. The call sequence is printed as digits — 1 decrement, 2 base,
3 increment — and is the row's result.
Row: `<type> <kind> <base handed type> <base handed kind> <every ecx is this> -> <sequence>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00640DA0
CALLEES = {0x00642DA0: ("dec", 0), 0x00662450: ("base", 2), 0x00643270: ("inc", 0)}
DIGIT = {"dec": "1", "base": "2", "inc": "3"}
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    rng = random.Random(2020)
    h = Harness(argv[1], CALLEES)
    for name in DIGIT:
        h.answers[name] = lambda ecx, args: rng.randint(-0x80000000, 0x7FFFFFFF)
    uc = h.m.uc
    this = h.alloc(0x100)
    for i in range(240):
        ty = rand32(rng)
        kind = rng.choice((-1, 0, 1, 2, 3, 4)) if rng.random() < 0.8 else rand32(rng)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x100)))
        before = bytes(uc.mem_read(this, 0x100))
        h.calls.clear()
        h.call(ENTRY, (ty, kind), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(this, 0x100)) == before, "wrote the object"
        seq = int("".join(DIGIT[c[0]] for c in h.calls))
        base = [c[2] for c in h.calls if c[0] == "base"]
        handed = base[0] if base else [0, 0]
        every = int(all(c[1] == this for c in h.calls))
        print(" ".join(str(x) for x in (ty, kind, *handed, every)) + f" -> {seq}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
