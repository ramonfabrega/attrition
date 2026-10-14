#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_supply_data_get_radius.py — `SupplyData::get_radius@0073b560` under unicorn (item 2112, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_supply_data_get_radius.py <install>/riseofnations.exe

`__thiscall`, no stack arguments. The constants at `[0xc061e4]`: `supply_radius` (`+0xcac`) plus
`supply_radius_upgrade` (`+0xcb0`) times `LeaderData::get_supply_upgrade@006e0880` (hooked) on
`[0xc061e0] + 0x6eec · (signed byte at +0x5)`, plus `terra_cotta_range` (`+0x45c`) when
`LeaderData::has_wonder@006ebc10` (hooked, one argument) answers non-zero for `0x211` on the same leader. The
constants are drawn within ±0x10000 and the upgrade answer within ±0x4000, so no product wraps.
Row: `<radius> <per upgrade> <terra cotta> <who> <leaders> <upgrades> <wonder> -> <eax> | <calls>`, a call
`upgrade,<ecx>` or `wonder,<ecx>,<arg>`; the source's 0x40 bytes and the constants' 0x1000 are random and held
unwritten.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0073B560
GET_SUPPLY_UPGRADE = 0x006E0880
HAS_WONDER = 0x006EBC10
CONSTANTS = 0x00C061E4
LEADERS = 0x00C061E0
SMALL = (0, 1, -1, 2, 3, 8, 12, 0x10000, -0x10000)


def small(rng, lim):
    return rng.choice(SMALL[:-2] + (lim, -lim)) if rng.random() < 0.4 else rng.randint(-lim, lim)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {GET_SUPPLY_UPGRADE: ("upgrade", 0), HAS_WONDER: ("wonder", 1)})
    uc = h.m.uc
    rng = random.Random(2112)
    this = h.alloc(0x40)
    consts = h.alloc(0x1000)
    for i in range(400):
        radius, per, terra = small(rng, 0x10000), small(rng, 0x10000), small(rng, 0x10000)
        who = rng.randint(-128, 127)
        leaders = rng.randint(0, 0xFFFFFFFF)
        upgrades = small(rng, 0x4000)
        wonder = 0 if rng.random() < 0.5 else rng.choice((1, -1, 0x7FFFFFFF, rng.randint(-0x80000000, 0x7FFFFFFF)))
        h.answers["upgrade"], h.answers["wonder"] = upgrades, wonder
        mem = bytearray(rng.randrange(256) for _ in range(0x40))
        mem[5] = who & 0xFF
        cmem = bytearray(rng.randrange(256) for _ in range(0x1000))
        cmem[0xCAC:0xCB0] = struct.pack("<i", radius)
        cmem[0xCB0:0xCB4] = struct.pack("<i", per)
        cmem[0x45C:0x460] = struct.pack("<i", terra)
        uc.mem_write(this, bytes(mem))
        uc.mem_write(consts, bytes(cmem))
        uc.mem_write(CONSTANTS, struct.pack("<I", consts))
        uc.mem_write(LEADERS, struct.pack("<I", leaders))
        before = bytes(uc.mem_read(this, 0x40)) + bytes(uc.mem_read(consts, 0x1000))
        h.calls.clear()
        r = h.call(ENTRY, (), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(this, 0x40)) + bytes(uc.mem_read(consts, 0x1000)) == before, "wrote memory"
        calls = ";".join(",".join([name, str(ecx & 0xFFFFFFFF), *map(str, args)]) for name, ecx, args, _ in h.calls)
        print(f"{radius} {per} {terra} {who} {leaders} {upgrades} {wonder} -> {r} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
