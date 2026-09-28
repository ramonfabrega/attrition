#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""train_arm.py — `Build::train@0062f9b0`'s CARRY_AIR arm, emulated (item 1019).

    uv run tools/emu/train_arm.py <install>/riseofnations.exe

The trainer is an Airbase `0/2007` (`type +0x1e4 & 0x200`); the trained unit
`0/10` is a Helicopter (`+0x2b4 & 0x20`), a missile (`+0x1e4 & 0x8000000`)
or, for contrast, a Fighter. Each scenario lays out the gather list at
`+0xb8` (`+0xc8` its count, `+0xcc` its head, a node's `+8` the
`GatherPoint`, whose `+4`/`+8` are the point) and answers every callee;
it prints the calls the arm made and whether it reached `62fc17`, the
`unit_masks &= ~0x4000000` every arm but one ends on. `docs/PRODUCTION.md`,
"The Helicopter and the missile under a point", has the table.
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hooks import Harness  # noqa: E402

TRAIN = 0x62F9B0
XOR = 0x63637

# `{address: (name, nargs)}`, the argument count off each callee's `ret`.
CALLEES = {
    0x43F360: ("WorldData::is_valid", 2),
    0x46F0A0: ("BuildData::num_gather", 0),
    0x5E4350: ("Unit::add_air_patrol_order", 6),
    0x5E48C0: ("Unit::add_strafe_order", 7),
    0x605310: ("Unit::set_stance", 2),
    0x6179D0: ("Unit::update_order", 0),
    0x617C10: ("Unit::come_out", 1),
    0x61A2E0: ("Unit::go_inside", 3),
    0x645330: ("ObjectData::num_aircraft_here", 1),
    0x6454A0: ("ObjectData::num_aircraft_limit", 0),
    0x6483C0: ("ObjectData::can_carry", 2),
    0x65AB40: ("ObjectsData::find_building_at", 7),
    0x65E0C0: ("Objects::init_unit", 7),
    0x6DB810: ("LeaderData::has_preq", 1),
    0x6EBAA0: ("LeaderData::is_enemy", 1),
    0x7E9FB0: ("MessageWin::add_message", 10),
    0x97F770: ("SoundGlobal::play", 1),
    0x647080: ("Object::die", 3),
}
QUIET = {"Objects::init_unit", "Unit::go_inside", "Unit.vslot108", "Build.vslot108", "Unit::set_stance"}


