#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_data_is_seen.py — `WorldData::is_seen@006b55c0`, `WorldData::is_seen_wcoord@006b2540` and
`WorldData::is_really_seen@006b42c0` under unicorn (item 2837, hand-written on tools/emu/hooks.py beside sweep_world_data_was_really_seen.py).

    uv run tools/emu/sweep_world_data_is_seen.py <install>/riseofnations.exe

Both are `__thiscall` with three stack arguments (two coordinate pointers and `who`), `ret 0xc`. `who >= 8`
(signed) is 1, so is a game record (cell `0xc061e8`) whose byte `+0x30` is 3 and a leader (cell `0xc061e0` names
the list, stride `0x6eec`) whose flags have `0x800` or whose short at `+0x59e4` is nonzero. Else, with flag
`0x2000`, a cell owner >= 0 that is the leader itself or both ways diplo 2 answers 1; else the fog byte
masked by the leader's byte at `+0x6929`. `is_really_seen` (form 2) is the FCoord form without the `reveal_map` exit, reading the world through the
global cell `0xc061d0` rather than `this` (`ecx` is a decoy there). The FCoord form reads the owner at wdata[(y>>1)*xs+(x>>1)] and fog
at `seen[x + fog_xs*y]` (`this+0x15c`); the WCoord form the owner at wdata[x+y*xs] and `wcoord_seen`
(`this+0x168`) at `x + y*xs`. A cell owner of 8 or more would index past the eight leaders: not drawn.
Row: `<form> <x> <y> <who> <reveal> <flags> <nu> <mask> <leader.who> <d_lc> <d_cl> <owner> <fog byte> -> <eax>`
where d_lc is `leader.diplos[owner]` and d_cl `list[owner].diplos[leader.who]` (0 where the owner is not 0..7).
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402


def popped(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


ENTRIES = {0: 0x006B55C0, 1: 0x006B2540, 2: 0x006B42C0}
GAME, LEADERS, WORLD = 0x00C061E8, 0x00C061E0, 0x00C061D0
LEADER = 0x6EEC
FOG = 4096
WD = 0x1C


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2837)
    saved = bytes(uc.mem_read(GAME, 4)), bytes(uc.mem_read(LEADERS, 4)), bytes(uc.mem_read(WORLD, 4))
    game = h.alloc(0x40)
    leaders = h.alloc(8 * LEADER)
    this = h.alloc(0x170)
    decoy = h.alloc(0x170)
    fog = h.alloc(FOG)
    wd = h.alloc(WD * 1024)
    xy = h.alloc(8)
    h.w32(GAME, game)
    h.w32(LEADERS, leaders)
    for i in range(600):
        form = i % 3
        who = rng.randrange(10)
        reveal = 3 if rng.random() < 0.1 else rng.randrange(256)
        uc.mem_write(game, bytes(rng.randrange(256) for _ in range(0x40)))
        h.w8(game + 0x30, reveal)
        for p in range(8):
            L = leaders + p * LEADER
            uc.mem_write(L, bytes(rng.randrange(256) for _ in range(0x200)))
            h.w32(L, rng.randrange(1 << 32) & ~0x800 & ~(0x2000 if rng.random() < 0.3 else 0)
                  | (0x2000 if rng.random() < 0.5 else 0))
            h.w32(L + 8, p if rng.random() < 0.85 else rng.randrange(8))
            for q in range(8):
                h.w32(L + 0x74 + 4 * q, 2 if rng.random() < 0.6 else rng.randrange(4))
            h.w16(L + 0x59E4, 0)
            h.w8(L + 0x6929, rng.randrange(256))
        if who < 8:
            L = leaders + who * LEADER
            if rng.random() < 0.15:
                h.w32(L, rng.randrange(1 << 32) | 0x800)
            if rng.random() < 0.15:
                h.w16(L + 0x59E4, rng.randint(1, 0x7FFF))
        xs = rng.randrange(4, 17)
        fx = xs if form == 1 else 2 * xs
        x, y = rng.randrange(fx), rng.randrange(min(fx, FOG // fx))
        cx, cy = (x, y) if form == 1 else (x >> 1, y >> 1)
        owner = rng.choice([-1, -3, 0, 1, 2, 3, 4, 5, 6, 7, 7])
        uc.mem_write(wd, bytes(rng.randrange(256) for _ in range(WD * 1024)))
        h.w8(wd + WD * (cx + cy * xs) + 0xF, owner & 0xFF)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x170)))
        h.w32(this + 0x0, xs)
        h.w32(this + 0xC, fx)
        h.w32(this + 0x134, wd)
        h.w32(this + 0x15C, fog)
        h.w32(this + 0x168, fog)
        uc.mem_write(fog, bytes(rng.randrange(256) for _ in range(FOG)))
        cell = uc.mem_read(fog + x + fx * y, 1)[0]
        h.w32(xy, x)
        h.w32(xy + 4, y)
        lw = lwho = d_lc = d_cl = nu = mask = 0
        if who < 8:
            L = leaders + who * LEADER
            lw = int.from_bytes(uc.mem_read(L, 4), "little")
            lwho = int.from_bytes(uc.mem_read(L + 8, 4), "little", signed=True)
            nu = int.from_bytes(uc.mem_read(L + 0x59E4, 2), "little")
            mask = uc.mem_read(L + 0x6929, 1)[0]
            if 0 <= owner < 8:
                d_lc = int.from_bytes(uc.mem_read(L + 0x74 + 4 * owner, 4), "little", signed=True)
                if 0 <= lwho < 8:
                    d_cl = int.from_bytes(uc.mem_read(leaders + owner * LEADER + 0x74 + 4 * lwho, 4),
                                          "little", signed=True)
        before = bytes(uc.mem_read(this, 0x170)), bytes(uc.mem_read(fog, FOG)), bytes(uc.mem_read(wd, WD * 1024))
        h.w32(WORLD, this)
        res = h.call(ENTRIES[form], (xy, xy + 4, who), ecx=decoy if form == 2 else this)
        popped(uc)
        assert (bytes(uc.mem_read(this, 0x170)), bytes(uc.mem_read(fog, FOG)),
                bytes(uc.mem_read(wd, WD * 1024))) == before, "wrote"
        lw = lw - (1 << 32) if lw >= 1 << 31 else lw
        print(f"{form} {x} {y} {who} {reveal} {lw} {nu} {mask} {lwho} {d_lc} {d_cl} {owner} {cell} -> {res}")
    uc.mem_write(GAME, saved[0])
    uc.mem_write(LEADERS, saved[1])
    uc.mem_write(WORLD, saved[2])
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
