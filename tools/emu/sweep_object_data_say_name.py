#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_data_say_name.py — `ObjectData::say_name@00645310` under unicorn (item 1942).

    uv run tools/emu/sweep_object_data_say_name.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object, one stack argument (`ret 4`): a string. The function calls `String::operator+=@00a1d440` (hooked) with `ecx` = the argument and the string at `[this+0x18]+0x74`
(the pointer plus a constant, never dereferenced) as its one argument. One row per input, `<ptr at this+0x18> <p0> -> | <ecx,arg>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00645310
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x3F800000, 0x7FC00000)


def check_esp(h):
    esp = h.m.uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def calls_str(h, base=0):
    return ";".join(",".join(str(x) for x in (ecx - base if i == 0 else ecx, *args)) for i, (_, ecx, args, _) in enumerate(h.calls))


def main(argv):
    from hooks import Harness
    rng = random.Random(1942)
    h = Harness(argv[1], {0xA1D440: ("op_plus_eq", 1)})
    rows = [(rng.choice(I32), rng.choice(I32)) for _ in range(40)] + [(rng.randint(-0x80000000, 0x7FFFFFFF), rng.randint(-0x80000000, 0x7FFFFFFF)) for _ in range(150)]
    for ptr, p0 in rows:
        this = h.alloc(0x100)
        h.m.uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x100)))
        h.w32(this + 0x18, ptr)
        h.calls.clear()
        h.call(ENTRY, (p0,), ecx=this)
        check_esp(h)
        print(f"{ptr} {p0} -> | {calls_str(h)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
