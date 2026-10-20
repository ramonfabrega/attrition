#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_balance_compute_modifier.py — `Balance::compute_modifier@00581cc0` under unicorn (item 3372).

    uv run tools/emu/sweep_balance_compute_modifier.py <install>/riseofnations.exe

Hand-written beside tools/sweep_gen.py, not generated: the generator has no flag for the three things
this function wants (parked 3282c, 3337a, 3372). Each is built here from the generator's own parts:

* TLS statics: `--tib`'s guest TIB gets `fs:[0x2c]` -> an array -> a block whose `+8` is 0, so the two
  `_TSS` guards (both 0 in the image) read "already initialised" and the function-local `rows`/`cols`
  `SimpleArray<int>` statics (0x00ee1418, 0x00ee1438) are the rows' own, laid as the constructor leaves
  them (vtable 0xb22a60, cap 5 or another, a malloc'd buffer). Growth runs the original's own
  `increase_size` through the vtable, its `malloc`/`free` imports and `memcpy` stubbed.
* `return_pack@005821c0` is a stub writing a drawn 5-word pack through its hidden out pointer
  (`args[0]`), answering that pointer; call 0 is the attacker's type, call 1 the target's.
* `type_damage@0057fb50` is a stub answering a drawn word (it has its own sweep, item 3362).
* `types.list` (0x00e85ddc) points at four type records, each with a vtable whose slot +0x18 is the
  image's own `TypeData::is_gaia_type` (reads the record's `+4`) or a stub answering the record's `+8`.
* the 399 x 399 `short` table (argument 2) is 318 KB; a row prints only the cells it fills (the rest
  are 0): the rows x cols the packs name, and their +-1 neighbours as decoys for a wrong stride.

One row per input:
`ta tb | (mode id word) x4 | td | pa(ty line obj age flags) pb | R n e.. C n e.. | idx:val,.. -> eax | R n e.. C n e..`
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sweeplib import TIB, set_tib  # noqa: E402

ENTRY = 0x00581CC0
RET_PACK, TYPE_DAMAGE, MEMCPY = 0x005821C0, 0x0057FB50, 0x0055E0AC
TYPES_LIST, IS_GAIA, ARRAY_VT = 0x00E85DDC, 0x004707A0, 0x00B22A60
ROWS_AT, COLS_AT = 0x00EE1418, 0x00EE1438
IAT_MALLOC, IAT_FREE = 0x00AC54F0, 0x00AC5500
N = 399
TABLE = 2 * N * N
ROWS = 600


def rd32(b, off):
    return int.from_bytes(b[off:off + 4], "little", signed=True)


def draw_pack(rng):
    def pick(neg_p, common, wide_max):
        r = rng.random()
        if r < neg_p:
            return -1 if rng.random() < 0.7 else -rng.randint(2, 0x7FFFFFFF)
        return rng.randint(0, common) if rng.random() < 0.75 else rng.randint(0, wide_max)

    ty = pick(0.3, 40, N - 1)
    line = pick(0.4, 4, 0x26)
    obj = pick(0.3, 1, 0x21)
    age = pick(0.3, 7, 0x1F)
    r = rng.random()
    if r < 0.2:
        flags = 0
    elif r < 0.45:
        flags = 0
        for _ in range(rng.randint(1, 3)):
            flags |= 1 << rng.randint(0, 31)
    elif r < 0.6:
        flags = 0xFFFFFFFF
    elif r < 0.7:
        flags = 1 << rng.randint(0, 31)
    else:
        p = rng.choice([0.1, 0.3, 0.5, 0.9])
        flags = sum(1 << b for b in range(32) if rng.random() < p)
    return (ty, line, obj, age, flags)


def candidates(p):
    """Every category row/column a pack could name (a superset), for filling the table's cells."""
    ty, line, obj, age, flags = p
    out = [(ty, 0), (line, 0x160), (obj, 0x165), (age, 0x167)]
    out = [b + v for v, b in out if v >= 0]
    return out + [0x16F + b for b in range(32) if flags >> b & 1]


