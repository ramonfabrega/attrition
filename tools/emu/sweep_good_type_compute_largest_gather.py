#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_good_type_compute_largest_gather.py — `GoodType::compute_largest_gather@0066e920` under unicorn.

    uv run tools/emu/sweep_good_type_compute_largest_gather.py <install>/riseofnations.exe

The function is `__thiscall`, `void`: it reads `this+4` (the good's type
index), walks the global `lands` array (the pointer at 0xe3a380 = `lands+16`;
nine records of stride 0x138, each four good indices at +0x04..+0x10 and four
amounts at +0x14..+0x20), skips records 1 and 2, and writes `this+0x2ec`,
clamped to 1..2. Void, so the row's result is the word written at `this+0x2ec`.

One row per input, `<good> <noise> -> <this+0x2ec after the call>`:

    good   the signed value at `this+4`: -5..40 (every comparison edge: -1 is
           TYPE_NONE, what the empty cells hold; 0..5 the six goods; 6.. none)
           plus a few large and random values
    noise  0 lays out the shipped table (rules.xml `<LAND>`s, the port's
           `world::LANDS`); n > 0 seeds (n) a junk fill of records 1 and 2
           (the two the original skips) with goods and amounts of any size,
           matching `good` often, and of every byte of each record outside
           the eight cells

What is stood in for: the `lands` array is laid out by hand in a mapped page
rather than built by `Lands::init`, from the table the port holds; nothing
else is read. The port's table is the shipped one and is hardwired, so the
rows vary the good and the junk the original must skip — a table whose lands
other than 1 and 2 differ is not a row, as the port could not be asked.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0066E920
G_LANDS = 0x00E3A380
THIS, LANDS = 0x10000000, 0x10010000
STRIDE, N = 0x138, 9

# (goods[4], amounts[4]) per land, in file order — the port's `world::LANDS`.
SHIPPED = [
    ([3, 0, -1, -1], [1, 1, 0, 0]),
    ([-1] * 4, [0] * 4),
    ([-1] * 4, [0] * 4),
    ([-1] * 4, [0] * 4),
    ([1, -1, -1, -1], [1, 0, 0, 0]),
    ([4, -1, -1, -1], [1, 0, 0, 0]),
    ([-1] * 4, [0] * 4),
    ([5, -1, -1, -1], [1, 0, 0, 0]),
    ([4, -1, -1, -1], [1, 0, 0, 0]),
]


def inputs():
    rng = random.Random(1619)
    out = []
    for noise in range(8):
        for good in range(-5, 41):
            out.append((good, noise))
    for _ in range(80):
        good = rng.choice((rng.randint(-1000, 1000), rng.randint(-(2**31), 2**31 - 1),
                           rng.randint(0, 5)))
        out.append((good, rng.randint(0, 40)))
    out += [(2**31 - 1, 3), (-(2**31), 3), (6, 9), (5, 9)]
    return out


def lay_out(m, good, noise):
    rng = random.Random(noise)
    table = bytearray(0x2000)
    for i in range(N):
        rec = i * STRIDE
        if noise:
            table[rec:rec + STRIDE] = bytes(rng.getrandbits(8) for _ in range(STRIDE))
        goods, amounts = SHIPPED[i]
        if noise and i in (1, 2):
            goods = [good if rng.random() < 0.6 else rng.randint(-3, 8) for _ in range(4)]
            amounts = [rng.choice((0, 1, 2, 3, 99, 2**31 - 1, -1, -(2**31), rng.randint(-50, 50)))
                       for _ in range(4)]
        struct.pack_into("<4i", table, rec + 4, *goods)
        struct.pack_into("<4i", table, rec + 0x14, *amounts)
    m.uc.mem_write(LANDS, bytes(table))


def main(argv):
    m = Machine(Image(argv[1]))
    m.uc.mem_map(THIS, 0x1000)
    m.uc.mem_map(LANDS, 0x2000)
    m.uc.mem_write(G_LANDS, struct.pack("<I", LANDS))
    for good, noise in inputs():
        lay_out(m, good, noise)
        m.uc.mem_write(THIS, bytes(0x1000))
        m.uc.mem_write(THIS + 4, struct.pack("<i", good))
        m.uc.mem_write(THIS + 0x2EC, struct.pack("<i", 0x5A5A5A5A))   # the function must overwrite it
        m.call(ENTRY, (), ecx=THIS)
        got = struct.unpack("<i", bytes(m.uc.mem_read(THIS + 0x2EC, 4)))[0]
        print(good, noise, "->", got)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
