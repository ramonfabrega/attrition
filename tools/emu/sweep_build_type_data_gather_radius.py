#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_type_data_gather_radius.py — `BuildTypeData::gather_radius@0063bc60` under unicorn (item 2061,
hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_build_type_data_gather_radius.py <install>/riseofnations.exe

`__thiscall`, no argument, `ret`. Two virtual tests each compared against the base class's own slot first: slot
`+0x90` (the base's is `0x472bb0`: the build flags' `+0x2c0 & 0x40`) and slot `+0x94` (`0x472b20`: `& 0x10000000`);
a row's vtable holds either the base's address (compared, never entered) or a hooked stub that answers the same
bit's truth in a word of its own. A gathering, non-flat type asks slot `+0x60` (hooked, two arguments) with
`(0x1a4, 0)`, then `(0x1a2, 0)`, each answered "the type's index is that one"; the constants (the cell
`0xc061e4`) give `+0xb4` for the second and `+0xb0` otherwise, laid out as the rules' 8 and 6.
Row: `<flags> <index> <own 0x90> <own 0x94> -> <eax> | <calls>`, each call `<name>:<ecx is this>:<arguments>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x0063BC60
CONSTS = 0x00C061E4
BASE_90, BASE_94 = 0x00472BB0, 0x00472B20


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2061)
    this = h.alloc(0x300)
    rec = h.alloc(0x200)
    h.w32(CONSTS, rec)
    st = {"g": 0, "f": 0, "ix": 0}
    stubs = {0x90: ("gathers", 0, lambda e, a: st["g"]), 0x94: ("flat", 0, lambda e, a: st["f"]),
             0x60: ("is", 2, lambda e, a: int(a[0] == st["ix"]))}
    vts = {}
    for own90 in (0, 1):
        for own94 in (0, 1):
            v = h.vtable({k: s for k, s in stubs.items() if k == 0x60 or (k == 0x90 and own90) or (k == 0x94 and own94)})
            if not own90:
                h.w32(v + 0x90, BASE_90)
            if not own94:
                h.w32(v + 0x94, BASE_94)
            vts[own90, own94] = v
    for _ in range(400):
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x300)))
        uc.mem_write(rec, bytes(rng.randrange(256) for _ in range(0x200)))
        h.w32(rec + 0xB4, 8)
        h.w32(rec + 0xB0, 6)
        flags = rng.getrandbits(32)
        if rng.random() < 0.7:
            flags |= 0x40
        if rng.random() < 0.6:
            flags &= ~0x10000000
        h.w32(this + 0x2C0, flags)
        st["ix"] = rng.choice((0x1A1, 0x1A2, 0x1A2, 0x1A3, 0x1A4, 0x1A4, 0x1A5, 0x1A6, rng.randrange(0x340)))
        st["g"] = rng.choice((1, 0x40, -1)) if flags & 0x40 else 0
        st["f"] = rng.choice((1, 0x10000000, -1)) if flags & 0x10000000 else 0
        own90, own94 = rng.random() < 0.4, rng.random() < 0.4
        h.w32(this, vts[int(own90), int(own94)])
        before = bytes(uc.mem_read(this, 0x300))
        h.calls.clear()
        res = h.call(ENTRY, (), ecx=this)
        assert uc.reg_read(UC_X86_REG_ESP) == STACK_BASE + STACK_SIZE - 0x100, "wrong pop"
        assert bytes(uc.mem_read(this, 0x300)) == before, "wrote"
        calls = ",".join(f"{name}:{int(ecx == this)}:" + "/".join(str(a) for a in args) for name, ecx, args, _ in h.calls)
        print(f"{flags} {st['ix']} {int(own90)} {int(own94)} -> {res} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
