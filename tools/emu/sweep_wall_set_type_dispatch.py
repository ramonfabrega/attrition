#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_wall_set_type_dispatch.py — `Wall::set_type@004723b0` under unicorn (item 2038, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_wall_set_type_dispatch.py <install>/riseofnations.exe

`__thiscall` with two stack arguments `(type, kind)`, `ret 8`: the header's dispatcher. When its own vtable slot
`+0x84` holds the wall's own retype (`0x640da0`), it runs that body inline — the stats taken off before kind 0,
the base retype handed both arguments, the stats put back unless kind 2, the three callees hooked; otherwise it
calls the slot with both arguments. A row's mode is 1 for the own address (never entered: the comparison is on
the pointer), 0 for a stub. The result is the call order: 1 off, 2 retype, 3 on, 4 the slot.
Row: `<mode> <type> <kind> <handed 0> <handed 1> <every ecx is this> -> <sequence>`.
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


ENTRY = 0x004723B0
OWN = 0x00640DA0
CALLEES = {0x00642DA0: ("dec", 0), 0x00662450: ("base", 2), 0x00643270: ("inc", 0)}
DIGIT = {"dec": "1", "base": "2", "inc": "3", "slot": "4"}


def main(argv):
    from hooks import Harness
    rng = random.Random(2038)
    h = Harness(argv[1], CALLEES)
    for name in ("dec", "base", "inc"):
        h.answers[name] = lambda ecx, args: rng.randint(-0x80000000, 0x7FFFFFFF)
    uc = h.m.uc
    vt = h.vtable({0x84: ("slot", 2, 0)})
    stub = h.r32(vt + 0x84)
    this = h.alloc(0x100)
    for i in range(300):
        mode = i % 2
        ty = rand32(rng)
        kind = rng.choice((-1, 0, 1, 2, 3, 4)) if rng.random() < 0.8 else rand32(rng)
        h.answers["slot"] = rand32(rng)
        h.w32(vt + 0x84, OWN if mode else stub)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x100)))
        h.w32(this, vt)
        before = bytes(uc.mem_read(this, 0x100))
        h.calls.clear()
        h.call(ENTRY, (ty, kind), ecx=this)
        popped(uc)
        assert bytes(uc.mem_read(this, 0x100)) == before, "wrote the object"
        seq = int("".join(DIGIT[c[0]] for c in h.calls))
        handed = [c[2] for c in h.calls if c[0] in ("base", "slot")]
        two = handed[0] if handed else [0, 0]
        every = int(all(c[1] == this for c in h.calls))
        print(" ".join(str(x) for x in (mode, ty, kind, *two, every)) + f" -> {seq}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
