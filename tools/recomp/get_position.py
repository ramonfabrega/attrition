#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4", "capstone==5.0.3"]
# ///
"""get_position.py — the original's `GraphicPieces::get_position` on a packet.

    uv run tools/recomp/get_position.py <exe> <frame-snapshot.bin> entries <piece>
    uv run tools/recomp/get_position.py <exe> <frame-snapshot.bin> sweep <piece> <node> <anim> <time>
    uv run tools/recomp/get_position.py <exe> <frame-snapshot.bin> pivot <piece> <node> <anim> <time> <restrictions>

A release node's world offset is `GraphicPieces::get_position@0090b750`'s
answer, and that function is packed SSE, so it does not lift (`docs/EMULATOR.md`
§8). This runs it under unicorn on a `RON_STATE_FRAME` packet, which is the
only place the piece's `AttachPos` entries exist: `GraphicEvents::
init_unit_events@008e2520` fills them when a unit of the type is first built,
so the packet must come from a game that fields the type.

- `entries` reads the piece's `AttachPos` rows straight off the packet —
  `positions` (`GraphicPieces +0x6a8`, its data at `+0x10` as every
  array's here), 0x1c bytes a row: `+0` time (a
  ushort, the event's frame), `+2` anim, `+3` node, `+4` the point's `x, y,
  z` and `+0x10` its direction, as floats — from `pos_start[piece]`
  (`+0x6c4`) for `pos_num[piece]` (`+0x6e0`) rows, with the piece's
  `RData +0x88` scale (`data_pieces`, `+0x718`). Each float is printed as
  its bits and its value.
- `sweep` calls the non-pivot arm (`param_6` NULL) at every whole degree
  `0..=360` and prints the truncated `(x, y, z)` and the floats. One
  `sim::launch::Bay` per key is read off it (item 1194's rows).
- `pivot` calls the pivot arm, `param_6` a four-entry `pivot_angles` table
  and `param_7` the piece's restriction count, at every whole facing degree
  `0..=360` and every turret step `fast_angle_to_degrees(i << 24)` read off
  the packet's own table. `sim::pivot::Release`'s `float_lands` are the
  cells where the exact integer form disagrees with this (item 602).

Item 1208 filed the shape: reached three times from one scratch script
(items 602/603, 1194, 1257); this is that script with its paths and piece
taken as arguments. Nothing it prints enters git — it is the user's
executable's state, read the way the decompiler is read.
"""
import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "emu"))
sys.path.insert(0, HERE)

import callfn  # noqa: E402
from difftest import EmuMachine, STUBS, load_into_unicorn  # noqa: E402
from image import Image  # noqa: E402
from snapshot import Snapshot  # noqa: E402
import step4  # noqa: E402
from unicorn import UC_HOOK_CODE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_EAX, UC_X86_REG_ECX, UC_X86_REG_ESP  # noqa: E402

GET_POSITION = 0x90B750
GRAPHIC_PIECES = 0xC06214  # the singleton's pointer


def machine(exe, packet):
    snap = Snapshot(packet)
    pe = Image(exe)
    image = callfn.Image(exe)
    m = EmuMachine(image, pe.imports())
    load_into_unicorn(m, image, snap)
    m.patch_iat()
    uc = m.uc

    class Stubs(step4.Stubs):
        def handle(self, name):
            if name == "memset":
                d, c, n = self.arg(0), self.arg(1), self.arg(2)
                self.uc.mem_write(d, bytes([c & 0xFF]) * n)
                self.ret(0, d)
                return True
            if name in ("memcpy", "memmove"):
                d, s, n = self.arg(0), self.arg(1), self.arg(2)
                self.uc.mem_write(d, bytes(self.uc.mem_read(s, n)))
                self.ret(0, d)
                return True
            return super().handle(name)

    stubs = Stubs(uc)
    (teb, _, _), _ = step4.find_teb(snap, snap.thread)
    step4.set_fs(uc, teb)
    uc.hook_del(m.stub_hook)

    def on_stub(uc, a, _s, _):
        n = m.stubs.get(a, "?")
        if not stubs.handle(n):
            print("unstubbed", n)
            uc.emu_stop()
            raise SystemExit(1)

    uc.hook_add(UC_HOOK_CODE, on_stub, begin=STUBS, end=STUBS + 0xFFF)
    uc.mem_map(step4.FRAME_STACK, step4.FRAME_STACK_SIZE)
    return snap, uc


def f2u(f):
    return struct.unpack("<I", struct.pack("<f", f))[0]


