#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_city_data_num_kinds_needed.py — `CityData::num_kinds_needed@00736640` under unicorn (item 2112,
hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_city_data_num_kinds_needed.py <install>/riseofnations.exe

`__thiscall`, no stack arguments. The city's own object, `objects[who][o]` — the per-player table at
`0xc0ab84 + 0x1c · (signed byte at +0x5e)`, slot `(signed 16-bit at +0x8)` — and its type record at `+0x18`, the
type at `+0x4`: `0x1a0` or `0x213` answers 0; otherwise the next level, `0x19f + (type != 0x19e)`, is asked of
`LeaderData::type_avail@006e33a0` (hooked, `(t, 1)`) on `[0xc061e0] + 0x6eec · who`; not available answers 0 (the
callee's own zero), available answers `city_buildings + 1` (`[0xc061e4] + 0x14c`) for `0x19f`, else
`metro_buildings + 1` (`+0x150`). The constants are drawn within ±0x10000.
Row: `<type> <who> <o> <leaders> <avail> <city> <metro> -> <eax> | <calls>`, a call `avail,<ecx>,<args…>`; the
city's 0x80 bytes, the object's 0x40, the record's 0x20 and the constants' 0x200 are random and held unwritten.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00736640
TYPE_AVAIL = 0x006E33A0
OBJECTS = 0x00C0AB84
CONSTANTS = 0x00C061E4
LEADERS = 0x00C061E0
TYPES = (0x19E, 0x19F, 0x1A0, 0x213, 0x19D, 0x1A1, 0x212, 0x214, 0, -1)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {TYPE_AVAIL: ("avail", 2)})
    uc = h.m.uc
    rng = random.Random(2112)
    this = h.alloc(0xA0)
    obj = h.alloc(0x40)
    rec = h.alloc(0x20)
    arr = h.alloc(0x400)
    consts = h.alloc(0x200)
    for i in range(400):
        ty = rng.choice(TYPES) if rng.random() < 0.85 else rng.randint(-0x80000000, 0x7FFFFFFF)
        who = rng.randint(-8, 15)
        o = rng.randint(0, 0xFF)
        leaders = rng.randint(0, 0xFFFFFFFF)
        avail = 0 if rng.random() < 0.4 else rng.choice((1, -1, rng.randint(-0x80000000, 0x7FFFFFFF)))
        city, metro = rng.randint(-0x10000, 0x10000), rng.randint(-0x10000, 0x10000)
        h.answers["avail"] = avail
        mem = bytearray(rng.randrange(256) for _ in range(0x80))
        mem[0x5E] = who & 0xFF
        mem[8:10] = struct.pack("<h", o)
        omem = bytearray(rng.randrange(256) for _ in range(0x40))
        omem[0x18:0x1C] = struct.pack("<I", rec)
        rmem = bytearray(rng.randrange(256) for _ in range(0x20))
        rmem[4:8] = struct.pack("<i", ty)
        cmem = bytearray(rng.randrange(256) for _ in range(0x200))
        cmem[0x14C:0x150] = struct.pack("<i", city)
        cmem[0x150:0x154] = struct.pack("<i", metro)
        uc.mem_write(this, bytes(mem))
        uc.mem_write(obj, bytes(omem))
        uc.mem_write(rec, bytes(rmem))
        uc.mem_write(consts, bytes(cmem))
        uc.mem_write(arr + 4 * o, struct.pack("<I", obj))
        uc.mem_write(OBJECTS + 0x1C * who, struct.pack("<I", arr))
        uc.mem_write(CONSTANTS, struct.pack("<I", consts))
        uc.mem_write(LEADERS, struct.pack("<I", leaders))
        held = ((this, 0xA0), (obj, 0x40), (rec, 0x20), (consts, 0x200))
        before = b"".join(bytes(uc.mem_read(a, n)) for a, n in held)
        h.calls.clear()
        r = h.call(ENTRY, (), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert b"".join(bytes(uc.mem_read(a, n)) for a, n in held) == before, "wrote memory"
        calls = ";".join(",".join([name, str(ecx & 0xFFFFFFFF), *map(str, args)]) for name, ecx, args, _ in h.calls)
        print(f"{ty} {who} {o} {leaders} {avail} {city} {metro} -> {r} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
