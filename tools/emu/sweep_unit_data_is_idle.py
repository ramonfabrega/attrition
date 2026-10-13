#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_is_idle.py — `UnitData::is_idle@0046fa40` and, through `sweep_unit_data_is_fleeing.py`,
`UnitData::is_fleeing@0046efa0` under unicorn (item 2090, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_unit_data_is_idle.py <install>/riseofnations.exe

`__thiscall`, no stack arguments. The order list at `+0xc8`: its head `+0xdc` null answers `is_idle` 1 and
`is_fleeing` 0 with nothing written. Otherwise the iterator is reset to the first node (`[head+4]`): `+0xd4` the
node, `+0xcc` the node's data (`+8`), the byte `+0xd0` the node's `+0xc`; `is_idle` answers the data null, and
`is_fleeing` asks the data's slot `+0x10` (`get_type`, a stub answered per row) and answers it `== 4`
(`FLEE_TO`). The unit's 0x100 bytes are random; the node is laid out per row.
Row: `<head 0|1> <data 0|1> <get_type> <node> <data> -> <eax> | <before hex> | <after hex> | <node hex> | <calls>`,
a call `data|other`; `<node>` and `<data>` are the addresses the iterator copies.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

IDLE, FLEEING = 0x0046FA40, 0x0046EFA0
WIN = 0x100
I32 = (0, 1, -1, 2, 3, 4, 5, 0x7FFFFFFF, -0x80000000, 0x104)


def main(argv, fleeing=False):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2090 + fleeing)
    ans = [0]
    vt = h.vtable({0x10: ("get_type", 0, lambda ecx, args: ans[0])})
    unit = h.alloc(WIN + 0x20)
    sentinel = h.alloc(0x20)
    node = h.alloc(0x20)
    data = h.alloc(0x20)
    h.w32(data, vt)
    for i in range(400):
        has_head = rng.random() < 0.8
        has_data = fleeing or rng.random() < 0.6
        ans[0] = 4 if rng.random() < 0.4 else (rng.choice(I32) if rng.random() < 0.5 else rng.randint(-0x80000000, 0x7FFFFFFF))
        mem = bytearray(rng.randrange(256) for _ in range(WIN))
        mem[0xDC:0xE0] = struct.pack("<I", sentinel if has_head else 0)
        uc.mem_write(unit, bytes(mem))
        h.w32(sentinel + 4, node)
        h.w32(node + 8, data if has_data else 0)
        h.w8(node + 0xC, rng.randrange(256))
        guard = bytes(uc.mem_read(unit + WIN, 0x20))
        h.calls.clear()
        r = h.call(FLEEING if fleeing else IDLE, (), ecx=unit)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(unit + WIN, 0x20)) == guard, "wrote past the window"
        after = bytes(uc.mem_read(unit, WIN))
        nodebytes = bytes(uc.mem_read(node, 0x10))
        calls = ";".join("data" if ecx == data else "other" for _, ecx, _, _ in h.calls)
        print(f"{int(has_head)} {int(has_data)} {ans[0]} {node} {data} -> {r} | {bytes(mem).hex()} | {after.hex()} | {nodebytes.hex()} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
