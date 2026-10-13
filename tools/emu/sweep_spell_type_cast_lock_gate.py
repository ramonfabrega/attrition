#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_spell_type_cast_lock_gate.py — `SpellType::cast_lock_gate@00671480` under unicorn (item 1971, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_spell_type_cast_lock_gate.py <install>/riseofnations.exe

`__thiscall`, two stack arguments `p0`, `p1`. The cell `0x00c0618c` points at a record whose pointer at `+0x14 + p1*0x1c` names an array;
object `array[p0]` has slot 0xac (hooked, answers a pointer) called, and the byte at `answer+0x81` is set to 1. Row: `<p0> <p1> <byte before> -> <byte after> | <ecx the slot saw>`;
the script asserts no other gate record's byte moved.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00671480
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def pop_ok(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(1971)
    vt = h.vtable({0xac: ("gate", 0, 0)})
    G = h.alloc(0x14 + 0x1c * 4 + 16)
    objs, recs = {}, {}
    for p1 in range(4):
        arr = h.alloc(16)
        h.w32(G + 0x14 + p1 * 0x1c, arr)
        for p0 in range(4):
            o, rec = h.alloc(0x10), h.alloc(0x100)
            h.w32(o, vt)
            h.w32(arr + 4 * p0, o)
            objs[(p0, p1)], recs[(p0, p1)] = o, rec
    byo = {o: recs[k] for k, o in objs.items()}
    h.answers["gate"] = lambda ecx, args: byo[ecx]
    for _ in range(300):
        p0, p1 = rng.randrange(4), rng.randrange(4)
        before = {k: rng.randrange(256) for k in recs}
        for k, r in recs.items():
            h.m.uc.mem_write(r + 0x81, bytes([before[k]]))
        uc.mem_write(0x00C0618C, G.to_bytes(4, "little"))
        h.calls.clear()
        h.call(ENTRY, (p0, p1))
        pop_ok(uc)
        for k, r in recs.items():
            now = uc.mem_read(r + 0x81, 1)[0]
            assert now == (before[k] if k != (p0, p1) else 1), f"record {k}: {now}"
        assert len(h.calls) == 1
        print(f"{p0} {p1} {before[(p0, p1)]} -> 1 | {h.calls[0][1] == objs[(p0, p1)]}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
