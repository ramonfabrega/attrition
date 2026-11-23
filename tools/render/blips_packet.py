#!/usr/bin/env python3
"""blips_packet.py — the minimap's blips, as a packet holds them (item 4596).

    python3 tools/render/blips_packet.py <frame-snapshot.bin> > <run>-blips.txt

`docs/RENDERER.md` §3.33. No function is called: this reads what
`WorldMap::render_blips@008cc700` left in the minimap's five layers at the
packet's frame boundary. The object is `MiscAccess::worldmap`
(`0x00e399ac`), a `WorldMap`; per layer `L` (0 to 4) its `Vert4Array`s
are `0x1c` apart, count `+0x4`, data `+0x10`:

- `blip_verts` `+0xbf8`: `(x, y, z, rhw)` floats a vertex;
- `blip_texs` `+0xd10`: `(u, v)` floats a vertex;
- `blip_colors` `+0xd9c`: one colour word a vertex, red in its low byte.

`add_blip`'s window, `minimap_w` `+0x1344`, `minimap_h` `+0x1348`,
`minimap_x_offset` `+0x134c` and `minimap_y_offset` `+0x1350`, and the
minimap camera's `to_screen_matrix` (`cam` `+0x808`, the matrix `+0x240`)
print first. Then one line a vertex quadruple (every blip is four
vertices, faces `(0, 1, 2)`, `(0, 2, 3)`):

    blip <layer> <k> x0 y0 ... x3 y3 u0 v0 ... u3 v3 c0 c1 c2 c3

floats as their IEEE bits in hex, colours as hex words. Nothing it prints
enters the repository.
"""
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "recomp"))
from snapshot import Snapshot  # noqa: E402

WORLDMAP = 0x00E399AC


def main():
    s = Snapshot(sys.argv[1])
    wm = s.u32(WORLDMAP)
    i32 = lambda va: struct.unpack("<i", s.read(va, 4))[0]
    bits = lambda va: s.u32(va)
    print(f"frame {s.frame}")
    print("window " + " ".join(str(i32(wm + o)) for o in (0x1344, 0x1348, 0x134C, 0x1350)))
    print("matrix " + " ".join(f"{bits(wm + 0x808 + 0x240 + 4 * k):08x}" for k in range(16)))
    for layer in range(5):
        va, ta, ca = (wm + base + layer * 0x1C for base in (0xBF8, 0xD10, 0xD9C))
        n, tn, cn = i32(va + 4), i32(ta + 4), i32(ca + 4)
        assert n % 4 == 0 and tn == n and cn == n, (layer, n, tn, cn)
        vd, td, cd = s.u32(va + 0x10), s.u32(ta + 0x10), s.u32(ca + 0x10)
        print(f"layer {layer} {n // 4}")
        for q in range(n // 4):
            xy, uv, col = [], [], []
            for k in range(4):
                v = q * 4 + k
                xy += [f"{bits(vd + v * 16):08x}", f"{bits(vd + v * 16 + 4):08x}"]
                uv += [f"{bits(td + v * 8):08x}", f"{bits(td + v * 8 + 4):08x}"]
                col.append(f"{bits(cd + v * 4):08x}")
            print(f"blip {layer} {q} " + " ".join(xy + uv + col))


if __name__ == "__main__":
    main()
