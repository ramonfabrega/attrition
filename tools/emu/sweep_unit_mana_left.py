#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_mana_left.py — `UnitData::mana_left@00609a30` under unicorn.

    uv run tools/emu/sweep_unit_mana_left.py <install>/riseofnations.exe

One row per input, `<mana> <burn> <owner> <french> <general> <fsc> <nonation>
<town> <city> -> <mana_left>`:

    mana      the type's MANA (`type+0x2ec`), any int
    burn      `this+0x96` mana_burn, a short
    owner     `this+9`, indexes the leader array (stride 0x6eec at 0xc061e0)
    french    the owner's tribe's racial power is 10 (1) or 5 (0)
    general   the type's virtual `is(0x36, 1)` (vtable+0x60) answers 1 or 0
    fsc       `constantsc->french_special_craft` (+0x69c at 0xc061e4)
    nonation  `gamec->info.flags & 4` (+0x20 at 0xc061e8)
    town      `game->info.starting_town` (+0x2c at 0xc061ec)
    city      the leader's `city_num` (+0x3f8)

`mana_left` calls `mana@00609a50`, which reads the leader array, the two
constants records and the tribe table through singletons; this script lays
those out in mapped pages and the real `has_tribe_bonus@006e1370` runs on
them. Two things are stood in for: the type's `is(0x36, 1)` is a virtual call
through `type->vtable+0x60`, answered by a four-byte stub that returns a
word we set; and the supply arm (`type+0x2b8 & 0x40`, then
`get_supply_upgrade`) and the space-air arm (`type+0x218 == 2`) are left
unreached (flag clear, kind 0), as the port leaves them as SEAMs.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00609A30
IS_SUPPLY, OBJECT_IS = 0x0046CE80, 0x00653790
G_LEADERS, G_CONSTANTS, G_GAMEC, G_GAME, G_TRIBES = 0xC061E0, 0xC061E4, 0xC061E8, 0xC061EC, 0xE7FA34

THIS, VT, TYPE, TVT = 0x10000000, 0x10001000, 0x10002000, 0x10003000
STUB, ANSWER = TVT + 0x800, TVT + 0xFF0
GAMEC, GAME, CONSTS = 0x10004000, 0x10005000, 0x10006000
LEADERS, TRIBES = 0x10010000, 0x10020000
TRIBE = 2


def w32(m, a, v):
    m.uc.mem_write(a, struct.pack("<i", v))


def setup(m):
    for a, n in ((THIS, 0x1000), (VT, 0x1000), (TYPE, 0x1000), (TVT, 0x1000), (GAMEC, 0x1000),
                 (GAME, 0x1000), (CONSTS, 0x1000), (LEADERS, 0x10000), (TRIBES, 0x2000)):
        m.uc.mem_map(a, n)
    w32(m, THIS, VT)                        # this->vtable
    w32(m, VT + 0xCC, IS_SUPPLY)            # the slots `mana` compares against
    w32(m, VT + 0xB8, OBJECT_IS)
    w32(m, THIS + 0x18, TYPE)               # this->type
    w32(m, TYPE, TVT)                       # type->vtable
    w32(m, TVT + 0x60, STUB)
    m.uc.mem_write(STUB, bytes([0xA1]) + struct.pack("<I", ANSWER) + bytes([0xC2, 0x08, 0x00]))
    for g, v in ((G_LEADERS, LEADERS), (G_CONSTANTS, CONSTS), (G_GAMEC, GAMEC), (G_GAME, GAME),
                 (G_TRIBES, TRIBES)):
        w32(m, g, v)


def rows():
    rng = random.Random(1575)
    out = []
    for mana in (0, 1, 7, 1000, 99_999, -5):
        for burn in (0, 1, 6, 999, 1000, 1001, 32_767, -7):
            for fsc in (0, 20):
                out.append((mana, burn, 1, 1, 1, fsc, 0, 1, 1))
    for _ in range(600):
        mana = rng.choice((0, rng.randint(-50, 5000), rng.randint(0, 100_000)))
        burn = rng.choice((0, rng.randint(-100, 6000), rng.randint(-32768, 32767)))
        out.append((mana, burn, rng.randint(0, 1), rng.randint(0, 1), rng.randint(0, 1),
                    rng.choice((0, rng.randint(-99, 500))), int(rng.random() < 0.2),
                    rng.randint(0, 1), rng.randint(0, 1)))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    setup(m)
    for r in rows():
        mana, burn, owner, french, general, fsc, nonation, town, city = r
        w32(m, TYPE + 0x2EC, mana)
        w32(m, TYPE + 0x218, 0)
        w32(m, TYPE + 0x2B8, 0)
        m.uc.mem_write(THIS + 9, bytes([owner]))
        m.uc.mem_write(THIS + 0x96, struct.pack("<h", burn))
        w32(m, ANSWER, general)
        w32(m, CONSTS + 0x69C, fsc)
        w32(m, GAMEC + 0x20, 4 if nonation else 0)
        m.uc.mem_write(GAME + 0x2C, bytes([town]))
        leader = LEADERS + owner * 0x6EEC
        m.uc.mem_write(LEADERS, bytes(0x10000))
        w32(m, leader + 4, 0)
        w32(m, leader + 0xC, TRIBE)
        w32(m, leader + 0x3F8, city)
        w32(m, TRIBES + TRIBE * 0x5F0 + 0x54, 10 if french else 5)
        print(" ".join(map(str, r)), "->", signed(m.call(ENTRY, (), ecx=THIS)))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
