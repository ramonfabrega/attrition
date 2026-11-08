#!/usr/bin/env python3
"""camera_oracle.py — the original's camera, built and projected under unicorn.

    uv run --locked --project tools/emu python tools/render/camera_oracle.py \
        <install>/riseofnations.exe sweep [SEED] [STATES] [POINTS]

The renderer's camera check (item 4018, `docs/RENDERER.md` §3.4). For each
synthesized camera state it lays out a `Camera` (fields at the offsets the
PDB's type record names), enters `Camera::update_transforms@008430a0` — which
builds the view in `BaseCamera::update_transforms@00537cc0` and the
projection in the graphics DLL's `set_projection` / `set_projection_parallel`
(`d3dgl.dll`, mapped beside the executable at its preferred base) — and then
calls `MiscAccess::world_to_screen@00583810` on a set of world points.

Nothing is answered by this script that the original computes, except four
imports the bottle would resolve and unicorn cannot: `memset` (both images),
the CRT's double `cos`/`sin` behind `cosf`/`sinf` (the half-degree tables'
fill) and the DLL's `__libm_sse2_tan_precise` — each answered with Python's
double-precision libm, the original's own float conversions around them
untouched. Three callees outside the projection are stubbed: the DLL's
`D3D11Context::set_projection` (a device upload), `GraphSys::set_camera_transform`
(slot 0x194, the same) and `Camera::compute_clip_planes@00842950` (culling,
which writes no matrix `world_to_screen` reads).

Rows, one per line, every float as its 32-bit pattern in hex:

    state <i> <zoom> <scale> <tilt> <dir> <lx> <ly> <ax> <ay> <az> <angle> <near> <far> <ratio> <parallel> <pdist> <xs> <ys> <natural> <conoff>
    mat <i> <16 floats of the camera's to_screen_matrix>
    pt <i> <x> <y> <z> <w> <h> -> <sx> <sy> <off>

`w h` of `0 0` takes the screen size from `graphsys` as the original does.
The table, like every emulator table, is derived from the user's own files
and stays outside the repository.
"""
import math
import os
import random
import struct
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "emu"))
from callfn import Image  # noqa: E402
from hooks import Harness  # noqa: E402
from unicorn.x86_const import UC_X86_REG_XMM0  # noqa: E402

UPDATE_TRANSFORMS = 0x008430A0
WORLD_TO_SCREEN = 0x00583810
COMPUTE_CLIP_PLANES = 0x00842950
COSF, SINF = 0x0041E6B0, 0x0041E6D0
CRT_COS, CRT_SIN, EXE_MEMSET = 0x00A5719E, 0x00A571AA, 0x0055E0B2
DLL_MEMSET, DLL_TAN = 0x100AB93C, 0x100BA4F0
DLL_SET_PROJECTION, DLL_SET_PROJECTION_PARALLEL = 0x10020700, 0x10020830
DLL_CONTEXT_SET_PROJECTION = 0x1001A970

# Globals (the PDB's S_GDATA32 addresses, read off the listing).
RENDER_STATES_COUNT, RENDER_STATES_LIST = 0x00C11D30, 0x00C11D3C
MISC_CAMERA, MISC_CONSOLE, MISC_OPTIONS, CAMERA_SCALE = 0x00C06200, 0x00C06210, 0x00C06204, 0x00C0625C
ACTIVE_CAMERA, GRAPHSYS = 0x00E335A8, 0x00E335A0
COS_FIRST, COS_DATA = 0x00C8D7F0, 0x00C8D2CC
SIN_FIRST, SIN_DATA = 0x00C8D7F4, 0x00C8D2B0
DEG = struct.unpack("<f", bytes.fromhex("35fa8e3c"))[0]  # 0x3c8efa35, the table's own constant


def f32(v):
    return struct.unpack("<f", struct.pack("<f", v))[0]


def fbits(v):
    return struct.unpack("<I", struct.pack("<f", v))[0]


