#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_path_finder_find_tpath_006897d0.py — `PathFinder::find_tpath@006897d0` under unicorn (item 3337, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_path_finder_find_tpath_006897d0.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 16-byte object of random bytes (no bias), 5 stack argument(s);
`p` windows in order: p0 argument 0, p1..p7 the place(s), in flag order, p8..p9 the stubs' answer window(s); the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c06188 points at 64 random bytes; the global cell 0x00c0618c points at 112 random bytes; 152 random bytes at 0x00e85e40; 112 random bytes at 0x00c0aec0; argument 0 names 16 random bytes; callee 0x00687d60 (2 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x0046f960 (0 args) stubbed; callee 0x0046d820 (1 args) stubbed; callee 0x006179d0 (0 args) stubbed; callee 0x00683770 (3 args) stubbed; callee 0x00687ae0 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `c2+0x14+0x1c*a[3]` points at 12 random bytes less `0` (placed); the word at `p2+4*a[4]` points at 32 random bytes less `0` (placed); the word at `c4+0x1c*a[3]` points at 16 random bytes less `0` (placed); the word at `p4+4*a[4]` points at 32 random bytes less `0` (placed); the word at `p5+0x18` points at 704 random bytes less `0` (placed); the word at `p3+0x18` points at 560 random bytes less `0` (placed); stub 0 answered from [0, 1, 1] (90%); stub 2 answered from [0, 0, 0, 1] (90%); stub 7 answered from [0, 1, 1, 1] (90%); stub 8 answered from [0, 1, 1, 1] (90%); stub 1 answered from [0, 1, 2, 3, 4, 5] (90%); stub 4 answers the address of -32 random bytes, a `p` window after the places; stub 9 answers the address of 88 random bytes, a `p` window after the places; stub 0 answers 30 values in call order, printed after the answers; stub 1 answers 60 values in call order, printed after the answers; stub 5 answers -1..4 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 3 prints the 16 bytes its argument 0 points at, hex, after the record (`/` a call); `0x00a46980` run once before the rows (a table builder the startup would have run); the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`.
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

