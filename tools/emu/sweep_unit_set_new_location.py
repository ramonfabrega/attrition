#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_set_new_location.py — `Unit::set_new_location@005f8d20` under unicorn (item 3289, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_unit_set_new_location.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 256-byte object of random bytes (no bias), 4 stack argument(s);
`p` windows in order: p0..p13 the place(s), in flag order; the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c06218 points at 16 random bytes; vtable slot +0xbc (0 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x005e4a60 (7 args) stubbed; callee 0x0064cd20 (4 args) stubbed; callee 0x00646d50 (1 args) stubbed; vtable slot +0x158 (3 args) stubbed; callee 0x00647080 (3 args) stubbed; callee 0x00647970 (0 args) stubbed; callee 0x008544a0 (4 args) stubbed; callee 0x0064d8c0 (0 args) stubbed; vtable slot +0x30 (0 args) stubbed; callee 0x005f9780 (0 args) stubbed; vtable slot +0x174 (1 args) stubbed; callee 0x005f96b0 (4 args) stubbed; callee 0x005d9010 (2 args) stubbed; callee 0x005d86f0 (3 args) stubbed; callee 0x00a46a00 (0 args) run, not hooked; the word at `m+0xf4` points at 40 random bytes less `0` (placed); the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `p0+4` points at 128 random bytes less `0` (placed); the word at `p0+8` points at 128 random bytes less `0` (placed); the word at `p0+12` points at 128 random bytes less `0` (placed); the word at `p0+16` points at 128 random bytes less `0` (placed); the word at `p0+20` points at 128 random bytes less `0` (placed); the word at `p0+24` points at 128 random bytes less `0` (placed); the word at `p0+28` points at 128 random bytes less `0` (placed); the word at `p0+32` points at 128 random bytes less `0` (placed); the word at `p0+36` points at 128 random bytes less `0` (placed); the word at `m+0x18` points at 768 random bytes less `0` (placed); the word at `c1+0x134` points at 2048 random bytes less `0` (placed); the word at `c1+0x138` points at 1280 random bytes less `0` (placed); slot +0xbc laid as 0x0046ce30 or its stub, a flag after the answers; slot +0x158 laid as 0x0060eda0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 4 answered from [0, 1, 2] (90%); stub 10 answered from [0, 0, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 8 writes a value in -5000..5000 through its argument 0, printed after the answers (`o` in a place); `0x00a46980` run once before the rows (a table builder the startup would have run).
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

ENTRY = 0x005f8d20
WIN = 256
OWN = 4
BIAS = []
ARGSET = [(2, [0, 1, 2]), (3, [0, 1])]
ARGFROM = []
CELLS = [(13297148, 1024), (12607880, 320), (12608024, 16)]
AT = []
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('slot', 188, 0), ('call', 7213936, 1), ('call', 6179424, 7), ('call', 6606112, 4), ('call', 6581584, 1), ('slot', 344, 3), ('call', 6582400, 3), ('call', 6584688, 0), ('call', 8733856, 4), ('call', 6609088, 0), ('slot', 48, 0), ('call', 6264704, 0), ('slot', 372, 1), ('call', 6264496, 4), ('call', 6131728, 2), ('call', 6129392, 3), ('run', 10775040, 0)]
PLACES = [('m', '0xf4', '0', 40), ('p0', '0', '0', 128), ('p0', '4', '0', 128), ('p0', '8', '0', 128), ('p0', '12', '0', 128), ('p0', '16', '0', 128), ('p0', '20', '0', 128), ('p0', '24', '0', 128), ('p0', '28', '0', 128), ('p0', '32', '0', 128), ('p0', '36', '0', 128), ('m', '0x18', '0', 768), ('c1', '0x134', '0', 2048), ('c1', '0x138', '0', 1280)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 16, ('set', [407095, 407052, 407105, 407174, 407259, 407312, 407381, 407466, 407535, 406564, 406649, 406718, 406771, 406728, 406797, 406850, 406919, 407004, 406033, 406102, 406187, 406240, 406309, 406394, 406463, 406516, 406473, 405518, 405571, 405656, 405725, 405778, 405847, 405932, 405985, 409126, 409211, 409264, 409333, 409290, 409359, 409412, 409497, 409566])), ('m', 20, ('set', [407095, 407052, 407105, 407174, 407259, 407312, 407381, 407466, 407535, 406564, 406649, 406718, 406771, 406728, 406797, 406850, 406919, 407004, 406033, 406102, 406187, 406240, 406309, 406394, 406463, 406516, 406473, 405518, 405571, 405656, 405725, 405778, 405847, 405932, 405985, 409126, 409211, 409264, 409333, 409290, 409359, 409412, 409497, 409566])), ('m', 180, ('set', [256, 256, 256, 256, 256, 256, 256, 256, 512, 768, 1024, 1280, 1536, 2560, 1792, 65280, 32768])), ('m', 168, ('set', [327680, 524288, 0, 65536, 196608, 327680, 524288])), ('c1', 0, ('range', 1, 8)), ('c1', 4, ('range', 1, 8)), ('c1', 24, ('range', 1, 24)), ('p11', 536, ('set', [0, 0, 1, 1, 2])), ('p11', 548, ('set', [0, 0, 32, 64, 100, 200, -50, 1000, 5000]))]
FILLS = [('c0', 'i//3')]
ARGRANGE = [(0, 0, 2560), (1, 0, 2560)]
SLOTBASE = [(188, 4640304), (344, 6352288)]
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [0, 1]), (1, [0, 1]), (4, [0, 1, 2]), (10, [0, 0, 1])]
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = []
ANSWERBY = []
ANSWERWIN = []
CDECL = False
TIB = False
ARGEQ = []
SEQ = [(0, 2, None)]
ANSRANGE = []
CELLTHIS = []
VTALSO = []
SPARSE = []
GTAB = []
RECORD = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
RECORDORDER = True
RECORDEDX = False
ROWSPLIT = {}
LINKS = []
LINKHI = []
OUTARG = [(8, 0, ('range', -5000, 5000))]
CDECLSTUBS = []
STUBWRITE = []
STUBCOPY = []
STACKCLEAN = 0
INITCALLS = [10774912]
MAXINSNS = 200000
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
    rng = random.Random(3289)
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
    for i in range(3000):
        h.brk = base  # the bump heap is reused a row; 3000 rows of windows outgrow it
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

