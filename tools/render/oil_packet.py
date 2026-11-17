#!/usr/bin/env python3
"""oil_packet.py — the land oil's terrain patches, as a packet holds them (item 4309).

    python3 tools/render/oil_packet.py <frame-snapshot.bin> > <run>-oil.txt

`docs/RENDERER.md` §3.24, "The land's oil". No function is called: this
reads the `OilPatchData` records `TerrainOil::create_doobers@008a28e0`
filled at load (through `add_rocky_patch@008a2b30`, whose offsets are
`internal_random`'s), which `TerrainOil::add_wcoord_for_render@008a26f0`
hands to `TerrainOut::add_weather_underlay@00874d00` once per drawn cell.

- `terrain` (`0x00C0B0F0`, the static `TerrainOut`); `terrain_oil` at
  `+0x4d5c` (`TerrainOut::render@008558b0` passes it; `+0x4d1c` in the type
  record, the usual 0x40 past it). `TerrainOil::patch_data` is a
  `PtrArray<OilPatchData>` at `+0x4`: its length at `+0x8`, its list at
  `+0x14`, one entry a cell, `y · W + x`.
- `OilPatchData` (type record, `0x3c`): `rock_patches_always` `+0x0` and
  `rock_patches_rough` `+0x1c`, each a `SimpleArray<int>` (length `+4`,
  data `+0x10`) of tiles `y · 4W + x`; `is_oil` `+0x38`, `is_vis` `+0x39`,
  `is_seen` `+0x3a`.
- `GameAccess::world` (the word at `0x00C06188`): `W` at `+0x0`; the cell's
  flags a `u16` at `[+0x134] + 0x1c · i`; the byte `add_wcoord_for_render`
  tests at `[+0x138] + (8y + 4) · [+0x18] + 4 + 8x`.

One header line (`frame f width w cells n`), then per patch a `cell x y
oil o vis v seen s flags 0xNNNN b138 0xNN` line, an `always x,y …` line and
a `rough x,y …` line of tiles. Nothing it prints enters the repository.
"""

import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
from snapshot import Snapshot  # noqa: E402

TERRAIN = 0x00C0B0F0
OIL = 0x4D5C
WORLD = 0x00C06188


def main(path):
    s = Snapshot(path)
    world = s.u32(WORLD)
    width = s.u32(world)
    cells = s.u32(world + 0x134)
    tbl = s.u32(world + 0x138)
    stride = s.u32(world + 0x18)
    n = s.u32(TERRAIN + OIL + 0x8)
    lst = s.u32(TERRAIN + OIL + 0x14)
    out = [f"frame {s.frame} width {width} cells {n}"]
    tw = 4 * width

    def ints(o):
        k = s.u32(o + 4)
        return struct.unpack_from(f"<{k}i", s.read(s.u32(o + 0x10), 4 * k)) if k else ()

    for i in range(n):
        p = s.u32(lst + 4 * i)
        if not p:
            continue
        x, y = i % width, i // width
        oil, vis, seen = s.read(p + 0x38, 3)
        flags = struct.unpack_from("<H", s.read(cells + 0x1C * i, 2))[0]
        b138 = s.read(tbl + (8 * y + 4) * stride + 4 + 8 * x, 1)[0]
        out.append(
            f"cell {x} {y} oil {oil} vis {vis} seen {seen} flags 0x{flags:04x} b138 0x{b138:02x}"
        )
        for tag, off in (("always", 0x0), ("rough", 0x1C)):
            tiles = " ".join(f"{t % tw},{t // tw}" for t in ints(p + off))
            out.append(f"{tag} {tiles}".rstrip())
    print("\n".join(out))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
