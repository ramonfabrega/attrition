#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_map_fog_get_fog_lookup.py — `WorldMapFog::get_fog_lookup@008d1830` under unicorn (item 1942).

    uv run tools/emu/sweep_world_map_fog_get_fog_lookup.py <install>/riseofnations.exe

`__thiscall`, `ecx` unused, two stack arguments (`ret 8`): pointers to two coordinate words. The table is the pointer at `0x00e8a894`, its row stride `[0x00e8a79c] + 2`; the answer
is the byte at `(y + 1) * (xs + 2) + x + 1`. The table here holds byte `i` = `(i*37 + 11) & 0xff`. One row per input, `<xs> <x> <y> -> <al>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x008d1830
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
    rows = []
    for _ in range(200):
        xs = rng.randint(1, 40)
        ys = rng.randint(1, 40)
        rows.append((xs, ys, rng.randint(-1, xs), rng.randint(-1, ys)))
    for xs, ys, x, y in rows:
        size = (xs + 2) * (ys + 3) + 4
        tab = h.alloc(size)
        h.m.uc.mem_write(tab, bytes((i * 37 + 11) & 0xFF for i in range(size)))
        h.m.uc.mem_write(0xE8A79C, st.pack("<i", xs))
        h.m.uc.mem_write(0xE8A894, st.pack("<I", tab))
        px, py = h.alloc(4), h.alloc(4)
        h.w32(px, x)
        h.w32(py, y)
        r = h.call(ENTRY, (px, py), ecx=0x1234)
        check_esp(h)
        print(f"{xs} {x} {y} -> {r & 0xFF}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
