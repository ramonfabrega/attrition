#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_balance_walk_rules_data.py — `Balance::walk_rules_data@00582cc0` under unicorn (item 2672, tools/sweep_gen.py, walk).

    uv run tools/emu/sweep_balance_walk_rules_data.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object, one stack argument: the walker. The function calls
the walker's callback (the first word of the walker's vtable) with `ecx` = the walker
and two stack arguments, `0xc12bf4` and `0xc12bf6` of the object. One row per input,
`<this> <walker> -> <result> | <ecx,arg0,arg1>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00582cc0
ROWS = 3
# `--count-only` (item 2672, parked 2667): a walker called a quarter-million times
# is one record, `<count> <first> <last> <sum a0> <sum a1>` (sums mod 2**32).
COUNT_ONLY = True


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    rng = random.Random(2672)
    for _ in range(ROWS):
        this = rng.randrange(0x1000, 0x7FFF0000) & ~3
        ans = rng.randint(-0x80000000, 0x7FFFFFFF)
        vt = h.vtable({0: ("walk", 2, ans & 0xFFFFFFFF)})
        walker = h.alloc(16)
        h.w32(walker, vt)
        h.calls.clear()
        r = h.call(ENTRY, (walker,), ecx=this, max_insns=200_000_000 if COUNT_ONLY else 200_000)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        if COUNT_ONLY:
            fmt = lambda c: ",".join(str(x) for x in (c[1], *c[2]))
            seen = " ".join([str(len(h.calls)), fmt(h.calls[0]), fmt(h.calls[-1]),
                             str(sum(c[2][0] & 0xFFFFFFFF for c in h.calls) & 0xFFFFFFFF),
                             str(sum(c[2][1] & 0xFFFFFFFF for c in h.calls) & 0xFFFFFFFF)])
        else:
            seen = ";".join(",".join(str(x) for x in (ecx, *args)) for _, ecx, args, _ in h.calls)
        print(f"{this} {walker} -> {r} | {seen}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

# sweep_gen: ["walk", "Balance::walk_rules_data@00582cc0", "--item", "2672", "--a0", "0xc12bf4", "--a1", "0xc12bf6", "--count-only", "--rows", "3", "--port", "(0..0x1edi64 * 0x1ed).map(|k| (0xc12bf4 + 2 * k, 0xc12bf6 + 2 * k))"]
