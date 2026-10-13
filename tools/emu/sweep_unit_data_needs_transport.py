#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_needs_transport.py — `UnitData::needs_transport@00609920` under unicorn (item 2038,
hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_unit_data_needs_transport.py <install>/riseofnations.exe

`__thiscall` with four stack arguments, two tile coordinates `(x1, y1, x2, y2)`, `ret 0x10`; `this` is never
read. The same tile answers 0 without a read; otherwise each tile's mask (a `u16`, row-major, the world through
the global cell `0xc061d0`: width at `+0x18`, the masks' pointer at `+0x138`) is ocean when its `0x30` field is
`0x20`, and the answer is 0 when both are or neither is, 1 from ocean to land, 2 from land to ocean. A row lays a
16 by 16 map of random masks, a quarter of them forced ocean, and draws the two tiles.
Row: `<x1> <y1> <x2> <y2> <mask 1> <mask 2> -> <eax>`.
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


ENTRY = 0x00609920
CELL = 0x00C061D0
W = 16


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2038)
    world, masks = h.alloc(0x200), h.alloc(W * W * 2)
    saved = h.r32(CELL)
    uc.mem_write(CELL, struct.pack("<I", world))
    for i in range(300):
        uc.mem_write(world, bytes(rng.randrange(256) for _ in range(0x200)))
        h.w32(world + 0x18, W)
        h.w32(world + 0x138, masks)
        cells = []
        for _ in range(W * W):
            m = rng.randrange(0x10000)
            if rng.random() < 0.25:
                m = (m & ~0x30) | 0x20
            cells.append(m)
        uc.mem_write(masks, struct.pack(f"<{W * W}H", *cells))
        x1, y1 = rng.randrange(W), rng.randrange(W)
        x2, y2 = (x1, y1) if rng.random() < 0.1 else (rng.randrange(W), rng.randrange(W))
        if rng.random() < 0.1:
            x2 = x1
        before = bytes(uc.mem_read(world, 0x200)), bytes(uc.mem_read(masks, W * W * 2))
        res = h.call(ENTRY, (x1, y1, x2, y2), ecx=rand32(rng) & 0xFFFFFFFF)
        popped(uc)
        assert (bytes(uc.mem_read(world, 0x200)), bytes(uc.mem_read(masks, W * W * 2))) == before, "wrote"
        m1, m2 = cells[y1 * W + x1], cells[y2 * W + x2]
        print(f"{x1} {y1} {x2} {y2} {m1} {m2} -> {res}")
    uc.mem_write(CELL, struct.pack("<i", saved))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
