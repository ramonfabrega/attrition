#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_city_data_granary_level.py — `CityData::granary_level@00736890` under unicorn (item 1971, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_city_data_granary_level.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the city; the word at `+0x4` (`city_flags`), bit 0x200: clear answers 0, set tail-calls
`LeaderData::get_granary@006db340` (hooked) with `ecx = [0xc061e0] + 0x6eec * (signed byte at +0x5e)` and answers what it answers.
Row: `<flags> <who> <leaders base> <get_granary answer> -> <eax> | <calls: ecx>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00736890
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def pop_ok(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


GRANARY = 0x006DB340


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {GRANARY: ("granary", 0)})
    uc = h.m.uc
    rng = random.Random(1971)
    for i in range(400):
        this = h.alloc(0x80)
        flags = rng.randrange(0x10000) if i % 4 else (0x200, 0, 0x1FF, 0xFDFF)[i % 3]
        who = rng.randint(-128, 127)
        base = rand32(rng) & 0xFFFFFFFF
        ans = rand32(rng)
        h.w32(this + 4, flags)
        h.m.uc.mem_write(this + 0x5E, bytes([who & 0xFF]))
        uc.mem_write(0x00C061E0, base.to_bytes(4, "little"))
        h.answers["granary"] = ans & 0xFFFFFFFF
        h.calls.clear()
        r = h.call(ENTRY, (), ecx=this)
        pop_ok(uc)
        seen = ";".join(str(ecx & 0xFFFFFFFF) for _, ecx, _, _ in h.calls)
        print(f"{flags} {who} {base} {ans} -> {r} | {seen}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
