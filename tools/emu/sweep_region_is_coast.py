#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_region_is_coast.py — `Region::is_coast@00680f90` under unicorn (item 2038, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_region_is_coast.py <install>/riseofnations.exe

`__thiscall` with one stack argument (the other region), `ret 4`. A region is land below `0x40` and sea from it;
its coast set is a bit set of sea regions at `+0x64` (bit `r - 0x40`). The answer is 1 for the region itself;
for a land `this` and a sea argument, the argument's bit in `this`'s set; for a sea `this` and a land argument,
`this`'s bit in the argument's set, read from the region list (`0x88` bytes each) through the global cell
`0xc061b8`'s `+0x10`; 0 for two lands or two seas. A row draws both regions in -2..=0x7f — a sea `this` is never
handed a negative argument, which would index before the list — and the two eight-byte sets as two words each.
Row: `<region> <other> <this set lo hi> <other's set lo hi> -> <eax>`.
"""
import os
import random
import struct
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


ENTRY = 0x00680F90
CELL = 0x00C061B8


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2038)
    regions, lst, this = h.alloc(0x40), h.alloc(64 * 0x88), h.alloc(0x88)
    saved = h.r32(CELL)
    uc.mem_write(CELL, struct.pack("<I", regions))
    for i in range(300):
        region = rng.choice((0, 1, 0x3F, 0x40, 0x41, 0x7F, -1, -2)) if rng.random() < 0.3 else rng.randrange(-2, 0x80)
        if rng.random() < 0.1:
            other = region
        else:
            lo = 0 if region >= 0x40 else -2
            other = rng.choice((0, 1, 0x3F, 0x40, 0x41, 0x7F)) if rng.random() < 0.3 else rng.randrange(lo, 0x80)
        uc.mem_write(regions, bytes(rng.randrange(256) for _ in range(0x40)))
        h.w32(regions + 0x10, lst)
        uc.mem_write(lst, bytes(rng.randrange(256) for _ in range(64 * 0x88)))
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x88)))
        h.w32(this + 0x10, region)
        mine = struct.unpack("<ii", uc.mem_read(this + 0x64, 8))
        theirs = struct.unpack("<ii", uc.mem_read(lst + other * 0x88 + 0x64, 8)) if 0 <= other < 0x40 else (0, 0)
        before = bytes(uc.mem_read(this, 0x88)), bytes(uc.mem_read(lst, 64 * 0x88)), bytes(uc.mem_read(regions, 0x40))
        res = h.call(ENTRY, (other,), ecx=this)
        popped(uc)
        after = bytes(uc.mem_read(this, 0x88)), bytes(uc.mem_read(lst, 64 * 0x88)), bytes(uc.mem_read(regions, 0x40))
        assert after == before, "wrote"
        print(" ".join(str(x) for x in (region, other, *mine, *theirs)) + f" -> {res}")
    uc.mem_write(CELL, struct.pack("<i", saved))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