class Oracle:
    def __init__(self, exe, packet):
        self.snap, self.uc = machine(exe, packet)
        self.gp = self.snap.u32(GRAPHIC_PIECES)
        st = step4.FRAME_STACK
        self.out, self.dir, self.pivots = st + 0x1000, st + 0x1100, st + 0x1200

    def call(self, args):
        uc = self.uc
        esp = step4.FRAME_STACK + step4.FRAME_STACK_SIZE - 0x200
        words = [callfn.SENTINEL] + list(args)
        uc.mem_write(esp, b"".join(struct.pack("<I", w & 0xFFFFFFFF) for w in words))
        uc.reg_write(UC_X86_REG_ESP, esp)
        uc.reg_write(UC_X86_REG_ECX, self.gp)
        uc.emu_start(GET_POSITION, callfn.SENTINEL, count=5_000_000)
        return uc.reg_read(UC_X86_REG_EAX)

    def get(self, piece, node, anim, time, deg, pivots=None, restrictions=0):
        uc = self.uc
        uc.mem_write(self.out, b"\0" * 12)
        uc.mem_write(self.dir, b"\0" * 12)
        p6 = 0
        if pivots is not None:
            uc.mem_write(self.pivots, struct.pack("<4f", *pivots))
            p6 = self.pivots
        r = self.call([piece, node, anim, time, f2u(float(deg)), p6, restrictions, self.out, self.dir])
        return r, struct.unpack("<3f", bytes(uc.mem_read(self.out, 12)))

    def entries(self, piece):
        s, gp = self.snap, self.gp
        # Each array's data pointer is its `+0x10` (the listing,
        # `90b776`..`90b7c4`: `+0x6f0`, `+0x6d4`, `+0x728`, `+0x6b8`).
        positions = s.u32(gp + 0x6A8 + 0x10)
        start = struct.unpack("<H", s.read(s.u32(gp + 0x6C4 + 0x10) + 2 * piece, 2))[0]
        num = struct.unpack("<H", s.read(s.u32(gp + 0x6E0 + 0x10) + 2 * piece, 2))[0]
        rdata = s.u32(s.u32(gp + 0x718 + 0x10) + 4 * piece)
        scale = struct.unpack("<f", s.read(rdata + 0x88, 4))[0]
        flags = s.read(rdata + 0x92, 1)[0]
        rows = []
        for i in range(start, start + num):
            raw = s.read(positions + 0x1C * i, 0x1C)
            time, anim, node = struct.unpack("<HbB", raw[:4])
            pt = struct.unpack("<3f", raw[4:16])
            dr = struct.unpack("<3f", raw[16:28])
            bits = struct.unpack("<3I", raw[4:16])
            rows.append((node, anim, time, pt, bits, dr))
        return start, num, scale, flags, rows


def main():
    if len(sys.argv) < 5:
        sys.exit(__doc__)
    exe, packet, mode = sys.argv[1], sys.argv[2], sys.argv[3]
    args = [int(a) for a in sys.argv[4:]]
    o = Oracle(exe, packet)
    if mode == "entries":
        (piece,) = args
        start, num, scale, flags, rows = o.entries(piece)
        print(f"piece {piece}: pos_start {start} pos_num {num} RData+0x88 {scale!r} +0x92 {flags:#x}")
        for node, anim, time, pt, bits, dr in rows:
            print(
                f"  node {node} anim {anim} time {time}: "
                f"{' '.join(f'{b:08x}' for b in bits)}  ({pt[0]!r}, {pt[1]!r}, {pt[2]!r})"
                f"  dir ({dr[0]!r}, {dr[1]!r}, {dr[2]!r})"
            )
    elif mode == "sweep":
        piece, node, anim, time = args
        for deg in range(0, 361):
            r, v = o.get(piece, node, anim, time, deg)
            print(deg, r, int(v[0]), int(v[1]), int(v[2]), repr(v[0]), repr(v[1]), repr(v[2]))
    elif mode == "pivot":
        piece, node, anim, time, restrictions = args
        k = node & 3
        for t in range(256):
            step = float(angle_to_degrees((t << 24) & 0xFFFFFFFF))
            pivots = [0.0] * 4
            pivots[k] = step
            for deg in range(0, 361):
                r, v = o.get(piece, node, anim, time, deg, pivots, restrictions)
                print(deg, t, int(step), int(v[0]), int(v[1]), int(v[2]))
    else:
        sys.exit(__doc__)


def angle_to_degrees(a):
    """`angle_to_degrees`: the engine's 32-bit angle to a whole degree
    `0..=360`, step for step as `sim::movement::angle_to_degrees` has it.
    A turret step is `fast_angle_to_degrees@00a28f70`'s table entry, this
    of the angle's top byte (item 602 checked it against run147's packet)."""
    a &= 0xFFFFFFFF
    quarter = a >> 30
    r = a - quarter * 0x4000_0000
    eighth = r >> 29
    r -= eighth * 0x2000_0000
    d30, r = divmod(r, 0x1555_5555)
    d15, r = divmod(r, 0x0AAA_AAAA)
    d5, r = divmod(r, 0x038E_38E3)
    return quarter * 90 + eighth * 45 + d30 * 30 + d15 * 15 + d5 * 5 + (r + 0x005B_05B0) // 0x00B6_0B60


if __name__ == "__main__":
    main()
