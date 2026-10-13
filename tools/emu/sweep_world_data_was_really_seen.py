#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_data_was_really_seen.py — `WorldData::was_really_seen@006b54f0` under unicorn (item 2044,
hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_world_data_was_really_seen.py <install>/riseofnations.exe

`__thiscall` with three stack arguments (two FCoord pointers `x`, `y`, and `who`), `ret 0xc`. `who >= 8`
(signed) is 1; so is a game record (the cell `0xc061e8`) whose byte `+0x30` is 3, and a leader (the cell
`0xc061e0` names the list, `0x6eec` a leader) whose word has `0x800` or whose short at `+0x59e4` is nonzero.
Else the answer is the fog byte `[this+0x160][x + [this+0xc]*y]` and the leader's byte at `+0x6929`. A row
draws `who` 0..9 (a negative one would index before the list: not drawn), each exit's field often at its
trigger, a small fog width and a cell inside it.
Row: `<x> <y> <who> <reveal> <leader word> <+0x59e4> <+0x6929> <fog width> <fog byte> -> <eax>`.
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


ENTRY = 0x006B54F0
GAME, LEADERS = 0x00C061E8, 0x00C061E0
LEADER = 0x6EEC
FOG = 64 * 64


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2044)
    saved = bytes(uc.mem_read(GAME, 4)), bytes(uc.mem_read(LEADERS, 4))
    game = h.alloc(0x40)
    leaders = h.alloc(8 * LEADER)
    this = h.alloc(0x170)
    fog = h.alloc(FOG)
    xy = h.alloc(8)
    h.w32(GAME, game)
    h.w32(LEADERS, leaders)
    for _ in range(300):
        who = rng.randrange(10)
        reveal = 3 if rng.random() < 0.15 else rng.randrange(256)
        uc.mem_write(game, bytes(rng.randrange(256) for _ in range(0x40)))
        h.w8(game + 0x30, reveal)
        lw = rand32(rng)
        if rng.random() < 0.7:
            lw &= ~0x800
        l59 = 0 if rng.random() < 0.8 else rng.randint(-0x8000, 0x7FFF)
        mask = rng.randrange(256)
        if who < 8:
            L = leaders + who * LEADER
            h.w32(L, lw)
            h.w16(L + 0x59E4, l59)
            h.w8(L + 0x6929, mask)
        xs = rng.randrange(1, 65)
        x, y = rng.randrange(xs), rng.randrange(FOG // 64 if xs <= 64 else 1)
        y = min(y, FOG // xs - 1)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x170)))
        h.w32(this + 0xC, xs)
        h.w32(this + 0x160, fog)
        uc.mem_write(fog, bytes(rng.randrange(256) for _ in range(FOG)))
        cell = uc.mem_read(fog + x + xs * y, 1)[0]
        h.w32(xy, x)
        h.w32(xy + 4, y)
        before = bytes(uc.mem_read(this, 0x170)), bytes(uc.mem_read(fog, FOG))
        res = h.call(ENTRY, (xy, xy + 4, who), ecx=this)
        popped(uc)
        assert (bytes(uc.mem_read(this, 0x170)), bytes(uc.mem_read(fog, FOG))) == before, "wrote"
        print(f"{x} {y} {who} {reveal} {lw} {l59} {mask} {xs} {cell} -> {res}")
    uc.mem_write(GAME, saved[0])
    uc.mem_write(LEADERS, saved[1])
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