def scenario(exe, label, *, unit, points, find=(-1, 0), enemy=0, preq=0, carry=1,
             carry_active=1, valid=1, here=1, limit=10, come_out=0):
    h = Harness(exe, CALLEES)
    h.mark(0x62FC17)
    who, base_o, uo = 0, 2007, 10
    btype = h.alloc(0x400)
    h.w32(btype + 0x1E4, 0x200)
    b = h.alloc(0x200)
    h.w32(b, h.vtable({0x108: ("Build.vslot108", 0, 1)}))
    h.w8(b + 9, who)
    h.w16(b + 0xA, base_o)
    h.w32(b + 0x10, 11616 ^ XOR)
    h.w32(b + 0x14, 13920 ^ XOR)
    h.w32(b + 0x18, btype)
    if points:
        nodes = []
        for (x, y) in points:
            gp = h.alloc(0x10)
            h.w32(gp + 4, x)
            h.w32(gp + 8, y)
            node = h.alloc(0x10)
            h.w32(node + 8, gp)
            nodes.append(node)
        for i, n in enumerate(nodes):
            h.w32(n, nodes[i + 1] if i + 1 < len(nodes) else 0)
        h.w32(b + 0xC8, len(points))
        h.w32(b + 0xCC, nodes[0])
    masks, flags, is_heli = {"helicopter": (0, 0x20, 1), "missile": (0x800_0000, 0, 0),
                             "fighter": (0, 0, 0)}[unit]
    utype = h.alloc(0x400)
    h.w32(utype + 0x1E4, masks)
    h.w32(utype + 0x2B4, flags)
    u = h.alloc(0x200)
    h.w32(u, h.vtable({
        0x108: ("Unit.vslot108", 0, 0),
        0xB8: ("Unit.is", 2, lambda ecx, a: {0x136: is_heli, 0x15F: 0}.get(a[0], 0)),
        0x158: ("Unit.die", 3, 0),
    }))
    h.w32(u + 0x18, utype)
    h.w32(u + 0x68, 0x400_0000)
    t = h.alloc(0x200)
    h.w32(t, h.vtable({0x10: ("Target.vslot10", 0, carry_active)}))
    ua = h.alloc(4 * 3000)
    h.w32(ua + 4 * uo, u)
    h.w32(0xC0AEC0 + who * 0x1C, ua)
    objs, oa = h.alloc(0x400), h.alloc(4 * 3000)
    h.w32(oa + 4 * uo, u)
    h.w32(oa + 4 * base_o, b)
    if find[0] >= 0:
        h.w32(oa + 4 * find[0], t)
    for w in range(8):
        h.w32(objs + 0x14 + w * 0x1C, oa)
    h.w32(0xC0618C, objs)
    con = h.alloc(0x400)
    h.w32(con + 0x298, 7)  # the console is not player 0: no bubble, no sound
    h.w32(0xC06210, con)
    h.w32(0xC06204, h.alloc(0x100))
    h.w32(0xC06188, h.alloc(0x100))
    d3 = h.alloc(4 * 8192)
    h.m.uc.mem_write(d3, b"".join(struct.pack("<i", i // 3) for i in range(8192)))
    h.w32(0xCAE5FC, d3)

    def found(ecx, a):
        h.w32(objs + 0x200, find[1])
        return find[0]

    h.answers.update({
        "Objects::init_unit": uo, "WorldData::is_valid": valid,
        "ObjectsData::find_building_at": found, "LeaderData::is_enemy": enemy,
        "LeaderData::has_preq": preq, "ObjectData::can_carry": carry,
        "ObjectData::num_aircraft_here": here, "ObjectData::num_aircraft_limit": limit,
        "Unit::come_out": come_out, "BuildData::num_gather": len(points),
    })
    eax = h.call(TRAIN, (0x100,), ecx=b)
    print(label)
    print(f"  -> {eax}; 62fc17 {'reached' if 0x62FC17 in h.marks else 'SKIPPED'}; "
          f"unit_masks {h.r32(u + 0x68):#x}")
    for name, ecx, args, ret in h.calls:
        if name not in QUIET:
            who_ = {u: "unit", b: "base", t: "target"}.get(ecx, f"{ecx:#x}")
            print(f"  {name}[{who_}]({', '.join(str(x) for x in args)}) = {ret}")


def main(exe):
    P = (13440, 9600)        # a ground point, tile (70, 50)
    ENEMY = (13824, 14976)   # a point on 1/2006's tile
    OTHER = (8544, 14688)    # a point on 0/2008's tile
    for unit in ("helicopter", "missile"):
        print(f"==== {unit}")
        s = lambda label, **k: scenario(exe, label, unit=unit, **k)  # noqa: E731
        s("no gather point, under the limit", points=[], here=1, limit=10)
        s("no gather point, over the limit", points=[], here=11, limit=10)
        s("no gather point, come_out refused", points=[], come_out=1)
        s("a ground point", points=[P])
        s("two ground points", points=[P, OTHER])
        s("an invalid point, over the limit", points=[P], valid=0, here=11, limit=10)
        s("an invalid point, under the limit", points=[P], valid=0)
        s("a point on its own base", points=[P], find=(2007, 0))
        s("a point on an enemy building", points=[ENEMY], find=(2006, 1), enemy=1)
        s("a point on an enemy building, MISSILE_DEFENSE_BONUS", points=[ENEMY], find=(2006, 1), enemy=1, preq=1)
        s("a point on a friendly base that carries", points=[OTHER], find=(2008, 0))
        s("a point on a friendly base, can_carry 0", points=[OTHER], find=(2008, 0), carry=0)
        s("a point on a friendly base, vslot 0x10 0", points=[OTHER], find=(2008, 0), carry_active=0)
    print("==== fighter (the num_gather loop, for contrast)")
    scenario(exe, "a ground point", unit="fighter", points=[P])
    scenario(exe, "a point on an enemy building", unit="fighter", points=[ENEMY], find=(2006, 1), enemy=1)


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(__doc__)
        sys.exit(2)
    main(sys.argv[1])
