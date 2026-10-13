#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_wall_data_get_garrison_limit.py — `WallData::get_garrison_limit@0063ed40` under unicorn (item 1966, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_wall_data_get_garrison_limit.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 64-byte random object. It calls `BuildTypeData::get_garrison_limit@00633f50` (hooked, one stack
argument) with `ecx` = the word at `this+0x18` and the byte at `this+9`, and answers what that returns.
Row: `<callee answer> | <object hex> -> <eax> | <ecx,arg>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0063ed40
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {0x00633F50: ("get_garrison_limit", 1)})
    rng = random.Random(1966)
    for _ in range(240):
        this = h.alloc(0x60)
        mem = bytes(rng.randrange(256) for _ in range(0x40))
        ans = rand32(rng)
        h.m.uc.mem_write(this, mem)
        h.answers["get_garrison_limit"] = ans & 0xFFFFFFFF
        h.calls.clear()
        r = h.call(ENTRY, (), ecx=this)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        seen = ";".join(",".join(str(x) for x in (ecx, *args)) for _, ecx, args, _ in h.calls)
        print(f"{ans} | {mem.hex()} -> {r} | {seen}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
