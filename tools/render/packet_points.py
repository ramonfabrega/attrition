#!/usr/bin/env python3
"""packet_points.py — the original's screen points on a packet's own camera.

    uv run --locked --project tools/emu python tools/render/packet_points.py \
        <install>/riseofnations.exe <frame-snapshot.bin> <gamelog> <frame>

The packet half of the renderer's camera check (item 4018,
`docs/RENDERER.md` §3.4). A `RON_STATE_FRAME` packet holds the game's own
`Camera` (`MiscAccess::camera`, which is `BaseCamera::active_camera`),
`graphsys` and the console as the frame left them. This maps the executable,
enters `MiscAccess::world_to_screen@00583810` for every `SUBOBJECT`'s
`x_internal y_internal z_internal` on the dump's `BEGIN FRAME <frame>`
block, and maps a packet page only when the call first touches it — the
projection reads a handful of objects, not the 800 MB.

Rows, in `camera_oracle.py`'s format with the one state numbered 0 (floats
as their 32-bit patterns):

    state 0 <zoom> <scale> <tilt> <dir> <lx> <ly> <ax> <ay> <az> <angle> <near> <far> <ratio> <parallel> <pdist> <xs> <ys> <natural> <conoff>
    mat 0 <16 floats of the packet's to_screen_matrix>
    pt 0 <x> <y> <z> <w> <h> -> <sx> <sy> <off>

Each point is projected twice: with the packet's screen size from
`graphsys` (`0 0`) and with an explicit 1024×768. Nothing it prints enters
the repository.
"""
import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "emu"))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
from callfn import Image, Machine  # noqa: E402
from snapshot import Snapshot  # noqa: E402

WORLD_TO_SCREEN = 0x00583810
MISC_CAMERA, MISC_CONSOLE, CAMERA_SCALE = 0x00C06200, 0x00C06210, 0x00C0625C
ACTIVE_CAMERA, GRAPHSYS = 0x00E335A8, 0x00E335A0
SCRATCH = 0x7E00_0000
PAGE = 0x1000


class PacketMachine(Machine):
    """`callfn.Machine` whose unmapped reads are answered from the packet."""

    def __init__(self, image, snap):
        self.snap = snap
        super().__init__(image)
        self.uc.mem_map(SCRATCH, PAGE)
        # The image's own data is the packet's at that frame: every range
        # inside the image is written over the file's initial bytes.
        lo, hi = image.base, image.base + image.size
        self.snap.load(self.uc.mem_write, [r for r in snap.ranges if lo <= r[0] and r[0] + r[1] <= hi])

    def _on_unmapped(self, uc, access, addr, size, value, _):
        page = addr & ~(PAGE - 1)
        for base, n, off in self.snap.ranges:
            if base <= page < base + n:
                uc.mem_map(page, PAGE)
                start = max(page, base)
                end = min(page + PAGE, base + n)
                uc.mem_write(start, self.snap.read(start, end - start))
                return True
        return super()._on_unmapped(uc, access, addr, size, value, _)


def positions(gamelog, frame):
    want, found, out, x = f"BEGIN FRAME {frame}", False, [], {}
    with open(gamelog, errors="replace") as f:
        for line in f:
            s = line.strip()
            if not found:
                found = s == want
                continue
            if s.startswith("BEGIN FRAME"):
                break
            for k in ("x_internal", "y_internal", "z_internal"):
                if s.startswith(k + " "):
                    x[k] = int(s.split()[1])
            if len(x) == 3:
                out.append((x["x_internal"], x["y_internal"], x["z_internal"]))
                x = {}
    return out


def main(argv):
    if len(argv) != 5:
        print(__doc__)
        return 2
    exe, packet, gamelog, frame = argv[1:5]
    snap = Snapshot(packet)
    m = PacketMachine(Image(exe), snap)
    u32 = lambda a: struct.unpack("<I", snap.read(a, 4))[0]
    cam, gs, con = u32(MISC_CAMERA), u32(GRAPHSYS), u32(MISC_CONSOLE)
    assert u32(ACTIVE_CAMERA) == cam, "the active camera is the game's camera"
    raw = lambda a: f"{u32(a):08x}"
    fields = [raw(cam + o) for o in (0x2F0,)] + [raw(CAMERA_SCALE)] + [raw(cam + o) for o in (
        0x2F4, 0x2F8, 0x304, 0x308, 0x30C, 0x310, 0x314, 0x12C, 0x130, 0x134, 0x13C)]
    conoff = struct.unpack("<h", snap.read(con + 0x12A, 2))[0]
    print("state 0 " + " ".join(fields) + f" {u32(cam + 0x280)} {raw(cam + 0x284)} "
          f"{u32(gs + 0x360)} {u32(gs + 0x364)} {raw(gs + 0x16C)} {conoff}")
    print("mat 0 " + " ".join(raw(cam + 0x240 + 4 * k) for k in range(16)))
    pts = positions(gamelog, frame)
    if not pts:
        raise SystemExit(f"no SUBOBJECT positions on {gamelog}'s frame {frame}")
    for x, y, z in pts:
        for w, h in ((0, 0), (1024, 768)):
            off = m.call(WORLD_TO_SCREEN, (x, y, z, w, h), ecx=SCRATCH, edx=SCRATCH + 4)
            sx, sy = struct.unpack("<ii", m.uc.mem_read(SCRATCH, 8))
            print(f"pt 0 {x} {y} {z} {w} {h} -> {sx} {sy} {off}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
