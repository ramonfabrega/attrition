#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_guy_process.py — `Guy::process@005e0230` under unicorn (item 3396, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_guy_process.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 176-byte object of random bytes (no bias), 0 stack argument(s);
`p` windows in order: p0..p3 the place(s), in flag order, p4..p9 the stubs' answer window(s); the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 832 random bytes; 128 random bytes at 0x00c0aec0; 1536 random bytes at 0x00d10000; callee 0x005d9240 (0 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x0046d250 (2 args) stubbed; the word at `c1+0x134` points at 448 random bytes less `0` (placed); the word at `c3+i8(m,0xa1)*0x1c` points at 4 random bytes less `i16(m,0x8c)*4` (placed); the word at `p1+0` points at 32 random bytes less `0` (placed); the word at `p2+0x18` points at 784 random bytes less `0` (placed); stub 1 answered from [-1, 0, 1, 1, 2, 3, 32767, 32768, 65535] (90%); stub 2 answers a window of its own to each call, in call order (112 bytes, ...); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call).
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
import sweeplib  # noqa: E402

ENTRY = 0x005e0230
WIN = 176
OWN = 0
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607980, 1376), (12607880, 320), (13297148, 832)]
TLS = []
OUTRET = []
TABLEARG = None
AT = [(12627648, 128), (13697024, 1536)]
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('call', 6132288, 0), ('call', 7033568, 2), ('call', 4641360, 2)]
PLACES = [('c1', '0x134', '0', 448), ('c3', 'i8(m,0xa1)*0x1c', 'i16(m,0x8c)*4', 4), ('p1', '0', '0', 32), ('p2', '0x18', '0', 784)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 160, ('set', [0, 256, 512, 768, 65536, 65792, 66048, 66304, 131072, 131328, 131584, 131840, 196608, 196864, 197120, 197376])), ('m', 140, ('set', [0, 0, 0, 0, 0, 1, 2])), ('m', 132, ('set', [0, 0, 0, 0, 0, 0, 1, -1, 256])), ('m', 12, ('range', 0, 3071)), ('m', 16, ('range', 0, 3071)), ('m', 32, ('set', [0, 178956970, 178956969, 178956971, -178956970, -178956971, -178956969, 536870912, 1073741824, -2147483648, 2147483647, -1, 1, 1431655765, -1431655766])), ('m', 36, ('set', [0, 178956970, 178956969, 178956971, -178956970, -178956971, -178956969, 536870912, 1073741824, -2147483648, 2147483647, -1, 1, 1431655765, -1431655766])), ('m', 40, ('set', [0, 178956970, 178956969, 178956971, -178956970, -178956971, -178956969, 536870912, 1073741824, -2147483648, 2147483647, -1, 1, 1431655765, -1431655766])), ('m', 44, ('set', [0, 178956970, 178956969, 178956971, -178956970, -178956971, -178956969, 536870912, 1073741824, -2147483648, 2147483647, -1, 1, 1431655765, -1431655766])), ('m', 48, ('set', [0, 178956970, 178956969, 178956971, -178956970, -178956971, -178956969, 536870912, 1073741824, -2147483648, 2147483647, -1, 1, 1431655765, -1431655766])), ('m', 52, ('set', [0, 178956970, 178956969, 178956971, -178956970, -178956971, -178956969, 536870912, 1073741824, -2147483648, 2147483647, -1, 1, 1431655765, -1431655766])), ('m', 56, ('set', [0, 178956970, 178956969, 178956971, -178956970, -178956971, -178956969, 536870912, 1073741824, -2147483648, 2147483647, -1, 1, 1431655765, -1431655766])), ('m', 60, ('set', [0, 178956970, 178956969, 178956971, -178956970, -178956971, -178956969, 536870912, 1073741824, -2147483648, 2147483647, -1, 1, 1431655765, -1431655766])), ('m', 152, ('set', [0, 4660, 131072, 135732, 2097152, 2101812, 2228224, 2232884, 16777216, 16781876, 16908288, 16912948, 18874368, 18879028, 19005440, 19010100, 65536, 70196, 196608, 201268, 18939904, 18944564])), ('c0', 1360, ('set', [0, 5, 100, -1, -5, -64, -3, 0, 64, 128, 0, 64, 0, 64, 0, 128, 32, 96, 32, 96, 31, -32, 33, 0, -1, -2, -3, 64, 63, 62, 61, 128, 127, 126, 125, 192, 191, 190, 189])), ('c1', 0, ('range', 1, 4)), ('c1', 4, ('range', 1, 4)), ('p0', 4, ('set', [0, 1, 1, 2, 65535, 65536, 3])), ('p0', 24, ('set', [13697024, 0, 0])), ('p0', 32, ('set', [0, 1, 1, 2, 65535, 65536, 3])), ('p0', 52, ('set', [13697120, 0, 0])), ('p0', 60, ('set', [0, 1, 1, 2, 65535, 65536, 3])), ('p0', 80, ('set', [13697216, 0, 0])), ('p0', 88, ('set', [0, 1, 1, 2, 65535, 65536, 3])), ('p0', 108, ('set', [13697312, 0, 0])), ('p0', 116, ('set', [0, 1, 1, 2, 65535, 65536, 3])), ('p0', 136, ('set', [13697408, 0, 0])), ('p0', 144, ('set', [0, 1, 1, 2, 65535, 65536, 3])), ('p0', 164, ('set', [13697504, 0, 0])), ('p0', 172, ('set', [0, 1, 1, 2, 65535, 65536, 3])), ('p0', 192, ('set', [13697600, 0, 0])), ('p0', 200, ('set', [0, 1, 1, 2, 65535, 65536, 3])), ('p0', 220, ('set', [13697696, 0, 0])), ('p0', 228, ('set', [0, 1, 1, 2, 65535, 65536, 3])), ('p0', 248, ('set', [13697792, 0, 0])), ('p0', 256, ('set', [0, 1, 1, 2, 65535, 65536, 3])), ('p0', 276, ('set', [13697888, 0, 0])), ('p0', 284, ('set', [0, 1, 1, 2, 65535, 65536, 3])), ('p0', 304, ('set', [13697984, 0, 0])), ('p0', 312, ('set', [0, 1, 1, 2, 65535, 65536, 3])), ('p0', 332, ('set', [13698080, 0, 0])), ('p0', 340, ('set', [0, 1, 1, 2, 65535, 65536, 3])), ('p0', 360, ('set', [13698176, 0, 0])), ('p0', 368, ('set', [0, 1, 1, 2, 65535, 65536, 3])), ('p0', 388, ('set', [13698272, 0, 0])), ('p0', 396, ('set', [0, 1, 1, 2, 65535, 65536, 3])), ('p0', 416, ('set', [13698368, 0, 0])), ('p0', 424, ('set', [0, 1, 1, 2, 65535, 65536, 3])), ('p0', 444, ('set', [13698464, 0, 0])), ('p3', 536, ('set', [0, 0, 1, 2, 0])), ('p3', 772, ('set', [1, 2, 3, 4, 4, 0, 2])), ('p3', 584, ('set', [0, 1, 2, 3, 4, 5, 6, 7, 1, 2, 3]))]
FILLS = [('c2', 'i//3')]
ARGRANGE = []
SLOTBASE = []
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(1, [-1, 0, 1, 1, 2, 3, 32767, 32768, 65535])]
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = []
ANSWERBY = []
ANSWERWIN = [(2, 112), (2, 112), (2, 112), (2, 112), (2, 112), (2, 112)]
CDECL = False
TIB = False
ARGEQ = []
SEQ = []
ANSRANGE = []
CELLTHIS = []
VTALSO = []
SPARSE = []
GTAB = []
RECORD = [1, 2]
RECORDORDER = False
RECORDEDX = False
ROWSPLIT = {}
LINKS = []
LINKHI = []
OUTARG = []
CDECLSTUBS = []
STUBWRITE = []
STUBCOPY = []
STUBALLOC = []
STACKCLEAN = 0
STOPSTUB = None
INITCALLS = []
MAXINSNS = 200000
ECXANSWER = []
ECXOUT = []
RING = []
DEREF = [(1, 0, 4, None), (1, 1, 4, None), (2, 0, 4, None), (2, 1, 4, None)]
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


