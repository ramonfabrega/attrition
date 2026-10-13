#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_get_packer_stance.py — `UnitData::get_packer_stance@00610970` under unicorn (item 2072, tools/sweep_gen.py, vcall --base).

    uv run tools/emu/sweep_unit_data_get_packer_stance.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 512-byte object of random bytes. It loads its own vtable slot +0x104 and compares
the pointer with the base implementation 0x0046ced0: equal, it calls slot +0x134 of the object at this+0x18; different, it calls the slot.
Both arms are handed [3]. A row's mode is 1 when the slot holds the base address (never entered), 0 when
it holds a stub; every called slot answers `v`, drawn per row.
Row: `<mode> <v> | <object hex> -> <eax> | <calls>`, a call `this|via,<args…>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00610970
BASE = 0x0046ced0
SLOT = 0x104
NARGS = 1
VIA = 24
VIA_SLOT = 308
WIN = 512
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(2072)
    vt = h.vtable({SLOT: ("slot", NARGS, 0)})
    stub = h.r32(vt + SLOT)
    inner = None
    if VIA is not None:
        ivt = h.vtable({VIA_SLOT: ("via", NARGS, 0)})
        inner = h.alloc(0x40)
        h.w32(inner, ivt)
    obj = h.alloc(WIN + 0x20)
    for i in range(240):
        mode = i % 2
        v = rand32(rng)
        h.calls.clear()
        h.answers["slot"] = h.answers["via"] = v
        h.w32(vt + SLOT, BASE if mode else stub)
        mem = bytearray(rng.randrange(256) for _ in range(WIN))
        mem[0:4] = vt.to_bytes(4, "little")
        if VIA is not None:
            mem[VIA:VIA + 4] = inner.to_bytes(4, "little")
        uc.mem_write(obj, bytes(mem))
        guard = bytes(uc.mem_read(obj + WIN, 0x20))
        r = h.call(ENTRY, (), ecx=obj)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(obj, WIN + 0x20)) == bytes(mem) + guard, "wrote the object"
        calls = ";".join(",".join([{obj: "this", inner: "via"}.get(ecx, "other"), *map(str, args)])
                         for _, ecx, args, _ in h.calls)
        print(f"{mode} {v} | {bytes(mem).hex()} -> {r} | {calls}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
