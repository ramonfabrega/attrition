#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_city_data_upgrades_to.py — `CityData::upgrades_to@007364f0` under unicorn (item 1998, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_city_data_upgrades_to.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`. `who` is the signed byte at `this+0x5e`, `o` the signed word at
`this+0x8`; the building is `objects.list[who][o]` (the per-player pointer at `0xc0ab84 + who * 0x1c`), its type
record the pointer at `+0x18`, and the `TypeIndex` at the record's `+4`: `METROPOLIS` (0x1a0) and
`FORBIDDENCITY` (0x213) answer -1, `VILLAGE` (0x19e) answers `TOWN` (0x19f), anything else `METROPOLIS`.
Every other byte of `this` is random; the rows hold `who` to 0..8 and `o` to 0..63.
Row: `<who> <o> <type index> -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x007364F0
TABLE = 0x00C0AB84
NAMED = (0x19E, 0x19F, 0x1A0, 0x213, 0x19D, 0x1A1, 0x212, 0x214, 0, -1)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(1998)
    lists = [h.alloc(4 * 64) for _ in range(9)]
    objs, recs, this = h.alloc(0x40 * 64), h.alloc(0x20 * 64), h.alloc(0x100)
    for who in range(9):
        uc.mem_write(TABLE + who * 0x1C, struct.pack("<I", lists[who]))
    for i in range(420):
        who, o = rng.randrange(9), rng.randrange(64)
        t = rng.choice(NAMED) if i % 3 else rng.randint(-0x80000000, 0x7FFFFFFF)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x100)))
        h.w8(this + 0x5E, who)
        h.w16(this + 0x8, o)
        obj, rec = objs + 0x40 * rng.randrange(64), recs + 0x20 * rng.randrange(64)
        uc.mem_write(obj, bytes(rng.randrange(256) for _ in range(0x40)))
        uc.mem_write(rec, bytes(rng.randrange(256) for _ in range(0x20)))
        h.w32(lists[who] + 4 * o, obj)
        h.w32(obj + 0x18, rec)
        h.w32(rec + 4, t)
        r = h.call(ENTRY, (), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        print(f"{who} {o} {t} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