def tls_lay(write, tib, tls):
    """`--tls OFF:V` (item 3377, parked 3372d): `fs:[0x2c]` -> a TLS array of 64 slots in the TIB
    page, every slot one zeroed block whose word at OFF is V. A function-local static's `_TSS` guard
    compares the block's epoch (`+8`) with its own word, so 0 reads "already initialised"."""
    arr, block = tib + 0x100, tib + 0x200
    write(tib + 0x2C, arr.to_bytes(4, "little"))
    write(block, bytes(0xE00))
    for k in range(64):
        write(arr + 4 * k, block.to_bytes(4, "little"))
    for off, v in tls:
        write(block + off, (v & 0xFFFFFFFF).to_bytes(4, "little"))


def outret_answer(vals, write):
    """`--outret IDX:N:SPEC/..` (item 3377, parked 3372d): a struct-returning callee. MSVC passes
    the caller's buffer as a hidden first stack argument and the callee answers it; call k writes
    `vals[k]`'s words there (the last call's past N)."""
    turn = [0]

    def answer(ecx, args):
        words = vals[min(turn[0], len(vals) - 1)]
        turn[0] += 1
        at = args[0] & 0xFFFFFFFF
        write(at, b"".join((w & 0xFFFFFFFF).to_bytes(4, "little") for w in words))
        return at
    return answer


