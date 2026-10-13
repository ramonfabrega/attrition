#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_data_hits_left.py — `ObjectData::hits_left@006535c0` under unicorn (item 1993, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_object_data_hits_left.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object (vtable at `+0`, damage int at `+0x24`). Calls slot `0x11c` twice, each with one stack argument 0 and `ecx = this`: the first answer is `max`, the second `cur`.
With `left = cur - damage` (wrapping, tested by sign): `left < 0` or `max < 0` answers 0, else `min(left, max)`.
Row: `<this> <first> <second> <damage> -> <eax> | <the calls, in order, name:ecx,args>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x006535c0
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def pop_ok(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(1993)
    vt = h.vtable({0x11C: ("slot", 1, lambda ecx, args: nxt.pop(0))})
    nxt = []
    for i in range(400):
        this = h.alloc(0x80)
        h.w32(this, vt)
        a, b, d = rand32(rng), rand32(rng), rand32(rng)
        if i % 3:
            a, b, d = rng.randint(0, 500), rng.randint(0, 500), rng.randint(-10, 600)
        h.w32(this + 0x24, d)
        nxt[:] = [a & 0xFFFFFFFF, b & 0xFFFFFFFF]
        h.calls.clear()
        r = h.call(ENTRY, (), ecx=this)
        pop_ok(uc)
        assert [(c[1], c[2]) for c in h.calls] == [(this, [0]), (this, [0])], h.calls
        print(f"{this} {a} {b} {d} -> {r}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
