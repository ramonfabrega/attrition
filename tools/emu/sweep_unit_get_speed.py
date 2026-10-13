#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_get_speed.py — `UnitData::get_speed@006086f0` under unicorn.

    uv run tools/emu/sweep_unit_get_speed.py <install>/riseofnations.exe

006086f0 is a thunk: it XOR-decodes the unit's own point (`this+0x10`,
`this+0x14`, `^ 0x63637`) and calls vslot 0x17c with `(x, y, flag)`. On a
`UnitData` the slot is `UnitData::get_speed(x, y, flag)@00608720` (the
`.rdata` word at 0xb41b08 + 0x17c, checked below), so this runs the pair.

SLICE: only the arm where the unit's **type is not land** (`type +0x218 != 0`)
is reachable without a packet. The land arm — the river halving, the
`unit_masks & 0x10` halving, the general's siege doubling — reads
`GameAccessConst::worldc`, `ObjectData::has_general` and the leaders, and
`UnitData::speed@0060aae0`'s land arm calls through the type's vtable; it
faults here and needs a packet. The group cap (`group >= 0`) reads the
`groups` global and is not driven either: `group = -1` throughout.

Synthesized, all in one page at THIS: the vtable word (the real one, in the
image), `this+0x18` -> a type page whose `+0x218` is the domain, `this+0x68`
(`unit_masks`, bit 0x40000 = the AI's), `this+0x80` (`group` = -1),
`this+0x9a` (`myspeed`), `this+0xdc` (the order ring's tail). Each order is
an object with two code stubs standing for its vslots `+0x10` (the
`OrderIndex`) and `+0x14` (is a move), and `+4` its flags byte.

A row is `speed domain ai o1 o2 o3 flag -> result`, domain 1 or 2, ai 0 or 1,
and each `o` an order kind in queue order, 0 = none:
1 transit move, 2 move that is the action, 3 attack, 4 guard.
"""
import itertools
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

GET_SPEED = 0x006086F0
UNIT_VTABLE = 0x00B41B08
SLOT_17C = 0x608720

THIS, TYPE, ORDERS, CODE = 0x1000_0000, 0x1000_1000, 0x1000_2000, 0x1000_3000

# kind -> (OrderIndex, vslot 0x14 answer, flags byte)
KINDS = {1: (1, 1, 0), 2: (1, 1, 4), 3: (10, 0, 4), 4: (12, 0, 4)}


def stub(m, at, value):
    m.uc.mem_write(at, b"\xb8" + struct.pack("<I", value) + b"\xc3")


def lay_out(m, speed, domain, ai, kinds):
    uc = m.uc
    uc.mem_write(THIS, bytes(0x1000))
    uc.mem_write(TYPE, bytes(0x1000))
    uc.mem_write(ORDERS, bytes(0x1000))
    uc.mem_write(THIS + 0x00, struct.pack("<I", UNIT_VTABLE))
    uc.mem_write(THIS + 0x18, struct.pack("<I", TYPE))
    uc.mem_write(THIS + 0x68, struct.pack("<I", 0x40000 if ai else 0))
    uc.mem_write(THIS + 0x80, struct.pack("<h", -1))
    uc.mem_write(THIS + 0x9A, struct.pack("<h", speed))
    # The unit's own point, stored XORed (the thunk decodes it).
    uc.mem_write(THIS + 0x10, struct.pack("<II", 1234 ^ 0x63637, 5678 ^ 0x63637))
    uc.mem_write(TYPE + 0x218, struct.pack("<I", domain))
    kinds = [k for k in kinds if k]
    if not kinds:
        return  # the ring's tail stays null: `get_action` answers null
    nodes = [ORDERS + 0x100 * i for i in range(len(kinds))]
    for i, k in enumerate(kinds):
        idx, mv, fl = KINDS[k]
        obj, vt, code = nodes[i] + 0x40, nodes[i] + 0x60, CODE + 0x20 * i
        stub(m, code, idx)
        stub(m, code + 0x10, mv)
        uc.mem_write(vt + 0x10, struct.pack("<II", code, code + 0x10))
        uc.mem_write(obj, struct.pack("<I", vt))
        uc.mem_write(obj + 4, bytes([fl]))
        nxt = nodes[(i + 1) % len(kinds)]
        uc.mem_write(nodes[i] + 4, struct.pack("<II", nxt, obj))
    # The stubs are rewritten at the same addresses each row; unicorn caches
    # translated blocks, so a stale one would answer for the last row's.
    uc.ctl_flush_tb()
    uc.mem_write(THIS + 0xDC, struct.pack("<I", nodes[-1]))


def cases(seed=1575):
    rng = random.Random(seed)
    edge_speeds = [-40, -1, 0, 1, 2, 3, 4, 5, 7, 8, 9, 10, 16, 17, 34, 100, 1000, 32767, -32768]
    seqs = [()]
    for n in (1, 2, 3):
        seqs += list(itertools.product((1, 2, 3, 4), repeat=n))
    for seq in seqs:
        for domain in (1, 2):
            for ai in (0, 1):
                for flag in (0, 1):
                    sp = rng.choice(edge_speeds) if rng.random() < 0.3 else rng.randint(-100, 3000)
                    yield sp, domain, ai, seq, flag
    for sp in edge_speeds:
        for ai in (0, 1):
            for k in (0, 3, 4):
                yield sp, 1, ai, (k,), 0
    for _ in range(200):
        yield (rng.randint(-32768, 32767), rng.choice((1, 2)), rng.randint(0, 1),
               tuple(rng.randint(0, 4) for _ in range(3)), rng.randint(0, 1))


def main(argv):
    exe = argv[1]
    img = Image(exe)
    m = Machine(img)
    for page in (THIS, TYPE, ORDERS, CODE):
        m.uc.mem_map(page, 0x1000)
    got = struct.unpack("<I", m.uc.mem_read(UNIT_VTABLE + 0x17C, 4))[0]
    assert got == SLOT_17C, f"UnitData vslot 0x17c is {got:#x}, not {SLOT_17C:#x}"
    for sp, domain, ai, seq, flag in cases():
        seq = tuple(seq) + (0,) * (3 - len(seq))
        lay_out(m, sp, domain, ai, seq)
        r = m.call(GET_SPEED, (flag,), ecx=THIS)
        print(f"{sp} {domain} {ai} {seq[0]} {seq[1]} {seq[2]} {flag} -> {signed(r)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
