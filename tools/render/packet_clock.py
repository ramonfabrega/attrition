#!/usr/bin/env python3
"""packet_clock.py — the game's clock as each packet holds it (item 4203).

    python3 tools/render/packet_clock.py <frame-snapshot.bin>... > packet-ticks.txt

`docs/HUD.md` §4 and §5. No function is called: this reads the `Game`
object the packet's `GameAccessConst::gamec` (`0x00c061e8`) points at,
and prints two of its fields by the PDB's type record:

- `Game::frame` (`+0x550`), the frame count;
- `Game::tick` (`+0x560`), the clock in seconds, which
  `IFaceMainBase::draw_button_row_area@0080f7b0` hands `say_timer` for the
  bar's `Normal: 00:03:38`.

One line a packet, `run map packet_frame frame tick`, the run and map
read off the path (`~/ron-golden/<run>/map-<n>/frame-snapshot.bin`) and
`packet_frame` the packet's own header. Packets are the original's
memory: what this prints stays outside the repository.
`client::hud::tests::the_clock_is_the_original_s_tick` reads the file.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
from snapshot import Snapshot  # noqa: E402

GAMEC = 0x00C061E8
FRAME = 0x550
TICK = 0x560


def main(paths):
    for path in sorted(paths):
        s = Snapshot(path)
        game = s.u32(GAMEC)
        parts = os.path.abspath(path).split(os.sep)
        run, map_dir = parts[-3], parts[-2]
        print(f"{run} {map_dir} {s.frame} {s.u32(game + FRAME)} {s.u32(game + TICK)}")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    main(sys.argv[1:])
