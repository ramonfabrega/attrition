#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_city_data_depop_can_queue.py — `CityData::depop_can_queue@00738060` under unicorn (item 2072, hand-written on tools/emu/hooks.py,
2038's `pop_cap` shape).

    uv run tools/emu/sweep_city_data_depop_can_queue.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`. It finds the city's own object through the owner's table
(`0xc0ab84 + who*0x1c`, `who` the signed byte at `+0x5e`, the index the signed short at `+0x8`), asks its vtable
slot `+0x3c` for its data record, and asks the record's own vtable slot `+0x190` with `(1, 0x286)`; it answers
1 when that answer is 0. A row draws the second slot's answer.
Row: `<answer> <calls> <first ecx is the object> <second ecx is the record> <second's two arguments> -> <eax>`.
"""
import os
import random
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


ENTRY = 0x00738060
TABLE = 0x00C0AB84
N = 32


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2072)
    slots = h.alloc(N * 4)
    obj, data, this = h.alloc(0x40), h.alloc(0x40), h.alloc(0x80)
    vt = h.vtable({0x3C: ("data", 0, data)})
    dvt = h.vtable({0x190: ("ask", 2, 0)})
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
        h.w32(data, dvt)
        ans = rand32(rng) if rng.random() < 0.6 else 0
        h.answers["ask"] = ans
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x80)))
        h.w8(this + 0x5E, who)
        h.w16(this + 0x8, idx)
        h.calls.clear()
        wins = ((this, 0x80), (obj, 0x40), (data, 0x40), (slots, N * 4))
        before = [bytes(uc.mem_read(a, n)) for a, n in wins]
        res = h.call(ENTRY, (), ecx=this)
        popped(uc)
        assert [bytes(uc.mem_read(a, n)) for a, n in wins] == before, "wrote"
        h.w32(TABLE + who * 0x1C, saved)
        first = int(h.calls[0][1] == obj) if h.calls else 0
        second = int(h.calls[1][1] == data) if len(h.calls) > 1 else 0
        args = " ".join(str(a) for a in h.calls[1][2]) if len(h.calls) > 1 else "-1 -1"
        print(f"{ans} {len(h.calls)} {first} {second} {args} -> {res}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
