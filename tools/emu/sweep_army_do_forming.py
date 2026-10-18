#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_army_do_forming.py — `Army::do_forming@006f43c0` under unicorn (item 3250, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_army_do_forming.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 152-byte object of random bytes (no bias), 1 stack argument(s);
`p` windows in order: p0..p5 sparse table 0 (2 rows × 3 slice(s), row-major), p6..p17 the place(s), in flag order, p18 stub 5's answer window; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00e85f20 points at 10064 random bytes; the global cell 0x00c061e0 points at 57344 random bytes; callee 0x006f56d0 (0 args) stubbed; callee 0x006f9120 (2 args) stubbed; callee 0x006f8750 (1 args) stubbed; callee 0x0065d260 (9 args) stubbed; callee 0x006edb50 (1 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x0070fba0 (10 args) stubbed; callee 0x0070d830 (5 args) stubbed; callee 0x0092cf40 (4 args) stubbed; callee 0x00a46a00 (0 args) stubbed; the word at `c0+0x14+0x1c*(m[0x34]&3)` points at 20 random bytes less `0` (placed); the word at `p6+0` points at 32 random bytes less `0` (placed); the word at `p6+4` points at 32 random bytes less `0` (placed); the word at `p6+8` points at 32 random bytes less `0` (placed); the word at `p6+12` points at 32 random bytes less `0` (placed); the word at `p6+16` points at 32 random bytes less `0` (placed); the word at `0xc09970+0x1c*(m[0x34]&3)` points at 12 random bytes less `0` (placed); the word at `p12+0` points at 16 random bytes less `0` (placed); the word at `p12+4` points at 16 random bytes less `0` (placed); the word at `p12+8` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 240 random bytes less `0` (placed); the word at `p5+0` points at 240 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 0, 0, 1] (90%); stub 1 answered from [0, 0, 0, 1, 2] (90%); stub 3 answered from [-1, -1, 0, 1, 5] (90%); stub 4 answered from [0, 1] (90%); stub 5 answers the address of 128 random bytes, a `p` window after the places; stub 9 answers 8 values in call order, printed after the answers; stub 9 answers -3000..3000 every row; stub 2 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 8 writes a value in -300000..300000 through its argument 2, printed after the answers (`o` in a place); stub 8 writes a value in -300000..300000 through its argument 3, printed after the answers (`o` in a place); stub 8 is `__cdecl`: the caller pops its arguments; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x8:0x4, +0x74:0xc, +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`.
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
from sweeplib import draw_reg, xmm_bits, set_tib  # noqa: E402

