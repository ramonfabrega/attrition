#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_set_orig_coast.py — `World::set_orig_coast@006b1b80` under unicorn (item 1942).

    uv run tools/emu/sweep_world_set_orig_coast.py <install>/riseofnations.exe

`__thiscall`, `ecx` unused, three stack arguments (`ret 0xc`), none read. The world is the pointer at `0x00c06188`; the word its `+0x134` points at loses bit `0x400`. One row per input,
`<old word> -> <new word>`; the script asserts the neighbouring words are unmoved.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x006b1b80
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
    words = list(range(0, 0x10000, 0x155)) + [0xFFFF, 0x0400, 0xFBFF, 0, 1] + [rng.randrange(0x10000) for _ in range(120)]
    for w in words:
        world = h.alloc(0x200)
        cell = h.alloc(16)
        h.m.uc.mem_write(cell, bytes(rng.randrange(256) for _ in range(16)))
        h.m.uc.mem_write(cell + 4, st.pack("<H", w))
        before = bytes(h.m.uc.mem_read(cell, 16))
        h.w32(world + 0x134, cell + 4)
        h.m.uc.mem_write(0xC06188, st.pack("<I", world))
        h.call(ENTRY, (rng.choice(I32), rng.choice(I32), rng.choice(I32)), ecx=0x1234)
        check_esp(h)
        after = bytes(h.m.uc.mem_read(cell, 16))
        assert after[:4] == before[:4] and after[6:] == before[6:], "touched another byte"
        print(f"{w} -> {st.unpack('<H', after[4:6])[0]}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
