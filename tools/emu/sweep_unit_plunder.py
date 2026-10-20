#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_plunder.py — `Unit::plunder@00605b00` under unicorn (item 3356, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_unit_plunder.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 128-byte object of random bytes (no bias), 4 stack argument(s);
`p` windows in order: p0..p11 sparse table 0 (2 rows × 6 slice(s), row-major), p12..p27 the place(s), in flag order, p28 stub 8's answer window; the global cell 0x00c061f0 points at 3200 random bytes; the global cell 0x00c061ec points at 2304 random bytes; the global cell 0x00c06210 points at 672 random bytes; the global cell 0x00c06208 points at 64 random bytes; the global cell 0x00c061ac points at 64 random bytes; the global cell 0x00c061fc points at 64 random bytes; the global cell 0x00e85ddc points at 256 random bytes; the global cell 0x00c06184 points at 16 random bytes; 4 random bytes at 0x00c8cd00; 1 random bytes at 0x00cb195a (each byte in 0..1); 64 random bytes at 0x00cc2328; callee 0x0073a1b0 (6 args) stubbed; callee 0x006d9380 (1 args) stubbed; callee 0x006e33a0 (2 args) stubbed; callee 0x00a1d440 (1 args) stubbed; callee 0x00a1d180 (1 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x00a16120 (2 args) stubbed; callee 0x0046e620 (0 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x00595260 (0 args) stubbed; the import cell 0x00ac5480 holds a stub (4 args); vtable slot +0xb8 (2 args) stubbed; vtable slot +0xc4 (0 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0xd0 (0 args) stubbed; vtable slot +0x60 (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0x68 (1 args) stubbed; vtable slot +0x78 (7 args) stubbed; the word at `0xc0aec0+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p12+0` points at 96 random bytes less `0` (placed); the word at `p12+4` points at 96 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p15+0` points at 96 random bytes less `0` (placed); the word at `p15+4` points at 96 random bytes less `0` (placed); the word at `p13+0x18` points at 768 random bytes less `0` (placed); the word at `p14+0x18` points at 768 random bytes less `0` (placed); the word at `p16+0x18` points at 768 random bytes less `0` (placed); the word at `p17+0x18` points at 768 random bytes less `0` (placed); the word at `m+0x18` points at 768 random bytes less `0` (placed); the word at `p5+0` points at 224 random bytes less `0` (placed); the word at `p11+0` points at 224 random bytes less `0` (placed); the word at `c4+0x10` points at 64 random bytes less `0` (placed); the word at `c5+0x10` points at 256 random bytes less `0` (placed); the word at `c6+0xc8` points at 32 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0xc4 laid as 0x0046ce60 or its stub, a flag after the answers; slot +0xcc laid as 0x0046ce80 or its stub, a flag after the answers; slot +0xd0 laid as 0x0046ce90 or its stub, a flag after the answers; stub 0 answered from [-1, -1, 0, 1, 5] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [1, 1, 1, 0, 0] (90%); stub 13 answered from [0, 1] (90%); stub 14 answered from [8, 9, 9, 9, 10] (90%); stub 16 answered from [0, 1] (90%); stub 17 answered from [0, 1] (90%); stub 18 answered from [0, 1] (90%); stub 19 answered from [0, 1] (90%); stub 20 answered from [0, 1] (90%); stub 21 answered from [0, 1] (90%); stub 8 answers the address of 64 random bytes, a `p` window after the places; the word at `p26+0` laid as the address of `p18` every row; the word at `p26+4` laid as the address of `p19` every row; the word at `p26+8` laid as the address of `p20` every row; the word at `p26+0xc` laid as the address of `p21` every row; the word at `p26+0xc8` laid as the address of `p20` every row; stub 0 answers 2 values in call order, printed after the answers; stub 2 answers 6 values in call order, printed after the answers; stub 12 answers 6 values in call order, printed after the answers; stub 23 answers 6 values in call order, printed after the answers; stub 7 answers 1..9 every row; stub 12 answers 0..65535 every row; stub 22 answers 0..3 every row; stub 23 answers -50..3000 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 15 is `__cdecl`: the caller pops its arguments; the stack 1024 bytes under the call is zeroed before each row; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x8:0x4, +0x914:0x4, +0x59bc:0x4, +0x59d8:0x4, +0x6928:0x4, +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`.
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

