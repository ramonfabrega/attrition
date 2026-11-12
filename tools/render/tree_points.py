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
    vdiff <cx> <cy> <d3dcolor>:<vertices> ...
    rstate <index> <enables> <zfunc> <alpha_func> <alpha_ref> <src_blend> <dst_blend> <cull_mode>
    rstage <index> <tfactor> <colorop0> <colorarg1_0> <colorarg2_0> <alphaop0> <alphaarg1_0> <alphaarg2_0>

`vdiff` (item 4173) is each cache entry's vertex diffuse, its fourth dword,
and how many of its vertices carry each value. `section` is each cache entry's seven sections: `cache_verts[sec].length`,
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
`stopped` and `last_*`; item 4087 adds `bank`/`last_bank` (+0x44),
`pitch`/`last_pitch` (+0x4c) and `last_norm` (+0xa4), the slope normal
`GuyData::find_slope_normal@005de140` eases by a quarter each draw.
Nothing it prints enters the repository.
`--guytree` (item 4087) prints instead `Tree<Guy_const_*,float>` at
`objects + 0x288` (0x00C0ADF8, the `ecx` `UnitOut::draw@00615b80` passes
`ordered_insert@0046e590`), the guys pass's list, in the order
`pop_all_zoomed_in@00655e70` walks it (in order, leftmost first):

    guytree <length> <head_node> <ordered>
    gnode <i> <node> <who>/<o> type <guy +0x8> gpiece <+0x88> anim <+0x9c> time <+0x74> metric <float>
    rbias <index> <enables> <zbias> <zfunc> <stencil_func> <stencil_ref> <fail> <zfail> <pass> <mask> <write_mask>

`rbias` is `render_states[0..3]`'s `zbias` (0x54) and stencil fields
(0x84..0x9c), which `rstate` leaves out.
`--slope X,Y ...` (item 4089) prints instead, for each point, the original's
own `WorldData::find_slope_normal@006b5130` on the packet's terrain and the
three `TerrainOut::find_data_z@00866560(·, ·, 0)` heights it reads, at
`(x, y)`, `(x + 20, y)` and `(x, y + 20)`:

    slope <x> <y> z <z0> <z1> <z2> normal <nx> <ny> <nz> bits <hex hex hex>

The normal is the world function's, before `GuyData::find_slope_normal`
negates it.
`--pose WHO/O:SLOT:CLOCK ...` (item 4092) prints instead, for each, what
`GuyOut::draw@005dc490` hands the skeleton of piece 60 (the rifleman,
`--piece N` to name another) for a guy in `SLOT` at `CLOCK`: the action
`AnimationPacket::action_ids[SLOT]` names, `animmgr.times` of it, and the
time `min(CLOCK · 200 / 3, times)` (`graphic_dt` 0, as the client draws);
then the original's own `HierObj::set_to_time@00536e30` and
`update_all_transforms@00536db0` run on the piece's `HierObj`
(`RData +0x48 → +8`) on the packet, and every node in the order its
child-then-sibling walk takes them:

    packet <piece> cur_action <a> cur_time <ms> mem_grade <g> section_render11 <b>
    pose <who>/<o> slot <s> clock <c> action <a> flags6c <f> anim_time <t> len <ms> ms <ms>
      pnode <i> depth <d> start <s> num <n> dead <0|1> keys <iKey1> <iKey2> q <x y z w> t <x y z> world <x y z>

`packet` is the piece's `AnimationPacket` as the last guy drawn left it,
and the two inputs `draw_unit@008e7da0` reads before it remaps a slot
(`8e7e0b`: it does when `mem_grade` ≥ 3 or `section_render[11]` is 0).
`world` is `world_transform.trans` (+0x74).
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
                        f"last {r(0x68, '<i')} {r(0x6C, '<i')} {r(0x70, '<i')} "
                        f"bank {r(0x44, '<f')!r} {r(0x48, '<f')!r} pitch {r(0x4C, '<f')!r} "
                        f"{r(0x50, '<f')!r} last_norm {struct.unpack_from('<3f', data, at + 0xA4)}"
                    )
                i = data.find(pat, i + 1)


