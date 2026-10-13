#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_update_los.py — `Object::update_los@00646fe0` under unicorn (item 2020, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_object_update_los.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`. `who` is the unsigned byte at `this+9`; the leader is the
`0x6eec`-byte record at `leaders.list` (0xe3a390, the image's data) `+ who * 0x6eec`, and its first byte's bit 0
gates: clear, it stores 0 at the signed byte `this+0x3c` and answers 0; set, it copies the signed byte at
`+0x21c` of the type record the pointer at `this+0x18` names into `this+0x3c` and answers it sign-extended.
`who` is held to 0..8; the leader's byte, the type's byte and every other byte of `this` are random.
Row: `<who> <leader byte> <type byte> <+0x3c before> <+0x3c after> <others untouched> -> <eax>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00646FE0
LEADERS = 0x00E3A390
STRIDE = 0x6EEC
WIN = 0x80


def sb(b):
    return b - 256 if b & 0x80 else b


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2020)
    this, rec = h.alloc(WIN), h.alloc(0x240)
    for i in range(300):
        who = rng.randrange(9)
        lb, tb = rng.randrange(256), rng.randrange(256)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(WIN)))
        uc.mem_write(rec, bytes(rng.randrange(256) for _ in range(0x240)))
        h.w8(this + 9, who)
        h.w32(this + 0x18, rec)
        h.w8(rec + 0x21C, tb)
        h.w8(LEADERS + who * STRIDE, lb)
        before = bytes(uc.mem_read(this, WIN))
        r = h.call(ENTRY, (), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        after = bytes(uc.mem_read(this, WIN))
        rest = int(before[:0x3C] + before[0x3D:] == after[:0x3C] + after[0x3D:])
        print(" ".join(str(x) for x in (who, lb, sb(tb), sb(before[0x3C]), sb(after[0x3C]), rest)) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
