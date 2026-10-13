#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_type_data_is_defense.py — `ObjectTypeData::is_defense@00470470` under unicorn (item 1993, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_object_type_data_is_defense.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object (its vtable at `+0`). Calls slot `0xfc`; a non-zero answer returns 1, else calls slot `0x100` and answers 1 when that is non-zero, else 0. Both calls are handed `ecx = this`, no stack arguments.
Row: `<this> <slot fc> <slot 100> -> <eax> | <the calls, in order, name:ecx>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00470470
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
    vt = h.vtable({0xFC: ("slotfc", 0, 0), 0x100: ("slot100", 0, 0)})
    for i in range(300):
        this = h.alloc(0x40)
        h.w32(this, vt)
        a = rng.choice((0, 0, 1, 2, -1, 0x100, 0x7FFFFFFF, -0x80000000))
        b = rng.choice((0, 0, 1, 2, -1, 0x100, 0x7FFFFFFF, -0x80000000))
        h.answers["slotfc"] = a & 0xFFFFFFFF
        h.answers["slot100"] = b & 0xFFFFFFFF
        h.calls.clear()
        r = h.call(ENTRY, (), ecx=this)
        pop_ok(uc)
        seen = ";".join(f"{n}:{ecx}" for n, ecx, args, _ in h.calls)
        assert all(not args for _, _, args, _ in h.calls)
        print(f"{this} {a} {b} -> {r} | {seen}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