GUY_TREE = 0x00C0ADF8
# Item 4089: `MiscAccess::terrain` (the `ecx` `WorldData::find_slope_normal`
# loads at `6b5139`), `TerrainOut::find_data_z` and the world's slope normal.
MISC_TERRAIN, FIND_DATA_Z, WORLD_SLOPE_NORMAL = 0x00C06218, 0x00866560, 0x006B5130
SQRT_IAT = 0x00AC5530


def guytree(snap):
    """Item 4087: the guys pass's tree on the packet, in order."""
    u32 = lambda a: struct.unpack("<I", snap.read(a, 4))[0]
    length, head, ordered = u32(GUY_TREE + 8), u32(GUY_TREE + 0xC), u32(GUY_TREE + 0x18)
    print(f"guytree {length} {head:08x} {ordered}")
    out, stack, node = [], [], head
    while (stack or node) and len(out) <= max(length, 0) + 8:
        while node:
            stack.append(node)
            node = u32(node)
        node = stack.pop()
        out.append(node)
        node = u32(node + 4)
    for k in range(4):
        rs = u32(u32(RENDER_STATES + 0x10) + 4 * k)
        f = [u32(rs + 0x50)] + list(struct.unpack("<i", snap.read(rs + 0x54, 4)))
        f += [u32(rs + o) for o in (0x58, 0x84, 0x88, 0x8C, 0x90, 0x94, 0x98, 0x9C)]
        print(f"rbias {k} {f[0]:#x} " + " ".join(str(v) for v in f[1:]))
    for i, n in enumerate(out):
        g = u32(n + 0xC)
        metric = struct.unpack("<f", snap.read(n + 0x10, 4))[0]
        gd = snap.read(g, 0xBC)
        r = lambda o_, f: struct.unpack_from(f, gd, o_)[0]
        print(
            f"gnode {i} {n:08x} {gd[0xA1]}/{r(0x8C, '<h')} type {r(0x8, '<i')} "
            f"gpiece {r(0x88, '<i')} anim {r(0x9C, '<b')} time {r(0x74, '<I')} metric {metric!r}"
        )


SET_TO_TIME, UPDATE_ALL_TRANSFORMS = 0x00536E30, 0x00536DB0
# `animmgr` (rise_z.map 0003:002b96a0): `actions` +0x0, `times` +0x1c
ANIMMGR, UNITS = 0x00EBF6A0, 0x00C0AEB0
GAMEC = 0x00C061E8


