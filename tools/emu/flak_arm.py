#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""flak_arm.py — a round at an aircraft, and whether the aircraft flies low, emulated (item 1102).

    uv run tools/emu/flak_arm.py <install>/riseofnations.exe

Two pieces of the executable, each run on objects laid out by hand:

1. **`UnitData::is_flying_low@0060a140`**, whole. A fixed-wing unit (type
   `+0x218` 2, `+0x2b4 & 0x20` clear, not `+0x1e4 & 0x8000000`) on the
   map (the executable's own `Unit` vtable, `0xb417d0`, whose vslot `0xbc`
   is `UnitData::is_on_map@0046ce30`; `inside_up` bit 15) with an order
   (`UnitData::order_type@00616e80` and `get_order@0060b030` answered).
   `vector_dist@0046cff0` runs as the game's. The listing's three arms: a
   `STRAFE` (0x10) at its target's point (vslot `0x100`: `+8` o, `+0xc`
   who), an `AIR_ATTACK_GROUND` (0x18) at its point (vslot `0xd4`: `+4`,
   `+8`), and any order's `get_air_order` (vslot `0xfc`) with `returning`
   (`+0x18`) set, at its home `(oxx +4, whose +8)`; each `< 0x900`.

2. **`Ammo::init@0067bbf0`'s air arm**, `67bef8`..`67c16a`, entered at its
   head with `edi` the round and `esi` 100 (`67be77`; the arm's two entries
   are `67be7e` and `67bec4`, both below that store). The target a live
   fixed-wing unit (vslot `0x18` answering 1); the shooter's vslot `0x148`
   (`has_objmask(0x80000000)`) answered; `is_flying_low` answered;
   `Random::get` answered from a list. Each row prints the draw sites (the
   return address, as the trace names them) and whether `+4 |= 0x10` (the
   round's miss) was written.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import SENTINEL  # noqa: E402
from hooks import Harness  # noqa: E402
from unicorn import UC_HOOK_CODE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_EDI, UC_X86_REG_EIP, UC_X86_REG_ESI  # noqa: E402

IS_FLYING_LOW = 0x60A140
UNIT_VTABLE = 0xB417D0
OBJECTS_STATIC = 0xC0AB84  # is_flying_low's `objects` list, + who * 0x1c
OBJECTS_PTR = 0xC0618C  # GameAccess::objects; its list at +0x14 + who * 0x1c
UNITS = 0xC0AEC0  # units' list, + who * 0x1c
GAME_RANDOM = 0xC06184
KEY = 0x63637
ARM, JOIN = 0x67BEF8, 0x67C16A
INIT = 0x67BBF0
DRAWS = (0x67C01D, 0x67C04E, 0x67C08A, 0x67C0C7, 0x67C0FA, 0x67C133)


def fixed_wing(h, **fields):
    t = h.alloc(0x300)
    h.w32(t + 0x218, fields.get("domain", 2))
    h.w32(t + 0x250, fields.get("fly_high", 0))
    h.w32(t + 0x254, fields.get("fly_low", 10))
    return t


def is_flying_low(exe, label, *, order, to, returning=1, who=1):
    """A Bomber at (9984, 12288) under `order`, its target or home `to` off."""
    h = Harness(exe, {0x616E80: ("UnitData::order_type", 0), 0x60B030: ("UnitData::get_order", 0)})
    x, y = 9984, 12288
    u = h.alloc(0x200)
    h.w32(u, UNIT_VTABLE)
    h.w32(u + 0x10, x ^ KEY)
    h.w32(u + 0x14, y ^ KEY)
    h.w32(u + 0x18, fixed_wing(h))
    h.w16(u + 0x82, -0x8000)
    tx, ty = x + to[0], y + to[1]
    tgt = h.alloc(0x40)
    h.w8(tgt + 0x8, 1)
    h.w32(tgt + 0x10, tx ^ KEY)
    h.w32(tgt + 0x14, ty ^ KEY)
    arr = h.alloc(0x40)
    h.w32(arr + 4 * 5, tgt)
    h.w32(OBJECTS_STATIC + who * 0x1C, arr)
    target = h.alloc(0x20)  # TargetOrder: +8 ox, +0xc whom
    h.w32(target + 0x8, 5)
    h.w32(target + 0xC, who)
    point = h.alloc(0x20)  # the ground order's point: +4, +8
    h.w32(point + 0x4, tx)
    h.w32(point + 0x8, ty)
    air = h.alloc(0x28)  # AirOrder: +4 oxx, +8 whose, +0x18 returning
    h.w32(air + 0x4, 5)
    h.w32(air + 0x8, who)
    h.w32(air + 0x18, returning)
    o = h.alloc(0x20)
    h.w32(o, h.vtable({0x100: ("Order.target", 0, target), 0xD4: ("Order.point", 0, point),
                       0xFC: ("Order.get_air_order", 0, air)}))
    h.answers["UnitData::order_type"] = order
    h.answers["UnitData::get_order"] = o
    low = h.call(IS_FLYING_LOW, (), ecx=u)
    print(f"  {label}: {low}")
    return low


def air_arm(exe, label, *, aa, shooter_domain, s_high, s_low, low, rolls):
    h = Harness(exe, {IS_FLYING_LOW: ("UnitData::is_flying_low", 0), 0xA39D70: ("Random::get", 2)})
    h.m.uc.hook_add(UC_HOOK_CODE, lambda uc, a, s, _: uc.reg_write(UC_X86_REG_EIP, SENTINEL),
                    begin=JOIN, end=JOIN)
    for d in DRAWS:
        h.mark(d)
    objs = h.alloc(0x200)
    h.w32(OBJECTS_PTR, objs)
    shooter = h.alloc(0x100)
    h.w32(shooter, h.vtable({0x148: ("Shooter.has_objmask", 1, lambda ecx, a: int(aa))}))
    h.w32(shooter + 0x18, fixed_wing(h, domain=shooter_domain, fly_high=s_high, fly_low=s_low))
    target = h.alloc(0x100)
    h.w32(target, h.vtable({0x18: ("Target.vslot18", 0, 1)}))
    h.w8(target + 0x8, 1)
    h.w32(target + 0x18, fixed_wing(h))  # the Bomber: FLY_HIGH 0, FLY_LOW 10
    for who, obj in ((0, shooter), (1, target)):
        arr = h.alloc(0x40)
        h.w32(arr + 4 * 3, obj)
        h.w32(objs + 0x14 + who * 0x1C, arr)
        h.w32(UNITS + who * 0x1C, arr)
    ammo = h.alloc(0x80)
    h.w32(ammo + 0x3C, 0)
    h.w32(ammo + 0x40, 3)
    h.w32(ammo + 0x48, 1)
    h.w32(ammo + 0x4C, 3)
    h.answers["UnitData::is_flying_low"] = int(low)
    queue = list(rolls)
    h.answers["Random::get"] = lambda ecx, a: queue.pop(0)
    h.m.uc.reg_write(UC_X86_REG_EDI, ammo)
    h.m.uc.reg_write(UC_X86_REG_ESI, 100)
    h.call(ARM, ())
    sites = [f"Ammo::init+{d + 5 - INIT:#x}" for d in DRAWS if d in h.marks]
    miss = h.m.uc.mem_read(ammo + 4, 1)[0] & 0x10 != 0
    asked = any(c[0] == "UnitData::is_flying_low" for c in h.calls)
    print(f"  {label}: low asked {asked}; draws {sites}; {'miss' if miss else 'hit'}")
    return sites, miss


def main(exe):
    print("UnitData::is_flying_low, a Bomber at (9984, 12288):")
    for order, name in ((0x10, "STRAFE"), (0x18, "AIR_ATTACK_GROUND")):
        for dx, dy in ((0x8FF, 0), (0x900, 0), (1600, 1600), (1700, 1700)):
            is_flying_low(exe, f"{name}, its target ({dx:+}, {dy:+}) off", order=order, to=(dx, dy))
    for ret in (1, 0):
        for dx in (0x8FF, 0x900):
            is_flying_low(exe, f"AIR_PATROL (0x11), returning {ret}, home {dx:+} off",
                          order=0x11, to=(dx, 0), returning=ret)
    is_flying_low(exe, "no order", order=0, to=(0, 0))

    print("Ammo::init's air arm, at a Bomber (FLY_HIGH 0, FLY_LOW 10):")
    battery = dict(aa=True, shooter_domain=0, s_high=50, s_low=90)
    for low in (True, False):
        for r in (49, 50, 89, 90):
            air_arm(exe, f"an Anti-Aircraft Battery (50/90), low {low}, roll {r}", low=low, rolls=[r], **battery)
    air_arm(exe, "a Fighter (ANTI_AIR, domain 2)", aa=True, shooter_domain=2, s_high=0, s_low=25,
            low=False, rolls=[])
    infantry = dict(aa=False, shooter_domain=0, s_high=0, s_low=33)
    for rolls in ([9, 32], [9, 33], [10]):
        air_arm(exe, f"an Infantry (0/33), low, rolls {rolls}", low=True, rolls=rolls, **infantry)
    air_arm(exe, "an Infantry (0/33), high, roll 0", low=False, rolls=[0], **infantry)


if __name__ == "__main__":
    main(sys.argv[1])
