#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_type_gather_radius.py — `UnitType::gather_radius@0061daa0` under unicorn (item 2038, hand-written
on tools/emu/hooks.py).

    uv run tools/emu/sweep_unit_type_gather_radius.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`. The type id at `+0x4` being `0x3d`, `0x3e` or `0x190` answers
`level*4 + 4`, `level` the vtable slot `+0x124`'s answer, without asking anything else; any other id asks slot
`+0x60` (`is`, two arguments) of `0x13d` and answers `level*4 + 4` when it says yes, `level + 2` when it says no.
A row draws the id (the three, their neighbours, random), the `is` answer and the level.
Row: `<id> <is answer> <level> <calls> <is handed 0> <is handed 1> <ecx is this> -> <eax>`.
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


ENTRY = 0x0061DAA0


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2038)
    vt = h.vtable({0x60: ("is", 2, 0), 0x124: ("level", 0, 0)})
    this = h.alloc(0x300)
    for i in range(300):
        tid = rng.choice((0x3D, 0x3E, 0x190, 0x3C, 0x3F, 0x18F, 0x191, 0x13D, 0)) if rng.random() < 0.7 \
            else rand32(rng)
        a = rng.choice((0, 0, 1, -1)) if rng.random() < 0.7 else rand32(rng)
        lvl = rng.choice((0, 1, 2, 3, 4, 5, -1)) if rng.random() < 0.7 else rand32(rng)
        h.answers["is"], h.answers["level"] = a, lvl
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x300)))
        h.w32(this, vt)
        h.w32(this + 4, tid)
        h.calls.clear()
        before = bytes(uc.mem_read(this, 0x300))
        res = h.call(ENTRY, (), ecx=this)
        popped(uc)
        assert bytes(uc.mem_read(this, 0x300)) == before, "wrote the object"
        isc = [c for c in h.calls if c[0] == "is"]
        handed = isc[0][2] if isc else [0, 0]
        names = "".join({"is": "1", "level": "2"}[c[0]] for c in h.calls) or "0"
        ecx_this = int(all(c[1] == this for c in h.calls))
        print(" ".join(str(x) for x in (tid, a, lvl, names, *handed, ecx_this)) + f" -> {res}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
