#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_city_data_get_farm_limit.py — `CityData::get_farm_limit@00738230` under unicorn (item 2090, hand-written
on tools/emu/hooks.py).

    uv run tools/emu/sweep_city_data_get_farm_limit.py <install>/riseofnations.exe

`__thiscall`, no stack arguments. The city's object is `objects[who][o]` — `who` the signed byte at `+0x5e`,
`o` the signed short at `+0x8`, the per-player list the word at `0xc0ab84 + 0x1c · who` — and its slot `+0x3c`
(`get_build`, a stub answering a building whose `+0x18` names the type record) gives the city's type index
(`+4`): `0x19f` level 2, `0x1a0` and `0x213` level 3, anything else 1. It answers
`LeaderData::get_farm_limit@006db270` (hooked, on `[0xc061e0] + 0x6eec · who`) `+ (level − 1) ·` the word at
`[0xc061e4] + 0x2a4` (`farms_per_city_level`), the multiply and the add wrapping.
Row: `<who> <o> <type> <leader limit> <per level> <leaders> -> <eax> | <calls>`, a call `name,ecx`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00738230
FARM_LIMIT = 0x006DB270
OBJECTS = 0x00C0AB84
LEADERS = 0x00C061E0
CONSTANTS = 0x00C061E4
PER_LEVEL = 0x2A4
TYPES = (0x19F, 0x1A0, 0x213, 0x19E, 0x1A1, 0x212, 0x214, 0, -1, 0x7FFFFFFF)
I32 = (0, 1, -1, 2, 0x7FFFFFFF, -0x80000000, 0x100, 3, 5, 10)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.6 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {FARM_LIMIT: ("get_farm_limit", 0)})
    uc = h.m.uc
    rng = random.Random(2090)
    city = h.alloc(0x80)
    lists = h.alloc(0x400)
    build = h.alloc(0x40)
    rec = h.alloc(0x20)
    vt = h.vtable({0x3C: ("get_build", 0, build)})
    obj = h.alloc(0x20)
    h.w32(obj, vt)
    consts = h.alloc(0x400)
    uc.mem_write(CONSTANTS, struct.pack("<I", consts))
    for i in range(400):
        who = rng.randint(0, 7) if rng.random() < 0.9 else rng.randint(-2, -1)
        o = rng.randint(0, 0xFF)
        t = rng.choice(TYPES) if rng.random() < 0.8 else rand32(rng)
        limit, per = rand32(rng), rand32(rng)
        leaders = rng.randint(0, 0xFFFFFFFF)
        h.answers["get_farm_limit"] = limit
        mem = bytearray(rng.randrange(256) for _ in range(0x60))
        mem[8:10] = struct.pack("<h", o)
        mem[0x5E] = who & 0xFF
        uc.mem_write(city, bytes(mem))
        uc.mem_write(OBJECTS + 0x1C * who, struct.pack("<I", lists))
        h.w32(lists + 4 * o, obj)
        h.w32(build + 0x18, rec)
        h.w32(rec + 4, t)
        h.w32(consts + PER_LEVEL, per)
        uc.mem_write(LEADERS, struct.pack("<I", leaders))
        before = bytes(uc.mem_read(city, 0x80))
        h.calls.clear()
        r = h.call(ENTRY, (), ecx=city)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(city, 0x80)) == before, "wrote the city"
        calls = ";".join(f"{name},{'obj' if ecx == obj else ecx & 0xFFFFFFFF}" for name, ecx, _, _ in h.calls)
        print(f"{who} {o} {t} {limit} {per} {leaders} -> {r} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
