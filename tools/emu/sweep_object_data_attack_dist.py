#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_data_attack_dist.py — `ObjectData::attack_dist@0064c880` under unicorn (item 1966, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_object_data_attack_dist.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 64-byte random object, two stack arguments. It calls the five-argument overload
`@006488f0` (hooked; `ecx` unchanged, four stack arguments) with `p0, p1` and the object's two words at `+0x10`, `+0x14`
each XOR 0x63637 (the point the original keeps encoded), and answers what that returns.
Row: `<ans> <p0> <p1> | <object hex> -> <eax> | <ecx,a0,a1,a2,a3>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0064c880
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {0x006488F0: ("attack_dist5", 4)})
    rng = random.Random(1966)
    for _ in range(240):
        this = h.alloc(0x60)
        mem = bytearray(rng.randrange(256) for _ in range(0x40))
        ans, p0, p1 = rand32(rng), rand32(rng), rand32(rng)
        h.m.uc.mem_write(this, bytes(mem))
        h.answers["attack_dist5"] = ans & 0xFFFFFFFF
        h.calls.clear()
        r = h.call(ENTRY, (p0, p1), ecx=this)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        seen = ";".join(",".join(str(x) for x in (ecx, *args)) for _, ecx, args, _ in h.calls)
        print(f"{ans} {p0} {p1} | {bytes(mem).hex()} -> {r} | {seen}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
