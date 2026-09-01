#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""callfn.py — call a function of the original's executable under unicorn.

    uv run tools/emu/callfn.py <install>/riseofnations.exe sweep [SEED] [N]

The dependency is declared inline (PEP 723), so `uv run` resolves it; nothing
else under tools/ needs a package beyond the standard library.

The image's sections are mapped at the preferred base (the executable is not
relocated in the bottle either: `tools/trace` assumes 0x400000 throughout),
no import is resolved, and a function is entered with a stack we build and
left at a sentinel return address. So a function that touches only its
arguments — or memory this script laid out first — runs; one that reaches a
DLL, a heap, or a singleton this script did not synthesize faults, and the
fault names the address it wanted, which is the cost of the next rung.

`sweep` prints one line per call, `<name> <args…> -> <eax>`, over a seeded
sweep of two pure functions — the fixture `crates/sim` reads back under
`RON_EMU_TABLE`:

    vector_dist@0046cff0      ecx=dx edx=dy                  (`world::vector_dist`)
    get_estimate@00688310     this, p1(x,y,_,_), p2(x,y,_,_), step   ret 0x24
                              (`path::…::estimate`)
"""
import random
import struct
import sys

from unicorn import UC_ARCH_X86, UC_HOOK_MEM_UNMAPPED, UC_MODE_32, Uc, UcError
from unicorn.x86_const import (UC_X86_REG_EAX, UC_X86_REG_ECX, UC_X86_REG_EDX,
                               UC_X86_REG_EIP, UC_X86_REG_ESP)

PAGE = 0x1000
STACK_BASE, STACK_SIZE = 0x7FF0_0000, 0x10_0000
SENTINEL = 0x7F00_0000


def align(v, a=PAGE):
    return (v + a - 1) & ~(a - 1)


class Image:
    """The executable's sections, mapped at their preferred base."""

    def __init__(self, path):
        b = open(path, "rb").read()
        pe = struct.unpack_from("<I", b, 0x3C)[0]
        assert b[pe:pe + 4] == b"PE\0\0", "not a PE image"
        nsec, opt_size = struct.unpack_from("<H", b, pe + 6)[0], struct.unpack_from("<H", b, pe + 20)[0]
        opt = pe + 24
        assert struct.unpack_from("<H", b, opt)[0] == 0x10B, "not PE32"
        self.base = struct.unpack_from("<I", b, opt + 28)[0]
        self.size = struct.unpack_from("<I", b, opt + 56)[0]
        self.sections = []
        for i in range(nsec):
            h = opt + opt_size + 40 * i
            name = b[h:h + 8].rstrip(b"\0").decode()
            vsize, va, rawsize, rawoff = struct.unpack_from("<IIII", b, h + 8)
            self.sections.append((name, va, vsize, b[rawoff:rawoff + rawsize]))

    def map_into(self, uc):
        uc.mem_map(self.base, align(self.size))
        for _, va, _, raw in self.sections:
            if raw:
                uc.mem_write(self.base + va, raw)


class Machine:
    def __init__(self, image):
        self.uc = Uc(UC_ARCH_X86, UC_MODE_32)
        image.map_into(self.uc)
        self.uc.mem_map(STACK_BASE, STACK_SIZE)
        self.uc.mem_map(SENTINEL, PAGE)
        self.fault = None
        self.uc.hook_add(UC_HOOK_MEM_UNMAPPED, self._on_unmapped)

    def _on_unmapped(self, uc, access, addr, size, value, _):
        self.fault = (access, addr, size, uc.reg_read(UC_X86_REG_EIP))
        return False

    def call(self, entry, stack_args=(), ecx=0, edx=0, max_insns=100_000):
        """Enter `entry` with `stack_args` pushed (first argument nearest the
        return address) and `ecx`/`edx` set; return `eax` at the sentinel."""
        uc = self.uc
        esp = STACK_BASE + STACK_SIZE - 0x100
        frame = struct.pack("<I", SENTINEL) + b"".join(struct.pack("<i", a) for a in stack_args)
        esp -= len(frame)
        uc.mem_write(esp, frame)
        uc.reg_write(UC_X86_REG_ESP, esp)
        uc.reg_write(UC_X86_REG_ECX, ecx & 0xFFFF_FFFF)
        uc.reg_write(UC_X86_REG_EDX, edx & 0xFFFF_FFFF)
        self.fault = None
        try:
            uc.emu_start(entry, SENTINEL, count=max_insns)
        except UcError as e:
            raise RuntimeError(f"{entry:08x}: {e}; fault={self.fault}") from e
        if uc.reg_read(UC_X86_REG_EIP) != SENTINEL:
            raise RuntimeError(f"{entry:08x}: did not return within {max_insns} instructions")
        return uc.reg_read(UC_X86_REG_EAX)


VECTOR_DIST = 0x0046CFF0
GET_ESTIMATE = 0x00688310
STEPS = (0x300, 0xC0, 0x30)


def signed(v):
    return v - (1 << 32) if v & 0x8000_0000 else v


def sweep(m, seed, n):
    rng = random.Random(seed)
    edges = [0, 1, 2, 3, 59_999, 60_000, 60_001, 46_340, 46_341, 65_535, 65_536, 100_000, 153_600, 200_000]
    pairs = [(a, b) for a in edges for b in edges]
    for _ in range(n):
        r = rng.random()
        lim = 3_000 if r < 0.4 else 200_000 if r < 0.9 else 2_000_000
        pairs.append((rng.randint(-lim, lim), rng.randint(-lim, lim)))
    for dx, dy in pairs:
        d = m.call(VECTOR_DIST, ecx=dx, edx=dy)
        print(f"vector_dist {dx} {dy} -> {signed(d)}")
    for dx, dy in pairs[:: max(1, len(pairs) // 400)]:
        for step in STEPS:
            x1, y1 = rng.randint(-100_000, 100_000), rng.randint(-100_000, 100_000)
            args = (x1, y1, 0, 0, x1 - dx, y1 - dy, 0, 0, step)
            e = m.call(GET_ESTIMATE, args, ecx=0)
            print(f"get_estimate {x1} {y1} {x1 - dx} {y1 - dy} {step} -> {signed(e)}")


def main(argv):
    if len(argv) < 3 or argv[2] != "sweep":
        print(__doc__)
        return 2
    seed = int(argv[3]) if len(argv) > 3 else 424242
    n = int(argv[4]) if len(argv) > 4 else 2000
    m = Machine(Image(argv[1]))
    sweep(m, seed, n)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
