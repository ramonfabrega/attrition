#!/usr/bin/env python3
"""surf_packet.py — the surf as a packet holds it (item 4225).

    python3 tools/render/surf_packet.py <frame-snapshot.bin> <install> > <run>-surf.txt

`docs/RENDERER.md` §3.18, "The surf, off the packet". No function is called:
this reads the meshes `Surf::draw@0089d6f0` left in the image's statics at
the packet's frame boundary, the ones the card draws as 163–169 on run1567.

- `Surf::final_verts` `0x00c8c0bc`, `final_tex0` `0x00c8c0d8`, `final_tex1`
  `0x00c8c0f4`, `final_color` `0x00c8c110`, `final_faces` `0x00c8c12c`
  (`sbl/rise_z.map`): one `PtrArray` (+4 count, +0x10 data) of per-texture
  arrays each, list k drawn with `surf_texture[k]` on stage 0 and
  `surf_texture[surf_texture_alpha_index[k]]` on stage 1.
- `surf_texture` `0x00c8c1d4`, `surf_texture_alpha_index` `0x00c8c1f0`,
  `surf_texture_rs_index` `0x00c8c20c`. The textures' own `filename`
  strings are empty on the packet, so a list's files are named by
  `<SURF>`'s `<TEXTURE>` elements in the install's
  `Data/effects_graphics.xml`, in order, which `Surf::init_class@0089f050`
  appends one by one.
- `surf` `0x00eb6b80` (`game.obj`): `wave_structs` +0x40,
  `breaker_structs` +0x5c (`Array<WaveStruct>`, 0x2c a record),
  `on_screen_waves` +0x78, `on_screen_breakers` +0x94, `wave_total_time`
  +0xb8, `wave_stall_time` +0xbc.

One header line `surf …`; then per non-empty list `list k texture mask rs
verts triangles`, its vertices `v x y z u0 v0 u1 v1 alpha` (floats as
decimals; the colour's rgb is asserted white) and its faces `f a b c`.
Nothing it prints enters the repository.
"""

import os
import re
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
from snapshot import Snapshot  # noqa: E402

FINAL_VERTS, FINAL_TEX0, FINAL_TEX1 = 0x00C8C0BC, 0x00C8C0D8, 0x00C8C0F4
FINAL_COLOR, FINAL_FACES = 0x00C8C110, 0x00C8C12C
SURF_TEXTURE, ALPHA_INDEX, RS_INDEX = 0x00C8C1D4, 0x00C8C1F0, 0x00C8C20C
SURF = 0x00EB6B80


def surf_files(install):
    """`<SURF>`'s `<TEXTURE>` files, in order, by their last component."""
    with open(os.path.join(install, "Data", "effects_graphics.xml"), encoding="utf-8", errors="replace") as f:
        text = f.read()
    m = re.search(r"<SURF\b.*?</SURF>", text, re.S | re.I)
    if not m:
        sys.exit("effects_graphics.xml: no <SURF> element")
    return [os.path.basename(p.replace("\\", "/")) for p in re.findall(r'<TEXTURE\b[^>]*\bfile="([^"]*)"', m.group(0))]


def main(path, install):
    s = Snapshot(path)
    u16 = lambda a: struct.unpack("<H", s.read(a, 2))[0]  # noqa: E731
    u32 = lambda a: struct.unpack("<I", s.read(a, 4))[0]  # noqa: E731
    i32 = lambda a: struct.unpack("<i", s.read(a, 4))[0]  # noqa: E731

    def array(at):
        return i32(at + 4), u32(at + 0x10)

    def entry(ptr_array, k):
        return array(u32(array(ptr_array)[1] + 4 * k))

    files = surf_files(install)
    n, _ = array(SURF_TEXTURE)
    if n != len(files):
        sys.exit(f"{path}: {n} surf textures, the XML names {len(files)}")
    na, da = array(ALPHA_INDEX)
    nr, dr = array(RS_INDEX)
    alpha = [i32(da + 4 * i) for i in range(na)]
    rs = [i32(dr + 4 * i) for i in range(nr)]
    print(f"surf frame {s.frame} waves {array(SURF + 0x40)[0]} breakers {array(SURF + 0x5C)[0]} "
          f"on_screen {array(SURF + 0x78)[0]} {array(SURF + 0x94)[0]} "
          f"total_time {i32(SURF + 0xB8)} stall_time {i32(SURF + 0xBC)}")
    lists, _ = array(FINAL_VERTS)
    for k in range(lists):
        nv, dv = entry(FINAL_VERTS, k)
        if not nv:
            continue
        (n0, d0), (n1, d1), (nc, dc) = entry(FINAL_TEX0, k), entry(FINAL_TEX1, k), entry(FINAL_COLOR, k)
        nf, df = entry(FINAL_FACES, k)
        if not nv == n0 == n1 == nc or nf % 3:
            sys.exit(f"{path}: list {k}'s arrays disagree ({nv}, {n0}, {n1}, {nc}, faces {nf})")
        mask = files[alpha[k]] if alpha[k] >= 0 else "-"
        print(f"list {k} {files[k]} {mask} {rs[k]} {nv} {nf // 3}")
        for i in range(nv):
            x, y, z = struct.unpack("<3f", s.read(dv + 12 * i, 12))
            u0, v0 = struct.unpack("<2f", s.read(d0 + 8 * i, 8))
            u1, v1 = struct.unpack("<2f", s.read(d1 + 8 * i, 8))
            c = u32(dc + 4 * i)
            if c & 0xFFFFFF != 0xFFFFFF:
                sys.exit(f"{path}: list {k} vertex {i} is not white ({c:08x})")
            print(f"v {x!r} {y!r} {z!r} {u0!r} {v0!r} {u1!r} {v1!r} {c >> 24}")
        for i in range(nf // 3):
            print("f " + " ".join(str(u16(df + 2 * (3 * i + j))) for j in range(3)))


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(__doc__)
        sys.exit(2)
    main(sys.argv[1], sys.argv[2])
