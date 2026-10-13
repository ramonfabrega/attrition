#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_group_toggle.py — `Group::toggle@007083c0` under unicorn (item 1976, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_group_toggle.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the group (its vtable at `+0`); two stack arguments. Calls `GroupData::member@0070f8f0` (hooked, three arguments: `p0`, `p1`, 1, on the same `this`);
a non-zero answer calls slot `0x10`, zero calls slot `0xc`, each with `p0, p1, 0, 0` on the same `this`. `eax` is the slot's answer.
Row: `<this> <p0> <p1> <member answer> -> <eax> | <the calls, in order, name:ecx,args>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x007083c0
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def pop_ok(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {0x0070F8F0: ("member", 3)})
    uc = h.m.uc
    rng = random.Random(1976)
    vt = h.vtable({0x10: ("slot10", 4, 1010), 0xC: ("slot0c", 4, 2020)})
    for i in range(300):
        this = h.alloc(0x40)
        h.w32(this, vt)
        p0, p1 = rand32(rng), rand32(rng)
        ans = rng.choice((0, 1, 2, -1, 0x100, 0x7FFFFFFF))
        h.answers["member"] = ans & 0xFFFFFFFF
        h.calls.clear()
        r = h.call(ENTRY, (p0, p1), ecx=this)
        pop_ok(uc)
        seen = ";".join(f"{n}:{ecx},{','.join(str(x) for x in args)}" for n, ecx, args, _ in h.calls)
        print(f"{this} {p0} {p1} {ans} -> {r} | {seen}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
