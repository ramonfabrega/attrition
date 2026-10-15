#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_data_was_seen.py — `WorldData::was_seen@006b53f0` under unicorn (item 2683, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_world_data_was_seen.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 368-byte object of random bytes (no bias), 3 stack argument(s);
the global cell 0x00c061e8 points at 64 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x006edb50 (1 args) stubbed; the word at `m+0x134` points at 448 random bytes less `0` (placed); the word at `m+0x160` points at 64 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4, +0x125e:0x10, +0x12de:0x10, +0x59e4:0x4, +0x6929:0x1 (`p` windows after the pointed buffers, before the places).
One row per input, `<args… stubs…> | <this>;<c…>;<p…> -> <eax> | <this>;<c…>;<p…>` (hex); the stubs'
answers follow the arguments, and the test holds every byte of every window.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402
from sweeplib import link_value  # noqa: E402
from sweeplib import draw_reg  # noqa: E402

ENTRY = 0x006b53f0
WIN = 368
OWN = 3
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607976, 64)]
AT = []
PTRARGS = [(0, 4), (1, 4)]
PNULLS = []
STUBS = [('call', 7265104, 1)]
PLACES = [('m', '0x134', '0', 448), ('m', '0x160', '0', 64)]
NULLS = []
WBIAS = [('p0', 0, ('set', [0, 1, 2, 3, 5, 6, 7])), ('p1', 0, ('set', [0, 1, 2, 3, 5, 6, 7])), ('m', 0, ('set', [1, 2, 3, 4])), ('m', 12, ('set', [1, 3, 8])), ('c0', 48, ('set', [0, 0, 1, 2, 256])), ('p2', 0, ('set', [0, 0, 0, 0, 2048, 4096, 1, 6144])), ('p3', 0, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p4', 0, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p3', 4, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p4', 4, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p3', 8, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p4', 8, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p3', 12, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p4', 12, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p5', 0, ('set', [0, 0, 0, 0, 5])), ('p7', 0, ('set', [0, 0, 0, 0, 2048, 4096, 1, 6144])), ('p8', 0, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p9', 0, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p8', 4, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p9', 4, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p8', 8, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p9', 8, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p8', 12, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p9', 12, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p10', 0, ('set', [0, 0, 0, 0, 5])), ('p12', 0, ('set', [0, 0, 0, 0, 2048, 4096, 1, 6144])), ('p13', 0, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p14', 0, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p13', 4, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p14', 4, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p13', 8, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p14', 8, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p13', 12, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p14', 12, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p15', 0, ('set', [0, 0, 0, 0, 5])), ('p17', 0, ('set', [0, 0, 0, 0, 2048, 4096, 1, 6144])), ('p18', 0, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p19', 0, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p18', 4, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p19', 4, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p18', 8, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p19', 8, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p18', 12, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p19', 12, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p20', 0, ('set', [0, 0, 0, 0, 5])), ('p22', 0, ('set', [0, 0, 0, 0, 2048, 4096, 1, 6144])), ('p23', 0, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p24', 0, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p23', 4, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p24', 4, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p23', 8, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p24', 8, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p23', 12, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p24', 12, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p25', 0, ('set', [0, 0, 0, 0, 5])), ('p27', 0, ('set', [0, 0, 0, 0, 2048, 4096, 1, 6144])), ('p28', 0, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p29', 0, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p28', 4, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p29', 4, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p28', 8, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p29', 8, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p28', 12, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p29', 12, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p30', 0, ('set', [0, 0, 0, 0, 5])), ('p32', 0, ('set', [0, 0, 0, 0, 2048, 4096, 1, 6144])), ('p33', 0, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p34', 0, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p33', 4, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p34', 4, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p33', 8, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p34', 8, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p33', 12, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p34', 12, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p35', 0, ('set', [0, 0, 0, 0, 5])), ('p37', 0, ('set', [0, 0, 0, 0, 2048, 4096, 1, 6144])), ('p38', 0, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p39', 0, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p38', 4, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p39', 4, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p38', 8, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p39', 8, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p38', 12, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p39', 12, ('set', [0, 0, 0, 1, 65536, 0, 2])), ('p40', 0, ('set', [0, 0, 0, 0, 5])), ('p42', 4, ('set', [0, 1, 2, 3, 7])), ('p42', 12, ('set', [0, 0, 16777216, 33554432, 50331648, -16777216, 117440512])), ('p42', 32, ('set', [0, 1, 2, 3, 7])), ('p42', 40, ('set', [0, 0, 16777216, 33554432, 50331648, -16777216, 117440512])), ('p42', 60, ('set', [0, 1, 2, 3, 7])), ('p42', 68, ('set', [0, 0, 16777216, 33554432, 50331648, -16777216, 117440512])), ('p42', 88, ('set', [0, 1, 2, 3, 7])), ('p42', 96, ('set', [0, 0, 16777216, 33554432, 50331648, -16777216, 117440512])), ('p42', 116, ('set', [0, 1, 2, 3, 7])), ('p42', 124, ('set', [0, 0, 16777216, 33554432, 50331648, -16777216, 117440512])), ('p42', 144, ('set', [0, 1, 2, 3, 7])), ('p42', 152, ('set', [0, 0, 16777216, 33554432, 50331648, -16777216, 117440512])), ('p42', 172, ('set', [0, 1, 2, 3, 7])), ('p42', 180, ('set', [0, 0, 16777216, 33554432, 50331648, -16777216, 117440512])), ('p42', 200, ('set', [0, 1, 2, 3, 7])), ('p42', 208, ('set', [0, 0, 16777216, 33554432, 50331648, -16777216, 117440512])), ('p42', 228, ('set', [0, 1, 2, 3, 7])), ('p42', 236, ('set', [0, 0, 16777216, 33554432, 50331648, -16777216, 117440512])), ('p42', 256, ('set', [0, 1, 2, 3, 7])), ('p42', 264, ('set', [0, 0, 16777216, 33554432, 50331648, -16777216, 117440512])), ('p42', 284, ('set', [0, 1, 2, 3, 7])), ('p42', 292, ('set', [0, 0, 16777216, 33554432, 50331648, -16777216, 117440512])), ('p42', 312, ('set', [0, 1, 2, 3, 7])), ('p42', 320, ('set', [0, 0, 16777216, 33554432, 50331648, -16777216, 117440512])), ('p42', 340, ('set', [0, 1, 2, 3, 7])), ('p42', 348, ('set', [0, 0, 16777216, 33554432, 50331648, -16777216, 117440512])), ('p42', 368, ('set', [0, 1, 2, 3, 7])), ('p42', 376, ('set', [0, 0, 16777216, 33554432, 50331648, -16777216, 117440512])), ('p42', 396, ('set', [0, 1, 2, 3, 7])), ('p42', 404, ('set', [0, 0, 16777216, 33554432, 50331648, -16777216, 117440512])), ('p42', 424, ('set', [0, 1, 2, 3, 7])), ('p42', 432, ('set', [0, 0, 16777216, 33554432, 50331648, -16777216, 117440512]))]
FILLS = []
ARGRANGE = [(2, 0, 9)]
SLOTBASE = []
ECXINT = None
EDX = None
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [0, 1])]
VTAT = None
VTATHALF = []
ANSWERWIN = []
CDECL = False
ARGEQ = []
SEQ = []
ANSRANGE = []
CELLTHIS = []
VTALSO = []
SPARSE = [(None, '0xc061e0', 28396, 8, [(0, 4), (4702, 16), (4830, 16), (23012, 4), (26921, 1)])]
RECORD = [0]
RECORDORDER = False
RECORDEDX = False
ROWSPLIT = {0: 0}
LINKS = []
OUTARG = []
CDECLSTUBS = []
STUBWRITE = []
ECXANSWER = []
RING = []
DEREF = []
RET = 'eax'
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.5 else rng.randint(-0x80000000, 0x7FFFFFFF)


