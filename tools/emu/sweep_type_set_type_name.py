#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_type_set_type_name.py — `Type::set_type_name@00663b70` under unicorn (item 1942).

    uv run tools/emu/sweep_type_set_type_name.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the type, one stack argument (`ret 4`, popped by the callee): a `String` whose length word sits at `+8`. When that word is zero the function substitutes
`this+0x60`; either way it enters `String::operator=@00a1eeb0` (hooked) with `ecx` = `this+0xb0` and the chosen string. One row per input,
`<length dword at +8> <p0 - this> -> | <ecx - this,arg - this>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00663b70
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x3F800000, 0x7FC00000)


def check_esp(h):
    esp = h.m.uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def calls_str(h, base=0):
    return ";".join(",".join(str(x) for x in (ecx - base if i == 0 else ecx, *args)) for i, (_, ecx, args, _) in enumerate(h.calls))


def main(argv):
    from hooks import Harness
    rng = random.Random(1942)
    h = Harness(argv[1], {0xA1EEB0: ("op_assign", 1)})
    lens = [0, 1, 0x10000, 0x20000, 0xFFFF0000, 0x7FFF, 0x8000, 0xFFFF, 0x10001]
    rows = [rng.choice(lens + [rng.randrange(1 << 32)]) for _ in range(190)]
    for ln in rows:
        this = h.alloc(0x100)
        h.m.uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x100)))
        p0 = h.alloc(0x40)
        h.m.uc.mem_write(p0, bytes(rng.randrange(256) for _ in range(0x40)))
        h.m.uc.mem_write(p0 + 8, struct.pack("<I", ln))
        h.calls.clear()
        h.call(ENTRY, (p0,), ecx=this)
        check_esp(h)
        assert len(h.calls) == 1
        _, ecx, args, _ = h.calls[0]
        print(f"{ln} {p0 - this} -> | {ecx - this},{args[0] - this}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
