#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_order_type.py — `UnitData::order_type@00616e80` under unicorn (item 2061, hand-written on
tools/emu/hooks.py and tools/emu/orderhead.py).

    uv run tools/emu/sweep_unit_data_order_type.py <install>/riseofnations.exe

`__thiscall`, no argument, `ret`. An empty list (`this+0xdc` null) answers 0 unasked; otherwise the cursor is
rewound to the head (orderhead.py) and the head order's slot `+0x10` is entered by a `jmp` (hooked, tail, no
argument, `ecx` = the order) and its answer is the function's. A head with a null order is not drawn: the jump
reads through it.
Row: `<head> <kind> -> <eax> | <calls> | <cursor>`, the calls `<name>:<ecx is the order>`, the cursor orderhead's
three marks.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from orderhead import Layout  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00616E80


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2061)
    lay = Layout(h)
    kind = [0]
    vt = h.vtable({0x10: ("kind", 0, lambda ecx, args: kind[0], True)})
    for _ in range(300):
        has = rng.random() < 0.8
        kind[0] = rng.randrange(0x40) if rng.random() < 0.8 else rng.randint(-0x80000000, 0x7FFFFFFF)
        this = lay.lay(rng, vt, has, True)
        h.calls.clear()
        res = h.call(ENTRY, (), ecx=this)
        assert uc.reg_read(UC_X86_REG_ESP) == STACK_BASE + STACK_SIZE - 0x100, "wrong pop"
        calls = ",".join(f"{name}:{int(ecx == lay.order)}" for name, ecx, _, _ in h.calls)
        print(f"{int(has)} {kind[0]} -> {res} | {calls} | {lay.after()}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
