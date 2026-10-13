#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_recharge.py — `UnitData::recharge@0060fdf0` under unicorn (item 2061, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_unit_data_recharge.py <install>/riseofnations.exe

`__thiscall`, one argument (an out-pointer, or null), `ret 4`. The base is the type's `+0x1f4` (the type at
`this+0x18`); a non-null out-pointer takes -1 first. The type's slot `+0x10c` (hooked, none) gates the rule:
0 answers the base. Past it, unless the constants' `+0xc20` (the cell `0xc061e4`) and `this+0x6c & 1` are both
set, the unit's supply is asked (the direct callee at `0x609ef0`, hooked, two arguments, the second the
out-pointer) and a non-zero answers the base. Otherwise the bombard test: `this`'s slot `+0xb8` with `(0x10b,
0)`, or — when that slot is the base class's own (`0x653790`, compared, never entered) — the type's slot `+0x60`
with the same two (both hooked); non-zero doubles the base, zero takes half again (`(3·b)/2`, truncated).
Row: `<base> <rule> <slow> <masks2> <supplied> <devirt> <bombard> <out> -> <eax> | <calls> | <out word>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0060FDF0
SUPPLY = 0x00609EF0
CONSTS = 0x00C061E4
IS = 0x00653790


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {SUPPLY: ("supply", 2)})
    uc = h.m.uc
    rng = random.Random(2061)
    this = h.alloc(0x100)
    ty = h.alloc(0x300)
    rec = h.alloc(0x1000)
    out = h.alloc(0x10)
    st = {"rule": 0, "bomb": 0}
    tvt = h.vtable({0x10C: ("rule", 0, lambda e, a: st["rule"]), 0x60: ("type_is", 2, lambda e, a: st["bomb"])})
    vt_own = h.vtable({0xB8: ("this_is", 2, lambda e, a: st["bomb"])})
    vt_base = h.alloc(0x200)
    h.w32(vt_base + 0xB8, IS)
    h.w32(CONSTS, rec)
    for _ in range(400):
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x100)))
        uc.mem_write(ty, bytes(rng.randrange(256) for _ in range(0x300)))
        uc.mem_write(rec, bytes(rng.randrange(256) for _ in range(0x1000)))
        uc.mem_write(out, bytes(rng.randrange(256) for _ in range(0x10)))
        h.w32(ty, tvt)
        h.w32(this + 0x18, ty)
        devirt = rng.random() < 0.6
        h.w32(this, vt_base if devirt else vt_own)
        base = rng.randrange(0x55) if rng.random() < 0.8 else rng.randint(-0x100, 0x100)
        h.w32(ty + 0x1F4, base)
        st["rule"] = rng.choice((0, 1, 1, 1, 2, -1))
        slow = rng.choice((0, 1, 0x100))
        h.w32(rec + 0xC20, slow)
        m2 = rng.randrange(256)
        h.w8(this + 0x6C, m2)
        sup = rng.choice((0, 0, 1, 2, -1))
        h.answers["supply"] = sup
        st["bomb"] = rng.choice((0, 0, 1, 2, -1))
        ptr = out if rng.random() < 0.6 else 0
        before = bytes(uc.mem_read(this, 0x100)), bytes(uc.mem_read(ty, 0x300))
        h.calls.clear()
        res = h.call(ENTRY, (ptr,), ecx=this)
        assert uc.reg_read(UC_X86_REG_ESP) == STACK_BASE + STACK_SIZE - 0x100, "wrong pop"
        assert (bytes(uc.mem_read(this, 0x100)), bytes(uc.mem_read(ty, 0x300))) == before, "wrote"
        want = {"rule": ty, "type_is": ty, "this_is": this, "supply": this}
        args = {"rule": [], "type_is": [0x10B, 0], "this_is": [0x10B, 0]}
        calls = ",".join(
            f"{name}:{int(ecx == want[name] and (a[1:] == [ptr] if name == 'supply' else a == args[name]))}"
            for name, ecx, a, _ in h.calls)
        word = h.r32(out) if ptr else 0
        print(f"{base} {st['rule']} {slow} {m2} {sup} {int(devirt)} {st['bomb']} {int(ptr != 0)} -> {res} | {calls} | {word}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
