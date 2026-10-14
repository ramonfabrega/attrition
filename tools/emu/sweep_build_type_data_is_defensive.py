#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_type_data_is_defensive.py — `BuildTypeData::is_defensive@00473600` under unicorn (item 2083,
hand-written on tools/emu/hooks.py; `sweep_wall_data_is_defensive.py` runs the same body through `+0x18`).

    uv run tools/emu/sweep_build_type_data_is_defensive.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`. It asks its own vtable slot `+0x60` (`TypeData::is`, two stack
arguments) `(0x1b7, 0)`, `(0x1bb, 0)` and `(0x1bf, 0)` in turn, answering 1 at the first non-zero; with all three
zero it answers `attack` (`+0x1e8`) `!= 0`. The slot is a stub whose answer is drawn per row and per type index
(zero half the time, else an edge or a random word), so the order and the short circuit are what the calls show.
Row: `<a 0x1b7> <a 0x1bb> <a 0x1bf> <attack> -> <eax> | <calls>`, a call `this|other,<args…>`; the record's
0x300 bytes are random and held unwritten.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00473600
IS = 0x60
ATTACK = 0x1E8
REC = 0x300
ASKED = (0x1B7, 0x1BB, 0x1BF)
I32 = (1, -1, 2, 0x7FFFFFFF, -0x80000000, 0x100, 0x10000)


def word(rng):
    if rng.random() < 0.5:
        return 0
    return rng.choice(I32) if rng.random() < 0.5 else rng.randint(-0x80000000, 0x7FFFFFFF)


def run(argv, entry, via, seed):
    """`via`: None, the record is `this`; else the offset of `this` that holds the record's pointer."""
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(seed)
    answers = {}
    vt = h.vtable({IS: ("is", 2, lambda ecx, args: answers.get(args[0], 0))})
    rec = h.alloc(REC + 0x20)
    wall = h.alloc(0x100 + 0x20) if via is not None else None
    for i in range(400):
        answers.clear()
        for t in ASKED:
            answers[t] = word(rng)
        attack = word(rng)
        mem = bytearray(rng.randrange(256) for _ in range(REC))
        mem[0:4] = vt.to_bytes(4, "little")
        mem[ATTACK:ATTACK + 4] = (attack & 0xFFFFFFFF).to_bytes(4, "little")
        uc.mem_write(rec, bytes(mem))
        before = bytes(uc.mem_read(rec, REC + 0x20))
        this = rec
        if wall is not None:
            wmem = bytearray(rng.randrange(256) for _ in range(0x100))
            wmem[via:via + 4] = rec.to_bytes(4, "little")
            uc.mem_write(wall, bytes(wmem))
            wbefore = bytes(uc.mem_read(wall, 0x120))
            this = wall
        h.calls.clear()
        r = h.call(entry, (), ecx=this)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(rec, REC + 0x20)) == before, "wrote the record"
        if wall is not None:
            assert bytes(uc.mem_read(wall, 0x120)) == wbefore, "wrote the wall"
        calls = ";".join(",".join(["this" if ecx == rec else "other", *map(str, args)])
                         for _, ecx, args, _ in h.calls)
        print(" ".join(str(answers[t]) for t in ASKED) + f" {attack} -> {r} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(run(sys.argv, ENTRY, None, 2083))
