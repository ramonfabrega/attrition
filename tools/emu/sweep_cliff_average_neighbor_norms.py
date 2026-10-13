#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_cliff_average_neighbor_norms.py — `Cliff::average_neighbor_norms@008a4060` under unicorn (item 1889).

    uv run tools/emu/sweep_cliff_average_neighbor_norms.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object; `ret 12` (an empty body). Row: `<seed> -> <bytes of the object that changed>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x008a4060
NARGS = 3
OBJ = 0x10000000
MASK = 0xFFFFFFFF
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)
rng = random.Random(1889)
INTS = list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(120)]
WORDS = [w & MASK for w in INTS]
_noise = {}


def fill(uc, seed=0):
    """The object's first 0x1000 bytes, noise, remembered for `untouched`."""
    r = random.Random(seed or rng.randint(1, 1 << 30))
    data = bytes(r.randrange(256) for _ in range(0x1000))
    uc.mem_write(OBJ, data)
    _noise["d"] = bytearray(data)


def i32(uc, off):
    return struct.unpack("<i", bytes(uc.mem_read(OBJ + off, 4)))[0]


def untouched(uc, allowed):
    """No byte of the object changed outside `allowed` offsets."""
    now = bytes(uc.mem_read(OBJ, 0x1000))
    return all(now[i] == _noise["d"][i] for i in range(0x1000) if i not in allowed)


def call(m, args, ecx=OBJ):
    """Run it; a `ret N` that pops the wrong count leaves `esp` off the frame's top."""
    from callfn import STACK_BASE, STACK_SIZE
    r = m.call(ENTRY, args, ecx=ecx)
    esp = m.uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
    return r


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    for s in range(120):
        fill(uc, s)
        call(m, tuple(rng.randint(-99, 99) for _ in range(NARGS)))
        assert untouched(uc, set()), "wrote"
        print(f"{s} -> 0")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
