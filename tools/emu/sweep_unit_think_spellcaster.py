#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_think_spellcaster.py — `Unit::think_spellcaster@005f27a0` under unicorn (item 3330, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_unit_think_spellcaster.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 256-byte object of random bytes (no bias), 0 stack argument(s);
`p` windows in order: p0..p7 sparse table 0 (4 rows × 2 slice(s), row-major), p8..p24 the place(s), in flag order; the global cell 0x00c061a8 points at 32 random bytes; the global cell 0x00c061e8 points at 2112 random bytes; the global cell 0x00c0618c points at 528 random bytes; the global cell 0x00c06184 points at 16 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 64 random bytes; vtable slot +0xd4 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xc4 (0 args) stubbed; vtable slot +0xd8 (0 args) stubbed; vtable slot +0xe8 (0 args) stubbed; callee 0x00675bc0 (3 args) stubbed; callee 0x00609a50 (0 args) stubbed; callee 0x0060a6a0 (0 args) stubbed; callee 0x00676a80 (4 args) stubbed; callee 0x0065c6b0 (13 args) stubbed; callee 0x0065ca80 (13 args) stubbed; callee 0x005e4a60 (7 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x00649d70 (0 args) stubbed; the word at `0xc0aec0+0x1c*m[9]` points at 16 random bytes less `0` (placed); the word at `p8+0` points at 128 random bytes less `0` (placed); the word at `p8+4` points at 128 random bytes less `0` (placed); the word at `p8+8` points at 128 random bytes less `0` (placed); the word at `p8+12` points at 128 random bytes less `0` (placed); the word at `m+0x18` points at 704 random bytes less `0` (placed); the word at `m+0xf4` points at 8 random bytes less `0` (placed); the word at `p14+0` points at 144 random bytes less `0` (placed); the word at `c0+0x10` points at 2576 random bytes less `0` (placed); the word at `p16+0x9d4` points at 468 random bytes less `0` (placed); the word at `p16+0x9dc` points at 468 random bytes less `0` (placed); the word at `p16+0x9e8` points at 468 random bytes less `0` (placed); the word at `p16+0x9ec` points at 468 random bytes less `0` (placed); the word at `p16+0x9f0` points at 468 random bytes less `0` (placed); the word at `p16+0xa04` points at 468 random bytes less `0` (placed); the word at `c2+0x14+0x1c*m[9]` points at 16 random bytes less `0` (placed); the word at `c4+0x134` points at 512 random bytes less `0` (placed); slot +0xd4 laid as 0x0046cea0 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0xc4 laid as 0x0046ce60 or its stub, a flag after the answers; stub 5 answered from [0, 1, 1, 1, 1] (90%); stub 6 answered from [0, 1, 1, 1, 1, 1] (90%); stub 0 answered from [0, 1, 1, 0, 0] (90%); stub 1 answered from [0, 0, 0, 1] (90%); stub 2 answered from [0, 0, 0, 1] (90%); stub 3 answered from [0, 1, 1, 1, 1, 1, 1] (90%); stub 4 answered from [0, 1, 1] (90%); stub 8 answered from [0, 1, 1, 1] (90%); the word at `p23+0` laid as the address of `p9` every row; the word at `p23+4` laid as the address of `p10` every row; the word at `p23+8` laid as the address of `p11` every row; the word at `p23+12` laid as the address of `p12` every row; stub 5 answers 4 values in call order, printed after the answers; stub 6 answers 3 values in call order, printed after the answers; stub 7 answers 3 values in call order, printed after the answers; stub 9 answers 2 values in call order, printed after the answers; stub 7 answers 0..1200 every row; stub 9 answers 0..30 every row; stub 10 answers -1..5 every row; stub 11 answers -1..5 every row; stub 13 answers 0..65535 every row; stub 14 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 4 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x50:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`.
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

ENTRY = 0x005f27a0
WIN = 256
OWN = 0
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607912, 32), (12607976, 2112), (12607884, 528), (12607876, 16), (12607880, 320), (13297148, 64)]
AT = []
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('slot', 212, 0), ('slot', 184, 2), ('slot', 96, 2), ('slot', 196, 0), ('slot', 216, 0), ('slot', 232, 0), ('call', 6773696, 3), ('call', 6330960, 0), ('call', 6334112, 0), ('call', 6777472, 4), ('call', 6670000, 13), ('call', 6670976, 13), ('call', 6179424, 7), ('call', 10722672, 2), ('call', 6593904, 0)]
PLACES = [(None, '0xc0aec0+0x1c*m[9]', '0', 16), ('p8', '0', '0', 128), ('p8', '4', '0', 128), ('p8', '8', '0', 128), ('p8', '12', '0', 128), ('m', '0x18', '0', 704), ('m', '0xf4', '0', 8), ('p14', '0', '0', 144), ('c0', '0x10', '0', 2576), ('p16', '0x9d4', '0', 468), ('p16', '0x9dc', '0', 468), ('p16', '0x9e8', '0', 468), ('p16', '0x9ec', '0', 468), ('p16', '0x9f0', '0', 468), ('p16', '0xa04', '0', 468), ('c2', '0x14+0x1c*m[9]', '0', 16), ('c4', '0x134', '0', 512)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 8, ('set', [0, 327680, 3801088, 19660800, 4294901760, 256, 327936, 3801344, 19661056, 4294902016, 512, 328192, 3801600, 19661312, 4294902272, 768, 328448, 3801856, 19661568, 4294902528])), ('m', 16, ('set', [407095, 407090, 407240, 407351, 407346, 407496, 406583, 406578, 406728, 406839, 406834, 406984])), ('m', 20, ('set', [407095, 407090, 407240, 407351, 407346, 407496, 406583, 406578, 406728, 406839, 406834, 406984])), ('m', 104, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 262144, 262144, 262144, 1, 262145])), ('m', 148, ('set', [0, 0, 0, 327680, 6553600, 65536000, 4294901760])), ('p0', 0, ('set', [0, 0, 0, 4, 4, 32768, 32772, 32768, 16, 6])), ('p1', 0, ('set', [-1, 2, 2, 3, 4, 5, 0, 2, 3])), ('p2', 0, ('set', [0, 0, 0, 4, 4, 32768, 32772, 32768, 16, 6])), ('p3', 0, ('set', [-1, 2, 2, 3, 4, 5, 0, 2, 3])), ('p4', 0, ('set', [0, 0, 0, 4, 4, 32768, 32772, 32768, 16, 6])), ('p5', 0, ('set', [-1, 2, 2, 3, 4, 5, 0, 2, 3])), ('p6', 0, ('set', [0, 0, 0, 4, 4, 32768, 32772, 32768, 16, 6])), ('p7', 0, ('set', [-1, 2, 2, 3, 4, 5, 0, 2, 3])), ('c1', 2080, ('set', [0, 131072, 4096, 135168, 4, 131076, 4100, 135172, 1, 65793, 197379])), ('c1', 40, ('set', [33554432, 50331648, 67108864, 83886080, 33554432, 0, 16777216, 4278190080])), ('c2', 348, ('set', [4, 4, 3, 2, 1, 0])), ('c2', 352, ('set', [4, 4, 3, 2, 1, 0])), ('c2', 356, ('set', [4, 4, 3, 2, 1, 0])), ('c2', 360, ('set', [4, 4, 3, 2, 1, 0])), ('p9', 8, ('set', [0, 1, 1, 1, 1, 1, 1, 3, 256, 257])), ('p10', 8, ('set', [0, 1, 1, 1, 1, 1, 1, 3, 256, 257])), ('p11', 8, ('set', [0, 1, 1, 1, 1, 1, 1, 3, 256, 257])), ('p12', 8, ('set', [0, 1, 1, 1, 1, 1, 1, 3, 256, 257])), ('p9', 104, ('set', [0, 0, 0, 1, 1, 1, 262144, 262145])), ('p10', 104, ('set', [0, 0, 0, 1, 1, 1, 262144, 262145])), ('p11', 104, ('set', [0, 0, 0, 1, 1, 1, 262144, 262145])), ('p12', 104, ('set', [0, 0, 0, 1, 1, 1, 262144, 262145])), ('p13', 696, ('set', [0, 16, 32, 48, 16, 32, 48, 17, 33, 8])), ('p15', 132, ('set', [0, 0, 5, 65536, -3])), ('p17', 464, ('set', [0, 10, 100, 500, 1000, -5, 600])), ('p18', 464, ('set', [0, 10, 100, 500, 1000, -5, 600])), ('p19', 464, ('set', [0, 10, 100, 500, 1000, -5, 600])), ('p20', 464, ('set', [0, 10, 100, 500, 1000, -5, 600])), ('p21', 464, ('set', [0, 10, 100, 500, 1000, -5, 600])), ('p22', 464, ('set', [0, 10, 100, 500, 1000, -5, 600])), ('c4', 0, ('set', [1, 2, 3, 4])), ('c5', 0, ('set', [0, 1, 2, 3])), ('c5', 4, ('set', [0, 1, 2, 3])), ('c5', 8, ('set', [0, 1, 2, 3])), ('c5', 12, ('set', [0, 1, 2, 3])), ('p24', 12, ('set', [0, 16777216, 33554432, 50331648, 4278190080, 2147483648, 2130706432])), ('p24', 40, ('set', [0, 16777216, 33554432, 50331648, 4278190080, 2147483648, 2130706432])), ('p24', 68, ('set', [0, 16777216, 33554432, 50331648, 4278190080, 2147483648, 2130706432])), ('p24', 96, ('set', [0, 16777216, 33554432, 50331648, 4278190080, 2147483648, 2130706432])), ('p24', 124, ('set', [0, 16777216, 33554432, 50331648, 4278190080, 2147483648, 2130706432])), ('p24', 152, ('set', [0, 16777216, 33554432, 50331648, 4278190080, 2147483648, 2130706432])), ('p24', 180, ('set', [0, 16777216, 33554432, 50331648, 4278190080, 2147483648, 2130706432])), ('p24', 208, ('set', [0, 16777216, 33554432, 50331648, 4278190080, 2147483648, 2130706432])), ('p24', 236, ('set', [0, 16777216, 33554432, 50331648, 4278190080, 2147483648, 2130706432])), ('p24', 264, ('set', [0, 16777216, 33554432, 50331648, 4278190080, 2147483648, 2130706432])), ('p24', 292, ('set', [0, 16777216, 33554432, 50331648, 4278190080, 2147483648, 2130706432])), ('p24', 320, ('set', [0, 16777216, 33554432, 50331648, 4278190080, 2147483648, 2130706432])), ('p24', 348, ('set', [0, 16777216, 33554432, 50331648, 4278190080, 2147483648, 2130706432])), ('p24', 376, ('set', [0, 16777216, 33554432, 50331648, 4278190080, 2147483648, 2130706432])), ('p24', 404, ('set', [0, 16777216, 33554432, 50331648, 4278190080, 2147483648, 2130706432])), ('p24', 432, ('set', [0, 16777216, 33554432, 50331648, 4278190080, 2147483648, 2130706432]))]
FILLS = []
ARGRANGE = []
SLOTBASE = [(212, 4640416), (184, 6633360), (196, 4640352)]
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(5, [0, 1, 1, 1, 1]), (6, [0, 1, 1, 1, 1, 1]), (0, [0, 1, 1, 0, 0]), (1, [0, 0, 0, 1]), (2, [0, 0, 0, 1]), (3, [0, 1, 1, 1, 1, 1, 1]), (4, [0, 1, 1]), (8, [0, 1, 1, 1])]
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = [('p23', 0, 'p9', 0, True), ('p23', 4, 'p10', 0, True), ('p23', 8, 'p11', 0, True), ('p23', 12, 'p12', 0, True)]
ANSWERBY = []
ANSWERWIN = []
CDECL = False
TIB = False
ARGEQ = []
SEQ = [(5, 4, None), (6, 3, None), (7, 3, None), (9, 2, None)]
ANSRANGE = [(7, 0, 1200), (9, 0, 30), (10, -1, 5), (11, -1, 5), (13, 0, 65535), (14, -1, 3)]
CELLTHIS = []
VTALSO = [('p13', 0), ('p9', 0), ('p10', 0), ('p11', 0), ('p12', 0)]
SPARSE = [('@', '0xe3a390', 28396, 4, [(0, 4), (80, 4)])]
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
    rng = random.Random(3330)
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
    for i in range(8000):
        h.brk = base  # the bump heap is reused a row; 8000 rows of windows outgrow it
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

# sweep_gen: ["--item", "3330", "obj", "Unit::think_spellcaster@005f27a0", "--name", "unit_think_spellcaster", "--win", "0x100", "--own", "0", "--rows", "8000", "--cell-win", "0xc061a8:0x20", "--cell-win", "0xc061e8:0x840", "--cell-win", "0xc0618c:0x210", "--cell-win", "0xc06184:0x10", "--cell-win", "0xc06188:0x140", "--cell-win", "0xcae5fc:0x40", "--sparse", "@0xe3a390:0x6eec:4:0x0,4/0x50,4", "--ptrtab", "0xc0aec0+0x1c*m[9]:4:0x80:vt", "--place", "m+0x18:0:0x2c0", "--place", "m+0xf4:0:8", "--place", "p14+0:0:0x90", "--place", "c0+0x10:0:0xa10", "--place", "p16+0x9d4:0:0x1d4", "--place", "p16+0x9dc:0:0x1d4", "--place", "p16+0x9e8:0:0x1d4", "--place", "p16+0x9ec:0:0x1d4", "--place", "p16+0x9f0:0:0x1d4", "--place", "p16+0xa04:0:0x1d4", "--place", "c2+0x14+0x1c*m[9]:0:16", "--place", "c4+0x134:0:0x200", "--alias", "p23+0:p9!", "--alias", "p23+4:p10!", "--alias", "p23+8:p11!", "--alias", "p23+12:p12!", "--vtat-also", "p13+0", "--stub", "slot:0xd4:0", "--stub", "slot:0xb8:2", "--stub", "slot:0x60:2", "--stub", "slot:0xc4:0", "--stub", "slot:0xd8:0", "--stub", "slot:0xe8:0", "--stub", "0x675bc0:3", "--stub", "0x609a50:0", "--stub", "0x60a6a0:0", "--stub", "0x676a80:4", "--stub", "0x65c6b0:13", "--stub", "0x65ca80:13", "--stub", "0x5e4a60:7", "--stub", "0xa39d70:2", "--stub", "0x649d70:0", "--seq", "5:4", "--answers", "5:0,1,1,1,1", "--seq", "6:3", "--answers", "6:0,1,1,1,1,1", "--seq", "7:3", "--answer-range", "7:0:1200", "--seq", "9:2", "--answer-range", "9:0:30", "--answers", "0:0,1,1,0,0", "--answers", "1:0,0,0,1", "--answers", "2:0,0,0,1", "--answers", "3:0,1,1,1,1,1,1", "--answers", "4:0,1,1", "--answers", "8:0,1,1,1", "--answer-range", "10:-1:5", "--answer-range", "11:-1:5", "--answer-range", "13:0:65535", "--answer-range", "14:-1:3", "--vtat-keep", "0xd4:0x46cea0", "--vtat-keep", "0xb8:0x653790", "--vtat-keep", "0xc4:0x46ce60", "--wbias", "m+0x8:0x0,0x50000,0x3a0000,0x12c0000,0xffff0000,0x100,0x50100,0x3a0100,0x12c0100,0xffff0100,0x200,0x50200,0x3a0200,0x12c0200,0xffff0200,0x300,0x50300,0x3a0300,0x12c0300,0xffff0300", "--wbias", "m+0x10:0x63637,0x63632,0x636c8,0x63737,0x63732,0x637c8,0x63437,0x63432,0x634c8,0x63537,0x63532,0x635c8", "--wbias", "m+0x14:0x63637,0x63632,0x636c8,0x63737,0x63732,0x637c8,0x63437,0x63432,0x634c8,0x63537,0x63532,0x635c8", "--wbias", "m+0x68:0,0,0,0,0,0,0,0,0,0,0x40000,0x40000,0x40000,1,0x40001", "--wbias", "m+0x94:0,0,0,0x50000,0x640000,0x3e80000,0xffff0000", "--wbias", "p0+0:0,0,0,4,4,0x8000,0x8004,0x8000,0x10,0x6", "--wbias", "p1+0:-1,2,2,3,4,5,0,2,3", "--wbias", "p2+0:0,0,0,4,4,0x8000,0x8004,0x8000,0x10,0x6", "--wbias", "p3+0:-1,2,2,3,4,5,0,2,3", "--wbias", "p4+0:0,0,0,4,4,0x8000,0x8004,0x8000,0x10,0x6", "--wbias", "p5+0:-1,2,2,3,4,5,0,2,3", "--wbias", "p6+0:0,0,0,4,4,0x8000,0x8004,0x8000,0x10,0x6", "--wbias", "p7+0:-1,2,2,3,4,5,0,2,3", "--wbias", "c1+0x820:0x0,0x20000,0x1000,0x21000,0x4,0x20004,0x1004,0x21004,0x1,0x10101,0x30303", "--wbias", "c1+0x28:0x2000000,0x3000000,0x4000000,0x5000000,0x2000000,0,0x1000000,0xff000000", "--wbias", "c2+0x15c:4,4,3,2,1,0", "--wbias", "c2+0x160:4,4,3,2,1,0", "--wbias", "c2+0x164:4,4,3,2,1,0", "--wbias", "c2+0x168:4,4,3,2,1,0", "--wbias", "p9..12+0x8:0,1,1,1,1,1,1,3,256,257", "--wbias", "p9..12+0x68:0,0,0,1,1,1,0x40000,0x40001", "--wbias", "p13+0x2b8:0,0x10,0x20,0x30,0x10,0x20,0x30,0x11,0x21,0x8", "--wbias", "p15+0x84:0,0,5,65536,-3", "--wbias", "p17+0x1d0:0,10,100,500,1000,-5,600", "--wbias", "p18+0x1d0:0,10,100,500,1000,-5,600", "--wbias", "p19+0x1d0:0,10,100,500,1000,-5,600", "--wbias", "p20+0x1d0:0,10,100,500,1000,-5,600", "--wbias", "p21+0x1d0:0,10,100,500,1000,-5,600", "--wbias", "p22+0x1d0:0,10,100,500,1000,-5,600", "--wbias", "c4+0:1,2,3,4", "--wbias", "c5+0x0:0,1,2,3", "--wbias", "c5+0x4:0,1,2,3", "--wbias", "c5+0x8:0,1,2,3", "--wbias", "c5+0xc:0,1,2,3", "--wbias", "p24+0xc:0,0x1000000,0x2000000,0x3000000,0xff000000,0x80000000,0x7f000000", "--wbias", "p24+0x28:0,0x1000000,0x2000000,0x3000000,0xff000000,0x80000000,0x7f000000", "--wbias", "p24+0x44:0,0x1000000,0x2000000,0x3000000,0xff000000,0x80000000,0x7f000000", "--wbias", "p24+0x60:0,0x1000000,0x2000000,0x3000000,0xff000000,0x80000000,0x7f000000", "--wbias", "p24+0x7c:0,0x1000000,0x2000000,0x3000000,0xff000000,0x80000000,0x7f000000", "--wbias", "p24+0x98:0,0x1000000,0x2000000,0x3000000,0xff000000,0x80000000,0x7f000000", "--wbias", "p24+0xb4:0,0x1000000,0x2000000,0x3000000,0xff000000,0x80000000,0x7f000000", "--wbias", "p24+0xd0:0,0x1000000,0x2000000,0x3000000,0xff000000,0x80000000,0x7f000000", "--wbias", "p24+0xec:0,0x1000000,0x2000000,0x3000000,0xff000000,0x80000000,0x7f000000", "--wbias", "p24+0x108:0,0x1000000,0x2000000,0x3000000,0xff000000,0x80000000,0x7f000000", "--wbias", "p24+0x124:0,0x1000000,0x2000000,0x3000000,0xff000000,0x80000000,0x7f000000", "--wbias", "p24+0x140:0,0x1000000,0x2000000,0x3000000,0xff000000,0x80000000,0x7f000000", "--wbias", "p24+0x15c:0,0x1000000,0x2000000,0x3000000,0xff000000,0x80000000,0x7f000000", "--wbias", "p24+0x178:0,0x1000000,0x2000000,0x3000000,0xff000000,0x80000000,0x7f000000", "--wbias", "p24+0x194:0,0x1000000,0x2000000,0x3000000,0xff000000,0x80000000,0x7f000000", "--wbias", "p24+0x1b0:0,0x1000000,0x2000000,0x3000000,0xff000000,0x80000000,0x7f000000", "--record", "0", "--record", "1", "--record", "2", "--record", "3", "--record", "4", "--record", "5", "--record", "6", "--record", "7", "--record", "8", "--record", "9", "--record", "10", "--record", "11", "--record", "12", "--record", "13", "--record", "14", "--record-order", "--ret", "eax", "--port", "{\n                // **The port is the crate's own steps**, `crate::spellcaster::think_spellcaster_steps`, which\n                // `Sim::think_spellcaster` runs over its `SimCaster`; here the view is the emulated original's windows.\n                //\n                // `a`: no stack argument; then the fifteen stubs' single answers (`a[k]` for stub k: 0 the unit's\n                // `is_special` slot +0xd4, 1 its `is` (+0xb8), 2 the type's `is` (+0x60), 3 `is_hero` (+0xc4), 4\n                // `is_moving` (+0xd8), 5 the objects' `is_captain` (+0xe8), 6 `is_castable`, 7 `mana`, 8 `is_cloaked`,\n                // 9 `get_range`, 10 `find`, 11 `find_unit`, 12 `add_cast_order`, 13 `Random::get`, 14 `get_army`);\n                // stubs 5, 6, 7 and 9 answer in call order from `a[15..19]`, `a[19..22]`, `a[22..25]`, `a[25..27]`\n                // (the last repeated); then the three vtable flags `a[27]`, `a[28]`, `a[29]` (slot +0xd4, +0xb8,\n                // +0xc4 laid as the base function: the arm reads the type's flags itself, or asks the type's slot).\n                // Windows: `c[0]` the spelltypes holder (`+0x10` the table `p16`), `c[1]` the game's constants\n                // (`+0x820..+0x822` the semaphore, `+0x2b` the difficulty), `c[2]` the objects singleton (`+0x15c +\n                // 4 * who` the count, `+0x200` the word `add_cast_order` is handed), `c[3]` the game's random, `c[4]`\n                // the world (`+0` the width, `+0x134` the cells `p24`), `c[5]` the division table; `p0..p7` the\n                // leaders' slices (a row a player: `+0` the flags, `+0x50` the difficulty), `p9..p12` the leader's\n                // units (`p8` their array, `p23` the objects' list, the same records), `p13` the unit's type, `p14`\n                // and `p15` the guy and its data (`+0x84` the speed), `p16` the spell table and `p17..p22` the six\n                // spells' rows (Bribe, Counterintelligence, Create Decoy, Ambush, Forced March, Sniper).\n                //\n                // Four words of `find`'s thirteen are the stack's and not the function's (the `sub esp, 0xc` between\n                // its pushes) and so is the `ecx` it is called with (`get_range` left it): both sides are masked in\n                // the comparison below.\n                use crate::spellcaster::{CasterView, craft, think_spellcaster_steps};\n                struct V<'a> {\n                    a: &'a [i32],\n                    m: &'a [u8],\n                    c: &'a [Vec<u8>],\n                    p: &'a [Vec<u8>],\n                    log: &'a mut Vec<String>,\n                    turn: [usize; 15],\n                }\n                fn rd32(w: &[u8], o: usize) -> i32 {\n                    i32::from_le_bytes(w[o..o + 4].try_into().expect(\"four bytes\"))\n                }\n                fn rd16(w: &[u8], o: usize) -> i32 {\n                    i32::from(i16::from_le_bytes(w[o..o + 2].try_into().expect(\"two bytes\")))\n                }\n                impl V<'_> {\n                    /// Stub `k`'s next answer of `n` from `a[base]`, the last repeated.\n                    fn seq(&mut self, k: usize, base: usize, n: usize) -> i32 {\n                        let i = self.turn[k].min(n - 1);\n                        self.turn[k] += 1;\n                        self.a[base + i]\n                    }\n                    /// The spell's row window and its slot in the table.\n                    fn row(spell: i32) -> (usize, usize) {\n                        match spell {\n                            0x275 => (17, 0x9d4),\n                            0x277 => (18, 0x9dc),\n                            0x27a => (19, 0x9e8),\n                            0x27b => (20, 0x9ec),\n                            0x27c => (21, 0x9f0),\n                            0x281 => (22, 0xa04),\n                            _ => panic!(\"a spell the original does not ask here\"),\n                        }\n                    }\n                    fn x(&self) -> i32 {\n                        rd32(self.m, 0x10) ^ 0x63637\n                    }\n                    fn y(&self) -> i32 {\n                        rd32(self.m, 0x14) ^ 0x63637\n                    }\n                    fn leader(&self) -> usize {\n                        2 * self.m[9] as usize\n                    }\n                }\n                impl CasterView for V<'_> {\n                    fn who(&self) -> i32 {\n                        i32::from(self.m[9])\n                    }\n                    fn human(&self) -> bool {\n                        rd32(&self.p[self.leader()], 0) & 4 != 0\n                    }\n                    fn forced_march(&self) -> bool {\n                        rd32(&self.p[self.leader()], 0) & 0x8000 != 0\n                    }\n                    fn semaphore(&self) -> [u8; 3] {\n                        [self.c[1][0x820], self.c[1][0x821], self.c[1][0x822]]\n                    }\n                    fn leader_diff(&self) -> i32 {\n                        rd32(&self.p[self.leader() + 1], 0)\n                    }\n                    fn game_diff(&self) -> i32 {\n                        i32::from(self.c[1][0x2b])\n                    }\n                    fn decoy(&self) -> bool {\n                        rd32(self.m, 0x68) & 1 != 0\n                    }\n                    fn is_spy(&mut self) -> bool {\n                        if self.a[28] == 0 {\n                            self.log.push(\"s1:m+0x0,58,0\".into());\n                            self.a[1] != 0\n                        } else {\n                            self.log.push(\"s2:p13+0x0,58,0\".into());\n                            self.a[2] != 0\n                        }\n                    }\n                    fn is_special(&mut self) -> bool {\n                        if self.a[27] == 0 {\n                            self.log.push(\"s0:m+0x0\".into());\n                            self.a[0] != 0\n                        } else {\n                            rd32(&self.p[13], 0x2b8) & 0x10 != 0\n                        }\n                    }\n                    fn is_hero(&mut self) -> bool {\n                        if self.a[29] == 0 {\n                            self.log.push(\"s3:m+0x0\".into());\n                            self.a[3] != 0\n                        } else {\n                            rd32(&self.p[13], 0x2b8) & 0x20 != 0\n                        }\n                    }\n                    fn is_moving(&mut self) -> bool {\n                        self.log.push(\"s4:m+0x0\".into());\n                        self.a[4] != 0\n                    }\n                    fn cloaked(&mut self) -> bool {\n                        self.log.push(\"s8:m+0x0\".into());\n                        self.a[8] != 0\n                    }\n                    fn avg_speed(&self) -> i32 {\n                        rd32(&self.p[15], 0x84)\n                    }\n                    fn castable(&mut self, spell: i32) -> bool {\n                        let (w, _) = Self::row(spell);\n                        self.log.push(format!(\"s6:p{w}+0x0,{},{},0\", rd16(self.m, 0xa), self.m[9]));\n                        self.seq(6, 19, 3) != 0\n                    }\n                    fn mana(&mut self) -> i32 {\n                        self.log.push(\"s7:m+0x0\".into());\n                        self.seq(7, 22, 3)\n                    }\n                    fn mana_burn(&self) -> i32 {\n                        rd16(self.m, 0x96)\n                    }\n                    fn spell_mana(&self, spell: i32) -> i32 {\n                        rd32(&self.p[Self::row(spell).0], 0x1d0)\n                    }\n                    fn range(&mut self, spell: i32) -> i32 {\n                        let (w, slot) = Self::row(spell);\n                        self.log.push(format!(\"s9:p{w}+0x0,p16+{slot:#x},{},-1,-1\", self.m[9]));\n                        self.seq(9, 25, 2)\n                    }\n                    fn find(&mut self, spell: i32, mode: i32, range: i32) -> i32 {\n                        let (_, slot) = Self::row(spell);\n                        self.log.push(format!(\n                            \"s10:*,{},{},{mode},{},{range},p16+{slot:#x},20,{spell},{},*,*,*,0\",\n                            self.x(),\n                            self.y(),\n                            self.m[9],\n                            self.m[9]\n                        ));\n                        self.a[10]\n                    }\n                    fn find_unit(&mut self, spell: i32, range: i32) -> i32 {\n                        let (_, slot) = Self::row(spell);\n                        self.log.push(format!(\n                            \"s11:c2+0x0,{},{},3,{},{range},0,20,{spell},{},9,0,0,p16+{slot:#x}\",\n                            self.x(),\n                            self.y(),\n                            self.m[9],\n                            self.m[9]\n                        ));\n                        self.a[11]\n                    }\n                    fn army(&mut self) -> i32 {\n                        self.log.push(\"s14:m+0x0\".into());\n                        self.a[14]\n                    }\n                    fn coin(&mut self, _hero: bool) -> i32 {\n                        self.log.push(\"s13:c3+0x0,0,65535\".into());\n                        self.a[13]\n                    }\n                    fn cell_owner(&self) -> i32 {\n                        let ix = rd32(&self.c[5], 4 * (self.x() >> 8) as usize);\n                        let iy = rd32(&self.c[5], 4 * (self.y() >> 8) as usize);\n                        let cell = (iy.wrapping_mul(rd32(&self.c[4], 0)).wrapping_add(ix)) as usize;\n                        i32::from(self.p[24][cell * 0x1c + 0xf] as i8)\n                    }\n                    fn count(&self) -> i32 {\n                        rd32(&self.c[2], 0x15c + 4 * self.m[9] as usize)\n                    }\n                    fn nth(&self, k: i32) -> i32 {\n                        k\n                    }\n                    fn alive(&self, o: i32) -> bool {\n                        self.p[9 + o as usize][8] & 1 != 0\n                    }\n                    fn is_captain(&mut self, o: i32) -> bool {\n                        self.log.push(format!(\"s5:p{}+0x0\", 9 + o));\n                        self.seq(5, 15, 4) != 0\n                    }\n                    fn is_decoy(&self, o: i32) -> bool {\n                        rd32(&self.p[9 + o as usize], 0x68) & 1 != 0\n                    }\n                    fn add_cast(&mut self, target: i32, spell: i32) {\n                        self.log.push(format!(\n                            \"s12:m+0x0,{target},{},{},{},{spell},0,0\",\n                            rd32(&self.c[2], 0x200),\n                            self.x(),\n                            self.y()\n                        ));\n                    }\n                }\n                let _ = craft::BRIBE;\n                let mut v = V {\n                    a,\n                    m: &*m,\n                    c: &*c,\n                    p: &*p,\n                    log: &mut log,\n                    turn: [0; 15],\n                };\n                i64::from(think_spellcaster_steps(&mut v))\n            }"]
