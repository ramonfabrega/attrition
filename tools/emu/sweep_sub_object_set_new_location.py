#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_sub_object_set_new_location.py — `SubObject::set_new_location@00662680` under unicorn (item 2044,
hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_sub_object_set_new_location.py <install>/riseofnations.exe

`__thiscall` with two stack arguments (the coordinates `x`, `y`), `ret 8`. It stores `x ^ 0x63637` at
`this+0x10` and `y ^ 0x63637` at `this+0x14`, then calls the callee at `0x8544a0` (hooked, four stack
arguments, `ecx` = the terrain cell `0xc06218`'s word) with a pointer to a frame word, the table words
(`[0xcae5fc]`, a table) at `x >> 6` and `y >> 6`, and 1; the word the callee leaves there, `^ 0x63637`, goes
to `this+0xc`; the answer is 1. The hook writes a drawn word through its first argument.
Row: `<x> <y> <z> <terrain> -> <eax> | <ecx>,<a1>,<a2>,<a3> | <this+0xc> <this+0x10> <this+0x14>`.
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


def signed(v):
    return v - (1 << 32) if v & 0x80000000 else v


def tab(i):
    return (i * 2654435761 >> 5) & 0xFFFF


ENTRY = 0x00662680
CALLEE = 0x008544A0
DIV3, TERRAIN = 0x00CAE5FC, 0x00C06218
N = 1024


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {CALLEE: ("z", 4)})
    uc = h.m.uc
    rng = random.Random(2044)
    saved = {a: bytes(uc.mem_read(a, 4)) for a in (DIV3, TERRAIN)}
    table = h.alloc(4 * N)
    uc.mem_write(table, b"".join(struct.pack("<I", tab(i)) for i in range(N)))
    h.w32(DIV3, table)
    this = h.alloc(0x40)
    z = [0]

    def answer(ecx, args):
        h.w32(args[0] & 0xFFFFFFFF, z[0])
        return rand32(rng) & 0xFFFFFFFF

    h.answers["z"] = answer
    for _ in range(300):
        x, y = rng.randrange(N << 6), rng.randrange(N << 6)
        if rng.random() < 0.1:
            x = rng.choice((0, 0x3F, 0x40, (N << 6) - 1))
        z[0] = rand32(rng)
        terrain = rand32(rng) & 0xFFFFFFFF
        h.w32(TERRAIN, terrain)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x40)))
        before = bytearray(uc.mem_read(this, 0x40))
        h.calls.clear()
        res = h.call(ENTRY, (x, y), ecx=this)
        popped(uc)
        after = bytearray(uc.mem_read(this, 0x40))
        fields = [h.r32(this + o) for o in (0xC, 0x10, 0x14)]
        before[0xC:0x18] = after[0xC:0x18]
        assert before == after, "wrote outside +0xc..+0x18"
        assert len(h.calls) == 1, h.calls
        _, ecx, args, _ = h.calls[0]
        calls = ",".join(str(v) for v in (signed(ecx), *args[1:]))
        print(f"{x} {y} {z[0]} {signed(terrain)} -> {res} | {calls} | {' '.join(map(str, fields))}")
    for a, v in saved.items():
        uc.mem_write(a, v)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
