#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_type_data_connects_to_roads.py — `BuildTypeData::connects_to_roads@006398c0` under unicorn
(item 2038, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_build_type_data_connects_to_roads.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`. Its own vtable slot `+0x90` is the gatherer test: when the slot
holds the base address (`0x472bb0`) the function reads `build_flags` (`+0x2c0`) `& 0x40` without a call;
otherwise it calls the slot and takes its answer. When that is nonzero, or `build_flags & 0x10` is set, it asks
slot `+0x60` (`is`, two arguments) of `0x1a4` and answers 0 if the slot says no; every other path answers 1. A
row's mode is 1 for the base slot (never entered: the comparison is on the pointer), 0 for a stub answering a
drawn value; `is` answers a drawn value too.
Row: `<mode> <flags> <gather answer> <is answer> <calls> <is handed 0> <is handed 1> <ecx is this> -> <eax>`.
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


ENTRY = 0x006398C0
BASE = 0x00472BB0


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2038)
    vt = h.vtable({0x90: ("gather", 0, 0), 0x60: ("is", 2, 0)})
    stub = h.r32(vt + 0x90)
    this = h.alloc(0x300)
    for i in range(300):
        mode = i % 2
        flags = rand32(rng)
        if rng.random() < 0.6:
            flags &= ~rng.choice((0x40, 0x10, 0x50))
        g = rng.choice((0, 0, 1, -1, 0x40)) if rng.random() < 0.7 else rand32(rng)
        w = rng.choice((0, 0, 1, -1)) if rng.random() < 0.7 else rand32(rng)
        h.answers["gather"], h.answers["is"] = g, w
        h.w32(vt + 0x90, BASE if mode else stub)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x300)))
        h.w32(this, vt)
        h.w32(this + 0x2C0, flags)
        h.calls.clear()
        before = bytes(uc.mem_read(this, 0x300))
        res = h.call(ENTRY, (), ecx=this)
        popped(uc)
        assert bytes(uc.mem_read(this, 0x300)) == before, "wrote the object"
        isc = [c for c in h.calls if c[0] == "is"]
        handed = isc[0][2] if isc else [0, 0]
        names = "".join({"gather": "1", "is": "2"}[c[0]] for c in h.calls) or "0"
        ecx_this = int(all(c[1] == this for c in h.calls))
        print(" ".join(str(x) for x in (mode, flags, g, w, names, *handed, ecx_this)) + f" -> {res}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
