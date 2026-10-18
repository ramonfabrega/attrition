#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_type_data_get_preq.py — `TypeData::get_preq@00668700` under unicorn (item 3261, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_type_data_get_preq.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 72-byte object of random bytes (no bias), 2 stack argument(s);
`p` windows in order: p0..p37 the place(s), in flag order; the global cell 0x00c061e8 points at 64 random bytes; the global cell 0x00c061e4 points at 2112 random bytes; callee 0x006e1370 (1 args) stubbed; the word at `0xe85ddc` points at 144 random bytes less `0x87c` (placed); the word at `p0+0` points at 464 random bytes less `0` (placed); the word at `p0+4` points at 464 random bytes less `0` (placed); the word at `p0+8` points at 464 random bytes less `0` (placed); the word at `p0+12` points at 464 random bytes less `0` (placed); the word at `p0+16` points at 464 random bytes less `0` (placed); the word at `p0+20` points at 464 random bytes less `0` (placed); the word at `p0+24` points at 464 random bytes less `0` (placed); the word at `p0+28` points at 464 random bytes less `0` (placed); the word at `p0+32` points at 464 random bytes less `0` (placed); the word at `p0+36` points at 464 random bytes less `0` (placed); the word at `p0+40` points at 464 random bytes less `0` (placed); the word at `p0+44` points at 464 random bytes less `0` (placed); the word at `p0+48` points at 464 random bytes less `0` (placed); the word at `p0+52` points at 464 random bytes less `0` (placed); the word at `p0+56` points at 464 random bytes less `0` (placed); the word at `p0+60` points at 464 random bytes less `0` (placed); the word at `p0+64` points at 464 random bytes less `0` (placed); the word at `p0+68` points at 464 random bytes less `0` (placed); the word at `p0+72` points at 464 random bytes less `0` (placed); the word at `p0+76` points at 464 random bytes less `0` (placed); the word at `p0+80` points at 464 random bytes less `0` (placed); the word at `p0+84` points at 464 random bytes less `0` (placed); the word at `p0+88` points at 464 random bytes less `0` (placed); the word at `p0+92` points at 464 random bytes less `0` (placed); the word at `p0+96` points at 464 random bytes less `0` (placed); the word at `p0+100` points at 464 random bytes less `0` (placed); the word at `p0+104` points at 464 random bytes less `0` (placed); the word at `p0+108` points at 464 random bytes less `0` (placed); the word at `p0+112` points at 464 random bytes less `0` (placed); the word at `p0+116` points at 464 random bytes less `0` (placed); the word at `p0+120` points at 464 random bytes less `0` (placed); the word at `p0+124` points at 464 random bytes less `0` (placed); the word at `p0+128` points at 464 random bytes less `0` (placed); the word at `p0+132` points at 464 random bytes less `0` (placed); the word at `p0+136` points at 464 random bytes less `0` (placed); the word at `p0+140` points at 464 random bytes less `0` (placed); the word at `0xc0aabc` points at 144 random bytes less `0x87c` (placed); the word at `p37+0` laid as the address of `p1` every row; the word at `p37+4` laid as the address of `p2` every row; the word at `p37+8` laid as the address of `p3` every row; the word at `p37+12` laid as the address of `p4` every row; the word at `p37+16` laid as the address of `p5` every row; the word at `p37+20` laid as the address of `p6` every row; the word at `p37+24` laid as the address of `p7` every row; the word at `p37+28` laid as the address of `p8` every row; the word at `p37+32` laid as the address of `p9` every row; the word at `p37+36` laid as the address of `p10` every row; the word at `p37+40` laid as the address of `p11` every row; the word at `p37+44` laid as the address of `p12` every row; the word at `p37+48` laid as the address of `p13` every row; the word at `p37+52` laid as the address of `p14` every row; the word at `p37+56` laid as the address of `p15` every row; the word at `p37+60` laid as the address of `p16` every row; the word at `p37+64` laid as the address of `p17` every row; the word at `p37+68` laid as the address of `p18` every row; the word at `p37+72` laid as the address of `p19` every row; the word at `p37+76` laid as the address of `p20` every row; the word at `p37+80` laid as the address of `p21` every row; the word at `p37+84` laid as the address of `p22` every row; the word at `p37+88` laid as the address of `p23` every row; the word at `p37+92` laid as the address of `p24` every row; the word at `p37+96` laid as the address of `p25` every row; the word at `p37+100` laid as the address of `p26` every row; the word at `p37+104` laid as the address of `p27` every row; the word at `p37+108` laid as the address of `p28` every row; the word at `p37+112` laid as the address of `p29` every row; the word at `p37+116` laid as the address of `p30` every row; the word at `p37+120` laid as the address of `p31` every row; the word at `p37+124` laid as the address of `p32` every row; the word at `p37+128` laid as the address of `p33` every row; the word at `p37+132` laid as the address of `p34` every row; the word at `p37+136` laid as the address of `p35` every row; the word at `p37+140` laid as the address of `p36` every row; stub 0 answers 0..2 every row.
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

