#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_objects_add_flock.py — `Objects::add_flock@0065c0e0` under unicorn (item 3320, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_objects_add_flock.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 8-byte object of random bytes (no bias), 4 stack argument(s);
`p` windows in order: p0..p5 sparse table 0 (3 rows × 2 slice(s), row-major), p6..p28 the place(s), in flag order, p29..p44 the stubs' answer window(s); the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c06184 points at 16 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c0618c points at 32 random bytes; 4 random bytes at 0x00c8da98 (each word in -2..4); 4 random bytes at 0x00cbe33c (each word in -1..7); 4 random bytes at 0x00cbe350 (each word in 0..12); 16 random bytes at 0x00cb7e90 (each byte in -4..4); 16 random bytes at 0x00cbb0e0 (each byte in -4..4); 320 random bytes at 0x00e90000; callee 0x00a39d70 (2 args) stubbed; callee 0x0065e0c0 (7 args) stubbed; callee 0x00605400 (3 args) stubbed; callee 0x00606590 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x58 (0 args) stubbed; the import cell 0x00ac54f0 holds a stub (1 args); callee 0x0055d14a (5 args) stubbed; callee 0x0060a870 (0 args) stubbed; the import cell 0x00ac5500 holds a stub (1 args); the word at `0xc8da90` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 16 random bytes less `0` (placed); the word at `p6+4` points at 16 random bytes less `0` (placed); the word at `p6+8` points at 16 random bytes less `0` (placed); the word at `p6+12` points at 16 random bytes less `0` (placed); the word at `0xc0afbc` points at 12 random bytes less `0` (placed); the word at `p11+0` points at 256 random bytes less `0` (placed); the word at `p11+4` points at 256 random bytes less `0` (placed); the word at `p11+8` points at 256 random bytes less `0` (placed); the word at `c2+0x134` points at 4032 random bytes less `0` (placed); the word at `p12+0xf4` points at 4 random bytes less `0` (placed); the word at `p13+0xf4` points at 4 random bytes less `0` (placed); the word at `p14+0xf4` points at 4 random bytes less `0` (placed); the word at `p16+0` points at 128 random bytes less `0` (placed); the word at `p17+0` points at 128 random bytes less `0` (placed); the word at `p18+0` points at 128 random bytes less `0` (placed); the word at `p12+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p13+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p14+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p22+4` points at 16 random bytes less `0` (placed); the word at `p23+4` points at 16 random bytes less `0` (placed); the word at `p24+4` points at 16 random bytes less `0` (placed); the word at `p12+0xf8` points at 8 random bytes less `0` (placed); stub 4 answers a window of its own to each call, in call order (4 bytes, ...); stub 5 answers a window of its own to each call, in call order (128 bytes, ...); the word at `p22+4` laid as the address of `p22` half the rows; the word at `p23+4` laid as the address of `p23` half the rows; the word at `p24+4` laid as the address of `p24` half the rows; the word at `p37+0` laid as the address of `p28` every row; the word at `p38+0` laid as the address of `p28` every row; the word at `p39+0` laid as the address of `p28` every row; the word at `p40+0` laid as the address of `p28` every row; the word at `p41+0` laid as the address of `p28` every row; the word at `p42+0` laid as the address of `p28` every row; the word at `p43+0` laid as the address of `p28` every row; the word at `p44+0` laid as the address of `p28` every row; stub 0 answers 40 values in call order, printed after the answers; stub 1 answers 3 values in call order, printed after the answers; stub 6 answers 9 values in call order, printed after the answers; stub 0 answers 0..65535 every row; stub 1 answers -1..2 every row; stub 6 answers 15269888..15270144 step 0x20 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 6 is `__cdecl`: the caller pops its arguments; stub 9 is `__cdecl`: the caller pops its arguments; 3 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x7b8:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`.
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

