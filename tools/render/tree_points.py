#!/usr/bin/env python3
"""tree_points.py — the original's map trees on a packet (item 4080).

    uv run --locked --project tools/emu python tools/render/tree_points.py \
        <install>/riseofnations.exe <frame-snapshot.bin> [--sweep CX,CY ...]

`docs/RENDERER.md` §3.7. The packet holds no per-tree record — the 3D trees
live in `RTreeData`'s cache as assembled vertices — so this calls the
original's own functions on the packet, as `ObjectsOut::sort_trees@00656c70`
does each time the fog or the camera moves: for every cell of the visible
list (`TerrainData::terrain_fog_x`/`_y`) whose `WData.flags` has `0x20`,
`ObjectsData::make_tree_mask@00658880`, then
`ObjectsData::place_tree@00658530` for each of the five slots at the cell's
centre `(x·0x300 + 0x180, y·0x300 + 0x180)`, then
`MiscAccess::world_to_screen@00583810` on the place. Rows:

    state <section_render[10]> <tree_scale bits> <low_tree_scale bits> <models> <tileset>
    forest <x> <y>                        every cell of the map with 0x20
    tree <cx> <cy> <mask> <slot> <x> <y> <z> <model> -> <sx> <sy>
    sweep <cx> <cy> <mask> <slot> <x> <y> <z>
    cache <cx> <cy> <tick> <minx> <miny> <minz> <maxx> <maxy> <maxz>
    section <cx> <cy> <sec> <verts> <struct_size> <faces>
    rstate <index> <enables> <zfunc> <alpha_func> <alpha_ref> <src_blend> <dst_blend> <cull_mode>
    rstage <index> <tfactor> <colorop0> <colorarg1_0> <colorarg2_0> <alphaop0> <alphaarg1_0> <alphaarg2_0>

`section` is each cache entry's seven sections: `cache_verts[sec].length`,
its `struct_size` in dwords and `cache_faces[sec]`'s length (item 4082).
`rstate` is `render_states[9]`, the `RenderState` `TreeFlushMethod::execute`
sets (`GraphicPieces::init_tree_map@008f7c20` gives it the method), as the
process holds it on the packet: the `RenderState` fields at their PDB
offsets (`enables` 0x50, `zfunc` 0x58, `cull_mode` 0x70, `src_blend` 0x74,
`dst_blend` 0x78, `alpha_func` 0x7c, `alpha_ref` 0x80). Item 4085 adds
`rstate` and `rstage` rows for `render_states[0..3]`, the land units'
(`GraphicPieces::init_land_unit_map@008fa4b0`); `rstage` is `tfactor`
(0x5c) and stage 0's `colorop`/`colorarg1`/`colorarg2`/`alphaop`/
`alphaarg1`/`alphaarg2` (0xac, 0xb4, 0xbc, 0xc4, 0xcc, 0xd4).

`cache` is each `RTreeData::tree_cache` entry the packet holds: the cell
(`TreeCacheEntry::wx`/`wy`), its `last_graphic_tick`, and the extent of the
world-space vertices it assembled over its seven sections (`cache_verts`,
a `VoidArray` whose `struct_size` counts dwords, `xyz` first).

`model` is `GraphicPieces::draw_tree@00905a10`'s `(int)(y + x) % count`.
`--sweep` adds every mask 0..15 and slot 0..4 at each named cell.
`--guys GPIECE:WHO/O ...` (item 4086) prints instead, for each, every
`GuyData` the packet holds with that `gpiece` (+0x88), `o` (+0x8c) and
`who` (+0xa1): its point, `angle`, `cur_time`/`end_time`, `cur_anim`,
`stopped` and `last_*`. Nothing it prints enters the repository.
"""
import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "emu"))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
sys.path.insert(0, HERE)
from callfn import Image  # noqa: E402
from snapshot import Snapshot  # noqa: E402
from packet_points import SCRATCH, WORLD_TO_SCREEN, PacketMachine  # noqa: E402

