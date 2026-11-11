#!/usr/bin/env python3
"""land_buckets.py — the land pass's own bucket for every cell, on a packet (item 4129).

    uv run --locked --project tools/emu python tools/render/land_buckets.py <frame-snapshot.bin>

`docs/RENDERER.md` §3.6, "The ground's base layer". No function is called:
this reads what `TerrainOut::generate_land_lists@0085f6a0` left in the
process. `MiscAccess::terrain` (0x00C06218) is the `TerrainOut`; its
`+0x47d8` is the array of `width + height` `TerrainList`s (0xc each) the
generator fills one diagonal at a time; each list's `pLinearCoords` (+8)
chains every `CoordInfo` (0x80) by `pNextLinear` (+8), and a `CoordInfo`
carries the cell (`xw` +0x14, `yw` +0x18), its `flags` (+0x1c), its
`patches` (+0x2c) and the bucket `TileSet::convert_land_tex_to_linear
@0087ace0` gave it (`trans_bucket`, +0x30, a `char`). Rows:

    tileset <name> buckets <TerrainList::num_buckets> bases <n>
    base <i> <textures> <supertile_filename>
    cell <xw> <yw> land <WData +2> sub <WData +3> wflags <WData +0, hex> bucket <trans_bucket> cflags <hex> patch <bucket/landindex|-> list <b>

`patch` is the cell's `PatchInfo` (`CoordInfo +0x2c`), which
`TerrainOut::create_transition_renderables` fills: its alpha's
`bucket` (+0x54, `AlphaInfo::bucket`) and `landindex` (+0x55), the bucket
of the base it lays over the cell. `list` is the `pCoords[b]` chain
(`TerrainList +4`, linked by `pNext` +4) the cell sits in, the list the
land pass walks.

`base` is `cur_tileset->land_textures[0]` (`TileSetData +0x14`, a
`PtrArray<TextureEffect>`), each entry's `textures.length` (+0x28) the
count `convert_land_tex_to_linear` sums. `cell` rows are sorted by
(`yw`, `xw`). Nothing it prints enters the repository.
"""

import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
from snapshot import Snapshot  # noqa: E402

WORLDC, MISC_TERRAIN = 0x00C061D0, 0x00C06218
# `tilesets` (S_GDATA32 0003:2631120), its `cur_tileset` at +0x20
TILESETS = 0x00E885D0


def main(argv):
    if len(argv) != 2:
        print(__doc__)
        return 2
    snap = Snapshot(argv[1])
    u32 = lambda a: struct.unpack("<I", snap.read(a, 4))[0]
    i32 = lambda a: struct.unpack("<i", snap.read(a, 4))[0]
    ts = u32(TILESETS + 0x20)
    name = snap.read(u32(ts), 64).decode("utf-16-le", "replace").split("\0")[0]
    nb = i32(ts + 0x268)
    lt = ts + 0x14
    n = i32(lt + 4)
    print(f"tileset {name} buckets {nb} bases {n}")
    for i in range(n):
        te = u32(u32(lt + 0x10) + 4 * i)
        fname = snap.read(u32(te), 128).decode("utf-16-le", "replace").split("\0")[0]
        print(f"base {i} {i32(te + 0x28)} {fname}")
    w = u32(WORLDC)
    width, height, tiles = u32(w), u32(w + 4), u32(w + 0x134)
    terrain = u32(MISC_TERRAIN)
    lists = u32(terrain + 0x47D8)
    rows = {}
    for k in range(width + height):
        c = u32(lists + 0xC * k + 8)
        seen = 0
        while c and seen < width * height:
            xw, yw = i32(c + 0x14), i32(c + 0x18)
            flags = struct.unpack("<H", snap.read(c + 0x1C, 2))[0]
            bucket = struct.unpack("<b", snap.read(c + 0x30, 1))[0]
            pa = u32(c + 0x2C)
            patches = f"{snap.read(pa + 0x54, 1)[0]}/{snap.read(pa + 0x55, 1)[0]}" if pa else "-"
            rows[(yw, xw)] = (bucket, flags, patches)
            c = u32(c + 8)
            seen += 1
    drawn = {}
    for k in range(width + height):
        heads = u32(lists + 0xC * k + 4)
        if not heads:
            continue
        for b in range(nb):
            c = u32(heads + 4 * b)
            seen = 0
            while c and seen < width * height:
                drawn[(i32(c + 0x18), i32(c + 0x14))] = b
                c = u32(c + 4)
                seen += 1
    for (yw, xw), (bucket, flags, patches) in sorted(rows.items()):
        wd = tiles + (width * yw + xw) * 0x1C
        wf = struct.unpack("<H", snap.read(wd, 2))[0]
        land, sub = struct.unpack("<bB", snap.read(wd + 2, 2))
        print(f"cell {xw} {yw} land {land} sub {sub} wflags {wf:04x} bucket {bucket} cflags {flags:04x} patch {patches} list {drawn.get((yw, xw), -1)}")
    print(f"cells {len(rows)} of {width * height}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
