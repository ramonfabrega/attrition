#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_path_finder_valid_tcoord.py — `PathFinder::valid_tcoord@00687d60` under unicorn (item 1998, hand-written
on tools/emu/hooks.py).

    uv run tools/emu/sweep_path_finder_valid_tcoord.py <install>/riseofnations.exe

`__thiscall`, stack `x`, `y` (`Coord`s), `ret 8`. It maps each through `div_3_table` — the table the pointer in
the global cell `0xcae5fc` names — at `c >> 6` (arithmetic), and calls `UnitData::invalid_loc@00607c30` (hooked,
eight arguments, `ret 0x20`) with `ecx` = the unit at `this+0x54` and the arguments
`(tx, ty, 0, 1, 1, 1, 0, this)`; it answers 1 when that answers 0, else 0. The table is 1,024 random words a
row and the coordinates are held to `0 .. 1024 << 6`, so the index stays inside it; `this` is random bytes
around the unit pointer.
Row: `<x> <y> <tx> <ty> <answer> <ecx is the unit> <handed ×8, `this` as 1 when it is this> -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00687D60
CALLEE = 0x00607C30
DIV3_CELL = 0x00CAE5FC
N = 1024


def main(argv):
    from hooks import Harness
    rng = random.Random(1998)
    cur = {}
    h = Harness(argv[1], {CALLEE: ("UnitData::invalid_loc", 8)})
    h.answers["UnitData::invalid_loc"] = lambda ecx, args: cur["ans"]
    uc = h.m.uc
    table, this, unit = h.alloc(4 * N), h.alloc(0x100), h.alloc(0x40)
    uc.mem_write(DIV3_CELL, struct.pack("<I", table))
    edges = [0, 1, 63, 64, 65, (N << 6) - 1]
    for i in range(420):
        x = edges[i % len(edges)] if i < 36 else rng.randrange(N << 6)
        y = edges[i // len(edges) % len(edges)] if i < 36 else rng.randrange(N << 6)
        words = [rng.randint(-0x80000000, 0x7FFFFFFF) if rng.random() < 0.3 else rng.randrange(N // 3) for _ in range(N)]
        uc.mem_write(table, struct.pack(f"<{N}i", *words))
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x100)))
        h.w32(this + 0x54, unit)
        cur["ans"] = rng.choice((0, 0, 1, -1, 0x7FFFFFFF, rng.randint(-0x80000000, 0x7FFFFFFF)))
        h.calls.clear()
        r = h.call(ENTRY, (x, y), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert len(h.calls) == 1, h.calls
        _, ecx, args, _ = h.calls[0]
        handed = [1 if (k == 7 and (a & 0xFFFFFFFF) == this) else a for k, a in enumerate(args)]
        print(" ".join(str(v) for v in (x, y, words[x >> 6], words[y >> 6], cur["ans"], int(ecx == unit), *handed))
              + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