ENTRY = 0x006f43c0
WIN = 152
OWN = 1
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607884, 128), (15228704, 10064), (12607968, 57344)]
AT = []
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('call', 7296720, 0), ('call', 7311648, 2), ('call', 7309136, 1), ('call', 6672992, 9), ('call', 7265104, 1), ('slot', 172, 0), ('call', 7404448, 10), ('call', 7395376, 5), ('call', 9621312, 4), ('call', 10775040, 0)]
PLACES = [('c0', '0x14+0x1c*(m[0x34]&3)', '0', 20), ('p6', '0', '0', 32), ('p6', '4', '0', 32), ('p6', '8', '0', 32), ('p6', '12', '0', 32), ('p6', '16', '0', 32), (None, '0xc09970+0x1c*(m[0x34]&3)', '0', 12), ('p12', '0', '0', 16), ('p12', '4', '0', 16), ('p12', '8', '0', 16), ('p2', '0', '0', 240), ('p5', '0', '0', 240)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 148, ('set', [0, 65536, 131072, 196608, 262144, 1, 65537, 131073, 196609, 262145, 4294901760])), ('m', 44, ('set', [0, 0, 1])), ('m', 72, ('set', [0, 1, 5, 29, 58, -2, 100])), ('m', 76, ('set', [0, 1, 7, 22, 29, -3, 90])), ('m', 80, ('set', [0, 16777216, 1073741824, 2147483647, -2147483648, -1073741824, 536870912, 305419896, -19088743, 89478485, -2147483647, 1073741823])), ('m', 84, ('set', [-1, 0, 1, 2, 3])), ('m', 88, ('set', [-1, 0, 1, 2, 3])), ('m', 92, ('set', [-1, 0, 1, 2, 3])), ('m', 96, ('set', [-1, 0, 1, 2, 3])), ('c1', 12, ('set', [0, 1, 1, 1])), ('c1', 2528, ('set', [0, 1, 1, 1])), ('c1', 5044, ('set', [0, 1, 1, 1])), ('c1', 7560, ('set', [0, 1, 1, 1])), ('c2', None, ('set', [0, 1])), ('p0', 0, ('set', [0, 1])), ('p3', 0, ('set', [0, 1])), ('p1', None, ('set', [0, 1])), ('p4', None, ('set', [0, 1])), ('p16', 232, ('set', [405895, 405894, 405893, 405892, 405891, 405888, 0, 5, -1, 2147889543, 405767])), ('p17', 232, ('set', [405895, 405894, 405893, 405892, 405891, 405888, 0, 5, -1, 2147889543, 405767])), ('p18', 112, ('set', [0, 65536, 131072, -65536])), ('p13', 4, ('set', [1, 1, 1, 3, 0, 2])), ('p14', 4, ('set', [1, 1, 1, 3, 0, 2])), ('p15', 4, ('set', [1, 1, 1, 3, 0, 2])), ('m', 52, ('set', [0, 1, 2])), ('m', 48, ('set', [-1, 0, 1, 2, 3, 4]))]
FILLS = []
ARGRANGE = []
SLOTBASE = []
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [0, 0, 0, 0, 0, 1]), (1, [0, 0, 0, 1, 2]), (3, [-1, -1, 0, 1, 5]), (4, [0, 1])]
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = []
ANSWERBY = []
ANSWERWIN = [(5, 128)]
CDECL = False
TIB = False
ARGEQ = []
SEQ = [(9, 8, None)]
ANSRANGE = [(9, -3000, 3000)]
CELLTHIS = []
VTALSO = [('p7', 0), ('p8', 0), ('p9', 0), ('p10', 0), ('p11', 0)]
SPARSE = [('@', '0xe3a390', 28396, 2, [(8, 4), (116, 12), (28344, 4)])]
GTAB = []
RECORD = [2, 3, 4, 6, 7, 8, 9]
RECORDORDER = False
RECORDEDX = True
ROWSPLIT = {}
LINKS = []
LINKHI = []
OUTARG = [(8, 2, ('range', -300000, 300000)), (8, 3, ('range', -300000, 300000))]
CDECLSTUBS = [8]
STUBWRITE = []
STUBCOPY = []
STACKCLEAN = 0
ECXANSWER = []
ECXOUT = []
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
    offset (`m+0x0`, `p0+0x10`), a local by its depth (`sp-0x14`), a word of the
    caller's frame by its height (`sp+0xc`), anything
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
    # A word at or above it, the caller's own frame (`ebp+0xc` past the function's arguments;
    # item 2921, parked 2918): its height over `esp`, not the harness's stack constant.
    # (Only 0x40 of it: the integer edge 0x7FFFFFFF lies 0xff over the top and is no pointer.)
    if top <= v < top + 0x40:
        return f"sp+{v - top:#x}"
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


def pair_answer(inner, ecx_out):
    """`--ecx-out IDX:V,V,..` (item 2818, parked 2816): a stub that answers
    as `inner` would, then leaves `ecx` as `ecx_out`, the pair `(eax, ecx)`
    `hooks.answer_regs` reads."""
    def answer(ecx, args):
        out = inner(ecx, args) if callable(inner) else inner
        return (out[0] if isinstance(out, tuple) else out), ecx_out
    return answer


