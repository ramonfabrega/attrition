#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_data_is_gathered_by.py — `BuildData::is_gathered_by@0062f520` under unicorn (item 2038,
hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_build_data_is_gathered_by.py <install>/riseofnations.exe

`__thiscall` with one stack argument (the unit index), `ret 4`. It walks the building's gatherer chain: the
head is the signed short `gather_down` at `+0x70`, each link the signed short at `+0x92` of the unit record the
owner's table names (`0xc0aec0 + who*0x1c`, `who` the byte at `+0x9`), and the walk ends at a negative link.
The answer is 1 when the argument is met on the chain, else 0. A row lays 32 unit records; a random
permutation's prefix is the chain (so it ends), the rest link anywhere.
Row: `<who> <head> <unit> <next>×32 -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def popped(uc):
    esp = uc.reg_read(UC_X86_REG_ESP)
    assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"


ENTRY = 0x0062F520
TABLE = 0x00C0AEC0
N = 32


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2038)
    slots = h.alloc(N * 4)
    recs = [h.alloc(0xA0) for _ in range(N)]
    for k, r in enumerate(recs):
        h.w32(slots + 4 * k, r)
    this = h.alloc(0x100)
    for i in range(300):
        who = rng.randrange(9)
        saved = h.r32(TABLE + who * 0x1C)
        h.w32(TABLE + who * 0x1C, slots)
        order = list(range(N))
        rng.shuffle(order)
        length = rng.randrange(N + 1)
        nxt = [rng.choice((-1, -2, -0x8000)) if rng.random() < 0.3 else rng.randrange(N) for _ in range(N)]
        chain = order[:length]
        for a, b in zip(chain, chain[1:]):
            nxt[a] = b
        if chain:
            nxt[chain[-1]] = rng.choice((-1, -2, -0x8000, -100))
        head = chain[0] if chain else rng.choice((-1, -2, -0x8000))
        for r, v in zip(recs, nxt):
            uc.mem_write(r, bytes(rng.randrange(256) for _ in range(0xA0)))
            h.w16(r + 0x92, v)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x100)))
        h.w8(this + 0x9, who)
        h.w16(this + 0x70, head)
        roll = rng.random()
        if roll < 0.45 and chain:
            unit = rng.choice(chain)
        elif roll < 0.8:
            unit = rng.randrange(N)
        else:
            unit = rng.choice((-1, -2, N, 0x7FFF, 0x10000)) if rng.random() < 0.7 else rand32(rng)
        before = bytes(uc.mem_read(this, 0x100)), [bytes(uc.mem_read(r, 0xA0)) for r in recs]
        res = h.call(ENTRY, (unit,), ecx=this)
        popped(uc)
        assert (bytes(uc.mem_read(this, 0x100)), [bytes(uc.mem_read(r, 0xA0)) for r in recs]) == before, "wrote"
        h.w32(TABLE + who * 0x1C, saved)
        print(" ".join(str(x) for x in (who, head, unit, *nxt)) + f" -> {res}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
