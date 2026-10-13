#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_good_type_data_is_flat.py — `GoodTypeData::is_flat@004780c0` under unicorn (item 1998, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_good_type_data_is_flat.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`. It calls its own slot `+0x60` (two stack arguments, `ret 8`) with
`(1, 0)`, then `(4, 0)`, then `(5, 0)`, stopping at the first non-zero answer (0), and answers 1 when all three
are zero. The slot is a stub whose answer is a function of its first argument, drawn per row so that each of
the three is zero about half the time; the script records every call.
Row: `<answer to 1> <answer to 4> <answer to 5> <calls made> <calls well-formed> -> <eax>` — a call is
well-formed when its `ecx` is `this` and its arguments are `(k, 0)` for the next `k` of 1, 4, 5.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x004780C0
I32 = (1, -1, 2, 0x7FFFFFFF, -0x80000000, 0x100, 255)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(1998)
    rows = [(a, b, c) for a in (0, 1) for b in (0, -1) for c in (0, 0x100)]
    for _ in range(220):
        rows.append(tuple(0 if rng.random() < 0.55 else (rng.choice(I32) if rng.random() < 0.5
                                                         else rng.randint(-0x80000000, 0x7FFFFFFF)) for _ in range(3)))
    for ans in rows:
        table = {1: ans[0], 4: ans[1], 5: ans[2]}
        h.calls.clear()
        vt = h.vtable({0x60: ("slot", 2, lambda ecx, args, t=table: t.get(args[0], 0x5A5A))})
        obj = h.alloc(0x40)
        h.w32(obj, vt)
        r = h.call(ENTRY, (), ecx=obj)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        ok = all(ecx == obj and args == [k, 0] for (_, ecx, args, _), k in zip(h.calls, (1, 4, 5)))
        print(" ".join(str(x) for x in (*ans, len(h.calls), int(ok))) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
