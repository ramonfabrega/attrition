#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_type_data_research_time.py — `TypeData::research_time@006639a0` under unicorn (item 2090, hand-written on
tools/emu/hooks.py; `sweep_type_data_time.py`'s layout).

    uv run tools/emu/sweep_type_data_research_time.py <install>/riseofnations.exe

`__thiscall`, one stack argument (`who`), `ret 4`. The record's slot `+0x48` against `TypeData::is_spell_type`
(`0x470590`) as in `time`: a spell costs `res_time` (`+0xc`) `· 100`; anything else asks its own slot `+0x6c`
(`time`, one argument: `who`) and answers `(answer · research_tick_premium) >> 8`, the product wrapping (`imul`)
and the shift **logical** (`shr`), the premium the word at `[0xc061e4] + 0x3a8`.
Row: `<base 0|1> <type> <0x48 answer> <0x6c answer> <who> <premium> <res_time> -> <eax> | <calls>`, a call
`name,rec|other,<args…>`; the record's 0x40 bytes are random and held unwritten.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x006639A0
IS_SPELL_TYPE = 0x00470590
CONSTANTS = 0x00C061E4
PREMIUM = 0x3A8
I32 = (0, 1, -1, 2, 0x7FFFFFFF, -0x80000000, 0x100, 30, 45, 600, 3000, 0x1000000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.5 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2090)
    ans = [0, 0]
    time = ("time", 1, lambda ecx, args: ans[1])
    vt_stub = h.vtable({0x48: ("is_spell_type", 0, lambda ecx, args: ans[0]), 0x6C: time})
    vt_base = h.vtable({0x6C: time})
    h.w32(vt_base + 0x48, IS_SPELL_TYPE)
    rec = h.alloc(0x60)
    consts = h.alloc(0x400)
    uc.mem_write(CONSTANTS, struct.pack("<I", consts))
    for i in range(400):
        base = i % 2
        t = rng.randint(0x270, 0x2B0) if rng.random() < 0.7 else rand32(rng)
        ans[0] = 0 if rng.random() < 0.4 else rand32(rng)
        ans[1] = rand32(rng)
        who = rng.randint(-2, 9) if rng.random() < 0.8 else rand32(rng)
        premium = rng.choice((256, 512, 128, 0, 1)) if rng.random() < 0.5 else rand32(rng)
        res = rand32(rng)
        mem = bytearray(rng.randrange(256) for _ in range(0x40))
        mem[0:4] = struct.pack("<I", vt_base if base else vt_stub)
        mem[4:8] = struct.pack("<i", t)
        mem[0xC:0x10] = struct.pack("<i", res)
        uc.mem_write(rec, bytes(mem))
        h.w32(consts + PREMIUM, premium)
        before = bytes(uc.mem_read(rec, 0x60))
        h.calls.clear()
        r = h.call(ENTRY, (who,), ecx=rec)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(rec, 0x60)) == before, "wrote the record"
        calls = ";".join(",".join([name, "rec" if ecx == rec else "other", *map(str, args)])
                         for name, ecx, args, _ in h.calls)
        print(f"{base} {t} {ans[0]} {ans[1]} {who} {premium} {res} -> {r} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
