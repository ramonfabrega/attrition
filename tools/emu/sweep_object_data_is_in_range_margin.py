#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_data_is_in_range_margin.py — `ObjectData::is_in_range@00648d70` under unicorn (item 2020,
hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_object_data_is_in_range_margin.py <install>/riseofnations.exe

`__thiscall`, four stack arguments, `ret 0x10`. A forwarder: it de-XORs the object's own position — the words at
`this+0x10` and `this+0x14`, each `^ 0x63637` — and calls the seven-argument overload at 0x6486b0 (hooked, `ret
0x1c`) with `ecx` = `this` and `(p0, p1, x, y, this, p3, 0)`, answering what it answers: `p2` is never read. The
position words, the four arguments and the callee's answer are drawn per row; every byte of `this` is checked
untouched. The fifth handed argument is printed as 1 when it is `this`.
Row: `<word 0x10> <word 0x14> <p0> <p1> <p2> <p3> <answer> <ecx is this> <handed ×7> -> <eax>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00648D70
CALLEE = 0x006486B0
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x63637)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    rng = random.Random(2020)
    cur = {}
    h = Harness(argv[1], {CALLEE: ("overload", 7)})
    h.answers["overload"] = lambda ecx, args: cur["ans"]
    uc = h.m.uc
    this = h.alloc(0x40)
    for i in range(240):
        w10, w14, p0, p1, p2, p3 = (rand32(rng) for _ in range(6))
        cur["ans"] = rand32(rng)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x40)))
        h.w32(this + 0x10, w10)
        h.w32(this + 0x14, w14)
        before = bytes(uc.mem_read(this, 0x40))
        h.calls.clear()
        r = h.call(ENTRY, (p0, p1, p2, p3), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(this, 0x40)) == before, "wrote the object"
        assert len(h.calls) == 1, h.calls
        _, ecx, args, _ = h.calls[0]
        args = [int((a & 0xFFFFFFFF) == this) if k == 4 else a for k, a in enumerate(args)]
        print(" ".join(str(x) for x in (w10, w14, p0, p1, p2, p3, cur["ans"], int(ecx == this), *args))
              + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
