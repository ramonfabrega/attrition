#!/usr/bin/env python3
"""surf_packet.py — the surf as a packet holds it (item 4225).

    python3 tools/render/surf_packet.py <frame-snapshot.bin> <install> > <run>-surf.txt
    python3 tools/render/surf_packet.py --sections <frame-snapshot.bin> > <run>-sections.txt
    python3 tools/render/surf_packet.py --init <frame-snapshot.bin> > <run>-surf-init.txt

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

`--sections` (item 4236, `docs/RENDERER.md` §3.26): the class's own state,
walked from `Surf::waves` `0x00c8c1b8` (`PtrArray<SurfWave>`, a
`SurfWave` one `PtrArray<SurfWaveSection>` with one slot per texture)
to each `SurfWaveSection` (0x5c): `section w k frames texframes faces
timing world_tex`, then `fv f x,y …` per `switch_node_verts` frame, `ft f
u,v …` per `switch_node_texs0` frame and `faces …`; then
`Surf::percent_timeline` `0x00c8c228` and `alpha_timeline` `0x00c8c244`
as `percent t …`, `alpha t …`, and `Surf::wave_shape_index` `0x00ebf7c4`
(two inline `SimpleArray<int>`) as `shape s [..]`.

`--init` (item 4236): what `Surf::init@008a0cf0` reads and writes. The
header `surf frame F width W wcoords X Y river_deltas D location_offset
L` (`World` at `*0x00c061d0`: +0 `xs`, +0x18 `tile_xs`, +0x1c `tile_ys`;
`surf` +0xfc; `0x00c062ac`); `chain c t …` per
`TerrainOut::sorted_waterline_tcoords` (`*0x00c06218` +0x47c4 count,
+0x47d0 data, 0x1c an element); `flags y <hex>` per row of
`WorldData::tdata` (+0x138, a little-endian `u16` per tile); and `wave
…`/`breaker …` per `WaveStruct`: `wx wy x y x_axis_x x_axis_y y_axis_x
y_axis_y shape type time`.
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


def sections(path):
    s = Snapshot(path)
    u32 = lambda a: struct.unpack("<I", s.read(a, 4))[0]  # noqa: E731
    i32 = lambda a: struct.unpack("<i", s.read(a, 4))[0]  # noqa: E731

    def arr(a):
        return i32(a + 4), u32(a + 0x10)

    def pairs(ta):
        nv, dv = arr(ta)
        return " ".join("{!r},{!r}".format(*struct.unpack("<2f", s.read(dv + 8 * i, 8))) for i in range(nv))

    nw, dw = arr(0x00C8C1B8)
    print(f"frame {s.frame} waves {nw}")
    for w in range(nw):
        ns, ds = arr(u32(dw + 4 * w))
        for k in range(ns):
            sec = u32(ds + 4 * k)
            if sec == 0:
                continue
            (nf, df), (nt, dt), (nfa, dfa) = arr(sec), arr(sec + 0x1C), arr(sec + 0x38)
            print(f"section {w} {k} frames {nf} texframes {nt} faces {nfa} timing {i32(sec + 0x54)} "
                  f"world_tex {i32(sec + 0x58)}")
            for f in range(nf):
                print("fv", f, pairs(u32(df + 4 * f)))
            for f in range(nt):
                print("ft", f, pairs(u32(dt + 4 * f)))
            print("faces", " ".join(str(struct.unpack("<H", s.read(dfa + 2 * i, 2))[0]) for i in range(nfa)))
    for name, at, fmt in (("percent", 0x00C8C228, "f"), ("alpha", 0x00C8C244, "i")):
        n, d = arr(at)
        for t in range(n):
            m, dd = arr(u32(d + 4 * t))
            print(name, t, " ".join(repr(struct.unpack("<" + fmt, s.read(dd + 4 * i, 4))[0]) for i in range(m)))
    for sh in range(2):
        m, dd = arr(0x00EBF7C4 + 0x1C * sh)
        print("shape", sh, [i32(dd + 4 * i) for i in range(m)])


def init(path):
    s = Snapshot(path)
    u32 = lambda a: struct.unpack("<I", s.read(a, 4))[0]  # noqa: E731
    i32 = lambda a: struct.unpack("<i", s.read(a, 4))[0]  # noqa: E731
    terrain, world = u32(0x00C06218), u32(0x00C061D0)
    w18, h1c = i32(world + 0x18), i32(world + 0x1C)
    print(f"surf frame {s.frame} width {i32(world)} wcoords {w18} {h1c} river_deltas {s.read(SURF + 0xFC, 1)[0]} "
          f"location_offset {i32(0x00C062AC)}")
    n, d = i32(terrain + 0x47C4), u32(terrain + 0x47D0)
    for c in range(n):
        m, dd = i32(d + 0x1C * c + 4), u32(d + 0x1C * c + 0x10)
        print("chain", c, " ".join(str(i32(dd + 4 * i)) for i in range(m)))
    raw = s.read(u32(world + 0x138), 2 * w18 * h1c)
    for y in range(h1c):
        print("flags", y, raw[2 * w18 * y:2 * w18 * (y + 1)].hex())
    for name, at in (("wave", SURF + 0x40), ("breaker", SURF + 0x5C)):
        k, dd = i32(at + 4), u32(at + 0x10)
        for i in range(k):
            b = dd + 0x2C * i
            f = struct.unpack("<6f", s.read(b + 8, 24))
            print(name, i32(b), i32(b + 4), " ".join(repr(v) for v in f), i32(b + 0x20), i32(b + 0x24),
                  i32(b + 0x28))


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--sections":
        sections(sys.argv[2])
    elif len(sys.argv) == 3 and sys.argv[1] == "--init":
        init(sys.argv[2])
    elif len(sys.argv) == 3:
        main(sys.argv[1], sys.argv[2])
    else:
        print(__doc__)
        sys.exit(2)
