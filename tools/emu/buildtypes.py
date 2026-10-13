"""buildtypes.py — the `buildtypes` table, laid out for the `BuildTypeData`
functions that walk a type's `FROM` chain through it (item 2072:
`fundamental_type`, `is_military_trainer`, `is_training_building`;
hand-written on `tools/emu/hooks.py`).

The global cell `0xc0aaa0` holds the list of type-record pointers, indexed
by `TypeIndex`: a record's `+0x4` is its own index, `+0x3c` its `FROM`
parent's (negative for a root), `+0x2c0` the flag word the trainers mask.
`basic_type` (0x639970) is the slot `+0xe4` every record's vtable holds; it
recurses through the parent's own slot, so the world runs the original's
walk rather than a stub of it.

`World(h, rng).lay()` writes a fresh forest of `N` records a row — a parent
always an earlier index, so no chain cycles — and returns `(froms, flags)`;
`restore()` puts the cell back. `check()` asserts no record byte moved.
"""
import struct

CELL = 0x00C0AAA0
BASIC_TYPE = 0x00639970
N = 10
REC = 0x300
SLOT = 0xE4


class World:
    def __init__(self, h, rng):
        self.h, self.rng = h, rng
        self.table = h.alloc(4 * N)
        self.recs = [h.alloc(REC) for _ in range(N)]
        self.vt = h.alloc(0x200)
        h.w32(self.vt + SLOT, BASIC_TYPE)
        self.saved = h.r32(CELL)
        h.w32(CELL, self.table)
        for i, r in enumerate(self.recs):
            h.w32(self.table + 4 * i, r)

    def lay(self):
        h, uc, rng = self.h, self.h.m.uc, self.rng
        froms, flags = [], []
        for i, r in enumerate(self.recs):
            uc.mem_write(r, bytes(rng.randrange(256) for _ in range(REC)))
            h.w32(r, self.vt)
            h.w32(r + 4, i)
            f = rng.randrange(i) if i and rng.random() < 0.65 else rng.choice((-1, -2, -0x80000000, -0x100))
            h.w32(r + 0x3C, f & 0xFFFFFFFF)
            w = rng.getrandbits(32)
            if rng.random() < 0.5:
                w ^= rng.choice((0x40000000, 0x80000000, 0xC0000000))
            h.w32(r + 0x2C0, w)
            froms.append(f)
            flags.append(w - (1 << 32) if w & 0x80000000 else w)
        self.before = [bytes(uc.mem_read(r, REC)) for r in self.recs]
        return froms, flags

    def check(self):
        uc = self.h.m.uc
        assert [bytes(uc.mem_read(r, REC)) for r in self.recs] == self.before, "wrote a record"

    def restore(self):
        self.h.w32(CELL, self.saved)
