#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_wcoord_to_tcoord.py — `WCoord::operator_TCoord@004613b0` under unicorn.

    uv run tools/emu/sweep_wcoord_to_tcoord.py <install>/riseofnations.exe

`__thiscall`, `ret 4`: `this` (ecx) points at a `WCoord` whose one field is
`value` at +0; the stack argument is the address of the `TCoord` the function
writes (`*out = value * 4 + 2`) and returns in eax. One row per input,
`<value> -> <the int written through the out pointer>`; eax is checked
to be the out pointer itself.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine  # noqa: E402

ENTRY = 0x004613B0
PAGE = 0x1000_0000
THIS, OUT = PAGE, PAGE + 0x100


def signed(v):
    return v - (1 << 32) if v & 0x8000_0000 else v


# The domain is the values whose `value * 4 + 2` fits an int (|value| < 2**29):
# the original wraps past it in 32 bits, the port's `i32` arithmetic would not,
# and no cell coordinate comes near it.
def inputs(seed=424242, n=400):
    rng = random.Random(seed)
    edges = [0, 1, 2, 3, 4, 5, -1, -2, -3, -4, -5, 191, 192, 255, 256, 1000, -1000,
             65_535, 65_536, -65_536, 536_870_911, -536_870_912,
             0x1FFF_FFFE, -0x1FFF_FFFE]
    vals = list(edges)
    for _ in range(n):
        lim = 200 if rng.random() < 0.5 else 100_000 if rng.random() < 0.8 else 0x1FFF_FFFF
        vals.append(rng.randint(-lim, lim))
    return vals


def main(argv):
    m = Machine(Image(argv[1]))
    m.uc.mem_map(PAGE, 0x1000)
    for v in inputs():
        m.uc.mem_write(THIS, struct.pack("<i", v))
        m.uc.mem_write(OUT, struct.pack("<i", 0x5A5A5A5A))
        eax = m.call(ENTRY, (OUT,), ecx=THIS)
        assert eax == OUT, f"eax {eax:#x} is not the out pointer"
        out = struct.unpack("<i", bytes(m.uc.mem_read(OUT, 4)))[0]
        print(f"{v} -> {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