ENTRY = 0x0065c0e0
WIN = 8
OWN = 4
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607980, 1376), (12607876, 16), (12607880, 320), (12607884, 32)]
AT = [(13163160, 4), (13361980, 4), (13362000, 4), (13336208, 16), (13349088, 16), (15269888, 320)]
ATRANGE = {13163160: (-2, 4, 4), 13361980: (-1, 7, 4), 13362000: (0, 12, 4), 13336208: (-4, 4, 1), 13349088: (-4, 4, 1)}
PTRARGS = []
PNULLS = []
STUBS = [('call', 10722672, 2), ('call', 6676672, 7), ('call', 6312960, 3), ('call', 6317456, 0), ('call', 7539392, 1), ('slot', 88, 0), ('iat', 11293936, 1), ('call', 5624138, 5), ('call', 6334576, 0), ('iat', 11293952, 1)]
PLACES = [(None, '0xc8da90', '0', 16), ('p6', '0', '0', 16), ('p6', '4', '0', 16), ('p6', '8', '0', 16), ('p6', '12', '0', 16), (None, '0xc0afbc', '0', 12), ('p11', '0', '0', 256), ('p11', '4', '0', 256), ('p11', '8', '0', 256), ('c2', '0x134', '0', 4032), ('p12', '0xf4', '0', 4), ('p13', '0xf4', '0', 4), ('p14', '0xf4', '0', 4), ('p16', '0', '0', 128), ('p17', '0', '0', 128), ('p18', '0', '0', 128), ('p12', '0xdc', '0', 16), ('p13', '0xdc', '0', 16), ('p14', '0xdc', '0', 16), ('p22', '4', '0', 16), ('p23', '4', '0', 16), ('p24', '4', '0', 16), ('p12', '0xf8', '0', 8)]
NULLS = [16, 17, 18]
NULLFORCE = {}
WBIAS = [('c2', 0, ('range', 2, 12)), ('c2', 4, ('range', 2, 12)), ('c0', 1360, ('set', [0, 1, 224, 225, 226, 227, 300, 449, 450, 451, 452])), ('p0', 0, ('set', [0, 1, 1, 1, 2, 3, 256, 257])), ('p2', 0, ('set', [0, 1, 1, 1, 2, 3, 256, 257])), ('p4', 0, ('set', [0, 1, 1, 1, 2, 3, 256, 257])), ('p1', 0, ('set', [0, 1, 2, -225, -224, -1, 224, 225, 226, 227, -300, 225, 0, 1])), ('p3', 0, ('set', [0, 1, 2, -225, -224, -1, 224, 225, 226, 227, -300, 225, 0, 1])), ('p5', 0, ('set', [0, 1, 2, -225, -224, -1, 224, 225, 226, 227, -300, 225, 0, 1])), ('p28', 4, ('range', 0, 112))]
FILLS = [('p15', '(0xff if (i//7)%2 else 0x20) if (i%7==0 and ((i//7)*2654435761>>11)%7==0) else (0xdf if i%7==0 else 0)')]
ARGRANGE = [(0, -12, 44), (1, -12, 44), (2, -1, 2), (3, -1, 2)]
SLOTBASE = []
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = []
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = [('p22', 4, 'p22', 0, False), ('p23', 4, 'p23', 0, False), ('p24', 4, 'p24', 0, False), ('p37', 0, 'p28', 0, True), ('p38', 0, 'p28', 0, True), ('p39', 0, 'p28', 0, True), ('p40', 0, 'p28', 0, True), ('p41', 0, 'p28', 0, True), ('p42', 0, 'p28', 0, True), ('p43', 0, 'p28', 0, True), ('p44', 0, 'p28', 0, True)]
ANSWERBY = []
ANSWERWIN = [(4, 4), (4, 4), (4, 4), (4, 4), (4, 4), (4, 4), (4, 4), (4, 4), (5, 128), (5, 128), (5, 128), (5, 128), (5, 128), (5, 128), (5, 128), (5, 128)]
CDECL = False
TIB = True
ARGEQ = []
SEQ = [(0, 40, None), (1, 3, None), (6, 9, None)]
ANSRANGE = [(0, 0, 65535), (1, -1, 2), (6, 15269888, 15270144, 32, 1)]
CELLTHIS = []
VTALSO = [('p29', 0), ('p30', 0), ('p31', 0), ('p32', 0), ('p33', 0), ('p34', 0), ('p35', 0), ('p36', 0)]
SPARSE = [('@', '0xe3a390', 28396, 3, [(0, 4), (1976, 4)])]
GTAB = []
RECORD = [0, 1, 2, 3, 4, 5, 6, 7, 8]
RECORDORDER = True
RECORDEDX = False
ROWSPLIT = {}
LINKS = []
LINKHI = []
OUTARG = []
CDECLSTUBS = [6, 9]
STUBWRITE = []
STUBCOPY = []
STACKCLEAN = 0
INITCALLS = []
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
    rng = random.Random(3320)
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
    for i in range(700):
        h.brk = base  # the bump heap is reused a row; 700 rows of windows outgrow it
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

