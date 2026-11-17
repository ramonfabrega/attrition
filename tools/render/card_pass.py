#!/usr/bin/env python3
"""card_pass.py — one pass of the card probe, draw by draw (item 4335).

    python3 tools/render/card_pass.py <rontrace.log> [--frame 3279] [--draws 121-169]
        [--over X0,Y0,X1,Y1] [--screen 1920x1080] [--verts]

`docs/RENDERER.md` §3.14 is what the records are; `rondata::trace::card` is the
reader the tests use, and this is the same parse in Python, for a question
asked of the card before a test is written. It was a scratch test five times
over (items 4310, 4327, 4329, 4330 and this one), which is why it is a tool.

Per draw of the pass drawn after `--frame` (default 3279, run1567's pass for
run1564's photograph) it prints one row of setters:

- the draw index, the caller (the first return address into the executable)
  and the two stages' files;
- the blend (`src`/`dst`, `RenderState +0x74`/`+0x78`), the alpha test
  (`+0x7c`, `+0x80`), the address modes (`+0xdc`, `+0xe4`; `W` wrap, `C`
  clamp), each stage's `mag_filter`, `min_filter` and `mip_filter` (`+0xec`,
  `+0xf4`, `+0xfc`; `MIPFilter`: `-` none, `P` point, `L` linear, `A`
  anisotropic) and `mipmap_bias` (`+0x104`);
- each stage's colour and alpha op with their arguments, by name;
- the pixel shader's options (`ShaderCompilationOptions`);
- the material's diffuse, the light's diffuse and `gTLAlpha` (`cbuf +0x1bc`);
- the stream released before the draw: its vertex count and stride, and the
  screen box its positions project to through the draw's own `gWorld`,
  `gView` and `gProjection` (the stream's first three floats are the
  position; a stream the probe cut at 64 KiB projects only what it kept).

`--over X0,Y0,X1,Y1` keeps the draws whose stream's box meets that screen
rectangle (and every draw with no stream, marked `?`). `--verts` prints each
kept stream's vertices: position, screen point, and the words after the
position as floats and as a BGRA colour. Nothing it prints enters the
repository.
"""

import argparse
import struct
import sys

K_CARD = 40

OPTIONS = [
    "DEFAULT", "ENABLE_LIGHTING", "ENABLE_FOG", "ENABLE_ALPHA_TEST",
    "ENABLE_TEXTURE_FACTOR", "DISABLE_WORLD_TRANSFORM", "DISABLE_VIEW_TRANSFORM",
    "ENABLE_VERTEX_COLOR_DIFFUSE", "ENABLE_VERTEX_COLOR_SPECULAR",
    "TRANSFORM_SCREEN_SPACE", "USE_UV0", "USE_UV1", "USE_UV2", "POSITION_XYZ",
    "TEX0_USE_UV1", "TEX1_USE_UV1", "HAS_TEX0", "HAS_TEX1", "NORMAL_MAP",
    "SPEC_MAP", "GLOSS_MAP", "TL_FAKE_LIGHT", "DELTA_UV", "ANIMATE_WATER_NORMALS",
    "SPEC_MODEL_TORRANCE", "SCREEN_REFLECTIONS", "NO_HDR", "TEX0_NOT_BOUND",
    "TEX1_NOT_BOUND", "DIFFUSE_AS_SPEC", "POINT_SPRITE", "NUKE_FLASH",
]
# Options every draw of a pass carries, left out of the row.
COMMON = {"DEFAULT", "POSITION_XYZ", "USE_UV0", "USE_UV1", "HAS_TEX0", "HAS_TEX1"}

# `TextureOp` (rise.pdb), the ones the card's passes use; others print as numbers.
TOP = {0: "DISABLE", 1: "SEL1", 2: "SEL2", 3: "MOD", 4: "MOD2X", 5: "MOD4X",
       6: "ADD", 12: "BLENDTA"}
# `TextureArg`; the high bit is the complement.
TA = {0: "CUR", 1: "DIF", 5: "TEX", 6: "TF"}
# `BlendFactor` and `CompareFunc` as the card's passes set them.
BLEND = {1: "ZERO", 2: "ONE", 5: "SA", 6: "ISA"}
ADDR = {0: "W", 2: "C"}
# `MIPFilter` (rise.pdb's enumerators): none, point, linear, anisotropic.
FILT = {0: "-", 1: "P", 2: "L", 3: "A", 4: "F", 5: "G"}


