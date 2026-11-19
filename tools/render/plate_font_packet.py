#!/usr/bin/env python3
"""plate_font_packet.py — the city plate's font, as each packet holds it (item 4423).

    python3 tools/render/plate_font_packet.py <frame-snapshot.bin>... > plate-font.txt

`docs/HUD.md` §3 and §6. No function is called: this reads the font the
plate's string names (`int_str_array +0x1680`, `onscreen_mouseover_size`,
item 4403) where the executable's font table keeps it, and the renderer
DLL's font object behind it, by the PDBs' type records:

- `fonts` (`?fonts@@3VFontFamily@@A`, `0x00ed8160` in `sbl/rise_z.map`):
  `FontFamily::lookups` (`+0x20`, an `ObjectArray<FontLookup>`: length
  `+0x24`, list `+0x30`); lookup 36 is the plate's (`FontLookup`, 0x20:
  `size` `+0x14`, `style` `+0x18`, `font` `+0x1c`); the `Font`'s first
  word is its `gs_font`;
- that object is a `D3D11GLFont` (`sbl/d3dgl.pdb`, field list `0x75D0`;
  its vtable's low 16 bits are `d3dgl.map`'s `??_7D3D11GLFont@@6B@`,
  `0x100e596c`, the DLL being relocated by a multiple of 64 KiB):
  `mCharacterInfo` `+340`, a `PtrArray<CharacterInfo>` whose
  `ArrayBaseMaster::length` is `+4` and `list` `+16`; `mFontHeight`
  `+380`, `mStyle` `+396`, `mPointSize` `+400`;
- each non-null entry a `CharacterInfo` (`0x7241`): `c` (`wchar_t`)
  `+16`, `advance` (`char`) `+18`.

The font fills an entry the first time its character is drawn, so a
packet holds only the characters its game has shown. One line a packet:
`run map packet_frame height point style`, then `code:advance` for every
held entry, the code in decimal. The run and map are read off the path
(`.../<run>/<map>/frame-snapshot.bin`). Packets are the original's
memory: what this prints stays outside the repository.
`client::selection::tests::the_plate_s_advances_are_the_packets_font`
reads the file.
"""

import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
from snapshot import Snapshot  # noqa: E402

FONTS = 0x00ED8160
LOOKUP = 36
VT_FONT_LOW = 0x596C


def font(s):
    lst = s.u32(FONTS + 0x30)
    if s.u32(FONTS + 0x24) <= LOOKUP:
        raise ValueError("fewer lookups than the plate's")
    g = s.u32(s.u32(lst + LOOKUP * 0x20 + 0x1C))
    if s.u32(g) & 0xFFFF != VT_FONT_LOW:
        raise ValueError(f"{g:#x} is not a D3D11GLFont (vtable {s.u32(g):#x})")
    length, cl = s.u32(g + 344), s.u32(g + 356)
    height, = struct.unpack("<i", s.read(g + 380, 4))
    style, point = struct.unpack("<ii", s.read(g + 396, 8))
    held = []
    for p in struct.unpack(f"<{length}I", s.read(cl, 4 * length)):
        if p:
            ci = s.read(p, 24)
            held.append((struct.unpack_from("<H", ci, 16)[0], struct.unpack_from("<b", ci, 18)[0]))
    return height, point, style, sorted(held)


def main(paths):
    for path in sorted(paths):
        s = Snapshot(path)
        parts = os.path.abspath(path).split(os.sep)
        run, map_dir = parts[-3], parts[-2]
        height, point, style, held = font(s)
        chars = " ".join(f"{c}:{a}" for c, a in held)
        print(f"{run} {map_dir} {s.frame} {height} {point} {style} {chars}")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    main(sys.argv[1:])
