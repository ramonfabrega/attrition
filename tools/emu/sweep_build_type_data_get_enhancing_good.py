#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_type_data_get_enhancing_good.py — `BuildTypeData::get_enhancing_good@00639880` under unicorn
(item 2083; `sweep_build_type_get_good.py`'s shape, with 1866's `esp` and untouched-bytes checks).

    uv run tools/emu/sweep_build_type_data_get_enhancing_good.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, a plain `ret`: `ecx` is the BuildTypeData and
the function reads one dword, the `TypeIndex` at `this+4`, subtracts 0x1a7 and
indexes a four-entry jump table (`0x6398b0`: 0, 1, 4, 5), anything else answering -1.
Prints one row per input, `<type_index, signed> -> <eax, signed>`:

    type_index   `this+4`, every value 0..0x2ff, then edges round the table,
                 negatives, 0x7fffffff, -0x80000000, and 200 seeded (1619)
                 32-bit values.

Stood in for: only a page of random bytes for `this`; nothing else is read, so nothing else
is laid out.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE, Image, Machine, signed  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00639880
THIS = 0x10000000


def inputs():
    out = list(range(0x300))
    out += [-1, -2, -0x1A7, -0x1A8, -0x100, -0x8000, 0x7FFFFFFF, -0x80000000, 0x80000001,
            0xFFFF, 0x10000, 0x1A7 + 4, 0x1A7 - 1,
            0x100 + 0x1A7, 0xFFFFFFFF - 0x1A7, 0x7FFFFE59, 0x80000000 + 0x1A7]
    rng = random.Random(2083)
    out += [rng.getrandbits(32) for _ in range(200)]
    return [signed(v & 0xFFFFFFFF) for v in out]


def main(argv):
    m = Machine(Image(argv[1]))
    m.uc.mem_map(THIS, 0x1000)
    rng = random.Random(2084)
    for t in inputs():
        page = bytearray(rng.randrange(256) for _ in range(0x1000))
        page[4:8] = struct.pack("<i", t)
        m.uc.mem_write(THIS, bytes(page))
        r = signed(m.call(ENTRY, (), ecx=THIS))
        esp = m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(m.uc.mem_read(THIS, 0x1000)) == bytes(page), "wrote the record"
        print(f"{t} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