def i32(b, off):
    return struct.unpack_from("<i", b, off)[0]


def f32(b, off):
    return struct.unpack_from("<f", b, off)[0]


def name_of(raw):
    raw = bytes(raw[:128])
    if len(raw) > 1 and raw[1] == 0 and raw[0] != 0:
        s = raw.decode("utf-16-le", "replace")
    else:
        s = raw.decode("latin-1")
    s = s.split("\0")[0]
    return s.replace("/", "\\").rsplit("\\", 1)[-1]


class Draw:
    def __init__(self, w, stream):
        self.pass_ = w[4]
        self.frame = struct.unpack("<i", struct.pack("<I", w[7]))[0]
        self.index = w[3]
        self.state = bytearray(0x170)
        self.cbuf = bytearray(0x250)
        self.options = []
        self.nopt = 0
        self.names = [bytearray(132), bytearray(132)]
        self.files = ["", ""]
        self.frames = [0, 0, 0, 0]
        self.stream = stream


def put(buf, off, words):
    for j, v in enumerate(words):
        at = off + 4 * j
        if at + 4 <= len(buf):
            struct.pack_into("<I", buf, at, v)


def draws(data, frame):
    """Every draw of the passes drawn after `frame`, in order."""
    out, pending, cur = [], None, None

    def finish(d):
        if d is not None:
            d.options = d.options[: d.nopt]
            d.files = [name_of(n) for n in d.names]

    n = len(data) // 32
    for k in range(n):
        w = struct.unpack_from("<8I", data, 32 * k)
        if w[0] != K_CARD:
            continue
        sub = w[1]
        if sub == 0:
            finish(cur)
            cur = Draw(w, pending)
            pending = None
            out.append(cur)
        elif sub == 11:
            size = min(w[3] * w[5], 0x10000)
            pending = {"lit": w[2], "vertices": w[3], "stride": w[5],
                       "bytes": bytearray(size)}
        elif sub == 12:
            if pending is not None:
                put(pending["bytes"], w[2], w[3:7])
        elif cur is not None:
            if sub == 2:
                put(cur.state, w[2], w[3:7])
            elif sub == 3:
                put(cur.cbuf, w[2], w[3:7])
            elif sub == 4:
                cur.nopt = w[2]
            elif sub == 5:
                cur.options.extend(w[3:7])
            elif sub == 6 and w[2] < 2:
                put(cur.names[w[2]], w[3], w[4:7])
            elif sub == 7:
                cur.frames = list(w[2:6])
    finish(cur)
    return [d for d in out if d.frame == frame]


def mat(b, off):
    return [f32(b, off + 4 * i) for i in range(16)]


def mv(m, v):
    return [sum(m[4 * r + c] * v[c] for c in range(4)) for r in range(4)]


def project(d, p, size):
    w, h = size
    world = mat(d.cbuf, 0xE0)
    c = mv(mat(d.cbuf, 0x40), mv(mat(d.cbuf, 0), mv(world, [p[0], p[1], p[2], 1.0])))
    if c[3] == 0:
        return None
    return ((c[0] / c[3] + 1) * w / 2, (1 - c[1] / c[3]) * h / 2)


