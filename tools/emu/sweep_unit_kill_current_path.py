#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_kill_current_path.py — `Unit::kill_current_path@005e31d0` under unicorn (item 2038, hand-written
on tools/emu/hooks.py).

    uv run tools/emu/sweep_unit_kill_current_path.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret` (the frame is realigned to 8 and restored). With a path length at
`+0xc0` above zero it pops waypoints — the count decremented, then the popped entry's flags (the byte at `+0xc`
of the `0x10`-byte entry the array at `+0xb8` holds at the new count) tested for bit 0, the final waypoint — and
stops after a final one or at an empty stack; then it calls its one callee (hooked, `ecx` = `this`). A length of
zero or below does nothing. A row draws the length (-2..=16) and sixteen flag bytes, a third of them final.
Row: `<length> <calls> <ecx is this> <flags>×16 -> <length after>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def popped(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


ENTRY = 0x005E31D0
CALLEES = {0x005E3920: ("clear", 0)}
N = 16


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], CALLEES)
    uc = h.m.uc
    rng = random.Random(2038)
    this, arr = h.alloc(0x100), h.alloc(N * 0x10)
    for i in range(300):
        length = rng.choice((-2, -1, 0, 1, 2, N)) if rng.random() < 0.3 else rng.randrange(1, N + 1)
        flags = []
        for _ in range(N):
            f = rng.randrange(256)
            f = f | 1 if rng.random() < 0.33 else f & ~1
            flags.append(f)
        uc.mem_write(arr, bytes(rng.randrange(256) for _ in range(N * 0x10)))
        for k, f in enumerate(flags):
            h.w8(arr + k * 0x10 + 0xC, f)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x100)))
        h.w32(this + 0xB8, arr)
        h.w32(this + 0xC0, length)
        h.calls.clear()
        before = bytes(uc.mem_read(this, 0x100)), bytes(uc.mem_read(arr, N * 0x10))
        h.call(ENTRY, (), ecx=this)
        popped(uc)
        after = bytes(uc.mem_read(this, 0x100)), bytes(uc.mem_read(arr, N * 0x10))
        assert after[1] == before[1], "wrote the path"
        assert after[0][:0xC0] + after[0][0xC4:] == before[0][:0xC0] + before[0][0xC4:], "wrote past the length"
        ecx_this = int(all(c[1] == this for c in h.calls))
        print(" ".join(str(x) for x in (length, len(h.calls), ecx_this, *flags)) + f" -> {h.r32(this + 0xC0)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
