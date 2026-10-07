#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_city_get_empty_trade_routes.py — `CityData::get_empty_trade_routes@007395c0`
under unicorn.

    uv run tools/emu/sweep_city_get_empty_trade_routes.py <install>/riseofnations.exe

`__thiscall`, two stack arguments, `ret 8`. One row per input, variable length:

    <a> <b> <t> <n> { <list> <slot> <flags> <p0> <p1> <p2> <p3> } x n -> <result>

    a b        the two arguments (in the game a city number and its owner)
    t          which city's `vans` is the `this`: the script lays out two
               `CityData` records (city 0 and city 1) and calls on city `t`
    n          `this->vans` count (`+0x78`); the list pointer is at `+0x84`
    list slot  one `CaravanLink` (8 bytes: `cara` = slot at +0, `who` = list
               index at +4): `caravans.lists[list].list[slot]` is the van
    flags      the van's `caravan_flags` (`+0xc`)
    p0..p3     the van's four shorts `city2 whom city3 whose` (`+0..+6`)
    result     `1 - #{vans with flags & 2 and (p0,p1)==(a,b) or (p2,p3)==(a,b)}`

The `caravans` global (`0xe3a290`; each `PtrArray` is 0x1c bytes, its `list`
pointer at +0x10) is pointed at hand-mapped pages of `Caravan*` and of
records. Nothing is stood in for: the function calls nothing. The generator
keeps the shape a game gives (the `this` city is an endpoint of each of its
vans, no van listed twice, owner coordinate 0) on most rows, and breaks it on
others — a wrong owner coordinate, a flag word with bit 2 and not bit 0,
endpoints that touch neither city, arguments off the shape — so the Rust
side can say which rows it compares. Seed 1619.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x007395C0
CARAVANS = 0xE3A290  # Caravans.lists[8], stride 0x1c, `list` at +0x10
CITY = [0x10000000, 0x10001000]
VANS = [0x10002000, 0x10003000]  # CaravanLink arrays
PTRS = [0x10010000, 0x10011000]  # Caravan*[] one page per list index (0, 1)
RECS = 0x10020000  # one 0x50-byte record per (list, slot)


def w32(m, a, v):
    m.uc.mem_write(a, struct.pack("<i", v))


def rec_addr(lst, slot):
    return RECS + (lst * 32 + slot) * 0x50


def setup(m):
    for a in CITY + VANS + PTRS:
        m.uc.mem_map(a, 0x1000)
    m.uc.mem_map(RECS, 0x10000)
    for w in range(8):  # lists 2..7 share a page; only 0 and 1 are used
        w32(m, CARAVANS + 0x1C * w + 0x10, PTRS[min(w, 1)])


