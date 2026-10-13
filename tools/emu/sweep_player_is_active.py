#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_player_is_active.py — `Player::is_active@0047e0e0` under unicorn.

    uv run tools/emu/sweep_player_is_active.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the player; returns whether the 16-bit word at +0x30 has bit 0 set and neither 0x100 nor 0xd0 set. The rows run every low byte under six high bytes.
One row per input, `<word> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0047e0e0
OBJ = 0x10000000


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    for hi in (0, 1, 2, 3, 0x80, 0xFF):
        for lo in range(256):
            w = hi << 8 | lo
            uc.mem_write(OBJ, bytes([0xA5]) * 0x1000)
            uc.mem_write(OBJ + 0x30, struct.pack("<H", w))
            print(f"{w} -> {signed(m.call(ENTRY, (), ecx=OBJ))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
