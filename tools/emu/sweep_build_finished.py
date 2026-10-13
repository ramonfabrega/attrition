#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_finished.py — `Build::finished@00470e50` under unicorn (item 1998, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_build_finished.py <install>/riseofnations.exe

`__thiscall`, one stack argument (an `int`). The body is `mov eax, [ecx]; jmp [eax+0x1b0]`:
the frame is torn down before the jump, so the slot is entered as an ordinary call — the caller's return address
at `[esp]`, the argument above it — and its own `ret 4` returns straight to the caller. The slot is a stub
that answers a value drawn per row and records its `ecx` and arguments (`sweep_gen.py vcall --tail` is the
`push; jmp` shape, not this one, so the script is by hand).
Row: `<answer> <p0> <handed p0> <ecx is this> -> <eax>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00470E50
SLOT, NARGS = 0x1B0, 1
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(1998)
    for i in range(220):
        v = I32[i] if i < len(I32) else rand32(rng)
        own = [rand32(rng) for _ in range(NARGS)]
        h.calls.clear()
        vt = h.vtable({SLOT: ("slot", NARGS, v)})
        obj = h.alloc(0x40)
        h.w32(obj, vt)
        r = h.call(ENTRY, own, ecx=obj)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert len(h.calls) == 1, h.calls
        _, ecx, args, _ = h.calls[0]
        print(" ".join(str(x) for x in (v, *own, *args, int(ecx == obj))) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
