#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_type_data_time.py — `TypeData::time@00663f20` under unicorn (item 2090, hand-written on tools/emu/hooks.py;
`sweep_type_data_research_time.py` runs its sibling on the same layout).

    uv run tools/emu/sweep_type_data_time.py <install>/riseofnations.exe

`__thiscall`, one stack argument (`who`), `ret 4`. The record's slot `+0x48` against `TypeData::is_spell_type`
(`0x470590`): equal, the type index at `+4` in `0x275..0x2ac` (signed) is a spell; else the slot is called and
its answer says. A spell asks `LeaderData::has_spell@006e0bc0` (hooked, one argument: the type) on
`[0xc061e0] + 0x6eec · who` and costs `job_time` (`+8`) `· 100` when held and `res_time` (`+0xc`) `· 100` when
not; anything else costs `job_time · 100`. The multiplies wrap (`imul`).
Row: `<base 0|1> <type> <slot answer> <has_spell answer> <who> <leaders> <job_time> <res_time> -> <eax> | <calls>`,
a call `name,ecx,<args…>`; the record's 0x40 bytes are random and held unwritten.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00663F20
IS_SPELL_TYPE = 0x00470590
HAS_SPELL = 0x006E0BC0
LEADERS = 0x00C061E0
I32 = (0, 1, -1, 2, 0x7FFFFFFF, -0x80000000, 0x100, 30, 45, 600, 0x1000000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.5 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {HAS_SPELL: ("has_spell", 1)})
    uc = h.m.uc
    rng = random.Random(2090)
    ans = [0]
    vt_stub = h.vtable({0x48: ("is_spell_type", 0, lambda ecx, args: ans[0])})
    vt_base = h.alloc(0x200)
    h.w32(vt_base + 0x48, IS_SPELL_TYPE)
    rec = h.alloc(0x60)
    for i in range(400):
        base = i % 2
        t = rng.randint(0x270, 0x2B0) if rng.random() < 0.7 else rand32(rng)
        ans[0] = 0 if rng.random() < 0.4 else rand32(rng)
        held = 0 if rng.random() < 0.5 else rand32(rng)
        h.answers["has_spell"] = held
        who = rng.randint(-2, 9) if rng.random() < 0.8 else rand32(rng)
        leaders = rand32(rng) & 0xFFFFFFFF
        job, res = rand32(rng), rand32(rng)
        mem = bytearray(rng.randrange(256) for _ in range(0x40))
        mem[0:16] = struct.pack("<Iiii", vt_base if base else vt_stub, t, job, res)
        uc.mem_write(rec, bytes(mem))
        uc.mem_write(LEADERS, struct.pack("<I", leaders))
        before = bytes(uc.mem_read(rec, 0x60))
        h.calls.clear()
        r = h.call(ENTRY, (who,), ecx=rec)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(rec, 0x60)) == before, "wrote the record"
        calls = ";".join(",".join([name, "rec" if ecx == rec else str(ecx & 0xFFFFFFFF), *map(str, args)])
                         for name, ecx, args, _ in h.calls)
        print(f"{base} {t} {ans[0]} {held} {who} {leaders} {job} {res} -> {r} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