ENTRY = 0x00605b00
WIN = 128
OWN = 4
BIAS = []
ARGSET = [(0, [0, 1, 2]), (-2, [0, 1]), (2, [0, 1]), (3, [0, 1])]
ARGFROM = [(0, 10, 2), (1, 9, 1)]
CELLS = [(12607984, 3200), (12607980, 2304), (12608016, 672), (12608008, 64), (12607916, 64), (12607996, 64), (15228380, 256), (12607876, 16)]
AT = [(13159680, 4), (13310298, 1), (13378344, 64)]
ATRANGE = {13310298: (0, 1, 1)}
PTRARGS = []
PNULLS = []
STUBS = [('call', 7578032, 6), ('call', 7181184, 1), ('call', 7222176, 2), ('call', 10605632, 1), ('call', 10604928, 1), ('call', 10612400, 1), ('call', 10604352, 0), ('call', 10576160, 2), ('call', 4646432, 0), ('call', 9186496, 1), ('call', 8298416, 10), ('call', 9959280, 1), ('call', 10722672, 2), ('call', 7213936, 1), ('call', 5853792, 0), ('iat', 11293824, 4), ('slot', 184, 2), ('slot', 196, 0), ('slot', 204, 0), ('slot', 208, 0), ('slot', 96, 0), ('slot', 304, 0), ('slot', 104, 1), ('slot', 120, 7)]
PLACES = [(None, '0xc0aec0+0x1c*0', '0', 8), ('p12', '0', '0', 96), ('p12', '4', '0', 96), (None, '0xc0aec0+0x1c*1', '0', 8), ('p15', '0', '0', 96), ('p15', '4', '0', 96), ('p13', '0x18', '0', 768), ('p14', '0x18', '0', 768), ('p16', '0x18', '0', 768), ('p17', '0x18', '0', 768), ('m', '0x18', '0', 768), ('p5', '0', '0', 224), ('p11', '0', '0', 224), ('c4', '0x10', '0', 64), ('c5', '0x10', '0', 256), ('c6', '0xc8', '0', 32)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 8, ('set', [0, 256, 65536, 65792])), ('m', 40, ('set', [0, 1, 65535, 1515847681])), ('m', 60, ('set', [0, 65536, 2868969472, 2868903936])), ('m', 104, ('set', [0, 0, 0, 0, 0, 1, 16384])), ('p13', 84, ('set', [0, 0, 0, 1, 2, 3])), ('p14', 84, ('set', [0, 0, 0, 1, 2, 3])), ('p16', 84, ('set', [0, 0, 0, 1, 2, 3])), ('p17', 84, ('set', [0, 0, 0, 1, 2, 3])), ('p22', 692, ('set', [0, 16, 16, 17, 32])), ('p18', 696, ('set', [0, 32, 64, 96, 32])), ('p19', 696, ('set', [0, 32, 64, 96, 32])), ('p20', 696, ('set', [0, 32, 64, 96, 32])), ('p21', 696, ('set', [0, 32, 64, 96, 32])), ('p23', None, ('set', [33313, 33312, 33314, 33316, 33349, 33401, 99967710, 99967711, 99967708, 99967709, 33313, 33315])), ('p24', None, ('set', [33313, 33312, 33314, 33316, 33349, 33401, 99967710, 99967711, 99967708, 99967709, 33313, 33315])), ('p1', 0, ('set', [0, 1065353216, 1056964608, 1073741824, 3212836864, 2147483648, 2143289344, 1266679808, 2139095039, 3221225472, 1036831949, 1078530011, 1069547520, 1065353216, 1065353216, 2139095040, 1, 1065353215])), ('p7', 0, ('set', [0, 1065353216, 1056964608, 1073741824, 3212836864, 2147483648, 2143289344, 1266679808, 2139095039, 3221225472, 1036831949, 1078530011, 1069547520, 1065353216, 1065353216, 2139095040, 1, 1065353215])), ('p2', 0, ('set', [0, 65536, 327680, 4294901760, 4660])), ('p8', 0, ('set', [0, 65536, 327680, 4294901760, 4660])), ('p3', 0, ('set', [0, 1, 2, 65536, 65535])), ('p9', 0, ('set', [0, 1, 2, 65536, 65535])), ('c0', 948, ('set', [0, 5, 25, 100, -7, 1000])), ('c0', 952, ('set', [0, 5, 25, 100, -7, 1000])), ('c0', 1388, ('set', [0, 15, 3, 100])), ('c0', 1392, ('set', [0, 60, 20, 1000])), ('c0', 1400, ('set', [0, 100, 50, 200, -5])), ('c0', 1440, ('set', [0, 25, 100, 50])), ('c0', 3188, ('set', [0, 15, 100, -3, 15, 15])), ('c1', 2080, ('set', [0, 0, 4096, 131072, 135168, 4294967295])), ('c2', 664, ('set', [0, 1, 0, 1, -1, 5]))]
FILLS = []
ARGRANGE = []
SLOTBASE = [(184, 6633360), (196, 4640352), (204, 4640384), (208, 4640400)]
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [-1, -1, 0, 1, 5]), (1, [0, 1]), (2, [1, 1, 1, 0, 0]), (13, [0, 1]), (14, [8, 9, 9, 9, 10]), (16, [0, 1]), (17, [0, 1]), (18, [0, 1]), (19, [0, 1]), (20, [0, 1]), (21, [0, 1])]
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = [('p26', 0, 'p18', 0, True), ('p26', 4, 'p19', 0, True), ('p26', 8, 'p20', 0, True), ('p26', 12, 'p21', 0, True), ('p26', 200, 'p20', 0, True)]
ANSWERBY = []
ANSWERWIN = [(8, 64)]
CDECL = False
TIB = True
ARGEQ = []
SEQ = [(0, 2, None), (2, 6, None), (12, 6, None), (23, 6, None)]
ANSRANGE = [(7, 1, 9), (12, 0, 65535), (22, 0, 3), (23, -50, 3000)]
CELLTHIS = []
VTALSO = [('p18', 0), ('p19', 0), ('p20', 0), ('p21', 0), ('p27', 0), ('p13', 0), ('p14', 0), ('p16', 0), ('p17', 0)]
SPARSE = [('@', '0xe3a390', 28396, 2, [(8, 4), (2324, 4), (22972, 4), (23000, 4), (26920, 4), (28344, 4)])]
GTAB = []
RECORD = [0, 1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23]
RECORDORDER = True
RECORDEDX = False
ROWSPLIT = {}
LINKS = []
LINKHI = []
OUTARG = []
CDECLSTUBS = [15]
STUBWRITE = []
STUBCOPY = []
STACKCLEAN = 1024
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
    rng = random.Random(3356)
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
    for i in range(6000):
        h.brk = base  # the bump heap is reused a row; 6000 rows of windows outgrow it
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