def i32(w, off):
    return int.from_bytes(w[off:off + 4], "little", signed=True)


def i16(w, off):
    return int.from_bytes(w[off:off + 2], "little", signed=True)


def i8(w, off):
    return int.from_bytes(w[off:off + 1], "little", signed=True)


def u16(w, off):
    return int.from_bytes(w[off:off + 2], "little")


def u8(w, off):
    return w[off]


def token(v, spans):
    """A recorded value: a pointer into a window by the window's name and
    offset (`m+0x0`, `p0+0x10`), a local by its depth (`sp-0x14`), anything
    else as a signed word (item 2281)."""
    v &= 0xFFFFFFFF
    for name, at, size in spans:
        if at <= v <= at + size:
            return f"{name}+{v - at:#x}"
    # A caller's local handed to a callee: its depth under the caller's
    # `esp`, which the harness holds every row (item 2336).
    top = STACK_BASE + STACK_SIZE - 0x100
    if top - 0x1000 <= v < top:
        return f"sp-{top - v:#x}"
    return str(v - (1 << 32) if v & 0x80000000 else v)


def by_row(calls, base, stride, n, spans):
    """`--record IDX@S` (item 2309): a stub's calls `(ecx, args)` split by
    the row of a sparse table their `ecx` lies in — one token a row, `ecx`
    written `r+OFF` within it, `-` for a row not called — and a last token
    for the calls whose `ecx` lies outside the table."""
    cols = [[] for _ in range(n + 1)]
    for ecx_, args_ in calls:
        k = ((ecx_ & 0xFFFFFFFF) - base) // stride
        if 0 <= k < n:
            head = f"r+{(ecx_ & 0xFFFFFFFF) - base - k * stride:#x}"
        else:
            k, head = n, token(ecx_, spans)
        cols[k].append(",".join([head] + [token(v, spans) for v in args_]))
    return ["/".join(col) or "-" for col in cols]


def in_order(calls, record, names, spans, split=None):
    """`--record-order` (item 2341): the recorded stubs' calls as one token
    a call, in the order they were made, each headed by the callee's stub
    index (`s3:m+0x0`), so two argument-less callees on one `ecx` are
    ordered (`add_think_order`, parked 2318); `-` when none was called.
    `split` maps a stub index to its `--sparse` table `(base, stride, n)`
    (`--record IDX@S`, item 2421): that stub's `ecx` is written `rK+OFF`,
    K the table row, when it lies inside the table."""
    want = {names[idx]: idx for idx in record}
    split = split or {}

    def head(i, e):
        if i in split:
            base, stride, n = split[i]
            k = ((e & 0xFFFFFFFF) - base) // stride
            if 0 <= k < n:
                return f"r{k}+{(e & 0xFFFFFFFF) - base - k * stride:#x}"
        return token(e, spans)
    return [f"s{want[n]}:" + ",".join([head(want[n], e)] + [token(v, spans) for v in a])
            for n, e, a in calls if n in want] or ["-"]