def stream_points(d, size):
    s = d.stream
    if not s or s["stride"] < 12:
        return []
    b, st = s["bytes"], s["stride"]
    pts = []
    for v in range(min(s["vertices"], len(b) // st)):
        p = struct.unpack_from("<3f", b, v * st)
        q = project(d, p, size)
        if q is not None:
            pts.append((v, p, q))
    return pts


def op(d, base, s):
    o, a1, a2 = (i32(d.state, base + 4 * s), i32(d.state, base + 8 + 4 * s),
                 i32(d.state, base + 16 + 4 * s))

    def arg(a):
        neg = "~" if a < 0 else ""
        return neg + TA.get(a & 0x7FFFFFFF, str(a & 0x7FFFFFFF))

    t = TOP.get(o, str(o))
    return t if o == 0 else f"{t}({arg(a1)},{arg(a2)})"


def row(d, size):
    st = d.state
    blend = f"{BLEND.get(i32(st, 0x74), i32(st, 0x74))}/{BLEND.get(i32(st, 0x78), i32(st, 0x78))}"
    atest = f"{i32(st, 0x7c)}>{i32(st, 0x80)}"
    addr = "".join(ADDR.get(i32(st, o), "?") for o in (0xDC, 0xE4, 0xE0, 0xE8))
    # mag, min, mip per stage (`+0xec`, `+0xf4`, `+0xfc`), then `mipmap_bias`.
    filt = "/".join("".join(FILT.get(i32(st, o + 4 * s), "?") for o in (0xEC, 0xF4, 0xFC))
                    for s in (0, 1))
    bias = "/".join(f"{f32(st, 0x104 + 4 * s):g}" for s in (0, 1))
    ops = " ".join(f"s{s}:{op(d, 0xAC, s)}|{op(d, 0xC4, s)}" for s in (0, 1))
    opts = [OPTIONS[o] if o < len(OPTIONS) else str(o) for o in d.options]
    opts = ",".join(o for o in opts if o not in COMMON)
    mdiff = f32(d.cbuf, 0x170)
    ldiff = f32(d.cbuf, 0x210)
    tla = f32(d.cbuf, 0x1BC)
    caller = f"{d.frames[0]:08x}" if d.frames[0] else "-"
    box = "?"
    if d.stream:
        pts = stream_points(d, size)
        if pts:
            xs = [q[0] for _, _, q in pts]
            ys = [q[1] for _, _, q in pts]
            box = f"{min(xs):.0f},{min(ys):.0f}..{max(xs):.0f},{max(ys):.0f}"
        box = f"{d.stream['vertices']}v/{d.stream['stride']} {box}"
    return (f"{d.index:4d} {caller} {d.files[0] or '-'} | {d.files[1] or '-'}  "
            f"blend {blend} atest {atest} addr {addr} filt {filt} bias {bias}  {ops}  [{opts}]  "
            f"mat {mdiff:.2f} light {ldiff:.2f} tl {tla:.2f}  {box}")


def meets(d, rect, size):
    if not d.stream:
        return True
    pts = stream_points(d, size)
    if not pts:
        return True
    xs = [q[0] for _, _, q in pts]
    ys = [q[1] for _, _, q in pts]
    return not (max(xs) < rect[0] or min(xs) > rect[2] or max(ys) < rect[1] or min(ys) > rect[3])


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("trace")
    ap.add_argument("--frame", type=int, default=3279)
    ap.add_argument("--draws", default=None, help="LO-HI, inclusive")
    ap.add_argument("--over", default=None, help="X0,Y0,X1,Y1 on the screen")
    ap.add_argument("--screen", default="1920x1080")
    ap.add_argument("--verts", action="store_true")
    a = ap.parse_args()
    size = tuple(int(x) for x in a.screen.split("x"))
    with open(a.trace, "rb") as f:
        data = f.read()
    pass_ = draws(data, a.frame)
    if not pass_:
        sys.exit(f"no card draw after frame {a.frame} in {a.trace}")
    lo, hi = (0, 1 << 30)
    if a.draws:
        lo, hi = (int(x) for x in a.draws.split("-"))
    rect = [float(x) for x in a.over.split(",")] if a.over else None
    passes = sorted({d.pass_ for d in pass_})
    print(f"# {a.trace}: {len(pass_)} draws after frame {a.frame}, passes {passes}")
    for d in pass_:
        if not lo <= d.index <= hi:
            continue
        if rect and not meets(d, rect, size):
            continue
        print(row(d, size))
        if a.verts and d.stream:
            b, st = d.stream["bytes"], d.stream["stride"]
            for v, p, q in stream_points(d, size):
                rest = b[v * st + 12:(v + 1) * st]
                fl = struct.unpack_from(f"<{len(rest) // 4}f", rest)
                col = [f"{x:08x}" for x in struct.unpack_from(f"<{len(rest) // 4}I", rest)]
                print(f"     v{v} ({p[0]:.1f},{p[1]:.1f},{p[2]:.1f}) -> ({q[0]:.1f},{q[1]:.1f})"
                      f"  {' '.join(f'{x:.4g}' for x in fl)}  [{' '.join(col)}]")


if __name__ == "__main__":
    main()
