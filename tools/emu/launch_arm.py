#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""launch_arm.py — `Group::action_launch_flight@006fbfb0`'s missile arm, emulated (item 1078).

    uv run tools/emu/launch_arm.py <install>/riseofnations.exe

A group of one Missile Silo `0/2010` with one missile inside, `@launchstrike`
(`ATTACK`) on `1/2006`. The silo's `count_inside` of `0x13b` (a nuke) and of
`0x139` (a V2) narrows the choice to missiles (`local_34`, `local_30`), which
skip the ctrl/alt and tank gates (`6fc3cb`); then `valid_target` for a V2,
the reach `get_speed · mana`, and the missile block `6fc681`..`6fc6ef`: a
missile whose order list's current order answers vslot `0x10` (its type)
non-zero is **passed over**. So a second strike pressed while the first
counts down in `launching` finds nothing, and `action_flight` is never
called. `docs/GOLDEN.md` §45 and `docs/PRODUCTION.md`, "The missile's
other arms", have the table.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hooks import Harness  # noqa: E402

LAUNCH_FLIGHT = 0x6FBFB0
XOR = 0x63637
ATTACK = 10
AIR_ATTACK_GROUND = 0x18
CALLEES = {
    0x5946D0: ("Game::current_age", 0),
    0x609A50: ("UnitData::mana", 0),
    0x60A370: ("UnitData::is_busy", 0),
    0x616E80: ("UnitData::order_type", 0),
    0x646D50: ("ObjectData::num_inside", 1),
    0x648BA0: ("Object::valid_target", 3),
    0x64DDA0: ("ObjectData::count_inside", 2),
    0x6FB260: ("Group::action_flight", 6),
    0x711720: ("GroupData::count", 3),
    0x713E80: ("Group::clear", 1),
    0x714350: ("Group::add", 4),
}


def scenario(exe, label, *, nuke=False, order=None, busy=0, valid=1, reach=250, shift=0):
    h = Harness(exe, CALLEES)
    who, silo_o, uo, tgt_who, tgt_o = 0, 2010, 13, 1, 2006
    g = h.alloc(0xA40)
    h.w32(g, h.vtable({0x10: ("Group.vslot10", 0, 0)}))
    h.w32(g + 0xC, 1)
    h.w16(g + 0x8CC, silo_o)
    h.w8(g + 0x49, 1)
    h.w8(g + 0x4A, who)
    b = h.alloc(0x200)
    h.w32(b, h.vtable({0x10: ("Silo.vslot10", 0, 1), 0xAC: ("Silo.vslotac", 0, b)}))
    h.w32(b + 0x10, 24960 ^ XOR)
    h.w32(b + 0x14, 19200 ^ XOR)
    h.w16(b + 0x28, uo)
    h.w8(b + 0x3E, who)
    ut = h.alloc(0x400)
    h.w32(ut + 0x218, 2)  # the air domain
    h.w32(ut + 0x1E4, 0x800_0000)  # a missile (`6fc3bb`)
    u = h.alloc(0x200)
    h.w32(u, h.vtable({
        0xB8: ("Unit.is", 2, lambda ecx, a: int(nuke and a[0] == 0x13B)),
        0x17C: ("Unit.get_speed", 3, 115),
    }))
    h.w32(u + 0x10, 24960 ^ XOR)
    h.w32(u + 0x14, 19200 ^ XOR)
    h.w32(u + 0x18, ut)
    h.w16(u + 0x28, -1)
    h.w8(u + 0x3E, 0xFF)
    if order is not None:
        o = h.alloc(0x40)
        h.w32(o, h.vtable({0x10: ("Order.get_type", 0, order)}))
        inner = h.alloc(0x20)
        h.w32(inner + 8, o)
        node = h.alloc(0x20)
        h.w32(node + 4, inner)
        h.w32(u + 0xDC, node)
    t = h.alloc(0x200)
    h.w32(t + 0x10, 34944 ^ XOR)
    h.w32(t + 0x14, 20352 ^ XOR)
    objs = h.alloc(0x400)
    oa0, oa1 = h.alloc(4 * 3000), h.alloc(4 * 3000)
    h.w32(oa0 + 4 * silo_o, b)
    h.w32(oa0 + 4 * uo, u)
    h.w32(oa1 + 4 * tgt_o, t)
    h.w32(objs + 0x14 + who * 0x1C, oa0)
    h.w32(objs + 0x14 + tgt_who * 0x1C, oa1)
    h.w32(0xC0618C, objs)
    h.w32(0xC0AEC0 + who * 0x1C, oa0)
    h.w32(0xC061EC, h.alloc(0x2000))  # the game: rush rules 0
    con = h.alloc(0x400)
    h.w32(con + 0x298, 7)  # the console is not player 0: no feedback
    h.w32(0xC06210, con)
    h.answers.update({
        "GroupData::count": 0,
        "ObjectData::count_inside": lambda ecx, a: int((a[1] == 0x13B) == nuke),
        "UnitData::is_busy": busy,
        "Object::valid_target": valid,
        "UnitData::mana": reach,
    })
    h.call(LAUNCH_FLIGHT, (tgt_o, tgt_who, ATTACK, shift, 0, 0), ecx=g)
    names = [c[0] for c in h.calls]
    flight = "action_flight" if "Group::action_flight" in names else "nothing"
    added = [c[2] for c in h.calls if c[0] == "Group::add"]
    print(f"{label}: {flight}; added {added}")
    if os.environ.get("EMU_CALLS"):
        for c in h.calls:
            print("   ", c[0], [hex(x) if x > 0xffff else x for x in c[2]], c[3])


def main(exe):
    scenario(exe, "a V2 with no order")
    scenario(exe, "a V2 on its AIR_ATTACK_GROUND (the first press counting down)", order=AIR_ATTACK_GROUND)
    scenario(exe, "  ... with shift", order=AIR_ATTACK_GROUND, shift=1)
    scenario(exe, "a V2, the target not valid", valid=0)
    scenario(exe, "a V2 out of reach", reach=1)
    scenario(exe, "a nuke with no order", nuke=True)
    scenario(exe, "a nuke on its AIR_ATTACK_GROUND", nuke=True, order=AIR_ATTACK_GROUND)


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(__doc__)
        sys.exit(2)
    main(sys.argv[1])
