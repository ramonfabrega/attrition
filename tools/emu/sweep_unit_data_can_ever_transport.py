#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_can_ever_transport.py — `UnitData::can_ever_transport@0046f290` under unicorn (item 2112,
hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_unit_data_can_ever_transport.py <install>/riseofnations.exe

`__thiscall`, no stack arguments. The type record at `+0x18`, its domain at `+0x218`: zero answers 1. Otherwise
`this`'s slot `+0x18` (a stub, the folded `return 1`) zero answers 0; then `units[byte +0x9][signed 16-bit +0xa]`
(the table at `0xc0aec0 + 0x1c · player`) — its type record's carry `+0x2d4` zero answers 0; then `is(0x15f, 0)`:
`this`'s slot `+0xb8` against `ObjectData::is@00653790` — equal, the record's slot `+0x60` (inlined), different,
the slot — and the answer is `is == 0`. A row's mode is 1 when slot `+0xb8` holds the base address.
Row: `<mode> <domain> <s18> <carry> <is> -> <eax> | <calls>`, a call `s18|is|rec60,this|rec|other,<args…>`; the
unit's 0x40 bytes, the record's 0x300, the looked-up unit's 0x40 and its record's 0x300 are random and held.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0046F290
OBJECTDATA_IS = 0x00653790
UNITS = 0x00C0AEC0
I32 = (1, -1, 2, 0x7FFFFFFF, -0x80000000, 0x100)


def word(rng):
    if rng.random() < 0.4:
        return 0
    return rng.choice(I32) if rng.random() < 0.5 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2112)
    vt = h.vtable({0x18: ("s18", 0, 0), 0xB8: ("is", 2, 0)})
    stub = h.r32(vt + 0xB8)
    rvt = h.vtable({0x60: ("rec60", 2, 0)})
    this = h.alloc(0x40)
    rec = h.alloc(0x300)
    other = h.alloc(0x40)
    orec = h.alloc(0x300)
    arr = h.alloc(0x40000)
    for i in range(400):
        mode = i % 2
        domain, s18, carry, isans = word(rng), word(rng), word(rng), word(rng)
        h.answers["s18"] = s18
        h.answers["is"] = h.answers["rec60"] = isans
        h.w32(vt + 0xB8, OBJECTDATA_IS if mode else stub)
        player, idx = rng.randrange(256), rng.randint(-0x8000, 0x7FFF)
        mem = bytearray(rng.randrange(256) for _ in range(0x40))
        mem[0:4] = struct.pack("<I", vt)
        mem[9] = player
        mem[0xA:0xC] = struct.pack("<h", idx)
        mem[0x18:0x1C] = struct.pack("<I", rec)
        rmem = bytearray(rng.randrange(256) for _ in range(0x300))
        rmem[0:4] = struct.pack("<I", rvt)
        rmem[0x218:0x21C] = struct.pack("<i", domain)
        omem = bytearray(rng.randrange(256) for _ in range(0x40))
        omem[0x18:0x1C] = struct.pack("<I", orec)
        ormem = bytearray(rng.randrange(256) for _ in range(0x300))
        ormem[0x2D4:0x2D8] = struct.pack("<i", carry)
        for a, b in ((this, mem), (rec, rmem), (other, omem), (orec, ormem)):
            uc.mem_write(a, bytes(b))
        base = arr + 0x20000
        uc.mem_write(base + 4 * idx, struct.pack("<I", other))
        uc.mem_write(UNITS + 0x1C * player, struct.pack("<I", base))
        held = ((this, 0x40), (rec, 0x300), (other, 0x40), (orec, 0x300))
        before = b"".join(bytes(uc.mem_read(a, n)) for a, n in held)
        h.calls.clear()
        r = h.call(ENTRY, (), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert b"".join(bytes(uc.mem_read(a, n)) for a, n in held) == before, "wrote memory"
        who = {this: "this", rec: "rec"}
        calls = ";".join(",".join([name, who.get(ecx, "other"), *map(str, args)]) for name, ecx, args, _ in h.calls)
        print(f"{mode} {domain} {s18} {carry} {isans} -> {r} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
