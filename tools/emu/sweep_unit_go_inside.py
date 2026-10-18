#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_go_inside.py — `Unit::go_inside@0061a2e0` under unicorn (item 3239, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_unit_go_inside.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 256-byte object of random bytes (no bias), 3 stack argument(s);
`p` windows in order: p0..p2 sparse table 0 (3 rows × 1 slice(s), row-major), p3..p27 the place(s), in flag order; the global cell 0x00c0618c points at 96 random bytes; the global cell 0x00c06210 points at 688 random bytes; the global cell 0x00e8d444 points at 5088 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; 32 random bytes at 0x00add000; vtable slot +0xe8 (0 args) stubbed; vtable slot +0xc0 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xa8 (0 args) stubbed; callee 0x005f96b0 (4 args) stubbed; callee 0x00647e90 (2 args) stubbed; callee 0x0070f8f0 (3 args) stubbed; callee 0x00401010 (4 args) stubbed; callee 0x00401020 (3 args) stubbed; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x005f8d20 (4 args) stubbed; callee 0x00605400 (3 args) stubbed; callee 0x00616f40 (3 args) stubbed; the word at `c0+0x14` points at 24 random bytes less `0` (placed); the word at `p3+0` points at 32 random bytes less `0` (placed); the word at `p3+4` points at 32 random bytes less `0` (placed); the word at `p3+8` points at 32 random bytes less `0` (placed); the word at `p3+12` points at 32 random bytes less `0` (placed); the word at `p3+16` points at 32 random bytes less `0` (placed); the word at `p3+20` points at 32 random bytes less `0` (placed); the word at `m+0xf8` points at 24 random bytes less `0` (placed); the word at `p10+0` points at 256 random bytes less `0` (placed); the word at `p10+4` points at 256 random bytes less `0` (placed); the word at `p10+8` points at 256 random bytes less `0` (placed); the word at `p10+12` points at 256 random bytes less `0` (placed); the word at `p10+16` points at 256 random bytes less `0` (placed); the word at `p10+20` points at 256 random bytes less `0` (placed); the word at `m+0xf4` points at 12 random bytes less `0` (placed); the word at `p17+0` points at 212 random bytes less `0` (placed); the word at `p17+4` points at 212 random bytes less `0` (placed); the word at `p17+8` points at 212 random bytes less `0` (placed); the word at `m+0x18` points at 800 random bytes less `0` (placed); the word at `p11+0x18` points at 800 random bytes less `0` (placed); the word at `p12+0x18` points at 800 random bytes less `0` (placed); the word at `p13+0x18` points at 800 random bytes less `0` (placed); the word at `p14+0x18` points at 800 random bytes less `0` (placed); the word at `p15+0x18` points at 800 random bytes less `0` (placed); the word at `p16+0x18` points at 800 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; slot +0xc0 laid as 0x0046ce40 or its stub, a flag after the answers; stub 0 answered from [1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); the word at `c0+0x30` laid as the address of `p3` every row; the word at `c0+0x4c` laid as the address of `p3` every row; the word at `p11+0xf4` laid as the address of `p17` every row; the word at `p12+0xf4` laid as the address of `p17` every row; the word at `p13+0xf4` laid as the address of `p17` every row; the word at `p14+0xf4` laid as the address of `p17` every row; the word at `p15+0xf4` laid as the address of `p17` every row; the word at `p16+0xf4` laid as the address of `p17` every row; stub 3 answers `p11+j`, the data of the object its `ecx` is in `p4+j` (j < 6), linked at `+0x8e`; stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 8 values in call order, printed after the answers; stub 6 answers 8 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 12 leaves `ecx` as a callee would, a value from [0, 7, 2147483632, 268435456, 1], after the other numbers on the row; 3 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`.
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

ENTRY = 0x0061a2e0
WIN = 256
OWN = 3
BIAS = []
ARGSET = [(2, [0, 0, 0, 1])]
ARGFROM = []
CELLS = [(12607884, 96), (12608016, 688), (15258692, 5088), (13297148, 1024)]
AT = [(11390976, 32)]
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('slot', 232, 0), ('slot', 192, 0), ('slot', 24, 0), ('slot', 168, 0), ('call', 6264496, 4), ('call', 6586000, 2), ('call', 7403760, 3), ('call', 4198416, 4), ('call', 4198432, 3), ('call', 6174704, 1), ('call', 6175008, 0), ('call', 6334576, 0), ('call', 6262048, 4), ('call', 6312960, 3), ('call', 6385472, 3)]
PLACES = [('c0', '0x14', '0', 24), ('p3', '0', '0', 32), ('p3', '4', '0', 32), ('p3', '8', '0', 32), ('p3', '12', '0', 32), ('p3', '16', '0', 32), ('p3', '20', '0', 32), ('m', '0xf8', '0', 24), ('p10', '0', '0', 256), ('p10', '4', '0', 256), ('p10', '8', '0', 256), ('p10', '12', '0', 256), ('p10', '16', '0', 256), ('p10', '20', '0', 256), ('m', '0xf4', '0', 12), ('p17', '0', '0', 212), ('p17', '4', '0', 212), ('p17', '8', '0', 212), ('m', '0x18', '0', 800), ('p11', '0x18', '0', 800), ('p12', '0x18', '0', 800), ('p13', '0x18', '0', 800), ('p14', '0x18', '0', 800), ('p15', '0x18', '0', 800), ('p16', '0x18', '0', 800)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 108, ('set', [0, 65536, 65537, 131072])), ('p11', 108, ('set', [0, 65536, 65537, 131072])), ('p12', 108, ('set', [0, 65536, 65537, 131072])), ('p13', 108, ('set', [0, 65536, 65537, 131072])), ('p14', 108, ('set', [0, 65536, 65537, 131072])), ('p15', 108, ('set', [0, 65536, 65537, 131072])), ('p16', 108, ('set', [0, 65536, 65537, 131072])), ('m', 140, ('set', [0, 65536, 131072, 196608, 262144, 327680, 2147483648, 2147549184, 4294901760])), ('m', 144, ('set', [65535, 65535, 0, 1, 2, 3, 4, 5, 65541, 131071])), ('p11', 144, ('set', [65535, 65535, 1, 2, 3, 4, 5, 131071])), ('p12', 144, ('set', [65535, 65535, 2, 3, 4, 5, 131071])), ('p13', 144, ('set', [65535, 65535, 3, 4, 5, 131071])), ('p14', 144, ('set', [65535, 65535, 4, 5, 131071])), ('p15', 144, ('set', [65535, 65535, 5, 131071])), ('p16', 144, ('set', [65535, 131071])), ('m', 8, ('set', [0, 1, 256, 257, 513, 512, 65537, 131328, 4294836737, 2147418113, 196610, 65536])), ('p11', 8, ('set', [0, 1, 256, 257, 513, 512, 65537, 131328, 4294836737, 2147418113, 196610, 65536])), ('p12', 8, ('set', [0, 1, 256, 257, 513, 512, 65537, 131328, 4294836737, 2147418113, 196610, 65536])), ('p13', 8, ('set', [0, 1, 256, 257, 513, 512, 65537, 131328, 4294836737, 2147418113, 196610, 65536])), ('p14', 8, ('set', [0, 1, 256, 257, 513, 512, 65537, 131328, 4294836737, 2147418113, 196610, 65536])), ('p15', 8, ('set', [0, 1, 256, 257, 513, 512, 65537, 131328, 4294836737, 2147418113, 196610, 65536])), ('p16', 8, ('set', [0, 1, 256, 257, 513, 512, 65537, 131328, 4294836737, 2147418113, 196610, 65536])), ('m', 16, ('set', [407095, 407154, 407176, 407542, 408638, 405303, 395734, 407285, 406118, 394294, 402999, 406843])), ('m', 20, ('set', [407095, 407154, 407176, 407542, 408638, 405303, 395734, 407285, 406118, 394294, 402999, 406843])), ('p11', 16, ('set', [407095, 407154, 407176, 407542, 408638, 405303, 395734, 407285, 406118, 394294, 402999, 406843])), ('p12', 16, ('set', [407095, 407154, 407176, 407542, 408638, 405303, 395734, 407285, 406118, 394294, 402999, 406843])), ('p13', 16, ('set', [407095, 407154, 407176, 407542, 408638, 405303, 395734, 407285, 406118, 394294, 402999, 406843])), ('p14', 16, ('set', [407095, 407154, 407176, 407542, 408638, 405303, 395734, 407285, 406118, 394294, 402999, 406843])), ('p15', 16, ('set', [407095, 407154, 407176, 407542, 408638, 405303, 395734, 407285, 406118, 394294, 402999, 406843])), ('p16', 16, ('set', [407095, 407154, 407176, 407542, 408638, 405303, 395734, 407285, 406118, 394294, 402999, 406843])), ('p11', 20, ('set', [407095, 407154, 407176, 407542, 408638, 405303, 395734, 407285, 406118, 394294, 402999, 406843])), ('p12', 20, ('set', [407095, 407154, 407176, 407542, 408638, 405303, 395734, 407285, 406118, 394294, 402999, 406843])), ('p13', 20, ('set', [407095, 407154, 407176, 407542, 408638, 405303, 395734, 407285, 406118, 394294, 402999, 406843])), ('p14', 20, ('set', [407095, 407154, 407176, 407542, 408638, 405303, 395734, 407285, 406118, 394294, 402999, 406843])), ('p15', 20, ('set', [407095, 407154, 407176, 407542, 408638, 405303, 395734, 407285, 406118, 394294, 402999, 406843])), ('p16', 20, ('set', [407095, 407154, 407176, 407542, 408638, 405303, 395734, 407285, 406118, 394294, 402999, 406843])), ('m', 180, ('set', [65280, 0, 256, 512, 768, 768])), ('p11', 180, ('set', [65280, 0, 256, 512, 768, 768])), ('p12', 180, ('set', [65280, 0, 256, 512, 768, 768])), ('p13', 180, ('set', [65280, 0, 256, 512, 768, 768])), ('p14', 180, ('set', [65280, 0, 256, 512, 768, 768])), ('p15', 180, ('set', [65280, 0, 256, 512, 768, 768])), ('p16', 180, ('set', [65280, 0, 256, 512, 768, 768])), ('p21', 4, ('set', [52, 53, 52, 53, 54, 51, 0, 16])), ('p21', 536, ('set', [0, 1, 2, 2, 2, 3])), ('p21', 692, ('set', [0, 32, 33, 1, 255, 32, 0])), ('p21', 776, ('set', [4294967295, 0, 1, 2, 3, 2, 2])), ('p22', 4, ('set', [52, 53, 52, 53, 54, 51, 0, 16])), ('p22', 536, ('set', [0, 1, 2, 2, 2, 3])), ('p22', 692, ('set', [0, 32, 33, 1, 255, 32, 0])), ('p22', 776, ('set', [4294967295, 0, 1, 2, 3, 2, 2])), ('p23', 4, ('set', [52, 53, 52, 53, 54, 51, 0, 16])), ('p23', 536, ('set', [0, 1, 2, 2, 2, 3])), ('p23', 692, ('set', [0, 32, 33, 1, 255, 32, 0])), ('p23', 776, ('set', [4294967295, 0, 1, 2, 3, 2, 2])), ('p24', 4, ('set', [52, 53, 52, 53, 54, 51, 0, 16])), ('p24', 536, ('set', [0, 1, 2, 2, 2, 3])), ('p24', 692, ('set', [0, 32, 33, 1, 255, 32, 0])), ('p24', 776, ('set', [4294967295, 0, 1, 2, 3, 2, 2])), ('p25', 4, ('set', [52, 53, 52, 53, 54, 51, 0, 16])), ('p25', 536, ('set', [0, 1, 2, 2, 2, 3])), ('p25', 692, ('set', [0, 32, 33, 1, 255, 32, 0])), ('p25', 776, ('set', [4294967295, 0, 1, 2, 3, 2, 2])), ('p26', 4, ('set', [52, 53, 52, 53, 54, 51, 0, 16])), ('p26', 536, ('set', [0, 1, 2, 2, 2, 3])), ('p26', 692, ('set', [0, 32, 33, 1, 255, 32, 0])), ('p26', 776, ('set', [4294967295, 0, 1, 2, 3, 2, 2])), ('p27', 4, ('set', [52, 53, 52, 53, 54, 51, 0, 16])), ('p27', 536, ('set', [0, 1, 2, 2, 2, 3])), ('p27', 692, ('set', [0, 32, 33, 1, 255, 32, 0])), ('p27', 776, ('set', [4294967295, 0, 1, 2, 3, 2, 2])), ('c1', 672, ('set', [0, 1])), ('c2', 0, ('set', [11390976])), ('c2', 2544, ('set', [11390976])), ('c2', 12, ('set', [0, 0, 1])), ('c2', 2556, ('set', [0, 0, 1])), ('c4', 16, ('set', [4198416])), ('c4', 24, ('set', [4198432]))]
FILLS = []
ARGRANGE = [(0, 0, 5), (1, 0, 2)]
SLOTBASE = [(232, 4640432), (192, 4640320)]
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [1]), (1, [0, 1]), (2, [0, 1]), (6, [0, 1])]
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = [('c0', 48, 'p3', 0, True), ('c0', 76, 'p3', 0, True), ('p11', 244, 'p17', 0, True), ('p12', 244, 'p17', 0, True), ('p13', 244, 'p17', 0, True), ('p14', 244, 'p17', 0, True), ('p15', 244, 'p17', 0, True), ('p16', 244, 'p17', 0, True)]
ANSWERBY = [(3, 4, 11, 6, 142)]
ANSWERWIN = []
CDECL = False
TIB = False
ARGEQ = []
SEQ = [(1, 8, None), (2, 8, None), (6, 8, None)]
ANSRANGE = []
CELLTHIS = []
VTALSO = [('p4', 0), ('p5', 0), ('p6', 0), ('p7', 0), ('p8', 0), ('p9', 0), ('p11', 0), ('p12', 0), ('p13', 0), ('p14', 0), ('p15', 0), ('p16', 0)]
SPARSE = [('@', '0xe3a390', 28396, 3, [(0, 4)])]
GTAB = []
RECORD = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
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
ECXANSWER = []
ECXOUT = [(12, [0, 7, 2147483632, 268435456, 1])]
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
    for i in range(800):
        h.brk = base  # the bump heap is reused a row; 800 rows of windows outgrow it
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

# sweep_gen: ["--item", "3239", "obj", "Unit::go_inside@0061a2e0", "--win", "0x100", "--own", "3", "--ret", "none", "--rows", "800", "--cell-win", "0xc0618c:0x60", "--cell-win", "0xc06210:0x2b0", "--cell-win", "0xe8d444:0x13e0", "--cell-win", "0xcae5fc:0x400", "--at", "0xadd000:0x20", "--sparse", "@0xe3a390:0x6eec:3:0,4", "--ptrtab", "c0+0x14:6:0x20:vt", "--ptrtab", "m+0xf8:6:0x100:vt", "--ptrtab", "m+0xf4:3:0xd4", "--stub", "slot:0xe8:0", "--stub", "slot:0xc0:0", "--stub", "slot:0x18:0", "--stub", "slot:0xa8:0", "--stub", "0x5f96b0:4", "--stub", "0x647e90:2", "--stub", "0x70f8f0:3", "--stub", "0x401010:4", "--stub", "0x401020:3", "--stub", "0x5e37f0:1", "--stub", "0x5e3920:0", "--stub", "0x60a870:0", "--stub", "0x5f8d20:4", "--stub", "0x605400:3", "--stub", "0x616f40:3", "--vtat-keep", "0xe8:0x46ceb0", "--vtat-keep", "0xc0:0x46ce40", "--alias", "c0+0x30:p3!", "--alias", "c0+0x4c:p3!", "--place", "m+0x18:0:0x320", "--place", "p11+0x18:0:0x320", "--alias", "p11+0xf4:p17!", "--place", "p12+0x18:0:0x320", "--alias", "p12+0xf4:p17!", "--place", "p13+0x18:0:0x320", "--alias", "p13+0xf4:p17!", "--place", "p14+0x18:0:0x320", "--alias", "p14+0xf4:p17!", "--place", "p15+0x18:0:0x320", "--alias", "p15+0xf4:p17!", "--place", "p16+0x18:0:0x320", "--alias", "p16+0xf4:p17!", "--answer-by", "3:4:11:6:0x8e", "--answers", "0:1", "--answers", "1:0,1", "--answers", "2:0,1", "--answers", "6:0,1", "--argrange", "0:0:5", "--argrange", "1:0:2", "--argset", "2:0,0,0,1", "--record", "0", "--record", "1", "--record", "2", "--record", "3", "--record", "4", "--record", "5", "--record", "6", "--record", "7", "--record", "8", "--record", "9", "--record", "10", "--record", "11", "--record", "12", "--record", "13", "--record", "14", "--record-order", "--wbias", "m+0x6c:0,0x10000,0x10001,0x20000", "--wbias", "p11..16+0x6c:0,0x10000,0x10001,0x20000", "--wbias", "m+0x8c:0,0x10000,0x20000,0x30000,0x40000,0x50000,0x80000000,0x80010000,0xffff0000", "--wbias", "m+0x90:0xffff,0xffff,0,1,2,3,4,5,0x10005,0x1ffff", "--wbias", "p11+0x90:0xffff,0xffff,1,2,3,4,5,0x1ffff", "--wbias", "p12+0x90:0xffff,0xffff,2,3,4,5,0x1ffff", "--wbias", "p13+0x90:0xffff,0xffff,3,4,5,0x1ffff", "--wbias", "p14+0x90:0xffff,0xffff,4,5,0x1ffff", "--wbias", "p15+0x90:0xffff,0xffff,5,0x1ffff", "--wbias", "p16+0x90:0xffff,0x1ffff", "--seq", "1:8", "--seq", "2:8", "--seq", "6:8", "--ecx-out", "12:0,7,0x7ffffff0,0x10000000,1", "--wbias", "m+8:0,1,0x100,0x101,0x201,0x200,0x10001,0x20100,0xfffe0201,0x7fff0001,0x30002,0x10000", "--wbias", "p11..16+8:0,1,0x100,0x101,0x201,0x200,0x10001,0x20100,0xfffe0201,0x7fff0001,0x30002,0x10000", "--wbias", "m+0x10:407095,407154,407176,407542,408638,405303,395734,407285,406118,394294,402999,406843", "--wbias", "m+0x14:407095,407154,407176,407542,408638,405303,395734,407285,406118,394294,402999,406843", "--wbias", "p11..16+0x10:407095,407154,407176,407542,408638,405303,395734,407285,406118,394294,402999,406843", "--wbias", "p11..16+0x14:407095,407154,407176,407542,408638,405303,395734,407285,406118,394294,402999,406843", "--wbias", "m+0xb4:0xff00,0,0x100,0x200,0x300,0x300", "--wbias", "p11..16+0xb4:0xff00,0,0x100,0x200,0x300,0x300", "--wbias", "p21+4:0x34,0x35,0x34,0x35,0x36,0x33,0,0x10", "--wbias", "p21+0x218:0,1,2,2,2,3", "--wbias", "p21+0x2b4:0,0x20,0x21,1,0xff,0x20,0", "--wbias", "p21+0x308:0xffffffff,0,1,2,3,2,2", "--wbias", "p22+4:0x34,0x35,0x34,0x35,0x36,0x33,0,0x10", "--wbias", "p22+0x218:0,1,2,2,2,3", "--wbias", "p22+0x2b4:0,0x20,0x21,1,0xff,0x20,0", "--wbias", "p22+0x308:0xffffffff,0,1,2,3,2,2", "--wbias", "p23+4:0x34,0x35,0x34,0x35,0x36,0x33,0,0x10", "--wbias", "p23+0x218:0,1,2,2,2,3", "--wbias", "p23+0x2b4:0,0x20,0x21,1,0xff,0x20,0", "--wbias", "p23+0x308:0xffffffff,0,1,2,3,2,2", "--wbias", "p24+4:0x34,0x35,0x34,0x35,0x36,0x33,0,0x10", "--wbias", "p24+0x218:0,1,2,2,2,3", "--wbias", "p24+0x2b4:0,0x20,0x21,1,0xff,0x20,0", "--wbias", "p24+0x308:0xffffffff,0,1,2,3,2,2", "--wbias", "p25+4:0x34,0x35,0x34,0x35,0x36,0x33,0,0x10", "--wbias", "p25+0x218:0,1,2,2,2,3", "--wbias", "p25+0x2b4:0,0x20,0x21,1,0xff,0x20,0", "--wbias", "p25+0x308:0xffffffff,0,1,2,3,2,2", "--wbias", "p26+4:0x34,0x35,0x34,0x35,0x36,0x33,0,0x10", "--wbias", "p26+0x218:0,1,2,2,2,3", "--wbias", "p26+0x2b4:0,0x20,0x21,1,0xff,0x20,0", "--wbias", "p26+0x308:0xffffffff,0,1,2,3,2,2", "--wbias", "p27+4:0x34,0x35,0x34,0x35,0x36,0x33,0,0x10", "--wbias", "p27+0x218:0,1,2,2,2,3", "--wbias", "p27+0x2b4:0,0x20,0x21,1,0xff,0x20,0", "--wbias", "p27+0x308:0xffffffff,0,1,2,3,2,2", "--wbias", "c1+0x2a0:0,1", "--wbias", "c2+0:0xadd000", "--wbias", "c2+0x9f0:0xadd000", "--wbias", "c2+0xc:0,0,1", "--wbias", "c2+0x9fc:0,0,1", "--wbias", "c4+0x10:0x401010", "--wbias", "c4+0x18:0x401020", "--port", "{\n            // `Unit::go_inside(this, p1, p2, p3)`: byte-window port (the crate's `Sim::go_inside`, garrison.rs,\n            // is a whole-Sim mutator and cannot take these windows, so the original's body is spelled out).\n            // Windows: `m` this; `p[11 + j]` the data record `objects[owner][j]` answers to slot +0xa8 (so\n            // `this` is `cur` = 0 for `m`, `1 + j` for it); `p[4 + j]` the object records themselves, shared by\n            // every owner's table; `p[21]` / `p[22 + j]` the type records (+0x18); `p[18..21]` the shared\n            // `+0xf4` array's records; `p[0..3]` the leaders rows; `c[1]` the console (`+0x2a0` play), `c[2]` the\n            // groups (0x9f0 apart), `c[3]` the div table. Stubs: s0 is_captain, s1 is_plane, s2 the object's\n            // slot +0x18, s3 slot +0xa8, s4 update_ceo_position, s5 insert_inside, s6 GroupData::member, s7/s8\n            // the group's slots +0x10 / +0x18, s9 close_orders, s10 clear_partial_path, s11 update_action, s12\n            // set_new_location, s13 set_angle, s14 set_anim. `a[3 + i]` is s(i)'s answer; s1, s2, s6 answer\n            // `a[18..26]`, `a[26..34]`, `a[34..42]` in call order; `a[42]` / `a[43]` lay the image's is_captain /\n            // is_plane at slots +0xe8 / +0xc0; `a[44]` is the `ecx` s12 leaves.\n            fn sh(w: &[u8], o: usize) -> i32 {\n                i32::from(i16::from_le_bytes([w[o], w[o + 1]]))\n            }\n            fn win<'w>(m: &'w mut [u8], p: &'w mut [Vec<u8>], cur: usize) -> &'w mut [u8] {\n                if cur == 0 { m } else { &mut p[10 + cur] }\n            }\n            fn tok(cur: usize) -> String {\n                if cur == 0 { \"m+0x0\".to_string() } else { format!(\"p{}+0x0\", 10 + cur) }\n            }\n            #[allow(clippy::too_many_arguments)]\n            fn go(\n                a: &[i32],\n                m: &mut [u8],\n                c: &mut [Vec<u8>],\n                p: &mut [Vec<u8>],\n                log: &mut Vec<String>,\n                n: &mut [usize; 3],\n                mut cur: usize,\n                p1: i32,\n                p2: i32,\n                p3: i32,\n            ) {\n                let (keep_e8, keep_c0) = (a[42] != 0, a[43] != 0);\n                loop {\n                    let w = win(m, p, cur);\n                    if rd(w, 0x6c) & 0x10000 != 0 {\n                        let x = rd(&c[3], 4 * ((rd(w, 0x10) ^ 0x63637) >> 6) as usize);\n                        let y = rd(&c[3], 4 * ((rd(w, 0x14) ^ 0x63637) >> 6) as usize);\n                        log.push(format!(\"s4:{},{x},{y},-1,-1\", tok(cur)));\n                    }\n                    if p3 != 0 {\n                        break;\n                    }\n                    let w = win(m, p, cur);\n                    let captain = if keep_e8 {\n                        i32::from(u16::from_le_bytes([w[0x8e], w[0x8f]]) >> 15)\n                    } else {\n                        log.push(format!(\"s0:{}\", tok(cur)));\n                        a[3]\n                    };\n                    if captain != 0 {\n                        break;\n                    }\n                    let idx = usize::try_from(sh(w, 0x8e)).expect(\"captain index\");\n                    log.push(format!(\"s3:p{}+0x0\", 4 + idx));\n                    cur = 1 + idx;\n                }\n                let tw = if cur == 0 { 21 } else { 21 + cur };\n                let (t4, t218, t2b4, t308) = (rd(&p[tw], 4), rd(&p[tw], 0x218), p[tw][0x2b4], rd(&p[tw], 0x308));\n                log.push(format!(\"s5:{},{p1},{p2}\", tok(cur)));\n                let skip = if keep_c0 {\n                    t218 == 2 && t2b4 & 0x20 == 0\n                } else {\n                    log.push(format!(\"s1:{}\", tok(cur)));\n                    let v = a[18 + n[0].min(7)];\n                    n[0] += 1;\n                    v != 0\n                };\n                if !skip {\n                    let w = win(m, p, cur);\n                    let (short, owner) = (sh(w, 0xa), i32::from(w[9]));\n                    let play = usize::try_from(rd(&c[1], 0x2a0)).expect(\"play\");\n                    let g = format!(\"c2+{:#x}\", play * 0x9f0);\n                    log.push(format!(\"s6:{g},{short},{owner},1\"));\n                    let ans = a[34 + n[2].min(7)];\n                    n[2] += 1;\n                    if ans != 0 {\n                        log.push(format!(\"s7:{g},{short},{owner},0,0\"));\n                        if rd(&c[2], play * 0x9f0 + 0xc) == 0 {\n                            log.push(format!(\"s8:{g},{p1},{p2},0\"));\n                        }\n                    }\n                }\n                let bj = usize::try_from(p1).expect(\"building index\");\n                log.push(format!(\"s2:p{}+0x0\", 4 + bj));\n                let ans = a[26 + n[1].min(7)];\n                n[1] += 1;\n                if ans != 0 && t308 > 1 {\n                    let w = win(m, p, cur);\n                    let f = rd(w, 0x68) & 0xfbff_ffffu32 as i32;\n                    wr(w, 0x68, f);\n                    wr(w, 0xc0, 0);\n                    log.push(format!(\"s9:{},0\", tok(cur)));\n                    log.push(format!(\"s10:{}\", tok(cur)));\n                    log.push(format!(\"s11:{}\", tok(cur)));\n                }\n                let i90 = sh(win(m, p, cur), 0x90);\n                if i90 >= 0 {\n                    let k = usize::try_from(i90).expect(\"index\");\n                    log.push(format!(\"s3:p{}+0x0\", 4 + k));\n                    if p[11 + k][8] & 1 != 0 {\n                        log.push(format!(\"s3:p{}+0x0\", 4 + k));\n                        go(a, m, c, p, log, n, 1 + k, p1, p2, 1);\n                    }\n                }\n                if t4 == 0x34 || t4 == 0x35 {\n                    let owner = usize::from(win(m, p, cur)[9]);\n                    let v = rd(&p[owner], 0) | 0x0200_0000;\n                    wr(&mut p[owner], 0, v);\n                    let (x, y) = (rd(&p[4 + bj], 0x10) ^ 0x63637, rd(&p[4 + bj], 0x14) ^ 0x63637);\n                    log.push(format!(\"s12:{},{x},{y},1,1\", tok(cur)));\n                    log.push(format!(\"s13:{},0,{},1\", tok(cur), a[44]));\n                    log.push(format!(\"s14:{},0,1,1\", tok(cur)));\n                }\n                let cnt = i32::from(win(m, p, cur)[0xb5] as i8);\n                for i in 0..cnt.max(0) as usize {\n                    wr(&mut p[18 + i], 0xd0, 0);\n                }\n            }\n            let mut n = [0usize; 3];\n            go(a, m, c, p, &mut log, &mut n, 0, a[0], a[1], a[2]);\n            if log.is_empty() {\n                log.push(\"-\".to_string());\n            }\n            0\n        }"]
