#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_on_duty.py — `Unit::on_duty@005fff70` under unicorn (item 2044, hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_unit_on_duty.py <install>/riseofnations.exe

`__thiscall`, no stack argument, `ret`. The type record at `this+0x18` gates on bit `0x10000` of its word at
`+0x2c8`; past the gate the callee at `0x608370` (hooked, no argument, `ecx` = this) answers an order or null,
and the order's vtable slot `+0x10` (hooked, `ecx` = the order) answers its kind. A row draws the flag word, a
null or a real order, and the kind from every value 0..0x20 and a few words.
Row: `<flag word> <order?> <kind> -> <eax> | <calls>`, the calls `<name>:<ecx is the expected object>` joined
by `,` (empty when none ran).
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


ENTRY = 0x005FFF70
ACTIVITY = 0x00608370


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {ACTIVITY: ("activity", 0)})
    uc = h.m.uc
    rng = random.Random(2044)
    this = h.alloc(0x40)
    ty = h.alloc(0x300)
    order = h.alloc(0x40)
    kind = [0]
    vt = h.vtable({0x10: ("kind", 0, lambda ecx, args: kind[0])})
    for _ in range(300):
        uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x40)))
        h.w32(this + 0x18, ty)
        uc.mem_write(ty, bytes(rng.randrange(256) for _ in range(0x300)))
        flags = rand32(rng)
        if rng.random() < 0.7:
            flags |= 0x10000
        elif rng.random() < 0.5:
            flags &= ~0x10000
        h.w32(ty + 0x2C8, flags)
        uc.mem_write(order, bytes(rng.randrange(256) for _ in range(0x40)))
        h.w32(order, vt)
        has = rng.random() < 0.85
        h.answers["activity"] = order if has else 0
        kind[0] = rng.randrange(0x20) if rng.random() < 0.85 else rand32(rng) & 0xFFFFFFFF
        before = bytes(uc.mem_read(this, 0x40)), bytes(uc.mem_read(ty, 0x300)), bytes(uc.mem_read(order, 0x40))
        h.calls.clear()
        res = h.call(ENTRY, (), ecx=this)
        popped(uc)
        assert (bytes(uc.mem_read(this, 0x40)), bytes(uc.mem_read(ty, 0x300)),
                bytes(uc.mem_read(order, 0x40))) == before, "wrote"
        want = {"activity": this, "kind": order}
        calls = ",".join(f"{name}:{int(ecx == want[name])}" for name, ecx, _, _ in h.calls)
        k = kind[0] - (1 << 32) if kind[0] & 0x80000000 else kind[0]
        print(f"{flags} {int(has)} {k} -> {res} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
