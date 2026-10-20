#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_cities_capture_city.py — `Cities::capture_city@00733380` under unicorn (item 3362, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_cities_capture_city.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 512-byte object of random bytes (no bias), 3 stack argument(s);
`p` windows in order: p0..p15 sparse table 0 (4 rows × 4 slice(s), row-major), p16..p42 the place(s), in flag order; the global cell 0x00c0618c points at 256 random bytes; the global cell 0x00c061ec points at 128 random bytes; the global cell 0x00c061f0 points at 784 random bytes; 112 random bytes at 0x00c09970; 112 random bytes at 0x00c0ab84; callee 0x006d15e0 (1 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x006eb930 (4 args) stubbed; callee 0x00597800 (0 args) stubbed; callee 0x006db790 (1 args) stubbed; callee 0x007af660 (3 args) stubbed; callee 0x00739340 (0 args) stubbed; callee 0x00736c40 (4 args) stubbed; callee 0x006f2d70 (5 args) stubbed; callee 0x007384c0 (0 args) stubbed; callee 0x004220e0 (1 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x94 (1 args) stubbed; vtable slot +0x1a8 (3 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `c3+0` points at 16 random bytes less `0` (placed); the word at `p16+0` points at 152 random bytes less `0` (placed); the word at `p16+4` points at 152 random bytes less `0` (placed); the word at `p16+8` points at 152 random bytes less `0` (placed); the word at `p16+12` points at 152 random bytes less `0` (placed); the word at `c0+0x14` points at 32 random bytes less `0` (placed); the word at `p21+0` points at 32 random bytes less `0` (placed); the word at `p21+4` points at 32 random bytes less `0` (placed); the word at `p21+8` points at 32 random bytes less `0` (placed); the word at `p21+12` points at 32 random bytes less `0` (placed); the word at `p21+16` points at 32 random bytes less `0` (placed); the word at `p21+20` points at 32 random bytes less `0` (placed); the word at `p21+24` points at 32 random bytes less `0` (placed); the word at `p21+28` points at 32 random bytes less `0` (placed); the word at `m+0x100` points at 32 random bytes less `0` (placed); the word at `p30+0` points at 136 random bytes less `0` (placed); the word at `p30+4` points at 136 random bytes less `0` (placed); the word at `p30+8` points at 136 random bytes less `0` (placed); the word at `p30+12` points at 136 random bytes less `0` (placed); the word at `p30+16` points at 136 random bytes less `0` (placed); the word at `p30+20` points at 136 random bytes less `0` (placed); the word at `p30+24` points at 136 random bytes less `0` (placed); the word at `p30+28` points at 136 random bytes less `0` (placed); the word at `p31+0x18` points at 720 random bytes less `0` (placed); the word at `p32+0x18` points at 720 random bytes less `0` (placed); the word at `p33+0x18` points at 720 random bytes less `0` (placed); the word at `p34+0x18` points at 720 random bytes less `0` (placed); stub 1 answered from [0, 1] (90%); stub 15 answered from [0, 1] (90%); stub 17 answered from [0, 1] (90%); stub 16 answered from [0, 1] (90%); the word at `p31+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p32+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p33+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p34+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p35+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p36+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p37+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p38+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `c3+0x1c` laid as the address of `p16` every row; the word at `c3+0x38` laid as the address of `p16` every row; the word at `c3+0x54` laid as the address of `p16` every row; the word at `c0+0x30` laid as the address of `p21` every row; the word at `c0+0x4c` laid as the address of `p21` every row; the word at `c0+0x68` laid as the address of `p21` every row; the word at `c4+0x0` laid as the address of `p21` every row; the word at `c4+0x1c` laid as the address of `p21` every row; the word at `c4+0x38` laid as the address of `p21` every row; the word at `c4+0x54` laid as the address of `p21` every row; the word at `p35+0x18` laid as the address of `p39` every row; the word at `p36+0x18` laid as the address of `p40` every row; the word at `p37+0x18` laid as the address of `p41` every row; the word at `p38+0x18` laid as the address of `p42` every row; the word at `p22+0x18` laid as the address of `p39` every row; the word at `p23+0x18` laid as the address of `p40` every row; the word at `p24+0x18` laid as the address of `p41` every row; the word at `p25+0x18` laid as the address of `p42` every row; the word at `p26+0x18` laid as the address of `p39` every row; the word at `p27+0x18` laid as the address of `p40` every row; the word at `p28+0x18` laid as the address of `p41` every row; the word at `p29+0x18` laid as the address of `p42` every row; stub 11 answers `p31+j`, the data of the object its `ecx` is in `p22+j` (j < 8), linked at `+0x74`; stub 12 answers `p31+j`, the data of the object its `ecx` is in `p22+j` (j < 8), linked at `+0x74`; stub 1 answers 8 values in call order, printed after the answers; stub 15 answers 8 values in call order, printed after the answers; stub 17 answers 8 values in call order, printed after the answers; stub 16 answers 4 values in call order, printed after the answers; stub 6 answers 0..6 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the call ends where stub 9 is first entered (the function is swept to that callee); stub 10 writes 2146500608 (4 bytes) at `ecx+0x10` each call; stub 10 writes 256 (4 bytes) at `ecx+0x8` each call; stub 13 answers from its own `ecx`: the word at `+0x80`; stub 7 leaves `ecx` as a callee would, a value from [0, 1, 127], after the other numbers on the row; 4 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x824:0x8, +0x692c:0x4, +0x6988:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p40+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`.
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

ENTRY = 0x00733380
WIN = 512
OWN = 3
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607884, 256), (12607980, 128), (12607984, 784)]
AT = [(12622192, 112), (12626820, 112)]
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('call', 7149024, 1), ('call', 7213936, 1), ('call', 7256368, 4), ('call', 5863424, 0), ('call', 7190416, 1), ('call', 8058464, 3), ('call', 7574336, 0), ('call', 7564352, 4), ('call', 7286128, 5), ('call', 7570624, 0), ('call', 4333792, 1), ('slot', 176, 0), ('slot', 172, 0), ('slot', 148, 1), ('slot', 424, 3), ('slot', 184, 2), ('slot', 76, 0), ('slot', 96, 2)]
PLACES = [('c3', '0', '0', 16), ('p16', '0', '0', 152), ('p16', '4', '0', 152), ('p16', '8', '0', 152), ('p16', '12', '0', 152), ('c0', '0x14', '0', 32), ('p21', '0', '0', 32), ('p21', '4', '0', 32), ('p21', '8', '0', 32), ('p21', '12', '0', 32), ('p21', '16', '0', 32), ('p21', '20', '0', 32), ('p21', '24', '0', 32), ('p21', '28', '0', 32), ('m', '0x100', '0', 32), ('p30', '0', '0', 136), ('p30', '4', '0', 136), ('p30', '8', '0', 136), ('p30', '12', '0', 136), ('p30', '16', '0', 136), ('p30', '20', '0', 136), ('p30', '24', '0', 136), ('p30', '28', '0', 136), ('p31', '0x18', '0', 720), ('p32', '0x18', '0', 720), ('p33', '0x18', '0', 720), ('p34', '0x18', '0', 720)]
NULLS = []
NULLFORCE = {}
WBIAS = [('c1', 116, ('set', [83886080, 100663296, 2113929216])), ('p2', 0, ('set', [0, 1, 1, 2])), ('p3', 0, ('set', [0, 1, 1, 2])), ('p6', 0, ('set', [0, 1, 1, 2])), ('p7', 0, ('set', [0, 1, 1, 2])), ('p10', 0, ('set', [0, 1, 1, 2])), ('p11', 0, ('set', [0, 1, 1, 2])), ('p14', 0, ('set', [0, 1, 1, 2])), ('p15', 0, ('set', [0, 1, 1, 2])), ('p17', 8, ('set', [0])), ('p18', 8, ('set', [0])), ('p17', 92, ('set', [0, 65536, 131072, 196608])), ('p18', 92, ('set', [0, 65536, 131072, 196608])), ('p31', 128, ('set', [4, 5, 6, 7])), ('p32', 128, ('set', [-1, 4, 5, 6, 7])), ('p33', 128, ('set', [-1, 4, 5, 6, 7])), ('p34', 128, ('set', [-1, 4, 5, 6, 7])), ('p35', 128, ('set', [-1, 4, 5, 6, 7])), ('p36', 128, ('set', [-1, 4, 5, 6, 7])), ('p37', 128, ('set', [-1, 4, 5, 6, 7])), ('p38', 128, ('set', [-1, 4, 5, 6, 7])), ('p31', 112, ('set', [131072, 196608])), ('p32', 112, ('set', [131072, 196608])), ('p33', 112, ('set', [131072, 196608])), ('p34', 112, ('set', [131072, 196608])), ('p35', 112, ('set', [131072, 196608])), ('p36', 112, ('set', [131072, 196608])), ('p37', 112, ('set', [131072, 196608])), ('p38', 112, ('set', [131072, 196608])), ('p39', 704, ('set', [0, 8192, 16, 8208, 64, 8256])), ('p40', 704, ('set', [0, 8192, 16, 8208, 64, 8256])), ('p41', 704, ('set', [0, 8192, 16, 8208, 64, 8256])), ('p42', 704, ('set', [0, 8192, 16, 8208, 64, 8256]))]
FILLS = []
ARGRANGE = [(0, 0, 1), (1, 0, 1), (2, 2, 3)]
SLOTBASE = []
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(1, [0, 1]), (15, [0, 1]), (17, [0, 1]), (16, [0, 1])]
VTAT = None
VTATHALF = []
VTIMAGE = [('p31', 0, 11805044), ('p32', 0, 11805044), ('p33', 0, 11805044), ('p34', 0, 11805044), ('p35', 0, 11805044), ('p36', 0, 11805044), ('p37', 0, 11805044), ('p38', 0, 11805044)]
ALIAS = [('c3', 28, 'p16', 0, True), ('c3', 56, 'p16', 0, True), ('c3', 84, 'p16', 0, True), ('c0', 48, 'p21', 0, True), ('c0', 76, 'p21', 0, True), ('c0', 104, 'p21', 0, True), ('c4', 0, 'p21', 0, True), ('c4', 28, 'p21', 0, True), ('c4', 56, 'p21', 0, True), ('c4', 84, 'p21', 0, True), ('p35', 24, 'p39', 0, True), ('p36', 24, 'p40', 0, True), ('p37', 24, 'p41', 0, True), ('p38', 24, 'p42', 0, True), ('p22', 24, 'p39', 0, True), ('p23', 24, 'p40', 0, True), ('p24', 24, 'p41', 0, True), ('p25', 24, 'p42', 0, True), ('p26', 24, 'p39', 0, True), ('p27', 24, 'p40', 0, True), ('p28', 24, 'p41', 0, True), ('p29', 24, 'p42', 0, True)]
ANSWERBY = [(11, 22, 31, 8, 116), (12, 22, 31, 8, 116)]
ANSWERWIN = []
CDECL = False
TIB = True
ARGEQ = []
SEQ = [(1, 8, None), (15, 8, None), (17, 8, None), (16, 4, None)]
ANSRANGE = [(6, 0, 6)]
CELLTHIS = []
VTALSO = [('p39', 0), ('p40', 0), ('p41', 0), ('p42', 0), ('p22', 0), ('p23', 0), ('p24', 0), ('p25', 0), ('p26', 0), ('p27', 0), ('p28', 0), ('p29', 0), ('p31', 0), ('p32', 0), ('p33', 0), ('p34', 0), ('p35', 0), ('p36', 0), ('p37', 0), ('p38', 0)]
SPARSE = [('@', '0xe3a390', 28396, 4, [(0, 4), (2084, 8), (26924, 4), (27016, 4)])]
GTAB = []
RECORD = [0, 1, 2, 5, 6, 7, 8, 9, 11, 12, 13, 14, 15, 16, 17]
RECORDORDER = True
RECORDEDX = False
ROWSPLIT = {0: 0, 1: 0, 2: 0}
LINKS = []
LINKHI = []
OUTARG = []
CDECLSTUBS = []
STUBWRITE = [(10, 'ecx', 16, 4, 2146500608), (10, 'ecx', 8, 4, 256)]
STUBCOPY = []
STACKCLEAN = 0
STOPSTUB = 9
INITCALLS = []
MAXINSNS = 200000
ECXANSWER = [(13, 128, None)]
ECXOUT = [(7, [0, 1, 127])]
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
    rng = random.Random(3362)
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
    for i in range(1200):
        h.brk = base  # the bump heap is reused a row; 1200 rows of windows outgrow it
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

