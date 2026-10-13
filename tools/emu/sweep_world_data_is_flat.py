#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_data_is_flat.py — `WorldData::is_flat@006b3120` under unicorn (item 2044, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_world_data_is_flat.py <install>/riseofnations.exe

`__thiscall` with two stack arguments (WCoord pointers `x`, `y`), `ret 8`; `this` is never read. The world (the
cell `0xc06188`: width at `+0`, cells of `0x1c` bytes at `[+0x134]`) gives the cell's flag short; `& 0x30` is
0, unasked; else the callee at `0x6b4830` (hooked, two stack arguments, `ecx` = the world) is asked with the
same two pointers, a nonzero answer is 0, and else `& 0x48` decides: clear is 1. A row draws a small world, a
cell inside it, its flag short with each bit often clear, and the callee's answer.
Row: `<width> <x> <y> <flags> <answer> -> <eax> | <calls>`, the call `<ecx is the world>,<a0 is x>,<a1 is y>`.
"""
import os
import random
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


ENTRY = 0x006B3120
CALLEE = 0x006B4830
WORLD = 0x00C06188
H = 8


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {CALLEE: ("ocean", 2)})
    uc = h.m.uc
    rng = random.Random(2044)
    saved = bytes(uc.mem_read(WORLD, 4))
    world = h.alloc(0x140)
    cells = h.alloc(8 * H * 0x1C)
    xy = h.alloc(8)
    h.w32(WORLD, world)
    for _ in range(300):
        w = rng.randrange(1, 9)
        uc.mem_write(world, bytes(rng.randrange(256) for _ in range(0x140)))
        h.w32(world, w)
        h.w32(world + 0x134, cells)
        n = w * H * 0x1C
        uc.mem_write(cells, bytes(rng.randrange(256) for _ in range(n)))
        x, y = rng.randrange(w), rng.randrange(H)
        flags = rng.randrange(0x10000)
        for bit in (0x10, 0x20, 0x8, 0x40):
            if rng.random() < 0.6:
                flags &= ~bit
        h.w16(cells + (x + w * y) * 0x1C, flags - 0x10000 if flags & 0x8000 else flags)
        ans = 0 if rng.random() < 0.6 else rand32(rng)
        h.answers["ocean"] = ans & 0xFFFFFFFF
        h.w32(xy, x)
        h.w32(xy + 4, y)
        before = bytes(uc.mem_read(cells, n)), bytes(uc.mem_read(xy, 8))
        h.calls.clear()
        res = h.call(ENTRY, (xy, xy + 4), ecx=rand32(rng) & 0xFFFFFFFF)
        popped(uc)
        assert (bytes(uc.mem_read(cells, n)), bytes(uc.mem_read(xy, 8))) == before, "wrote"
        calls = ";".join(f"{int(ecx == world)},{int(args[0] == xy)},{int(args[1] == xy + 4)}"
                         for _, ecx, args, _ in h.calls)
        print(f"{w} {x} {y} {flags} {ans} -> {res} | {calls}")
    uc.mem_write(WORLD, saved)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
