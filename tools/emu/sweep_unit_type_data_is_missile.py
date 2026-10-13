#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_type_data_is_missile.py — `UnitTypeData::is_missile@0061d430` under unicorn (item 2020, hand-written
on tools/emu/hooks.py).

    uv run tools/emu/sweep_unit_type_data_is_missile.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`. It reads its own vtable slot `+0xf0`: when the slot is the base
`has_objmask` (0x470860) it answers `[this+0x1e4] & 0x8000000 != 0` without a call; otherwise it calls the slot
with the one argument `0x8000000` and answers the slot's answer `!= 0`. A row's mode is 1 when the slot holds the
base address (never entered: the comparison is on the pointer), 0 when it is a stub that answers a value drawn
per row. The mask word is random, the missile bit forced on or off half the time.
Row: `<mode> <mask word> <slot answer> <calls> <handed> <ecx is this> -> <eax>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0061D430
BASE = 0x00470860
SLOT = 0xF0
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2020)
    vt = h.vtable({SLOT: ("slot", 1, 0)})
    stub = h.r32(vt + SLOT)
    obj = h.alloc(0x240)
    for i in range(240):
        mode = i % 2
        word = rand32(rng)
        if rng.random() < 0.5:
            word = (word | 0x8000000) if rng.random() < 0.5 else (word & ~0x8000000)
        v = rand32(rng)
        h.calls.clear()
        h.answers["slot"] = v
        h.w32(vt + SLOT, BASE if mode else stub)
        uc.mem_write(obj, bytes(rng.randrange(256) for _ in range(0x200)))
        h.w32(obj, vt)
        h.w32(obj + 0x1E4, word)
        before = bytes(uc.mem_read(obj, 0x200))
        r = h.call(ENTRY, (), ecx=obj)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(obj, 0x200)) == before, "wrote the object"
        handed = h.calls[0][2][0] if h.calls else 0
        ecx_this = int(h.calls[0][1] == obj) if h.calls else 1
        print(" ".join(str(x) for x in (mode, word, v, len(h.calls), handed, ecx_this)) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