def win_write(inner, at, size, v, write, read=None):
    """`--stub-write IDX:WIN+OFF:SIZE:VAL` (item 2346): a stub that writes VAL
    at a window's word each call — a callee that unlinks the head of the
    container its caller loops over (`Object::kill_contents`) — then answers
    as `inner` would."""
    def answer(ecx, args):
        if v == "dec":
            # `dec` (item 2749, parked 2734): one less each call, a count a
            # callee (`remove_current`, `Array::remove`) lowers.
            w = (int.from_bytes(read(at, size), "little") - 1) & ((1 << 8 * size) - 1)
            write(at, w.to_bytes(size, "little"))
        else:
            write(at, (v & ((1 << 8 * size) - 1)).to_bytes(size, "little"))
        return inner(ecx, args) if callable(inner) else inner
    return answer


def copy_answer(inner, read, write, cap):
    """`--stub-copy IDX` (item 2921, parked 2906): a stubbed `__cdecl` callee
    that does what `memcpy` does — argument 2's bytes from argument 1 to
    argument 0, at most `cap` — and answers the destination. The thunk the
    original calls is an import (`jmp [0xac5428]`), which the emulator
    resolves to nothing, so a plain stub copied nothing."""
    def answer(ecx, args):
        dst, src, n = (a & 0xFFFFFFFF for a in args[:3])
        if n:
            write(dst, read(src, min(n, cap)))
        return dst
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
    rng = random.Random(3250)
    # `--stub-cdecl IDX`: the callee's arguments are the caller's to pop (item 2325).
    h = Harness(argv[1], {a: (f"{a:x}", n, False, i in CDECLSTUBS) for i, (k, a, n) in enumerate(STUBS)
                          if k == "call"})
    uc = h.m.uc
    # `--tib`: an SEH-framed function reads and writes `fs:[0]` (item 3120).
    if TIB:
        set_tib(uc)
    slots = {off: (f"slot{off:x}", n, 0) for k, off, n in STUBS if k == "slot"}
    vt = h.vtable(slots) if slots else None
    names = [f"slot{a:x}" if k == "slot" else f"iat{a:x}" if k == "iat" else f"run{a:x}" if k == "run" else f"{a:x}" for k, a, _ in STUBS]
    # `--stub iat:VA:N`: a stub whose address is laid in an import's IAT cell,
    # which the emulator resolves to nothing, so `call [VA]` lands on it
    # (item 2336: `timeGetTime`, parked 2327).
    iats = [(a, h.stub(f"iat{a:x}", n, 0)) for k, a, n in STUBS if k == "iat"]
    # `--stub-cdecl IDX` on an `iat` stub (`malloc`: `call [VA]; add esp, 4`).
    for (_, sa), (i, (k, _, _)) in zip(iats, [e for e in enumerate(STUBS) if e[1][0] == "iat"]):
        if i in CDECLSTUBS:
            h.hooked[sa] = h.hooked[sa][:3] + (True,)
    stubbed = {off: h.r32(vt + off) & 0xFFFFFFFF for off, _ in SLOTBASE}
    # `--vtat-image WIN+OFF:VA` (item 2863): the stub slots copied into the image's own vtable at VA,
    # so a record whose vtable word is compared with VA (`Build::vftable`) still answers its slots.
    for va_ in sorted({va for _, _, va in VTIMAGE}):
        for off in slots:
            uc.mem_write(va_ + off, h.r32(vt + off).to_bytes(4, "little"))
    base = h.brk
    for i in range(1500):
        h.brk = base  # the bump heap is reused a row; 1500 rows of windows outgrow it
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
            if va in ATRANGE:
                lo, hi, esz = ATRANGE[va]
                for k in range(0, size - esz + 1, esz):
                    buf[k:k + esz] = rng.randint(lo, hi).to_bytes(esz, "little", signed=True)
            uc.mem_write(va, bytes(buf))
            wins.append((va, buf))
        gelems = []
        for cnt, lo, hi, ptr, esize in GTAB:
            # A counted array behind image words (item 2709, parked 2699): the count
            # drawn from LO..HI at CNT, the array's pointer at PTR, one record of
            # ESIZE random bytes an element — the head CNT..PTR+4 and the array are
            # `c` windows, the records `p` windows after the pointer arguments'.
            n = rng.randint(lo, hi)
            recs = [h.alloc(esize + 0x20) for _ in range(max(n, 0))]
            arr = h.alloc(4 * max(n, 1) + 0x20)
            head = bytearray(rng.randrange(256) for _ in range(ptr + 4 - cnt))
            head[0:4] = (n & 0xFFFFFFFF).to_bytes(4, "little")
            head[ptr - cnt:] = arr.to_bytes(4, "little")
            body = bytearray(b"".join(r.to_bytes(4, "little") for r in recs) or bytes(4))
            wins.append((cnt, head))
            wins.append((arr, body))
            gelems += [(r, bytearray(rng.randrange(256) for _ in range(esize))) for r in recs]
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
        wins += gelems
        nc = len(CELLS) + len(AT) + 2 * len(GTAB)
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
        forced = []
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
                forced += NULLFORCE.get(k, [])
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
        for win, off, v in forced:
            # `--place-null K:WIN+OFF=V`: on a null row of place K, the word at WIN+OFF is V, written after every
            # window is drawn, so a guard word beside the null pointer is held (item 2900, Loop 2899).
            w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
            w[off:off + 4] = (v & 0xFFFFFFFF).to_bytes(4, "little")
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
        for idx, ofirst, dfirst, nobj, loff in ANSWERBY:
            for j in range(nobj):
                tail = list(range(j + 1, nobj))
                link = (j + 1 if rng.random() < 0.7 else rng.choice(tail)) if tail and rng.random() < 0.9 else -1
                p[dfirst + j][loff:loff + 2] = (link & 0xFFFF).to_bytes(2, "little")
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
        for win, off, va_ in VTIMAGE:
            # The image's vtable at VA (stubbed above) laid half the rows, the stubbed vtable otherwise.
            w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
            w[off:off + 4] = (va_ if rng.random() < 0.5 else vt).to_bytes(4, "little")
        for win, off in VTATHALF:
            # The stubbed vtable laid half the rows at a record the function
            # compares with a class's vtable, the other half left as drawn
            # (a `--wbias` of the class's address; item 2223).
            if rng.random() < 0.5:
                w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
                w[off:off + 4] = vt.to_bytes(4, "little")
        for win, off, tgt, plus, always in ALIAS:
            # `--alias`: the word laid as a window's own address (plus K) half the rows, the other half left as
            # drawn: a pointer that names another window, or its own (item 2797, parked 2791).
            if always or rng.random() < 0.5:
                at_ = this if tgt == "m" else wins[(0 if tgt[0] == "c" else nc) + int(tgt[1:])][0]
                w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
                w[off:off + 4] = ((at_ + plus) & 0xFFFFFFFF).to_bytes(4, "little")
        for lo, hi, off, size, d, terms in LINKS:
            # A link field (`o_up`, a type's upgrade) of figures p(lo)..p(hi),
            # drawn from the figures on one side of each one's own index or a
            # terminator, so a walk through them ends (item 2276).
            n = hi - lo + 1
            for k in range(n):
                v = link_value(rng, n, k, d, terms)
                p[lo + k][off:off + size] = (v & ((1 << 8 * size) - 1)).to_bytes(size, "little")
        for lo, hi, off, vals in LINKHI:
            # `--link-hi pA..B+OFF:V,V,..|LO..HI`: the short beside a link (the player half of `obj+0x2c`'s word),
            # drawn from the list or range every figure after the links, so a word that reads it is never clobbered.
            for k in range(hi - lo + 1):
                p[lo + k][off:off + 2] = (wdraw(rng, vals) & 0xFFFF).to_bytes(2, "little")
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
        for idx, ofirst, dfirst, nobj, loff in ANSWERBY:
            # `--answer-by IDX:OBJ0:DAT0:N:LINKOFF` (item 2863, parked 2861): stub IDX answers
            # the address of `p<DAT0+j>` to the `ecx` that lies in `p<OBJ0+j>` — an object's
            # data record, the same one each ask — and each data record's short at LINKOFF
            # names a later object or is -1, so a walk down the links ends (the links
            # are laid with the windows, before they are written to memory).
            spans_ = [(wins[nc + ofirst + j][0], len(wins[nc + ofirst + j][1]), wins[nc + dfirst + j][0])
                      for j in range(nobj)]

            def by_ecx(ecx, args, spans_=spans_):
                for a_, n_, d_ in spans_:
                    if a_ <= (ecx & 0xFFFFFFFF) < a_ + n_:
                        return d_
                return 0
            h.answers[names[idx]] = by_ecx
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
            h.answers[names[idx]] = win_write(h.answers[names[idx]], at + off, size, v, uc.mem_write,
                                                      lambda a_, n_: bytes(uc.mem_read(a_, n_)))
        for idx in STUBCOPY:
            h.answers[names[idx]] = copy_answer(h.answers[names[idx]], lambda a_, n_: bytes(uc.mem_read(a_, n_)),
                                                uc.mem_write, 0x1000)
        derefs = [[] for _ in DEREF]
        for (idx, k, n, wr), sink in zip(DEREF, derefs):
            h.answers[names[idx]] = read_arg(h.answers[names[idx]], k, n,
                                             lambda a_, n_: bytes(uc.mem_read(a_, n_)), sink, wr)
        # `--ecx-out IDX:V,V,..` (item 2818, parked 2816): stub IDX leaves `ecx`
        # as a callee would, a value drawn from the list, after the other numbers on the row.
        ecxo = [rng.choice(vals) & 0xFFFFFFFF for _, vals in ECXOUT]
        for (idx, _), v in zip(ECXOUT, ecxo):
            h.answers[names[idx]] = pair_answer(h.answers[names[idx]], v)
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
        # `--edx-win` (item 2801): `edx` the address of a window, not an integer.
        edxa = 0 if EDX is None else edx
        if EDXWIN is not None:
            edxa = this if EDXWIN == "m" else wins[(0 if EDXWIN[0] == "c" else nc) + int(EDXWIN[1:])][0]
        # `--xmm` (item 2807): xmm N the float V/4 for an integer V in the range, V on the row.
        xmmv = [rng.randint(spec[1], spec[2]) if spec[0] == "range" else rng.choice(spec[1]) for _, spec in XMM]
        xmm = {n: xmm_bits(v / 4) for (n, _), v in zip(XMM, xmmv)}
        # `--clean-stack N` (item 2921, parked 2906): the stack under the call zeroed, so a byte of a
        # caller's local the function did not write reads 00 and not an earlier row's.
        if STACKCLEAN:
            uc.mem_write(STACK_BASE + STACK_SIZE - 0x100 - STACKCLEAN, bytes(STACKCLEAN))
        h.calls.clear()
        h.edxs.clear()
        r = h.call(ENTRY, tuple(args), ecx=this if ECXINT is None else ecx,
                   edx=edxa, xmm=xmm, ret=RET)  # Harness.call signs eax; signed() again would double a negative
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
                 + ([] if ECXINT is None else [str(ecx)]) + ([] if EDX is None else [str(edx)])
                 + [str(v) for v in xmmv] + [str(v) for v in ecxo])
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

