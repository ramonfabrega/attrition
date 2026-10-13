#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_guy_set_pitch.py — `Guy::set_pitch@005d86a0` under unicorn (item 1932).

    uv run tools/emu/sweep_guy_set_pitch.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object, the float in `xmm1` (the decompile's `in_XMM1_Da`), a plain `ret`. The function moves
the word at `this+0x4c` to `this+0x50` and stores `xmm1`'s bits at `this+0x4c`. One row per input,
`<bits> <old word> -> <new+0x4c + (new+0x50 << 32)>`; the script asserts every other byte of the object is untouched
and `esp` is where the call left it.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, STACK_BASE, STACK_SIZE, signed  # noqa: E402
from sweeplib import OBJ, ARG, setup, xmm_bits  # noqa: E402,F401
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

def signed64(v):
    return v - (1 << 64) if v >> 63 else v


ENTRY = 0x005d86a0
EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x3F800000, 0x7FC00000)


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    setup(uc)
    rng = random.Random(1932)
    bits = list(EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]
    for b in bits:
        old = rng.choice((0, rng.randint(-0x80000000, 0x7FFFFFFF)))
        fill = bytearray(rng.randrange(256) for _ in range(0x100))
        fill[76:80] = struct.pack("<i", old)
        uc.mem_write(OBJ, bytes(fill))
        m.call(ENTRY, (), ecx=OBJ, xmm={1: xmm_bits(b & 0xFFFFFFFF, raw=True)})
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        after = bytes(uc.mem_read(OBJ, 0x100))
        assert after[:76] == fill[:76] and after[84:] == fill[84:], "touched another byte"
        lo = struct.unpack("<I", after[76:80])[0]
        hi = struct.unpack("<I", after[80:84])[0]
        print(f"{b} {old} -> {signed64((hi << 32) | lo)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
