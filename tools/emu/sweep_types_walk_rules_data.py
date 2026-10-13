#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_types_walk_rules_data.py — `Types::walk_rules_data@00669800` under unicorn (item 1921).

    uv run tools/emu/sweep_types_walk_rules_data.py <install>/riseofnations.exe

`__thiscall`, `ecx` unused, one stack argument: the walker. The function walks the
pointer array in the global cell `types.list` (0x00e85ddc, 0xc98 bytes = 806 entries)
and calls slot 0xc4 of each entry's vtable, `ecx` = the entry, handed the walker. The
entries here are 16 bytes apart from the row's first; the slot is hooked. One row per
input, `<first entry> <walker> -> <eax> | <ecx of each call, in order>`; the script
asserts every call was handed the walker alone and that `esp` is popped by 4.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00669800
CELL = 0x00E85DDC
N = 0xC98 // 4


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    rng = random.Random(1921)
    vt = h.vtable({0xC4: ("rules", 1, 5)})
    for _ in range(12):
        arr = h.alloc(0xC98)
        first = h.alloc(16)
        h.w32(first, vt)
        for i in range(1, N):
            o = h.alloc(16)
            h.w32(o, vt)
        for i in range(N):
            h.w32(arr + 4 * i, first + 16 * i)
        h.m.uc.mem_write(CELL, struct.pack("<I", arr))
        walker = h.alloc(16) + rng.randrange(0, 4) * 4
        ecx = rng.randrange(0x1000, 0x7FFF0000) & ~3
        h.calls.clear()
        r = h.call(ENTRY, (walker,), ecx=ecx)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert all(args == [walker] for _, _, args, _ in h.calls), "a call was handed more than the walker"
        print(f"{first} {walker} -> {r} | " + ",".join(str(c[1]) for c in h.calls))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