# sweep_gen: ["obj", "Army::do_forming@006f43c0", "--item", "3250", "--name", "army_do_forming", "--win", "0x98", "--own", "1", "--cell-win", "0xc0618c:0x80", "--cell-win", "0xe85f20:0x2750", "--cell-win", "0xc061e0:0xe000", "--sparse", "@0xe3a390:0x6eec:2:8,4/0x74,12/0x6eb8,4", "--stub", "0x6f56d0:0", "--stub", "0x6f9120:2", "--stub", "0x6f8750:1", "--stub", "0x65d260:9", "--stub", "0x6edb50:1", "--stub", "slot:0xac:0", "--stub", "0x70fba0:10", "--stub", "0x70d830:5", "--stub", "0x92cf40:4", "--stub-cdecl", "8", "--stub", "0xa46a00:0", "--outarg", "8:2:-300000..300000", "--outarg", "8:3:-300000..300000", "--seq", "9:8", "--answer-range", "9:-3000:3000", "--ptrtab", "c0+0x14+0x1c*(m[0x34]&3):5:0x20:vt", "--ptrtab", "0xc09970+0x1c*(m[0x34]&3):3:0x10", "--place", "p2+0:0:0xf0", "--place", "p5+0:0:0xf0", "--answer-win", "5:0x80", "--answers", "0:0,0,0,0,0,1", "--answers", "1:0,0,0,1,2", "--answers", "3:-1,-1,0,1,5", "--answers", "4:0,1", "--wbias", "m+0x94:0,0x10000,0x20000,0x30000,0x40000,1,0x10001,0x20001,0x30001,0x40001,0xffff0000", "--wbias", "m+0x2c:0,0,1", "--wbias", "m+0x48:0,1,5,29,58,-2,100", "--wbias", "m+0x4c:0,1,7,22,29,-3,90", "--wbias", "m+0x50:0,0x1000000,0x40000000,0x7fffffff,-2147483648,-0x40000000,0x20000000,0x12345678,-0x1234567,0x5555555,-0x7fffffff,0x3fffffff", "--wbias", "m+0x54:-1,0,1,2,3", "--wbias", "m+0x58:-1,0,1,2,3", "--wbias", "m+0x5c:-1,0,1,2,3", "--wbias", "m+0x60:-1,0,1,2,3", "--wbias", "c1+0xc:0,1,1,1", "--wbias", "c1+0x9e0:0,1,1,1", "--wbias", "c1+0x13b4:0,1,1,1", "--wbias", "c1+0x1d88:0,1,1,1", "--wbias", "c2+*:0,1", "--wbias", "p0+0:0,1", "--wbias", "p3+0:0,1", "--wbias", "p1+*:0,1", "--wbias", "p4+*:0,1", "--wbias", "p16+0xe8:0x63187,0x63186,0x63185,0x63184,0x63183,0x63180,0,5,-1,0x80063187,0x63107", "--wbias", "p17+0xe8:0x63187,0x63186,0x63185,0x63184,0x63183,0x63180,0,5,-1,0x80063187,0x63107", "--wbias", "p18+0x70:0,0x10000,0x20000,-65536", "--wbias", "p13..15+0x4:1,1,1,3,0,2", "--record", "2", "--record", "3", "--record", "4", "--record", "6", "--record", "7", "--record", "8", "--record", "9", "--record-edx", "--rows", "1500", "--port", "{\n                // `a`: the function's own (unused) stack argument, then the eight stubs' single answers in the order the\n                // line names them: 1 `is_engaged`, 2 `count` (COUNT_SIEGE), 3 `set_stance`, 4 `find_building`, 5 `is_ally`,\n                // 6 slot +0xac's address (the data record, p18), 7 `action_move_to`, 8 `action_siege_attack_to`, 9 `project`, 10\n                // `sin_table`; 11 and 12 the point `project` writes (x', y'), 13..20 `sin_table`'s answers in call order\n                // (the last repeated). Windows:\n                // `c[0]` the objects singleton, `c[1]` the groups array (stride 0x9d4; `+8` the army's id, `+0xc` its\n                // units), `c[2]` the leaders' table of war flags (`+0x74 + 4 * (target * 0x1bbb + who)`), `p0..p5` the leaders\n                // table's slices a row (stride 0x6eec; per leader: the player `+8`, the war flags `+0x74 + 4 * target`, the\n                // pointer `+0x6eb8` to the Military record), `p12` the cities' array and\n                // `p13..p15` their records (flags the byte at +4), `p16`, `p17` the leaders' Military records (age at +0xe8), `p18` the\n                // target's data record (the city number the short at +0x72). Logs, in order: set_stance, find_building,\n                // is_ally, action_move_to, action_siege_attack_to, project, sin_table, each call `ecx,edx,args` (the `edx` of a\n                // callee that is not handed one is whatever was left, `*`; `project`'s two out-pointers are caller locals, `*`); `Group::action_siege_attack_to`'s two stack words\n                // between the point and the angle are never written (`sub esp, 8`), so they are masked on both sides.\n                //\n                // **The port is the crate's own steps**, `crate::army::do_forming_steps`, which `Sim::do_forming` calls\n                // over its `FormSim`; here the view is the emulated original's windows.\n                use crate::army::{FormView, do_forming_steps};\n                use crate::combat::Stance;\n                use crate::movement::Angle;\n                use crate::orders::{MoveKind, QueuePos};\n                use crate::world::{Cell, Pos};\n                struct V<'a> {\n                    a: &'a [i32],\n                    m: &'a mut [u8],\n                    c: &'a mut [Vec<u8>],\n                    p: &'a [Vec<u8>],\n                    log: &'a mut Vec<String>,\n                    sins: usize,\n                }\n                fn rd32(w: &[u8], o: usize) -> i32 {\n                    i32::from_le_bytes(w[o..o + 4].try_into().expect(\"four bytes\"))\n                }\n                fn rd16(w: &[u8], o: usize) -> i32 {\n                    i16::from_le_bytes(w[o..o + 2].try_into().expect(\"two bytes\")) as i32\n                }\n                impl V<'_> {\n                    fn push(&mut self, k: usize, s: String) {\n                        if self.log[k] == \"-\" {\n                            self.log[k] = s;\n                        } else {\n                            self.log[k].push('/');\n                            self.log[k].push_str(&s);\n                        }\n                    }\n                    fn leader(&self) -> usize {\n                        rd16(self.m, 0x94) as usize\n                    }\n                    fn gtok(g: i32) -> String {\n                        format!(\"c1+{:#x}\", g as usize * 0x9d4)\n                    }\n                }\n                impl FormView for V<'_> {\n                    fn project(&mut self, p: Pos, angle: Angle, d: i32) -> Pos {\n                        self.push(5, format!(\"{},{d},{},{},sp-0x10,sp-0x14\", angle.0, p.x, p.y));\n                        Pos::new(self.a[11], self.a[12])\n                    }\n                    fn sin_table(&mut self, folded: i32, d: i32) -> i32 {\n                        self.push(6, format!(\"{folded},{d}\"));\n                        let v = self.a[13 + self.sins.min(7)];\n                        self.sins += 1;\n                        v\n                    }\n                    fn engaged(&mut self) -> bool {\n                        self.a[1] != 0\n                    }\n                    fn group_count(&self) -> i32 {\n                        rd16(self.m, 0x96)\n                    }\n                    fn muster(&self) -> Cell {\n                        Cell {\n                            x: rd32(self.m, 0x48),\n                            y: rd32(self.m, 0x4c),\n                        }\n                    }\n                    fn muster_angle(&self) -> Angle {\n                        Angle(rd32(self.m, 0x50))\n                    }\n                    fn group_id(&self, i: i32) -> i32 {\n                        rd32(self.m, 0x54 + 4 * i as usize)\n                    }\n                    fn group_populated(&mut self, g: i32) -> bool {\n                        rd32(&self.c[1], g as usize * 0x9d4 + 0xc) != 0\n                    }\n                    fn group_claim(&mut self, g: i32) {\n                        let v = rd16(self.m, 2);\n                        let o = g as usize * 0x9d4 + 8;\n                        self.c[1][o..o + 4].copy_from_slice(&v.to_le_bytes());\n                    }\n                    fn hurry(&self) -> bool {\n                        rd32(self.m, 0x2c) != 0\n                    }\n                    fn has_target_who(&self) -> bool {\n                        rd32(self.m, 0x34) >= 0\n                    }\n                    fn target_who_differs(&self) -> bool {\n                        rd32(self.m, 0x34) != rd32(&self.p[3 * self.leader()], 0)\n                    }\n                    fn target_enemy(&mut self) -> bool {\n                        let (l, t) = (self.leader(), rd32(self.m, 0x34) as usize);\n                        let who = rd32(&self.p[3 * l], 0) as usize;\n                        rd32(&self.p[3 * l + 1], 4 * t) == 0\n                            || rd32(&self.c[2], (t * 0x1bbb + who) * 4 + 0x74) == 0\n                    }\n                    fn young(&mut self) -> bool {\n                        (rd32(&self.p[16 + self.leader()], 0xe8) ^ 0x63187) < 4\n                    }\n                    fn siege_count(&mut self) -> i32 {\n                        self.a[2]\n                    }\n                    fn set_stance(&mut self, s: Stance) {\n                        let v = match s {\n                            Stance::Raid => 3,\n                            Stance::Aggressive => 0,\n                            _ => panic!(\"a stance the original does not set here\"),\n                        };\n                        self.push(0, format!(\"m+0x0,*,{v}\"));\n                    }\n                    fn enemy_combat_building_near(&mut self, p: Pos) -> bool {\n                        let l = self.leader();\n                        self.push(\n                            1,\n                            format!(\"c0+0x0,*,{},{},3,{l},768,512,9,0,0\", p.x, p.y),\n                        );\n                        self.a[4] >= 0\n                    }\n                    fn target_is_me(&self) -> bool {\n                        rd32(self.m, 0x34) == self.leader() as i32\n                    }\n                    fn target_ally(&mut self) -> bool {\n                        let (l, t) = (self.leader(), rd32(self.m, 0x34));\n                        self.push(2, format!(\"{},*,{t}\", 14918544 + l * 0x6eec));\n                        self.a[5] != 0\n                    }\n                    fn has_target_obj(&self) -> bool {\n                        rd32(self.m, 0x30) >= 0\n                    }\n                    fn target_city_flags(&mut self) -> Option<i32> {\n                        let n = rd16(&self.p[18], 0x72);\n                        (n >= 0).then(|| i32::from(self.p[13 + n as usize][4]))\n                    }\n                    fn move_to(\n                        &mut self,\n                        g: i32,\n                        to: Pos,\n                        queue: QueuePos,\n                        set_angle: bool,\n                        angle: Angle,\n                        kind: MoveKind,\n                        action: bool,\n                    ) {\n                        let q = match queue {\n                            QueuePos::New => 2,\n                            _ => panic!(\"a queue the original does not pass here\"),\n                        };\n                        let k = match kind {\n                            MoveKind::AttackTo => 2,\n                            _ => panic!(\"a kind the original does not pass here\"),\n                        };\n                        let s = format!(\n                            \"{},*,{},{},{q},{},{},{k},{},-1,-1,0\",\n                            Self::gtok(g),\n                            to.x,\n                            to.y,\n                            i32::from(set_angle),\n                            angle.0,\n                            i32::from(action),\n                        );\n                        self.push(3, s);\n                    }\n                    fn siege_attack_to(&mut self, g: i32, to: Pos, angle: Angle) {\n                        let s = format!(\"{},*,{},{},*,*,{}\", Self::gtok(g), to.x, to.y, angle.0);\n                        self.push(4, s);\n                    }\n                }\n                log.extend([\"-\", \"-\", \"-\", \"-\", \"-\", \"-\", \"-\"].map(String::from));\n                let mut v = V {\n                    a,\n                    m,\n                    c,\n                    p: &*p,\n                    log: &mut log,\n                    sins: 0,\n                };\n                i64::from(do_forming_steps(&mut v))\n            }", "--wbias", "m+0x34:0,1,2", "--wbias", "m+0x30:-1,0,1,2,3,4"]
