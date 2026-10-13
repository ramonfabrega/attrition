#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_death_obj_data_blocking.py — `DeathObjData::blocking@008d4a90` under unicorn (item 1976, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_death_obj_data_blocking.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the record: `who` at `+0x18`, `o` at `+0x1c`. The object is `units[who].list[o]` — the pointer array in the global cell at `0xc0aec0 + 0x1c*who`, element `o` — and the call is slot `0x120` of the object at its `+0x18`, no arguments.
Answers 1 when the slot's answer is not zero.
Row: `<who> <o> <slot answer> <type> -> <eax> | <ecx>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x008d4a90
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
    vt = h.vtable({0x120: ("slot", 0, 0)})
    for _ in range(300):
        this = h.alloc(0x40)
        who, o = rng.randrange(4), rng.randrange(8)
        arr = h.alloc(0x40)
        obj, typ = h.alloc(0x40), h.alloc(0x40)
        h.w32(this + 0x18, who)
        h.w32(this + 0x1C, o)
        uc.mem_write(0x00C0AEC0 + 0x1C * who, struct.pack("<I", arr))
        h.w32(arr + 4 * o, obj)
        h.w32(obj + 0x18, typ)
        h.w32(typ, vt)
        ans = rand32(rng) if rng.random() < 0.8 else 0
        h.answers["slot"] = ans & 0xFFFFFFFF
        h.calls.clear()
        r = h.call(ENTRY, (), ecx=this)
        assert len(h.calls) == 1, h.calls
        print(f"{who} {o} {ans} {typ} -> {r} | {h.calls[0][1]}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
