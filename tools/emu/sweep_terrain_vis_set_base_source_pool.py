#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_terrain_vis_set_base_source_pool.py — `TerrainVis::set_base_source_pool@008b2a80` under unicorn (item 1942).

    uv run tools/emu/sweep_terrain_vis_set_base_source_pool.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object, two stack arguments (`ret 8`): the first lands at `this+0x100` (`source_pools[0]`) and the second at `this+0x11c` (`source_fvf[0]`). One row per input,
`<p0> <p1> <old 0x100> <old 0x11c> -> <new 0x100> <new 0x11c>`; the script asserts every other byte of the object is unmoved.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x008b2a80
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x3F800000, 0x7FC00000)


def check_esp(h):
    esp = h.m.uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def calls_str(h, base=0):
    return ";".join(",".join(str(x) for x in (ecx - base if i == 0 else ecx, *args)) for i, (_, ecx, args, _) in enumerate(h.calls))


def main(argv):
    from hooks import Harness
    rng = random.Random(1942)
    h = Harness(argv[1], {})
    rows = [(rng.choice(I32), rng.choice(I32)) for _ in range(40)] + [(rng.randint(-0x80000000, 0x7FFFFFFF), rng.randint(-0x80000000, 0x7FFFFFFF)) for _ in range(150)]
    for p0, p1 in rows:
        this = h.alloc(0x200)
        fill = bytearray(rng.randrange(256) for _ in range(0x200))
        h.m.uc.mem_write(this, bytes(fill))
        o0, o1 = struct.unpack("<i", fill[0x100:0x104])[0], struct.unpack("<i", fill[0x11C:0x120])[0]
        h.call(ENTRY, (p0, p1), ecx=this)
        check_esp(h)
        after = bytes(h.m.uc.mem_read(this, 0x200))
        assert after[:0x100] == fill[:0x100] and after[0x104:0x11C] == fill[0x104:0x11C] and after[0x120:] == fill[0x120:], "touched another byte"
        n0, n1 = struct.unpack("<i", after[0x100:0x104])[0], struct.unpack("<i", after[0x11C:0x120])[0]
        print(f"{p0} {p1} {o0} {o1} -> {n0} {n1}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