# sweep_gen: ["--item", "3320", "obj", "Objects::add_flock@0065c0e0", "--win", "8", "--own", "4", "--ret", "none", "--tib", "--rows", "700", "--cell-win", "0xc061ec:0x560", "--cell-win", "0xc06184:0x10", "--cell-win", "0xc06188:0x140", "--cell-win", "0xc0618c:0x20", "--at", "0xc8da98:4:-2..4", "--at", "0xcbe33c:4:-1..7", "--at", "0xcbe350:4:0..12", "--at", "0xcb7e90:16:-4..4/b", "--at", "0xcbb0e0:16:-4..4/b", "--at", "0xe90000:0x140", "--sparse", "@0xe3a390:0x6eec:3:0,4/0x7b8,4", "--ptrtab", "0xc8da90:4:0x10", "--ptrtab", "0xc0afbc:3:0x100", "--place", "c2+0x134:0:4032", "--place", "p12+0xf4:0:4", "--place", "p13+0xf4:0:4", "--place", "p14+0xf4:0:4", "--place", "p16+0:0:0x80", "--place", "p17+0:0:0x80", "--place", "p18+0:0:0x80", "--place", "p12+0xdc:0:0x10", "--place", "p13+0xdc:0:0x10", "--place", "p14+0xdc:0:0x10", "--place", "p22+4:0:0x10", "--place", "p23+4:0:0x10", "--place", "p24+4:0:0x10", "--place", "p12+0xf8:0:8", "--place-null", "16", "--place-null", "17", "--place-null", "18", "--alias", "p22+4:p22", "--alias", "p23+4:p23", "--alias", "p24+4:p24", "--stub", "0xa39d70:2", "--stub", "0x65e0c0:7", "--stub", "0x605400:3", "--stub", "0x606590:0", "--stub", "0x730ac0:1", "--stub", "slot:0x58:0", "--stub", "iat:0xac54f0:1", "--stub", "0x55d14a:5", "--stub", "0x60a870:0", "--stub", "iat:0xac5500:1", "--stub-cdecl", "6", "--stub-cdecl", "9", "--seq", "0:40", "--answer-range", "0:0:65535", "--seq", "1:3", "--answer-range", "1:-1:2", "--seq", "6:9", "--answer-range", "6:0xe90000:0xe90100:0x20:distinct", "--argrange", "0:-12:44", "--argrange", "1:-12:44", "--argrange", "2:-1:2", "--argrange", "3:-1:2", "--wbias", "c2+0:2..12", "--wbias", "c2+4:2..12", "--wbias", "c0+0x550:0,1,224,225,226,227,300,449,450,451,452", "--wbias", "p0+0:0,1,1,1,2,3,0x100,0x101", "--wbias", "p2+0:0,1,1,1,2,3,0x100,0x101", "--wbias", "p4+0:0,1,1,1,2,3,0x100,0x101", "--wbias", "p1+0:0,1,2,-225,-224,-1,224,225,226,227,-300,225,0,1", "--wbias", "p3+0:0,1,2,-225,-224,-1,224,225,226,227,-300,225,0,1", "--wbias", "p5+0:0,1,2,-225,-224,-1,224,225,226,227,-300,225,0,1", "--wbias", "p28+4:0..0x70", "--fill", "p15:(0xff if (i//7)%2 else 0x20) if (i%7==0 and ((i//7)*2654435761>>11)%7==0) else (0xdf if i%7==0 else 0)", "--answer-win", "4:4", "--answer-win", "4:4", "--answer-win", "4:4", "--answer-win", "4:4", "--answer-win", "4:4", "--answer-win", "4:4", "--answer-win", "4:4", "--answer-win", "4:4", "--answer-win", "5:0x80", "--answer-win", "5:0x80", "--answer-win", "5:0x80", "--answer-win", "5:0x80", "--answer-win", "5:0x80", "--answer-win", "5:0x80", "--answer-win", "5:0x80", "--answer-win", "5:0x80", "--vtat-also", "p29+0", "--alias", "p37+0:p28!", "--vtat-also", "p30+0", "--alias", "p38+0:p28!", "--vtat-also", "p31+0", "--alias", "p39+0:p28!", "--vtat-also", "p32+0", "--alias", "p40+0:p28!", "--vtat-also", "p33+0", "--alias", "p41+0:p28!", "--vtat-also", "p34+0", "--alias", "p42+0:p28!", "--vtat-also", "p35+0", "--alias", "p43+0:p28!", "--vtat-also", "p36+0", "--alias", "p44+0:p28!", "--record", "0", "--record", "1", "--record", "2", "--record", "3", "--record", "4", "--record", "5", "--record", "6", "--record", "7", "--record", "8", "--record-order", "--port", "            // The row: `a[0..4]` the tile `x`, `y`, `who`, `n`; `a[4 + i]` stub `i`'s answer, the stubs being\n            // 0 `Random::get`, 1 `init_unit`, 2 `Unit::set_angle`, 3 `update_z`, 4 `get_obj`, 5 the pool\n            // object's slot `+0x58`, 6 `malloc` (list, then nodes), 7 `_eh_vector_constructor_iterator_`,\n            // 8 `update_action`, 9 `free`; then the `--seq` answers: `a[14..54]` the forty rolls, `a[54..57]`\n            // `init_unit`'s answers, `a[57..66]` `malloc`'s (all `0xe90000 + 0x20 k`, the first the `WCoordList`'s\n            // storage).\n            //\n            // Windows: `c[0]` the game (`+0x550` the frame), `c[1]` the random, `c[2]` the world (`+0` width,\n            // `+4` height, `+0x134` the cells), `c[3]` the objects, `c[4]` the recycler's length, `c[5]` and\n            // `c[6]` `circle_radius[3]`, `[8]`, `c[7]`, `c[8]` `circle_x`, `circle_y`, `c[9]` the heap malloc\n            // answers from. `p[0..6]` the leaders' flag and stamp words, `p[6]` the recycler's list (`p[7..11]`\n            // its nodes), `p[11]` the units array (`p[12..15]` the units), `p[15]` the cells, `p[16..19]` the\n            // units' `+0xf4` records (`p[19..22]` the figures they name), `p[22..25]` the units' ring heads\n            // (`p[25..28]` the nodes before them), `p[28]` the order base table, `p[29..37]` the pool objects\n            // `get_obj` answers, `p[37..45]` the orders its slot answers.\n            //\n            // **The port is the crate's own steps**, `crate::flock::add_flock_steps`; the view is the emulated\n            // original's windows. The ring is the original's circular `LinkListBase` laid in the windows (and in\n            // `c[9]`, byte-addressed, since a node, a list and a node may overlap there), the order's fields what\n            // `Objects::add_flock` stores in line.\n            use crate::flock::{FlockView, add_flock_steps};\n            use crate::world::Pos;\n            #[derive(Clone, Copy)]\n            enum Win {\n                C(usize),\n                P(usize),\n            }\n            struct V<'a> {\n                a: &'a [i32],\n                c: &'a mut [Vec<u8>],\n                p: &'a mut [Vec<u8>],\n                log: &'a mut Vec<String>,\n                /// Where an image address lies: `(base, size, window)`.\n                ranges: Vec<(u32, u32, Win)>,\n                rolls: usize,\n                inits: usize,\n                mallocs: usize,\n                orders: usize,\n                list: u32,\n                pushed: usize,\n            }\n            impl V<'_> {\n                fn at(&self, addr: u32) -> (Win, usize) {\n                    for &(base, size, w) in &self.ranges {\n                        if addr >= base && addr - base < size {\n                            return (w, (addr - base) as usize);\n                        }\n                    }\n                    panic!(\"address {addr:#x} lies in no window\");\n                }\n                fn win(&mut self, w: Win) -> &mut Vec<u8> {\n                    match w {\n                        Win::C(i) => &mut self.c[i],\n                        Win::P(i) => &mut self.p[i],\n                    }\n                }\n                fn r8(&mut self, addr: u32) -> u8 {\n                    let (w, o) = self.at(addr);\n                    self.win(w)[o]\n                }\n                fn w8(&mut self, addr: u32, v: u8) {\n                    let (w, o) = self.at(addr);\n                    self.win(w)[o] = v;\n                }\n                fn r32(&mut self, addr: u32) -> u32 {\n                    u32::from_le_bytes([\n                        self.r8(addr),\n                        self.r8(addr + 1),\n                        self.r8(addr + 2),\n                        self.r8(addr + 3),\n                    ])\n                }\n                fn w32(&mut self, addr: u32, v: u32) {\n                    for (i, b) in v.to_le_bytes().into_iter().enumerate() {\n                        self.w8(addr + i as u32, b);\n                    }\n                }\n                fn seq(&self, base: usize, n: usize, k: usize) -> i32 {\n                    self.a[base + k.min(n - 1)]\n                }\n            }\n            impl FlockView for V<'_> {\n                fn leader_live(&self, who: i32) -> bool {\n                    self.p[2 * who as usize][0] & 1 != 0\n                }\n                fn stamp(&self, who: i32) -> i64 {\n                    i64::from(rd(&self.p[2 * who as usize + 1], 0))\n                }\n                fn set_stamp(&mut self, who: i32, frame: i64) {\n                    wr(&mut self.p[2 * who as usize + 1], 0, frame as i32);\n                }\n                fn frame(&self) -> i64 {\n                    i64::from(rd(&self.c[0], 0x550))\n                }\n                fn roll(&mut self, _site: &'static str) -> i32 {\n                    let v = self.seq(14, 40, self.rolls);\n                    self.rolls += 1;\n                    self.log.push(\"s0:c1+0x0,0,65535\".into());\n                    v\n                }\n                fn world_size(&self) -> (i32, i32) {\n                    (rd(&self.c[2], 0), rd(&self.c[2], 4))\n                }\n                fn forest(&self, x: i32, y: i32) -> bool {\n                    let w = rd(&self.c[2], 0);\n                    self.p[15][((w * y + x) * 0x1c) as usize] & 0x20 != 0\n                }\n                fn push_cell(&mut self, x: i32, y: i32) {\n                    if self.pushed == 0 {\n                        // `increase_size`'s `malloc(4 \u00d7 8)`, `ecx` the size.\n                        self.log.push(\"s6:32,32\".into());\n                        self.list = self.seq(57, 9, self.mallocs) as u32;\n                        self.mallocs += 1;\n                    }\n                    let at = self.list + 8 * self.pushed as u32;\n                    self.w32(at, x as u32);\n                    self.w32(at + 4, y as u32);\n                    self.pushed += 1;\n                }\n                fn cell(&self, k: usize) -> (i32, i32) {\n                    let at = self.list + 8 * k as u32;\n                    let (w, o) = self.at(at);\n                    let win = match w {\n                        Win::C(i) => &self.c[i],\n                        Win::P(i) => &self.p[i],\n                    };\n                    (rd(win, o), rd(win, o + 4))\n                }\n                fn ring_radius(&self, k: usize) -> i32 {\n                    match k {\n                        3 => rd(&self.c[5], 0),\n                        8 => rd(&self.c[6], 0),\n                        _ => panic!(\"circle_radius[{k}] is not a window\"),\n                    }\n                }\n                fn ring_at(&self, i: usize) -> (i32, i32) {\n                    (\n                        i32::from(self.c[7][i] as i8),\n                        i32::from(self.c[8][i] as i8),\n                    )\n                }\n                fn make_bird(&mut self, at: Pos) -> Option<usize> {\n                    self.log\n                        .push(format!(\"s1:c3+0x0,9,403,{},{},-1,-1,-1\", at.x, at.y));\n                    let idx = self.seq(54, 3, self.inits);\n                    self.inits += 1;\n                    if idx < 0 {\n                        return None;\n                    }\n                    Some(idx as usize)\n                }\n                fn turn(&mut self, b: usize, angle: i32) {\n                    self.log\n                        .push(format!(\"s2:p{}+0x0,{angle},c3+0x0,1\", 12 + b));\n                }\n                fn update_z(&mut self, b: usize) {\n                    self.log.push(format!(\"s3:p{}+0x0\", 12 + b));\n                }\n                fn lift(&mut self, b: usize) {\n                    let z = rd(&self.p[19 + b], 0x14);\n                    wr(&mut self.p[19 + b], 0x70, z);\n                    wr(&mut self.p[19 + b], 0x14, z.wrapping_add(200));\n                }\n                fn fly_to(&mut self, b: usize, at: Pos) {\n                    let j = self.orders.min(7);\n                    self.orders += 1;\n                    // `get_obj`'s `ecx` is the figure's index \u00d7 4 (a stale register).\n                    self.log.push(format!(\"s4:{},24\", 4 * b));\n                    self.log.push(format!(\"s5:p{}+0x0\", 29 + j));\n                    let order_at = (self.a[9] as u32)\n                        .wrapping_sub(0xa0 * (7 - j) as u32);\n                    let off = rd(&self.p[28], 4);\n                    let o = &mut self.p[37 + j];\n                    wr(o, 4, at.x);\n                    wr(o, 8, at.y);\n                    wr(o, 0x10, 0);\n                    wr(o, 0xc, 0);\n                    wr(o, 0x2c, 0);\n                    wr(o, 0x18, -1);\n                    wr(o, 0x1c, -1);\n                    wr(o, 0x20, 0x640);\n                    o[(off + 4) as usize] &= 0xfb;\n                    let data = order_at.wrapping_add(off as u32);\n                    // A node from the recycler, or a fresh one.\n                    let len = rd(&self.c[4], 0);\n                    let node = if len == 0 {\n                        self.log.push(format!(\"s6:p{}+0x0,20\", 37 + j));\n                        let m = self.seq(57, 9, self.mallocs) as u32;\n                        self.mallocs += 1;\n                        self.w32(m, 1);\n                        self.log.push(format!(\n                            \"s7:p{}+0x0,c9+{:#x},16,1,4648320,4449680\",\n                            37 + j,\n                            m + 4 - 0xe90000\n                        ));\n                        m + 4\n                    } else {\n                        let l = len.max(1) - 1;\n                        wr(&mut self.c[4], 0, l);\n                        rd(&self.p[6], 4 * l as usize) as u32\n                    };\n                    self.w8(node + 0xc, 0);\n                    self.w32(node + 8, data);\n                    self.w32(node + 4, 0);\n                    self.w32(node, 0);\n                    let head = rd(&self.p[12 + b], 0xdc) as u32;\n                    if head != 0 {\n                        self.w32(node, head);\n                        let tail = self.r32(head + 4);\n                        self.w32(node + 4, tail);\n                        let tail = self.r32(head + 4);\n                        self.w32(tail, node);\n                        let h = rd(&self.p[12 + b], 0xdc) as u32;\n                        self.w32(h + 4, node);\n                        wr(&mut self.p[12 + b], 0xdc, node as i32);\n                    } else {\n                        wr(&mut self.p[12 + b], 0xdc, node as i32);\n                        self.w32(node, node);\n                        let h = rd(&self.p[12 + b], 0xdc) as u32;\n                        self.w32(h + 4, h);\n                    }\n                    let metric = self.r8(node + 0xc);\n                    let u = &mut self.p[12 + b];\n                    let n = rd(u, 0xd8).wrapping_add(1);\n                    wr(u, 0xd8, n);\n                    wr(u, 0xd4, node as i32);\n                    wr(u, 0xcc, data as i32);\n                    u[0xd0] = metric;\n                    self.log.push(format!(\"s8:p{}+0x0\", 12 + b));\n                }\n            }\n            let mut ranges = vec![(0xe90000u32, 0x140u32, Win::C(9))];\n            for k in 0..4 {\n                ranges.push((rd(&p[6], 4 * k) as u32, 0x10, Win::P(7 + k)));\n            }\n            for k in 0..3 {\n                let h = rd(&p[12 + k], 0xdc) as u32;\n                if h != 0 {\n                    ranges.push((h, 0x10, Win::P(22 + k)));\n                    let t = rd(&p[22 + k], 4) as u32;\n                    if t != h && t != 0 {\n                        ranges.push((t, 0x10, Win::P(25 + k)));\n                    }\n                }\n            }\n            let mut v = V {\n                a,\n                c,\n                p,\n                log: &mut log,\n                ranges,\n                rolls: 0,\n                inits: 0,\n                mallocs: 0,\n                orders: 0,\n                list: 0,\n                pushed: 0,\n            };\n            add_flock_steps(&mut v, a[0], a[1], a[2], a[3]);\n            // A row that called nothing records `-`.\n            if log.is_empty() {\n                log.push(\"-\".into());\n            }\n            0"]
