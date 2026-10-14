#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_is_flying_high.py — `UnitData::is_flying_high@0060a310` under unicorn (item 2090, hand-written
on tools/emu/hooks.py).

    uv run tools/emu/sweep_unit_data_is_flying_high.py <install>/riseofnations.exe

`__thiscall`, no stack arguments. The type record at `+0x18`: domain `+0x218` must be 2, the byte `+0x2b4` must
not carry 0x20 and the word `+0x1e4` not 0x8000000 (a fixed-wing plane), else 0 unasked. Then on the map: the
unit's slot `+0xbc` against `is_on_map` (`0x46ce30`) — equal, bit 15 of the word at `+0x82`, else the slot asked
(a stub answered per row). On the map, `UnitData::is_flying_low@0060a140` (hooked) is asked and its answer
negated; off it, 0.
Row: `<base 0|1> <domain> <byte 2b4> <word 1e4> <word 82> <slot answer> <low answer> -> <eax> | <calls>`, a call
`name,unit|other`; the unit's 0x100 bytes and the record's 0x300 are random and held unwritten.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0060A310
IS_ON_MAP = 0x0046CE30
IS_FLYING_LOW = 0x0060A140
I32 = (0, 1, -1, 2, 0x7FFFFFFF, -0x80000000, 0x100)


def word(rng):
    if rng.random() < 0.5:
        return 0
    return rng.choice(I32) if rng.random() < 0.5 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {IS_FLYING_LOW: ("is_flying_low", 0)})
    uc = h.m.uc
    rng = random.Random(2090)
    ans = [0]
    vt_stub = h.vtable({0xBC: ("is_on_map", 0, lambda ecx, args: ans[0])})
    vt_base = h.alloc(0x200)
    h.w32(vt_base + 0xBC, IS_ON_MAP)
    unit = h.alloc(0x120)
    rec = h.alloc(0x320)
    for i in range(400):
        base = i % 2
        domain = 2 if rng.random() < 0.8 else rng.choice((0, 1, 3, -1, 0x102, rng.randint(-0x80000000, 0x7FFFFFFF)))
        b2b4 = rng.randrange(256) & (0xFF if rng.random() < 0.15 else 0xDF)
        w1e4 = rng.randint(0, 0xFFFFFFFF) & (0xFFFFFFFF if rng.random() < 0.15 else 0xF7FFFFFF)
        w82 = rng.randrange(0x10000)
        ans[0] = word(rng)
        low = word(rng)
        h.answers["is_flying_low"] = low
        mem = bytearray(rng.randrange(256) for _ in range(0x100))
        mem[0:4] = struct.pack("<I", vt_base if base else vt_stub)
        mem[0x18:0x1C] = struct.pack("<I", rec)
        mem[0x82:0x84] = struct.pack("<H", w82)
        rmem = bytearray(rng.randrange(256) for _ in range(0x300))
        rmem[0x218:0x21C] = struct.pack("<i", domain)
        rmem[0x2B4] = b2b4
        rmem[0x1E4:0x1E8] = struct.pack("<I", w1e4)
        uc.mem_write(unit, bytes(mem))
        uc.mem_write(rec, bytes(rmem))
        before = bytes(uc.mem_read(unit, 0x120)) + bytes(uc.mem_read(rec, 0x320))
        h.calls.clear()
        r = h.call(ENTRY, (), ecx=unit)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(unit, 0x120)) + bytes(uc.mem_read(rec, 0x320)) == before, "wrote memory"
        calls = ";".join(f"{name},{'unit' if ecx == unit else 'other'}" for name, ecx, _, _ in h.calls)
        print(f"{base} {domain} {b2b4} {w1e4} {w82} {ans[0]} {low} -> {r} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