# sweep_gen: ["obj", "Unit::set_new_location@005f8d20", "--item", "3289", "--win", "0x100", "--own", "4", "--argrange", "0:0:0xa00", "--argrange", "1:0:0xa00", "--argset", "2:0,1,2", "--argset", "3:0,1", "--cell-win", "0xcae5fc:0x400", "--cell-win", "0xc06188:0x140", "--cell-win", "0xc06218:0x10", "--fill", "c0:i//3", "--ptrtab", "m+0xf4:10:0x80", "--place", "m+0x18:0:0x300", "--place", "c1+0x134:0:0x800", "--place", "c1+0x138:0:0x500", "--stub", "slot:0xbc:0", "--stub", "0x6e1370:1", "--stub", "0x5e4a60:7", "--stub", "0x64cd20:4", "--stub", "0x646d50:1", "--stub", "slot:0x158:3", "--stub", "0x647080:3", "--stub", "0x647970:0", "--stub", "0x8544a0:4", "--stub", "0x64d8c0:0", "--stub", "slot:0x30:0", "--stub", "0x5f9780:0", "--stub", "slot:0x174:1", "--stub", "0x5f96b0:4", "--stub", "0x5d9010:2", "--stub", "0x5d86f0:3", "--stub", "run:0xa46a00:0", "--slotbase", "0xbc:0x46ce30", "--slotbase", "0x158:0x60eda0", "--answers", "0:0,1", "--seq", "0:2", "--answers", "1:0,1", "--answers", "4:0,1,2", "--answers", "10:0,0,1", "--outarg", "8:0:-5000..5000", "--wbias", "m+0x10:0x63637,0x6360c,0x63641,0x63686,0x636db,0x63710,0x63755,0x637aa,0x637ef,0x63424,0x63479,0x634be,0x634f3,0x634c8,0x6350d,0x63542,0x63587,0x635dc,0x63211,0x63256,0x632ab,0x632e0,0x63325,0x6337a,0x633bf,0x633f4,0x633c9,0x6300e,0x63043,0x63098,0x630dd,0x63112,0x63157,0x631ac,0x631e1,0x63e26,0x63e7b,0x63eb0,0x63ef5,0x63eca,0x63f0f,0x63f44,0x63f99,0x63fde", "--wbias", "m+0x14:0x63637,0x6360c,0x63641,0x63686,0x636db,0x63710,0x63755,0x637aa,0x637ef,0x63424,0x63479,0x634be,0x634f3,0x634c8,0x6350d,0x63542,0x63587,0x635dc,0x63211,0x63256,0x632ab,0x632e0,0x63325,0x6337a,0x633bf,0x633f4,0x633c9,0x6300e,0x63043,0x63098,0x630dd,0x63112,0x63157,0x631ac,0x631e1,0x63e26,0x63e7b,0x63eb0,0x63ef5,0x63eca,0x63f0f,0x63f44,0x63f99,0x63fde", "--wbias", "m+0xb4:0x100,0x100,0x100,0x100,0x100,0x100,0x100,0x100,0x200,0x300,0x400,0x500,0x600,0xa00,0x700,0xff00,0x8000", "--wbias", "m+0xa8:0x50000,0x80000,0x0,0x10000,0x30000,0x50000,0x80000", "--wbias", "c1+0:1..8", "--wbias", "c1+0x4:1..8", "--wbias", "c1+0x18:1..24", "--wbias", "p11+0x218:0,0,1,1,2", "--wbias", "p11+0x224:0,0,32,64,100,200,-50,1000,5000", "--record", "1", "--record", "2", "--record", "3", "--record", "4", "--record", "5", "--record", "6", "--record", "7", "--record", "8", "--record", "9", "--record", "10", "--record", "11", "--record", "12", "--record", "13", "--record", "14", "--record", "15", "--record-order", "--rows", "3000", "--init-call", "0xa46980", "--port", "{\n                // **No crate method exists for this function**: the port spreads it over `collide.rs`'s\n                // `Sim::set_new_location` (the move, the shore half, the goody box), `lib.rs`'s `crew_des`\n                // (guy 0 and the tracked crew), `vision::moved_to` (the half-cell test) and `transport.rs`\n                // (the transport spells). It has no `_steps` over a view, and it does not model the squad\n                // arm at all (`guy_mark` is 1 on every dumped unit), the Iroquois `unit_masks & 0x800`\n                // write, or `update_ceo_position` and the `+0x174` slot. So this block is an inline copy,\n                // read from the listing, over the same bytes; it calls the crate's own `sin_component`,\n                // `fog_of` and the world-cell constants where the crate has them.\n                //\n                // `a`: 0 `x`, 1 `y`, 2 `param_3` (move the guys), 3 `param_4` (turn them); then stub\n                // k's answer at `4 + k`, 21 what `find_tcoord_z` writes through its first argument, 22\n                // and 23 the `+0xbc` slot's first and second answers, 24 whether `+0xbc` is laid as\n                // `UnitData::is_on_map`, 25 whether `+0x158` is laid as `Object::die`.\n                // Windows: `c[0]` the `div_3_table`, `c[1]` the world record, `c[2]` the terrain cell's\n                // record, `p[0]` the guys' pointer array, `p[1..=10]` the guys, `p[11]` the type record,\n                // `p[12]` the world's `+0x134` table, `p[13]` its `+0x138` table.\n                use crate::movement::{Angle, sin_component};\n                use crate::vision::fog_of;\n                use crate::world::{UNITS_PER_CELL, UNITS_PER_TILE};\n                const KEY: i32 = 0x63637;\n                fn rd32(w: &[u8], o: usize) -> i32 {\n                    i32::from_le_bytes(w[o..o + 4].try_into().expect(\"four bytes\"))\n                }\n                fn wr32(w: &mut [u8], o: usize, v: i32) {\n                    w[o..o + 4].copy_from_slice(&v.to_le_bytes());\n                }\n                fn rd16(w: &[u8], o: usize) -> u16 {\n                    u16::from_le_bytes(w[o..o + 2].try_into().expect(\"two bytes\"))\n                }\n                fn cell(v: i32) -> i32 {\n                    v.div_euclid(UNITS_PER_CELL)\n                }\n                fn tile(v: i32) -> i32 {\n                    v.div_euclid(UNITS_PER_TILE)\n                }\n                fn half(v: i32) -> i32 {\n                    (v + i32::from(v < 0)) >> 1\n                }\n                fn model(a: &[i32], m: &mut [u8], c: &[Vec<u8>], p: &mut [Vec<u8>], calls: &mut Vec<String>) -> i32 {\n                    let _ = c;\n                    let (x, y, p3, p4) = (a[0], a[1], a[2], a[3]);\n                    let ans = |k: usize| a[4 + k];\n                    let (out_z, seq, is_on_map_base, die_base) = (a[21], [a[22], a[23]], a[24] == 1, a[25] == 1);\n                    let this = \"m+0x0\";\n                    let (ox, oy) = (rd32(m, 0x10) ^ KEY, rd32(m, 0x14) ^ KEY);\n                    let (otx, oty) = (tile(ox), tile(oy));\n                    let (ocx, ocy) = (cell(ox), cell(oy));\n                    let mut asked = 0usize;\n                    let mut on_map = |m: &[u8]| -> bool {\n                        let r = if is_on_map_base {\n                            i32::from(rd16(m, 0x82) >> 15)\n                        } else {\n                            asked += 1;\n                            seq[asked - 1]\n                        };\n                        r != 0\n                    };\n                    let cell_change = on_map(m) && !(ocx == cell(x) && ocy == cell(y));\n                    let tile_change = on_map(m) && !(otx == tile(x) && oty == tile(y));\n                    let ty218 = rd32(&p[11], 0x218);\n                    let (w0, w4, w18) = (rd32(&c[1], 0), rd32(&c[1], 4), rd32(&c[1], 0x18));\n                    let tile_bits = |p: &[Vec<u8>]| -> u16 {\n                        rd16(&p[13], (2 * w18.wrapping_mul(tile(y)).wrapping_add(tile(x))) as usize)\n                    };\n                    let mut enter = cell_change;\n                    if !cell_change && tile_change {\n                        let bit = (rd16(&p[12], (0x1c * (w0 * ocy + ocx)) as usize) >> 8) & 1;\n                        let owner = i32::from(m[9]);\n                        calls.push(format!(\"s1:{},18\", 0x00e3_a390 + owner * 0x6eec));\n                        if ans(1) != 0 && ty218 == 0 {\n                            let flags = rd32(m, 0x68);\n                            let on = tile_bits(p) & 0x30 == 0x30;\n                            wr32(m, 0x68, if on { flags | 0x800 } else { flags & !0x800 });\n                        }\n                        enter = bit != 0;\n                    }\n                    if enter {\n                        let (f68, f6c) = (rd32(m, 0x68), rd32(m, 0x6c));\n                        if (f68 & 0x800000 != 0 && f6c & 0x2000 == 0) || p[11][0x2b4] & 0x10 != 0 {\n                            if ty218 == 0 && tile_bits(p) & 0x30 == 0x20 {\n                                calls.push(format!(\"s2:{this},-1,-1,-1,-1,650,0,0\"));\n                                return 0;\n                            }\n                            if ty218 == 1 && tile_bits(p) & 0x30 != 0x20 {\n                                calls.push(format!(\"s3:{this},0,-1,1,1\"));\n                                calls.push(format!(\"s4:{this},1\"));\n                                if ans(4) == 0 {\n                                    let slot = if die_base { 6 } else { 5 };\n                                    calls.push(format!(\"s{slot}:{this},0,-1,0\"));\n                                }\n                                return 0;\n                            }\n                        }\n                        if cell_change {\n                            calls.push(format!(\"s7:{this}\"));\n                        }\n                    }\n                    wr32(m, 0x10, x ^ KEY);\n                    wr32(m, 0x14, y ^ KEY);\n                    calls.push(format!(\"s8:c2+0x0,sp-0x34,{},{},1\", tile(x), tile(y)));\n                    wr32(m, 0xc, out_z ^ KEY);\n                    if cell_change {\n                        calls.push(format!(\"s9:{this}\"));\n                        calls.push(format!(\"s10:{this}\"));\n                        if ans(10) == 0\n                            && rd32(m, 0x68) & 1 == 0\n                            && ty218 == 0\n                            && (rd16(&p[12], (0x1c * (w0 * cell(y) + cell(x))) as usize) as i16) < 0\n                        {\n                            calls.push(format!(\"s11:{this}\"));\n                        }\n                    }\n                    if tile_change {\n                        if (otx >> 1) != fog_of(x) || (oty >> 1) != fog_of(y) {\n                            calls.push(format!(\"s12:{this},{}\", i32::from(p3 == 0)));\n                        }\n                        if rd32(m, 0x6c) & 0x10000 != 0 {\n                            calls.push(format!(\"s13:{this},{otx},{oty},{},{}\", tile(x), tile(y)));\n                        }\n                    }\n                    let n = i32::from(m[0xb5] as i8);\n                    let angle = rd32(m, 0x50);\n                    let (nx, ny) = (rd32(m, 0x10) ^ KEY, rd32(m, 0x14) ^ KEY);\n                    if m[0xb5] == 1 {\n                        if p4 != 0 {\n                            calls.push(format!(\"s14:p1+0x0,{angle},{p3}\"));\n                        }\n                        wr32(&mut p[1], 0x5c, nx);\n                        wr32(&mut p[1], 0x60, ny);\n                        if p3 != 0 {\n                            calls.push(format!(\"s15:p1+0x0,{nx},{ny},1\"));\n                        }\n                        return 1;\n                    }\n                    let aa = m[0xaa];\n                    let mag = (i32::from(aa == 5) + 1).wrapping_mul(rd32(&p[11], 0x224));\n                    let mut cosc = if mag == 0 { 0 } else { sin_component(Angle(angle.wrapping_sub(0x4000_0000)), mag) };\n                    let mut sinc = if mag == 0 { 0 } else { sin_component(Angle(angle), mag) };\n                    let per_row = if aa == 8 { i32::from(n != 1) + 1 } else { n.min(3) };\n                    let q = (n - 1) / per_row;\n                    if rd32(m, 0x68) & 2 != 0 {\n                        cosc = cosc.wrapping_neg();\n                        sinc = sinc.wrapping_neg();\n                    }\n                    let rows1 = per_row - 1;\n                    let off_x = half(q.wrapping_mul(sinc)).wrapping_sub(half(rows1.wrapping_mul(cosc)));\n                    let off_y = half(q.wrapping_mul(cosc)).wrapping_add(half(rows1.wrapping_mul(sinc)));\n                    for i in 0..n.max(0) {\n                        let (row, col) = (i / per_row, i % per_row);\n                        let w = 1 + i as usize;\n                        if p4 != 0 {\n                            calls.push(format!(\"s14:p{w}+0x0,{angle},{p3}\"));\n                        }\n                        let mut gx = col.wrapping_mul(cosc).wrapping_sub(row.wrapping_mul(sinc)).wrapping_add(nx).wrapping_add(off_x);\n                        let mut gy = ny.wrapping_sub(col.wrapping_mul(sinc)).wrapping_sub(row.wrapping_mul(cosc)).wrapping_add(off_y);\n                        let (lx, ly) = (w0.wrapping_mul(0x300), w4.wrapping_mul(0x300));\n                        if gx >= lx {\n                            gx = lx - 1;\n                        } else if gx < 0 {\n                            gx = 0;\n                        }\n                        if gy >= ly {\n                            gy = ly - 1;\n                        } else if gy < 0 {\n                            gy = 0;\n                        }\n                        wr32(&mut p[w], 0x5c, gx);\n                        wr32(&mut p[w], 0x60, gy);\n                        if p3 != 0 {\n                            calls.push(format!(\"s15:p{w}+0x0,{gx},{gy},1\"));\n                        }\n                    }\n                    1\n                }\n                let mut calls = Vec::new();\n                let r = model(a, m, c, p, &mut calls);\n                if calls.is_empty() {\n                    calls.push(\"-\".to_string());\n                }\n                log.extend(calls);\n                i64::from(r)\n            }"]