ENTRY = 0x00668700
WIN = 72
OWN = 2
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607976, 64), (12607972, 2112)]
AT = []
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('call', 7213936, 1)]
PLACES = [(None, '0xe85ddc', '0x87c', 144), ('p0', '0', '0', 464), ('p0', '4', '0', 464), ('p0', '8', '0', 464), ('p0', '12', '0', 464), ('p0', '16', '0', 464), ('p0', '20', '0', 464), ('p0', '24', '0', 464), ('p0', '28', '0', 464), ('p0', '32', '0', 464), ('p0', '36', '0', 464), ('p0', '40', '0', 464), ('p0', '44', '0', 464), ('p0', '48', '0', 464), ('p0', '52', '0', 464), ('p0', '56', '0', 464), ('p0', '60', '0', 464), ('p0', '64', '0', 464), ('p0', '68', '0', 464), ('p0', '72', '0', 464), ('p0', '76', '0', 464), ('p0', '80', '0', 464), ('p0', '84', '0', 464), ('p0', '88', '0', 464), ('p0', '92', '0', 464), ('p0', '96', '0', 464), ('p0', '100', '0', 464), ('p0', '104', '0', 464), ('p0', '108', '0', 464), ('p0', '112', '0', 464), ('p0', '116', '0', 464), ('p0', '120', '0', 464), ('p0', '124', '0', 464), ('p0', '128', '0', 464), ('p0', '132', '0', 464), ('p0', '136', '0', 464), ('p0', '140', '0', 464), (None, '0xc0aabc', '0x87c', 144)]
NULLS = []
NULLFORCE = {}
WBIAS = [('c0', 40, ('set', [524288, 524288, 0, 65536, 131072, 262144])), ('c0', 52, ('set', [262144, 327680, 393216, 458752, 262400, 327936, 393472, 459008, 262656, 328192, 393728, 459264, 262912, 328448, 393984, 459520, 263168, 328704, 394240, 459776, 262145, 327681, 393217, 458753, 262401, 327937, 393473, 459009, 262657, 328193, 393729, 459265, 262913, 328449, 393985, 459521, 263169, 328705, 394241, 459777, 262146, 327682, 393218, 458754, 262402, 327938, 393474, 459010, 262658, 328194, 393730, 459266, 262914, 328450, 393986, 459522, 263170, 328706, 394242, 459778, 262147, 327683, 393219, 458755, 262403, 327939, 393475, 459011, 262659, 328195, 393731, 459267, 262915, 328451, 393987, 459523, 263171, 328707, 394243, 459779, 262148, 327684, 393220, 458756, 262404, 327940, 393476, 459012, 262660, 328196, 393732, 459268, 262916, 328452, 393988, 459524, 263172, 328708, 394244, 459780])), ('c1', 2092, ('set', [0, 0, 1, 3])), ('m', 0, ('set', [11812424])), ('m', 4, ('set', [49, 50, 96, 413, 414, 420, 543, 560, 622, 623, 624, 625, 627, 628, 623, 624, 625, 627, 628, 96, 96, 96, 96])), ('m', 48, ('set', [-1, -1, 543, 544, 545, 546, 547, 548, 549, 550, 551, 552, 553, 554, 555, 556, 557, 558, 559, 560, 561, 562, 563, 564, 565, 566, 567, 568, 569, 570, 571, 572, 573, 574, 575, 576, 577, 578])), ('m', 52, ('set', [-1, -1, 543, 544, 545, 546, 547, 548, 549, 550, 551, 552, 553, 554, 555, 556, 557, 558, 559, 560, 561, 562, 563, 564, 565, 566, 567, 568, 569, 570, 571, 572, 573, 574, 575, 576, 577, 578, 544, 545, 546, 547, 548, 549, 550, 544, 545, 546, 547, 548, 549, 550, 544, 545, 546, 547, 548, 549, 550, 544, 545, 546, 547, 548, 549, 550, 572, 573, 574, 575, 576, 577, 578, 572, 573, 574, 575, 576, 577, 578, 572, 573, 560, 572, 573, 560])), ('m', 56, ('set', [-1, -1, 543, 544, 545, 546, 547, 548, 549, 550, 551, 552, 553, 554, 555, 556, 557, 558, 559, 560, 561, 562, 563, 564, 565, 566, 567, 568, 569, 570, 571, 572, 573, 574, 575, 576, 577, 578, 544, 545, 546, 547, 548, 549, 550, 544, 545, 546, 547, 548, 549, 550, 544, 545, 546, 547, 548, 549, 550, 551, 552, 553, 554, 555, 556, 557, 558, 559, 560, 561, 562, 563, 564, 565, 566, 567, 568, 569, 570, 571, 572, 573, 574, 575, 576, 577, 578, 572, 573, 560, 561, 572, 573, 560, 561])), ('m', 64, ('set', [420, 420, 421, 0, -1])), ('p1', 0, ('set', [11812424])), ('p1', 4, ('set', [543])), ('p1', 456, ('range', 0, 6)), ('p2', 0, ('set', [11812424])), ('p2', 4, ('set', [544])), ('p2', 456, ('range', 0, 6)), ('p3', 0, ('set', [11812424])), ('p3', 4, ('set', [545])), ('p3', 456, ('range', 0, 6)), ('p4', 0, ('set', [11812424])), ('p4', 4, ('set', [546])), ('p4', 456, ('range', 0, 6)), ('p5', 0, ('set', [11812424])), ('p5', 4, ('set', [547])), ('p5', 456, ('range', 0, 6)), ('p6', 0, ('set', [11812424])), ('p6', 4, ('set', [548])), ('p6', 456, ('range', 0, 6)), ('p7', 0, ('set', [11812424])), ('p7', 4, ('set', [549])), ('p7', 456, ('range', 0, 6)), ('p8', 0, ('set', [11812424])), ('p8', 4, ('set', [550])), ('p8', 456, ('range', 0, 6)), ('p9', 0, ('set', [11812424])), ('p9', 4, ('set', [551])), ('p9', 20, ('set', [3])), ('p9', 456, ('range', 0, 6)), ('p10', 0, ('set', [11812424])), ('p10', 4, ('set', [552])), ('p10', 20, ('set', [3])), ('p10', 456, ('range', 0, 6)), ('p11', 0, ('set', [11812424])), ('p11', 4, ('set', [553])), ('p11', 20, ('set', [3])), ('p11', 456, ('range', 0, 6)), ('p12', 0, ('set', [11812424])), ('p12', 4, ('set', [554])), ('p12', 20, ('set', [3])), ('p12', 456, ('range', 0, 6)), ('p13', 0, ('set', [11812424])), ('p13', 4, ('set', [555])), ('p13', 20, ('set', [3])), ('p13', 456, ('range', 0, 6)), ('p14', 0, ('set', [11812424])), ('p14', 4, ('set', [556])), ('p14', 20, ('set', [3])), ('p14', 456, ('range', 0, 6)), ('p15', 0, ('set', [11812424])), ('p15', 4, ('set', [557])), ('p15', 20, ('set', [3])), ('p15', 456, ('range', 0, 6)), ('p16', 0, ('set', [11812424])), ('p16', 4, ('set', [558])), ('p16', 20, ('set', [2])), ('p16', 456, ('range', 0, 6)), ('p17', 0, ('set', [11812424])), ('p17', 4, ('set', [559])), ('p17', 20, ('set', [2])), ('p17', 456, ('range', 0, 6)), ('p18', 0, ('set', [11812424])), ('p18', 4, ('set', [560])), ('p18', 20, ('set', [2])), ('p18', 456, ('range', 0, 6)), ('p19', 0, ('set', [11812424])), ('p19', 4, ('set', [561])), ('p19', 20, ('set', [2])), ('p19', 456, ('range', 0, 6)), ('p20', 0, ('set', [11812424])), ('p20', 4, ('set', [562])), ('p20', 20, ('set', [2])), ('p20', 456, ('range', 0, 6)), ('p21', 0, ('set', [11812424])), ('p21', 4, ('set', [563])), ('p21', 20, ('set', [2])), ('p21', 456, ('range', 0, 6)), ('p22', 0, ('set', [11812424])), ('p22', 4, ('set', [564])), ('p22', 20, ('set', [2])), ('p22', 456, ('range', 0, 6)), ('p23', 0, ('set', [11812424])), ('p23', 4, ('set', [565])), ('p23', 20, ('set', [1])), ('p23', 456, ('range', 0, 6)), ('p24', 0, ('set', [11812424])), ('p24', 4, ('set', [566])), ('p24', 20, ('set', [1])), ('p24', 456, ('range', 0, 6)), ('p25', 0, ('set', [11812424])), ('p25', 4, ('set', [567])), ('p25', 20, ('set', [1])), ('p25', 456, ('range', 0, 6)), ('p26', 0, ('set', [11812424])), ('p26', 4, ('set', [568])), ('p26', 20, ('set', [1])), ('p26', 456, ('range', 0, 6)), ('p27', 0, ('set', [11812424])), ('p27', 4, ('set', [569])), ('p27', 20, ('set', [1])), ('p27', 456, ('range', 0, 6)), ('p28', 0, ('set', [11812424])), ('p28', 4, ('set', [570])), ('p28', 20, ('set', [1])), ('p28', 456, ('range', 0, 6)), ('p29', 0, ('set', [11812424])), ('p29', 4, ('set', [571])), ('p29', 20, ('set', [1])), ('p29', 456, ('range', 0, 6)), ('p30', 0, ('set', [11812424])), ('p30', 4, ('set', [572])), ('p30', 20, ('set', [0])), ('p30', 456, ('range', 0, 6)), ('p31', 0, ('set', [11812424])), ('p31', 4, ('set', [573])), ('p31', 20, ('set', [0])), ('p31', 456, ('range', 0, 6)), ('p32', 0, ('set', [11812424])), ('p32', 4, ('set', [574])), ('p32', 20, ('set', [0])), ('p32', 456, ('range', 0, 6)), ('p33', 0, ('set', [11812424])), ('p33', 4, ('set', [575])), ('p33', 20, ('set', [0])), ('p33', 456, ('range', 0, 6)), ('p34', 0, ('set', [11812424])), ('p34', 4, ('set', [576])), ('p34', 20, ('set', [0])), ('p34', 456, ('range', 0, 6)), ('p35', 0, ('set', [11812424])), ('p35', 4, ('set', [577])), ('p35', 20, ('set', [0])), ('p35', 456, ('range', 0, 6)), ('p36', 0, ('set', [11812424])), ('p36', 4, ('set', [578])), ('p36', 20, ('set', [0])), ('p36', 456, ('range', 0, 6))]
FILLS = []
ARGRANGE = [(0, -1, 4), (1, -1, 3)]
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
ALIAS = [('p37', 0, 'p1', 0, True), ('p37', 4, 'p2', 0, True), ('p37', 8, 'p3', 0, True), ('p37', 12, 'p4', 0, True), ('p37', 16, 'p5', 0, True), ('p37', 20, 'p6', 0, True), ('p37', 24, 'p7', 0, True), ('p37', 28, 'p8', 0, True), ('p37', 32, 'p9', 0, True), ('p37', 36, 'p10', 0, True), ('p37', 40, 'p11', 0, True), ('p37', 44, 'p12', 0, True), ('p37', 48, 'p13', 0, True), ('p37', 52, 'p14', 0, True), ('p37', 56, 'p15', 0, True), ('p37', 60, 'p16', 0, True), ('p37', 64, 'p17', 0, True), ('p37', 68, 'p18', 0, True), ('p37', 72, 'p19', 0, True), ('p37', 76, 'p20', 0, True), ('p37', 80, 'p21', 0, True), ('p37', 84, 'p22', 0, True), ('p37', 88, 'p23', 0, True), ('p37', 92, 'p24', 0, True), ('p37', 96, 'p25', 0, True), ('p37', 100, 'p26', 0, True), ('p37', 104, 'p27', 0, True), ('p37', 108, 'p28', 0, True), ('p37', 112, 'p29', 0, True), ('p37', 116, 'p30', 0, True), ('p37', 120, 'p31', 0, True), ('p37', 124, 'p32', 0, True), ('p37', 128, 'p33', 0, True), ('p37', 132, 'p34', 0, True), ('p37', 136, 'p35', 0, True), ('p37', 140, 'p36', 0, True)]
ANSWERBY = []
ANSWERWIN = []
CDECL = False
TIB = False
ARGEQ = []
SEQ = []
ANSRANGE = [(0, 0, 2)]
CELLTHIS = []
VTALSO = []
SPARSE = []
GTAB = []
RECORD = []
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
    rng = random.Random(3261)
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