def pose(exe, packet, args):
    """Item 4092: the original's pose of a guy on the packet."""
    piece = 60
    if args[:1] == ["--piece"]:
        piece, args = int(args[1]), args[2:]
    snap = Snapshot(packet)
    m = PacketMachine(Image(exe), snap)
    m.uc.mem_map(0, 0x1000)

    def rd(a, n):
        try:
            return bytes(m.uc.mem_read(a, n))
        except Exception:
            return snap.read(a, n)

    u32 = lambda a: struct.unpack("<I", rd(a, 4))[0]
    rec = u32(u32(u32(GRAPHIC_PIECES) + 0x718 + 0x10) + 4 * piece)
    hier, ap = u32(u32(rec + 0x48) + 8), u32(rec + 0x54)
    gs = u32(GRAPHSYS)
    print(f"packet {piece} cur_action {u32(ap + 8)} cur_time {u32(ap + 0xC)} "
          f"mem_grade {u32(gs + 0xE4)} section_render11 {rd(u32(gs + 0x13C) + 11, 1)[0]}")
    nodes, stack = [], [(hier, 0)]
    while stack:
        h, d = stack.pop()
        while h:
            nodes.append((h, d))
            if u32(h + 0x9C):
                stack.append((u32(h + 0x9C), d))
            h, d = u32(h + 0x98), d + 1
    ids, acts, times = u32(ap + 0x20), u32(ANIMMGR + 0x10), u32(ANIMMGR + 0x1C + 0x10)
    for w in args:
        wo, slot, clock = w.split(":")
        who, o = (int(v) for v in wo.split("/"))
        slot, clock = int(slot), int(clock)
        act = struct.unpack("<i", rd(ids + 4 * slot, 4))[0]
        obj, length = u32(acts + 4 * act), u32(times + 4 * act)
        ms = min(clock * 200 // 3, length)
        flags = rd(u32(u32(UNITS + 0x10 + who * 0x1C) + 4 * o) + 0x6C, 1)[0]
        anim_time = struct.unpack("<f", rd(obj + 4, 4))[0]
        print(f"pose {who}/{o} slot {slot} clock {clock} action {act} flags6c {flags:#x} "
              f"anim_time {anim_time!r} len {length} ms {ms}")
        m.call(SET_TO_TIME, (obj, ms), ecx=hier, max_insns=5_000_000)
        m.call(UPDATE_ALL_TRANSFORMS, (), max_insns=5_000_000)
        for i, (h, d) in enumerate(nodes):
            k1, k2 = struct.unpack("<HH", rd(h + 0xAC, 4))
            q = struct.unpack("<4f", rd(h + 0x34, 16))
            t = struct.unpack("<3f", rd(h + 0x44, 12))
            wt = struct.unpack("<3f", rd(h + 0x74, 12))
            f = lambda v, p: " ".join(f"{x:.{p}f}" for x in v)
            print(f"  pnode {i} depth {d} start {u32(h + 0xA0)} num {u32(h + 0xA4)} dead {rd(h + 0xB0, 1)[0]} "
                  f"keys {k1} {k2} q {f(q, 6)} t {f(t, 4)} world {f(wt, 3)}")


def main(argv):
    if len(argv) < 3:
        print(__doc__)
        return 2
    exe, packet = argv[1], argv[2]
    if argv[3:4] == ["--slope"]:
        snap = Snapshot(packet)
        m = PacketMachine(Image(exe), snap)
        m.uc.mem_map(0, 0x1000)
        u32 = lambda a: struct.unpack("<I", snap.read(a, 4))[0]
        terrain = u32(MISC_TERRAIN)
        # `sqrt` (0x41e6f0's `call 0xa571b0`) is the CRT's import through the
        # IAT slot 0xac5530, a double in `xmm0`: answered by `sqrtsd; ret`.
        m.uc.mem_write(SCRATCH + 0x400, bytes.fromhex("f20f51c0c3"))
        m.uc.mem_write(SQRT_IAT, struct.pack("<I", SCRATCH + 0x400))
        for w in argv[4:]:
            x, y = (int(v) for v in w.split(","))
            zs = []
            for px, py in ((x, y), (x + 20, y), (x, y + 20)):
                m.call(FIND_DATA_Z, (SCRATCH + 0x200, px, py, 0), ecx=terrain, edx=0)
                zs.append(struct.unpack("<i", m.uc.mem_read(SCRATCH + 0x200, 4))[0])
            m.uc.mem_write(SCRATCH + 0x210, struct.pack("<ii", x, y))
            m.call(WORLD_SLOPE_NORMAL, (SCRATCH + 0x210, SCRATCH + 0x214, SCRATCH + 0x220), ecx=SCRATCH + 0x300, edx=0)
            raw = m.uc.mem_read(SCRATCH + 0x220, 12)
            n = struct.unpack("<3f", raw)
            bits = " ".join(f"{v:08x}" for v in struct.unpack("<3I", raw))
            print(f"slope {x} {y} z {zs[0]} {zs[1]} {zs[2]} normal {n[0]!r} {n[1]!r} {n[2]!r} bits {bits}")
        return 0
    if argv[3:4] == ["--pose"]:
        pose(exe, packet, argv[4:])
        return 0
    if argv[3:4] == ["--guytree"]:
        guytree(Snapshot(packet))
        return 0
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
        # Item 4173: each vertex's fourth dword, the diffuse `D3DCOLOR`.
        diffuse = {}
        for sec in range(7):
            va = e + 0x20 * sec
            base, n, stride = u32(va), i32(va + 4), 4 * u32(va + 0x14)
            if base and n > 0 and stride >= 16:
                raw = snap.read(base, n * stride)
                for i in range(n):
                    w = struct.unpack_from("<I", raw, i * stride + 12)[0]
                    diffuse[w] = diffuse.get(w, 0) + 1
        print(f"vdiff {i32(e + 0x3F0)} {i32(e + 0x3F4)} " + " ".join(f"{w:08x}:{c}" for w, c in sorted(diffuse.items())))
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