class Oracle:
    def __init__(self, exe):
        dll = os.path.join(os.path.dirname(exe), "d3dgl.dll")
        callees = {
            COMPUTE_CLIP_PLANES: ("compute_clip_planes", 0),
            CRT_COS: ("crt_cos", 0),
            CRT_SIN: ("crt_sin", 0),
            EXE_MEMSET: ("exe_memset", 3, False, True),
            DLL_MEMSET: ("dll_memset", 3, False, True),
            DLL_TAN: ("dll_tan", 0),
            DLL_CONTEXT_SET_PROJECTION: ("context_set_projection", 1),
        }
        self.h = h = Harness(exe, callees)
        Image(dll).map_into(h.m.uc)
        uc = h.m.uc

        def libm(fn):
            def answer(ecx, args):
                x = struct.unpack("<d", struct.pack("<Q", uc.reg_read(UC_X86_REG_XMM0) & (2**64 - 1)))[0]
                uc.reg_write(UC_X86_REG_XMM0, struct.unpack("<Q", struct.pack("<d", fn(x)))[0])
                return 0
            return answer

        def memset(ecx, args):
            dst, val, n = args
            uc.mem_write(dst & 0xFFFF_FFFF, bytes([val & 0xFF]) * n)
            return dst

        h.answers.update(crt_cos=libm(math.cos), crt_sin=libm(math.sin), dll_tan=libm(math.tan),
                         exe_memset=memset, dll_memset=memset)
        # The half-degree tables: 360 entries each, filled the way
        # `fast_half_degree_to_cosine@00a29010` fills them on first use —
        # `cosf((float)i * 0.5f * DEG)` — through the original's `cosf`/`sinf`.
        for first, data, fn in ((COS_FIRST, COS_DATA, COSF), (SIN_FIRST, SIN_DATA, SINF)):
            table = h.alloc(360 * 4)
            for i in range(360):
                arg = f32(f32(f32(i) * 0.5) * DEG)
                bits = h.call(fn, xmm={0: fbits(arg)}, ret="xmm0") & 0xFFFF_FFFF
                h.w32(table + 4 * i, bits)
            h.w32(data, table)
            h.w32(first, 0)
        # render_states: a count above zero, and the two flags the camera sets.
        flags = h.alloc(0x100)
        holder = h.alloc(0x100)
        h.w32(holder + 0x68, flags)
        h.w32(RENDER_STATES_COUNT, 1)
        h.w32(RENDER_STATES_LIST, holder)
        # The console (no drag) and the options (`selecting_spot`).
        self.console = h.alloc(0x400)
        h.w32(MISC_CONSOLE, self.console)
        h.w32(MISC_OPTIONS, h.alloc(0x100))
        # graphsys: a D3D11GLSys-sized object whose vtable names the three slots.
        self.gs = h.alloc(0x900)
        vt = h.vtable({0x194: ("set_camera_transform", 1, 0)})
        h.w32(vt + 0x198, DLL_SET_PROJECTION)
        h.w32(vt + 0x19C, DLL_SET_PROJECTION_PARALLEL)
        h.w32(self.gs, vt)
        h.w32(GRAPHSYS, self.gs)
        self.cam = h.alloc(0x370)
        self.out = h.alloc(16)

    def build(self, s):
        h, cam, gs = self.h, self.cam, self.gs
        h.m.uc.mem_write(cam, bytes(0x370))
        h.w32(cam, h.alloc(0x40))  # a vtable pointer nobody calls
        for off, key in ((0x2F0, "zoom"), (0x2F4, "tilt"), (0x2F8, "dir"), (0x304, "lx"), (0x308, "ly"),
                         (0x30C, "ax"), (0x310, "ay"), (0x314, "az"), (0x12C, "angle"), (0x130, "near"),
                         (0x134, "far"), (0x13C, "ratio"), (0x284, "pdist")):
            h.w32(cam + off, fbits(s[key]))
        h.w32(cam + 0x280, s["parallel"])
        h.w32(MISC_CAMERA, cam)
        h.w32(ACTIVE_CAMERA, cam)
        h.w32(CAMERA_SCALE, fbits(s["scale"]))
        h.w32(gs + 0x360, s["xs"])
        h.w32(gs + 0x364, s["ys"])
        h.w32(gs + 0x16C, fbits(s["natural"]))
        h.w16(self.console + 0x12A, s["conoff"])
        h.call(UPDATE_TRANSFORMS, ecx=cam, max_insns=400_000)
        return [struct.unpack("<f", h.m.uc.mem_read(cam + 0x240 + 4 * k, 4))[0] for k in range(16)]

    def project(self, x, y, z, w, hgt):
        h = self.h
        off = h.call(WORLD_TO_SCREEN, (x, y, z, w, hgt), ecx=self.out, edx=self.out + 4)
        return h.r32(self.out), h.r32(self.out + 4), off


