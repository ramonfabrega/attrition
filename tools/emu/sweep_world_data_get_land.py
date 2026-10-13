#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_data_get_land.py — `WorldData::get_land@0046f9d0` under unicorn (item 1993, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_world_data_get_land.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the world, passed through unchanged. Stack arguments two Coord pointers (`p1`, `p2`) and an int `p3`. The table pointer at `0xcae5fc` (`div_3_table`) maps `coord >> 8` (arithmetic) to a word;
the function writes the two mapped words to its own frame and calls `get_land@006b4730` (hooked; three stack arguments: pointers to those words, then `p3`), answering its result.
Row: `<c1> <c2> <p3> <callee answer> -> <eax> | <ecx>,<*a0>,<*a1>,<a2>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0046f9d0
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def pop_ok(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def tab(i):
    return (i * 2654435761 >> 5) & 0xFFFF


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {0x006B4730: ("land", 3)})
    uc = h.m.uc
    rng = random.Random(1993)
    N = 1024
    table = h.alloc(4 * N)
    uc.mem_write(table, b"".join(struct.pack("<I", tab(i)) for i in range(N)))
    uc.mem_write(0x00CAE5FC, struct.pack("<I", table))
    cs = h.alloc(8)
    seen = []

    def answer(ecx, args):
        seen.append((ecx, h.r32(args[0]), h.r32(args[1]), args[2]))
        return ans[0]

    ans = [0]
    h.answers["land"] = answer
    for _ in range(300):
        c1, c2 = rng.randrange(N) << 8 | rng.randrange(256), rng.randrange(N) << 8 | rng.randrange(256)
        p3 = rand32(rng)
        ans[0] = rand32(rng) & 0xFFFFFFFF
        seen.clear()
        this = rng.randrange(1, 1 << 32)
        h.w32(cs, c1)
        h.w32(cs + 4, c2)
        r = h.call(ENTRY, (cs, cs + 4, p3), ecx=this)
        pop_ok(uc)
        assert len(seen) == 1, seen
        e, a0, a1, a2 = seen[0]
        e = e - (1 << 32) if e & 0x80000000 else e
        print(f"{c1} {c2} {p3} {ans[0] - (1 << 32) if ans[0] & 0x80000000 else ans[0]} {this} -> {r} | {e},{a0},{a1},{a2}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
