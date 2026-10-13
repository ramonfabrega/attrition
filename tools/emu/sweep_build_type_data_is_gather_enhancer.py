#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_type_data_is_gather_enhancer.py — `BuildTypeData::is_gather_enhancer@00472b50` under unicorn
(item 2090, hand-written on tools/emu/hooks.py; 2083's `sweep_build_type_data_is_defensive.py` without the attack word).

    uv run tools/emu/sweep_build_type_data_is_gather_enhancer.py <install>/riseofnations.exe

`__thiscall`, no stack arguments, plain `ret`. It asks its own vtable slot `+0x60` (`TypeData::is`, two stack
arguments) `(0x1a7, 0)`, `(0x1a8, 0)`, `(0x1a9, 0)` and `(0x1aa, 0)` in turn, answering 1 at the first non-zero
and 0 past the fourth. The slot is a stub answered per row and per type index (zero half the time), so the order
and the short circuit are what the calls show.
Row: `<a 0x1a7> <a 0x1a8> <a 0x1a9> <a 0x1aa> -> <eax> | <calls>`, a call `this|other,<args…>`; the record's
0x100 bytes are random and held unwritten.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00472B50
IS = 0x60
REC = 0x100
ASKED = (0x1A7, 0x1A8, 0x1A9, 0x1AA)
I32 = (1, -1, 2, 0x7FFFFFFF, -0x80000000, 0x100, 0x10000)


def word(rng):
    if rng.random() < 0.5:
        return 0
    return rng.choice(I32) if rng.random() < 0.5 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2090)
    answers = {}
    vt = h.vtable({IS: ("is", 2, lambda ecx, args: answers.get(args[0], 0))})
    rec = h.alloc(REC + 0x20)
    for i in range(400):
        answers.clear()
        for t in ASKED:
            answers[t] = word(rng)
        mem = bytearray(rng.randrange(256) for _ in range(REC))
        mem[0:4] = vt.to_bytes(4, "little")
        uc.mem_write(rec, bytes(mem))
        before = bytes(uc.mem_read(rec, REC + 0x20))
        h.calls.clear()
        r = h.call(ENTRY, (), ecx=rec)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(rec, REC + 0x20)) == before, "wrote the record"
        calls = ";".join(",".join(["this" if ecx == rec else "other", *map(str, args)])
                         for _, ecx, args, _ in h.calls)
        print(" ".join(str(answers[t]) for t in ASKED) + f" -> {r} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
