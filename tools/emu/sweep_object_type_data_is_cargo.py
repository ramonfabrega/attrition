#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_type_data_is_cargo.py — `ObjectTypeData::is_cargo@00661d80` under unicorn (item 2020, hand-written
on tools/emu/hooks.py).

    uv run tools/emu/sweep_object_type_data_is_cargo.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`. It reads its own vtable slot `+0xc`: when the slot is the base
`TypeData::is_unit_type` (0x4707e0) it tests the `TypeIndex` at `this+4` against `0x32 <= t < 0x19e` itself, with
no call; otherwise it calls the slot (no arguments) and goes on only when it answers non-zero. Past either, it
answers `[this+0x218] == 0` (the domain, land). A row's mode is 1 when the slot holds the base address, 0 when
it is a stub answering a value drawn per row. The index is drawn round the range's edges and at random; the
domain is 0, 1, 2 or a random word.
Row: `<mode> <type index> <domain> <slot answer> <calls> -> <eax>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00661D80
BASE = 0x004707E0
SLOT = 0x0C
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)
EDGES = (0x30, 0x31, 0x32, 0x33, 0x100, 0x19C, 0x19D, 0x19E, 0x19F, 0, -1, 0x7FFFFFFF, -0x80000000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2020)
    vt = h.vtable({SLOT: ("slot", 0, 0)})
    stub = h.r32(vt + SLOT)
    obj = h.alloc(0x240)
    for i in range(300):
        mode = i % 2
        t = rng.choice(EDGES) if rng.random() < 0.5 else rng.randrange(-0x10, 0x220)
        dom = rng.choice((0, 0, 1, 2, rand32(rng)))
        v = rand32(rng)
        h.calls.clear()
        h.answers["slot"] = v
        h.w32(vt + SLOT, BASE if mode else stub)
        uc.mem_write(obj, bytes(rng.randrange(256) for _ in range(0x240)))
        h.w32(obj, vt)
        h.w32(obj + 4, t)
        h.w32(obj + 0x218, dom)
        before = bytes(uc.mem_read(obj, 0x240))
        r = h.call(ENTRY, (), ecx=obj)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(obj, 0x240)) == before, "wrote the object"
        assert all(c[1] == obj for c in h.calls), "the slot's ecx is this"
        print(" ".join(str(x) for x in (mode, t, dom, v, len(h.calls))) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
