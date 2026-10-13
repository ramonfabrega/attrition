#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_balance_return_modifier.py — `Balance::return_modifier@00581ca0` under unicorn (item 1976, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_balance_return_modifier.py <install>/riseofnations.exe

`__thiscall` (`ecx` unread); stack arguments two type indices `a`, `b`. The answer is the sign-extended 16-bit word `table[a * 0x1ed + b]`, the table at `0xc06afc`.
The script fills `table[0 .. 0x1ed²)` with a hash of the index, so a wrong stride or a swapped pair reads the wrong word.
Row: `<a> <b> -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00581ca0
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
    rng = random.Random(1976)
    def hv(i):
        return (i * 2654435761 >> 7) & 0xFFFF
    N = 0x1ED * 0x1ED
    uc.mem_write(0x00C06AFC, b"".join(struct.pack("<H", hv(i)) for i in range(N)))
    for _ in range(400):
        a, b = rng.randrange(0x1ED), rng.randrange(0x1ED)
        r = h.call(ENTRY, (a, b), ecx=rng.getrandbits(32))
        pop_ok(uc)
        print(f"{a} {b} -> {r}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
