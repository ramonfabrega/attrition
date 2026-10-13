#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_is_packing.py — `UnitData::is_packing@0060aa60` under unicorn (item 2061, hand-written on
tools/emu/hooks.py and tools/emu/orderhead.py).

    uv run tools/emu/sweep_unit_data_is_packing.py <install>/riseofnations.exe

`__thiscall`, no argument, `ret`. An empty list answers 0 unasked; otherwise the cursor is rewound to the head
(orderhead.py) and a null head order answers 0. A real one's slot `+0xf4` (hooked, no argument, `ecx` = the
order) answers its spell record or null; null answers 0, else the slot is asked again, the record's `+0x20`
indexes the type list (the cell `0xe85ddc` names an array of record pointers) and that type's slot `0x50`
(hooked, no argument, `ecx` = the type) answers; any non-zero is 1.
Row: `<head> <order> <spell?> <index> <answer> -> <eax> | <calls> | <cursor>`, the calls `<name>:<ecx is the one
expected>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from orderhead import Layout  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0060aa60
TYPES = 0x00E85DDC
SLOT = 0x50


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2061)
    lay = Layout(h)
    spell = h.alloc(0x40)
    table = h.alloc(0x40 * 4)
    ty = h.alloc(0x20)
    other = h.alloc(0x20)
    st = {"spell": 0, "ans": 0}
    vt = h.vtable({0xF4: ("spell", 0, lambda ecx, args: st["spell"])})
    tvt = h.vtable({SLOT: ("asks", 0, lambda ecx, args: st["ans"])})
    ovt = h.vtable({SLOT: ("other", 0, lambda ecx, args: 0x55)})
    h.w32(ty, tvt)
    h.w32(other, ovt)
    for i in range(0x40):
        h.w32(table + 4 * i, other)
    h.w32(TYPES, table)
    for _ in range(300):
        has = rng.random() < 0.85
        order = rng.random() < 0.85
        has_spell = rng.random() < 0.8
        idx = rng.randrange(0x40)
        st["ans"] = rng.choice((0, 0, 1, 2, -1, 0x100))
        this = lay.lay(rng, vt, has, order)
        uc.mem_write(spell, bytes(rng.randrange(256) for _ in range(0x40)))
        h.w32(spell + 0x20, idx)
        st["spell"] = spell if has_spell else 0
        h.w32(table + 4 * idx, ty)
        h.calls.clear()
        res = h.call(ENTRY, (), ecx=this)
        assert uc.reg_read(UC_X86_REG_ESP) == STACK_BASE + STACK_SIZE - 0x100, "wrong pop"
        h.w32(table + 4 * idx, other)
        want = {"spell": lay.order, "asks": ty, "other": None}
        calls = ",".join(f"{name}:{int(ecx == want[name])}" for name, ecx, _, _ in h.calls)
        print(f"{int(has)} {int(order)} {int(has_spell)} {idx} {st['ans']} -> {res} | {calls} | {lay.after()}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