def states(rng, n):
    zooms = [173823.375, 139058.703, 55623.48, 41717.609, 27811.74, 20858.805, 13905.87]
    screens = [(1920, 1080, 1.000925898551941), (1024, 768, 1.0), (1280, 720, 1.0), (1600, 900, 0.9975)]
    out = []
    for i in range(n):
        zoom = rng.choice(zooms) if i % 3 else rng.uniform(5_000, 200_000)
        tilt = rng.choice([60.0, 60.0, 45.0, 30.0]) if i % 4 else rng.uniform(-260, 120)
        dirs = [45.0, 0.0, 90.0, -45.0, 135.0, 359.5, 721.25, -1.0, -181.0]
        d = rng.choice(dirs) if i % 5 else rng.uniform(-720, 720)
        parallel = 0 if i % 6 == 5 else 1
        near = rng.uniform(50, zoom * 0.6)
        xs, ys, natural = rng.choice(screens)
        out.append(dict(
            zoom=f32(zoom), scale=f32(1.0 if i % 7 else rng.uniform(0.5, 2.0)), tilt=f32(tilt), dir=f32(d),
            lx=f32(rng.uniform(0, 40_000)), ly=f32(rng.uniform(0, 40_000)),
            ax=f32(0.0 if i % 2 else rng.uniform(-800, 800)), ay=f32(0.0 if i % 2 else rng.uniform(-800, 800)),
            az=f32(0.0 if i % 3 else rng.uniform(-400, 2_000)),
            angle=f32(45.0 if i % 4 else rng.uniform(15, 80)), near=f32(near),
            far=f32(near + rng.uniform(2_000, 80_000)), ratio=f32(ys / xs if i % 5 else rng.uniform(0.5, 0.8)),
            parallel=parallel, pdist=f32(zoom / 2 if i % 3 else rng.uniform(2_000, 80_000)),
            xs=xs, ys=ys, natural=f32(natural), conoff=0 if i % 4 else rng.randint(-60, 60)))
    return out


def sweep(o, seed, nstates, npoints):
    rng = random.Random(seed)
    for i, s in enumerate(states(rng, nstates)):
        m = o.build(s)
        keys = ("zoom", "scale", "tilt", "dir", "lx", "ly", "ax", "ay", "az", "angle", "near", "far", "ratio")
        print(f"state {i} " + " ".join(f"{fbits(s[k]):08x}" for k in keys)
              + f" {s['parallel']} {fbits(s['pdist']):08x} {s['xs']} {s['ys']} {fbits(s['natural']):08x} {s['conoff']}")
        print(f"mat {i} " + " ".join(f"{fbits(v):08x}" for v in m))
        made = 0
        cx, cy = int(s["lx"] + s["ax"]), int(s["ly"] + s["ay"])
        tries = 0
        while made < npoints and tries < npoints * 50:
            tries += 1
            span = rng.choice([500, 4_000, 15_000, 40_000])
            x, y = cx + rng.randint(-span, span), cy + rng.randint(-span, span)
            z = rng.choice([0, 0, rng.randint(-300, 600), rng.randint(0, 6_000)])
            if made == 0:
                x, y, z = cx, cy, 0
            # Inputs only: a perspective point behind or at the eye has no
            # pixel to agree on. The original's own `w` row picks the domain.
            wv = m[3] * x + m[7] * y + m[11] * z + m[15]
            if not s["parallel"] and wv < 1.0:
                continue
            w, hgt = (0, 0) if made % 2 else (s["xs"], s["ys"])
            sx, sy, off = o.project(x, y, z, w, hgt)
            print(f"pt {i} {x} {y} {z} {w} {hgt} -> {sx} {sy} {off}")
            made += 1


def main(argv):
    if len(argv) < 3 or argv[2] != "sweep":
        print(__doc__)
        return 2
    seed = int(argv[3]) if len(argv) > 3 else 4018
    nstates = int(argv[4]) if len(argv) > 4 else 240
    npoints = int(argv[5]) if len(argv) > 5 else 40
    sweep(Oracle(argv[1]), seed, nstates, npoints)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
