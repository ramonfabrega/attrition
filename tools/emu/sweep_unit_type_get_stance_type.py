#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_type_get_stance_type.py — `UnitTypeData::get_stance_type@0061d350` under unicorn.

    uv run tools/emu/sweep_unit_type_get_stance_type.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the type record, a plain `ret`. One row per input,
`<role> <unit_flags2> <type_index> -> <StanceTypes>`:

    role         the type's `role` (`type+0x2c8`), any 32-bit word; bit 0x10000
                 is the military bit the function tests
    unit_flags2  the type's `unit_flags2` (`type+0x2b8`), any 32-bit word; the
                 original reads only its low byte (`& 4`, `& 6`)
    type_index   the dword at `type+4` (the decompile's `_padding_`), signed

The answer is the raw `eax`, signed: 0 combat, 1 worker, 2 caster, 3 packer,
-1 none. The record is one mapped page with the three words set and nothing
else stood in for: the function reads no other field and calls nothing.

Edges: every combination of the two flag bits the function reads (all 256
low-byte values of `unit_flags2`, with and without high bits) against
military / non-military roles (and role words with other bits and with
0x10000 alone), against the index boundaries 0x31..=0x36 and around them.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0061D350
THIS = 0x10000000


def inputs(seed=1619):
    roles = (0, 0x10000, 0xFFFEFFFF, 0xFFFFFFFF, 0x8000, 0x20000, 0x1, 0x10001)
    indices = (0, 1, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x60, -1, -0x32,
               0x7FFFFFFF, -0x80000000)
    out = []
    for role in (0, 0x10000):
        for f2 in range(256):
            for idx in (0x31, 0x32, 0x35, 0x36):
                out.append((role, f2, idx))
    for role in roles:
        for f2 in (0, 2, 4, 6, 0x100, 0x102, 0x104, 0x106, 0xFFFFFF00, 0xFFFFFFFF):
            for idx in indices:
                out.append((role, f2, idx))
    rng = random.Random(seed)
    for _ in range(400):
        out.append((rng.getrandbits(32), rng.getrandbits(32),
                    rng.choice((rng.randint(0x2E, 0x39), rng.randint(-0x80000000, 0x7FFFFFFF)))))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    m.uc.mem_map(THIS, 0x1000)
    for role, f2, idx in inputs():
        m.uc.mem_write(THIS + 0x2C8, struct.pack("<I", role))
        m.uc.mem_write(THIS + 0x2B8, struct.pack("<I", f2))
        m.uc.mem_write(THIS + 0x4, struct.pack("<i", idx))
        print(f"{role} {f2} {idx} -> {signed(m.call(ENTRY, (), ecx=THIS))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
