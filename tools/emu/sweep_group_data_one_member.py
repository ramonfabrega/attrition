#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_group_data_one_member.py — `GroupData::one_member@0046f820` under unicorn (item 1840).

    uv run tools/emu/sweep_group_data_one_member.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the group: a tail into the real `GroupData::get_num_cap_const@0070f960` (`num` at `+0xc`, `who` byte at `+0x4a`, `list` of shorts at `+0x8cc`; each entry indexes `objects[who]`'s pointer array at `0xc0ab84 + 0x1c*who`; an entry counts unless its object's flag dword at `+8` has bit 0 clear or its vtable slot `+0xe8` is 0); answers 1 when that count is exactly 1. Rows `<who> <num> <f0> <v0> … <f5> <v5>`: six objects, the list reading them in a fixed permutation.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046f820
OBJ = 0x10000000
ARG = 0x20000000
VT = 0x30000000
CODE = 0x31000000
TVT = 0x32000000
TYPE = OBJ + 0x8000
WORLD_CELL = 0x00C06188
VALS = (0, 1, 2, -1, 0x100, 0x7FFFFFFF, -0x80000000, 0xFF, -7, 12345)


def stub(uc, off, v, ret=0):
    """`mov eax, v; ret` (or `ret N`) at CODE+off; the address is fresh per row, as unicorn serves a stale block for rewritten code."""
    tail = b"\xc3" if not ret else b"\xc2" + struct.pack("<H", ret)
    uc.mem_write(CODE + off, b"\xb8" + struct.pack("<i", v) + tail)
    return CODE + off


def argstub(uc, off, v):
    """`eax = v + 1000*[esp+4] + [esp+8]; ret 8`: the answer carries the two stack arguments the caller pushed."""
    code = b"\x8b\x44\x24\x04" + b"\x69\xc0" + struct.pack("<i", 1000) + b"\x03\x44\x24\x08" + b"\x05" + struct.pack("<i", v) + b"\xc2\x08\x00"
    uc.mem_write(CODE + off, code)
    return CODE + off


PERM = (3, 0, 5, 1, 4, 2)
OBJS = 0x10001000
OBJTAB = 0x00C0AB84


def rows():
    out = []
    rng = random.Random(1840)
    for who in (0, 1, 7):
        for num in range(-1, 7):
            for _ in range(20):
                row = [who, num]
                for _ in range(6):
                    row += [rng.choice((0, 1, 2, 3, 0xFF, -1)), rng.choice((0, 0, 1, -1, 5))]
                out.append(tuple(row))
    # exact-one rows for every position
    for num in range(1, 7):
        for k in range(num):
            row = [0, num]
            for j in range(6):
                on = PERM[k] == j
                row += [1 if on else 0, 1 if on else 0]
            out.append(tuple(row))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARG, 0x1000)
    uc.mem_map(VT, 0x1000)
    uc.mem_map(CODE, 0x20000)
    uc.mem_map(TVT, 0x1000)
    for n, row in enumerate(rows()):
        who, num = row[0], row[1]
        tab = OBJTAB + 0x1C * who
        uc.mem_write(tab, struct.pack("<I", OBJS))
        uc.mem_write(OBJ + 0xC, struct.pack("<i", num))
        uc.mem_write(OBJ + 0x4A, struct.pack("<B", who))
        uc.mem_write(OBJ + 0x8CC, struct.pack("<6h", *PERM))
        for k in range(6):
            f, v = row[2 + 2 * k], row[3 + 2 * k]
            o = OBJS + 0x1000 + k * 0x100
            ovt = VT + k * 0x100
            uc.mem_write(OBJS + 4 * k, struct.pack("<I", o))
            uc.mem_write(o, struct.pack("<I", ovt))
            uc.mem_write(o + 8, struct.pack("<i", f))
            uc.mem_write(ovt + 0xE8, struct.pack("<I", stub(uc, (n * 6 + k) * 0x10, v)))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
