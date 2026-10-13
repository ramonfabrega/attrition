#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_data_is_in_range_this.py — `ObjectData::is_in_range@0064e470` under unicorn (item 2020,
hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_object_data_is_in_range_this.py <install>/riseofnations.exe

`__thiscall`, two stack arguments, `ret 8`. A forwarder: it de-XORs the object's own position — the words at
`this+0x10` and `this+0x14`, each `^ 0x63637` — and calls the four-argument overload at 0x64e4a0 (hooked, `ret
0x10`) with `ecx` = `this` and `(p0, p1, x, y)`, answering what it answers. The position words, both arguments
and the callee's answer are drawn per row; every byte of `this` is checked untouched.
Row: `<word 0x10> <word 0x14> <p0> <p1> <answer> <ecx is this> <handed ×4> -> <eax>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0064E470
CALLEE = 0x0064E4A0
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x63637)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    rng = random.Random(2020)
    cur = {}
    h = Harness(argv[1], {CALLEE: ("overload", 4)})
    h.answers["overload"] = lambda ecx, args: cur["ans"]
    uc = h.m.uc
    this = h.alloc(0x40)
    for i in range(240):
        w10, w14, p0, p1 = (rand32(rng) for _ in range(4))
        cur["ans"] = rand32(rng)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x40)))
        h.w32(this + 0x10, w10)
        h.w32(this + 0x14, w14)
        before = bytes(uc.mem_read(this, 0x40))
        h.calls.clear()
        r = h.call(ENTRY, (p0, p1), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(this, 0x40)) == before, "wrote the object"
        assert len(h.calls) == 1, h.calls
        _, ecx, args, _ = h.calls[0]
        print(" ".join(str(x) for x in (w10, w14, p0, p1, cur["ans"], int(ecx == this), *args)) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
