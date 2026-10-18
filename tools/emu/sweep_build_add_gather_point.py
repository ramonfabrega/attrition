#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_add_gather_point.py — `Build::add_gather_point@00622e70` under unicorn (item 3272, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_build_add_gather_point.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 256-byte object of random bytes (no bias), 4 stack argument(s);
`p` windows in order: p0..p25 the place(s), in flag order; the global cell 0x00c0618c points at 384 random bytes; the global cell 0x00c8d900 points at 32 random bytes; 4 random bytes at 0x00c8d908; 16 random bytes at 0x00e90000; 16 random bytes at 0x00e90010; 16 random bytes at 0x00e90020; 16 random bytes at 0x00e90030; 16 random bytes at 0x00e90040; 16 random bytes at 0x00e90050; 16 random bytes at 0x00e90100; 16 random bytes at 0x00e90110; 16 random bytes at 0x00e91000; 16 random bytes at 0x00e91010; 16 random bytes at 0x00e91020; 16 random bytes at 0x00e91030; 16 random bytes at 0x00e91040; 16 random bytes at 0x00e91050; 16 random bytes at 0x00e91100; callee 0x00623180 (0 args) stubbed; the import cell 0x00ac54f0 holds a stub (1 args); callee 0x00470c60 (1 args) run, not hooked; callee 0x00609dc0 (1 args) stubbed; callee 0x0060b030 (0 args) stubbed; callee 0x006179d0 (0 args) stubbed; callee 0x005e48c0 (7 args) stubbed; callee 0x005e4350 (6 args) stubbed; callee 0x0046edd0 (1 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x30 (0 args) stubbed; vtable slot +0x84 (0 args) stubbed; vtable slot +0x8c (0 args) stubbed; the word at `0xc0aec0+u8(m,9)*0x1c` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `p0+4` points at 128 random bytes less `0` (placed); the word at `p0+8` points at 128 random bytes less `0` (placed); the word at `p1+0x18` points at 560 random bytes less `0` (placed); the word at `p2+0x18` points at 560 random bytes less `0` (placed); the word at `p3+0x18` points at 560 random bytes less `0` (placed); the word at `p1+0x24` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p2+0x24` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p3+0x24` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p1+0x28` points at 64 random bytes less `0` (placed); the word at `p2+0x28` points at 64 random bytes less `0` (placed); the word at `p3+0x28` points at 64 random bytes less `0` (placed); the word at `p10+0x20` points at 32 random bytes less `0` (placed); the word at `p11+0x20` points at 32 random bytes less `0` (placed); the word at `p12+0x20` points at 32 random bytes less `0` (placed); the word at `p10+0x24` points at 96 random bytes less `0` (placed, null a third of the rows); the word at `p11+0x24` points at 96 random bytes less `0` (placed, null a third of the rows); the word at `p12+0x24` points at 96 random bytes less `0` (placed, null a third of the rows); the word at `p16+0x14` points at 64 random bytes less `0` (placed); the word at `p17+0x14` points at 64 random bytes less `0` (placed); the word at `p18+0x14` points at 64 random bytes less `0` (placed); the word at `p16+0x30` points at 64 random bytes less `0` (placed); the word at `p17+0x30` points at 64 random bytes less `0` (placed); the word at `p18+0x30` points at 64 random bytes less `0` (placed); the word at `m+0x18` points at 32 random bytes less `0` (placed); stub 3 answered from [0, 0, 0, 0, 0, 1, -1] (90%); stub 9 answered from [1, 1, 1, 0] (90%); stub 10 answered from [1, 1, 1, 0] (90%); the stubbed vtable laid at `p25+0`; the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 1 answers 15274240..15274240 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 3 writes a value in [0, 0, 0, 1] through its argument 0, printed after the answers (`o` in a place); stub 1 is `__cdecl`: the caller pops its arguments; stub 4 answers from its own `ecx`: the word at `+0x24`; stub 5 answers from its own `ecx`: the word at `+0x28`; stub 11 answers from its own `ecx`: the word at `+0x20`; stub 12 answers from its own `ecx`: the word at `+0x20`; stub 13 answers from its own `ecx`: the word at `+0x24`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`.
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

ENTRY = 0x00622e70
WIN = 256
OWN = 4
BIAS = []
ARGSET = [(2, [3, 0, 1, 2, 259])]
ARGFROM = []
CELLS = [(12607884, 384), (13162752, 32)]
AT = [(13162760, 4), (15269888, 16), (15269904, 16), (15269920, 16), (15269936, 16), (15269952, 16), (15269968, 16), (15270144, 16), (15270160, 16), (15273984, 16), (15274000, 16), (15274016, 16), (15274032, 16), (15274048, 16), (15274064, 16), (15274240, 16)]
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('call', 6435200, 0), ('iat', 11293936, 1), ('run', 4656224, 1), ('call', 6331840, 1), ('call', 6336560, 0), ('call', 6388176, 0), ('call', 6179008, 7), ('call', 6177616, 6), ('call', 4648400, 1), ('slot', 96, 2), ('slot', 184, 2), ('slot', 48, 0), ('slot', 132, 0), ('slot', 140, 0)]
PLACES = [(None, '0xc0aec0+u8(m,9)*0x1c', '0', 32), ('p0', '0', '0', 128), ('p0', '4', '0', 128), ('p0', '8', '0', 128), ('p1', '0x18', '0', 560), ('p2', '0x18', '0', 560), ('p3', '0x18', '0', 560), ('p1', '0x24', '0', 64), ('p2', '0x24', '0', 64), ('p3', '0x24', '0', 64), ('p1', '0x28', '0', 64), ('p2', '0x28', '0', 64), ('p3', '0x28', '0', 64), ('p10', '0x20', '0', 32), ('p11', '0x20', '0', 32), ('p12', '0x20', '0', 32), ('p10', '0x24', '0', 96), ('p11', '0x24', '0', 96), ('p12', '0x24', '0', 96), ('p16', '0x14', '0', 64), ('p17', '0x14', '0', 64), ('p18', '0x14', '0', 64), ('p16', '0x30', '0', 64), ('p17', '0x30', '0', 64), ('p18', '0x30', '0', 64), ('m', '0x18', '0', 32)]
NULLS = [7, 8, 9, 16, 17, 18]
NULLFORCE = {}
WBIAS = [('c0', 348, ('set', [0, 1, 2, 3, 3, 3])), ('c0', 352, ('set', [0, 1, 2, 3, 3, 3])), ('c0', 356, ('set', [0, 1, 2, 3, 3, 3])), ('m', 8, ('set', [0, 0, 0, 0, 0, 0, 65536, 256, 4294901760, 65792])), ('m', 96, ('set', [8, 8, 8, 8, 8, 0, 16, 9, 24])), ('m', 204, ('set', [0, 15269888, 15269904, 15269920, 15269936, 15269952, 15269968])), ('c1', 0, ('set', [15270144])), ('c1', 4, ('set', [15270160])), ('c2', 0, ('set', [1, 2])), ('c3', 0, ('set', [15269904])), ('c3', 4, ('set', [15269920])), ('c3', 8, ('set', [15273984])), ('c4', 0, ('set', [15269920])), ('c4', 4, ('set', [15269888])), ('c4', 8, ('set', [15274000])), ('c5', 0, ('set', [15269888])), ('c5', 4, ('set', [15269904])), ('c5', 8, ('set', [15274016])), ('c6', 0, ('set', [15269936])), ('c6', 4, ('set', [15269936])), ('c6', 8, ('set', [15274032])), ('c7', 0, ('set', [15269968])), ('c7', 4, ('set', [15269968])), ('c7', 8, ('set', [15274048])), ('c8', 0, ('set', [15269952])), ('c8', 4, ('set', [15269952])), ('c8', 8, ('set', [15274064])), ('c11', 12, ('set', [3, 3, 0, 1, 2, 259])), ('c12', 12, ('set', [3, 3, 0, 1, 2, 259])), ('c13', 12, ('set', [3, 3, 0, 1, 2, 259])), ('c14', 12, ('set', [3, 3, 0, 1, 2, 259])), ('c15', 12, ('set', [3, 3, 0, 1, 2, 259])), ('c16', 12, ('set', [3, 3, 0, 1, 2, 259])), ('p1', 8, ('set', [1, 1, 1, 1, 0, 3, 2])), ('p2', 8, ('set', [1, 1, 1, 1, 0, 3, 2])), ('p3', 8, ('set', [1, 1, 1, 1, 0, 3, 2])), ('p4', 536, ('set', [2, 2, 2, 2, 1, 0])), ('p5', 536, ('set', [2, 2, 2, 2, 1, 0])), ('p6', 536, ('set', [2, 2, 2, 2, 1, 0])), ('p7', 32, ('set', [0, 1, 1])), ('p8', 32, ('set', [0, 1, 1])), ('p9', 32, ('set', [0, 1, 1])), ('p16', 8, ('set', [0, 1, 2, 3])), ('p17', 8, ('set', [0, 1, 2, 3])), ('p18', 8, ('set', [0, 1, 2, 3])), ('p13', 12, ('set', [0, 0, 5, -7, 100])), ('p14', 12, ('set', [0, 0, 5, -7, 100])), ('p15', 12, ('set', [0, 0, 5, -7, 100])), ('p13', 16, ('set', [0, 0, 3, -9, 100])), ('p14', 16, ('set', [0, 0, 3, -9, 100])), ('p15', 16, ('set', [0, 0, 3, -9, 100]))]
FILLS = []
ARGRANGE = [(3, 0, 2)]
SLOTBASE = []
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(3, [0, 0, 0, 0, 0, 1, -1]), (9, [1, 1, 1, 0]), (10, [1, 1, 1, 0])]
VTAT = ('p25', 0)
VTATHALF = []
VTIMAGE = [('m', 0, 11805044)]
ALIAS = []
ANSWERBY = []
ANSWERWIN = []
CDECL = False
TIB = True
ARGEQ = []
SEQ = []
ANSRANGE = [(1, 15274240, 15274240)]
CELLTHIS = []
VTALSO = [('p7', 0), ('p8', 0), ('p9', 0), ('p10', 0), ('p11', 0), ('p12', 0)]
SPARSE = []
GTAB = []
RECORD = [0, 3, 6, 7, 8, 9, 10]
RECORDORDER = True
RECORDEDX = False
ROWSPLIT = {}
LINKS = []
LINKHI = []
OUTARG = [(3, 0, ('set', [0, 0, 0, 1]))]
CDECLSTUBS = [1]
STUBWRITE = []
STUBCOPY = []
STACKCLEAN = 0
INITCALLS = []
MAXINSNS = 200000
ECXANSWER = [(4, 36, None), (5, 40, None), (11, 32, None), (12, 32, None), (13, 36, None)]
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
    rng = random.Random(3272)
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

