#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_turn_speed.py — `UnitData::turn_speed@0060a600` under unicorn (item 1976, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_unit_data_turn_speed.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the unit data: the type record at `+0x18` (a word at its `+0x2c4`), the mask word at `+0x68`.
The constants record is the pointer in the global cell `0xc061e4`: `unit_turn_speed` at `+8`, `unit_pack_turn_bonus` at `+0xc`.
Answers `(word >> 8) * turn`, times the bonus when mask bit `0x80000` is set; 32-bit wrapping multiplies.
Row: `<word> <turn> <bonus> <masks> -> <eax>`, the word and masks unsigned.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0060a600
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def pop_ok(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(1976)
    for i in range(400):
        this, typ, con = h.alloc(0x100), h.alloc(0x400), h.alloc(0x40)
        word = rng.getrandbits(32) if i % 3 else rng.choice((0, 0xFF, 0x100, 0x1FF, 0xFFFFFFFF, 0x80000000))
        turn, bonus = rand32(rng), rand32(rng)
        masks = rng.getrandbits(32)
        if i % 2:
            masks |= 0x80000
        else:
            masks &= ~0x80000 & 0xFFFFFFFF
        h.w32(this + 0x18, typ)
        h.w32(typ + 0x2C4, word)
        uc.mem_write(0x00C061E4, struct.pack("<I", con))
        h.w32(con + 8, turn)
        h.w32(con + 0xC, bonus)
        h.w32(this + 0x68, masks)
        r = h.call(ENTRY, (), ecx=this)
        print(f"{word} {turn} {bonus} {masks} -> {r}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
