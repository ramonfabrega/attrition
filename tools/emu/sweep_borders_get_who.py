#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_borders_get_who.py — `Borders::get_who@00883190` under unicorn (item 1993, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_borders_get_who.py <install>/riseofnations.exe

`__thiscall` (`ecx` unread); stack arguments two ints `x`, `y` (not pointers). The world is the pointer in the global cell `0xc061d0`: width int at `+0`, cell array pointer at `+0x134`, cells 0x1c bytes.
Answers the signed byte at `+0xf` of cell `(y*width + x)` when non-negative, else -1.
Row: `<width> <x> <y> <byte> -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00883190
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def pop_ok(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(1993)
    world, arr = h.alloc(0x200), h.alloc(0x1c * 24 * 24)
    for i in range(400):
        w = rng.randint(1, 24)
        x, y = rng.randrange(w), rng.randrange(24)
        uc.mem_write(0x00C061D0, struct.pack("<I", world))
        h.w32(world, w)
        h.w32(world + 0x134, arr)
        uc.mem_write(arr, bytes(rng.randrange(256) for _ in range(0x1c * 24 * 24)))
        b = rng.choice((0, 1, 7, 0x7F, 0x80, 0xFF, 0xFE)) if i % 2 else rng.randrange(256)
        h.w8(arr + 0x1c * (y * w + x) + 0xF, b)
        r = h.call(ENTRY, (x, y), ecx=rng.randrange(1 << 32))
        pop_ok(uc)
        print(f"{w} {x} {y} {b} -> {r}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
