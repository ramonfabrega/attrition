#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_wall_data_can_capture.py — `WallData::can_capture@00471360` under unicorn (item 2112, tools/sweep_gen.py, vcall --base --base-call).

    uv run tools/emu/sweep_wall_data_can_capture.py <install>/riseofnations.exe

`__thiscall`, 0 stack argument(s) of its own. The object at `this+0x18` has its vtable's slot +0x64 compared
with the base implementation 0x00472bc0: equal, it calls that object's own slot +0x60 handed ['414', '0']
(`pN` the function's own argument N); different, it jumps into slot +0x64 with its own arguments, whose
answer is the answer. A row's mode is 1 when the slot holds the base address (never entered), 0 when it holds a
stub; both slots answer `v`, drawn per row, zero a quarter of the time.
Row: `<mode> <v> <own…> -> <eax> | <calls>`, a call `slot|base,this|rec|other,<args…>`; the 512 bytes of
`this` (and the 0x40 of the record) are random and held unwritten.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00471360
BASE = 0x00472bc0
SLOT = 0x64
BSLOT = 0x60
OWN = 0
HANDED = ['414', '0']
VIA = 24
WIN = 512
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2112)
    vt = h.vtable({SLOT: ("slot", OWN, 0), BSLOT: ("base", len(HANDED), 0)})
    stub = h.r32(vt + SLOT)
    obj = h.alloc(WIN + 0x20)
    rec = h.alloc(0x60) if VIA is not None else obj
    for i in range(240):
        mode = i % 2
        v = 0 if rng.random() < 0.25 else rand32(rng)
        own = [rand32(rng) for _ in range(OWN)]
        h.calls.clear()
        h.answers["slot"] = h.answers["base"] = v
        h.w32(vt + SLOT, BASE if mode else stub)
        mem = bytearray(rng.randrange(256) for _ in range(WIN))
        if VIA is None:
            mem[0:4] = vt.to_bytes(4, "little")
        else:
            mem[VIA:VIA + 4] = rec.to_bytes(4, "little")
            rmem = bytearray(rng.randrange(256) for _ in range(0x40))
            rmem[0:4] = vt.to_bytes(4, "little")
            uc.mem_write(rec, bytes(rmem))
        uc.mem_write(obj, bytes(mem))
        before = bytes(uc.mem_read(obj, WIN + 0x20)) + bytes(uc.mem_read(rec, 0x40))
        r = h.call(ENTRY, own, ecx=obj)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(obj, WIN + 0x20)) + bytes(uc.mem_read(rec, 0x40)) == before, "wrote memory"
        who = {obj: "this", rec: "rec"} if VIA is not None else {obj: "this"}
        calls = ";".join(",".join([name, who.get(ecx, "other"), *map(str, args)]) for name, ecx, args, _ in h.calls)
        print(" ".join(map(str, [mode, v, *own])) + f" -> {r} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