PLACE_TREE, MAKE_TREE_MASK = 0x00658530, 0x00658880
FOG_N, FOG_X, FOG_Y = 0x00C8BF9C, 0x00C8BFA8, 0x00C8BFC4
WORLDC, GRAPHSYS, GRAPHIC_PIECES = 0x00C061D0, 0x00E335A0, 0x00C06214
TREE_SCALE, LOW_TREE_SCALE = 0x00C06340, 0x00C062CC
# `tilesets` (S_GDATA32 0003:2631120), its `cur_tileset` at +0x20
TILESETS = 0x00E885D0
# `render_states` (`Array<RenderState *>`, rise_z.map 0003:0000bd2c), its `list` at +0x10
RENDER_STATES = 0x00C11D2C


def guys(snap, wants):
    """Item 4086: every `GuyData` the packet holds for `wants`, a list of
    `(gpiece, who, o)`, found by `gpiece` (+0x88) and `o` (+0x8c) side by
    side and checked by `who` (+0xa1), at the PDB's offsets."""
    pats = {struct.pack("<ih", g, o): (g, who, o) for g, who, o in wants}
    for base, size, off in snap.ranges:
        snap.f.seek(off)
        data = snap.f.read(size)
        for pat, (g, who, o) in pats.items():
            i = data.find(pat)
            while i >= 0:
                at = i - 0x88
                if at >= 0 and at + 0xBC <= size and data[at + 0xA1] == who:
                    r = lambda o_, f: struct.unpack_from(f, data, at + o_)[0]
                    print(
                        f"guy {who}/{o} {base + at:08x} type {r(0x8, '<i')} x {r(0xC, '<i')} "
                        f"{r(0x10, '<i')} {r(0x14, '<i')} angle {r(0x18, '<I'):#x} "
                        f"last_angle {r(0x1C, '<I'):#x} cur_time {r(0x74, '<I')} "
                        f"end_time {r(0x78, '<I')} last_time {r(0x7C, '<i')} gpiece {g} "
                        f"variation {r(0x90, '<h')} cur_anim {r(0x9C, '<b')} "
                        f"stopped {data[at + 0x9D]} guy_num {data[at + 0xA2]} "
                        f"last {r(0x68, '<i')} {r(0x6C, '<i')} {r(0x70, '<i')}"
                    )
                i = data.find(pat, i + 1)