def out_answer(inner, k, v, write):
    """`--outarg IDX:K:VALS` (item 2325): a stub that writes `v` through its
    argument K — the address of the caller's local, as `get_inside` writes
    the player index — unless that argument is null, then answers as
    `inner` would (a value, or a function of `ecx` and the arguments)."""
    def answer(ecx, args):
        if args[k]:
            write(args[k] & 0xFFFFFFFF, v)
        return inner(ecx, args) if callable(inner) else inner
    return answer


def win_write(inner, at, size, v, write):
    """`--stub-write IDX:WIN+OFF:SIZE:VAL` (item 2346): a stub that writes VAL
    at a window's word each call — a callee that unlinks the head of the
    container its caller loops over (`Object::kill_contents`) — then answers
    as `inner` would."""
    def answer(ecx, args):
        write(at, (v & ((1 << 8 * size) - 1)).to_bytes(size, "little"))
        return inner(ecx, args) if callable(inner) else inner
    return answer


def read_arg(inner, k, n, read, sink, written=None):
    """`--deref IDX:K:N` (item 2379, parked 2348): a stub that reads the N
    bytes its argument K points at when it is called — a caller's local
    `Command`, gone once the call returns — into `sink` as hex (`null` for a
    null argument), then answers as `inner` would. `written` (item 2644,
    parked 2636) lists the byte indices the function wrote: every other byte
    of a local is stack residue and prints `00`."""
    def hexed(b):
        if written is not None:
            b = bytes(c if i in written else 0 for i, c in enumerate(b))
        return b.hex()

    def answer(ecx, args):
        if isinstance(k, tuple):  # `@VA` (item 2513): an absolute cell, read as it stands now
            sink.append(hexed(read(k[1], n)))
            return inner(ecx, args) if callable(inner) else inner
        a = args[k] & 0xFFFFFFFF
        sink.append(hexed(read(a, n)) if a else "null")
        return inner(ecx, args) if callable(inner) else inner
    return answer


def wdraw(rng, vals):
    """A bias's value: a set's member, or a range's — its own edges and the
    small integers inside it three rows in ten, so a boundary inside a wide
    range is drawn on purpose rather than by luck (Loop 2003, item 2181)."""
    if vals[0] == "set":
        return rng.choice(vals[1])
    lo, hi = vals[1], vals[2]
    if rng.random() < 0.3:
        return rng.choice([v for v in (lo, lo + 1, -1, 0, 1, 2, hi - 1, hi) if lo <= v <= hi])
    return rng.randint(lo, hi)


def seq_values(rng, n, pick, terms, rand32, span=None):
    """`--seq IDX:N[:T,T..]`'s answers a row: N drawn from the stub's
    `--answers` list (90%) or a random word, the last from the terminators
    T when given — answered past N, so a loop that runs while the stub
    answers inside a set ends (`Unit::repath`, item 2368, parked 2342)."""
    # `span` (`--answer-range` beside `--seq`, item 2421): every answer in LO..HI.
    vals = [rng.randint(*span) if span else rng.choice(pick) if pick and rng.random() < 0.9 else rand32(rng)
            for _ in range(n)]
    if terms:
        vals[-1] = rng.choice(terms)
    return vals


