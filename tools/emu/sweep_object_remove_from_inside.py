#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_remove_from_inside.py — `Object::remove_from_inside@006480f0` under unicorn (item 3372, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_object_remove_from_inside.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 1024-byte object of random bytes (no bias), 0 stack argument(s);
`p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p3 sparse table 1 (2 rows × 1 slice(s), row-major), p4..p7 sparse table 2 (2 rows × 2 slice(s), row-major), p8..p32 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00c061f0 points at 2304 random bytes; vtable slot +0x8 (0 args) stubbed; vtable slot +0xd4 (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0xe8 (0 args) stubbed; callee 0x00651a80 (1 args) stubbed; callee 0x0064d8c0 (0 args) stubbed; callee 0x0046d310 (0 args) stubbed; callee 0x006e1370 (1 args) stubbed; the word at `c0+0x14` points at 12 random bytes less `0` (placed); the word at `c0+0x30` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 12 random bytes less `0` (placed); the word at `p1+0` points at 12 random bytes less `0` (placed); the word at `p2+0` points at 12 random bytes less `0` (placed); the word at `p3+0` points at 12 random bytes less `0` (placed); the word at `p8+0` points at 72 random bytes less `0` (placed); the word at `p8+4` points at 72 random bytes less `0` (placed); the word at `p8+8` points at 72 random bytes less `0` (placed); the word at `p9+0` points at 72 random bytes less `0` (placed); the word at `p9+4` points at 72 random bytes less `0` (placed); the word at `p9+8` points at 72 random bytes less `0` (placed); the word at `p10+0` points at 16 random bytes less `0` (placed); the word at `p10+4` points at 16 random bytes less `0` (placed); the word at `p10+8` points at 16 random bytes less `0` (placed); the word at `p11+0` points at 16 random bytes less `0` (placed); the word at `p11+4` points at 16 random bytes less `0` (placed); the word at `p11+8` points at 16 random bytes less `0` (placed); the word at `p12+0` points at 192 random bytes less `0` (placed); the word at `p12+4` points at 192 random bytes less `0` (placed); the word at `p12+8` points at 192 random bytes less `0` (placed); the word at `p13+0` points at 192 random bytes less `0` (placed); the word at `p13+4` points at 192 random bytes less `0` (placed); the word at `p13+8` points at 192 random bytes less `0` (placed); the word at `m+0x18` points at 72 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answered from [0, 1, 1] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [0, 1, 1] (90%); stub 4 answers -1..2 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 4 writes a value in 0..1 through its argument 0, printed after the answers (`o` in a place); 2 rows 0x1c apart at `@0xc0ab84`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 2 rows 0x1c apart at `@0xc0aec0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0xa28:0x10 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`.
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

ENTRY = 0x006480f0
WIN = 1024
OWN = 0
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607884, 128), (12607984, 2304)]
AT = []
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('slot', 8, 0), ('slot', 212, 0), ('slot', 32, 0), ('slot', 232, 0), ('call', 6625920, 1), ('call', 6609088, 0), ('call', 4641552, 0), ('call', 7213936, 1)]
PLACES = [('c0', '0x14', '0', 12), ('c0', '0x30', '0', 12), ('p0', '0', '0', 12), ('p1', '0', '0', 12), ('p2', '0', '0', 12), ('p3', '0', '0', 12), ('p8', '0', '0', 72), ('p8', '4', '0', 72), ('p8', '8', '0', 72), ('p9', '0', '0', 72), ('p9', '4', '0', 72), ('p9', '8', '0', 72), ('p10', '0', '0', 16), ('p10', '4', '0', 16), ('p10', '8', '0', 16), ('p11', '0', '0', 16), ('p11', '4', '0', 16), ('p11', '8', '0', 16), ('p12', '0', '0', 192), ('p12', '4', '0', 192), ('p12', '8', '0', 192), ('p13', '0', '0', 192), ('p13', '4', '0', 192), ('p13', '8', '0', 192), ('m', '0x18', '0', 72)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 8, ('set', [0, 256, 65536, 65792, 131072, 131328])), ('m', 40, ('set', [65535, 0, 1, 2])), ('m', 60, ('set', [0, 65536])), ('p32', 64, ('set', [428, 427, 429, 428, 427])), ('c1', 2120, ('set', [0, 1, 5])), ('c1', 2184, ('set', [0, 1, 7])), ('p26', 128, ('set', [4294901760, 0, 65536, 131072, 65536])), ('p26', 180, ('set', [0, 1])), ('p26', 104, ('set', [0, 0, 1, 2])), ('p26', 140, ('set', [0, 2147483648, 2147549184, 2147483648])), ('p27', 128, ('set', [4294901760, 0, 65536, 131072, 65536])), ('p27', 180, ('set', [0, 1])), ('p27', 104, ('set', [0, 0, 1, 2])), ('p27', 140, ('set', [0, 2147483648, 2147549184, 2147483648])), ('p28', 128, ('set', [4294901760, 0, 65536, 131072, 65536])), ('p28', 180, ('set', [0, 1])), ('p28', 104, ('set', [0, 0, 1, 2])), ('p28', 140, ('set', [0, 2147483648, 2147549184, 2147483648])), ('p29', 128, ('set', [4294901760, 0, 65536, 131072, 65536])), ('p29', 180, ('set', [0, 1])), ('p29', 104, ('set', [0, 0, 1, 2])), ('p29', 140, ('set', [0, 2147483648, 2147549184, 2147483648])), ('p30', 128, ('set', [4294901760, 0, 65536, 131072, 65536])), ('p30', 180, ('set', [0, 1])), ('p30', 104, ('set', [0, 0, 1, 2])), ('p30', 140, ('set', [0, 2147483648, 2147549184, 2147483648])), ('p31', 128, ('set', [4294901760, 0, 65536, 131072, 65536])), ('p31', 180, ('set', [0, 1])), ('p31', 104, ('set', [0, 0, 1, 2])), ('p31', 140, ('set', [0, 2147483648, 2147549184, 2147483648]))]
FILLS = []
ARGRANGE = []
SLOTBASE = [(232, 4640432)]
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [1, 1, 1, 0]), (1, [0, 1]), (2, [0, 1, 1]), (3, [0, 1, 1]), (6, [0, 1]), (7, [0, 1, 1])]
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = []
ANSWERBY = []
ANSWERWIN = []
CDECL = False
TIB = False
ARGEQ = []
SEQ = []
ANSRANGE = [(4, -1, 2)]
CELLTHIS = []
VTALSO = [('p20', 0), ('p21', 0), ('p22', 0), ('p23', 0), ('p24', 0), ('p25', 0), ('p26', 0), ('p27', 0), ('p28', 0), ('p29', 0), ('p30', 0), ('p31', 0)]
SPARSE = [('@', '0xc0ab84', 28, 2, [(0, 4)]), ('@', '0xc0aec0', 28, 2, [(0, 4)]), ('@', '0xe3a390', 28396, 2, [(0, 4), (2600, 16)])]
GTAB = []
RECORD = [0, 1, 2, 3, 4, 5, 6, 7]
RECORDORDER = True
RECORDEDX = False
ROWSPLIT = {}
LINKS = []
LINKHI = []
OUTARG = [(4, 0, ('range', 0, 1))]
CDECLSTUBS = []
STUBWRITE = []
STUBCOPY = []
STACKCLEAN = 0
STOPSTUB = None
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


def win_write(inner, at, size, v, write, read=None, rel=False):
    """`--stub-write IDX:WIN+OFF:SIZE:VAL` (item 2346): a stub that writes VAL
    at a window's word each call — a callee that unlinks the head of the
    container its caller loops over (`Object::kill_contents`) — then answers
    as `inner` would."""
    def answer(ecx, args):
        # `ecx+OFF` (item 3358, `Group::action_gather_point`): `at` is the offset, the word the
        # stub's own `ecx` names -- a caller's local (`Group::add` on a stack group).
        to = (ecx + at) & 0xFFFFFFFF if rel else at
        if v in ("dec", "inc"):
            # `dec` (item 2749, parked 2734): one less each call, a count a
            # callee (`remove_current`, `Array::remove`) lowers; `inc` one more
            # (`Group::add` raising the group's count).
            w = (int.from_bytes(read(to, size), "little") + (1 if v == "inc" else -1)) & ((1 << 8 * size) - 1)
            write(to, w.to_bytes(size, "little"))
        else:
            write(to, (v & ((1 << 8 * size) - 1)).to_bytes(size, "little"))
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
        if vals[0] == "flav":
            vals = ("set", rng.choice(vals[1]))
        for o in (range(0, len(buf) - 3, 4) if off is None
                  else range(off[0], len(buf) - 3, off[1]) if isinstance(off, tuple) else (off,)):
            v = wdraw(rng, vals)
            buf[o:o + 4] = (v & 0xFFFFFFFF).to_bytes(4, "little")


def main(argv):
    from hooks import Harness
    rng = random.Random(3372)
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
    stubbed = {e[0]: h.r32(vt + e[0]) & 0xFFFFFFFF for e in SLOTBASE}
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
    stopped = [False]
    if STOPSTUB is not None:
        from callfn import SENTINEL
        from unicorn import UC_HOOK_CODE
        from unicorn.x86_const import UC_X86_REG_EIP
        stop_addr = next(a_ for a_, v_ in h.hooked.items() if v_[0] == names[STOPSTUB])

        def stop_hook(uc_, addr_, size_, _):
            if stopped[0]:
                uc_.reg_write(UC_X86_REG_EIP, SENTINEL)
        uc.hook_add(UC_HOOK_CODE, stop_hook, begin=stop_addr, end=stop_addr)
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
        for off, addr, *always in SLOTBASE:
            on = True if always else rng.random() < 0.5
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
            if win == "ecx":
                # The word at the stub's own `ecx + OFF` (item 3358).
                h.answers[names[idx]] = win_write(h.answers[names[idx]], off, size, v, uc.mem_write,
                                                          lambda a_, n_: bytes(uc.mem_read(a_, n_)), rel=True)
                continue
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
        # `--stop-stub IDX` (item 3348, `Unit::work`): the call ends where stub IDX is first entered
        # (the call is recorded, the answer drawn, `eip` set to the return sentinel by a hook that runs
        # after the harness's own): a function too large to sweep whole is swept to a chosen callee,
        # and the port stops at the same call. `emu_stop` was tried and runs on past the hook.
        stopped[0] = False
        if STOPSTUB is not None:
            inner_stop = h.answers[names[STOPSTUB]]

            def stop_answer(ecx, args, inner_stop=inner_stop):
                stopped[0] = True
                return inner_stop(ecx, args) if callable(inner_stop) else inner_stop
            h.answers[names[STOPSTUB]] = stop_answer
        r = h.call(ENTRY, tuple(args), ecx=this if ECXINT is None else ecx,
                   edx=edxa, xmm=xmm, ret=RET, max_insns=MAXINSNS)  # Harness.call signs eax; signed() again would double a negative
        esp = uc.reg_read(UC_X86_REG_ESP)
        # A `__cdecl` function leaves its arguments for the caller (item 2181).
        want_esp = STACK_BASE + STACK_SIZE - 0x100 - (4 * len(args) if CDECL else 0)
        assert stopped[0] or esp == want_esp, f"esp {esp:x}: wrong pop"
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

# sweep_gen: ["obj", "Object::remove_from_inside@006480f0", "--item", "3372", "--rows", "3000", "--port", "{\n                // Window layout (sweep_gen's flag order): `m` the unit; `c[0]` `GameAccess::objects` (`0xc0618c`),\n                // `c[1]` the constants. `p[0..4]` the rows of the objects table (`0xc0ab84`) and the units table\n                // (`0xc0aec0`), `p[4+2r]` / `p[5+2r]` leader `r`'s flags word and counters (`+0xa28..`),\n                // `p[8..14]` the arrays (objects of `c[0]`, hosts, units), `p[14+3l+i]` the objects' records,\n                // `p[20+3l+i]` the hosts, `p[26+3l+i]` the units, `p[32]` the type record. The answers `a`: `a[0..8]`\n                // the stubs' (slot 0x8, 0xd4, 0x20, 0xe8, get_inside, add_to_world, is_peasant, has_tribe_bonus),\n                // `a[8]` get_inside's list, `a[9]` the 0xe8 image flag.\n                use sim::remove_inside_view::{Garr, RemoveInsideView, remove_from_inside_steps};\n                struct V<'a> {\n                    m: &'a mut [u8],\n                    c: &'a mut [Vec<u8>],\n                    p: &'a mut [Vec<u8>],\n                    a: &'a [i32],\n                    log: &'a mut Vec<String>,\n                }\n                fn r32(w: &[u8], o: usize) -> i32 {\n                    i32::from_le_bytes(w[o..o + 4].try_into().expect(\"four bytes\"))\n                }\n                fn w32(w: &mut [u8], o: usize, v: i32) {\n                    w[o..o + 4].copy_from_slice(&v.to_le_bytes());\n                }\n                fn r16(w: &[u8], o: usize) -> i32 {\n                    i32::from(i16::from_le_bytes([w[o], w[o + 1]]))\n                }\n                fn w16(w: &mut [u8], o: usize, v: i32) {\n                    w[o..o + 2].copy_from_slice(&(v as i16).to_le_bytes());\n                }\n                fn slot(base: usize, list: i32, i: i32) -> usize {\n                    assert!((0..2).contains(&list) && (0..3).contains(&i), \"slot {list}/{i}\");\n                    base + 3 * list as usize + i as usize\n                }\n                impl V<'_> {\n                    fn rec(&mut self, s: String) {\n                        self.log.push(s);\n                    }\n                    fn own(&self) -> usize {\n                        slot(26, i32::from(self.m[9]), r16(self.m, 0xa))\n                    }\n                    fn leader(&self) -> usize {\n                        let r = usize::from(self.m[9]);\n                        assert!(r < 2);\n                        r\n                    }\n                }\n                impl RemoveInsideView for V<'_> {\n                    fn host_flag(&mut self) -> bool {\n                        self.rec(\"s4:m+0x0,sp-0x10\".into());\n                        let (idx, list) = (self.a[4], self.a[8]);\n                        if idx < 0 {\n                            return false;\n                        }\n                        let h = slot(20, list, idx);\n                        self.rec(format!(\"s2:p{h}+0x0\"));\n                        self.a[2] != 0\n                    }\n                    fn is_unit(&mut self) -> bool {\n                        self.rec(\"s0:m+0x0\".into());\n                        self.a[0] != 0\n                    }\n                    fn own_up(&mut self) -> (i32, i32) {\n                        let o = &self.p[self.own()];\n                        (r16(o, 0x82), i32::from(o[0xb4] as i8))\n                    }\n                    fn own_down(&mut self) -> (i32, i32) {\n                        (r16(self.m, 0x28), i32::from(self.m[0x3e] as i8))\n                    }\n                    fn set_object_down(&mut self, list: i32, idx: i32, down: (i32, i32)) {\n                        let o = &mut self.p[slot(14, list, idx)];\n                        w16(o, 0x28, down.0);\n                        o[0x3e] = down.1 as u8;\n                    }\n                    fn set_unit_up(&mut self, list: i32, idx: i32, up: (i32, i32)) {\n                        let o = &mut self.p[slot(26, list, idx)];\n                        w16(o, 0x82, up.0);\n                        o[0xb4] = up.1 as u8;\n                    }\n                    fn clear_own_links(&mut self) {\n                        let i = self.own();\n                        w16(&mut self.p[i], 0x82, -1);\n                        w16(&mut self.p[i], 0x28, -1);\n                    }\n                    fn add_to_world(&mut self) {\n                        self.rec(\"s5:m+0x0\".into());\n                    }\n                    fn masked(&mut self) -> bool {\n                        self.p[self.own()][0x68] & 1 != 0\n                    }\n                    fn is_captain(&mut self) -> bool {\n                        let i = self.own();\n                        if self.a[9] != 0 {\n                            return (u16::from_le_bytes([self.p[i][0x8e], self.p[i][0x8f]]) >> 15) != 0;\n                        }\n                        self.rec(format!(\"s3:p{i}+0x0\"));\n                        self.a[3] != 0\n                    }\n                    fn is_peasant(&mut self) -> bool {\n                        self.rec(\"s6:m+0x0\".into());\n                        self.a[6] != 0\n                    }\n                    fn type_word(&mut self) -> i32 {\n                        r32(&self.p[32], 0x40)\n                    }\n                    fn is_special(&mut self) -> bool {\n                        self.rec(\"s1:m+0x0\".into());\n                        self.a[1] != 0\n                    }\n                    fn has_tribe_bonus(&mut self, id: i32) -> bool {\n                        let l = 4 + 2 * self.leader();\n                        self.rec(format!(\"s7:p{l}+0x0,{id}\"));\n                        self.a[7] != 0\n                    }\n                    fn bump(&mut self, which: Garr, delta: i32) {\n                        let l = 5 + 2 * self.leader();\n                        let off = match which {\n                            Garr::Peasants => 0,\n                            Garr::Stable => 4,\n                            Garr::Scout => 8,\n                            Garr::Barracks => 12,\n                        };\n                        let v = r32(&self.p[l], off).wrapping_add(delta);\n                        w32(&mut self.p[l], off, v);\n                    }\n                    fn lakota_food(&mut self) -> i32 {\n                        r32(&self.c[1], 0x848)\n                    }\n                    fn barracks_gather(&mut self) -> i32 {\n                        r32(&self.c[1], 0x888)\n                    }\n                    fn set_leader_flag(&mut self) {\n                        let l = 4 + 2 * self.leader();\n                        let v = r32(&self.p[l], 0) | 0x200_0000;\n                        w32(&mut self.p[l], 0, v);\n                    }\n                }\n                let mut v = V {\n                    m,\n                    c,\n                    p,\n                    a,\n                    log: &mut log,\n                };\n                remove_from_inside_steps(&mut v);\n                0\n            }", "--name", "object_remove_from_inside", "--cell-win", "0xc0618c:0x80", "--cell-win", "0xc061f0:0x900", "--sparse", "@0xc0ab84:28:2:0,4", "--sparse", "@0xc0aec0:28:2:0,4", "--sparse", "@0xe3a390:0x6eec:2:0,4/0xa28,16", "--stub", "slot:0x8:0", "--stub", "slot:0xd4:0", "--stub", "slot:0x20:0", "--stub", "slot:0xe8:0", "--stub", "0x651a80:1", "--stub", "0x64d8c0:0", "--stub", "0x46d310:0", "--stub", "0x6e1370:1", "--place", "c0+0x14:0:12", "--place", "c0+0x30:0:12", "--place", "p0+0:0:12", "--place", "p1+0:0:12", "--place", "p2+0:0:12", "--place", "p3+0:0:12", "--place", "p8+0:0:72", "--place", "p8+4:0:72", "--place", "p8+8:0:72", "--place", "p9+0:0:72", "--place", "p9+4:0:72", "--place", "p9+8:0:72", "--place", "p10+0:0:16", "--place", "p10+4:0:16", "--place", "p10+8:0:16", "--place", "p11+0:0:16", "--place", "p11+4:0:16", "--place", "p11+8:0:16", "--place", "p12+0:0:192", "--place", "p12+4:0:192", "--place", "p12+8:0:192", "--place", "p13+0:0:192", "--place", "p13+4:0:192", "--place", "p13+8:0:192", "--place", "m+0x18:0:0x48", "--wbias", "m+8:0,0x100,0x10000,0x10100,0x20000,0x20100", "--wbias", "m+0x28:0xffff,0,1,2", "--wbias", "m+0x3c:0,0x10000", "--wbias", "p32+0x40:0x1ac,0x1ab,0x1ad,0x1ac,0x1ab", "--wbias", "c1+0x848:0,1,5", "--wbias", "c1+0x888:0,1,7", "--wbias", "p26+0x80:0xffff0000,0,0x10000,0x20000,0x10000", "--wbias", "p26+0xb4:0,1", "--wbias", "p26+0x68:0,0,1,2", "--wbias", "p26+0x8c:0,0x80000000,0x80010000,0x80000000", "--wbias", "p27+0x80:0xffff0000,0,0x10000,0x20000,0x10000", "--wbias", "p27+0xb4:0,1", "--wbias", "p27+0x68:0,0,1,2", "--wbias", "p27+0x8c:0,0x80000000,0x80010000,0x80000000", "--wbias", "p28+0x80:0xffff0000,0,0x10000,0x20000,0x10000", "--wbias", "p28+0xb4:0,1", "--wbias", "p28+0x68:0,0,1,2", "--wbias", "p28+0x8c:0,0x80000000,0x80010000,0x80000000", "--wbias", "p29+0x80:0xffff0000,0,0x10000,0x20000,0x10000", "--wbias", "p29+0xb4:0,1", "--wbias", "p29+0x68:0,0,1,2", "--wbias", "p29+0x8c:0,0x80000000,0x80010000,0x80000000", "--wbias", "p30+0x80:0xffff0000,0,0x10000,0x20000,0x10000", "--wbias", "p30+0xb4:0,1", "--wbias", "p30+0x68:0,0,1,2", "--wbias", "p30+0x8c:0,0x80000000,0x80010000,0x80000000", "--wbias", "p31+0x80:0xffff0000,0,0x10000,0x20000,0x10000", "--wbias", "p31+0xb4:0,1", "--wbias", "p31+0x68:0,0,1,2", "--wbias", "p31+0x8c:0,0x80000000,0x80010000,0x80000000", "--answers", "0:1,1,1,0", "--answers", "1:0,1", "--answers", "2:0,1,1", "--answers", "3:0,1,1", "--answer-range", "4:-1:2", "--outarg", "4:0:0..1", "--answers", "6:0,1", "--answers", "7:0,1,1", "--slotbase", "0xe8:0x46ceb0", "--vtat-also", "p20+0", "--vtat-also", "p21+0", "--vtat-also", "p22+0", "--vtat-also", "p23+0", "--vtat-also", "p24+0", "--vtat-also", "p25+0", "--vtat-also", "p26+0", "--vtat-also", "p27+0", "--vtat-also", "p28+0", "--vtat-also", "p29+0", "--vtat-also", "p30+0", "--vtat-also", "p31+0", "--record", "0", "--record", "1", "--record", "2", "--record", "3", "--record", "4", "--record", "5", "--record", "6", "--record", "7", "--record-order", "--ret", "none"]
