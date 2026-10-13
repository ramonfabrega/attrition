#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_terrain_map_river_point.py — `TerrainMap::river_point@009b32f0` under unicorn (item 1942).

    uv run tools/emu/sweep_terrain_map_river_point.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the map, two stack arguments (`ret 8`). The function calls `RiverOut::add_spline_point@00885a40` (hooked, two arguments) with `ecx` = `[this+0x294]+8` and the two
arguments in order. One row per input, `<cur_river> <p0> <p1> -> | <ecx,a0,a1>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x009b32f0
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x3F800000, 0x7FC00000)


def check_esp(h):
    esp = h.m.uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def calls_str(h, base=0):
    return ";".join(",".join(str(x) for x in (ecx - base if i == 0 else ecx, *args)) for i, (_, ecx, args, _) in enumerate(h.calls))


def main(argv):
    from hooks import Harness
    rng = random.Random(1942)
    h = Harness(argv[1], {0x885A40: ("add_spline_point", 2)})
    rows = [(rng.choice(I32), rng.choice(I32), rng.choice(I32)) for _ in range(40)] + [tuple(rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(3)) for _ in range(150)]
    for cur, p0, p1 in rows:
        this = h.alloc(0x300)
        h.m.uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x300)))
        h.w32(this + 0x294, cur)
        h.calls.clear()
        h.call(ENTRY, (p0, p1), ecx=this)
        check_esp(h)
        print(f"{cur} {p0} {p1} -> | {calls_str(h)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
