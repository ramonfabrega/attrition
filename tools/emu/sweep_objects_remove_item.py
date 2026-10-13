#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_objects_remove_item.py — `Objects::remove_item@0065b850` under unicorn (item 1932).

    uv run tools/emu/sweep_objects_remove_item.py <install>/riseofnations.exe

`__thiscall`, `ecx` unused, one stack argument (`ret 4`): an index into `items`, the pointer array whose base is the word at `0xc0a110`.
The function calls virtual slot `0x90` of `items[index]` with that object as `ecx` and no arguments. Each object here has its own fake
vtable whose slot 0x90 is hooked; one row per input, `<index> <count of objects> -> <the answer of slot 0x90 on that object> | <which object, ecx>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, STACK_BASE, STACK_SIZE, signed  # noqa: E402
from sweeplib import OBJ, ARG, setup, xmm_bits  # noqa: E402,F401
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0065b850
EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x3F800000, 0x7FC00000)


def main(argv):
    from hooks import Harness
    import struct as st
    h = Harness(argv[1], {})
    rng = random.Random(1932)
    count = 40
    arr = h.alloc(4 * count)
    objs = []
    for k in range(count):
        o = h.alloc(0x20)
        vt = h.vtable({0x90: (f"slot{k}", 0, 1000 + k * 7)})
        h.w32(o, vt)
        h.w32(arr + 4 * k, o)
        objs.append(o)
    h.m.uc.mem_write(0xC0A110, st.pack("<I", arr))
    rows = [(i, count) for i in range(count)] + [(rng.randrange(0, count), count) for _ in range(120)]
    for idx, _ in rows:
        h.calls.clear()
        this = rng.randrange(0x1000, 0x7FFF0000) & ~3
        r = h.call(ENTRY, (idx,), ecx=this)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        seen = ";".join(f"{objs.index(ecx)}" for _, ecx, args, _ in h.calls)
        assert all(not args for _, _, args, _ in h.calls), "slot handed arguments"
        print(f"{idx} {count} -> {r} | {seen}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
