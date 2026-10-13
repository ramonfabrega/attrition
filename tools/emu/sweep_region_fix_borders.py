#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_region_fix_borders.py — `Region::fix_borders@00680f60` under unicorn (item 2020, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_region_fix_borders.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`; `this` is never read. The regions singleton is the pointer in the
global cell `0xc061b8`, its list the pointer at `+0x10` (re-read every pass), and the loop zeroes the 32-bit word
at `+0x2c` of each `0x88`-byte region, `0x2200` bytes — 64 regions. The list is random bytes with a 0x40-byte
guard past it; `this` is random. The row prints the list before and after in hex, and whether `this`, the
singleton and the guard are untouched.
Row: `<untouched> | <before hex> -> <eax> | <after hex>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00680F60
CELL = 0x00C061B8
SPAN = 0x2200
GUARD = 0x40


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2020)
    regions, lst, this = h.alloc(0x40), h.alloc(SPAN + GUARD), h.alloc(0x40)
    uc.mem_write(CELL, struct.pack("<I", regions))
    for i in range(40):
        uc.mem_write(regions, bytes(rng.randrange(256) for _ in range(0x40)))
        h.w32(regions + 0x10, lst)
        uc.mem_write(lst, bytes(rng.randrange(256) for _ in range(SPAN + GUARD)))
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x40)))
        rb, tb = bytes(uc.mem_read(regions, 0x40)), bytes(uc.mem_read(this, 0x40))
        before = bytes(uc.mem_read(lst, SPAN + GUARD))
        r = h.call(ENTRY, (), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        after = bytes(uc.mem_read(lst, SPAN + GUARD))
        same = int(bytes(uc.mem_read(regions, 0x40)) == rb and bytes(uc.mem_read(this, 0x40)) == tb)
        print(f"{same} | {before.hex()} -> {r} | {after.hex()}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
