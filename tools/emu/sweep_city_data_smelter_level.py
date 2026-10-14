#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_city_data_smelter_level.py — `CityData::smelter_level@007367a0` under unicorn (item 2112, hand-written on
tools/emu/hooks.py; 2090's `sweep_city_data_lumber_level.py` with a jump table in front).

    uv run tools/emu/sweep_city_data_smelter_level.py <install>/riseofnations.exe

`__thiscall`, no stack arguments. The good-type list at `[0xc096f4] + 0x10`, its fifth record (`[list + 0x10]`),
the kind at `+0x4`: compared unsigned against 5 and dispatched through the table at `0x7367fc` — 0 `GRANARY`
(0x1a7), 1 `LUMBERMILL` (0x1a8), 4 `SMELTER` (0x1a9), 5 `REFINERY` (0x1aa), anything else answers 0. The type is
handed to `CityData::count_buildings@00739390` (hooked, `this`, three arguments `(t, 0, 1)`); zero answers 0, else
the function jumps into `LeaderData::get_smelter@006db3f0` (hooked) on `[0xc061e0] + 0x6eec · (signed byte at
+0x5e)`, whose answer is the answer.
Row: `<kind> <who> <leaders> <count> <smelter> -> <eax> | <calls>`, a call `count,this|other,<args…>` or
`smelter,<ecx>`; the city's 0x80 bytes, the list's and the record's 0x20 are random and held unwritten.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x007367A0
COUNT_BUILDINGS = 0x00739390
GET_SMELTER = 0x006DB3F0
GOODTYPES = 0x00C096F4
LEADERS = 0x00C061E0
I32 = (1, -1, 2, 4, 0x7FFFFFFF, -0x80000000, 0x100, 0x10000)


def word(rng):
    if rng.random() < 0.4:
        return 0
    return rng.choice(I32) if rng.random() < 0.5 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {COUNT_BUILDINGS: ("count", 3), GET_SMELTER: ("smelter", 0)})
    uc = h.m.uc
    rng = random.Random(2112)
    this = h.alloc(0xA0)
    lst = h.alloc(0x40)
    rec = h.alloc(0x40)
    for i in range(400):
        kind = rng.randint(-2, 7) if rng.random() < 0.85 else rng.randint(-0x80000000, 0x7FFFFFFF)
        who = rng.randint(-128, 127)
        leaders = rng.randint(0, 0xFFFFFFFF)
        count, smelter = word(rng), word(rng)
        h.answers["count"], h.answers["smelter"] = count, smelter
        mem = bytearray(rng.randrange(256) for _ in range(0x80))
        mem[0x5E] = who & 0xFF
        lmem = bytearray(rng.randrange(256) for _ in range(0x20))
        lmem[0x10:0x14] = struct.pack("<I", rec)
        rmem = bytearray(rng.randrange(256) for _ in range(0x20))
        rmem[4:8] = struct.pack("<i", kind)
        uc.mem_write(this, bytes(mem))
        uc.mem_write(lst, bytes(lmem))
        uc.mem_write(rec, bytes(rmem))
        uc.mem_write(GOODTYPES, struct.pack("<I", lst))
        uc.mem_write(LEADERS, struct.pack("<I", leaders))
        before = bytes(uc.mem_read(this, 0xA0)) + bytes(uc.mem_read(lst, 0x20)) + bytes(uc.mem_read(rec, 0x20))
        h.calls.clear()
        r = h.call(ENTRY, (), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        after = bytes(uc.mem_read(this, 0xA0)) + bytes(uc.mem_read(lst, 0x20)) + bytes(uc.mem_read(rec, 0x20))
        assert after == before, "wrote memory"
        calls = ";".join(
            ",".join(["count", "this" if ecx == this else "other", *map(str, args)]) if name == "count"
            else f"smelter,{ecx & 0xFFFFFFFF}"
            for name, ecx, args, _ in h.calls)
        print(f"{kind} {who} {leaders} {count} {smelter} -> {r} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
