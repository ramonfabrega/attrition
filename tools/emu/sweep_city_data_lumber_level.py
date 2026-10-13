#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_city_data_lumber_level.py — `CityData::lumber_level@00736820` under unicorn (item 2090, hand-written on
tools/emu/hooks.py; 1971's `sweep_city_data_granary_level.py` with the ladder inline).

    uv run tools/emu/sweep_city_data_lumber_level.py <install>/riseofnations.exe

`__thiscall`, no stack arguments. The 16-bit word at `+0x4` (`city_flags`), bit 0x400: clear answers 0; set asks
`LeaderData::has_preq@006db810` (hooked, one argument) on `[0xc061e0] + 0x6eec · (signed byte at +0x5e)` for
`LUMBERMILL4` (`0x2c4`), `0x2c3` and `0x2c2` from the top, answering 4, 3 at the first held and `1 + held` at
the last. The hook answers per argument (zero half the time).
Row: `<flags> <who> <leaders> <a 0x2c2> <a 0x2c3> <a 0x2c4> -> <eax> | <calls>`, a call `ecx,<arg>`; the
city's 0x80 bytes are random and held unwritten.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00736820
HAS_PREQ = 0x006DB810
LEADERS = 0x00C061E0
ASKED = (0x2C2, 0x2C3, 0x2C4)
I32 = (1, -1, 2, 0x7FFFFFFF, -0x80000000, 0x100, 0x10000)


def word(rng):
    if rng.random() < 0.5:
        return 0
    return rng.choice(I32) if rng.random() < 0.5 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {HAS_PREQ: ("has_preq", 1)})
    uc = h.m.uc
    rng = random.Random(2090)
    answers = {}
    h.answers["has_preq"] = lambda ecx, args: answers.get(args[0], 0)
    this = h.alloc(0xA0)
    for i in range(400):
        answers.clear()
        for t in ASKED:
            answers[t] = word(rng)
        flags = rng.randrange(0x10000) if i % 4 else (0x400, 0, 0x3FF, 0xFBFF)[i % 3]
        who = rng.randint(-128, 127)
        leaders = rng.randint(0, 0xFFFFFFFF)
        mem = bytearray(rng.randrange(256) for _ in range(0x80))
        mem[4:6] = struct.pack("<H", flags)
        mem[0x5E] = who & 0xFF
        uc.mem_write(this, bytes(mem))
        uc.mem_write(LEADERS, struct.pack("<I", leaders))
        before = bytes(uc.mem_read(this, 0xA0))
        h.calls.clear()
        r = h.call(ENTRY, (), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(this, 0xA0)) == before, "wrote the city"
        calls = ";".join(f"{ecx & 0xFFFFFFFF},{args[0]}" for _, ecx, args, _ in h.calls)
        print(f"{flags} {who} {leaders} " + " ".join(str(answers[t]) for t in ASKED) + f" -> {r} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