def main(argv):
    from hooks import Harness
    rng = random.Random(3372)
    h = Harness(argv[1], {RET_PACK: ("retpack", 2, False, False), TYPE_DAMAGE: ("typedmg", 2, False, False),
                          MEMCPY: ("memcpy", 3, False, True)})
    uc = h.m.uc
    set_tib(uc)
    uc.mem_write(TIB + 0x2C, (TIB + 0x100).to_bytes(4, "little"))
    uc.mem_write(TIB + 0x100, (TIB + 0x200).to_bytes(4, "little"))
    uc.mem_write(TIB + 0x208, bytes(4))
    # `malloc` and `free` imports (`call [cell]; add esp, 4`: cdecl) and `memcpy`.
    for cell, name, fn in ((IAT_MALLOC, "malloc", lambda ecx, a: h.alloc(max(a[0] & 0xFFFFFFFF, 16))),
                           (IAT_FREE, "free", lambda ecx, a: 0)):
        sa = h.stub(name, 1, fn)
        h.hooked[sa] = h.hooked[sa][:3] + (True,)
        uc.mem_write(cell, sa.to_bytes(4, "little"))

    def memcpy(ecx, a):
        dst, src, n = (v & 0xFFFFFFFF for v in a[:3])
        uc.mem_write(dst, bytes(uc.mem_read(src, n)))
        return dst
    h.answers["memcpy"] = memcpy
    vt_native = h.alloc(0x200)
    h.w32(vt_native + 0x18, IS_GAIA)
    vt_stub = h.vtable({0x18: ("gaia", 0, lambda ecx, a: 1 if h.r32(ecx + 8) else 0)})
    table = h.alloc(TABLE + 0x40)
    base = h.brk
    packs = [None, None]
    state = {"calls": 0, "td": 0, "asked": None}

    def retpack(ecx, a):
        k = state["calls"]
        state["calls"] += 1
        state["order"].append(a[1])
        uc.mem_write(a[0] & 0xFFFFFFFF, struct.pack("<iiiiI", *packs[k]))
        return a[0]
    h.answers["retpack"] = retpack

    def typedmg(ecx, a):
        state["asked"] = tuple(a)
        return state["td"]
    h.answers["typedmg"] = typedmg
    for _ in range(ROWS):
        h.brk = base
        ids = []
        recs = []
        for i in range(4):
            mode = rng.randint(0, 1)
            r = rng.random()
            idv = (rng.choice([0x191, 0x192, 0x193, 0x19D, 0x19E, 0x19F]) if r < 0.2 else
                   rng.randint(0, 0x300) if r < 0.9 else rng.randint(-0x80000000, 0x7FFFFFFF))
            if rng.random() < 0.04:
                idv = rng.randint(0x192, 0x19D)
            word = rng.choice([0, 0, 0, 0, 0, 0, 0, 1, 2, -1])
            rec = h.alloc(0x20)
            h.w32(rec, vt_native if mode == 0 else vt_stub)
            h.w32(rec + 4, idv)
            h.w32(rec + 8, word)
            recs.append(rec)
            ids.append((mode, idv, word))
        arr = h.alloc(16)
        for i, rec in enumerate(recs):
            h.w32(arr + 4 * i, rec)
        h.w32(TYPES_LIST, arr)
        ta, tb = rng.randint(0, 3), rng.randint(0, 3)
        td = rng.choice([100, 0, 1, -100, 250, 1075, 100, 100]) if rng.random() < 0.6 else rng.randint(-0x80000000, 0x7FFFFFFF)
        pa, pb = draw_pack(rng), draw_pack(rng)
        packs[0], packs[1] = pa, pb
        # The table: cells filled for the candidates (and decoy neighbours); the rest 0.
        rs, cs = set(candidates(pa)), set(candidates(pb))
        rx = {r + d for r in rs for d in (-1, 0, 1) if 0 <= r + d < N}
        cx = {c + d for c in cs for d in (-1, 0, 1) if 0 <= c + d < N}
        mode = rng.randint(0, 3)
        tbl = bytearray(TABLE)
        cells = []
        for r in sorted(rx):
            for c in sorted(cx):
                v = (rng.randint(80, 120) if mode == 0 else 100 if mode == 3 else
                     (0 if rng.random() < 0.1 else rng.randint(-60, 300)) if mode == 1 else rng.randint(-0x8000, 0x7FFF))
                tbl[2 * (r * N + c):2 * (r * N + c) + 2] = struct.pack("<h", v)
                cells.append(f"{r * N + c}:{v}")
        uc.mem_write(table, bytes(tbl))
        # The two statics, as the constructor leaves them: count drawn, cap 5 or another, a buffer.
        init = []
        for at in (ROWS_AT, COLS_AT):
            cap = rng.choice([5, 5, 5, 5, 0, 1, 2, 3, 4, 8, 40])
            cnt = rng.randint(0, cap)
            buf = h.alloc(4 * max(cap, 1) + 0x10) if cap else 0
            ents = [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(cnt)]
            for i, e in enumerate(ents):
                h.w32(buf + 4 * i, e)
            h.w32(at, ARRAY_VT)
            h.w32(at + 4, cnt)
            h.w32(at + 8, cap)
            h.w32(at + 0xC, 0xFFFF)  # the short -1 the constructor writes, a zero short above it
            h.w32(at + 0x10, buf)
            uc.mem_write(at + 0x14, bytes(4))
            init.append((cnt, ents))
        state.update(calls=0, td=td, asked=None, order=[])
        h.calls.clear()
        eax = h.call(ENTRY, (ta, tb, table), ecx=0, max_insns=2_000_000)
        def is_gaia(t):
            m, i, x = ids[t]
            return (0x192 <= i < 0x19E) if m == 0 else x != 0
        gaia = is_gaia(ta) or is_gaia(tb)
        if gaia:
            assert state["calls"] == 0 and state["asked"] is None, "a gaia type reached the tables"
        else:
            assert state["order"] == [ta, tb], f"return_pack asked for {state['order']}"
            assert state["asked"] == (ta, tb), f"type_damage asked {state['asked']}"
        after = []
        for at in (ROWS_AT, COLS_AT):
            cnt = h.r32(at + 4)
            buf = h.r32(at + 0x10) & 0xFFFFFFFF
            after.append((cnt, [h.r32(buf + 4 * i) for i in range(cnt)]))

        def fmt(tag, s):
            return f"{tag} {s[0]} " + " ".join(map(str, s[1]))
        w = " ".join(f"{m} {i} {x}" for m, i, x in ids)
        print(f"{ta} {tb} | {w} | {td} | {' '.join(map(str, pa))} {' '.join(map(str, pb))} | "
              f"{fmt('R', init[0])} {fmt('C', init[1])} | {','.join(cells) or '-'} -> {eax} | "
              f"{fmt('R', after[0])} {fmt('C', after[1])}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
