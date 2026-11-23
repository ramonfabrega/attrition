#!/usr/bin/env python3
"""pings_packet.py — the minimap's pings, as a packet holds them (item 4603).

    python3 tools/render/pings_packet.py <frame-snapshot.bin> > <run>-pings.txt

`docs/RENDERER.md` §3.34. No function is called. This reads what
`WorldMap::render_ping@008ca6d0` reads at the packet's frame boundary, and
what its texture pings left behind:

- the game (`GameAccessConst::gamec`, cell `0x00c061e8`): `frame` `+0x550`,
  `graphic_tick` `+0x844`;
- the message window (`MiscAccess::message_win`, cell `0x00c06208`): its
  `events` list (`+0x48c`, a `MessageWinList`; count `+0x10`, head `+0x14`;
  each node `next` `+0x0`, `data` `+0x8`), one `MessageWinEntry` a node:
  `x` `+0x4`, `y` `+0x8`, `frame` `+0xc`, `ping` `+0x10`, `ping_radius`
  `+0x14`, `ping_stamp` `+0x18`, `ping_times` `+0x1c`, `doober` `+0x20`,
  `str` `+0x2c`, `color` `+0x68` (its first word, red in the low byte);
- the scene's own pings (`MiscAccess::scene`, cell `0x00c0620c`, `ping_x`
  `+0x230`, count `+0x4`) and the ping splines (`MiscAccess::ping_spline_gen`,
  cell `0x00c06224`, `pingspline_list` `+0x178`, count `+0x4`);
- item 4601a: the profile's minimap mode (`player_profile.prefs.stat_mode`,
  `0x00ecd074`) and the game's `semaphore` bytes 0 and 1 (`Game +0x814`, a
  `BitMask<256>` whose bytes are at its `+0xc`);
- the minimap (`MiscAccess::worldmap`, cell `0x00e399ac`): its seven
  `TexPing` pointers (`+0x10a0` to `+0x10b8`), each ping's phases
  (`TexPing` `+0x18` count, `+0x24` list; a `TexPingPhase` is `0x20`), the
  background's slot `+0x40` (the function and what `tex_xs`/`tex_ys` hold),
  and per ping texture the `tex_ping_verts`, `_colors` and `_texs`
  (`PtrArray`s at `+0xec8`, `+0xee4`, `+0xf00`; count `+0x4`, list `+0x10`).
  `render_texture_pings` zeroes those counts after it draws, so the count
  reads 0; the vertex data past it is the last frame's, printed as far as
  the array's capacity (`+0x8`).

Nothing it prints enters the repository.
"""
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "recomp"))
from snapshot import Snapshot  # noqa: E402

GAMEC, MESSAGE_WIN, SCENE, SPLINES, WORLDMAP = (
    0x00C061E8, 0x00C06208, 0x00C0620C, 0x00C06224, 0x00E399AC)
# `player_profile.prefs.stat_mode`, the dword `WorldMap::on_redraw` compares
# at `8c8437`.
PROFILE_STAT_MODE = 0x00ECD074
PINGS = ("attack", "wonder", "citycap", "cityloss", "waypoint", "normal", "rare")


def main():
    s = Snapshot(sys.argv[1])
    i32 = lambda va: struct.unpack("<i", s.read(va, 4))[0]
    f32 = lambda va: struct.unpack("<f", s.read(va, 4))[0]

    def text(va):
        guts = s.u32(va)
        if not guts:
            return ""
        p, n = s.u32(guts), i32(guts + 4)
        if not p or not 0 <= n < 512:
            return "?"
        return s.read(p, 2 * n).decode("utf-16-le", "replace")

    game = s.u32(GAMEC)
    print(f"frame {s.frame} game.frame {i32(game + 0x550)} graphic_tick {i32(game + 0x844)}")
    # 4601a: the profile's minimap mode and the game's semaphore bytes.
    sem = s.read(game + 0x814 + 0xC, 2)
    print(f"profile stat_mode {i32(PROFILE_STAT_MODE)} semaphore0 {sem[0]:02x} semaphore1 {sem[1]:02x}")
    mw = s.u32(MESSAGE_WIN)
    ev = mw + 0x48C
    n = i32(ev + 0x10)
    print(f"events {n}")
    node = s.u32(ev + 0x14)
    for k in range(n):
        e = s.u32(node + 8)
        fields = [i32(e + o) for o in (0x4, 0x8, 0xC, 0x10, 0x14, 0x18, 0x1C, 0x20)]
        print(f"event {k} x {fields[0]} y {fields[1]} frame {fields[2]} ping {fields[3]} "
              f"radius {fields[4]} stamp {fields[5]} times {fields[6]} doober {fields[7]} "
              f"color {s.u32(e + 0x68):08x} str {text(e + 0x2C)!r}")
        node = s.u32(node)
    scene = s.u32(SCENE)
    print(f"scene_pings {i32(scene + 0x230 + 4)}")
    print(f"splines {i32(s.u32(SPLINES) + 0x178 + 4)}")
    wm = s.u32(WORLDMAP)
    bg = wm + 0x294
    slot = s.u32(s.u32(bg) + 0x40)
    print(f"background slot40 {slot:08x} tex_xs {i32(bg + 0x60)} tex_ys {i32(bg + 0x64)}")
    for k, name in enumerate(PINGS):
        tp = s.u32(wm + 0x10A0 + 4 * k)
        m, lst = i32(tp + 0x18), s.u32(tp + 0x24)
        print(f"texping {name} rotates {i32(tp + 0x30)} speed {f32(tp + 0x34)} phases {m}")
        for j in range(m):
            ph = lst + 0x20 * j
            print(f"  phase {j} tex {i32(ph)} alpha {i32(ph + 4)} scale {i32(ph + 8)} "
                  f"by {f32(ph + 0xC)} speed {f32(ph + 0x10)} start {f32(ph + 0x14)} "
                  f"end {f32(ph + 0x18)} color {s.u32(ph + 0x1C):08x}")
    nt = i32(wm + 0xEAC + 4)
    for t in range(nt):
        va = s.u32(s.u32(wm + 0xEC8 + 0x10) + 4 * t)
        ca = s.u32(s.u32(wm + 0xEE4 + 0x10) + 4 * t)
        ta = s.u32(s.u32(wm + 0xF00 + 0x10) + 4 * t)
        cnt, cap = i32(va + 4), i32(va + 8)
        print(f"texture {t} verts {cnt} cap {cap} colors {i32(ca + 4)} cap {i32(ca + 8)} "
              f"texs {i32(ta + 4)} cap {i32(ta + 8)}")
        vd, cd, td = s.u32(va + 0x10), s.u32(ca + 0x10), s.u32(ta + 0x10)
        for v in range(max(cap, 0)):
            x, y, z, w = (f32(vd + 16 * v + 4 * q) for q in range(4))
            u, vv = f32(td + 8 * v), f32(td + 8 * v + 4)
            col = s.u32(cd + 4 * v) if v < i32(ca + 8) else 0
            print(f"  vert {t} {v} {x:.4f} {y:.4f} {z:.3f} {w:.3f} uv {u:.4f} {vv:.4f} {col:08x}")


if __name__ == "__main__":
    main()