# sweep_gen: ["--item", "3261", "obj", "TypeData::get_preq@00668700", "--own", "2", "--win", "0x48", "--rows", "1200", "--argrange", "0:-1:4", "--argrange", "1:-1:3", "--cell-win", "0xc061e8:0x40", "--cell-win", "0xc061e4:0x840", "--wbias", "c0+0x28:0x80000,0x80000,0,0x10000,0x20000,0x40000", "--wbias", "c0+0x34:0x40000,0x50000,0x60000,0x70000,0x40100,0x50100,0x60100,0x70100,0x40200,0x50200,0x60200,0x70200,0x40300,0x50300,0x60300,0x70300,0x40400,0x50400,0x60400,0x70400,0x40001,0x50001,0x60001,0x70001,0x40101,0x50101,0x60101,0x70101,0x40201,0x50201,0x60201,0x70201,0x40301,0x50301,0x60301,0x70301,0x40401,0x50401,0x60401,0x70401,0x40002,0x50002,0x60002,0x70002,0x40102,0x50102,0x60102,0x70102,0x40202,0x50202,0x60202,0x70202,0x40302,0x50302,0x60302,0x70302,0x40402,0x50402,0x60402,0x70402,0x40003,0x50003,0x60003,0x70003,0x40103,0x50103,0x60103,0x70103,0x40203,0x50203,0x60203,0x70203,0x40303,0x50303,0x60303,0x70303,0x40403,0x50403,0x60403,0x70403,0x40004,0x50004,0x60004,0x70004,0x40104,0x50104,0x60104,0x70104,0x40204,0x50204,0x60204,0x70204,0x40304,0x50304,0x60304,0x70304,0x40404,0x50404,0x60404,0x70404", "--wbias", "c1+0x82c:0,0,1,3", "--wbias", "m+0:0xb43e48", "--wbias", "m+4:0x31,0x32,0x60,0x19d,0x19e,0x1a4,0x21f,0x230,0x26e,0x26f,0x270,0x271,0x273,0x274,0x26f,0x270,0x271,0x273,0x274,0x60,0x60,0x60,0x60", "--wbias", "m+0x30:-1,-1,0x21f,0x220,0x221,0x222,0x223,0x224,0x225,0x226,0x227,0x228,0x229,0x22a,0x22b,0x22c,0x22d,0x22e,0x22f,0x230,0x231,0x232,0x233,0x234,0x235,0x236,0x237,0x238,0x239,0x23a,0x23b,0x23c,0x23d,0x23e,0x23f,0x240,0x241,0x242", "--wbias", "m+0x34:-1,-1,0x21f,0x220,0x221,0x222,0x223,0x224,0x225,0x226,0x227,0x228,0x229,0x22a,0x22b,0x22c,0x22d,0x22e,0x22f,0x230,0x231,0x232,0x233,0x234,0x235,0x236,0x237,0x238,0x239,0x23a,0x23b,0x23c,0x23d,0x23e,0x23f,0x240,0x241,0x242,0x220,0x221,0x222,0x223,0x224,0x225,0x226,0x220,0x221,0x222,0x223,0x224,0x225,0x226,0x220,0x221,0x222,0x223,0x224,0x225,0x226,0x220,0x221,0x222,0x223,0x224,0x225,0x226,0x23c,0x23d,0x23e,0x23f,0x240,0x241,0x242,0x23c,0x23d,0x23e,0x23f,0x240,0x241,0x242,0x23c,0x23d,0x230,0x23c,0x23d,0x230", "--wbias", "m+0x38:-1,-1,0x21f,0x220,0x221,0x222,0x223,0x224,0x225,0x226,0x227,0x228,0x229,0x22a,0x22b,0x22c,0x22d,0x22e,0x22f,0x230,0x231,0x232,0x233,0x234,0x235,0x236,0x237,0x238,0x239,0x23a,0x23b,0x23c,0x23d,0x23e,0x23f,0x240,0x241,0x242,0x220,0x221,0x222,0x223,0x224,0x225,0x226,0x220,0x221,0x222,0x223,0x224,0x225,0x226,0x220,0x221,0x222,0x223,0x224,0x225,0x226,0x227,0x228,0x229,0x22a,0x22b,0x22c,0x22d,0x22e,0x22f,0x230,0x231,0x232,0x233,0x234,0x235,0x236,0x237,0x238,0x239,0x23a,0x23b,0x23c,0x23d,0x23e,0x23f,0x240,0x241,0x242,0x23c,0x23d,0x230,0x231,0x23c,0x23d,0x230,0x231", "--wbias", "m+0x40:0x1a4,0x1a4,0x1a5,0,-1", "--stub", "0x6e1370:1", "--answer-range", "0:0:2", "--place", "0xe85ddc:0x87c:0x90", "--place", "p0+0:0:0x1d0", "--place", "p0+4:0:0x1d0", "--place", "p0+8:0:0x1d0", "--place", "p0+12:0:0x1d0", "--place", "p0+16:0:0x1d0", "--place", "p0+20:0:0x1d0", "--place", "p0+24:0:0x1d0", "--place", "p0+28:0:0x1d0", "--place", "p0+32:0:0x1d0", "--place", "p0+36:0:0x1d0", "--place", "p0+40:0:0x1d0", "--place", "p0+44:0:0x1d0", "--place", "p0+48:0:0x1d0", "--place", "p0+52:0:0x1d0", "--place", "p0+56:0:0x1d0", "--place", "p0+60:0:0x1d0", "--place", "p0+64:0:0x1d0", "--place", "p0+68:0:0x1d0", "--place", "p0+72:0:0x1d0", "--place", "p0+76:0:0x1d0", "--place", "p0+80:0:0x1d0", "--place", "p0+84:0:0x1d0", "--place", "p0+88:0:0x1d0", "--place", "p0+92:0:0x1d0", "--place", "p0+96:0:0x1d0", "--place", "p0+100:0:0x1d0", "--place", "p0+104:0:0x1d0", "--place", "p0+108:0:0x1d0", "--place", "p0+112:0:0x1d0", "--place", "p0+116:0:0x1d0", "--place", "p0+120:0:0x1d0", "--place", "p0+124:0:0x1d0", "--place", "p0+128:0:0x1d0", "--place", "p0+132:0:0x1d0", "--place", "p0+136:0:0x1d0", "--place", "p0+140:0:0x1d0", "--place", "0xc0aabc:0x87c:0x90", "--alias", "p37+0:p1!", "--alias", "p37+4:p2!", "--alias", "p37+8:p3!", "--alias", "p37+12:p4!", "--alias", "p37+16:p5!", "--alias", "p37+20:p6!", "--alias", "p37+24:p7!", "--alias", "p37+28:p8!", "--alias", "p37+32:p9!", "--alias", "p37+36:p10!", "--alias", "p37+40:p11!", "--alias", "p37+44:p12!", "--alias", "p37+48:p13!", "--alias", "p37+52:p14!", "--alias", "p37+56:p15!", "--alias", "p37+60:p16!", "--alias", "p37+64:p17!", "--alias", "p37+68:p18!", "--alias", "p37+72:p19!", "--alias", "p37+76:p20!", "--alias", "p37+80:p21!", "--alias", "p37+84:p22!", "--alias", "p37+88:p23!", "--alias", "p37+92:p24!", "--alias", "p37+96:p25!", "--alias", "p37+100:p26!", "--alias", "p37+104:p27!", "--alias", "p37+108:p28!", "--alias", "p37+112:p29!", "--alias", "p37+116:p30!", "--alias", "p37+120:p31!", "--alias", "p37+124:p32!", "--alias", "p37+128:p33!", "--alias", "p37+132:p34!", "--alias", "p37+136:p35!", "--alias", "p37+140:p36!", "--wbias", "p1+0:0xb43e48", "--wbias", "p1+4:0x21f", "--wbias", "p1+0x1c8:0..6", "--wbias", "p2+0:0xb43e48", "--wbias", "p2+4:0x220", "--wbias", "p2+0x1c8:0..6", "--wbias", "p3+0:0xb43e48", "--wbias", "p3+4:0x221", "--wbias", "p3+0x1c8:0..6", "--wbias", "p4+0:0xb43e48", "--wbias", "p4+4:0x222", "--wbias", "p4+0x1c8:0..6", "--wbias", "p5+0:0xb43e48", "--wbias", "p5+4:0x223", "--wbias", "p5+0x1c8:0..6", "--wbias", "p6+0:0xb43e48", "--wbias", "p6+4:0x224", "--wbias", "p6+0x1c8:0..6", "--wbias", "p7+0:0xb43e48", "--wbias", "p7+4:0x225", "--wbias", "p7+0x1c8:0..6", "--wbias", "p8+0:0xb43e48", "--wbias", "p8+4:0x226", "--wbias", "p8+0x1c8:0..6", "--wbias", "p9+0:0xb43e48", "--wbias", "p9+4:0x227", "--wbias", "p9+0x14:3", "--wbias", "p9+0x1c8:0..6", "--wbias", "p10+0:0xb43e48", "--wbias", "p10+4:0x228", "--wbias", "p10+0x14:3", "--wbias", "p10+0x1c8:0..6", "--wbias", "p11+0:0xb43e48", "--wbias", "p11+4:0x229", "--wbias", "p11+0x14:3", "--wbias", "p11+0x1c8:0..6", "--wbias", "p12+0:0xb43e48", "--wbias", "p12+4:0x22a", "--wbias", "p12+0x14:3", "--wbias", "p12+0x1c8:0..6", "--wbias", "p13+0:0xb43e48", "--wbias", "p13+4:0x22b", "--wbias", "p13+0x14:3", "--wbias", "p13+0x1c8:0..6", "--wbias", "p14+0:0xb43e48", "--wbias", "p14+4:0x22c", "--wbias", "p14+0x14:3", "--wbias", "p14+0x1c8:0..6", "--wbias", "p15+0:0xb43e48", "--wbias", "p15+4:0x22d", "--wbias", "p15+0x14:3", "--wbias", "p15+0x1c8:0..6", "--wbias", "p16+0:0xb43e48", "--wbias", "p16+4:0x22e", "--wbias", "p16+0x14:2", "--wbias", "p16+0x1c8:0..6", "--wbias", "p17+0:0xb43e48", "--wbias", "p17+4:0x22f", "--wbias", "p17+0x14:2", "--wbias", "p17+0x1c8:0..6", "--wbias", "p18+0:0xb43e48", "--wbias", "p18+4:0x230", "--wbias", "p18+0x14:2", "--wbias", "p18+0x1c8:0..6", "--wbias", "p19+0:0xb43e48", "--wbias", "p19+4:0x231", "--wbias", "p19+0x14:2", "--wbias", "p19+0x1c8:0..6", "--wbias", "p20+0:0xb43e48", "--wbias", "p20+4:0x232", "--wbias", "p20+0x14:2", "--wbias", "p20+0x1c8:0..6", "--wbias", "p21+0:0xb43e48", "--wbias", "p21+4:0x233", "--wbias", "p21+0x14:2", "--wbias", "p21+0x1c8:0..6", "--wbias", "p22+0:0xb43e48", "--wbias", "p22+4:0x234", "--wbias", "p22+0x14:2", "--wbias", "p22+0x1c8:0..6", "--wbias", "p23+0:0xb43e48", "--wbias", "p23+4:0x235", "--wbias", "p23+0x14:1", "--wbias", "p23+0x1c8:0..6", "--wbias", "p24+0:0xb43e48", "--wbias", "p24+4:0x236", "--wbias", "p24+0x14:1", "--wbias", "p24+0x1c8:0..6", "--wbias", "p25+0:0xb43e48", "--wbias", "p25+4:0x237", "--wbias", "p25+0x14:1", "--wbias", "p25+0x1c8:0..6", "--wbias", "p26+0:0xb43e48", "--wbias", "p26+4:0x238", "--wbias", "p26+0x14:1", "--wbias", "p26+0x1c8:0..6", "--wbias", "p27+0:0xb43e48", "--wbias", "p27+4:0x239", "--wbias", "p27+0x14:1", "--wbias", "p27+0x1c8:0..6", "--wbias", "p28+0:0xb43e48", "--wbias", "p28+4:0x23a", "--wbias", "p28+0x14:1", "--wbias", "p28+0x1c8:0..6", "--wbias", "p29+0:0xb43e48", "--wbias", "p29+4:0x23b", "--wbias", "p29+0x14:1", "--wbias", "p29+0x1c8:0..6", "--wbias", "p30+0:0xb43e48", "--wbias", "p30+4:0x23c", "--wbias", "p30+0x14:0", "--wbias", "p30+0x1c8:0..6", "--wbias", "p31+0:0xb43e48", "--wbias", "p31+4:0x23d", "--wbias", "p31+0x14:0", "--wbias", "p31+0x1c8:0..6", "--wbias", "p32+0:0xb43e48", "--wbias", "p32+4:0x23e", "--wbias", "p32+0x14:0", "--wbias", "p32+0x1c8:0..6", "--wbias", "p33+0:0xb43e48", "--wbias", "p33+4:0x23f", "--wbias", "p33+0x14:0", "--wbias", "p33+0x1c8:0..6", "--wbias", "p34+0:0xb43e48", "--wbias", "p34+4:0x240", "--wbias", "p34+0x14:0", "--wbias", "p34+0x1c8:0..6", "--wbias", "p35+0:0xb43e48", "--wbias", "p35+4:0x241", "--wbias", "p35+0x14:0", "--wbias", "p35+0x1c8:0..6", "--wbias", "p36+0:0xb43e48", "--wbias", "p36+4:0x242", "--wbias", "p36+0x14:0", "--wbias", "p36+0x1c8:0..6"]
