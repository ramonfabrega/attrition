#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_type_data_get_kill_value.py — `BuildTypeData::get_kill_value@00639790` under unicorn (item 2112,
hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_build_type_data_get_kill_value.py <install>/riseofnations.exe

`__thiscall`, one stack argument `who`, `ret 4`. It first asks its own slot `+0x7c` handed `0` (a stub answered
per row: the base value). A negative `who` answers that. Otherwise, with `t` the type at `+0x4`, the two words
`num_buildings[t - 0x19e]` (`[0xc061e0] + 0x6eec · who + 2t + 0x5222`) and `num_queued[t]` (`+ 0x5a22`), unsigned,
are summed and multiply `([+0x274] + [+0x270]) · BUILD_SUPPORT_FACTOR` (`[0xc061e4] + 0x37c`), and the base is
added — all 32-bit, wrapping. The leader cell is chosen a row so the two words land in a buffer of the script's,
so `who` and `t` are drawn freely.
Row: `<who> <t> <s270> <s274> <factor> <built> <queued> <base> -> <eax> | <calls>`, a call `base,this|other,<arg>`;
the type's 0x300 bytes, the constants' 0x400 and the buffer's 0x810 are random and held unwritten.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00639790
CONSTANTS = 0x00C061E4
LEADERS = 0x00C061E0
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, 0x10000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2112)
    vt = h.vtable({0x7C: ("base", 1, 0)})
    this = h.alloc(0x300)
    consts = h.alloc(0x400)
    buf = h.alloc(0x810)
    for i in range(400):
        who = rng.randint(0, 7) if rng.random() < 0.6 else rand32(rng)
        t = rng.randint(0x19E, 0x21F) if rng.random() < 0.6 else rand32(rng)
        s270, s274, factor, base = rand32(rng), rand32(rng), rand32(rng), rand32(rng)
        built, queued = rng.randrange(0x10000), rng.randrange(0x10000)
        h.answers["base"] = base
        mem = bytearray(rng.randrange(256) for _ in range(0x300))
        mem[0:4] = struct.pack("<I", vt)
        mem[4:8] = struct.pack("<i", t)
        mem[0x270:0x274] = struct.pack("<i", s270)
        mem[0x274:0x278] = struct.pack("<i", s274)
        cmem = bytearray(rng.randrange(256) for _ in range(0x400))
        cmem[0x37C:0x380] = struct.pack("<i", factor)
        bmem = bytearray(rng.randrange(256) for _ in range(0x810))
        bmem[0:2] = struct.pack("<H", built)
        bmem[0x800:0x802] = struct.pack("<H", queued)
        leaders = (buf - 0x5222 - 2 * (who * 0x3776 + t)) & 0xFFFFFFFF
        uc.mem_write(this, bytes(mem))
        uc.mem_write(consts, bytes(cmem))
        uc.mem_write(buf, bytes(bmem))
        uc.mem_write(CONSTANTS, struct.pack("<I", consts))
        uc.mem_write(LEADERS, struct.pack("<I", leaders))
        held = ((this, 0x300), (consts, 0x400), (buf, 0x810))
        before = b"".join(bytes(uc.mem_read(a, n)) for a, n in held)
        h.calls.clear()
        r = h.call(ENTRY, (who,), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert b"".join(bytes(uc.mem_read(a, n)) for a, n in held) == before, "wrote memory"
        calls = ";".join(",".join([name, "this" if ecx == this else "other", *map(str, args)])
                         for name, ecx, args, _ in h.calls)
        print(f"{who} {t} {s270} {s274} {factor} {built} {queued} {base} -> {r} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
