#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_objects_data_find_good_at.py — `ObjectsData::find_good_at@00658160` under unicorn (item 1998, hand-written
on tools/emu/hooks.py).

    uv run tools/emu/sweep_objects_data_find_good_at.py <install>/riseofnations.exe

The `Coord` overload: `__thiscall`, four stack arguments (`x`, `y` and two it never reads), `ret 0x10`. It maps `x`
and `y` through `div_3_table` (the pointer in the global cell `0xcae5fc`) at `c >> 8` (arithmetic) and calls the
`WCoord` overload `ObjectsData::find_good_at@0065bec0` (hooked, five arguments, `ret 0x14`) with
`(wx, wy, -1, 1, 0)`, answering what that answers. Its `ecx` is not `this`: the listing loads `x` into `ecx` for
the shift and calls with it, so the callee is handed `x >> 8` as its object — the decompile's
`(ObjectsData *)(x >> 8)` is the listing's own reading, and the row prints the `ecx` the callee saw. The table is 1,024 random words a row and the coordinates are held to
`0 .. 1024 << 8`; the two unread arguments are random words.
Row: `<x> <y> <p2> <p3> <wx> <wy> <answer> <callee ecx> <handed ×5> -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00658160
CALLEE = 0x0065BEC0
DIV3_CELL = 0x00CAE5FC
N = 1024


def main(argv):
    from hooks import Harness
    rng = random.Random(1998)
    cur = {}
    h = Harness(argv[1], {CALLEE: ("ObjectsData::find_good_at", 5)})
    h.answers["ObjectsData::find_good_at"] = lambda ecx, args: cur["ans"]
    uc = h.m.uc
    table, this = h.alloc(4 * N), h.alloc(0x100)
    uc.mem_write(DIV3_CELL, struct.pack("<I", table))
    edges = [0, 1, 255, 256, 257, (N << 8) - 1]
    for i in range(420):
        x = edges[i % len(edges)] if i < 36 else rng.randrange(N << 8)
        y = edges[i // len(edges) % len(edges)] if i < 36 else rng.randrange(N << 8)
        p2, p3 = rng.randint(-0x80000000, 0x7FFFFFFF), rng.randint(-0x80000000, 0x7FFFFFFF)
        words = [rng.randint(-0x80000000, 0x7FFFFFFF) if rng.random() < 0.3 else rng.randrange(N // 3) for _ in range(N)]
        uc.mem_write(table, struct.pack(f"<{N}i", *words))
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x100)))
        cur["ans"] = rng.choice((-1, 0, 1, rng.randrange(0x10000), rng.randint(-0x80000000, 0x7FFFFFFF)))
        h.calls.clear()
        r = h.call(ENTRY, (x, y, p2, p3), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert len(h.calls) == 1, h.calls
        _, ecx, args, _ = h.calls[0]
        print(" ".join(str(v) for v in (x, y, p2, p3, words[x >> 8], words[y >> 8], cur["ans"], ecx - (1 << 32) if ecx & 0x80000000 else ecx, *args))
              + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
