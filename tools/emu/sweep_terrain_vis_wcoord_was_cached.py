#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_terrain_vis_wcoord_was_cached.py — `TerrainVis::wcoord_was_cached@008b13b0` under unicorn (item 1932).

    uv run tools/emu/sweep_terrain_vis_wcoord_was_cached.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the TerrainVis, three stack arguments (`ret 0xc`): the `CoordInfo` pointer, one unread, and an index. The answer is 1 when
`CoordInfo.last_vis_frame[index]` (the word at `+0x6c+4*index`) equals `this+0xfc` (`last_moved_frame`), else 0. One row per input,
`<index> <frame in the record> <last_moved_frame> -> <answer>`; the indices run past the array's three on purpose.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, STACK_BASE, STACK_SIZE, signed  # noqa: E402
from sweeplib import OBJ, ARG, setup, xmm_bits  # noqa: E402,F401
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x008b13b0
EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x3F800000, 0x7FC00000)


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    setup(uc)
    rng = random.Random(1932)
    rows = []
    for i in range(240):
        idx = i % 8
        lm = rng.choice(EDGES + (rng.randint(-0x80000000, 0x7FFFFFFF),))
        fr = lm if i % 2 == 0 else rng.choice((lm + 1, lm - 1, rng.randint(-0x80000000, 0x7FFFFFFF)))
        rows.append((idx, ((fr + 0x80000000) & 0xFFFFFFFF) - 0x80000000, lm))
    for idx, fr, lm in rows:
        uc.mem_write(OBJ, bytes(rng.randrange(256) for _ in range(0x100)))
        uc.mem_write(ARG, bytes(rng.randrange(256) for _ in range(0x100)))
        uc.mem_write(OBJ + 0xFC, struct.pack("<i", lm))
        uc.mem_write(ARG + 0x6C + 4 * idx, struct.pack("<i", fr))
        r = m.call(ENTRY, (ARG, rng.randint(-0x80000000, 0x7FFFFFFF), idx), ecx=OBJ)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        print(f"{idx} {fr} {lm} -> {signed(r)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