# sweep_gen: ["--item", "3356", "obj", "Unit::plunder@00605b00", "--win=0x80", "--own=4", "--ret=none", "--tib", "--clean-stack=0x400", "--rows=6000", "--name=unit_plunder", "--cell-win=0xc061f0:0xc80", "--cell-win=0xc061ec:0x900", "--cell-win=0xc06210:0x2a0", "--cell-win=0xc06208:0x40", "--cell-win=0xc061ac:0x40", "--cell-win=0xc061fc:0x40", "--cell-win=0xe85ddc:0x100", "--cell-win=0xc06184:0x10", "--at=0xc8cd00:4", "--at=0xcb195a:1:0..1/b", "--at=0xcc2328:0x40", "--sparse=@0xe3a390:0x6eec:2:0x8,4/0x914,4/0x59bc,4/0x59d8,4/0x6928,4/0x6eb8,4", "--ptrtab=0xc0aec0+0x1c*0:2:0x60:vt", "--ptrtab=0xc0aec0+0x1c*1:2:0x60:vt", "--stub=0x73a1b0:6", "--stub=0x6d9380:1", "--stub=0x6e33a0:2", "--stub=0xa1d440:1", "--stub=0xa1d180:1", "--stub=0xa1eeb0:1", "--stub=0xa1cf40:0", "--stub=0xa16120:2", "--stub=0x46e620:0", "--stub=0x8c2cc0:1", "--stub=0x7e9fb0:10", "--stub=0x97f770:1", "--stub=0xa39d70:2", "--stub=0x6e1370:1", "--stub=0x595260:0", "--stub=iat:0xac5480:4", "--stub=slot:0xb8:2", "--stub=slot:0xc4:0", "--stub=slot:0xcc:0", "--stub=slot:0xd0:0", "--stub=slot:0x60:0", "--stub=slot:0x130:0", "--stub=slot:0x68:1", "--stub=slot:0x78:7", "--stub-cdecl=15", "--place=p13+0x18:0:0x300", "--place=p14+0x18:0:0x300", "--place=p16+0x18:0:0x300", "--place=p17+0x18:0:0x300", "--place=m+0x18:0:0x300", "--place=p5+0:0:0xe0", "--place=p11+0:0:0xe0", "--place=c4+0x10:0:0x40", "--place=c5+0x10:0:0x100", "--place=c6+0xc8:0:0x20", "--answer-win=8:0x40", "--vtat-also=p18+0", "--vtat-also=p19+0", "--vtat-also=p20+0", "--vtat-also=p21+0", "--vtat-also=p27+0", "--alias=p26+0:p18!", "--alias=p26+4:p19!", "--alias=p26+8:p20!", "--alias=p26+0xc:p21!", "--alias=p26+0xc8:p20!", "--vtat-keep=0xb8:0x653790", "--vtat-keep=0xc4:0x46ce60", "--vtat-keep=0xcc:0x46ce80", "--vtat-keep=0xd0:0x46ce90", "--argset=0:0,1,2", "--argset=1!:0,1", "--argset=2:0,1", "--argset=3:0,1", "--argfrom=0:0xa:2", "--argfrom=1:9:1", "--wbias=m+8:0,0x100,0x10000,0x10100", "--wbias=m+0x28:0,1,0xffff,0x5a5a0001", "--wbias=m+0x3c:0,0x10000,0xab010000,0xab000000", "--wbias=m+0x68:0,0,0,0,0,1,0x4000", "--wbias=p13+0x54:0,0,0,1,2,3", "--wbias=p14+0x54:0,0,0,1,2,3", "--wbias=p16+0x54:0,0,0,1,2,3", "--wbias=p17+0x54:0,0,0,1,2,3", "--wbias=p22+0x2b4:0,0x10,0x10,0x11,0x20", "--wbias=p18+0x2b8:0,0x20,0x40,0x60,0x20", "--wbias=p19+0x2b8:0,0x20,0x40,0x60,0x20", "--wbias=p20+0x2b8:0,0x20,0x40,0x60,0x20", "--wbias=p21+0x2b8:0,0x20,0x40,0x60,0x20", "--wbias=p23+*:0x8221,0x8220,0x8222,0x8224,0x8245,0x8279,0x5f562de,0x5f562df,0x5f562dc,0x5f562dd,0x8221,0x8223", "--wbias=p24+*:0x8221,0x8220,0x8222,0x8224,0x8245,0x8279,0x5f562de,0x5f562df,0x5f562dc,0x5f562dd,0x8221,0x8223", "--wbias=p1+0:0x0,0x3f800000,0x3f000000,0x40000000,0xbf800000,0x80000000,0x7fc00000,0x4b800000,0x7f7fffff,0xc0000000,0x3dcccccd,0x40490fdb,0x3fc00000,0x3f800000,0x3f800000,0x7f800000,0x1,0x3f7fffff", "--wbias=p7+0:0x0,0x3f800000,0x3f000000,0x40000000,0xbf800000,0x80000000,0x7fc00000,0x4b800000,0x7f7fffff,0xc0000000,0x3dcccccd,0x40490fdb,0x3fc00000,0x3f800000,0x3f800000,0x7f800000,0x1,0x3f7fffff", "--wbias=p2+0:0,0x10000,0x50000,0xffff0000,0x1234", "--wbias=p8+0:0,0x10000,0x50000,0xffff0000,0x1234", "--wbias=p3+0:0,1,2,0x10000,0xffff", "--wbias=p9+0:0,1,2,0x10000,0xffff", "--wbias=c0+0x3b4:0,5,25,100,-7,1000", "--wbias=c0+0x3b8:0,5,25,100,-7,1000", "--wbias=c0+0x56c:0,15,3,100", "--wbias=c0+0x570:0,60,20,1000", "--wbias=c0+0x578:0,100,50,200,-5", "--wbias=c0+0x5a0:0,25,100,50", "--wbias=c0+0xc74:0,15,100,-3,15,15", "--wbias=c1+0x820:0,0,0x1000,0x20000,0x21000,0xffffffff", "--wbias=c2+0x298:0,1,0,1,-1,5", "--answers=0:-1,-1,0,1,5", "--seq=0:2", "--answers=1:0,1", "--answers=2:1,1,1,0,0", "--seq=2:6", "--answer-range=7:1:9", "--answers=13:0,1", "--answers=14:8,9,9,9,10", "--answer-range=12:0:65535", "--seq=12:6", "--answers=16:0,1", "--answers=17:0,1", "--answers=18:0,1", "--answers=19:0,1", "--answers=20:0,1", "--answers=21:0,1", "--answer-range=22:0:3", "--answer-range=23:-50:3000", "--seq=23:6", "--record=0", "--record=1", "--record=2", "--record=3", "--record=4", "--record=5", "--record=6", "--record=7", "--record=9", "--record=10", "--record=11", "--record=12", "--record=13", "--record=14", "--record=15", "--record=16", "--record=17", "--record=18", "--record=19", "--record=20", "--record=21", "--record=22", "--record=23", "--record-order", "--port", "// Row: `a[0..4]` the arguments (`p1`..`p4`), `a[4 + k]` stub `k`'s answer, then the seq values\n// in flag order: `a[28..30]` `find_hero`'s, `a[30..36]` `type_avail`'s, `a[36..42]`\n// `Random::get`'s, `a[42..48]` the price slot's; then `a[48..52]` whether slot `+0xb8`, `+0xc4`,\n// `+0xcc`, `+0xd0` held the native function (`ObjectData::is`, `is_hero`, `is_supply`,\n// `is_caravan`) and so was inlined. Stubs: 0 `find_hero`, 1 `tribe_can_type`, 2 `type_avail`,\n// 3 `String += String`, 4 `String += int`, 5 `String =`, 6 `String::close`, 7 `String::reinit`,\n// 8 `Recycler::pop` (not recorded: its `ecx` is a register the stubbed callees left), 9\n// `get_neon`, 10 `add_message`, 11 `SoundGlobal::play`, 12 `Random::get`, 13 `has_tribe_bonus`,\n// 14 `get_patch_version`, 15 `_itow_s`, then the slots `+0xb8` (16), `+0xc4` (17), `+0xcc` (18),\n// `+0xd0` (19), `+0x60` (20), `+0x130` (21), `+0x68` (22), `+0x78` (23). Windows: `m` the\n// unit, `c0` `constants`, `c1` `game`, `c2` the console, `c3` the message window, `c4`\n// `goodtypes`, `c5` `unittypes`, `c6` `types.list`, `c7` `game_random`, `c8` the localised\n// string table's base, `c9` `ScenarioData::plunder`, `c10` `String::buff`; `p[6r..6r+6]` leader\n// `r`'s `+8`, `+0x914`, `+0x59bc`, `+0x59d8`, `+0x6928` and `+0x6eb8`; `p12` `units[0]` and\n// `p13`/`p14` its two units, `p15` `units[1]` and `p16`/`p17`; `p18..p21` their type records;\n// `p22` the unit's own type record; `p23`/`p24` the buckets; `p25` the goods' array; `p26` the\n// unit types' array (entries 0 to 3 name `p18` to `p21`, entry `0x32` names `p20`); `p27` the\n// record `types[0x32]`; `p28` the text bubble.\n//\n// **The port is the crate's own steps**, `crate::plunder::plunder_steps`, which\n// `Sim::unit_plunder` calls over its `SimPlunder`; here the view is the emulated original's\n// windows. `LeaderDataEncrypt::scratch` is the bucket write's twin and nothing of the row\n// reads it. `wcscpy_s` is not reached: no stub can lay the pointer in the caller's local\n// that `String::reinit` would, so `reinit` answers non-zero (parked: a stub that writes\n// through its `ecx`).\nuse crate::plunder::{PlunderView, Str, plunder_steps};\nstruct V<'a> {\n    a: &'a [i32],\n    m: &'a mut [u8],\n    c: &'a [Vec<u8>],\n    p: &'a mut [Vec<u8>],\n    log: &'a mut Vec<String>,\n    n0: usize,\n    n2: usize,\n    n12: usize,\n    n23: usize,\n}\nimpl V<'_> {\n    fn leader(who: i32) -> i32 {\n        0xe3a390 + who * 0x6eec\n    }\n    fn row(who: i32) -> usize {\n        6 * who as usize\n    }\n    fn unit(who: i32, idx: i32) -> usize {\n        13 + 3 * who as usize + idx as usize\n    }\n    fn tw(who: i32, idx: i32) -> usize {\n        18 + 2 * who as usize + idx as usize\n    }\n    fn s(s: Str) -> &'static str {\n        match s {\n            Str::A => \"sp-0x5c\",\n            Str::B => \"sp-0x70\",\n        }\n    }\n    fn tc(&self, who: i32) -> i32 {\n        0xe39ce0 + 100 * i32::from(self.p[Self::row(who) + 4][0])\n    }\n    fn loc(&self) -> i32 {\n        rd(&self.c[8], 0)\n    }\n    fn f8(&self, who: i32) -> i32 {\n        rd(&self.p[Self::row(who)], 0)\n    }\n}\nimpl PlunderView for V<'_> {\n    type Ty = usize;\n    type Bub = usize;\n    fn flags68(&mut self) -> i32 {\n        rd(self.m, 0x68)\n    }\n    fn this_who(&mut self) -> i32 {\n        i32::from(self.m[9])\n    }\n    fn this_index(&mut self) -> i32 {\n        i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]))\n    }\n    fn this_coord(&mut self) -> (i32, i32) {\n        (rd(self.m, 0x10) ^ 0x63637, rd(self.m, 0x14) ^ 0x63637)\n    }\n    fn this_redirects(&mut self) -> bool {\n        rd(&self.p[22], 0x2b4) & 0x10 != 0\n    }\n    fn this_redirect(&mut self) -> (i32, i32) {\n        (\n            i32::from(self.m[0x3e] as i8),\n            i32::from(i16::from_le_bytes([self.m[0x28], self.m[0x29]])),\n        )\n    }\n    fn thedespot_unit_plunder(&mut self) -> i32 {\n        rd(&self.c[0], 0xc74)\n    }\n    fn general_plunder(&mut self) -> i32 {\n        rd(&self.c[0], 0x3b8)\n    }\n    fn caravan_plunder(&mut self) -> i32 {\n        rd(&self.c[0], 0x3b4)\n    }\n    fn aztec_slay_bonus(&mut self) -> i32 {\n        rd(&self.c[0], 0x56c)\n    }\n    fn aztec_slay_cap(&mut self) -> i32 {\n        rd(&self.c[0], 0x570)\n    }\n    fn aztec_plunder(&mut self) -> i32 {\n        rd(&self.c[0], 0x578)\n    }\n    fn inca_refund(&mut self) -> i32 {\n        rd(&self.c[0], 0x5a0)\n    }\n    fn hero_count(&mut self, who: i32, ty: i32) -> i32 {\n        let r = Self::row(who);\n        let (w, o) = if ty == 0x160 { (r + 2, 2) } else { (r + 3, 0) };\n        i32::from(i16::from_le_bytes([self.p[w][o], self.p[w][o + 1]]))\n    }\n    fn find_hero(&mut self, x: i32, y: i32, who: i32, ty: i32) -> i32 {\n        self.log\n            .push(format!(\"s0:{x},{x},{y},{},0,{ty},0\", self.f8(who)));\n        let r = self.a[28 + self.n0.min(1)];\n        self.n0 += 1;\n        r\n    }\n    fn type_avail(&mut self, who: i32, good: i32) -> bool {\n        self.log.push(format!(\"s2:{},{good},1\", Self::leader(who)));\n        let r = self.a[30 + self.n2.min(5)];\n        self.n2 += 1;\n        r != 0\n    }\n    fn has_tribe_bonus(&mut self, who: i32, bonus: i32) -> bool {\n        self.log.push(format!(\"s13:{},{bonus}\", Self::leader(who)));\n        self.a[4 + 13] != 0\n    }\n    fn bucket(&mut self, who: i32, good: i32) -> i32 {\n        rd(&self.p[23 + who as usize], 4 * good as usize) ^ 0x8221\n    }\n    fn slay_count(&mut self, who: i32) -> i32 {\n        rd(&self.p[23 + who as usize], 0xdc) ^ 0x62766\n    }\n    fn add_good(&mut self, who: i32, good: i32, n: i32) {\n        let w = 23 + who as usize;\n        let new = (rd(&self.p[w], 4 * good as usize) ^ 0x8221).wrapping_add(n);\n        wr(&mut self.p[w], 4 * good as usize, new ^ 0x8221);\n    }\n    fn plunder_scale(&mut self, who: i32) -> u32 {\n        rd(&self.p[Self::row(who) + 1], 0) as u32\n    }\n    fn unit_type(&mut self, who: i32, idx: i32) -> usize {\n        Self::tw(who, idx)\n    }\n    fn unit_is_42(&mut self, who: i32, idx: i32, ty: usize) -> bool {\n        if self.a[48] != 0 {\n            self.log.push(format!(\"s20:p{ty}+0x0\"));\n            self.a[4 + 20] != 0\n        } else {\n            self.log\n                .push(format!(\"s16:p{}+0x0,66,0\", Self::unit(who, idx)));\n            self.a[4 + 16] != 0\n        }\n    }\n    fn unit_cached_type(&mut self, who: i32, idx: i32) -> i32 {\n        rd(&self.p[Self::unit(who, idx)], 0x54)\n    }\n    fn set_unit_cached_type(&mut self, who: i32, idx: i32, t: i32) {\n        wr(&mut self.p[Self::unit(who, idx)], 0x54, t);\n    }\n    fn tribe_default_type(&mut self, who: i32) -> i32 {\n        self.log.push(format!(\"s1:{},p27+0x0\", Self::leader(who)));\n        if self.a[4 + 1] != 0 {\n            0x32\n        } else {\n            self.log.push(format!(\"s22:p27+0x0,{}\", self.f8(who)));\n            self.a[4 + 22]\n        }\n    }\n    fn unittype_record(&mut self, i: i32) -> usize {\n        // The array's entries are laid as the addresses of the unit types' windows.\n        match i {\n            0..=3 => 18 + i as usize,\n            0x32 => 20,\n            _ => unreachable!(\"an entry nothing laid: {i}\"),\n        }\n    }\n    fn price(&mut self, ty: usize, good: i32, who: i32) -> i32 {\n        self.log\n            .push(format!(\"s23:p{ty}+0x0,{good},{who},-1,-1,1,1,-1\"));\n        let r = self.a[42 + self.n23.min(5)];\n        self.n23 += 1;\n        r\n    }\n    fn unit_is_hero(&mut self, who: i32, idx: i32) -> bool {\n        if self.a[49] != 0 {\n            rd(&self.p[Self::tw(who, idx)], 0x2b8) & 0x20 != 0\n        } else {\n            self.log\n                .push(format!(\"s17:p{}+0x0\", Self::unit(who, idx)));\n            self.a[4 + 17] != 0\n        }\n    }\n    fn unit_is_supply(&mut self, who: i32, idx: i32) -> bool {\n        if self.a[50] != 0 {\n            rd(&self.p[Self::tw(who, idx)], 0x2b8) & 0x40 != 0\n        } else {\n            self.log\n                .push(format!(\"s18:p{}+0x0\", Self::unit(who, idx)));\n            self.a[4 + 18] != 0\n        }\n    }\n    fn unit_is_caravan(&mut self, who: i32, idx: i32) -> bool {\n        if self.a[51] != 0 {\n            self.log.push(format!(\"s21:p{}+0x0\", Self::tw(who, idx)));\n            self.a[4 + 21] != 0\n        } else {\n            self.log\n                .push(format!(\"s19:p{}+0x0\", Self::unit(who, idx)));\n            self.a[4 + 19] != 0\n        }\n    }\n    fn patch_version(&mut self) -> i32 {\n        self.log.push(\"s14:c1+0x0\".into());\n        self.a[4 + 14]\n    }\n    fn game_bytes(&mut self) -> (i32, i32) {\n        (i32::from(self.c[1][0x821]), i32::from(self.c[1][0x822]))\n    }\n    fn scenario_plunder(&mut self) -> bool {\n        self.c[9][0] != 0\n    }\n    fn rand(&mut self, lo: i32, hi: i32) -> i32 {\n        self.log.push(format!(\"s12:c7+0x0,{lo},{hi}\"));\n        let r = self.a[36 + self.n12.min(5)];\n        self.n12 += 1;\n        r\n    }\n    fn console_who(&mut self) -> i32 {\n        rd(&self.c[2], 0x298)\n    }\n    fn str_assign_loc(&mut self, s: Str, off: i32) {\n        self.log.push(format!(\n            \"s5:{},{}\",\n            Self::s(s),\n            self.loc().wrapping_add(off)\n        ));\n    }\n    fn str_cat_loc(&mut self, s: Str, off: i32) {\n        self.log.push(format!(\n            \"s3:{},{}\",\n            Self::s(s),\n            self.loc().wrapping_add(off)\n        ));\n    }\n    fn str_cat_int(&mut self, s: Str, n: i32) {\n        self.log.push(format!(\"s4:{},{n}\", Self::s(s)));\n    }\n    fn str_cat_good(&mut self, s: Str, good: i32) {\n        let name = rd(&self.p[25], 4 * good as usize).wrapping_add(0x74);\n        self.log.push(format!(\"s3:{},{name}\", Self::s(s)));\n    }\n    fn str_cat_str(&mut self, dst: Str, src: Str) {\n        self.log\n            .push(format!(\"s3:{},{}\", Self::s(dst), Self::s(src)));\n    }\n    fn itow(&mut self, n: i32) {\n        self.log.push(format!(\"s15:sp-0x70,{n},c10+0x0,1024,10\"));\n    }\n    fn buff_len(&mut self) -> i32 {\n        // The table ends in zeros: the window's end is a terminator.\n        let b = &self.c[10];\n        (0..b.len() / 2)\n            .find(|&i| b[2 * i] == 0 && b[2 * i + 1] == 0)\n            .unwrap_or(b.len() / 2) as i32\n    }\n    fn str_reinit(&mut self, s: Str, len: i32) -> i32 {\n        self.log.push(format!(\"s7:{},{len},-1\", Self::s(s)));\n        self.a[4 + 7]\n    }\n    fn str_copy_buff(&mut self, _s: Str) {\n        unreachable!(\"reinit answers non-zero in the sweep\");\n    }\n    fn str_close(&mut self, s: Str) {\n        self.log.push(format!(\"s6:{}\", Self::s(s)));\n    }\n    fn pop_bubble(&mut self) -> usize {\n        28\n    }\n    fn bubble_text(&mut self, b: usize, s: Str) {\n        self.log.push(format!(\"s5:p{b}+0x4,{}\", Self::s(s)));\n    }\n    fn bubble_who(&mut self, b: usize, who: i32) {\n        self.p[b][0x1a] = who as u8;\n    }\n    fn neon(&mut self, who: i32) -> i32 {\n        self.log.push(format!(\"s9:{},sp-0x48\", self.tc(who)));\n        self.a[4 + 9]\n    }\n    fn add_message(&mut self, s: Str, b: usize, color: i32, who: i32) {\n        let (x, y) = self.this_coord();\n        let tc = self.tc(who);\n        self.log.push(format!(\n            \"s10:c3+0x0,{},{x},{y},0,{color},1,{tc},0,p{b}+0x0,{tc}\",\n            Self::s(s)\n        ));\n    }\n    fn play(&mut self, sound: i32) {\n        self.log.push(format!(\"s11:c3+0x0,{sound}\"));\n    }\n}\nlet mut v = V {\n    a,\n    m,\n    c,\n    p,\n    log: &mut log,\n    n0: 0,\n    n2: 0,\n    n12: 0,\n    n23: 0,\n};\nplunder_steps(&mut v, a[0], a[1], a[2], a[3]);\n// The recorder prints `-` for a row that called nothing.\nif log.is_empty() {\n    log.push(\"-\".into());\n}\n0"]
