#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_same_damage.py — `Unit::same_damage@005f9400` under unicorn (item 3234, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_unit_same_damage.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 256-byte object of random bytes (no bias), 2 stack argument(s);
`p` windows in order: p0..p28 the place(s), in flag order; the global cell 0x00c0618c points at 64 random bytes; callee 0x00609c00 (2 args) stubbed; callee 0x00647080 (3 args) stubbed; vtable slot +0xa8 (0 args) stubbed; vtable slot +0x11c (1 args) stubbed; vtable slot +0x158 (0 args) stubbed; the word at `c0+0x14` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 48 random bytes less `0` (placed); the word at `p0+4` points at 48 random bytes less `0` (placed); the word at `p0+8` points at 48 random bytes less `0` (placed); the word at `p0+12` points at 48 random bytes less `0` (placed); the word at `p0+16` points at 48 random bytes less `0` (placed); the word at `p0+20` points at 48 random bytes less `0` (placed); the word at `m+0xe0` points at 24 random bytes less `0` (placed); the word at `p7+0` points at 160 random bytes less `0` (placed); the word at `p7+4` points at 160 random bytes less `0` (placed); the word at `p7+8` points at 160 random bytes less `0` (placed); the word at `p7+12` points at 160 random bytes less `0` (placed); the word at `p7+16` points at 160 random bytes less `0` (placed); the word at `p7+20` points at 160 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `0xc0aedc` points at 16 random bytes less `0` (placed); the word at `p14+0` points at 152 random bytes less `0` (placed); the word at `p14+4` points at 152 random bytes less `0` (placed); the word at `p14+8` points at 152 random bytes less `0` (placed); the word at `p15+0` points at 152 random bytes less `0` (placed); the word at `p15+4` points at 152 random bytes less `0` (placed); the word at `p15+8` points at 152 random bytes less `0` (placed); the word at `m+0x18` points at 784 random bytes less `0` (placed); the word at `p8+0x18` points at 784 random bytes less `0` (placed); the word at `p9+0x18` points at 784 random bytes less `0` (placed); the word at `p10+0x18` points at 784 random bytes less `0` (placed); the word at `p11+0x18` points at 784 random bytes less `0` (placed); the word at `p12+0x18` points at 784 random bytes less `0` (placed); the word at `p13+0x18` points at 784 random bytes less `0` (placed); slot +0x158 laid as 0x00647080 or its stub, a flag after the answers; slot +0x11c laid as 0x00610890 or its stub, a flag after the answers; the word at `c0+0x30` laid as the address of `p0+12` every row; stub 2 answers `p8+j`, the data of the object its `ecx` is in `p1+j` (j < 6), linked at `+0x90`; stub 0 answers 4 values in call order, printed after the answers; stub 3 answers 5 values in call order, printed after the answers; stub 0 answers -20..60 every row; stub 3 answers 1..40 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); figures p16..p18 link at +0x8e (2 bytes) above their own index or to -1; figures p19..p21 link at +0x8e (2 bytes) above their own index or to -1; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`.
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

ENTRY = 0x005f9400
WIN = 256
OWN = 2
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607884, 64)]
AT = []
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('call', 6331392, 2), ('call', 6582400, 3), ('slot', 168, 0), ('slot', 284, 1), ('slot', 344, 0)]
PLACES = [('c0', '0x14', '0', 24), ('p0', '0', '0', 48), ('p0', '4', '0', 48), ('p0', '8', '0', 48), ('p0', '12', '0', 48), ('p0', '16', '0', 48), ('p0', '20', '0', 48), ('m', '0xe0', '0', 24), ('p7', '0', '0', 160), ('p7', '4', '0', 160), ('p7', '8', '0', 160), ('p7', '12', '0', 160), ('p7', '16', '0', 160), ('p7', '20', '0', 160), (None, '0xc0aec0', '0', 16), (None, '0xc0aedc', '0', 16), ('p14', '0', '0', 152), ('p14', '4', '0', 152), ('p14', '8', '0', 152), ('p15', '0', '0', 152), ('p15', '4', '0', 152), ('p15', '8', '0', 152), ('m', '0x18', '0', 784), ('p8', '0x18', '0', 784), ('p9', '0x18', '0', 784), ('p10', '0x18', '0', 784), ('p11', '0x18', '0', 784), ('p12', '0x18', '0', 784), ('p13', '0x18', '0', 784)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 144, ('set', [65535, 0, 1, 2])), ('m', 8, ('set', [0, 1, 256, 257, 65536, 65537, 65792, 65793, 131072, 131073, 131328, 131329, 4294901760, 4294901761, 4294902016, 4294902017])), ('p8', 8, ('set', [0, 1, 65536, 65537, 131072, 131073, 4294901760, 4294901761])), ('p9', 8, ('set', [0, 1, 65536, 65537, 131072, 131073, 4294901760, 4294901761])), ('p10', 8, ('set', [0, 1, 65536, 65537, 131072, 131073, 4294901760, 4294901761])), ('p11', 8, ('set', [0, 1, 65536, 65537, 131072, 131073, 4294901760, 4294901761])), ('p12', 8, ('set', [0, 1, 65536, 65537, 131072, 131073, 4294901760, 4294901761])), ('p13', 8, ('set', [0, 1, 65536, 65537, 131072, 131073, 4294901760, 4294901761])), ('p22', 776, ('set', [1, 2, 3, 4])), ('p23', 776, ('set', [1, 2, 3, 4])), ('p24', 776, ('set', [1, 2, 3, 4])), ('p25', 776, ('set', [1, 2, 3, 4])), ('p26', 776, ('set', [1, 2, 3, 4])), ('p27', 776, ('set', [1, 2, 3, 4])), ('p28', 776, ('set', [1, 2, 3, 4])), ('m', 32, ('range', 1, 40)), ('p1', 32, ('range', 1, 40)), ('p2', 32, ('range', 1, 40)), ('p3', 32, ('range', 1, 40)), ('p4', 32, ('range', 1, 40)), ('p5', 32, ('range', 1, 40)), ('p6', 32, ('range', 1, 40)), ('p8', 32, ('range', 1, 40)), ('p9', 32, ('range', 1, 40)), ('p10', 32, ('range', 1, 40)), ('p11', 32, ('range', 1, 40)), ('p12', 32, ('range', 1, 40)), ('p13', 32, ('range', 1, 40)), ('p16', 8, ('set', [0, 1, 1, 1])), ('p17', 8, ('set', [0, 1, 1, 1])), ('p18', 8, ('set', [0, 1, 1, 1])), ('p19', 8, ('set', [0, 1, 1, 1])), ('p20', 8, ('set', [0, 1, 1, 1])), ('p21', 8, ('set', [0, 1, 1, 1]))]
FILLS = []
ARGRANGE = [(0, 0, 2), (1, 0, 1)]
SLOTBASE = [(344, 6582400), (284, 6359184)]
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
ALIAS = [('c0', 48, 'p0', 12, True)]
ANSWERBY = [(2, 1, 8, 6, 144)]
ANSWERWIN = []
CDECL = False
TIB = False
ARGEQ = []
SEQ = [(0, 4, None), (3, 5, None)]
ANSRANGE = [(0, -20, 60), (3, 1, 40)]
CELLTHIS = []
VTALSO = [('p16', 0), ('p17', 0), ('p18', 0), ('p19', 0), ('p20', 0), ('p21', 0), ('p1', 0), ('p2', 0), ('p3', 0), ('p4', 0), ('p5', 0), ('p6', 0), ('p8', 0), ('p9', 0), ('p10', 0), ('p11', 0), ('p12', 0), ('p13', 0)]
SPARSE = []
GTAB = []
RECORD = [0, 1, 3, 4]
RECORDORDER = True
RECORDEDX = False
ROWSPLIT = {}
LINKS = [(16, 18, 142, 2, '>', [-1]), (19, 21, 142, 2, '>', [-1])]
LINKHI = []
OUTARG = []
CDECLSTUBS = []
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
    rng = random.Random(3234)
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
    for i in range(2000):
        h.brk = base  # the bump heap is reused a row; 2000 rows of windows outgrow it
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

# sweep_gen: ["obj", "Unit::same_damage@005f9400", "--item=3234", "--win=0x100", "--own=2", "--ret=none", "--rows=2000", "--cell-win=0xc0618c:0x40", "--ptrtab=c0+0x14:6:0x30:vt", "--alias=c0+0x30:p0+12!", "--ptrtab=m+0xe0:6:0xa0:vt", "--place=0xc0aec0:0:0x10", "--place=0xc0aedc:0:0x10", "--place=p14+0:0:0x98", "--place=p14+4:0:0x98", "--place=p14+8:0:0x98", "--place=p15+0:0:0x98", "--place=p15+4:0:0x98", "--place=p15+8:0:0x98", "--place=m+0x18:0:0x310", "--place=p8+0x18:0:0x310", "--place=p9+0x18:0:0x310", "--place=p10+0x18:0:0x310", "--place=p11+0x18:0:0x310", "--place=p12+0x18:0:0x310", "--place=p13+0x18:0:0x310", "--stub=0x609c00:2", "--stub=0x647080:3", "--stub=slot:0xa8:0", "--stub=slot:0x11c:1", "--stub=slot:0x158:0", "--slotbase=0x158:0x647080", "--vtat-keep=0x11c:0x610890", "--answer-by=2:1:8:6:0x90", "--seq=0:4", "--answer-range=0:-20:60", "--seq=3:5", "--answer-range=3:1:40", "--argrange=0:0:2", "--argrange=1:0:1", "--vtat-also=p16+0", "--vtat-also=p17+0", "--vtat-also=p18+0", "--vtat-also=p19+0", "--vtat-also=p20+0", "--vtat-also=p21+0", "--link=p16..18+0x8e:2:>:-1", "--link=p19..21+0x8e:2:>:-1", "--wbias=m+0x90:65535,0,1,2", "--wbias=m+8:0,1,256,257,65536,65537,65792,65793,131072,131073,131328,131329,4294901760,4294901761,4294902016,4294902017", "--wbias=p8+8:0,1,65536,65537,131072,131073,4294901760,4294901761", "--wbias=p9+8:0,1,65536,65537,131072,131073,4294901760,4294901761", "--wbias=p10+8:0,1,65536,65537,131072,131073,4294901760,4294901761", "--wbias=p11+8:0,1,65536,65537,131072,131073,4294901760,4294901761", "--wbias=p12+8:0,1,65536,65537,131072,131073,4294901760,4294901761", "--wbias=p13+8:0,1,65536,65537,131072,131073,4294901760,4294901761", "--wbias=p22+0x308:1,2,3,4", "--wbias=p23+0x308:1,2,3,4", "--wbias=p24+0x308:1,2,3,4", "--wbias=p25+0x308:1,2,3,4", "--wbias=p26+0x308:1,2,3,4", "--wbias=p27+0x308:1,2,3,4", "--wbias=p28+0x308:1,2,3,4", "--wbias=m+0x20:1..40", "--wbias=p1+0x20:1..40", "--wbias=p2+0x20:1..40", "--wbias=p3+0x20:1..40", "--wbias=p4+0x20:1..40", "--wbias=p5+0x20:1..40", "--wbias=p6+0x20:1..40", "--wbias=p8+0x20:1..40", "--wbias=p9+0x20:1..40", "--wbias=p10+0x20:1..40", "--wbias=p11+0x20:1..40", "--wbias=p12+0x20:1..40", "--wbias=p13+0x20:1..40", "--wbias=p16+8:0,1,1,1", "--wbias=p17+8:0,1,1,1", "--wbias=p18+8:0,1,1,1", "--wbias=p19+8:0,1,1,1", "--wbias=p20+8:0,1,1,1", "--wbias=p21+8:0,1,1,1", "--record=0", "--record=1", "--record=3", "--record=4", "--record-order", "--port", "{\n                // Args `a[0]` the unit's index, `a[1]` its player (`param_1`, `param_2`); stub answers `a[2..7]` (unused:\n                // 0 `total_damage` and 3 slot `+0x11c` are `--seq`, 2 slot `+0xa8` answers by its `ecx`); `a[7..11]` the\n                // four `total_damage` answers and `a[11..16]` the five `hits` answers, in call order; `a[16]` slot\n                // `+0x158` is `Object::die` itself, `a[17]` slot `+0x11c` is `UnitData::hits` itself. Windows: m `this`;\n                // p1.. the six objects (the two players' arrays are one array, the second at +12) and p8.. the data\n                // each answers (slot `+0xa8`, its `o_down` laid by the answer); p16..p21 the two players' units (three\n                // a player, `o_up` laid forward); p22 `this`'s type record, p23.. the data's.\n                const M: usize = usize::MAX;\n                macro_rules! w {\n                    ($r:expr) => {\n                        if $r == M { &*m } else { &p[$r] }\n                    };\n                }\n                let rs = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));\n                let name = |r: usize| {\n                    if r == M {\n                        \"m+0x0\".to_string()\n                    } else {\n                        format!(\"p{r}+0x0\")\n                    }\n                };\n                let entry = |who: usize, k: usize| if who == 0 { k } else { 3 + k };\n                let (die_base, hits_img) = (a[16] != 0, a[17] != 0);\n                let (mut tn, mut hn) = (0usize, 0usize);\n                macro_rules! hits {\n                    ($r:expr) => {{\n                        let r = $r;\n                        if hits_img {\n                            rd(w!(r), 0x20)\n                        } else {\n                            log.push(format!(\"s3:{},0\", name(r)));\n                            let v = a[11 + hn.min(4)];\n                            hn += 1;\n                            v\n                        }\n                    }};\n                }\n                macro_rules! td {\n                    ($r:expr) => {{\n                        let r = $r;\n                        log.push(format!(\"s0:{},0,0\", name(r)));\n                        let v = a[7 + tn.min(3)];\n                        tn += 1;\n                        v\n                    }};\n                }\n                let mut this = M;\n                let mut sv = rs(m, 0x90);\n                while sv >= 0 {\n                    let e = entry(w!(this)[9] as usize, sv as usize);\n                    let d = 8 + e;\n                    if rd(&p[d], 8) & 1 == 0 {\n                        break;\n                    }\n                    this = d;\n                    sv = rs(&p[d], 0x90);\n                }\n                let ty = if this == M { 22 } else { 23 + (this - 8) };\n                let uber = rd(&p[ty], 0x308);\n                let iv4 = hits!(this) / uber;\n                let h1 = hits!(this);\n                let td0 = td!(this);\n                let iv6 = (td0.wrapping_shl(8)) / h1;\n                let (pa, pb) = (a[1] as usize, a[0] as usize);\n                let ob = 1 + entry(pa, pb);\n                let tdu = td!(16 + 3 * pa + pb);\n                let hob = hits!(ob);\n                let mut iv5 = (tdu.wrapping_shl(8)) / hob;\n                if iv5 < iv6 {\n                    iv5 = iv6;\n                }\n                let h2 = hits!(this);\n                let td2 = td!(this);\n                let (mut sidx, who) = (rs(w!(this), 0xa), w!(this)[9] as usize);\n                let prod = h2.wrapping_mul(iv5);\n                let mut iv7 = (prod.wrapping_add((prod >> 31) & 0xff) >> 8).wrapping_sub(td2);\n                if iv7 < 0 {\n                    iv7 = 0;\n                }\n                if iv7 != 0 {\n                    loop {\n                        if sidx < 0 {\n                            break;\n                        }\n                        let u = 16 + 3 * who + sidx as usize;\n                        if rd(&p[u], 8) & 1 == 0 {\n                            break;\n                        }\n                        let next = rs(&p[u], 0x8e);\n                        if next < 0 {\n                            let h = hits!(u);\n                            let t = td!(u);\n                            let mut cap = h.wrapping_sub(t).wrapping_sub(1);\n                            if iv7 < cap {\n                                cap = iv7;\n                            }\n                            wr(&mut p[u], 0x24, cap);\n                            break;\n                        }\n                        if iv7 < iv4 {\n                            wr(&mut p[u], 0x24, iv7);\n                            break;\n                        }\n                        if die_base {\n                            log.push(format!(\"s1:{},0,-1,0\", name(u)));\n                        } else {\n                            log.push(format!(\"s4:{}\", name(u)));\n                        }\n                        iv7 = iv7.wrapping_sub(iv4);\n                        if iv7 == 0 {\n                            break;\n                        }\n                        sidx = next;\n                    }\n                }\n                0\n            }"]
