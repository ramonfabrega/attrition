#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_game_action_cheat_ai_toggle.py — `Game::action_cheat_ai_toggle@005930c0` under unicorn (item 1966, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_game_action_cheat_ai_toggle.py <install>/riseofnations.exe

`__thiscall`, `ecx` unused. The cell `0x00c061ec` points at the game record, whose byte at `+0x820` is the semaphore;
when bit 2 is clear it toggles the int the cell `0xc061c4` names (`ai_off`) between 0 and 1. Row: `<semaphore byte> <word before> -> <word after>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x005930c0
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    from callfn import Image, Machine, signed
    from sweeplib import REC, OUT, cell, setup
    import struct
    m = Machine(Image(argv[1]))
    uc = m.uc
    setup(uc)
    rng = random.Random(1966)
    for i in range(300):
        f = rng.randrange(256) if i >= 20 else (0, 4, 1, 0xFB, 0xFF)[i % 5]
        ai = rand32(rng) if i % 3 else (0, 1, 2)[i % 2]
        cell(uc, 0x00C061EC, REC)
        cell(uc, 0x00C061C4, OUT)
        uc.mem_write(REC + 0x820, bytes([f]))
        uc.mem_write(OUT, struct.pack("<i", ai))
        r = m.call(ENTRY, (), ecx=0)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(REC + 0x820, 1)) == bytes([f]), "the semaphore moved"
        print(f"{f} {ai} -> {struct.unpack('<i', uc.mem_read(OUT, 4))[0]}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
