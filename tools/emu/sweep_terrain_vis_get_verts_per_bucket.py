#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_terrain_vis_get_verts_per_bucket.py — `TerrainVis::get_verts_per_bucket@008b0060` under unicorn (item 1942).

    uv run tools/emu/sweep_terrain_vis_get_verts_per_bucket.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object, a plain `ret`, no argument read. The graphics system is the pointer at `0x00e335a0` and its signed `sys_grade` the dword at `+0xe0`: grade three or
more answers 9, otherwise the answer is `this[0x10 + 4*cur_zoom]` with `cur_zoom` the dword at `this+4`. One row per input, `<grade> <cur_zoom> <the table entry> -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x008b0060
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
    import struct as st
    rows = [(g, z) for g in (-0x80000000, -1, 0, 1, 2, 3, 4, 0x7FFFFFFF) for z in (0, 1, 2, 5, 7)] + [(rng.randint(-3, 6), rng.randrange(8)) for _ in range(150)]
    for g, z in rows:
        this = h.alloc(0x100)
        fill = bytearray(rng.randrange(256) for _ in range(0x100))
        fill[4:8] = st.pack("<i", z)
        h.m.uc.mem_write(this, bytes(fill))
        gs = h.alloc(0x200)
        h.w32(gs + 0xE0, g)
        h.m.uc.mem_write(0xE335A0, st.pack("<I", gs))
        r = h.call(ENTRY, (), ecx=this)
        check_esp(h)
        print(f"{g} {z} {st.unpack('<i', fill[0x10 + 4 * z:0x14 + 4 * z])[0]} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
