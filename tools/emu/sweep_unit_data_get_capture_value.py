#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_get_capture_value.py — `UnitData::get_capture_value@00610a70` under unicorn (item 2083,
hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_unit_data_get_capture_value.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`. A decoy (`unit_masks` `+0x68` bit 0) answers 0 unasked; else the
type record at `+0x18` is asked its slot `+0x10c` (`is_siege`, no arguments), non-zero answering 0; else the
type's `+0x308` against the unit's signed byte `+0xb5` (`guy_mark`), the smaller (`cmovge`), floored at 1. The
slot's answer is zero half the time, else an edge or a random word; `+0x308` and the byte are drawn from edges
and random values. Both objects' bytes are random and held unwritten.
Row: `<decoy> <slot answer> <type +0x308> <guy_mark> <calls> -> <eax>`, `calls` the number of slot calls with
`ecx` the type record.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00610A70
SIEGE = 0x10C
WIN = 0x100
TWIN = 0x340
I32 = (0, 1, -1, 2, 3, 4, 5, 8, 12, 0x7F, 0x80, -0x80, 0x7FFFFFFF, -0x80000000)


def pick(rng):
    return rng.choice(I32) if rng.random() < 0.6 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2083)
    vt = h.vtable({SIEGE: ("siege", 0, 0)})
    unit = h.alloc(WIN + 0x20)
    ty = h.alloc(TWIN + 0x20)
    for _ in range(400):
        siege = 0 if rng.random() < 0.5 else pick(rng)
        h.answers["siege"] = siege
        squad = pick(rng) if rng.random() < 0.5 else rng.randint(-2, 20)
        mem = bytearray(rng.randrange(256) for _ in range(WIN))
        if rng.random() < 0.6:
            mem[0x68] &= 0xFE
        if rng.random() < 0.6:
            mem[0xB5] = rng.randint(-3, 20) & 0xFF
        mem[0x18:0x1C] = ty.to_bytes(4, "little")
        tmem = bytearray(rng.randrange(256) for _ in range(TWIN))
        tmem[0:4] = vt.to_bytes(4, "little")
        tmem[0x308:0x30C] = (squad & 0xFFFFFFFF).to_bytes(4, "little")
        uc.mem_write(unit, bytes(mem))
        uc.mem_write(ty, bytes(tmem))
        before = bytes(uc.mem_read(unit, WIN + 0x20)), bytes(uc.mem_read(ty, TWIN + 0x20))
        h.calls.clear()
        r = h.call(ENTRY, (), ecx=unit)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert (bytes(uc.mem_read(unit, WIN + 0x20)), bytes(uc.mem_read(ty, TWIN + 0x20))) == before, "wrote"
        assert all(c[1] == ty and c[2] == [] for c in h.calls), "a call not on the type"
        mark = mem[0xB5] - 256 if mem[0xB5] & 0x80 else mem[0xB5]
        print(f"{mem[0x68] & 1} {siege} {squad} {mark} {len(h.calls)} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
