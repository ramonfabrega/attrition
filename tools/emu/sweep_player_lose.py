#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_player_lose.py — `Player::lose@006ee250` under unicorn (item 2020, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_player_lose.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`. It ORs 0x80 into the word of flags at `this+0x30`, calls the
player's `end_game` (0x6ee110, hooked, no arguments, `ecx` = `this`), and then, unless flag 0x40 is set, plays a
sound (0x97f770, hooked, one argument, `ret 4`) with `ecx` = the unsigned byte at `this+0x36`: 0x7f when that
byte equals the int at `+0x2a0` of the console the global cell `0xc06210` names, else 0x66. The flags, the
byte and the console's int (equal to the byte a third of the time) are drawn per row; every byte of `this`
past the flags word is checked untouched.
Row: `<flags> <play byte> <console play> <flags after> <end_game calls, ecx this> <sound calls> <sound ecx>
<sound arg> -> <eax>` — the last two 0 with no sound.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x006EE250
CALLEES = {0x006EE110: ("end_game", 0), 0x0097F770: ("sound", 1)}
CONSOLE_CELL = 0x00C06210
WIN = 0x80


def main(argv):
    from hooks import Harness
    rng = random.Random(2020)
    h = Harness(argv[1], CALLEES)
    h.answers["end_game"] = lambda ecx, args: rng.randint(-0x80000000, 0x7FFFFFFF)
    h.answers["sound"] = lambda ecx, args: rng.randint(-0x80000000, 0x7FFFFFFF)
    uc = h.m.uc
    this, console = h.alloc(WIN), h.alloc(0x300)
    uc.mem_write(CONSOLE_CELL, struct.pack("<I", console))
    for i in range(300):
        flags = rng.randrange(0x10000)
        play = rng.randrange(256) if i % 4 else rng.choice((0, 1, 7, 8, 0x7F, 0x80, 0xFF))
        cp = play if rng.random() < 0.34 else rng.choice((rng.randrange(256), rng.randint(-0x80000000, 0x7FFFFFFF),
                                                          play - 256, play + 256))
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(WIN)))
        h.w16(this + 0x30, flags - 0x10000 if flags & 0x8000 else flags)
        h.w8(this + 0x36, play)
        h.w32(console + 0x2A0, cp)
        before = bytes(uc.mem_read(this, WIN))
        h.calls.clear()
        r = h.call(ENTRY, (), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        after = bytes(uc.mem_read(this, WIN))
        assert before[:0x30] + before[0x32:] == after[:0x30] + after[0x32:], "wrote past the flags"
        fa = struct.unpack("<H", after[0x30:0x32])[0]
        eg = [c for c in h.calls if c[0] == "end_game"]
        snd = [c for c in h.calls if c[0] == "sound"]
        assert h.calls and h.calls[0][0] == "end_game", "end_game first"
        sec, sarg = (snd[0][1], snd[0][2][0]) if snd else (0, 0)
        print(" ".join(str(x) for x in (flags, play, cp, fa, int(len(eg) == 1 and eg[0][1] == this),
                                        len(snd), sec, sarg)) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
