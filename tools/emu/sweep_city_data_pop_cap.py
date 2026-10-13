#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_city_data_pop_cap.py — `CityData::pop_cap@00737d40` under unicorn (item 2038, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_city_data_pop_cap.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`. It finds the city's own object through the owner's table
(`0xc0ab84 + who*0x1c`, `who` the signed byte at `+0x5e`, the index the signed short at `+0x8`), asks its vtable
slot `+0x3c` for its data record, reads the type record at that record's `+0x18` and the type id at `+0x4`:
`0x19f` answers twice `village_pop` (the constants through the global cell `0xc061e4`, `+0x3e8`), `0x1a0` and
`0x213` three times it, anything else once. A row draws the id (the three, their neighbours, random) and the
constant.
Row: `<id> <village_pop> <slot calls> <slot ecx is the object> -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def popped(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


ENTRY = 0x00737D40
TABLE = 0x00C0AB84
CONSTS = 0x00C061E4
N = 32


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2038)
    slots = h.alloc(N * 4)
    obj, data, ty, consts, this = h.alloc(0x40), h.alloc(0x40), h.alloc(0x40), h.alloc(0x400), h.alloc(0x80)
    vt = h.vtable({0x3C: ("data", 0, data)})
    saved_c = h.r32(CONSTS)
    uc.mem_write(CONSTS, struct.pack("<I", consts))
    for i in range(300):
        who = rng.randrange(9)
        saved = h.r32(TABLE + who * 0x1C)
        h.w32(TABLE + who * 0x1C, slots)
        idx = rng.randrange(N)
        uc.mem_write(slots, bytes(rng.randrange(256) for _ in range(N * 4)))
        h.w32(slots + 4 * idx, obj)
        uc.mem_write(obj, bytes(rng.randrange(256) for _ in range(0x40)))
        h.w32(obj, vt)
        uc.mem_write(data, bytes(rng.randrange(256) for _ in range(0x40)))
        h.w32(data + 0x18, ty)
        tid = rng.choice((0x19F, 0x1A0, 0x213, 0x19E, 0x1A1, 0x212, 0x214, 0, -1)) if rng.random() < 0.8 \
            else rand32(rng)
        uc.mem_write(ty, bytes(rng.randrange(256) for _ in range(0x40)))
        h.w32(ty + 4, tid)
        pop = rng.choice((0, 1, 5, 10, 25, -3)) if rng.random() < 0.6 else rng.randint(-0x2AAAAAAA, 0x2AAAAAAA)
        uc.mem_write(consts, bytes(rng.randrange(256) for _ in range(0x400)))
        h.w32(consts + 0x3E8, pop)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x80)))
        h.w8(this + 0x5E, who)
        h.w16(this + 0x8, idx)
        h.calls.clear()
        before = [bytes(uc.mem_read(a, n)) for a, n in ((this, 0x80), (obj, 0x40), (data, 0x40), (ty, 0x40),
                                                         (consts, 0x400))]
        res = h.call(ENTRY, (), ecx=this)
        popped(uc)
        after = [bytes(uc.mem_read(a, n)) for a, n in ((this, 0x80), (obj, 0x40), (data, 0x40), (ty, 0x40),
                                                        (consts, 0x400))]
        assert after == before, "wrote"
        h.w32(TABLE + who * 0x1C, saved)
        ecx_obj = int(all(c[1] == obj for c in h.calls))
        print(f"{tid} {pop} {len(h.calls)} {ecx_obj} -> {res}")
    uc.mem_write(CONSTS, struct.pack("<i", saved_c))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
