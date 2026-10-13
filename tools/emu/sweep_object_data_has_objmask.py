#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_data_has_objmask.py — `ObjectData::has_objmask@0046cf10` under unicorn (item 1971, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_object_data_has_objmask.py <install>/riseofnations.exe

`__thiscall`, `ecx` = an object whose word at `+0x18` is another object with a fake vtable; the function jumps into slot 0xf0
(tail, `ecx` = that inner object, the stack untouched) and answers what it answers. The slot takes 1 stack argument(s).
Row: `<inner> <p0> <slot answer> -> <eax> | <ecx the slot saw>,<its first argument>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0046cf10
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
    rng = random.Random(1971)
    vt = h.vtable({0xf0: ("slot", 1, 0)})
    for _ in range(300):
        this, inner = h.alloc(0x40), h.alloc(0x40)
        h.w32(this + 0x18, inner)
        h.w32(inner, vt)
        p0, ans = rand32(rng), rand32(rng)
        h.answers["slot"] = ans & 0xFFFFFFFF
        h.calls.clear()
        r = h.call(ENTRY, (p0,), ecx=this)
        pop_ok(uc)
        assert len(h.calls) == 1, h.calls
        _, ecx, args, _ = h.calls[0]
        print(f"{inner} {p0} {ans} -> {r} | {ecx},{args[0] if args else 0}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
