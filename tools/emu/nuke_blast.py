#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""nuke_blast.py — `Nuke::do_damage@0092bc80`, emulated frame by frame (item 1091).

    uv run tools/emu/nuke_blast.py <install>/riseofnations.exe

One nuke is laid into `nuke_effect` (`0xc0a780`) the way `Nuke::add_nuke@0092ba30`
lays it — its point, its radius pair `(100.0, splash_area · 192.0)`, its start
frame, the shooter's player and object — with `+0x104` 10, `+0x108` 30 and
`+0x110` 110, the three `Nuke::init@0092c960` and `NukeOut::init@0092c9a0`
write. Then the executable's own `Nuke::do_damage` runs once a frame, on
`game->frame` = start + t, with its callees answered (`hooks.py`):

- `Objects::find_units@0065a620` records the radius it is handed (its fifth
  argument) and answers one unit, whose `vector_dist@0046cff0` is chosen;
- `Object::do_damage@0064a480` records its `count`, the 8.8 fraction;
- `been_damaged_before@00925c20` answers 0, `Game::war_allowed@005946d0` 0,
  `NukeOut::graph_do_damage@00925970` nothing.

The world is laid 0 × 0, so the building ring's bounds test refuses every
cell: the buildings take the same `r` and are read off the listing
(`92c1..`). The radius is SSE scalar float32 (`divss`, `mulss`, `addss`,
`cvttss2si` at `92bf3a`..`92bf8d`); this prints it, the frame a unit at each
distance is struck and with what fraction, the frame the Armageddon counter
(`game +0x6e0`) moves, and the frame the nuke is dropped.
`docs/PRODUCTION.md`, "The nuke (item 1091)", has the table.
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hooks import Harness  # noqa: E402
from unicorn import UC_HOOK_CODE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_EAX, UC_X86_REG_ESP, UC_X86_REG_EIP  # noqa: E402

DO_DAMAGE = 0x92BC80
NUKE = 0xC0A780
GAME = 0xC061EC
OBJECTS = 0xC0618C
WORLD = 0xC06188
UNITS = 0xC0AEC0  # `units.field_0x10`: a player's unit table at `+ who · 0x1c` (`92c026`)
CALLEES = {
    0x925970: ("NukeOut::graph_do_damage", 0),
    0x65A620: ("Objects::find_units", 11),
    0x925C20: ("Nuke::been_damaged_before", 3),
    0x64A480: ("Object::do_damage", 8),
    0x5946D0: ("Game::war_allowed", 0),
    0x42DAF0: ("ArrayBase<int>::add", 1),
}
CDECL = {0x46CFF0: "vector_dist", 0x92D130: "find_angle"}
START = 5000


def f32(x):
    return struct.unpack("<I", struct.pack("<f", x))[0]


def arr(h, n, words):
    """A `SimpleArray`-shaped block: `words` at a fresh pointer; returns it."""
    p = h.alloc(4 * max(n, 1) + 16)
    for i, w in enumerate(words):
        h.w32(p + 4 * i, w)
    return p