def shape(rng, name, buf):
    """The window `name`'s fills, then its biases, every row (parked 2113)."""
    for win, expr in FILLS:
        if win == name:
            for i in range(len(buf) // 4):
                buf[4 * i:4 * i + 4] = (eval(expr, {"i": i}) & 0xFFFFFFFF).to_bytes(4, "little")
    for win, off, vals in WBIAS:
        if win != name:
            continue
        for o in (range(0, len(buf) - 3, 4) if off is None else (off,)):
            v = wdraw(rng, vals)
            buf[o:o + 4] = (v & 0xFFFFFFFF).to_bytes(4, "little")


def main(argv):
    from hooks import Harness
    rng = random.Random(2683)
    # `--stub-cdecl IDX`: the callee's arguments are the caller's to pop (item 2325).
    h = Harness(argv[1], {a: (f"{a:x}", n, False, i in CDECLSTUBS) for i, (k, a, n) in enumerate(STUBS)
                          if k == "call"})
    uc = h.m.uc
    slots = {off: (f"slot{off:x}", n, 0) for k, off, n in STUBS if k == "slot"}
    vt = h.vtable(slots) if slots else None
    names = [f"slot{a:x}" if k == "slot" else f"iat{a:x}" if k == "iat" else f"{a:x}" for k, a, _ in STUBS]
    # `--stub iat:VA:N`: a stub whose address is laid in an import's IAT cell,
    # which the emulator resolves to nothing, so `call [VA]` lands on it
    # (item 2336: `timeGetTime`, parked 2327).
    iats = [(a, h.stub(f"iat{a:x}", n, 0)) for k, a, n in STUBS if k == "iat"]
    # `--stub-cdecl IDX` on an `iat` stub (`malloc`: `call [VA]; add esp, 4`).
    for (_, sa), (i, (k, _, _)) in zip(iats, [e for e in enumerate(STUBS) if e[1][0] == "iat"]):
        if i in CDECLSTUBS:
            h.hooked[sa] = h.hooked[sa][:3] + (True,)
    stubbed = {off: h.r32(vt + off) & 0xFFFFFFFF for off, _ in SLOTBASE}
    base = h.brk
    for i in range(220):
        h.brk = base  # the bump heap is reused a row; 220 rows of windows outgrow it
        this = h.alloc(WIN + 0x20)
        mem = bytearray(rng.randrange(256) for _ in range(WIN))
        for off, vals in BIAS:
            v = rng.choice(vals) if rng.random() < 0.9 else rng.randint(-0x80000000, 0x7FFFFFFF)
            mem[off:off + 4] = (v & 0xFFFFFFFF).to_bytes(4, "little")
        shape(rng, "m", mem)
        if vt is not None and VTAT is None:
            mem[0:4] = vt.to_bytes(4, "little")
        args = [rand32(rng) for _ in range(OWN)]
        for idx, vals in ARGSET:
            args[idx] = rng.choice(vals) if rng.random() < 0.9 else args[idx]
        for idx, off, size in ARGFROM:
            if rng.random() < 0.45:
                args[idx] = int.from_bytes(mem[off:off + size], "little", signed=True)
        for idx, lo, hi in ARGRANGE:
            args[idx] = rng.randint(lo, hi)
        wins = []
        for va, size in CELLS:
            rec = h.alloc(size + 0x20)
            buf = bytearray(rng.randrange(256) for _ in range(size))
            uc.mem_write(rec, bytes(buf))
            uc.mem_write(va, rec.to_bytes(4, "little"))
            wins.append((rec, buf))
        for va, size in AT:
            buf = bytearray(rng.randrange(256) for _ in range(size))
            uc.mem_write(va, bytes(buf))
            wins.append((va, buf))
        for idx, size in PTRARGS:
            buf = h.alloc(size + 0x20)
            b = bytearray(rng.randrange(256) for _ in range(size))
            uc.mem_write(buf, bytes(b))
            args[idx] = buf
            # `--ptrarg-null IDX`: the argument laid as 0 a third of the rows,
            # its buffer still drawn so every later window keeps its index
            # (item 2271; a guarded null argument ran one arm).
            if idx in PNULLS and rng.random() < 1 / 3:
                args[idx] = 0
            wins.append((buf, b))
        nc = len(CELLS) + len(AT)
        c = [b for _, b in wins[:nc]]
        p = [b for _, b in wins[nc:]]
        for k, b in enumerate(c):
            shape(rng, f"c{k}", b)
        for k, b in enumerate(p):
            shape(rng, f"p{k}", b)
        tables = []
        for win, loc, stride, n, slices in SPARSE:
            # A table of N rows STRIDE apart of which the function reads a
            # few slices: the table zeroed, each row's slices `p` windows, the
            # pointer laid at LOC — or, `@`, the slices written at the image's
            # own table (parked 2201, item 2247: the leaders table).
            env = {"a": args, "m": mem, "c": c, "p": p,
                   "i32": i32, "i16": i16, "i8": i8, "u16": u16, "u8": u8}
            if win == "@":
                tbase = eval(loc, env) & 0xFFFFFFFF
            else:
                tbase = h.alloc(stride * n + 0x20)
                uc.mem_write(tbase, bytes(stride * n))
            tables.append((tbase, stride, n))
            for k in range(n):
                for off, size in slices:
                    b = bytearray(rng.randrange(256) for _ in range(size))
                    p.append(b)
                    wins.append((tbase + k * stride + off, b))
                    shape(rng, f"p{len(p) - 1}", b)
            if win != "@":
                ptr = tbase.to_bytes(4, "little")
                if win is None:
                    uc.mem_write(eval(loc, env) & 0xFFFFFFFF, ptr)
                else:
                    w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
                    at = int(loc, 0)
                    w[at:at + 4] = ptr
        # The stubs' answers are drawn before the windows are placed, so a
        # placed table may be indexed by one (`s`, item 2160); `--answers`
        # draws stub IDX's from a list (parked 2141).
        answers = [rand32(rng) for _ in STUBS]
        for idx, vals in ANSWERS:
            answers[idx] = rng.choice(vals) if rng.random() < 0.9 else answers[idx]
        # `--answer-range`: an answer the function indexes a table by, held
        # to the range every row (item 2217).
        for idx, lo, hi in ANSRANGE:
            answers[idx] = rng.randint(lo, hi)
        # `--outarg`: the values stubs write through a pointer argument, drawn
        # with the answers so a place may index by one (`o`, item 2325).
        outs = [wdraw(rng, vals) for _, _, vals in OUTARG]
        k = -1
        for win, loc, expr, size in PLACES:
            # A table pointer laid as `buffer - (the offset the function adds)`,
            # so its read lands in a buffer of SIZE random bytes (parked 2113).
            k += 1
            env = {"a": args, "s": answers, "o": outs, "m": mem, "c": c, "p": p,
                   "i32": i32, "i16": i16, "i8": i8, "u16": u16, "u8": u8}
            off = eval(expr, env)
            buf = h.alloc(size + 0x20)
            b = bytearray(rng.randrange(256) for _ in range(size))
            p.append(b)
            wins.append((buf, b))
            shape(rng, f"p{len(p) - 1}", b)
            ptr = ((buf - off) & 0xFFFFFFFF).to_bytes(4, "little")
            # `--place-null K`: place K's pointer laid as 0 a third of the
            # rows, the buffer still drawn so every later window keeps its
            # index (item 2253; a guarded null ran one arm).
            if k in NULLS and rng.random() < 1 / 3:
                ptr = bytes(4)
            at = eval(loc, env)
            if win is None:
                uc.mem_write(at & 0xFFFFFFFF, ptr)
            elif win[0] == "s":
                # A stub answers the placed buffer, a record a later place
                # may reach into (item 2241).
                answers[int(win[1:])] = (buf - off) & 0xFFFFFFFF if any(ptr) else 0
            else:
                w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
                w[at:at + 4] = ptr
        callwins = {}
        for idx, size in ANSWERWIN:
            # A stub that answers a record: the address of SIZE random bytes,
            # a new `p` window after the places (parked 2151, item 2170).
            buf = h.alloc(size + 0x20)
            b = bytearray(rng.randrange(256) for _ in range(size))
            p.append(b)
            wins.append((buf, b))
            shape(rng, f"p{len(p) - 1}", b)
            answers[idx] = buf
            callwins.setdefault(idx, []).append(buf)
        if VTAT is not None:
            # The stubbed vtable laid in a window: a slot of an object the
            # function reaches through a table, not of `this` (item 2170).
            win, off = VTAT
            w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
            w[off:off + 4] = vt.to_bytes(4, "little")
        for win, off in VTALSO:
            # The stubbed vtable laid in a window as well, `this` keeping it:
            # a record the function reaches whose slots it asks too (item 2217).
            w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
            w[off:off + 4] = vt.to_bytes(4, "little")
        for win, off in VTATHALF:
            # The stubbed vtable laid half the rows at a record the function
            # compares with a class's vtable, the other half left as drawn
            # (a `--wbias` of the class's address; item 2223).
            if rng.random() < 0.5:
                w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
                w[off:off + 4] = vt.to_bytes(4, "little")
        for lo, hi, off, size, d, terms in LINKS:
            # A link field (`o_up`, a type's upgrade) of figures p(lo)..p(hi),
            # drawn from the figures on one side of each one's own index or a
            # terminator, so a walk through them ends (item 2276).
            n = hi - lo + 1
            for k in range(n):
                v = link_value(rng, n, k, d, terms)
                p[lo + k][off:off + size] = (v & ((1 << 8 * size) - 1)).to_bytes(size, "little")
        for idx, win, off in ARGEQ:
            # An argument the function compares with a word of a window laid
            # after the arguments were drawn: the word copied (item 2200).
            if rng.random() < 0.45:
                w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
                args[idx] = i32(w, off)
        # `--ring` (item 2400, parked 2369): a circular doubly linked list
        # laid in the `c` windows LO..HI at BASE + STRIDE k. The head is
        # the word at HWIN+HOFF; each node's `prev` is its predecessor on
        # the `next` walk from the head (the head's, the last node), so a
        # walk stopping at the head ends on the wrap; a fifth of the rows
        # then point one later node's `prev` at the head, an early stop.
        for hwin, hoff, lo_, hi_, noff, poff, base_, stride_ in RING:
            hw = mem if hwin == "m" else (c if hwin[0] == "c" else p)[int(hwin[1:])]
            head = i32(hw, hoff) & 0xFFFFFFFF
            node = lambda a_: (a_ - base_) // stride_
            if head and base_ <= head < base_ + stride_ * (hi_ - lo_ + 1) and (head - base_) % stride_ == 0:
                walk, seen = [node(head)], {node(head)}
                while True:
                    a_ = i32(c[lo_ + walk[-1]], noff) & 0xFFFFFFFF
                    if not (base_ <= a_ < base_ + stride_ * (hi_ - lo_ + 1)) or (a_ - base_) % stride_:
                        break
                    nxt = node(a_)
                    if nxt in seen:
                        break
                    walk.append(nxt)
                    seen.add(nxt)
                for j, k in enumerate(walk):
                    c[lo_ + k][poff:poff + 4] = (base_ + stride_ * walk[j - 1]).to_bytes(4, "little")
                if len(walk) > 1 and rng.random() < 0.2:
                    c[lo_ + rng.choice(walk[1:])][poff:poff + 4] = head.to_bytes(4, "little")
        for a_, b in wins:
            uc.mem_write(a_, bytes(b))
        for cell, stub in iats:
            uc.mem_write(cell, stub.to_bytes(4, "little"))
        based = []
        for off, addr in SLOTBASE:
            on = rng.random() < 0.5
            h.w32(vt + off, addr if on else stubbed[off])
            based.append(int(on))
        for name, v in zip(names, answers):
            h.answers[name] = v & 0xFFFFFFFF
        # A stub with several `--answer-win` windows answers one a call, in
        # call order, the last repeated past them (parked 2623, item 2633).
        for idx, bufs in callwins.items():
            if len(bufs) > 1:
                turn = [0]

                def answer(ecx, args, bufs=bufs, turn=turn):
                    v = bufs[min(turn[0], len(bufs) - 1)]
                    turn[0] += 1
                    return v
                h.answers[names[idx]] = answer
        # A callee called more than once a row answers each call in turn:
        # N values drawn a row (from its `--answers` list when it has one),
        # the last repeated past N — from the terminators when given (item
        # 2368) — printed after the answers (item 2217).
        seqs = []
        for idx, n, terms in SEQ:
            vals = seq_values(rng, n, dict(ANSWERS).get(idx), terms, rand32,
                              {i: (lo, hi) for i, lo, hi in ANSRANGE}.get(idx))
            seqs.append(vals)
            turn = [0]

            def answer(ecx, args, vals=vals, turn=turn):
                v = vals[min(turn[0], len(vals) - 1)]
                turn[0] += 1
                return v & 0xFFFFFFFF
            h.answers[names[idx]] = answer
        for (idx, k, _), v in zip(OUTARG, outs):
            h.answers[names[idx]] = out_answer(h.answers[names[idx]], k, v,
                                               lambda a_, v_: uc.mem_write(a_, (v_ & 0xFFFFFFFF).to_bytes(4, "little")))
        # `--ecx-answer IDX:OFF[:FLAGOFF]` (item 2400): a slot answering from
        # its own `ecx`: the word at `ecx+OFF`, or with FLAGOFF `ecx+OFF`
        # itself when the word at `ecx+FLAGOFF` is set, else null.
        for idx, off, flag in ECXANSWER:
            if flag is None:
                h.answers[names[idx]] = lambda ecx, args, off=off: h.r32(ecx + off) & 0xFFFFFFFF
            else:
                h.answers[names[idx]] = lambda ecx, args, off=off, flag=flag: (ecx + off) & 0xFFFFFFFF if h.r32(ecx + flag) else 0
        for idx, win, off, size, v in STUBWRITE:
            at = this if win == "m" else wins[(0 if win[0] == "c" else nc) + int(win[1:])][0]
            h.answers[names[idx]] = win_write(h.answers[names[idx]], at + off, size, v, uc.mem_write)
        derefs = [[] for _ in DEREF]
        for (idx, k, n, wr), sink in zip(DEREF, derefs):
            h.answers[names[idx]] = read_arg(h.answers[names[idx]], k, n,
                                             lambda a_, n_: bytes(uc.mem_read(a_, n_)), sink, wr)
        uc.mem_write(this, bytes(mem))
        # A global cell that holds `this` itself: a method that reads the
        # global as well as its object sees one object (item 2217).
        for va in CELLTHIS:
            uc.mem_write(va, this.to_bytes(4, "little"))
        guard = bytes(uc.mem_read(this + WIN, 0x20))
        # `ecx` an integer rather than the object (parked 2141), the row's last.
        ecx = None if ECXINT is None else draw_reg(rng, ECXINT, WILD, rand32)
        # `--edx` (item 2346): an integer in `edx` beside `ecx`, after it on the row.
        edx = None if EDX is None else draw_reg(rng, EDX, WILD, rand32)
        h.calls.clear()
        h.edxs.clear()
        r = h.call(ENTRY, tuple(args), ecx=this if ECXINT is None else ecx,
                   edx=0 if EDX is None else edx, ret=RET)  # Harness.call signs eax; signed() again would double a negative
        esp = uc.reg_read(UC_X86_REG_ESP)
        # A `__cdecl` function leaves its arguments for the caller (item 2181).
        want_esp = STACK_BASE + STACK_SIZE - 0x100 - (4 * len(args) if CDECL else 0)
        assert esp == want_esp, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(this + WIN, 0x20)) == guard, "wrote past the window"
        before = ";".join([bytes(mem).hex()] + [bytes(b).hex() for _, b in wins])
        after = ";".join([bytes(uc.mem_read(this, WIN)).hex()]
                         + [bytes(uc.mem_read(a, len(b))).hex() for a, b in wins])
        shown = ([str(a) for a in args] + [str(v) for v in answers] + [str(v) for v in outs]
                 + [str(v) for s in seqs for v in s] + [str(f) for f in based]
                 + ([] if ECXINT is None else [str(ecx)]) + ([] if EDX is None else [str(edx)]))
        # `--record IDX`: what stub IDX was handed, each call `ecx,arg..`,
        # calls joined by `/`, `-` when it was not called, after ` # `
        # (item 2281: a tail target's `ecx` and argument, parked 2272).
        log = ""
        if RECORD or DEREF:
            # `this` past the window (`this + 0x28`, a member's address) is last,
            # so it never shadows a window the heap laid after the object (item 2464).
            spans = [("m", this, WIN)] + [(f"c{k}" if k < nc else f"p{k - nc}", a_, len(b))
                                          for k, (a_, b) in enumerate(wins)] + [("this", this, 0x1000)]
            cols = []
            if RECORDORDER:
                cols = in_order([(n_, e_, a_) for n_, e_, a_, _ in h.calls], RECORD, names, spans,
                                {i: tables[t] for i, t in ROWSPLIT.items()})
            for idx in ([] if RECORDORDER else RECORD):
                calls = [(ecx_, list(args_)) for name_, ecx_, args_, _ in h.calls if name_ == names[idx]]
                if RECORDEDX:
                    # `--record-edx` (item 2400): `ecx,edx,arg..` a call.
                    cols.append("/".join(",".join(token(v, spans) for v in [e, d] + list(a_))
                                         for (n_, e, a_, _), d in zip(h.calls, h.edxs) if n_ == names[idx]) or "-")
                elif idx in ROWSPLIT:
                    cols += by_row(calls, *tables[ROWSPLIT[idx]], spans)
                else:
                    cols.append("/".join(",".join(token(v, spans) for v in [e] + a_)
                                         for e, a_ in calls) or "-")
            # `--deref`: the bytes each call's argument pointed at (item 2379).
            cols += ["/".join(sink) or "-" for sink in derefs]
            log = " # " + " ".join(cols)
        print(" ".join(shown) + f" | {before} -> {r} | {after}" + log)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

