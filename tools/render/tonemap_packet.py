#!/usr/bin/env python3
"""tonemap_packet.py — the Extended Edition's tone mapper and anti-alias, as a packet holds them (item 4168).

    python3 tools/render/tonemap_packet.py <frame-snapshot.bin> > <run>-tonemap.txt

`docs/RENDERER.md` §3.14. No function is called: this reads what the
renderer DLL left on the heap at the packet's frame boundary. The packet
keeps private memory and the executable's image, not `d3dgl.dll`'s, so
the objects are reached from the executable's `graphsys`
(`?graphsys@@3PAVGraphSys@@A`, `0x00e335a0` in `sbl/rise_z.map`), which
points at a heap `D3D11GLSys`. Layouts are `sbl/d3dgl.pdb`'s type
records (`llvm-pdbutil dump --types`):

- `D3D11GLSys` (field list `0x71DE`): `mToneMapper` +4976,
  `mAntiAlias` +4980;
- `GSToneMapper` (`0x4B9B`): `mDirty` +4 (bool), `mExposure` +8,
  `mExposureAdjustment` +12, `mWhitePoint` +16, `mThreshold` +20,
  `mMinLuminance` +24, `mSaturation` +28, `mBloomStrength` +32,
  `mBloomTightness` +36, `mTint` +40 (a `Color`, bytes r g b a);
- `D3D11ToneMapper` (`0x7CC4`): `mData` +80, a `ToneMapData` (`0x7CBC`,
  the constant buffer `update_params` last wrote): `gTint` float4 +0,
  `gExposure` +16, `gWhitePoint` +20, `gThreshold` +24, `gMinLuminance`
  +28, `gSaturation` +32, `gTightness` +36, `gStrength` +40;
- `D3D11AntiAlias` (`0x5773`): `mAAEnabled` +36 (int).

Each object's type is checked by its vtable: both must lie at their own
`d3dgl.map` addresses (`??_7D3D11ToneMapper@@6B@` `0x100e4fc4`,
`??_7D3D11AntiAlias@@6B@` `0x100e44ac`) under one relocation, a multiple
of 64 KiB. Floats print as their IEEE bits (hex) and as a decimal; one
line, `key value` pairs. Nothing it prints enters the repository.
"""

import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
from snapshot import Snapshot  # noqa: E402

GRAPHSYS = 0x00E335A0
VT_TONEMAPPER = 0x100E4FC4
VT_ANTIALIAS = 0x100E44AC


def main(path):
    s = Snapshot(path)
    gs = s.u32(GRAPHSYS)
    tm, aa = s.u32(gs + 4976), s.u32(gs + 4980)
    b = s.read(tm, 208)
    a = s.read(aa, 40)
    reloc = (struct.unpack_from("<I", b, 0)[0] - VT_TONEMAPPER) & 0xFFFFFFFF
    reloc_aa = (struct.unpack_from("<I", a, 0)[0] - VT_ANTIALIAS) & 0xFFFFFFFF
    if reloc != reloc_aa or reloc & 0xFFFF:
        sys.exit(f"{path}: the vtables do not relocate together ({reloc:#x}, {reloc_aa:#x})")
    out = [f"frame {s.frame}", f"reloc {reloc:#x}", f"dirty {b[4]}"]

    def f(name, buf, at):
        bits = struct.unpack_from("<I", buf, at)[0]
        out.append(f"{name} {bits:#010x}/{struct.unpack_from('<f', buf, at)[0]!r}")

    for name, at in [("exposure", 8), ("exposure_adjustment", 12), ("white_point", 16),
                     ("threshold", 20), ("min_luminance", 24), ("saturation", 28),
                     ("bloom_strength", 32), ("bloom_tightness", 36)]:
        f(name, b, at)
    out.append("tint " + ",".join(str(x) for x in b[40:44]))
    for k in range(4):
        f(f"cb_tint{k}", b, 80 + 4 * k)
    for name, at in [("cb_exposure", 16), ("cb_white_point", 20), ("cb_threshold", 24),
                     ("cb_min_luminance", 28), ("cb_saturation", 32), ("cb_tightness", 36),
                     ("cb_strength", 40)]:
        f(name, b, 80 + at)
    out.append(f"aa_enabled {struct.unpack_from('<i', a, 36)[0]}")
    print(" ".join(out))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
