#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_player_is_active_connection.py — `Player::is_active_connection@004b71d0` under unicorn (item 1966, tools/sweep_gen.py, obj).

    uv run tools/emu/sweep_player_is_active_connection.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 256-byte object of random bytes (+0x30 in [0, 1, 257, 209, 65, 2, 128, 17, 16, 4294901761, 196609, 7], +0x7c in [0, 0, 5, -1]), 0 stack argument(s).
One row per input, `<args…> | <before hex> -> <eax> | <after hex>`; the test holds every byte.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE, signed  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x004b71d0
WIN = 256
OWN = 0
BIAS = [(48, [0, 1, 257, 209, 65, 2, 128, 17, 16, 4294901761, 196609, 7]), (124, [0, 0, 5, -1])]
ARGSET = []
ARGFROM = []
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def main(argv):
    from hooks import Harness
    rng = random.Random(1966)
    h = Harness(argv[1], {})
    for i in range(220):
        this = h.alloc(WIN + 0x20)
        mem = bytearray(rng.randrange(256) for _ in range(WIN))
        for off, vals in BIAS:
            v = rng.choice(vals) if rng.random() < 0.9 else rng.randint(-0x80000000, 0x7FFFFFFF)
            mem[off:off + 4] = (v & 0xFFFFFFFF).to_bytes(4, "little")
        args = [rng.choice(I32) if rng.random() < 0.5 else rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(OWN)]
        for idx, vals in ARGSET:
            args[idx] = rng.choice(vals) if rng.random() < 0.9 else args[idx]
        for idx, off, size in ARGFROM:
            if rng.random() < 0.45:
                args[idx] = int.from_bytes(mem[off:off + size], "little", signed=True)
        h.m.uc.mem_write(this, bytes(mem))
        guard = bytes(h.m.uc.mem_read(this + WIN, 0x20))
        r = signed(h.call(ENTRY, tuple(args), ecx=this))
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {esp:x}: wrong pop"
        assert bytes(h.m.uc.mem_read(this + WIN, 0x20)) == guard, "wrote past the window"
        after = bytes(h.m.uc.mem_read(this, WIN))
        print(" ".join(str(a) for a in args) + f" | {bytes(mem).hex()} -> {r} | {after.hex()}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
