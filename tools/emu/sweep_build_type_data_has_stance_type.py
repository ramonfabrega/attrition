#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_type_data_has_stance_type.py — `BuildTypeData::has_stance_type@00639770` under unicorn (item 1966, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_build_type_data_has_stance_type.py <install>/riseofnations.exe

`__thiscall`, `ecx` = an object whose first word is a fake vtable; one stack argument `p0`. It calls slot 0x138
(hooked, no arguments) and answers 1 when the slot's answer equals `p0`, else 0. Row: `<slot answer> <p0> -> <eax>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00639770
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    rng = random.Random(1966)
    h = Harness(argv[1], {})
    vt = h.vtable({0x138: ("stance", 0, 0)})
    this = h.alloc(0x40)
    h.w32(this, vt)
    for _ in range(300):
        s = rand32(rng)
        p0 = s if rng.random() < 0.4 else rand32(rng)
        h.answers["stance"] = s & 0xFFFFFFFF
        r = h.call(ENTRY, (p0,), ecx=this)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        print(f"{s} {p0} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
