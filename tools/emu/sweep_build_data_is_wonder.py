#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_data_is_wonder.py — `BuildData::is_wonder@00472320` under unicorn (item 2090, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_build_data_is_wonder.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`. `ecx` becomes the type record at `this+0x18`; when that record's
vtable **is** `BuildType::vftable` (`0xb42b94`, the image's own) it answers `0x20e <= [rec+4] < 0x21f`, signed;
otherwise it jumps to the record's slot `+0x1c` (`is_wonder_type`) with `ecx` the record, and that slot's answer
is the answer. Half the rows take each arm; the slot is a stub answered per row.
Row: `<real vtable 0|1> <type> <slot answer> -> <eax> | <calls>`, a call `rec|other`; the building's 0x40 bytes
and the record's 0x40 are random and held unwritten.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00472320
BUILDTYPE_VFTABLE = 0x00B42B94
I32 = (0, 1, -1, 2, 0x7FFFFFFF, -0x80000000, 0x100, 0x20D, 0x20E, 0x21E, 0x21F, 0x19E)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2090)
    ans = [0]
    vt = h.vtable({0x1C: ("is_wonder_type", 0, lambda ecx, args: ans[0])})
    this = h.alloc(0x60)
    rec = h.alloc(0x60)
    for i in range(400):
        real = i % 2
        t = rng.randint(0x200, 0x225) if rng.random() < 0.6 else rng.choice(I32)
        ans[0] = rng.choice(I32) if rng.random() < 0.5 else rng.randint(-0x80000000, 0x7FFFFFFF)
        mem = bytearray(rng.randrange(256) for _ in range(0x40))
        mem[0x18:0x1C] = struct.pack("<I", rec)
        rmem = bytearray(rng.randrange(256) for _ in range(0x40))
        rmem[0:4] = struct.pack("<I", BUILDTYPE_VFTABLE if real else vt)
        rmem[4:8] = struct.pack("<i", t)
        uc.mem_write(this, bytes(mem))
        uc.mem_write(rec, bytes(rmem))
        before = bytes(uc.mem_read(this, 0x60)) + bytes(uc.mem_read(rec, 0x60))
        h.calls.clear()
        r = h.call(ENTRY, (), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(this, 0x60)) + bytes(uc.mem_read(rec, 0x60)) == before, "wrote memory"
        calls = ";".join("rec" if ecx == rec else "other" for _, ecx, _, _ in h.calls)
        print(f"{real} {t} {ans[0]} -> {r} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
