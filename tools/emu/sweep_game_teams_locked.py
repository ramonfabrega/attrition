#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_game_teams_locked.py — `Game::teams_locked@00594880` under unicorn (item 1971, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_game_teams_locked.py <install>/riseofnations.exe

`__thiscall`, `ecx` unused. The cell `0x00c061ec` points at the game record, whose byte at `+0x24` is the team style;
0, 8 and 0xb answer 0, any other style 1. Row: `<style byte> -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00594880
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def pop_ok(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def main(argv):
    from callfn import Image, Machine
    from sweeplib import REC, cell, setup
    m = Machine(Image(argv[1]))
    uc = m.uc
    setup(uc)
    for b in list(range(256)) * 2:
        cell(uc, 0x00C061EC, REC)
        uc.mem_write(REC + 0x24, bytes([b]))
        r = m.call(ENTRY, (), ecx=0)
        pop_ok(uc)
        print(f"{b} -> {r}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
