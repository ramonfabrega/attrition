#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_is_moving.py — `UnitData::is_moving@00610af0` under unicorn (item 2061, hand-written on
tools/emu/hooks.py and tools/emu/orderhead.py).

    uv run tools/emu/sweep_unit_data_is_moving.py <install>/riseofnations.exe

`__thiscall`, no argument, `ret`. An empty list answers 0 unasked; otherwise the cursor is rewound to the head
(orderhead.py), a null head order answers 0, and a real one's slot `+0x14` is entered by a `jmp` (hooked, tail,
no argument, `ecx` = the order) and answers for the function.
Row: `<head> <order> <answer> -> <eax> | <calls> | <cursor>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from orderhead import Layout  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00610AF0


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2061)
    lay = Layout(h)
    ans = [0]
    vt = h.vtable({0x14: ("move", 0, lambda ecx, args: ans[0], True)})
    for _ in range(300):
        has = rng.random() < 0.8
        order = rng.random() < 0.8
        ans[0] = rng.choice((0, 1, 1, 2, -1, 0x100))
        this = lay.lay(rng, vt, has, order)
        h.calls.clear()
        res = h.call(ENTRY, (), ecx=this)
        assert uc.reg_read(UC_X86_REG_ESP) == STACK_BASE + STACK_SIZE - 0x100, "wrong pop"
        calls = ",".join(f"{name}:{int(ecx == lay.order)}" for name, ecx, _, _ in h.calls)
        print(f"{int(has)} {int(order)} {ans[0]} -> {res} | {calls} | {lay.after()}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
