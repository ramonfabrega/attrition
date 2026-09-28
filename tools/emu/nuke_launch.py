#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""nuke_launch.py — `Build::do_missile_launch@00622670`'s nuke arm, emulated (item 1091).

    uv run tools/emu/nuke_launch.py <install>/riseofnations.exe

A silo carrying the executable's own `Build` vtable (`0xb42174`), so the
head's `is(MISSILESILO)` asks the type's vslot `0x60` and vslot `0x164` is
the game's own `Wall::update_local_seen@0063ed50`, hooked by address. One
missile in `launching` (`+0x44`: count at `+0x4`, data at `+0x10`),
`recharging` (`+0x7a`) 1, the console's player the silo's. The missile's
vslot `0xb8` is `ObjectData::is@00653790`'s address, so the arm asks its
type's vslot `0x60` for `0x13b`, as the game does (`62272c`..`62274c`).
Each row prints whether `+0x40` (`visible`) was written `0xff`, whether
vslot `0x164` ran, the sound, and whether the missile came out.
`docs/PRODUCTION.md`, "The nuke (item 1091)", has the table.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hooks import Harness  # noqa: E402

LAUNCH = 0x622670
BUILD_VTABLE = 0xB42174
OBJECTDATA_IS = 0x653790
UNITS = 0xC0AEC0
CONSOLE = 0xC06210
AIR_ATTACK_GROUND, STRAFE = 0x18, 0x10
CALLEES = {
    0x462E70: ("SimpleArray<int>::remove", 1),
    0x616E80: ("UnitData::order_type", 0),
    0x6EDB50: ("LeaderData::is_ally", 1),
    0x63ED50: ("Wall::update_local_seen", 0),
    0x7E9FB0: ("MessageWin::add_message", 10),
    0x8107D0: ("IFaceMainBase::do_notice", 5),
    0x97F770: ("SoundGlobal::play", 1),
    0xA1EEB0: ("String::operator=", 1),
    0x617C10: ("Unit::come_out", 1),
}


def scenario(exe, label, *, nuke, order, recharging=1):
    h = Harness(exe, CALLEES)
    h.answers["UnitData::order_type"] = order
    stype = h.alloc(0x300)
    h.w32(stype, h.vtable({0x60: ("SiloType.is", 2, lambda ecx, a: int(a[0] == 0x208))}))
    silo = h.alloc(0x100)
    h.w32(silo, BUILD_VTABLE)
    h.w8(silo + 0x9, 0)
    h.w32(silo + 0x18, stype)
    data = h.alloc(0x10)
    h.w32(data, 7)  # the missile's object number
    arr = h.alloc(0x20)
    h.w32(arr + 0x4, 1)
    h.w32(arr + 0x10, data)
    h.w32(silo + 0x44, arr)
    h.w16(silo + 0x7A, recharging)
    mtype = h.alloc(0x300)
    h.w32(mtype, h.vtable({0x60: ("MissileType.is", 2, lambda ecx, a: int(nuke and a[0] == 0x13B))}))
    uvt = h.vtable({0x9C: ("Unit.process", 0, 0)})
    h.w32(uvt + 0xB8, OBJECTDATA_IS)
    unit = h.alloc(0x100)
    h.w32(unit, uvt)
    h.w32(unit + 0x18, mtype)
    table = h.alloc(0x40)
    h.w32(table + 7 * 4, unit)
    h.w32(UNITS, table)
    console = h.alloc(0x400)
    h.w32(console + 0x298, 0)
    h.w32(CONSOLE, console)
    h.call(LAUNCH, (), ecx=silo)
    names = [c[0] for c in h.calls]
    sound = [c[2][0] for c in h.calls if c[0] == "SoundGlobal::play"]
    print(f"{label}: +0x40 {h.m.uc.mem_read(silo + 0x40, 1)[0]:#04x}; "
          f"vslot 0x164 {'ran' if 'Wall::update_local_seen' in names else 'no'}; "
          f"sound {sound}; come_out {'yes' if 'Unit::come_out' in names else 'no'}; "
          f"recharging {h.r32(silo + 0x7A) & 0xFFFF}")


def main(exe):
    scenario(exe, "a nuke on AIR_ATTACK_GROUND", nuke=True, order=AIR_ATTACK_GROUND)
    scenario(exe, "a V2 on AIR_ATTACK_GROUND", nuke=False, order=AIR_ATTACK_GROUND)
    scenario(exe, "a nuke with no order", nuke=True, order=0)
    scenario(exe, "a nuke, recharging 2", nuke=True, order=AIR_ATTACK_GROUND, recharging=2)


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(__doc__)
        sys.exit(2)
    main(sys.argv[1])