def main(argv):
    if len(argv) < 3:
        print(__doc__)
        return 2
    exe, packet = argv[1], argv[2]
    if argv[3:4] == ["--guys"]:
        wants = []
        for w in argv[4:]:
            g, wo = w.split(":")
            who, o = wo.split("/")
            wants.append((int(g), int(who), int(o)))
        guys(Snapshot(packet), wants)
        return 0
    sweeps = []
    rest = argv[3:]
    while rest:
        if rest[0] != "--sweep" or len(rest) < 2:
            print(__doc__)
            return 2
        cx, cy = (int(v) for v in rest[1].split(","))
        sweeps.append((cx, cy))
        rest = rest[2:]
    snap = Snapshot(packet)
    m = PacketMachine(Image(exe), snap)
    m.uc.mem_map(0, 0x1000)  # fs:0, the SEH chain head, at fs's base 0
    u32 = lambda a: struct.unpack("<I", snap.read(a, 4))[0]
    sr = u32(u32(GRAPHSYS) + 0x13C)
    td = u32(u32(GRAPHIC_PIECES) + 0x60)
    models = u32(td + 0xAC + 4)
    ts = u32(u32(TILESETS + 0x20))
    name = snap.read(u32(ts), 64).decode("utf-16-le", "replace").split("\0")[0]
    print(f"state {snap.read(sr + 10, 1)[0]} {u32(TREE_SCALE):08x} {u32(LOW_TREE_SCALE):08x} {models} {name}")
    w = u32(WORLDC)
    width, height, tiles = u32(w), u32(w + 4), u32(w + 0x134)
    forest = lambda x, y: struct.unpack("<H", snap.read(tiles + (width * y + x) * 0x1C, 2))[0] & 0x20
    for y in range(height):
        for x in range(width):
            if forest(x, y):
                print(f"forest {x} {y}")
    scr = SCRATCH + 0x100

    def place(cx, cy, mask, slot):
        m.uc.mem_write(scr + 0x10, struct.pack("<4f", cx * 0x300 + 0x180, cy * 0x300 + 0x180, 0.0, 1.0))
        m.call(PLACE_TREE, (scr + 0x40, scr + 0x10, slot, mask), ecx=SCRATCH, edx=0)
        x, y, z, _ = struct.unpack("<4f", m.uc.mem_read(scr + 0x40, 16))
        return int(x), int(y), int(z)

    for k in range(u32(FOG_N)):
        cx, cy = u32(u32(FOG_X) + 4 * k), u32(u32(FOG_Y) + 4 * k)
        if not forest(cx, cy):
            continue
        m.call(MAKE_TREE_MASK, (cx, cy, scr), ecx=SCRATCH, edx=0)
        mask = struct.unpack("<I", m.uc.mem_read(scr, 4))[0]
        for slot in range(5):
            if snap.read(sr + 10, 1)[0] == 0 and slot == 2:
                continue
            x, y, z = place(cx, cy, mask, slot)
            m.call(WORLD_TO_SCREEN, (x, y, z, 0, 0), ecx=SCRATCH, edx=SCRATCH + 4)
            sx, sy = struct.unpack("<ii", m.uc.mem_read(SCRATCH, 8))
            print(f"tree {cx} {cy} {mask} {slot} {x} {y} {z} {(x + y) % models} -> {sx} {sy}")
    i32 = lambda a: struct.unpack("<i", snap.read(a, 4))[0]
    for k in range(i32(td + 0xE4 + 4)):
        e = u32(u32(td + 0xE4 + 0x10) + 4 * k)
        pts = []
        for sec in range(7):
            va = e + 0x20 * sec
            base, n, stride = u32(va), i32(va + 4), 4 * u32(va + 0x14)
            if base and n > 0:
                raw = snap.read(base, n * stride)
                pts += [struct.unpack_from("<3f", raw, i * stride) for i in range(n)]
        lo = [min(p[j] for p in pts) for j in range(3)]
        hi = [max(p[j] for p in pts) for j in range(3)]
        print(f"cache {i32(e + 0x3F0)} {i32(e + 0x3F4)} {i32(e + 0x3F8)} " + " ".join(f"{v:.3f}" for v in lo + hi))
        for sec in range(7):
            va, fa = e + 0x20 * sec, e + 0xE0 + 0x1C * sec
            print(f"section {i32(e + 0x3F0)} {i32(e + 0x3F4)} {sec} {i32(va + 4)} {u32(va + 0x14)} {i32(fa + 4)}")
    rs = u32(u32(RENDER_STATES + 0x10) + 4 * 9)
    fields = [u32(rs + o) for o in (0x50, 0x58, 0x7C, 0x80, 0x74, 0x78, 0x70)]
    print(f"rstate 9 {fields[0]:#x} " + " ".join(str(v) for v in fields[1:]))
    # Item 4085: the land units' states, `init_land_unit_map@008fa4b0`'s
    # method lists 0..4 over `render_states[0..3]`, with the stage-0 ops.
    for k in range(4):
        rs = u32(u32(RENDER_STATES + 0x10) + 4 * k)
        f = [u32(rs + o) for o in (0x50, 0x58, 0x7C, 0x80, 0x74, 0x78, 0x70)]
        print(f"rstate {k} {f[0]:#x} " + " ".join(str(v) for v in f[1:]))
        g = [u32(rs + o) for o in (0x5C, 0xAC, 0xB4, 0xBC, 0xC4, 0xCC, 0xD4)]
        print(f"rstage {k} {g[0]:#010x} " + " ".join(str(v) for v in g[1:]))
    for cx, cy in sweeps:
        for mask in range(16):
            for slot in range(5):
                x, y, z = place(cx, cy, mask, slot)
                print(f"sweep {cx} {cy} {mask} {slot} {x} {y} {z}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
