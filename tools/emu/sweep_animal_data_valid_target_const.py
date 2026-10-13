#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_animal_data_valid_target_const.py — `AnimalData::valid_target_const@005d8120` under unicorn (item 1998,
hand-written on tools/emu/hooks.py).

    uv run tools/emu/sweep_animal_data_valid_target_const.py <install>/riseofnations.exe

`__thiscall` (`ecx` unread), stack `o`, `who`, `ret 8`. The decompile prints an empty `if` and `return 0`; the
listing reads on: when both are non-negative it answers `byte [object + 8] & 1`, the object being
`objects.list[who][o]` — the per-player pointer at `0xc0ab84 + who * 0x1c` (`GameAccess::objects`' `+0x14`),
indexed by `o`. The rows hold `who` to 0..8 (the table's players) and `o` to 0..63; a negative `who` or `o`
answers 0 without a read (the table is left unset on those rows).
Row: `<o> <who> <byte> -> <eax>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x005D8120
TABLE = 0x00C0AB84


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {})
    uc = h.m.uc
    rng = random.Random(1998)
    lists = [h.alloc(4 * 64) for _ in range(9)]
    objs = h.alloc(0x40 * 64)
    for who in range(9):
        uc.mem_write(TABLE + who * 0x1C, struct.pack("<I", lists[who]))
    rows = []
    for o, who in ((-1, 0), (0, -1), (-1, -1), (-0x80000000, 3), (5, -0x80000000)):
        rows.append((o, who, 0xFF))
    for i in range(400):
        rows.append((rng.randrange(64), rng.randrange(9), rng.choice((0, 1, 2, 3, 0xFE, 0xFF)) if i % 2 else rng.randrange(256)))
    for o, who, b in rows:
        if o >= 0 and who >= 0:
            obj = objs + 0x40 * rng.randrange(64)
            uc.mem_write(obj, bytes(rng.randrange(256) for _ in range(0x40)))
            h.w8(obj + 8, b)
            h.w32(lists[who] + 4 * o, obj)
        r = h.call(ENTRY, (o, who), ecx=rng.randrange(1 << 32))
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        print(f"{o} {who} {b} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