ENTRY = 0x006897d0
WIN = 16
OWN = 5
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(13297148, 4096), (12607880, 64), (12607884, 112)]
AT = [(15228480, 152), (12627648, 112)]
ATRANGE = {}
PTRARGS = [(0, 16)]
PNULLS = []
STUBS = [('call', 6847840, 2), ('call', 7033568, 2), ('call', 4651360, 0), ('call', 4642848, 1), ('call', 6388176, 0), ('call', 6829936, 3), ('call', 6847200, 0), ('slot', 24, 0), ('slot', 20, 0), ('slot', 64, 0)]
PLACES = [('p0', '0', '0', 128), ('c2', '0x14+0x1c*a[3]', '0', 12), ('p2', '4*a[4]', '0', 32), ('c4', '0x1c*a[3]', '0', 16), ('p4', '4*a[4]', '0', 32), ('p5', '0x18', '0', 704), ('p3', '0x18', '0', 560)]
NULLS = []
NULLFORCE = {}
WBIAS = [('c1', 24, ('range', 8, 40)), ('c1', 28, ('range', 8, 40)), ('p0', 8, ('set', [1, 1, 1, 2, 3, 4, 5])), ('p1', 0, ('range', 150, 900)), ('p1', 4, ('range', 150, 900)), ('p1', 8, ('set', [0, 96, 384, 48, 96])), ('p1', 12, ('set', [0, 0, 1, 1, 2, 3, 33])), ('p1', 16, ('range', 0, 4000)), ('p1', 20, ('range', 0, 4000)), ('p1', 24, ('set', [0, 96, 384, 48, 96])), ('p1', 28, ('set', [0, 0, 1, 1, 2, 3, 33])), ('p1', 32, ('range', 0, 1500)), ('p1', 36, ('range', 0, 1500)), ('p1', 40, ('set', [0, 96, 384, 48, 96])), ('p1', 44, ('set', [0, 0, 1, 1, 2, 3, 33])), ('p1', 48, ('range', 0, 4000)), ('p1', 52, ('range', 0, 4000)), ('p1', 56, ('set', [0, 96, 384, 48, 96])), ('p1', 60, ('set', [0, 0, 1, 1, 2, 3, 33])), ('p1', 64, ('range', 0, 1500)), ('p1', 68, ('range', 0, 1500)), ('p1', 72, ('set', [0, 96, 384, 48, 96])), ('p1', 76, ('set', [0, 0, 1, 1, 2, 3, 33])), ('c3', 132, ('set', [0])), ('p6', 692, ('set', [0, 0, 0, 32, 64, 224])), ('p7', 536, ('set', [0, 0, 1, 1, 2, 3])), ('p9', 76, ('set', [0, 50, 191, 192, 300, 1000, 65336, 65486, 3276800, 3276850, 3276991, 3276992, 3277100, 3277800, 3342136, 3342286, 12517376, 12517426, 12517567, 12517568, 12517676, 12518376, 12582712, 12582862, 12582912, 12582962, 12583103, 12583104, 12583212, 12583912, 12648248, 12648398, 19660800, 19660850, 19660991, 19660992, 19661100, 19661800, 19726136, 19726286, 65536000, 65536050, 65536191, 65536192, 65536300, 65537000, 65601336, 65601486, -13107200, -13107150, -13107009, -13107008, -13106900, -13106200, -13041864, -13041714, -3276800, -3276750, -3276609, -3276608, -3276500, -3275800, -3211464, -3211314]))]
FILLS = [('c0', 'i//3')]
ARGRANGE = [(3, 0, 2), (4, 0, 2), (1, 250, 750), (2, 250, 750)]
SLOTBASE = []
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [0, 1, 1]), (2, [0, 0, 0, 1]), (7, [0, 1, 1, 1]), (8, [0, 1, 1, 1]), (1, [0, 1, 2, 3, 4, 5])]
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = []
ANSWERBY = []
ANSWERWIN = [(4, -32), (9, 88)]
CDECL = False
TIB = False
ARGEQ = []
SEQ = [(0, 30, None), (1, 60, None)]
ANSRANGE = [(5, -1, 4)]
CELLTHIS = []
VTALSO = [('p3', 0), ('p8', 0)]
SPARSE = []
GTAB = []
RECORD = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
RECORDORDER = True
RECORDEDX = False
ROWSPLIT = {}
LINKS = []
LINKHI = []
OUTARG = []
CDECLSTUBS = []
STUBWRITE = []
STUBCOPY = []
STACKCLEAN = 0
INITCALLS = [10774912]
MAXINSNS = 200000
ECXANSWER = []
ECXOUT = []
RING = []
DEREF = [(1, 0, 4, None), (1, 1, 4, None), (3, 0, 16, None)]
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
    # A third member of `span` is a STEP: answers LO, LO+STEP, ... (`--answer-range IDX:LO:HI:STEP`, item 3320);
    # a fourth, 1, draws them without replacement (`:STEP:distinct`), so no two calls of a row answer one address.
    if span and len(span) > 3 and span[3]:
        slots = rng.sample(range((span[1] - span[0]) // span[2] + 1), min(n, (span[1] - span[0]) // span[2] + 1))
        vals = [span[0] + span[2] * k for k in slots]
        vals += [vals[-1]] * (n - len(vals))
    else:
        vals = [span[0] + rng.randint(0, (span[1] - span[0]) // (span[2:] or (1,))[0]) * (span[2:] or (1,))[0]
                if span else rng.choice(pick) if pick and rng.random() < 0.9 else rand32(rng)
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
    rng = random.Random(3337)
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
    # `--init-call VA` (this sweep, item 3268 `Object::update_seen`): an argument-less `__cdecl` function
    # the image's startup would have run (a table builder: `circle_init`, `ring_init`), run once before the
    # rows, so the tables the function reads are the original's own. A `--stub-copy` stub answers `memcpy`.
    for idx in STUBCOPY:
        h.answers[names[idx]] = copy_answer(0, lambda a_, n_: bytes(uc.mem_read(a_, n_)), uc.mem_write, 0x10000)
    for va_ in INITCALLS:
        h.call(va_, (), max_insns=500_000_000)
    base = h.brk
    for i in range(4000):
        h.brk = base  # the bump heap is reused a row; 4000 rows of windows outgrow it
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
            ex_ = idx < 0  # `--argset IDX!:V,..` (item 3272): every row from the list, no wild tenth
            idx = ~idx if ex_ else idx
            args[idx] = rng.choice(vals) if ex_ or rng.random() < 0.9 else args[idx]
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
        for idx, lo, hi, *st in ANSRANGE:
            answers[idx] = lo + rng.randint(0, (hi - lo) // (st or (1,))[0]) * (st or (1,))[0]
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
        nullflags = []
        for idx, size in ANSWERWIN:
            # A stub that answers a record: the address of SIZE random bytes,
            # a new `p` window after the places (parked 2151, item 2170).
            # A negative SIZE is |SIZE| bytes whose call answers null a third
            # of the time (a lookup that may find nothing; item 3268).
            nullable = size < 0
            size = abs(size)
            buf = h.alloc(size + 0x20)
            b = bytearray(rng.randrange(256) for _ in range(size))
            p.append(b)
            wins.append((buf, b))
            shape(rng, f"p{len(p) - 1}", b)
            if nullable:
                # Each nullable call's flag (1 = null) is printed after the other numbers on the row.
                nullflags.append(int(rng.random() < 1 / 3))
                if nullflags[-1]:
                    buf = 0
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
                              {i: (lo, hi, *st) for i, lo, hi, *st in ANSRANGE}.get(idx))
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
                   edx=edxa, xmm=xmm, ret=RET, max_insns=MAXINSNS)  # Harness.call signs eax; signed() again would double a negative
        esp = uc.reg_read(UC_X86_REG_ESP)
        # A `__cdecl` function leaves its arguments for the caller (item 2181).
        want_esp = STACK_BASE + STACK_SIZE - 0x100 - (4 * len(args) if CDECL else 0)
        assert esp == want_esp, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(this + WIN, 0x20)) == guard, "wrote past the window"
        before = ";".join([bytes(mem).hex()] + [bytes(b).hex() for _, b in wins])
        after = ";".join([bytes(uc.mem_read(this, WIN)).hex()]
                         + [bytes(uc.mem_read(a, len(b))).hex() for a, b in wins])
        shown = ([str(a) for a in args] + [str(v) for v in answers] + [str(v) for v in outs]
                 + [str(v) for s in seqs for v in s] + [str(f) for f in based] + [str(f) for f in nullflags]
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

# sweep_gen: ["obj", "PathFinder::find_tpath@006897d0", "--item", "3337", "--name", "path_finder_find_tpath_006897d0", "--own", "5", "--win", "0x10", "--ret", "eax", "--ptrarg", "0:0x10", "--cell-win", "0xcae5fc:0x1000", "--cell-win", "0xc06188:0x40", "--at", "0xe85e40:0x98", "--cell-win", "0xc0618c:0x70", "--at", "0xc0aec0:0x70", "--place", "p0+0:0:0x80", "--place", "c2+0x14+0x1c*a[3]:0:12", "--place", "p2+4*a[4]:0:0x20", "--place", "c4+0x1c*a[3]:0:16", "--place", "p4+4*a[4]:0:0x20", "--place", "p5+0x18:0:0x2c0", "--place", "p3+0x18:0:0x230", "--argrange", "3:0:2", "--argrange", "4:0:2", "--fill", "c0:i//3", "--vtat-also", "p3+0", "--stub", "0x687d60:2", "--stub", "0x6b52e0:2", "--stub", "0x46f960:0", "--stub", "0x46d820:1", "--stub", "0x6179d0:0", "--stub", "0x683770:3", "--stub", "0x687ae0:0", "--stub", "slot:0x18:0", "--stub", "slot:0x14:0", "--stub", "slot:0x40:0", "--answer-win", "4:-0x20", "--answer-win", "9:0x58", "--vtat-also", "p8+0", "--answers", "0:0,1,1", "--answers", "2:0,0,0,1", "--answers", "7:0,1,1,1", "--answers", "8:0,1,1,1", "--answer-range", "5:-1:4", "--seq", "0:30", "--seq", "1:60", "--answers", "1:0,1,2,3,4,5", "--record", "0", "--record", "1", "--record", "2", "--record", "3", "--record", "4", "--record", "5", "--record", "6", "--record", "7", "--record", "8", "--record", "9", "--record-order", "--deref", "1:0:4", "--deref", "1:1:4", "--deref", "3:0:16", "--argrange", "1:250:750", "--argrange", "2:250:750", "--wbias", "c1+0x18:8..40", "--wbias", "c1+0x1c:8..40", "--wbias", "p0+8:1,1,1,2,3,4,5", "--wbias", "p1+0:150..900", "--wbias", "p1+4:150..900", "--wbias", "p1+8:0,0x60,0x180,0x30,0x60", "--wbias", "p1+12:0,0,1,1,2,3,0x21", "--wbias", "p1+16:0..4000", "--wbias", "p1+20:0..4000", "--wbias", "p1+24:0,0x60,0x180,0x30,0x60", "--wbias", "p1+28:0,0,1,1,2,3,0x21", "--wbias", "p1+32:0..1500", "--wbias", "p1+36:0..1500", "--wbias", "p1+40:0,0x60,0x180,0x30,0x60", "--wbias", "p1+44:0,0,1,1,2,3,0x21", "--wbias", "p1+48:0..4000", "--wbias", "p1+52:0..4000", "--wbias", "p1+56:0,0x60,0x180,0x30,0x60", "--wbias", "p1+60:0,0,1,1,2,3,0x21", "--wbias", "p1+64:0..1500", "--wbias", "p1+68:0..1500", "--wbias", "p1+72:0,0x60,0x180,0x30,0x60", "--wbias", "p1+76:0,0,1,1,2,3,0x21", "--wbias", "c3+0x84:0", "--wbias", "p6+0x2b4:0,0,0,0x20,0x40,0xe0", "--wbias", "p7+0x218:0,0,1,1,2,3", "--wbias", "p9+0x4c:0,50,191,192,300,1000,65336,65486,3276800,3276850,3276991,3276992,3277100,3277800,3342136,3342286,12517376,12517426,12517567,12517568,12517676,12518376,12582712,12582862,12582912,12582962,12583103,12583104,12583212,12583912,12648248,12648398,19660800,19660850,19660991,19660992,19661100,19661800,19726136,19726286,65536000,65536050,65536191,65536192,65536300,65537000,65601336,65601486,-13107200,-13107150,-13107009,-13107008,-13106900,-13106200,-13041864,-13041714,-3276800,-3276750,-3276609,-3276608,-3276500,-3275800,-3211464,-3211314", "--init-call", "0xa46980", "--rows", "4000", "--port", "{\n            // **The port is the crate's own steps**,\n            // `crate::tpath::find_tpath_steps`, which `Sim::find_tpath` runs\n            // over its `SimTpath`; here the view is the emulated original's\n            // windows.\n            //\n            // `a`: the five stack arguments (the `Stack<PathData>*`, the\n            // from-position `x`, `y`, the unit class and slot), the ten\n            // stubs' single answers (`a[5..15]`: 2 `can_transport`, 5\n            // `astar_path`, 7 and 8 the slots `+0x18` and `+0x14`; the\n            // others unused), stub 0's thirty `valid_tcoord` answers\n            // (`a[15..45]`), stub 1's sixty `get_tregion` answers\n            // (`a[45..105]`), then the one null flag of `update_order`\n            // (`a[105]`). Stubs: 0 `valid_tcoord`, 1 `get_tregion` (its\n            // two arguments are locals; the tile pair is printed after the\n            // record), 2 `UnitData::can_transport`, 3 `Stack<PathData>::push`\n            // (its argument a local whose 16 bytes the row prints), 4\n            // `Unit::update_order`, 5 `astar_path`, 6 `kill_lists`, 7/8/9 the\n            // slots `+0x18`, `+0x14`, `+0x40`. Windows: `c0` `div_3_table`,\n            // `c1` the world (`+0x18`/`+0x1c` its extent in tiles), `c2` the\n            // objects singleton, `c3` the pathfinder (`+0x54` the unit,\n            // `+0x58`/`+0x5c` the start tile, `+0x68`/`+0x6c` the tile\n            // offset, `+0x74`, `+0x84` the `saving` flag held 0, `+0x94`),\n            // `c4` the units singleton; `p0` the stack (`+0` list, `+8`\n            // length), `p1` its list, `p2` the objects row, `p3` the\n            // object (`+0` vtable, `+0x18` type), `p4` the units row, `p5`\n            // the unit data (`+0x18` type), `p6` its type (`+0x2b4`), `p7`\n            // the object's type (`+0x218` domain), `p8` the order\n            // `update_order` answers, `p9` the `MoveOrder` its slot `+0x40`\n            // answers (`+0x4c`/`+0x4e`).\n            use crate::Pos;\n            use crate::orders::PathData;\n            use crate::tpath::{TpathView, find_tpath_steps};\n            fn rd32(w: &[u8], o: usize) -> i32 {\n                i32::from_le_bytes(w[o..o + 4].try_into().expect(\"four bytes\"))\n            }\n            fn rd16(w: &[u8], o: usize) -> i32 {\n                i32::from(i16::from_le_bytes(\n                    w[o..o + 2].try_into().expect(\"two bytes\"),\n                ))\n            }\n            fn wr32(w: &mut [u8], o: usize, v: i32) {\n                w[o..o + 4].copy_from_slice(&v.to_le_bytes());\n            }\n            fn hex(w: [i32; 4]) -> String {\n                let mut s = String::new();\n                for v in w {\n                    for b in v.to_le_bytes() {\n                        s.push_str(&format!(\"{b:02x}\"));\n                    }\n                }\n                s\n            }\n            struct V<'a> {\n                a: &'a [i32],\n                c: &'a mut [Vec<u8>],\n                p: &'a mut [Vec<u8>],\n                log: &'a mut Vec<String>,\n                pushed: Vec<String>,\n                tiles_x: Vec<String>,\n                tiles_y: Vec<String>,\n                tregions: usize,\n                valids: usize,\n            }\n            impl TpathView for V<'_> {\n                fn begin(&mut self, start: Pos) {\n                    // The original also writes `pathfinder +0x54` (the unit's\n                    // pointer, from the units row), `+0x68`/`+0x6c` and\n                    // `+0x94` (zero): fields the crate holds no copy of, so\n                    // the emulated windows carry them.\n                    let unit = self.p[4][4 * self.a[4] as usize..][..4].to_vec();\n                    self.c[3][0x54..0x58].copy_from_slice(&unit);\n                    wr32(&mut self.c[3], 0x58, start.x);\n                    wr32(&mut self.c[3], 0x5c, start.y);\n                    wr32(&mut self.c[3], 0x68, 0);\n                    wr32(&mut self.c[3], 0x6c, 0);\n                    wr32(&mut self.c[3], 0x94, 0);\n                }\n                fn pop(&mut self) -> Option<PathData> {\n                    let len = rd32(&self.p[0], 8);\n                    if len < 1 {\n                        return None;\n                    }\n                    let k = (len - 1) as usize * 16;\n                    wr32(&mut self.p[0], 8, len - 1);\n                    let l = &self.p[1];\n                    Some(PathData {\n                        to: Pos::new(rd32(l, k), rd32(l, k + 4)),\n                        tolerance: rd32(l, k + 8),\n                        flags: rd32(l, k + 12) as u8,\n                    })\n                }\n                fn push(&mut self, e: PathData) {\n                    // The third push of a call is the second local.\n                    let local = if self.pushed.len() == 2 { 0x68 } else { 0x58 };\n                    self.log.push(format!(\"s3:p0+0x0,sp-0x{local:x}\"));\n                    self.pushed\n                        .push(hex([e.to.x, e.to.y, e.tolerance, i32::from(e.flags)]));\n                }\n                fn clear(&mut self) {\n                    wr32(&mut self.p[0], 8, 0);\n                }\n                fn depth(&self) -> i32 {\n                    rd32(&self.p[0], 8)\n                }\n                fn tile_in_bounds(&self, t: Pos) -> bool {\n                    t.x >= 0\n                        && t.y >= 0\n                        && t.x < rd32(&self.c[1], 0x18)\n                        && t.y < rd32(&self.c[1], 0x1c)\n                }\n                fn is_helicopter(&self) -> bool {\n                    rd32(&self.p[6], 0x2b4) & 0x20 != 0\n                }\n                fn domain_below_2(&self) -> bool {\n                    rd32(&self.p[7], 0x218) < 2\n                }\n                fn can_transport(&mut self) -> bool {\n                    self.log.push(\"s2:p5+0x0\".to_string());\n                    self.a[5 + 2] != 0\n                }\n                fn tregion(&mut self, t: Pos) -> i32 {\n                    // The goal's pair first, then the start's: two different\n                    // locals.\n                    if self.tregions.is_multiple_of(2) {\n                        self.log.push(\"s1:c1+0x0,sp-0x30,sp-0x34\".to_string());\n                    } else {\n                        self.log.push(\"s1:c1+0x0,sp-0x40,sp-0x3c\".to_string());\n                    }\n                    let v = self.a[45 + self.tregions.min(59)];\n                    self.tregions += 1;\n                    self.tiles_x.push(hex([t.x, 0, 0, 0])[..8].to_string());\n                    self.tiles_y.push(hex([t.y, 0, 0, 0])[..8].to_string());\n                    v\n                }\n                fn valid_tcoord(&mut self, p: Pos) -> bool {\n                    self.log.push(format!(\"s0:c3+0x0,{},{}\", p.x, p.y));\n                    let v = self.a[15 + self.valids.min(29)];\n                    self.valids += 1;\n                    v != 0\n                }\n                fn order_offset(&mut self) -> Option<Pos> {\n                    self.log.push(\"s7:p3+0x0\".to_string());\n                    if self.a[5 + 7] == 0 {\n                        return None;\n                    }\n                    self.log.push(\"s4:p5+0x0\".to_string());\n                    if self.a[105] != 0 {\n                        return None;\n                    }\n                    self.log.push(\"s4:p5+0x0\".to_string());\n                    self.log.push(\"s8:p8+0x0\".to_string());\n                    if self.a[5 + 8] == 0 {\n                        return None;\n                    }\n                    self.log.push(\"s4:p5+0x0\".to_string());\n                    self.log.push(\"s9:p8+0x0\".to_string());\n                    Some(Pos::new(rd16(&self.p[9], 0x4c), rd16(&self.p[9], 0x4e)))\n                }\n                fn set_offset(&mut self, off: Pos) {\n                    wr32(&mut self.c[3], 0x68, off.x);\n                    wr32(&mut self.c[3], 0x6c, off.y);\n                }\n                fn search(&mut self) -> i32 {\n                    self.log.push(\"s5:c3+0x0,p0+0x0,192,0\".to_string());\n                    // `pathfinder +0x74` zeroed before `kill_lists`.\n                    wr32(&mut self.c[3], 0x74, 0);\n                    self.log.push(\"s6:c3+0x0\".to_string());\n                    self.a[5 + 5]\n                }\n                fn drop_unfinal_top(&mut self) {\n                    let len = rd32(&self.p[0], 8);\n                    let k = if len >= 1 { (len - 1) as usize * 16 } else { 0 };\n                    if rd32(&self.p[1], k + 12) & 1 == 0 {\n                        wr32(&mut self.p[0], 8, len.max(1) - 1);\n                    }\n                }\n            }\n            let mut v = V {\n                a,\n                c,\n                p,\n                log: &mut log,\n                pushed: Vec::new(),\n                tiles_x: Vec::new(),\n                tiles_y: Vec::new(),\n                tregions: 0,\n                valids: 0,\n            };\n            let r = find_tpath_steps(&mut v, Pos::new(a[1], a[2]));\n            let col = |w: &Vec<String>| {\n                if w.is_empty() {\n                    \"-\".to_string()\n                } else {\n                    w.join(\"/\")\n                }\n            };\n            let (x, y, pushed) = (col(&v.tiles_x), col(&v.tiles_y), col(&v.pushed));\n            // No call recorded at all is a `-` column of its own.\n            if v.log.is_empty() {\n                v.log.push(\"-\".to_string());\n            }\n            v.log.push(x);\n            v.log.push(y);\n            v.log.push(pushed);\n            i64::from(r)\n        }\n"]
