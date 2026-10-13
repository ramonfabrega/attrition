#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_spell_type_cast_supplies.py — `SpellType::cast_supplies@006714c0` under unicorn (item 1942).

    uv run tools/emu/sweep_spell_type_cast_supplies.py <install>/riseofnations.exe

`__thiscall`, `ecx` unused, two stack arguments (`ret 8`): an index `p0` and a player `p1`. The record is `**(0x00c0aec0 + p1*0x1c)[p0]`: the word at offset `0x96` of the `p0`th
object of the `p1`th array is zeroed. One row per input, `<p0> <p1> <old word> -> <new word>`; the script asserts every object's other bytes, and every other object, are unmoved.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x006714c0
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x3F800000, 0x7FC00000)


def check_esp(h):
    esp = h.m.uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


def calls_str(h, base=0):
    return ";".join(",".join(str(x) for x in (ecx - base if i == 0 else ecx, *args)) for i, (_, ecx, args, _) in enumerate(h.calls))


def main(argv):
    from hooks import Harness
    rng = random.Random(1942)
    h = Harness(argv[1], {})
    import struct as st
    rows = [(i, p) for p in range(8) for i in (0, 1, 5, 11)] + [(rng.randrange(12), rng.randrange(8)) for _ in range(150)]
    objs = []
    for pl in range(8):
        arr = h.alloc(4 * 12)
        for k in range(12):
            o = h.alloc(0x100)
            h.w32(arr + 4 * k, o)
            objs.append((pl, k, o))
        h.m.uc.mem_write(0xC0AEC0 + 0x1C * pl, st.pack("<I", arr))
    for i, p in rows:
        for pl, k, o in objs:
            h.m.uc.mem_write(o, bytes(rng.randrange(256) for _ in range(0x100)))
        before = {(pl, k): bytes(h.m.uc.mem_read(o, 0x100)) for pl, k, o in objs}
        addr = {(pl, k): o for pl, k, o in objs}
        old = st.unpack("<H", before[(p, i)][0x96:0x98])[0]
        h.call(ENTRY, (i, p), ecx=0x1234)
        check_esp(h)
        for key, o in addr.items():
            now = bytes(h.m.uc.mem_read(o, 0x100))
            if key == (p, i):
                assert now[:0x96] == before[key][:0x96] and now[0x98:] == before[key][0x98:], "touched another byte"
                new = st.unpack("<H", now[0x96:0x98])[0]
            else:
                assert now == before[key], f"object {key} moved"
        print(f"{i} {p} {old} -> {new}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
