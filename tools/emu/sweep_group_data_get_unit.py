#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_group_data_get_unit.py — `GroupData::get_unit@0070cc90` under unicorn (item 1932).

    uv run tools/emu/sweep_group_data_get_unit.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the GroupData, a plain `ret`. When the byte at `this+0x49` (`buildings`) is zero and the signed word at `this+0xc`
(`num`) is positive the function tail-calls `find_leader@0070ccb0(this, 0)` (hooked) and answers its result; otherwise it answers -1 and
calls nothing. One row per input, `<buildings> <num> <find_leader's answer> -> <eax> | <ecx,args…>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, STACK_BASE, STACK_SIZE, signed  # noqa: E402
from sweeplib import OBJ, ARG, setup, xmm_bits  # noqa: E402,F401
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0070cc90
EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x3F800000, 0x7FC00000)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {0x70CCB0: ("find_leader", 1)})
    rng = random.Random(1932)
    rows = []
    for b in (0, 1, 2, 0x80, 0xFF):
        for num in (-0x80000000, -1, 0, 1, 2, 100, 0x7FFFFFFF):
            rows.append((b, num, rng.randint(-0x80000000, 0x7FFFFFFF)))
    rows += [(rng.choice((0, 0, 1, rng.randrange(256))), rng.choice((rng.randint(-5, 50), rng.randint(-0x80000000, 0x7FFFFFFF))),
              rng.randint(-0x80000000, 0x7FFFFFFF)) for _ in range(180)]
    for b, num, ans in rows:
        this = h.alloc(0x100)
        h.m.uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x100)))
        h.w8(this + 0x49, b)
        h.w32(this + 0xC, num)
        h.calls.clear()
        h.answers["find_leader"] = ans & 0xFFFFFFFF
        r = h.call(ENTRY, (), ecx=this)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        seen = ";".join(",".join(str(x) for x in (ecx - this, *args)) for _, ecx, args, _ in h.calls)
        print(f"{b} {num} {ans} -> {r} | {seen}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
