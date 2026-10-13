#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_data_health_level.py — `ObjectData::health_level@006534b0` under unicorn (item 2061, hand-written
on tools/emu/hooks.py).

    uv run tools/emu/sweep_object_data_health_level.py <install>/riseofnations.exe

`__thiscall`, no argument, `ret`. `this`'s slot `+0x11c` (hooked, one argument, 0) answers the hit points and
its slot `+0x18` (hooked, none) whether it is a unit's figure; a figure's damage is its unit's total (the direct
callee at `0x609c00`, hooked, two arguments, both 0), the unit found in the per-player table at `0xc0aec0 +
who·0x1c` (`who` the byte `+9`) by the short `+0xa`; anything else's is `this+0x24`.
Row: `<hits> <damage> <figure> -> <eax> | <calls>`, the calls `<name>:<ecx and arguments as expected>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x006534B0
TOTAL = 0x00609C00
UNITS = 0x00C0AEC0


def draw_hits(rng):
    return rng.choice((0, 1, 2, 9, 10, 11, 99, 100, 101, 1000)) if rng.random() < 0.3 else rng.randrange(20000)


def draw_damage(rng, hits):
    r = rng.random()
    if r < 0.5 and hits:
        # Every threshold's edge: the left hits a percentage exactly, or one either side.
        pct = rng.choice((90, 75, 50, 25, 10, 0, 100))
        return hits - hits * pct // 100 + rng.choice((-1, 0, 1))
    if r < 0.6:
        return rng.randint(-200, 0)
    return rng.randrange(25000)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {TOTAL: ("total", 2)})
    uc = h.m.uc
    rng = random.Random(2061)
    this = h.alloc(0x40)
    unit = h.alloc(0x40)
    table = h.alloc(0x40 * 4)
    st = {"hits": 0, "fig": 0}
    vt = h.vtable({0x11C: ("hits", 1, lambda e, a: st["hits"]), 0x18: ("figure", 0, lambda e, a: st["fig"])})
    for _ in range(400):
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x40)))
        h.w32(this, vt)
        who = rng.randrange(8)
        slot = rng.randrange(0x40)
        h.w8(this + 9, who)
        h.w16(this + 0xA, slot)
        for i in range(0x40):
            h.w32(table + 4 * i, rng.randrange(1, 0x1000))
        h.w32(table + 4 * slot, unit)
        h.w32(UNITS + who * 0x1C, table)
        hits = draw_hits(rng)
        dmg = draw_damage(rng, hits)
        st["hits"] = hits
        st["fig"] = rng.choice((0, 1, 2, -1)) if rng.random() < 0.5 else 0
        if st["fig"]:
            h.answers["total"] = dmg
            h.w32(this + 0x24, rng.randint(-0x80000000, 0x7FFFFFFF))
        else:
            h.answers["total"] = rng.randint(-0x80000000, 0x7FFFFFFF)
            h.w32(this + 0x24, dmg)
        before = bytes(uc.mem_read(this, 0x40))
        h.calls.clear()
        res = h.call(ENTRY, (), ecx=this)
        assert uc.reg_read(UC_X86_REG_ESP) == STACK_BASE + STACK_SIZE - 0x100, "wrong pop"
        assert bytes(uc.mem_read(this, 0x40)) == before, "wrote"
        want = {"hits": (this, [0]), "figure": (this, []), "total": (unit, [0, 0])}
        calls = ",".join(f"{name}:{int((ecx, args) == want[name])}" for name, ecx, args, _ in h.calls)
        print(f"{hits} {dmg} {int(st['fig'] != 0)} -> {res} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
