#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_group_data_get_o.py — `GroupData::get_o@0070f8b0` under unicorn (item 1976, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_group_data_get_o.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the group: count at `+0xc`, the buildings byte at `+0x49`, a signed 16-bit word at `+0x8cc`; one stack argument, an optional out-pointer.
A non-null out-pointer is zeroed first. A count below 1 answers -1; a non-zero buildings byte answers the 16-bit word; otherwise it tail-calls `GroupData::find_leader@0070ccb0` (hooked, one argument: the same pointer) and answers what it answers.
Row: `<this> <ptr> <num> <buildings> <word> <find_leader answer> -> <eax> | <ecx>,<arg> or empty | <the word at ptr after, or -1 for no pointer>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0070f8b0
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def pop_ok(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {0x0070CCB0: ("find_leader", 1)})
    uc = h.m.uc
    rng = random.Random(1976)
    this, ptr0 = h.alloc(0x900), h.alloc(8)
    for i in range(400):
        ptr = ptr0 if i % 3 else 0
        num = rng.choice((-5, -1, 0, 1, 2, 9, 0x7FFFFFFF, -0x80000000))
        bld = rng.choice((0, 0, 1, 2, 0xFF))
        word = rng.randint(-0x8000, 0x7FFF)
        ans = rand32(rng)
        h.w32(this + 0xC, num)
        h.w8(this + 0x49, bld)
        h.w16(this + 0x8CC, word)
        if ptr:
            h.w32(ptr, rng.randint(1, 1000))
        h.answers["find_leader"] = ans & 0xFFFFFFFF
        h.calls.clear()
        r = h.call(ENTRY, (ptr,), ecx=this)
        pop_ok(uc)
        seen = ";".join(f"{ecx},{args[0]}" for _, ecx, args, _ in h.calls)
        after = h.r32(ptr) if ptr else -1
        print(f"{this} {ptr} {num} {bld} {word} {ans} -> {r} | {seen} | {after}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