def blast(exe, dist, t_range, splash=10, nations=2, resources=0):
    h = Harness(exe, CALLEES)
    uc = h.m.uc
    seen = {"d": dist}

    def cdecl(uc, addr, size, _):
        name = CDECL[addr]
        esp = uc.reg_read(UC_X86_REG_ESP)
        ret = struct.unpack("<I", uc.mem_read(esp, 4))[0]
        h.calls.append((name, 0, [], 0))
        uc.reg_write(UC_X86_REG_EAX, seen["d"] if name == "vector_dist" else 0)
        uc.reg_write(UC_X86_REG_ESP, esp + 4)
        uc.reg_write(UC_X86_REG_EIP, ret)

    for a in CDECL:
        uc.hook_add(UC_HOOK_CODE, cdecl, begin=a, end=a)
    # nuke_effect: one nuke at (23040, 34560), radii (100, splash·192).
    h.w32(NUKE + 0x44, 1)
    h.w32(NUKE + 0x50, arr(h, 2, [f32(23040.0), f32(34560.0)]))
    h.w32(NUKE + 0x60, 1)
    h.w32(NUKE + 0x6C, arr(h, 2, [f32(100.0), f32(float(splash * 0xC0))]))
    h.w32(NUKE + 0x7C, 1)
    h.w32(NUKE + 0x88, arr(h, 1, [START]))
    h.w32(NUKE + 0x98, 1)
    h.w32(NUKE + 0xA4, arr(h, 1, [0]))  # the shooter's player
    h.w32(NUKE + 0xB4, 1)
    h.w32(NUKE + 0xC0, arr(h, 1, [10]))  # the shooter's object
    # The two damaged lists (`+0xdc`, `+0xf8`) are left null: `ArrayBase<int>::add`
    # is answered, and the drop at `+0x110 + 0x140` frees only a list that is not.
    h.w32(NUKE + 0xD0, 1)
    h.w32(NUKE + 0xDC, arr(h, 1, [0]))
    h.w32(NUKE + 0xEC, 1)
    h.w32(NUKE + 0xF8, arr(h, 1, [0]))
    h.w32(NUKE + 0x104, 10)
    h.w32(NUKE + 0x108, 30)
    h.w32(NUKE + 0x110, 110)
    game = h.alloc(0x900)
    h.w32(GAME, game)
    h.w8(game + 0x2D, resources)
    h.w32(game + 0x6A0, nations)
    h.w32(game + 0x820, h.alloc(0x10))
    world = h.alloc(0x200)
    h.w32(WORLD, world)
    # One unit of player 1, object 3, found by every search.
    unit_vt = h.vtable({
        0x08: ("Unit.vslot08", 0, 1),
        0x0C: ("Unit.vslot0c", 0, 1),
        0xB8: ("Unit.is", 2, 0),
        0xBC: ("Unit.vslotbc", 0, 1),
    })
    unit = h.alloc(0x100)
    h.w32(unit, unit_vt)
    entry = h.alloc(0x20)
    h.w8(entry + 9, 1)
    h.w16(entry + 10, 3)
    objs = h.alloc(0x400)
    h.w32(OBJECTS, objs)
    h.w32(objs + 0x214, arr(h, 1, [entry]))
    shooter = h.alloc(0x100)
    for p in range(8):
        h.w32(objs + 0x14 + p * 0x1C, arr(h, 16, [shooter] * 16))
    table = arr(h, 16, [unit] * 16)
    for p in range(8):
        h.w32(UNITS + p * 0x1C, table)
    rows = []
    for t in t_range:
        h.calls.clear()
        h.w32(game + 0x550, START + t)
        before = h.r32(game + 0x6E0)
        h.w32(objs + 0x208, 1)
        h.call(DO_DAMAGE, (), ecx=NUKE, max_insns=2_000_000)
        r = [c[2][4] for c in h.calls if c[0] == "Objects::find_units"]
        hit = [c[2][5] for c in h.calls if c[0] == "Object::do_damage"]
        arm = h.r32(game + 0x6E0) - before
        rows.append((t, r[0] if r else None, hit[0] if hit else None, arm, h.r32(NUKE + 0x7C)))
    return rows


def main(exe):
    print("t  radius  count(d=1804)  armageddon  nukes")
    for t, r, c, arm, n in blast(exe, 1804, range(0, 45)):
        print(f"{t:3} {r!s:>6} {c!s:>6} {arm:3} {n:3}")
    for t, r, c, arm, n in blast(exe, 1804, (109, 110, 111, 429, 430)):
        print(f"{t:3} {r!s:>6} {c!s:>6} {arm:3} {n:3}")
    print("d   first t struck  count")
    for d in (0, 100, 101, 518, 1039, 1804, 1859, 1860, 1870, 1920):
        rows = blast(exe, d, range(0, 41))
        first = next(((t, c) for t, _, c, _, _ in rows if c is not None), None)
        print(f"{d:5} {first}")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(__doc__)
        sys.exit(2)
    main(sys.argv[1])