# sweep_gen: ["obj", "Cities::capture_city@00733380", "--item=3362", "--win=0x200", "--own=3", "--ret=none", "--tib", "--rows=1200", "--argrange=0:0:1", "--argrange=1:0:1", "--argrange=2:2:3", "--cell-win=0xc0618c:0x100", "--cell-win=0xc061ec:0x80", "--cell-win=0xc061f0:0x310", "--at=0xc09970:0x70", "--at=0xc0ab84:0x70", "--stub=0x6d15e0:1", "--stub=0x6e1370:1", "--stub=0x6eb930:4", "--stub=0x597800:0", "--stub=0x6db790:1", "--stub=0x7af660:3", "--stub=0x739340:0", "--stub=0x736c40:4", "--stub=0x6f2d70:5", "--stub=0x7384c0:0", "--stub=0x4220e0:1", "--stub=slot:0xb0:0", "--stub=slot:0xac:0", "--stub=slot:0x94:1", "--stub=slot:0x1a8:3", "--stub=slot:0xb8:2", "--stub=slot:0x4c:0", "--stub=slot:0x60:2", "--sparse=@0xe3a390:0x6eec:4:0,4/0x824,8/0x692c,4/0x6988,4", "--ptrtab=c3+0:4:0x98", "--ptrtab=c0+0x14:8:0x20:vt", "--ptrtab=m+0x100:8:0x88:vt", "--place=p31+0x18:0:0x2d0", "--place=p32+0x18:0:0x2d0", "--place=p33+0x18:0:0x2d0", "--place=p34+0x18:0:0x2d0", "--alias=c3+0x1c:p16!", "--alias=c3+0x38:p16!", "--alias=c3+0x54:p16!", "--alias=c0+0x30:p21!", "--alias=c0+0x4c:p21!", "--alias=c0+0x68:p21!", "--alias=c4+0x0:p21!", "--alias=c4+0x1c:p21!", "--alias=c4+0x38:p21!", "--alias=c4+0x54:p21!", "--alias=p35+0x18:p39!", "--alias=p36+0x18:p40!", "--alias=p37+0x18:p41!", "--alias=p38+0x18:p42!", "--alias=p22+0x18:p39!", "--alias=p23+0x18:p40!", "--alias=p24+0x18:p41!", "--alias=p25+0x18:p42!", "--alias=p26+0x18:p39!", "--alias=p27+0x18:p40!", "--alias=p28+0x18:p41!", "--alias=p29+0x18:p42!", "--vtat-also=p39+0", "--vtat-also=p40+0", "--vtat-also=p41+0", "--vtat-also=p42+0", "--vtat-image=p31+0:0xb42174", "--vtat-image=p32+0:0xb42174", "--vtat-image=p33+0:0xb42174", "--vtat-image=p34+0:0xb42174", "--vtat-image=p35+0:0xb42174", "--vtat-image=p36+0:0xb42174", "--vtat-image=p37+0:0xb42174", "--vtat-image=p38+0:0xb42174", "--answer-by=11:22:31:8:0x74", "--answer-by=12:22:31:8:0x74", "--ecx-answer=13:0x80", "--ecx-out=7:0,1,127", "--stop-stub=9", "--stub-write=10:ecx+0x10:4:0x7ff10000", "--stub-write=10:ecx+8:4:256", "--wbias=c1+0x74:0x05000000,0x06000000,0x7e000000", "--wbias=p2+0:0,1,1,2", "--wbias=p3+0:0,1,1,2", "--wbias=p6+0:0,1,1,2", "--wbias=p7+0:0,1,1,2", "--wbias=p10+0:0,1,1,2", "--wbias=p11+0:0,1,1,2", "--wbias=p14+0:0,1,1,2", "--wbias=p15+0:0,1,1,2", "--wbias=p17+8:0", "--wbias=p18+8:0", "--wbias=p17+0x5c:0,0x10000,0x20000,0x30000", "--wbias=p18+0x5c:0,0x10000,0x20000,0x30000", "--wbias=p31+0x80:4,5,6,7", "--wbias=p32+0x80:-1,4,5,6,7", "--wbias=p33+0x80:-1,4,5,6,7", "--wbias=p34+0x80:-1,4,5,6,7", "--wbias=p35+0x80:-1,4,5,6,7", "--wbias=p36+0x80:-1,4,5,6,7", "--wbias=p37+0x80:-1,4,5,6,7", "--wbias=p38+0x80:-1,4,5,6,7", "--wbias=p31..38+0x70:0x20000,0x30000", "--wbias=p39..42+0x2c0:0,0x2000,0x10,0x2010,0x40,0x2040", "--answers=1:0,1", "--seq=1:8", "--answers=15:0,1", "--seq=15:8", "--answers=17:0,1", "--seq=17:8", "--answers=16:0,1", "--seq=16:4", "--answer-range=6:0:6", "--record=0@0", "--record=1@0", "--record=2@0", "--record=5", "--record=6", "--record=7", "--record=8", "--record=9", "--record=11", "--record=12", "--record=13", "--record=14", "--record=15", "--record=16", "--record=17", "--record-order", "--port", "{\n                // Window layout. `m` the `Cities` (unread); `c[0]` the objects record (the per-player arrays at\n                // `+0x14 + 0x1c * who`, every row aliased to `p[21]`), `c[1]` the game (`+0x77` the local\n                // player, held off `a`), `c[2]` the constants (`+0x304` the plunder per level), `c[3]` and\n                // `c[4]` the cities and objects tables (every row aliased to `p[16]` and `p[21]`). `p[4 * r\n                // + s]` the leaders table's row `r` in four slices (`+0`, `+0x824..+0x82c`, `+0x692c`,\n                // `+0x6988`); `p[17 + k]` city record `k` (`+4` flags, `+8` its building, `+0x90` its name);\n                // `p[22 + j]` the objects (`+0x18` their type) and `p[31 + j]` the data each answers by slots\n                // `+0xac` and `+0xb0` (`+0` vtable: `Build::vftable` or the stubbed one; `+8` flags; `+0x70`\n                // its city, `+0x74` its next member, `+0x80` the swap's answer; `+0x18` its type, `p[39 +\n                // j % 4]`, `+0x2c0` the flags). `a`: the three arguments, one answer a stub (`a[3 + k]`), then\n                // the `--seq` runs in flag order (stubs 1, 15, 17, 16) and `City::capture`'s leftover `ecx`\n                // (`a[49]`).\n                //\n                // **The port is the crate's own steps**, `crate::capture_view::capture_city_steps`, which\n                // `Sim::capture_city` calls over its `SimCapture`; here the view is the emulated original's\n                // windows.\n                use crate::capture_view::{CaptureView, capture_city_steps};\n                const SEQ_AT: [(usize, usize, usize); 4] = [(1, 21, 8), (15, 29, 8), (17, 37, 8), (16, 45, 4)];\n                const BUILD_VTABLE: u32 = 0x00b4_2174;\n                struct V<'a> {\n                    a: &'a [i32],\n                    c: &'a [Vec<u8>],\n                    p: &'a mut [Vec<u8>],\n                    log: &'a mut Vec<String>,\n                    turn: [usize; 18],\n                }\n                fn rd32(w: &[u8], o: usize) -> i32 {\n                    i32::from_le_bytes(w[o..o + 4].try_into().expect(\"four bytes\"))\n                }\n                fn rd16(w: &[u8], o: usize) -> i32 {\n                    i32::from(i16::from_le_bytes([w[o], w[o + 1]]))\n                }\n                impl V<'_> {\n                    fn rec(&mut self, k: usize, ecx: &str, args: &[String]) {\n                        let mut t = format!(\"s{k}:{ecx}\");\n                        for v in args {\n                            t.push(',');\n                            t.push_str(v);\n                        }\n                        self.log.push(t);\n                    }\n                    /// Stub `k`'s next `--seq` answer, the last repeated.\n                    fn seq(&mut self, k: usize) -> i32 {\n                        let &(_, at, n) = SEQ_AT.iter().find(|s| s.0 == k).expect(\"a seq stub\");\n                        let i = self.turn[k].min(n - 1);\n                        self.turn[k] += 1;\n                        self.a[at + i]\n                    }\n                    /// Slot `+0xac` of the object `b`.\n                    fn slot_ac(&mut self, b: usize) {\n                        self.rec(12, &format!(\"p{}+0x0\", 22 + b), &[]);\n                    }\n                    fn data(&self, b: usize) -> &[u8] {\n                        &self.p[31 + b]\n                    }\n                    fn is_build(&self, b: usize) -> bool {\n                        rd32(self.data(b), 0) as u32 == BUILD_VTABLE\n                    }\n                    fn swap(&mut self, a: u8, b: usize) -> Option<usize> {\n                        self.rec(13, &format!(\"p{}+0x0\", 31 + b), &[a.to_string()]);\n                        let n = rd32(self.data(b), 0x80);\n                        usize::try_from(n).ok()\n                    }\n                    fn leader(&self, who: u8) -> String {\n                        format!(\"r{who}+0x0\")\n                    }\n                    fn word(&mut self, who: u8, slice: usize, off: usize) -> &mut [u8] {\n                        &mut self.p[4 * who as usize + slice][off..off + 4]\n                    }\n                }\n                impl CaptureView for V<'_> {\n                    type B = usize;\n                    fn diplo_agrees(&mut self, a: u8, o: u8) -> bool {\n                        rd32(&self.p[4 * o as usize + 2 + a as usize], 0) == 1\n                    }\n                    fn action_clear_all(&mut self, o: u8, a: u8) {\n                        let l = self.leader(o);\n                        self.rec(0, &l, &[a.to_string()]);\n                    }\n                    fn city_is_capital(&mut self, _o: u8, c: usize) -> bool {\n                        rd16(&self.p[17 + c], 4) & 0x10 != 0\n                    }\n                    fn has_tribe_bonus(&mut self, who: u8, id: i32) -> bool {\n                        let l = self.leader(who);\n                        self.rec(1, &l, &[id.to_string()]);\n                        self.seq(1) != 0\n                    }\n                    fn find_capital(&mut self, o: u8, c: usize) {\n                        let l = self.leader(o);\n                        self.rec(\n                            2,\n                            &l,\n                            &[\"sp-0x48\".into(), \"sp-0x3c\".into(), c.to_string(), o.to_string()],\n                        );\n                    }\n                    fn count_lost(&mut self, o: u8) {\n                        let w = self.word(o, 1, 4);\n                        let v = i32::from_le_bytes([w[0], w[1], w[2], w[3]]).wrapping_add(1);\n                        w.copy_from_slice(&v.to_le_bytes());\n                    }\n                    fn count_captured(&mut self, a: u8) {\n                        let w = self.word(a, 1, 0);\n                        let v = i32::from_le_bytes([w[0], w[1], w[2], w[3]]).wrapping_add(1);\n                        w.copy_from_slice(&v.to_le_bytes());\n                    }\n                    fn achievement_ping(&mut self, a: u8) {\n                        // The arm is held off by the game window's `+0x74`: its byte `+0x77` is never `a`.\n                        assert_ne!(self.c[1][0x77], a, \"the achievements arm is out of domain\");\n                    }\n                    fn mark_economy(&mut self, who: u8) {\n                        let w = self.word(who, 0, 0);\n                        let v = i32::from_le_bytes([w[0], w[1], w[2], w[3]]) | 0x0200_0000;\n                        w.copy_from_slice(&v.to_le_bytes());\n                    }\n                    /// Unrecorded in the sweep: its `ecx` is `leadersc + who * 0x6eec`, no window of ours.\n                    fn city_radius_probe(&mut self, _o: u8, _c: usize) {}\n                    fn city_building(&mut self, _o: u8, c: usize) -> usize {\n                        rd16(&self.p[17 + c], 8) as usize\n                    }\n                    fn swap_head(&mut self, _o: u8, b: usize, a: u8) -> Option<usize> {\n                        self.rec(11, &format!(\"p{}+0x0\", 22 + b), &[]);\n                        self.swap(a, b)\n                    }\n                    fn swap_member(&mut self, _o: u8, b: usize, a: u8) -> Option<usize> {\n                        self.slot_ac(b);\n                        self.swap(a, b)\n                    }\n                    fn activate(&mut self, _a: u8, n: usize) {\n                        self.slot_ac(n);\n                        self.rec(\n                            14,\n                            &format!(\"p{}+0x0\", 31 + n),\n                            &[\"1\".into(), \"1\".into(), \"0\".into()],\n                        );\n                    }\n                    fn city_of(&mut self, _a: u8, n: usize) -> Option<usize> {\n                        self.slot_ac(n);\n                        usize::try_from(rd16(self.data(n), 0x72)).ok()\n                    }\n                    fn level(&mut self, _a: u8, nc: usize) -> i32 {\n                        self.rec(6, &format!(\"p{}+0x0\", 17 + nc), &[]);\n                        self.a[3 + 6]\n                    }\n                    fn plunder_per_level(&mut self) -> i32 {\n                        rd32(&self.c[2], 0x304)\n                    }\n                    fn achieve(&mut self, kind: i32, who: u8, _o: u8, c: usize) {\n                        self.rec(\n                            5,\n                            &c.to_string(),\n                            &[kind.to_string(), who.to_string(), format!(\"p{}+0x90\", 17 + c)],\n                        );\n                    }\n                    fn city_capture(&mut self, nc: usize, _o: u8, c: usize, a: u8, n: usize) {\n                        self.rec(\n                            7,\n                            &format!(\"p{}+0x0\", 17 + nc),\n                            &[format!(\"p{}+0x0\", 17 + c), nc.to_string(), a.to_string(), n.to_string()],\n                        );\n                    }\n                    fn update_city(&mut self, old: usize, o: u8, n: usize, a: u8) {\n                        let stale = self.a[49].to_string();\n                        self.rec(\n                            8,\n                            &stale.clone(),\n                            &[old.to_string(), o.to_string(), n.to_string(), a.to_string(), stale],\n                        );\n                    }\n                    fn first_member(&mut self, _o: u8, b: usize) -> Option<usize> {\n                        self.slot_ac(b);\n                        usize::try_from(rd16(self.data(b), 0x74)).ok()\n                    }\n                    fn next_member(&mut self, _o: u8, m: usize) -> Option<usize> {\n                        self.slot_ac(m);\n                        usize::try_from(rd16(self.data(m), 0x74)).ok()\n                    }\n                    fn not_civilian(&mut self, _o: u8, m: usize) -> bool {\n                        self.slot_ac(m);\n                        rd32(&self.p[39 + m % 4], 0x2c0) & 0x2000 != 0\n                    }\n                    fn is_ident(&mut self, _o: u8, m: usize, id: i32) -> bool {\n                        self.slot_ac(m);\n                        if self.is_build(m) {\n                            self.rec(17, &format!(\"p{}+0x0\", 39 + m % 4), &[id.to_string(), \"0\".into()]);\n                            self.seq(17) != 0\n                        } else {\n                            self.rec(15, &format!(\"p{}+0x0\", 31 + m), &[id.to_string(), \"0\".into()]);\n                            self.seq(15) != 0\n                        }\n                    }\n                    fn is_active(&mut self, _o: u8, m: usize) -> bool {\n                        self.slot_ac(m);\n                        if self.is_build(m) {\n                            self.data(m)[8] & 4 != 0\n                        } else {\n                            self.rec(16, &format!(\"p{}+0x0\", 31 + m), &[]);\n                            self.seq(16) != 0\n                        }\n                    }\n                    fn find_buildings(&mut self, _a: u8, nc: usize) {\n                        self.rec(9, &format!(\"p{}+0x0\", 17 + nc), &[]);\n                    }\n                }\n                let mut v = V {\n                    a,\n                    c: &*c,\n                    p,\n                    log: &mut log,\n                    turn: [0; 18],\n                };\n                capture_city_steps(&mut v, a[0] as u8, a[1] as usize, a[2] as u8);\n                0\n            }"]