def table_hook(base, esize, n, draw, cells):
    """`--table-arg` (item 3377, parked 3372d): a read hook over an N-cell table that fills each
    cell the function reads the first time it is read: a value from `draw()`, written before the
    load (unicorn's read hook runs first, and the load sees the write), recorded in `cells` as the
    signed value the cell holds."""
    def hook(uc_, access, addr, size, value, _):
        first = max(addr - base, 0) // esize
        last = min((addr + size - 1 - base) // esize, n - 1)
        for k in range(first, last + 1):
            if k not in cells:
                b = (draw() & ((1 << 8 * esize) - 1)).to_bytes(esize, "little")
                cells[k] = int.from_bytes(b, "little", signed=True)
                uc_.mem_write(base + esize * k, b)
    return hook


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
    rng = random.Random(3396)
    # `--stub-cdecl IDX`: the callee's arguments are the caller's to pop (item 2325).
    h = Harness(argv[1], {a: (f"{a:x}", n, False, i in CDECLSTUBS) for i, (k, a, n) in enumerate(STUBS)
                          if k == "call"})
    uc = h.m.uc
    # `--tib`: an SEH-framed function reads and writes `fs:[0]` (item 3120).
    if TIB:
        set_tib(uc)
    if TLS:
        tls_lay(uc.mem_write, sweeplib.TIB, TLS)
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
    # `--table-arg` (item 3377): the table laid once, zeroed; a read hook fills the cells read.
    tcells = {}
    twritten = [False]
    if TABLEARG is not None:
        from unicorn import UC_HOOK_MEM_READ, UC_HOOK_MEM_WRITE
        tidx, trows, tcols, tesz, tvals = TABLEARG
        tsize = trows * tcols * tesz
        tbase = h.alloc(tsize + 0x20)
        uc.mem_write(tbase, bytes(tsize))
        uc.hook_add(UC_HOOK_MEM_READ, table_hook(tbase, tesz, trows * tcols, lambda: wdraw(rng, tvals), tcells),
                    begin=tbase, end=tbase + tsize - 1)
        uc.hook_add(UC_HOOK_MEM_WRITE, lambda *_: twritten.__setitem__(0, True), begin=tbase, end=tbase + tsize - 1)
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
    for i in range(5000):
        h.brk = base  # the bump heap is reused a row; 5000 rows of windows outgrow it
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
        # `--stub-alloc IDX` (parked 3282b, 3339a, 3348b/c, 3372): a stubbed `malloc` answering a fresh
        # address of the bump heap each call, so a stack-resident `SimpleArray` grows into its own buffer.
        for idx in STUBALLOC:
            h.answers[names[idx]] = lambda ecx, args: h.alloc(max(args[0] & 0xFFFFFFFF, 16))
        derefs = [[] for _ in DEREF]
        for (idx, k, n, wr), sink in zip(DEREF, derefs):
            h.answers[names[idx]] = read_arg(h.answers[names[idx]], k, n,
                                             lambda a_, n_: bytes(uc.mem_read(a_, n_)), sink, wr)
        # `--ecx-out IDX:V,V,..` (item 2818, parked 2816): stub IDX leaves `ecx`
        # as a callee would, a value drawn from the list, after the other numbers on the row.
        ecxo = [rng.choice(vals) & 0xFFFFFFFF for _, vals in ECXOUT]
        for (idx, _), v in zip(ECXOUT, ecxo):
            h.answers[names[idx]] = pair_answer(h.answers[names[idx]], v)
        # `--outret IDX:N:SPEC/..` (item 3377): N calls' words drawn a row, printed last on the row.
        orets = []
        for idx, n, specs in OUTRET:
            vals = [[wdraw(rng, sp_) for sp_ in specs] for _ in range(n)]
            orets.append(vals)
            h.answers[names[idx]] = outret_answer(vals, uc.mem_write)
        if TABLEARG is not None:
            for k_ in tcells:
                uc.mem_write(tbase + tesz * k_, bytes(tesz))
            tcells.clear()
            twritten[0] = False
            args[tidx] = tbase
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
        # A recursive function stubs its own entry (item 3386): the harness's call runs it once.
        if ENTRY in h.hooked:
            h.skip_once.add(ENTRY)
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
        assert not twritten[0], "wrote the --table-arg table"
        before = ";".join([bytes(mem).hex()] + [bytes(b).hex() for _, b in wins])
        after = ";".join([bytes(uc.mem_read(this, WIN)).hex()]
                         + [bytes(uc.mem_read(a, len(b))).hex() for a, b in wins])
        shown = ([str(a) for a in args] + [str(v) for v in answers] + [str(v) for v in outs]
                 + [str(v) for s in seqs for v in s] + [str(f) for f in based] + [str(f) for f in nullflags]
                 + ([] if ECXINT is None else [str(ecx)]) + ([] if EDX is None else [str(edx)])
                 + [str(v) for v in xmmv] + [str(v) for v in ecxo]
                 + [str(((w + 0x80000000) & 0xFFFFFFFF) - 0x80000000) for vals in orets for call in vals for w in call])
        tshown = "" if TABLEARG is None else " @ " + (",".join(f"{k_}:{v_}" for k_, v_ in sorted(tcells.items())) or "-")
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
        print(" ".join(shown) + f" | {before} -> {r} | {after}" + tshown + log)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

# sweep_gen: ["obj", "Guy::process@005e0230", "--item", "3396", "--win", "0xb0", "--own", "0", "--ret", "none", "--rows", "5000", "--cell-win", "0xc061ec:0x560", "--cell-win", "0xc06188:0x140", "--cell-win", "0xcae5fc:0x340", "--at", "0xc0aec0:0x80", "--at", "0xd10000:0x600", "--fill", "c2:i//3", "--place", "c1+0x134:0:0x1c0", "--place", "c3+i8(m,0xa1)*0x1c:i16(m,0x8c)*4:4", "--place", "p1+0:0:0x20", "--place", "p2+0x18:0:0x310", "--stub", "0x5d9240:0", "--stub", "0x6b52e0:2", "--stub", "0x46d250:2", "--answers", "1:-1,0,1,1,2,3,32767,32768,65535", "--answer-win", "2:0x70", "--answer-win", "2:0x70", "--answer-win", "2:0x70", "--answer-win", "2:0x70", "--answer-win", "2:0x70", "--answer-win", "2:0x70", "--deref", "1:0:4", "--deref", "1:1:4", "--deref", "2:0:4", "--deref", "2:1:4", "--record", "1", "--record", "2", "--wbias", "m+0xa0:0,256,512,768,65536,65792,66048,66304,131072,131328,131584,131840,196608,196864,197120,197376", "--wbias", "m+0x8c:0,0,0,0,0,1,2", "--wbias", "m+0x84:0,0,0,0,0,0,1,-1,256", "--wbias", "m+0xc:0..3071", "--wbias", "m+0x10:0..3071", "--wbias", "m+0x20:0,178956970,178956969,178956971,-178956970,-178956971,-178956969,536870912,1073741824,-2147483648,2147483647,-1,1,1431655765,-1431655766", "--wbias", "m+0x24:0,178956970,178956969,178956971,-178956970,-178956971,-178956969,536870912,1073741824,-2147483648,2147483647,-1,1,1431655765,-1431655766", "--wbias", "m+0x28:0,178956970,178956969,178956971,-178956970,-178956971,-178956969,536870912,1073741824,-2147483648,2147483647,-1,1,1431655765,-1431655766", "--wbias", "m+0x2c:0,178956970,178956969,178956971,-178956970,-178956971,-178956969,536870912,1073741824,-2147483648,2147483647,-1,1,1431655765,-1431655766", "--wbias", "m+0x30:0,178956970,178956969,178956971,-178956970,-178956971,-178956969,536870912,1073741824,-2147483648,2147483647,-1,1,1431655765,-1431655766", "--wbias", "m+0x34:0,178956970,178956969,178956971,-178956970,-178956971,-178956969,536870912,1073741824,-2147483648,2147483647,-1,1,1431655765,-1431655766", "--wbias", "m+0x38:0,178956970,178956969,178956971,-178956970,-178956971,-178956969,536870912,1073741824,-2147483648,2147483647,-1,1,1431655765,-1431655766", "--wbias", "m+0x3c:0,178956970,178956969,178956971,-178956970,-178956971,-178956969,536870912,1073741824,-2147483648,2147483647,-1,1,1431655765,-1431655766", "--wbias", "m+0x98:0,4660,131072,135732,2097152,2101812,2228224,2232884,16777216,16781876,16908288,16912948,18874368,18879028,19005440,19010100,65536,70196,196608,201268,18939904,18944564", "--wbias", "c0+0x550:0,5,100,-1,-5,-64,-3,0,64,128,0,64,0,64,0,128,32,96,32,96,31,-32,33,0,-1,-2,-3,64,63,62,61,128,127,126,125,192,191,190,189", "--wbias", "c1+0:1..4", "--wbias", "c1+4:1..4", "--wbias", "p0+4:0,1,1,2,65535,65536,3", "--wbias", "p0+24:13697024,0,0", "--wbias", "p0+32:0,1,1,2,65535,65536,3", "--wbias", "p0+52:13697120,0,0", "--wbias", "p0+60:0,1,1,2,65535,65536,3", "--wbias", "p0+80:13697216,0,0", "--wbias", "p0+88:0,1,1,2,65535,65536,3", "--wbias", "p0+108:13697312,0,0", "--wbias", "p0+116:0,1,1,2,65535,65536,3", "--wbias", "p0+136:13697408,0,0", "--wbias", "p0+144:0,1,1,2,65535,65536,3", "--wbias", "p0+164:13697504,0,0", "--wbias", "p0+172:0,1,1,2,65535,65536,3", "--wbias", "p0+192:13697600,0,0", "--wbias", "p0+200:0,1,1,2,65535,65536,3", "--wbias", "p0+220:13697696,0,0", "--wbias", "p0+228:0,1,1,2,65535,65536,3", "--wbias", "p0+248:13697792,0,0", "--wbias", "p0+256:0,1,1,2,65535,65536,3", "--wbias", "p0+276:13697888,0,0", "--wbias", "p0+284:0,1,1,2,65535,65536,3", "--wbias", "p0+304:13697984,0,0", "--wbias", "p0+312:0,1,1,2,65535,65536,3", "--wbias", "p0+332:13698080,0,0", "--wbias", "p0+340:0,1,1,2,65535,65536,3", "--wbias", "p0+360:13698176,0,0", "--wbias", "p0+368:0,1,1,2,65535,65536,3", "--wbias", "p0+388:13698272,0,0", "--wbias", "p0+396:0,1,1,2,65535,65536,3", "--wbias", "p0+416:13698368,0,0", "--wbias", "p0+424:0,1,1,2,65535,65536,3", "--wbias", "p0+444:13698464,0,0", "--wbias", "p3+0x218:0,0,1,2,0", "--wbias", "p3+0x304:1,2,3,4,4,0,2", "--wbias", "p3+0x248:0,1,2,3,4,5,6,7,1,2,3"]
