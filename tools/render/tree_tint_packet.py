#!/usr/bin/env python3
"""tree_tint_packet.py — the tint the trees' textures load under, as a packet holds it (item 4221).

    python3 tools/render/tree_tint_packet.py <frame-snapshot.bin> > <run>-tree-tint.txt

`docs/RENDERER.md` §3.7, "The trees' own light". `GraphicPieces::init_tree_data@008eeb30`
sets each tree rdata's `TintColor` param to `GraphicPieces::tint_color`;
`TreeTransformMethod::load@00921aa0` hands it to `ImageIO::set_tint` before each of
the seven `Texture::init`s. The global is `?graphic_pieces@@3VGraphicPieces@@A`,
`0x00e896d8` in `sbl/rise_z.map`; `tint_color` is a `Color` at +0xf54
(`rise.pdb`'s `GraphicPieces`), its first dword the bytes red, green, blue, alpha.
One line, `key value` pairs. Nothing it prints enters the repository.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
from snapshot import Snapshot  # noqa: E402

GRAPHIC_PIECES = 0x00E896D8
TINT_COLOR = 0xF54


def main(path):
    s = Snapshot(path)
    b = s.read(GRAPHIC_PIECES + TINT_COLOR, 4)
    print(f"frame {s.frame} tree_tint " + ",".join(str(x) for x in b))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
