#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""redraw_arm.py — `Unit::do_air_physics@005e86d0`'s altitude redraw, emulated (item 1078).

    uv run tools/emu/redraw_arm.py <install>/riseofnations.exe

The head of the step (`5e86d0`..`5e87cb`): on a frame `(o + frame) & 7 == 0`
a unit that is not `is(0x130)` (the Bomber line), whose vslot `0x30` answers
0 or whose type is `0x193`, and whose `unit_flags & 0x20` is clear draws
`Random::get(0, 0xffff)` and sets the order's `cruising_alt` to `(r % 7 +
13) · 100`; any other takes `0x640`, ±200 when vslot `0x30` answers.

The unit carries the executable's own `Unit` vtable (`0xb417d0`), so vslot
`0x30` runs the code the game runs — `Window::get_button@0041bff0`, a folded
`return 0` — and vslot `0xb8` is `ObjectData::is@00653790`, which asks the
type's vslot `0x60`. The type, the order and the game are laid out by hand.
Execution stops at `5e87cb`, the `field_0xc0 = 0` store the redraw joins.
`docs/GOLDEN.md` §45 and `docs/PRODUCTION.md`, "The missile's other arms",
have the table.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import SENTINEL  # noqa: E402
from hooks import Harness  # noqa: E402
from unicorn import UC_HOOK_CODE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_EIP  # noqa: E402

AIR_PHYSICS = 0x5E86D0
JOIN = 0x5E87CB
UNIT_VTABLE = 0xB417D0
GAME = 0xC061EC
GAME_RANDOM = 0xC06184
CALLEES = {0xA39D70: ("Random::get", 2)}


def scenario(exe, label, *, ty, flags, bomber, o, frame, roll=40000):
    h = Harness(exe, CALLEES)
    h.m.uc.hook_add(UC_HOOK_CODE, lambda uc, a, s, _: uc.reg_write(UC_X86_REG_EIP, SENTINEL),
                    begin=JOIN, end=JOIN)
    h.mark(0x5E8778)  # the draw block
    t = h.alloc(0x400)
    h.w32(t, h.vtable({0x60: ("Type.is", 2, lambda ecx, a: int(bomber and a[0] == 0x130))}))
    h.w32(t + 0x4, ty)
    h.w32(t + 0x2B4, flags)
    u = h.alloc(0x200)
    h.w32(u, UNIT_VTABLE)
    h.w16(u + 0xA, o)
    h.w32(u + 0x18, t)
    air = h.alloc(0x40)
    h.w32(air + 0xC, 0x640)
    order = h.alloc(0x40)
    h.w32(order, h.vtable({0x84: ("Order.vslot84", 0, air)}))
    game = h.alloc(0x800)
    h.w32(game + 0x550, frame)
    h.w32(GAME, game)
    h.w32(GAME_RANDOM, h.alloc(0x40))
    h.answers["Random::get"] = roll
    h.call(AIR_PHYSICS, (order, 9984, 12288), ecx=u)
    draws = [c for c in h.calls if c[0] == "Random::get"]
    print(f"{label}: (o + frame) & 7 = {(o + frame) & 7}; "
          f"draw {'yes' if 0x5E8778 in h.marks else 'no'} {[(c[2]) for c in draws]}; "
          f"cruising_alt {h.r32(air + 0xC)}")


def main(exe):
    # o = 13 and frame 3443 is a multiple of 8; 3444 is not.
    for f in (3443, 3444):
        scenario(exe, "V2 Rocket (0x139, FLAGS h)", ty=0x139, flags=0x80, bomber=False, o=13, frame=f)
        scenario(exe, "Nuclear Missile (0x13b)", ty=0x13B, flags=0, bomber=False, o=13, frame=f)
        scenario(exe, "a Bomber (is 0x130)", ty=0x130, flags=0, bomber=True, o=13, frame=f)
        scenario(exe, "a Helicopter (unit_flags & 0x20)", ty=0x136, flags=0x20, bomber=False, o=13, frame=f)
        scenario(exe, "type 0x192", ty=0x192, flags=0, bomber=False, o=13, frame=f)


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(__doc__)
        sys.exit(2)
    main(sys.argv[1])
