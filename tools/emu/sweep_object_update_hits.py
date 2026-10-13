#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_update_hits.py — `Object::update_hits@00647010` under unicorn (item 2044, hand-written on
tools/emu/hooks.py).

    uv run tools/emu/sweep_object_update_hits.py <install>/riseofnations.exe

`__thiscall` with one stack argument never read, `ret 4`. The unit's type index `t` is `[[this+0x18]+4]`; the
type list (the cell `0xe85ddc` names an array of record pointers) gives `t`'s kind at `+4`. A kind of `0x32` or
`0x33` reads the leader's byte at `0xe40fb0 + who*0x6eec` (`who` the byte at `this+9`): bit `0x10` takes index
`0x44`, else `8` takes `0x43`, else `4` takes `0x42`, else `t` stands. The answer is that index's record of the
object-type table (the cell `0xc061f8`, its `+0x10` an array of record pointers) at `+0x210`, stored at
`this+0x20` too. A row draws `t` 0..0x4f, its kind often 0x32/0x33, `who` 0..7 and the leader byte.
Row: `<t> <kind> <who> <leader byte> <hits[t]> <hits[0x42]> <hits[0x43]> <hits[0x44]> <p0> -> <eax> | <this+0x20>`.
"""
import os
import random
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


ENTRY = 0x00647010
TYPES, OBJTYPES = 0x00E85DDC, 0x00C061F8
LEADER_BITS = 0x00E40FB0
N = 0x50


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2044)
    saved = {a: bytes(uc.mem_read(a, 4)) for a in (TYPES, OBJTYPES)}
    tlist = h.alloc(4 * N)
    trecs = [h.alloc(8) for _ in range(N)]
    olist = h.alloc(4 * N)
    orecs = [h.alloc(0x214) for _ in range(N)]
    ot = h.alloc(0x20)
    for k in range(N):
        h.w32(tlist + 4 * k, trecs[k])
        h.w32(olist + 4 * k, orecs[k])
    h.w32(TYPES, tlist)
    h.w32(OBJTYPES, ot)
    h.w32(ot + 0x10, olist)
    this = h.alloc(0x40)
    rec = h.alloc(0x10)
    for _ in range(300):
        t = rng.randrange(N)
        kind = rng.choice((0x32, 0x33)) if rng.random() < 0.6 else rng.choice((0x31, 0x34, t, 0x42, 0)) \
            if rng.random() < 0.7 else rand32(rng)
        h.w32(trecs[t] + 4, kind)
        hits = {k: rand32(rng) for k in (t, 0x42, 0x43, 0x44)}
        for k, v in hits.items():
            h.w32(orecs[k] + 0x210, v)
        who = rng.randrange(8)
        bits = rng.choice((0, 4, 8, 0x10, 0xC, 0x14, 0x18, 0x1C)) | (rng.randrange(256) & 0xE3 if rng.random() < 0.5 else 0)
        saved_bit = bytes(uc.mem_read(LEADER_BITS + who * 0x6EEC, 1))
        h.w8(LEADER_BITS + who * 0x6EEC, bits)
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x40)))
        h.w32(this + 0x18, rec)
        h.w8(this + 0x9, who)
        uc.mem_write(rec, bytes(rng.randrange(256) for _ in range(0x10)))
        h.w32(rec + 4, t)
        p0 = rand32(rng)
        before = bytearray(uc.mem_read(this, 0x40))
        res = h.call(ENTRY, (p0,), ecx=this)
        popped(uc)
        after = bytearray(uc.mem_read(this, 0x40))
        stored = h.r32(this + 0x20)
        before[0x20:0x24] = after[0x20:0x24]
        assert before == after, "wrote outside this+0x20"
        uc.mem_write(LEADER_BITS + who * 0x6EEC, saved_bit)
        print(f"{t} {kind} {who} {bits} {hits[t]} {hits[0x42]} {hits[0x43]} {hits[0x44]} {p0} -> {res} | {stored}")
    for a, v in saved.items():
        uc.mem_write(a, v)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
