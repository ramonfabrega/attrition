#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_terrain_map_start_river.py — `TerrainMap::start_river@009b3320` under unicorn (item 1942).

    uv run tools/emu/sweep_terrain_map_start_river.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the map, two unused stack arguments (`ret 8`). The function calls `RiversOut::start_river@00888f70` (hooked, no arguments, `ecx` = `this`) and stores its answer at
`this+0x294`. One row per input, `<answer> <p0> <p1> <old word> -> <stored word> | <ecx - this>`; the script asserts every other byte of the object is unmoved.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x009b3320
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x3F800000, 0x7FC00000)


def check_esp(h):
    esp = h.m.uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def calls_str(h, base=0):
    return ";".join(",".join(str(x) for x in (ecx - base if i == 0 else ecx, *args)) for i, (_, ecx, args, _) in enumerate(h.calls))


def main(argv):
    from hooks import Harness
    rng = random.Random(1942)
    h = Harness(argv[1], {0x888F70: ("start_river", 0)})
    rows = [(rng.choice(I32), rng.choice(I32), rng.choice(I32)) for _ in range(40)] + [tuple(rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(3)) for _ in range(150)]
    for ans, p0, p1 in rows:
        this = h.alloc(0x300)
        fill = bytearray(rng.randrange(256) for _ in range(0x300))
        h.m.uc.mem_write(this, bytes(fill))
        old = struct.unpack("<i", fill[0x294:0x298])[0]
        h.calls.clear()
        h.answers["start_river"] = ans & 0xFFFFFFFF
        h.call(ENTRY, (p0, p1), ecx=this)
        check_esp(h)
        after = bytes(h.m.uc.mem_read(this, 0x300))
        assert after[:0x294] == fill[:0x294] and after[0x298:] == fill[0x298:], "touched another byte"
        print(f"{ans} {p0} {p1} {old} -> {struct.unpack('<i', after[0x294:0x298])[0]} | {calls_str(h, this)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
