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

`model` is `GraphicPieces::draw_tree@00905a10`'s `(int)(y + x) % count`.
`--sweep` adds every mask 0..15 and slot 0..4 at each named cell. Nothing
it prints enters the repository.
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


def main(argv):
    if len(argv) < 3:
        print(__doc__)
        return 2
    exe, packet = argv[1], argv[2]
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
    for cx, cy in sweeps:
        for mask in range(16):
            for slot in range(5):
                x, y, z = place(cx, cy, mask, slot)
                print(f"sweep {cx} {cy} {mask} {slot} {x} {y} {z}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
