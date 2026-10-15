#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_rivers_walk_data.py — `Rivers::walk_data@008890e0` under unicorn (item 2583, tools/sweep_gen.py, mix).

    uv run tools/emu/sweep_rivers_walk_data.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object, one stack argument: the walker. The function makes
3 step(s) in order — direct callees (hooked), the walker's callback (slot 0, two
arguments) and its slot 4 (one argument), the last gated on [walker+0x4] != 0.
One row per input, `<this> <walker> <g> <gate> -> <result> | <ecx,args…>;…` (the gate column is
the leftover `ecx` of the step before when a step names `stale`).
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x008890e0
CALLS = [(8943856, 0, 'this', ''), (8937504, 0, 'stale', ''), (4861824, 2, '0xc11c10', 'p0,stale|this')]
GLOBAL = None
GATE = 0x4
GATE_THIS = False
GATE_AT = None
LEN_AT = None
VBASE = None
STALE = True


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {a: (f"{a:x}", n) for a, n, _, _ in CALLS})
    rng = random.Random(2583)
    vt = h.vtable({0: ("walk", 2, 7), 4: ("walk4", 1, 9)})
    for i in range(140):
        this = rng.randrange(0x1000, 0x7FFF0000) & ~3
        if GATE_THIS or GATE_AT is not None or LEN_AT is not None:
            this = h.alloc(0x100)  # a written object: the gate is the short at [this]
        g = rng.randrange(0x1000, 0x7FFF0000) & ~3
        if VBASE is not None:
            # `this` sits inside a record; [this-VBASE] points at a record whose +4 is the displacement g
            this = h.alloc(0x200) + 0x100
            g = rng.randint(-0x100, 0x1000)  # the displacement, in the g column
            vb = h.alloc(0x20)
            h.w32(vb + 4, g)
            h.w32(this - VBASE, vb)
        gv = 0 if i % 2 == 0 else rng.randint(1, 0x7FFFFFFF)
        if STALE:
            # the `ecx` a callback or a direct callee leaves (parked 1922, 2498, 2563), printed in the gate column
            sv = g  # a walker word is the gate: the leftover is the `g` column, the gate column stays the gate
            if GATE is None:
                gv = sv = rng.randrange(0x1000, 0x7FFF0000) & ~3
            h.answers["walk"] = (7, sv)
            h.answers["walk4"] = (9, sv)
            for a, _, _, _ in CALLS:
                h.answers[f"{a:x}"] = (0, sv)
        walker = h.alloc(32)
        h.w32(walker, vt)
        if GATE is not None:
            h.w32(walker + GATE, gv)
        if GATE_AT is not None:
            # a bit at another offset: the gate column is the masked byte (0 on even rows)
            off, mask = GATE_AT
            v = rng.randrange(256)
            v = v & ~mask & 0xFF if i % 2 == 0 else v | mask
            h.w8(this + off, v)
            gv = v & mask
        elif LEN_AT is not None:
            # a walk length read from memory: the sign-extended short, the gate column
            gv = 0 if i % 3 == 0 else rng.choice([-1, 1, rng.randint(-0x8000, 0x7FFF)]) or 1
            h.w16(this + LEN_AT, gv)
        elif GATE_THIS:
            gv = gv & 0xFFFF if i % 4 != 3 else 0
            h.w16(this, gv - 0x10000 if gv & 0x8000 else gv)
        if GLOBAL is not None:
            rec = h.alloc(0x40)
            h.w32(rec + 0x10, g)
            h.m.uc.mem_write(GLOBAL, struct.pack("<I", rec))
        h.calls.clear()
        r = h.call(ENTRY, (walker,), ecx=this)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        seen = ";".join(",".join(str(x) for x in (ecx, *args)) for _, ecx, args, _ in h.calls)
        print(f"{this} {walker} {g} {gv} -> {r} | {seen}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

# sweep_gen: ["mix", "Rivers::walk_data@008890e0", "--item", "2583", "--gate", "4", "--gate-ne", "--step", "gated/call:0x8878f0:0:this:", "--step", "gated/call:0x886020:0:stale:", "--step", "call:0x4a2f80:2:0xc11c10:p0,stale|this"]