# sweep_gen: ["obj", "WorldData::was_seen@006b53f0", "--item", "2683", "--win", "0x170", "--own", "3", "--cell-win", "0xc061e8:0x40", "--ptrarg", "0:4", "--ptrarg", "1:4", "--argrange", "2:0:9", "--sparse", "0xc061e0:0x6eec:8:0,4/0x125e,16/0x12de,16/0x59e4,4/0x6929,1", "--place", "m+0x134:0:448", "--place", "m+0x160:0:64", "--wbias", "p0+0:0,1,2,3,5,6,7", "--wbias", "p1+0:0,1,2,3,5,6,7", "--wbias", "m+0:1,2,3,4", "--wbias", "m+0xc:1,3,8", "--wbias", "c0+0x30:0,0,1,2,256", "--wbias", "p2+0:0,0,0,0,2048,4096,1,6144", "--wbias", "p3+0:0,0,0,1,65536,0,2", "--wbias", "p4+0:0,0,0,1,65536,0,2", "--wbias", "p3+4:0,0,0,1,65536,0,2", "--wbias", "p4+4:0,0,0,1,65536,0,2", "--wbias", "p3+8:0,0,0,1,65536,0,2", "--wbias", "p4+8:0,0,0,1,65536,0,2", "--wbias", "p3+12:0,0,0,1,65536,0,2", "--wbias", "p4+12:0,0,0,1,65536,0,2", "--wbias", "p5+0:0,0,0,0,5", "--wbias", "p7+0:0,0,0,0,2048,4096,1,6144", "--wbias", "p8+0:0,0,0,1,65536,0,2", "--wbias", "p9+0:0,0,0,1,65536,0,2", "--wbias", "p8+4:0,0,0,1,65536,0,2", "--wbias", "p9+4:0,0,0,1,65536,0,2", "--wbias", "p8+8:0,0,0,1,65536,0,2", "--wbias", "p9+8:0,0,0,1,65536,0,2", "--wbias", "p8+12:0,0,0,1,65536,0,2", "--wbias", "p9+12:0,0,0,1,65536,0,2", "--wbias", "p10+0:0,0,0,0,5", "--wbias", "p12+0:0,0,0,0,2048,4096,1,6144", "--wbias", "p13+0:0,0,0,1,65536,0,2", "--wbias", "p14+0:0,0,0,1,65536,0,2", "--wbias", "p13+4:0,0,0,1,65536,0,2", "--wbias", "p14+4:0,0,0,1,65536,0,2", "--wbias", "p13+8:0,0,0,1,65536,0,2", "--wbias", "p14+8:0,0,0,1,65536,0,2", "--wbias", "p13+12:0,0,0,1,65536,0,2", "--wbias", "p14+12:0,0,0,1,65536,0,2", "--wbias", "p15+0:0,0,0,0,5", "--wbias", "p17+0:0,0,0,0,2048,4096,1,6144", "--wbias", "p18+0:0,0,0,1,65536,0,2", "--wbias", "p19+0:0,0,0,1,65536,0,2", "--wbias", "p18+4:0,0,0,1,65536,0,2", "--wbias", "p19+4:0,0,0,1,65536,0,2", "--wbias", "p18+8:0,0,0,1,65536,0,2", "--wbias", "p19+8:0,0,0,1,65536,0,2", "--wbias", "p18+12:0,0,0,1,65536,0,2", "--wbias", "p19+12:0,0,0,1,65536,0,2", "--wbias", "p20+0:0,0,0,0,5", "--wbias", "p22+0:0,0,0,0,2048,4096,1,6144", "--wbias", "p23+0:0,0,0,1,65536,0,2", "--wbias", "p24+0:0,0,0,1,65536,0,2", "--wbias", "p23+4:0,0,0,1,65536,0,2", "--wbias", "p24+4:0,0,0,1,65536,0,2", "--wbias", "p23+8:0,0,0,1,65536,0,2", "--wbias", "p24+8:0,0,0,1,65536,0,2", "--wbias", "p23+12:0,0,0,1,65536,0,2", "--wbias", "p24+12:0,0,0,1,65536,0,2", "--wbias", "p25+0:0,0,0,0,5", "--wbias", "p27+0:0,0,0,0,2048,4096,1,6144", "--wbias", "p28+0:0,0,0,1,65536,0,2", "--wbias", "p29+0:0,0,0,1,65536,0,2", "--wbias", "p28+4:0,0,0,1,65536,0,2", "--wbias", "p29+4:0,0,0,1,65536,0,2", "--wbias", "p28+8:0,0,0,1,65536,0,2", "--wbias", "p29+8:0,0,0,1,65536,0,2", "--wbias", "p28+12:0,0,0,1,65536,0,2", "--wbias", "p29+12:0,0,0,1,65536,0,2", "--wbias", "p30+0:0,0,0,0,5", "--wbias", "p32+0:0,0,0,0,2048,4096,1,6144", "--wbias", "p33+0:0,0,0,1,65536,0,2", "--wbias", "p34+0:0,0,0,1,65536,0,2", "--wbias", "p33+4:0,0,0,1,65536,0,2", "--wbias", "p34+4:0,0,0,1,65536,0,2", "--wbias", "p33+8:0,0,0,1,65536,0,2", "--wbias", "p34+8:0,0,0,1,65536,0,2", "--wbias", "p33+12:0,0,0,1,65536,0,2", "--wbias", "p34+12:0,0,0,1,65536,0,2", "--wbias", "p35+0:0,0,0,0,5", "--wbias", "p37+0:0,0,0,0,2048,4096,1,6144", "--wbias", "p38+0:0,0,0,1,65536,0,2", "--wbias", "p39+0:0,0,0,1,65536,0,2", "--wbias", "p38+4:0,0,0,1,65536,0,2", "--wbias", "p39+4:0,0,0,1,65536,0,2", "--wbias", "p38+8:0,0,0,1,65536,0,2", "--wbias", "p39+8:0,0,0,1,65536,0,2", "--wbias", "p38+12:0,0,0,1,65536,0,2", "--wbias", "p39+12:0,0,0,1,65536,0,2", "--wbias", "p40+0:0,0,0,0,5", "--wbias", "p42+4:0,1,2,3,7", "--wbias", "p42+12:0,0,16777216,33554432,50331648,-16777216,117440512", "--wbias", "p42+32:0,1,2,3,7", "--wbias", "p42+40:0,0,16777216,33554432,50331648,-16777216,117440512", "--wbias", "p42+60:0,1,2,3,7", "--wbias", "p42+68:0,0,16777216,33554432,50331648,-16777216,117440512", "--wbias", "p42+88:0,1,2,3,7", "--wbias", "p42+96:0,0,16777216,33554432,50331648,-16777216,117440512", "--wbias", "p42+116:0,1,2,3,7", "--wbias", "p42+124:0,0,16777216,33554432,50331648,-16777216,117440512", "--wbias", "p42+144:0,1,2,3,7", "--wbias", "p42+152:0,0,16777216,33554432,50331648,-16777216,117440512", "--wbias", "p42+172:0,1,2,3,7", "--wbias", "p42+180:0,0,16777216,33554432,50331648,-16777216,117440512", "--wbias", "p42+200:0,1,2,3,7", "--wbias", "p42+208:0,0,16777216,33554432,50331648,-16777216,117440512", "--wbias", "p42+228:0,1,2,3,7", "--wbias", "p42+236:0,0,16777216,33554432,50331648,-16777216,117440512", "--wbias", "p42+256:0,1,2,3,7", "--wbias", "p42+264:0,0,16777216,33554432,50331648,-16777216,117440512", "--wbias", "p42+284:0,1,2,3,7", "--wbias", "p42+292:0,0,16777216,33554432,50331648,-16777216,117440512", "--wbias", "p42+312:0,1,2,3,7", "--wbias", "p42+320:0,0,16777216,33554432,50331648,-16777216,117440512", "--wbias", "p42+340:0,1,2,3,7", "--wbias", "p42+348:0,0,16777216,33554432,50331648,-16777216,117440512", "--wbias", "p42+368:0,1,2,3,7", "--wbias", "p42+376:0,0,16777216,33554432,50331648,-16777216,117440512", "--wbias", "p42+396:0,1,2,3,7", "--wbias", "p42+404:0,0,16777216,33554432,50331648,-16777216,117440512", "--wbias", "p42+424:0,1,2,3,7", "--wbias", "p42+432:0,0,16777216,33554432,50331648,-16777216,117440512", "--stub", "0x6edb50:1", "--answers", "0:0,1", "--record", "0@0", "--port", "{\n    let lead = a[2];\n    let b = 2 + 5 * lead.clamp(0, 7) as usize;\n    let (px, py) = (rd(&p[0], 0), rd(&p[1], 0));\n    let flags = rd(&p[b], 0);\n    let sh = |w: &[u8], i: usize| i16::from_le_bytes([w[2 * i], w[2 * i + 1]]);\n    let mut call: Option<String> = None;\n    let r = if lead >= 8 || rd(&c[0], 0x30) as u8 > 1 || flags & 0x1000 != 0 || flags & 0x800 != 0 || sh(&p[b + 3], 0) != 0 {\n        1\n    } else {\n        let xs = rd(m, 0);\n        let idx = (py >> 1).wrapping_mul(xs).wrapping_add(px >> 1) as usize;\n        let ent = &p[42][28 * idx..28 * idx + 28];\n        let who = ent[0xf] as i8 as i32;\n        let region = i16::from_le_bytes([ent[4], ent[5]]) as usize;\n        let mut hit = false;\n        if who >= 0 {\n            call = Some(format!(\"r+0x0,{who}\"));\n            if a[3] != 0 {\n                let wb = 2 + 5 * who as usize;\n                hit = sh(&p[wb + 1], region) != 0 || sh(&p[wb + 2], region) != 0;\n            }\n        }\n        if hit {\n            1\n        } else {\n            let fog = rd(m, 0xc);\n            i64::from(p[43][fog.wrapping_mul(py).wrapping_add(px) as usize] & p[b + 4][0])\n        }\n    };\n    for k in 0..8 {\n        log.push(match &call {\n            Some(t) if k == lead => t.clone(),\n            _ => \"-\".into(),\n        });\n    }\n    log.push(\"-\".into());\n    r\n}"]
