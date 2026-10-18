#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_find_goody_box.py — `Unit::find_goody_box@005f2540` under unicorn (item 3239, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_unit_find_goody_box.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 128-byte object of random bytes (no bias), 0 stack argument(s);
`p` windows in order: p0..p7 the place(s), in flag order; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00cae5fc points at 2048 random bytes; 196 random bytes at 0x00adc400 (each word in -3..3); 196 random bytes at 0x00adcaf0 (each word in -3..3); callee 0x006b53f0 (3 args) stubbed; callee 0x0065c040 (1 args) stubbed; callee 0x005f7690 (2 args) stubbed; vtable slot +0x48 (2 args) stubbed; the word at `0xc0a110` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `c0+0x134` points at 2048 random bytes less `0` (placed); the word at `c1+0x134` points at 2048 random bytes less `0` (placed); the word at `m+0x18` points at 560 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 6 values in call order, printed after the answers; stub 3 answers 6 values in call order, printed after the answers; stub 1 answers -1..3 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 0 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`.
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

ENTRY = 0x005f2540
WIN = 128
OWN = 0
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607880, 320), (12607952, 320), (13297148, 2048)]
AT = [(11387904, 196), (11389680, 196)]
ATRANGE = {11387904: (-3, 3, 4), 11389680: (-3, 3, 4)}
PTRARGS = []
PNULLS = []
STUBS = [('call', 7033840, 3), ('call', 6668352, 1), ('call', 6256272, 2), ('slot', 72, 2)]
PLACES = [(None, '0xc0a110', '0', 16), ('p0', '0', '0', 32), ('p0', '4', '0', 32), ('p0', '8', '0', 32), ('p0', '12', '0', 32), ('c0', '0x134', '0', 2048), ('c1', '0x134', '0', 2048), ('m', '0x18', '0', 560)]
NULLS = []
NULLFORCE = {}
WBIAS = [('c0', 0, ('range', 3, 6)), ('c0', 4, ('range', 3, 6)), ('c1', 0, ('range', 3, 6)), ('c1', 4, ('range', 3, 6)), ('p5', 0, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 4, ('set', [0, 0, 0, 1])), ('p6', 0, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 28, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 32, ('set', [0, 0, 0, 1])), ('p6', 28, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 56, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 60, ('set', [0, 0, 0, 1])), ('p6', 56, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 84, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 88, ('set', [0, 0, 0, 1])), ('p6', 84, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 112, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 116, ('set', [0, 0, 0, 1])), ('p6', 112, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 140, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 144, ('set', [0, 0, 0, 1])), ('p6', 140, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 168, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 172, ('set', [0, 0, 0, 1])), ('p6', 168, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 196, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 200, ('set', [0, 0, 0, 1])), ('p6', 196, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 224, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 228, ('set', [0, 0, 0, 1])), ('p6', 224, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 252, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 256, ('set', [0, 0, 0, 1])), ('p6', 252, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 280, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 284, ('set', [0, 0, 0, 1])), ('p6', 280, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 308, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 312, ('set', [0, 0, 0, 1])), ('p6', 308, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 336, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 340, ('set', [0, 0, 0, 1])), ('p6', 336, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 364, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 368, ('set', [0, 0, 0, 1])), ('p6', 364, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 392, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 396, ('set', [0, 0, 0, 1])), ('p6', 392, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 420, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 424, ('set', [0, 0, 0, 1])), ('p6', 420, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 448, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 452, ('set', [0, 0, 0, 1])), ('p6', 448, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 476, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 480, ('set', [0, 0, 0, 1])), ('p6', 476, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 504, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 508, ('set', [0, 0, 0, 1])), ('p6', 504, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 532, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 536, ('set', [0, 0, 0, 1])), ('p6', 532, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 560, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 564, ('set', [0, 0, 0, 1])), ('p6', 560, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 588, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 592, ('set', [0, 0, 0, 1])), ('p6', 588, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 616, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 620, ('set', [0, 0, 0, 1])), ('p6', 616, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 644, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 648, ('set', [0, 0, 0, 1])), ('p6', 644, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 672, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 676, ('set', [0, 0, 0, 1])), ('p6', 672, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 700, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 704, ('set', [0, 0, 0, 1])), ('p6', 700, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 728, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 732, ('set', [0, 0, 0, 1])), ('p6', 728, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 756, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 760, ('set', [0, 0, 0, 1])), ('p6', 756, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 784, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 788, ('set', [0, 0, 0, 1])), ('p6', 784, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 812, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 816, ('set', [0, 0, 0, 1])), ('p6', 812, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 840, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 844, ('set', [0, 0, 0, 1])), ('p6', 840, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 868, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 872, ('set', [0, 0, 0, 1])), ('p6', 868, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 896, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 900, ('set', [0, 0, 0, 1])), ('p6', 896, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 924, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 928, ('set', [0, 0, 0, 1])), ('p6', 924, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 952, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 956, ('set', [0, 0, 0, 1])), ('p6', 952, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('p5', 980, ('set', [32768, 32768, 32769, 0, 32767, 98304])), ('p5', 984, ('set', [0, 0, 0, 1])), ('p6', 980, ('set', [65536, 131072, 196864, 256, 0, 65792, 131328, 130816])), ('m', 16, ('set', [407095, 407351, 406583, 406839, 406071, 406327, 405559, 405815, 409143, 409399, 408631, 408887, 408119, 408375, 407607])), ('m', 20, ('set', [407095, 407351, 406583, 406839, 406071, 406327, 405559, 405815, 409143, 409399, 408631, 408887, 408119, 408375, 407607])), ('m', 112, ('set', [0, 256, 512, 768, 1024, 1280, 1536, 1792, 2048, 2304, 2560, 2816, 3072, 3328, 3584])), ('m', 116, ('set', [0, 256, 512, 768, 1024, 1280, 1536, 1792, 2048, 2304, 2560, 2816, 3072, 3328, 3584])), ('p7', 536, ('set', [0, 0, 0, 1]))]
FILLS = [('c2', '(i*5+i//3)%7')]
ARGRANGE = []
SLOTBASE = []
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [0, 0, 0, 1]), (3, [0, 1])]
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = []
ANSWERBY = []
ANSWERWIN = []
CDECL = False
TIB = False
ARGEQ = []
SEQ = [(0, 8, None), (1, 6, None), (3, 6, None)]
ANSRANGE = [(1, -1, 3)]
CELLTHIS = []
VTALSO = [('p1', 0), ('p2', 0), ('p3', 0), ('p4', 0)]
SPARSE = []
GTAB = []
RECORD = [2, 3]
RECORDORDER = False
RECORDEDX = False
ROWSPLIT = {}
LINKS = []
LINKHI = []
OUTARG = []
CDECLSTUBS = []
STUBWRITE = []
STUBCOPY = []
STACKCLEAN = 0
ECXANSWER = []
ECXOUT = []
RING = []
DEREF = [(0, 0, 4, None), (0, 1, 4, None), (1, 0, 4, None)]
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
    rng = random.Random(3239)
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

# sweep_gen: ["--item", "3239", "obj", "Unit::find_goody_box@005f2540", "--win", "0x80", "--ret", "eax", "--cell-win", "0xc06188:0x140", "--cell-win", "0xc061d0:0x140", "--cell-win", "0xcae5fc:0x800", "--at", "0xadc400:196:-3..3", "--at", "0xadcaf0:196:-3..3", "--ptrtab", "0xc0a110:4:0x20:vt", "--place", "c0+0x134:0:2048", "--place", "c1+0x134:0:2048", "--place", "m+0x18:0:0x230", "--stub", "0x6b53f0:3", "--stub", "0x65c040:1", "--stub", "0x5f7690:2", "--stub", "slot:0x48:2", "--seq", "0:8", "--seq", "1:6", "--seq", "3:6", "--answers", "0:0,0,0,1", "--answers", "3:0,1", "--answer-range", "1:-1:3", "--fill", "c2:(i*5+i//3)%7", "--wbias", "c0+0:3..6", "--wbias", "c0+4:3..6", "--wbias", "c1+0:3..6", "--wbias", "c1+4:3..6", "--wbias", "p5+0:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+4:0,0,0,1", "--wbias", "p6+0:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+28:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+32:0,0,0,1", "--wbias", "p6+28:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+56:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+60:0,0,0,1", "--wbias", "p6+56:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+84:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+88:0,0,0,1", "--wbias", "p6+84:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+112:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+116:0,0,0,1", "--wbias", "p6+112:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+140:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+144:0,0,0,1", "--wbias", "p6+140:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+168:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+172:0,0,0,1", "--wbias", "p6+168:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+196:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+200:0,0,0,1", "--wbias", "p6+196:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+224:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+228:0,0,0,1", "--wbias", "p6+224:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+252:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+256:0,0,0,1", "--wbias", "p6+252:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+280:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+284:0,0,0,1", "--wbias", "p6+280:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+308:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+312:0,0,0,1", "--wbias", "p6+308:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+336:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+340:0,0,0,1", "--wbias", "p6+336:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+364:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+368:0,0,0,1", "--wbias", "p6+364:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+392:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+396:0,0,0,1", "--wbias", "p6+392:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+420:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+424:0,0,0,1", "--wbias", "p6+420:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+448:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+452:0,0,0,1", "--wbias", "p6+448:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+476:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+480:0,0,0,1", "--wbias", "p6+476:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+504:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+508:0,0,0,1", "--wbias", "p6+504:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+532:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+536:0,0,0,1", "--wbias", "p6+532:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+560:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+564:0,0,0,1", "--wbias", "p6+560:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+588:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+592:0,0,0,1", "--wbias", "p6+588:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+616:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+620:0,0,0,1", "--wbias", "p6+616:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+644:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+648:0,0,0,1", "--wbias", "p6+644:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+672:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+676:0,0,0,1", "--wbias", "p6+672:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+700:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+704:0,0,0,1", "--wbias", "p6+700:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+728:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+732:0,0,0,1", "--wbias", "p6+728:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+756:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+760:0,0,0,1", "--wbias", "p6+756:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+784:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+788:0,0,0,1", "--wbias", "p6+784:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+812:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+816:0,0,0,1", "--wbias", "p6+812:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+840:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+844:0,0,0,1", "--wbias", "p6+840:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+868:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+872:0,0,0,1", "--wbias", "p6+868:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+896:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+900:0,0,0,1", "--wbias", "p6+896:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+924:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+928:0,0,0,1", "--wbias", "p6+924:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+952:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+956:0,0,0,1", "--wbias", "p6+952:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "p5+980:0x8000,0x8000,0x8001,0,0x7fff,0x18000", "--wbias", "p5+984:0,0,0,1", "--wbias", "p6+980:0x10000,0x20000,0x30100,0x100,0,0x10100,0x20100,0x1ff00", "--wbias", "m+0x10:407095,407351,406583,406839,406071,406327,405559,405815,409143,409399,408631,408887,408119,408375,407607", "--wbias", "m+0x14:407095,407351,406583,406839,406071,406327,405559,405815,409143,409399,408631,408887,408119,408375,407607", "--wbias", "m+0x70:0,256,512,768,1024,1280,1536,1792,2048,2304,2560,2816,3072,3328,3584", "--wbias", "m+0x74:0,256,512,768,1024,1280,1536,1792,2048,2304,2560,2816,3072,3328,3584", "--wbias", "p7+0x218:0,0,0,1", "--record", "2", "--record", "3", "--deref", "0:0:4", "--deref", "0:1:4", "--deref", "1:0:4", "--rows", "1500", "--port", "{\n                // `Unit::find_goody_box`: a unit whose type (`this+0x18`, `p7`) has `+0x218` clear sweeps the 49\n                // cells (`move_x` at 0xadcaf0 = c4, `move_y` at 0xadc400 = c3) round the cell its position\n                // (`this+0x10/+0x14`, through `div_3_table`, c2) names. A cell must be in the world (c0, cells\n                // `p5`), in the unit's own region (+4), carry GOODY (the sign of the flag short) and be seen\n                // at any of its four half-cells (stub 0, `was_seen`, called in the listing's order); then\n                // the second world (`worldc`, c1, cells `p6`) answers at once for a flag-0x100-clear cell of\n                // type 1 or 2, else `find_goody_at` (stub 1) names an item whose slot 0x48 (stub 3) must\n                // answer. The cell the unit already walks to ends the sweep; any other `get_goody_box`es.\n                let sh = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));\n                let hexw = |v: i32| v.to_le_bytes().iter().map(|b| format!(\"{b:02x}\")).collect::<String>();\n                let col = |v: &Vec<String>| if v.is_empty() { \"-\".to_string() } else { v.join(\"/\") };\n                let (mut l_gb, mut l_it, mut l_wx, mut l_wy, mut l_fy): (Vec<String>, Vec<String>, Vec<String>, Vec<String>, Vec<String>) =\n                    (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());\n                let mut res: i64 = 0;\n                if rd(&p[7], 0x218) == 0 {\n                    let div = |v: i32| rd(&c[2], 4 * usize::try_from(v).expect(\"table index\"));\n                    let bx = div((rd(m, 0x10) ^ 0x63637) >> 8);\n                    let by = div((rd(m, 0x14) ^ 0x63637) >> 8);\n                    let (w0, h0, w1) = (rd(&c[0], 0), rd(&c[0], 4), rd(&c[1], 0));\n                    let cell = |x: i32, y: i32| usize::try_from(w0 * y + x).expect(\"cell\") * 0x1c;\n                    let region = sh(&p[5], cell(bx, by) + 4) as u16;\n                    let who = i32::from(m[9]);\n                    let (mut k0, mut k1, mut k3) = (0usize, 0usize, 0usize);\n                    for i in 0..49 {\n                        let x = rd(&c[4], 4 * i) + bx;\n                        let y = rd(&c[3], 4 * i) + by;\n                        if x < 0 || y < 0 || x >= w0 || y >= h0 {\n                            continue;\n                        }\n                        let o = cell(x, y);\n                        if sh(&p[5], o + 4) as u16 != region || sh(&p[5], o) >= 0 {\n                            continue;\n                        }\n                        let mut seen = false;\n                        for (px, py) in [(2 * x + 1, 2 * y + 1), (2 * x, 2 * y + 1), (2 * x + 1, 2 * y), (2 * x, 2 * y)] {\n                            l_wx.push(hexw(px));\n                            l_wy.push(hexw(py));\n                            let ans = a[4 + k0.min(7)];\n                            k0 += 1;\n                            if ans != 0 {\n                                seen = true;\n                                break;\n                            }\n                        }\n                        if !seen {\n                            continue;\n                        }\n                        let o2 = usize::try_from(w1 * y + x).expect(\"worldc cell\") * 0x1c;\n                        let fl = u16::from_le_bytes([p[6][o2], p[6][o2 + 1]]);\n                        let mut accept = false;\n                        if fl & 0x100 == 0 {\n                            let t = p[6][o2 + 2] as i8;\n                            accept = t == 1 || t == 2;\n                        }\n                        if !accept {\n                            l_fy.push(hexw(y));\n                            let found = a[12 + k1.min(5)];\n                            k1 += 1;\n                            if found < 0 {\n                                accept = true;\n                            } else {\n                                let it = 1 + usize::try_from(found).expect(\"item\");\n                                l_it.push(format!(\"p{it}+0x0,{who},0\"));\n                                let ans = a[18 + k3.min(5)];\n                                k3 += 1;\n                                accept = ans != 0;\n                            }\n                        }\n                        if !accept {\n                            continue;\n                        }\n                        let (ox, oy) = (div(rd(m, 0x70) >> 8), div(rd(m, 0x74) >> 8));\n                        if !(x == ox && y == oy) {\n                            l_gb.push(format!(\"m+0x0,{x},{y}\"));\n                            res = 1;\n                        }\n                        break;\n                    }\n                }\n                log.push(col(&l_gb));\n                log.push(col(&l_it));\n                log.push(col(&l_wx));\n                log.push(col(&l_wy));\n                log.push(col(&l_fy));\n                res\n            }"]
