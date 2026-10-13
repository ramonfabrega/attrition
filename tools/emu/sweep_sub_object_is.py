#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_sub_object_is.py — `SubObject::is@00476e50` under unicorn (item 1976, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_sub_object_is.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the sub-object; tail-jumps into slot `0x60` of the object at `+0x18`, the two stack arguments passed through.
Row: `<inner> <p0> <p1> <slot answer> -> <eax> | <ecx>,<p0>,<p1>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00476e50
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
    rng = random.Random(1976)
    vt = h.vtable({0x60: ("slot", 2, 0)})
    for _ in range(300):
        this, inner = h.alloc(0x40), h.alloc(0x40)
        h.w32(this + 0x18, inner)
        h.w32(inner, vt)
        p0, p1, ans = rand32(rng), rand32(rng), rand32(rng)
        h.answers["slot"] = ans & 0xFFFFFFFF
        h.calls.clear()
        r = h.call(ENTRY, (p0, p1), ecx=this)
        pop_ok(uc)
        assert len(h.calls) == 1, h.calls
        _, ecx, args, _ = h.calls[0]
        print(f"{inner} {p0} {p1} {ans} -> {r} | {ecx},{args[0]},{args[1]}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
