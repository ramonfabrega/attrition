#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_queue_walk_data.py — `BuildQueue::walk_data@006305f0` under unicorn (item 2698, tools/sweep_gen.py, mix).

    uv run tools/emu/sweep_build_queue_walk_data.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object, one stack argument: the walker. The function makes
5 step(s) in order — direct callees (hooked), the walker's callback (slot 0, two
arguments) and its slot 4 (one argument, from the global cell), the last gated on [walker+0x4] != 0.
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

ENTRY = 0x006305f0
CALLS = [(6489792, 0, 'this', ''), (6490064, 1, 'this', 'count')]
GLOBAL = 0xc06378
GATE = 0x4
GATE_THIS = False
GATE_AT = None
LEN_AT = None
VBASE = None
STALE = False
STALE_COL = False
VBASE2 = None
BIT_AT = None
REC = False
STORE = None
ELEMS = ('count', 0x0, 0x4, 20)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {a: (f"{a:x}", n) for a, n, _, _ in CALLS})
    rng = random.Random(2698)
    vt = h.vtable({0: ("walk", 2, 7), 4: ("walk4", 1, 9)})
    for i in range(140):
        this = rng.randrange(0x1000, 0x7FFF0000) & ~3
        if GATE_THIS or GATE_AT is not None or BIT_AT is not None or LEN_AT is not None or STORE is not None or (ELEMS and ELEMS[0] in ('list', 'count')):
            this = h.alloc(0x100)  # a written object: the gate is the short at [this]
        g = rng.randrange(0x1000, 0x7FFF0000) & ~3
        if VBASE is not None:
            # `this` sits inside a record; [this-VBASE] points at a record whose +4 is the displacement g
            this = h.alloc(0x200) + 0x100
            g = rng.randint(-0x100, 0x1000)  # the displacement, in the g column
            vb = h.alloc(0x20)
            h.w32(vb + 4, g)
            h.w32(this - VBASE, vb)
        d2 = 0
        if VBASE2 is not None:
            # a second displacement record at [this-VBASE2], its +4 the `d2` column (parked 2668, 2674)
            d2 = rng.randint(-0x100, 0x1000)
            vb2 = h.alloc(0x20)
            h.w32(vb2 + 4, d2)
            h.w32(this - VBASE2, vb2)
        gv = 0 if i % 2 == 0 else rng.randint(1, 0x7FFFFFFF)
        if STALE:
            # the `ecx` a callback or a direct callee leaves (parked 1922, 2498, 2563), printed in the gate column
            sv = g  # a walker word is the gate: the leftover is the `g` column, the gate column stays the gate
            if STALE_COL:
                sv = rng.randrange(0x1000, 0x7FFF0000) & ~3  # its own column (parked 2673), printed after the gate
            elif GATE is None:
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
            if REC:
                g = rec  # the record base the cell holds is the g column
            h.w32(rec + 0x10, g)
            h.m.uc.mem_write(GLOBAL, struct.pack("<I", rec))
        el = ""
        if ELEMS and ELEMS[0] == 'table':
            # the elements: pointer words of the table *(*cell+0x10), one per STEP bytes in LO..HI
            _, lo, hi, stp = ELEMS
            tbl = h.alloc(hi + 0x10)
            h.w32(rec + 0x10, tbl)
            elems = [rng.randrange(0x1000, 0x7FFF0000) & ~3 for _ in range(lo, hi, stp)]
            for k, e in enumerate(elems):
                h.w32(tbl + lo + k * stp, e)
            el = "".join(f" {e}" for e in elems)
        elif ELEMS and ELEMS[0] == 'count':
            # a count at this+CNT and a record array at [this+PTR], STRIDE bytes a record (parked 2689)
            _, cnt, ptr, stp = ELEMS
            count = rng.choice([0, 1, 2, 3, 5, -1])
            base = h.alloc(max(count, 1) * stp + 0x10)
            h.w32(this + cnt, count)
            h.w32(this + ptr, base)
            elems = [base + k * stp for k in range(max(count, 0))]
            el = f" {count}" + "".join(f" {e}" for e in elems)
        elif ELEMS:
            # a linked list: the head at this+HEAD, each node's next at +NEXT, ended by null
            _, head, nxt = ELEMS
            elems = [h.alloc(0x40) for _ in range(rng.choice([0, 1, 2, 3, 5]))]
            for k, e in enumerate(elems):
                h.w32(e + nxt, elems[k + 1] if k + 1 < len(elems) else 0)
            h.w32(this + head, elems[0] if elems else 0)
            el = "".join(f" {e}" for e in elems)
        sw = f" {sv}" if STALE and STALE_COL else ""
        if VBASE2 is not None:
            sw += f" {d2}"
        if BIT_AT is not None:
            # a second gate: a bit of `this`, its masked byte the last extra column (parked 2669, 2675)
            boff, bmask = BIT_AT
            bv = rng.randrange(256)
            bv = bv & ~bmask & 0xFF if i % 3 == 0 else bv | bmask
            h.w8(this + boff, bv)
            sw += f" {bv & bmask}"
        if STORE is not None:
            off, val, size, woff = STORE
            w = 0 if i % 3 == 0 else rng.randint(1, 0x7FFFFFFF)
            h.w32(walker + woff, w)
            sw += f" {w}"
            h.m.uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x100)))
            pre = bytes(h.m.uc.mem_read(this, 0x100))
        h.calls.clear()
        r = h.call(ENTRY, (walker,), ecx=this)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        seen = ";".join(",".join(str(x) for x in (ecx, *args)) for _, ecx, args, _ in h.calls)
        after = ""
        if STORE is not None:
            post = bytes(h.m.uc.mem_read(this, 0x100))
            changed = [k for k in range(0x100) if post[k] != pre[k]]
            assert all(off <= k < off + size for k in changed), f"wrote outside the store: {changed}"
            after = " | " + post[off:off + size].hex() + " " + pre[off:off + size].hex()
        print(f"{this} {walker} {g} {gv}{sw}{el} -> {r} | {seen}{after}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

# sweep_gen: ["mix", "BuildQueue::walk_data@006305f0", "--item", "2698", "--glob", "0xc06378", "--gate", "4", "--gate-ne", "--elems", "count:0:4:20", "--step", "walk4:6000", "--step", "walk:this:this+4", "--step", "gated/zero/call:0x6306c0:0:this:", "--step", "gated/nz/call:0x6307d0:1:this:count", "--step", "each/walk:elem:elem+0x12"]
