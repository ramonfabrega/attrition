#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_data_is_enemy_territory.py — `WorldData::is_enemy_territory@006b2490` under unicorn (item 2112,
hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_world_data_is_enemy_territory.py <install>/riseofnations.exe

`__thiscall` (`this` unread), three pointer arguments `(WCoord *x, WCoord *y, int *who)`, `ret 0xc`. The world at
`[0xc06188]`: the cell record `[world + 0x134] + 0x1c · (y · [world] + x)`, 32-bit wrapping, and its owner the
signed byte at `+0xf`. A negative owner, or the owner `== who`, answers 0; then, on the leader
`L = [0xc061e0] + 0x6eec · who`, the owner `== [L + 8]` (`me`, the leader's own `who`) answers 0, and the inlined
`is_ally`: `L.diplos[owner]` (`+0x74`) `== 2` and the owner's leader's `diplos[me] == 2` answers 0, anything else
1. The cell table and the leader table are placed a row so the reads land in the script's buffers: `x`, `y` and
the width are drawn freely, `who` within -2..15, `me` within -4..15 and the owner byte over -128..15.
Row: `<x> <y> <width> <owner> <who> <me> <d1> <d2> -> <eax>`, `d1` the word at `L.diplos[owner]` and `d2` at the
owner's `diplos[me]` (read back, random where unwritten); the three arguments, the world's 0x200 bytes, the
cell buffer and the leader table are held unwritten.
"""
import hashlib
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x006B2490
WORLD = 0x00C06188
LEADERS = 0x00C061E0
STRIDE = 0x6EEC
D = (2, 2, 2, 0, 1, 3, -1)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2112)
    world = h.alloc(0x200)
    cell = h.alloc(0x40)
    args = h.alloc(0x20)
    table = h.alloc(19 * STRIDE)
    uc.mem_write(table, bytes(rng.randrange(256) for _ in range(19 * STRIDE)))
    lbase = table + 2 * STRIDE
    for i in range(400):
        x, y, width = (rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(3))
        if rng.random() < 0.5:
            x, y, width = rng.randint(0, 255), rng.randint(0, 255), rng.randint(1, 256)
        who = rng.randint(-2, 15)
        me = who if rng.random() < 0.5 else rng.randint(-4, 15)
        r_ = rng.random()
        owner = who if r_ < 0.15 else me if r_ < 0.3 else rng.randint(-128, -1) if r_ < 0.4 else rng.randint(0, 15)
        idx = (y * width + x) & 0xFFFFFFFF
        cells = (cell - 0x1C * idx) & 0xFFFFFFFF
        wmem = bytearray(rng.randrange(256) for _ in range(0x200))
        wmem[0:4] = struct.pack("<i", width)
        wmem[0x134:0x138] = struct.pack("<I", cells)
        cmem = bytearray(rng.randrange(256) for _ in range(0x40))
        cmem[0xF] = owner & 0xFF
        uc.mem_write(world, bytes(wmem))
        uc.mem_write(cell, bytes(cmem))
        uc.mem_write(args, struct.pack("<iii", x, y, who))
        uc.mem_write(WORLD, struct.pack("<I", world))
        uc.mem_write(LEADERS, struct.pack("<I", lbase))
        L = lbase + STRIDE * who
        h.w32(L + 8, me)
        if owner >= 0:
            h.w32(L + 0x74 + 4 * owner, rng.choice(D))
            h.w32(lbase + STRIDE * owner + 0x74 + 4 * me, rng.choice(D))
            d1 = h.r32(L + 0x74 + 4 * owner)
            d2 = h.r32(lbase + STRIDE * owner + 0x74 + 4 * me)
        else:
            d1 = d2 = 0
        held = ((world, 0x200), (cell, 0x40), (args, 0x20), (table, 19 * STRIDE))
        before = hashlib.sha1(b"".join(bytes(uc.mem_read(a, n)) for a, n in held)).digest()
        r = h.call(ENTRY, (args, args + 4, args + 8), ecx=0)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        after = hashlib.sha1(b"".join(bytes(uc.mem_read(a, n)) for a, n in held)).digest()
        assert after == before, "wrote memory"
        print(f"{x} {y} {width} {owner} {who} {me} {d1} {d2} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