def gen():
    rng = random.Random(1619)
    rows = []

    def van(t, kind=None):
        lst = rng.randint(0, 1)
        slot = rng.randint(0, 5)
        flags = rng.choice((1, 3, 5, 7, 3, 7, 1, 0, 2, 4, 6))
        if kind is None:
            kind = rng.choice(("here", "here", "here", "loop", "foreign", "wild"))
        other = 1 - t
        if kind == "loop":  # both ends this city
            e = [(t, 0), (t, 0)]
        elif kind == "here":
            e = [(t, 0), (rng.choice((t, other)), 0)]
            rng.shuffle(e)
        elif kind == "foreign":  # right cities, a wrong owner coordinate
            e = [(t, 0), (rng.choice((t, other)), rng.choice((1, 2, -1, 255)))]
            if rng.random() < 0.5:
                e[0] = (e[0][0], rng.choice((1, 3, -2)))
            rng.shuffle(e)
        else:  # anything: other cities, the edges of a short
            e = [
                (rng.choice((-1, 0, 1, 2, 5, 32767, -32768)), rng.choice((0, 0, 1, -1, 7, 32767)))
                for _ in range(2)
            ]
        return [lst, slot, flags, e[0][0], e[0][1], e[1][0], e[1][1]]

    def row(t, a, b, vans):
        seen, out = set(), []
        for v in vans:
            if (v[0], v[1]) in seen:
                continue
            seen.add((v[0], v[1]))
            out.append(v)
        rows.append([a, b, t, len(out)] + [x for v in out for x in v])

    for t in (0, 1):
        row(t, 1 - t, 0, [])
        for flags in range(8):  # every flag word against each endpoint shape
            for nth in range(3):
                e = [(t, 0), (1 - t, 0)]
                if nth == 1:
                    e.reverse()
                if nth == 2:
                    e = [(t, 0), (t, 0)]
                row(t, 1 - t, 0, [[0, 0, flags, e[0][0], e[0][1], e[1][0], e[1][1]]])
        # two routes to the same city, in different lists and slots
        row(t, 1 - t, 0, [[0, 1, 3, t, 0, 1 - t, 0], [1, 1, 3, 1 - t, 0, t, 0]])
        # a match needs both shorts of one end, never one of each
        row(t, 1 - t, 0, [[0, 0, 3, 1 - t, 7, t, 0], [0, 1, 3, 1 - t, 0, t, 0]])
        row(t, 1 - t, 0, [[0, 0, 3, 5, 0, 1 - t, 0]])
        row(t, 1 - t, 0, [[0, 0, 3, t, 0, 1 - t, 9]])
        row(t, 1 - t, 0, [[0, 0, 3, 1 - t, 9, 1 - t, 0]])
        # the arguments against one van: a changes, b changes, both
        for a, b in ((1 - t, 0), (1 - t, 1), (t, 0), (1 - t, -1), (65537 + (1 - t), 0), (1 - t, 65536)):
            row(t, a, b, [[0, 0, 3, t, 0, 1 - t, 0]])
    while len(rows) < 700:
        t = rng.randint(0, 1)
        row(t, 1 - t, 0, [van(t) for _ in range(rng.choice((0, 1, 1, 2, 2, 3, 4, 6)))])
    for _ in range(60):  # off the game's shape: arguments and vans from the short range
        t = rng.randint(0, 1)
        a = rng.choice((0, 1, -1, 2, 32767, -32768, 32768, 65536, 70000, -70000, rng.randint(-5, 5)))
        b = rng.choice((0, 1, -1, 7, 32767, -32768, 65536, rng.randint(-5, 5)))
        vs = []
        for _ in range(rng.randint(1, 4)):
            v = van(t, "wild")
            if rng.random() < 0.6:
                # a short holding `a` mod 2^16: an argument outside a short
                # can never equal what the record's `movswl` sign-extends
                v[rng.choice((3, 5))] = (a + 32768) % 65536 - 32768
                v[rng.choice((4, 6))] = (b + 32768) % 65536 - 32768
            vs.append(v)
        row(t, a, b, vs)
    return rows


def main(argv):
    m = Machine(Image(argv[1]))
    setup(m)
    for r in gen():
        a, b, t, n = r[:4]
        vans = [r[4 + 7 * i : 11 + 7 * i] for i in range(n)]
        m.uc.mem_write(VANS[t], bytes(0x1000))
        m.uc.mem_write(PTRS[0], bytes(0x1000))
        m.uc.mem_write(PTRS[1], bytes(0x1000))
        m.uc.mem_write(RECS, bytes(0x10000))
        w32(m, CITY[t] + 0x78, n)
        w32(m, CITY[t] + 0x84, VANS[t])
        for i, (lst, slot, flags, p0, p1, p2, p3) in enumerate(vans):
            w32(m, VANS[t] + 8 * i, slot)
            w32(m, VANS[t] + 8 * i + 4, lst)
            rec = rec_addr(lst, slot)
            w32(m, PTRS[min(lst, 1)] + 4 * slot, rec)
            m.uc.mem_write(rec, struct.pack("<hhhh", p0, p1, p2, p3))
            m.uc.mem_write(rec + 0xC, bytes([flags]))
        print(" ".join(map(str, r)), "->", signed(m.call(ENTRY, (a, b), ecx=CITY[t])))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
