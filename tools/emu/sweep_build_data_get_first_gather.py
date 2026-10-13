#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_data_get_first_gather.py — `BuildData::get_first_gather@0046f140` under unicorn (item 2090,
hand-written on tools/emu/hooks.py; `sweep_unit_data_is_idle.py`'s iterator on the gather list).

    uv run tools/emu/sweep_build_data_get_first_gather.py <install>/riseofnations.exe

`__thiscall`, no stack arguments. The gather list at `+0xb8`: its head `+0xcc` null, or its count `+0xc8` not
positive (signed), answers 0 with nothing written. Otherwise the iterator is reset to the head itself (no
sentinel): `+0xc4` the node, `+0xbc` the node's data (`+8`), the byte `+0xc0` the node's `+0xc`; it answers the
data, a point laid out per row. The building's 0x100 bytes are random.
Row: `<head 0|1> <count> <node> <data> -> <eax> | <before hex> | <after hex> | <node hex>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0046F140
WIN = 0x100
COUNTS = (0, 1, 2, 5, -1, 0x7FFFFFFF, -0x80000000)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2090)
    bld = h.alloc(WIN + 0x20)
    node = h.alloc(0x20)
    data = h.alloc(0x20)
    for i in range(400):
        has_head = rng.random() < 0.8
        count = rng.choice(COUNTS) if rng.random() < 0.6 else rng.randint(-0x80000000, 0x7FFFFFFF)
        mem = bytearray(rng.randrange(256) for _ in range(WIN))
        mem[0xC8:0xD0] = struct.pack("<iI", count, node if has_head else 0)
        uc.mem_write(bld, bytes(mem))
        uc.mem_write(node, bytes(rng.randrange(256) for _ in range(0x10)))
        h.w32(node + 8, data)
        guard = bytes(uc.mem_read(bld + WIN, 0x20))
        r = h.call(ENTRY, (), ecx=bld)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(bld + WIN, 0x20)) == guard, "wrote past the window"
        after = bytes(uc.mem_read(bld, WIN))
        nodebytes = bytes(uc.mem_read(node, 0x10))
        print(f"{int(has_head)} {count} {node} {data} -> {r} | {bytes(mem).hex()} | {after.hex()} | {nodebytes.hex()}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