# sweep_gen: ["obj", "Build::add_gather_point@00622e70", "--item", "3272", "--tib", "--win", "0x100", "--own", "4", "--ret", "none", "--rows", "3000", "--cell-win", "0xc0618c:0x180", "--cell-win", "0xc8d900:0x20", "--at", "0xc8d908:4", "--at", "0xe90000:16", "--at", "0xe90010:16", "--at", "0xe90020:16", "--at", "0xe90030:16", "--at", "0xe90040:16", "--at", "0xe90050:16", "--at", "0xe90100:16", "--at", "0xe90110:16", "--at", "0xe91000:16", "--at", "0xe91010:16", "--at", "0xe91020:16", "--at", "0xe91030:16", "--at", "0xe91040:16", "--at", "0xe91050:16", "--at", "0xe91100:16", "--stub", "0x623180:0", "--stub", "iat:0xac54f0:1", "--stub", "run:0x470c60:1", "--stub", "0x609dc0:1", "--stub", "0x60b030:0", "--stub", "0x6179d0:0", "--stub", "0x5e48c0:7", "--stub", "0x5e4350:6", "--stub", "0x46edd0:1", "--stub", "slot:0x60:2", "--stub", "slot:0xb8:2", "--stub", "slot:0x30:0", "--stub", "slot:0x84:0", "--stub", "slot:0x8c:0", "--stub-cdecl", "1", "--ecx-answer", "4:0x24", "--ecx-answer", "5:0x28", "--ecx-answer", "11:0x20", "--ecx-answer", "12:0x20", "--ecx-answer", "13:0x24", "--answer-range", "1:0xe91100:0xe91100", "--answers", "3:0,0,0,0,0,1,-1", "--answers", "9:1,1,1,0", "--answers", "10:1,1,1,0", "--outarg", "3:0:0,0,0,1", "--place", "0xc0aec0+u8(m,9)*0x1c:0:0x20", "--place", "p0+0:0:0x80", "--place", "p0+4:0:0x80", "--place", "p0+8:0:0x80", "--place", "p1+0x18:0:0x230", "--place", "p2+0x18:0:0x230", "--place", "p3+0x18:0:0x230", "--place", "p1+0x24:0:0x40", "--place", "p2+0x24:0:0x40", "--place", "p3+0x24:0:0x40", "--place", "p1+0x28:0:0x40", "--place", "p2+0x28:0:0x40", "--place", "p3+0x28:0:0x40", "--place", "p10+0x20:0:0x20", "--place", "p11+0x20:0:0x20", "--place", "p12+0x20:0:0x20", "--place", "p10+0x24:0:0x60", "--place", "p11+0x24:0:0x60", "--place", "p12+0x24:0:0x60", "--place", "p16+0x14:0:0x40", "--place", "p17+0x14:0:0x40", "--place", "p18+0x14:0:0x40", "--place", "p16+0x30:0:0x40", "--place", "p17+0x30:0:0x40", "--place", "p18+0x30:0:0x40", "--place", "m+0x18:0:0x20", "--place-null", "7", "--place-null", "8", "--place-null", "9", "--place-null", "16", "--place-null", "17", "--place-null", "18", "--vtat", "p25+0", "--vtat-also", "p7+0", "--vtat-also", "p8+0", "--vtat-also", "p9+0", "--vtat-also", "p10+0", "--vtat-also", "p11+0", "--vtat-also", "p12+0", "--vtat-image", "m+0:0xb42174", "--argset", "2:3,0,1,2,259", "--argrange", "3:0:2", "--wbias", "c0+0x15c:0,1,2,3,3,3", "--wbias", "c0+0x160:0,1,2,3,3,3", "--wbias", "c0+0x164:0,1,2,3,3,3", "--wbias", "m+0x8:0x0,0x0,0x0,0x0,0x0,0x0,0x10000,0x100,0xffff0000,0x10100", "--wbias", "m+0x60:8,8,8,8,8,0,0x10,9,0x18", "--wbias", "m+0xcc:0,0xe90000,0xe90010,0xe90020,0xe90030,0xe90040,0xe90050", "--wbias", "c1+0:0xe90100", "--wbias", "c1+4:0xe90110", "--wbias", "c2+0:1,2", "--wbias", "c3+0:0xe90010", "--wbias", "c3+4:0xe90020", "--wbias", "c3+8:0xe91000", "--wbias", "c4+0:0xe90020", "--wbias", "c4+4:0xe90000", "--wbias", "c4+8:0xe91010", "--wbias", "c5+0:0xe90000", "--wbias", "c5+4:0xe90010", "--wbias", "c5+8:0xe91020", "--wbias", "c6+0:0xe90030", "--wbias", "c6+4:0xe90030", "--wbias", "c6+8:0xe91030", "--wbias", "c7+0:0xe90050", "--wbias", "c7+4:0xe90050", "--wbias", "c7+8:0xe91040", "--wbias", "c8+0:0xe90040", "--wbias", "c8+4:0xe90040", "--wbias", "c8+8:0xe91050", "--wbias", "c11+0xc:3,3,0,1,2,0x103", "--wbias", "c12+0xc:3,3,0,1,2,0x103", "--wbias", "c13+0xc:3,3,0,1,2,0x103", "--wbias", "c14+0xc:3,3,0,1,2,0x103", "--wbias", "c15+0xc:3,3,0,1,2,0x103", "--wbias", "c16+0xc:3,3,0,1,2,0x103", "--wbias", "p1..3+8:1,1,1,1,0,3,2", "--wbias", "p4..6+0x218:2,2,2,2,1,0", "--wbias", "p7..9+0x20:0,1,1", "--wbias", "p16..18+8:0,1,2,3", "--wbias", "p13..15+0xc:0,0,5,-7,100", "--wbias", "p13..15+0x10:0,0,3,-9,100", "--record", "0", "--record", "3", "--record", "6", "--record", "7", "--record", "8", "--record", "9", "--record", "10", "--record-order", "--port", "            // `a[0..4]` the point's `x`, `y`, `action` (the low byte is stored) and the queue position (2 `QUEUE_NEW`,\n            // 0 `QUEUE_FIRST`, 1 `QUEUE_LAST`); `a[4 + i]` stub `i`'s answer, `a[18]` the player stub 3 writes through its\n            // argument. Stubs: 0 `clear_gather`, 1 `malloc` (always the point at `0xe91100`, `c[17]`), 2 `LinkListBase::add`\n            // (run, not hooked: with `Recycler::pop` over `c[1]`, `c[2]`), 3 `UnitData::home_base`, 4 `get_order`, 5\n            // `update_order`, 6 `add_strafe_order`, 7 `add_air_patrol_order`, 8 `SimpleArray::make_valid`, 9 and 10 the\n            // `is(0x1bf, 1)` asks (the type record's slot `+0x60` on the `Build::vftable` arm, else `this`'s `+0xb8`), 11 the\n            // order's `is_air`, 12 and 13 `update_order()`'s slots `+0x84` (the heights) and `+0x8c` (the patrol).\n            // Windows: `m` the building; `c[0]` the objects singleton (`+0x15c` the unit counts), `c[1]` the pool's array\n            // (`c[2]` its count), `c[3..9]` list nodes at `0xe90000 + 0x10 k`, `c[9..11]` the pool's nodes, `c[11..17]` the\n            // nodes' points at `0xe91000 + 0x10 k`, `c[17]` the new point; `p[0]` the owner's units array, `p[1..4]` the\n            // units, `p[4..7]` their types, `p[7..10]` their orders, `p[10..13]` their `update_order()`s, `p[13..16]` the\n            // heights, `p[16..19]` the patrols (`p[19..22]` the x arrays, `p[22..25]` the y arrays), `p[25]` this's type.\n            //\n            // **The port is the crate's own steps**, `crate::rally::add_gather_point_steps`, which `Sim::add_gather_point`\n            // calls over its `GatherPointView`; here the view is the emulated original's windows. The list is the\n            // original's circular `LinkListBase` laid in the windows (its cursor and length with it: this crate keeps a\n            // `Vec`), and the sixth argument of `add_air_patrol_order` is a stale `ecx` the view tracks (`stale`).\n            use crate::orders::QueuePos;\n            use crate::rally::{GatherPoint, GatherPointView, add_gather_point_steps};\n            struct V<'a> {\n                a: &'a [i32],\n                m: &'a mut [u8],\n                c: &'a mut [Vec<u8>],\n                p: &'a mut [Vec<u8>],\n                log: &'a mut Vec<String>,\n                stale: String,\n            }\n            /// The window of the table at an image address, and its name.\n            fn cw(va: i32) -> usize {\n                let va = va as u32;\n                match va {\n                    0xe90000..=0xe9005f => 3 + ((va - 0xe90000) / 16) as usize,\n                    0xe90100 => 9,\n                    0xe90110 => 10,\n                    0xe91000..=0xe9105f => 11 + ((va - 0xe91000) / 16) as usize,\n                    0xe91100 => 17,\n                    _ => panic!(\"address {va:#x} is no table\"),\n                }\n            }\n            /// The table `k`'s image address.\n            fn tab(k: usize) -> u32 {\n                match k {\n                    3..=8 => 0xe90000 + 0x10 * (k as u32 - 3),\n                    9 | 10 => 0xe90100 + 0x10 * (k as u32 - 9),\n                    11..=16 => 0xe91000 + 0x10 * (k as u32 - 11),\n                    _ => 0xe91100,\n                }\n            }\n            /// The recorder's name for an image address: the first table with `at <= va <= at + 16`, so a node that\n            /// starts where another ends is named by that one (`c3+0x10` for `c4`).\n            fn tok(va: i32) -> String {\n                let k = (3..18)\n                    .find(|&k| (tab(k)..=tab(k) + 16).contains(&(va as u32)))\n                    .expect(\"a table\");\n                format!(\"c{k}+{:#x}\", va as u32 - tab(k))\n            }\n            impl V<'_> {\n                fn player(&self) -> usize {\n                    usize::from(self.m[9])\n                }\n                fn id(&self) -> i32 {\n                    i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]))\n                }\n            }\n            impl GatherPointView for V<'_> {\n                fn clear_gather(&mut self) {\n                    self.log.push(\"s0:m+0x0\".into());\n                }\n                fn add(&mut self, q: GatherPoint, pos: QueuePos) {\n                    // `malloc(0x10)` answers `c[17]`: the `GatherPoint`.\n                    let pt = &mut self.c[17];\n                    wr(pt, 0, 0xb42168u32 as i32);\n                    wr(pt, 4, q.pos.x);\n                    wr(pt, 8, q.pos.y);\n                    pt[0xc] = q.action;\n                    // `Recycler::pop`, then `LinkListBase::add`.\n                    let n = rd(&self.c[2], 0);\n                    let n = n.max(1) - 1;\n                    wr(&mut self.c[2], 0, n);\n                    let node = rd(&self.c[1], 4 * n as usize);\n                    let nw = cw(node);\n                    self.c[nw][0xc] = 0;\n                    wr(&mut self.c[nw], 8, 0xe91100);\n                    wr(&mut self.c[nw], 4, 0);\n                    wr(&mut self.c[nw], 0, 0);\n                    let head = rd(self.m, 0xcc);\n                    if head != 0 {\n                        let hw = cw(head);\n                        let tail = rd(&self.c[hw], 4);\n                        wr(&mut self.c[nw], 0, head);\n                        wr(&mut self.c[nw], 4, tail);\n                        let tw = cw(tail);\n                        wr(&mut self.c[tw], 0, node);\n                        wr(&mut self.c[hw], 4, node);\n                    } else {\n                        wr(&mut self.c[nw], 0, node);\n                        wr(&mut self.c[nw], 4, node);\n                    }\n                    wr(self.m, 0xc8, rd(self.m, 0xc8).wrapping_add(1));\n                    wr(self.m, 0xcc, node);\n                    wr(self.m, 0xc4, node);\n                    wr(self.m, 0xbc, 0xe91100);\n                    self.m[0xc0] = self.c[nw][0xc];\n                    if pos == QueuePos::Last {\n                        wr(self.m, 0xcc, rd(&self.c[nw], 0));\n                    }\n                }\n                fn hangar(&mut self) -> bool {\n                    if rd(self.m, 0x60) & 8 == 0 {\n                        return false;\n                    }\n                    if rd(self.m, 0) as u32 == 0xb42174 {\n                        self.log.push(\"s9:p25+0x0,447,1\".into());\n                        self.a[4 + 9] != 0\n                    } else {\n                        self.log.push(\"s10:m+0x0,447,1\".into());\n                        self.a[4 + 10] != 0\n                    }\n                }\n                fn planes(&mut self) -> usize {\n                    rd(&self.c[0], 0x15c + 4 * self.player()).max(0) as usize\n                }\n                fn homed_plane(&mut self, i: usize) -> bool {\n                    if rd(&self.p[1 + i], 8) & 1 == 0 || rd(&self.p[4 + i], 0x218) != 2 {\n                        return false;\n                    }\n                    self.log.push(format!(\"s3:p{}+0x0,sp-0x34\", 1 + i));\n                    // The answer is the row's `a[7]`; the player goes out through the argument.\n                    self.id() == self.a[4 + 3] && self.player() as i32 == self.a[18]\n                }\n                fn heights(&mut self, i: usize) -> Option<(i32, i32)> {\n                    if rd(&self.p[1 + i], 0x24) == 0 || rd(&self.p[7 + i], 0x20) == 0 {\n                        return None;\n                    }\n                    let h = &self.p[13 + i];\n                    Some((rd(h, 0xc), rd(h, 0x10)))\n                }\n                fn points(&mut self) -> Vec<GatherPoint> {\n                    let head = rd(self.m, 0xcc);\n                    self.stale = tok(head);\n                    let mut out = Vec::new();\n                    let mut node = head;\n                    loop {\n                        let nw = &self.c[cw(node)];\n                        let d = &self.c[cw(rd(nw, 8))];\n                        out.push(GatherPoint {\n                            pos: crate::Pos {\n                                x: rd(d, 4),\n                                y: rd(d, 8),\n                            },\n                            action: d[0xc],\n                        });\n                        node = rd(nw, 0);\n                        if node == head {\n                            break;\n                        }\n                    }\n                    // The cursor ends on the head after the lap.\n                    let hw = &self.c[cw(head)];\n                    let (data, metric) = (rd(hw, 8), hw[0xc]);\n                    wr(self.m, 0xc4, head);\n                    wr(self.m, 0xbc, data);\n                    self.m[0xc0] = metric;\n                    out\n                }\n                fn strafe(&mut self, i: usize, q: GatherPoint) {\n                    self.log.push(format!(\n                        \"s6:p{}+0x0,{},{},{},{},1,2,0\",\n                        1 + i,\n                        q.pos.x,\n                        q.pos.y,\n                        self.id(),\n                        self.player()\n                    ));\n                    self.stale = format!(\"p{}+0x0\", 1 + i);\n                }\n                fn air_patrol(&mut self, i: usize, q: GatherPoint) -> bool {\n                    self.log.push(format!(\n                        \"s7:p{}+0x0,{},{},{},{},1,{}\",\n                        1 + i,\n                        q.pos.x,\n                        q.pos.y,\n                        self.id(),\n                        self.player(),\n                        self.stale\n                    ));\n                    self.stale = format!(\"p{}+0x0\", 10 + i);\n                    rd(&self.p[10 + i], 0x24) != 0\n                }\n                fn patrol_push(&mut self, i: usize, q: GatherPoint) {\n                    let idx = rd(&self.p[16 + i], 8);\n                    self.log.push(format!(\"s8:p{}+0x4,{idx}\", 16 + i));\n                    self.log.push(format!(\"s8:p{}+0x20,{idx}\", 16 + i));\n                    wr(&mut self.p[19 + i], 4 * idx as usize, q.pos.x);\n                    wr(&mut self.p[22 + i], 4 * idx as usize, q.pos.y);\n                    self.stale = format!(\"p{}+0x0\", 22 + i);\n                }\n                fn patrol_heights(&mut self, i: usize, alt: i32, sharp: i32) {\n                    wr(&mut self.p[16 + i], 0x4c, alt);\n                    wr(&mut self.p[16 + i], 0x50, sharp);\n                }\n            }\n            let pos = match a[3] {\n                2 => QueuePos::New,\n                0 => QueuePos::First,\n                _ => QueuePos::Last,\n            };\n            let mut v = V {\n                a,\n                m,\n                c,\n                p,\n                log: &mut log,\n                stale: String::new(),\n            };\n            let q = GatherPoint {\n                pos: crate::Pos { x: a[0], y: a[1] },\n                action: a[2] as u8,\n            };\n            add_gather_point_steps(&mut v, q, pos);\n            // A row that called nothing records `-`.\n            if log.is_empty() {\n                log.push(\"-\".into());\n            }\n            0"]
