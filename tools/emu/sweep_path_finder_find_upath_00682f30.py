#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_path_finder_find_upath_00682f30.py — `PathFinder::find_upath@00682f30` under unicorn (item 3339, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_path_finder_find_upath_00682f30.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 16-byte object of random bytes (no bias), 6 stack argument(s);
`p` windows in order: p0 argument 0, p1..p7 the place(s), in flag order, p8..p9 the stubs' answer window(s); the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c06188 points at 64 random bytes; the global cell 0x00c0618c points at 112 random bytes; the global cell 0x7c01002c points at 16 random bytes; 152 random bytes at 0x00e85e40; 112 random bytes at 0x00c0aec0; 20 random bytes at 0x00ee14fc; argument 0 names 16 random bytes; callee 0x0046f960 (0 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x00687c80 (3 args) stubbed; callee 0x00683770 (3 args) stubbed; callee 0x00687ae0 (0 args) stubbed; callee 0x006179d0 (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x0046e8c0 (1 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; the word at `p0+0` points at 512 random bytes less `0` (placed); the word at `c2+0x14+0x1c*a[3]` points at 12 random bytes less `0` (placed); the word at `p2+4*a[4]` points at 32 random bytes less `0` (placed); the word at `c5+0x1c*a[3]` points at 16 random bytes less `0` (placed); the word at `p4+4*a[4]` points at 16 random bytes less `0` (placed); the word at `p3+0x18` points at 560 random bytes less `0` (placed); the word at `c6+4` points at 512 random bytes less `0` (placed); stub 0 answered from [0, 0, 1] (90%); stub 2 answered from [0, 1, 1, 1] (90%); stub 8 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1, 2, 3] (90%); stub 5 answers the address of 32 random bytes, a `p` window after the places; stub 9 answers the address of 88 random bytes, a `p` window after the places; the word at `c3+0` laid as the address of `c3` every row; stub 1 answers 24 values in call order, printed after the answers; stub 2 answers 12 values in call order, the last from [1], printed after the answers; stub 3 answers -1..4 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); `0x00a46980` run once before the rows (a table builder the startup would have run); the stubbed vtable laid at `p8+0` as well as at `this`.
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

ENTRY = 0x00682f30
WIN = 16
OWN = 6
BIAS = []
ARGSET = [(-2, [30, 47, 48, 48, 96, 96, 144, 144, 192, 192, 240, 288, 336, 71, 73, 95, 119, 121, 167, 169, 215, 217, 100, 130, 300, 600, 624, 576, 1000, 1200, 1250, 2400, 2424, 2376]), (-3, [30, 47, 48, 48, 96, 96, 144, 144, 192, 192, 240, 288, 336, 71, 73, 95, 119, 121, 167, 169, 215, 217, 100, 130, 300, 600, 624, 576, 1000, 1200, 1250, 2400, 2424, 2376]), (5, [0, 1])]
ARGFROM = []
CELLS = [(13297148, 4096), (12607880, 64), (12607884, 112), (2080440364, 16)]
AT = [(15228480, 152), (12627648, 112), (15602940, 20)]
ATRANGE = {}
PTRARGS = [(0, 16)]
PNULLS = []
STUBS = [('call', 4651360, 0), ('call', 7033568, 2), ('call', 6847616, 3), ('call', 6829936, 3), ('call', 6847200, 0), ('call', 6388176, 0), ('call', 6171824, 1), ('call', 4647104, 1), ('slot', 20, 0), ('slot', 64, 0)]
PLACES = [('p0', '0', '0', 512), ('c2', '0x14+0x1c*a[3]', '0', 12), ('p2', '4*a[4]', '0', 32), ('c5', '0x1c*a[3]', '0', 16), ('p4', '4*a[4]', '0', 16), ('p3', '0x18', '0', 560), ('c6', '4', '0', 512)]
NULLS = []
NULLFORCE = {}
WBIAS = [('c1', 0, ('range', 2, 14)), ('c1', 4, ('range', 2, 14)), ('c3', 8, ('set', [5, 5, 1000])), ('c6', 0, ('set', [0, 0, 5])), ('c6', 8, ('set', [0, 1, 2, 3, 8, 64])), ('c6', 16, ('set', [-1, 4])), ('c4', 132, ('set', [0, 0, 1])), ('p0', 4, ('set', [0, 1, 2, 3, 5, 8, 16, 40])), ('p0', 8, ('set', [-1, 0, 1, 2, 3, 4, 4, 4, 5, 5, 6, 7, 8, 9, 12])), ('p0', 12, ('set', [-1, 2, 4])), ('p3', 16, ('set', [407167, 407167, 407167, 407167, 407119, 407199])), ('p3', 20, ('set', [407167, 407167, 407167, 407167, 407119, 407199])), ('p6', 536, ('set', [0, 1, 2, 2, 3])), ('p9', 28, ('set', [0, 0, 1, 3])), ('p1', 0, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 72, 72, 72, 72, 696, 72, 72, 72])), ('p1', 4, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 72, 72, 696, 72, 72, 72, 72])), ('p1', 8, ('set', [0, 0, 24, 96, 384])), ('p1', 12, ('set', [0, 0, 1, 2, 2, 2, 2, 3, 6, 6, 7])), ('p1', 16, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 120, 120, 120, 120, 648, 72, 72, 72])), ('p1', 20, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 120, 120, 648, 72, 72, 72, 72])), ('p1', 24, ('set', [0, 0, 24, 96, 384])), ('p1', 28, ('set', [0, 0, 1, 2, 2, 2, 2, 3, 6, 6, 7])), ('p1', 32, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 168, 168, 168, 168, 600, 72, 72, 72])), ('p1', 36, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 168, 168, 600, 72, 72, 72, 72])), ('p1', 40, ('set', [0, 0, 24, 96, 384])), ('p1', 44, ('set', [0, 0, 1, 2, 2, 2, 2, 3, 6, 6, 7])), ('p1', 48, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 216, 216, 216, 216, 552, 72, 72, 72])), ('p1', 52, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 216, 216, 552, 72, 72, 72, 72])), ('p1', 56, ('set', [0, 0, 24, 96, 384])), ('p1', 60, ('set', [0, 0, 1, 2, 2, 2, 2, 3, 6, 6, 7])), ('p1', 64, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 264, 264, 264, 264, 504, 72, 72, 72])), ('p1', 68, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 264, 264, 504, 72, 72, 72, 72])), ('p1', 72, ('set', [0, 0, 24, 96, 384])), ('p1', 76, ('set', [0, 0, 1, 2, 2, 2, 2, 3, 6, 6, 7])), ('p1', 80, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 312, 312, 312, 312, 456, 72, 72, 72])), ('p1', 84, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 312, 312, 456, 72, 72, 72, 72])), ('p1', 88, ('set', [0, 0, 24, 96, 384])), ('p1', 92, ('set', [0, 0, 1, 2, 2, 2, 2, 3, 6, 6, 7])), ('p1', 96, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 360, 360, 360, 360, 408, 72, 72, 72])), ('p1', 100, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 360, 360, 408, 72, 72, 72, 72])), ('p1', 104, ('set', [0, 0, 24, 96, 384])), ('p1', 108, ('set', [0, 0, 1, 2, 2, 2, 2, 3, 6, 6, 7])), ('p1', 112, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 408, 408, 408, 408, 360, 72, 72, 72])), ('p1', 116, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 408, 408, 360, 72, 72, 72, 72])), ('p1', 120, ('set', [0, 0, 24, 96, 384])), ('p1', 124, ('set', [0, 0, 1, 2, 2, 2, 2, 3, 6, 6, 7])), ('p1', 128, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 456, 456, 456, 456, 312, 72, 72, 72])), ('p1', 132, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 456, 456, 312, 72, 72, 72, 72])), ('p1', 136, ('set', [0, 0, 24, 96, 384])), ('p1', 140, ('set', [0, 0, 1, 2, 2, 2, 2, 3, 6, 6, 7])), ('p1', 144, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 504, 504, 504, 504, 264, 72, 72, 72])), ('p1', 148, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 504, 504, 264, 72, 72, 72, 72])), ('p1', 152, ('set', [0, 0, 24, 96, 384])), ('p1', 156, ('set', [0, 0, 1, 2, 2, 2, 2, 3, 6, 6, 7])), ('p1', 160, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 552, 552, 552, 552, 216, 72, 72, 72])), ('p1', 164, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 552, 552, 216, 72, 72, 72, 72])), ('p1', 168, ('set', [0, 0, 24, 96, 384])), ('p1', 172, ('set', [0, 0, 1, 2, 2, 2, 2, 3, 6, 6, 7])), ('p1', 176, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 600, 600, 600, 600, 168, 72, 72, 72])), ('p1', 180, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 600, 600, 168, 72, 72, 72, 72])), ('p1', 184, ('set', [0, 0, 24, 96, 384])), ('p1', 188, ('set', [0, 0, 1, 2, 2, 2, 2, 3, 6, 6, 7])), ('p1', 192, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 648, 648, 648, 648, 120, 72, 72, 72])), ('p1', 196, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 648, 648, 120, 72, 72, 72, 72])), ('p1', 200, ('set', [0, 0, 24, 96, 384])), ('p1', 204, ('set', [0, 0, 1, 2, 2, 2, 2, 3, 6, 6, 7])), ('p1', 208, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 696, 696, 696, 696, 72, 72, 72, 72])), ('p1', 212, ('set', [24, 72, 72, 120, 120, 168, 168, 216, 264, 312, 600, 1200, 2400, 696, 696, 72, 72, 72, 72, 72])), ('p1', 216, ('set', [0, 0, 24, 96, 384])), ('p1', 220, ('set', [0, 0, 1, 2, 2, 2, 2, 3, 6, 6, 7]))]
FILLS = [('c0', 'i//3-2')]
ARGRANGE = [(3, 0, 2), (4, 0, 2)]
SLOTBASE = []
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [0, 0, 1]), (2, [0, 1, 1, 1]), (8, [0, 1, 1]), (1, [0, 1, 2, 3])]
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = [('c3', 0, 'c3', 0, True)]
ANSWERBY = []
ANSWERWIN = [(5, 32), (9, 88)]
CDECL = False
TIB = True
ARGEQ = []
SEQ = [(1, 24, None), (2, 12, [1])]
ANSRANGE = [(3, -1, 4)]
CELLTHIS = []
VTALSO = [('p8', 0)]
SPARSE = []
GTAB = []
RECORD = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
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
INITCALLS = [10774912]
MAXINSNS = 400000
ECXANSWER = []
ECXOUT = []
RING = []
DEREF = [(1, 0, 4, None), (1, 1, 4, None)]
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
    rng = random.Random(3339)
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

# sweep_gen: ["obj", "PathFinder::find_upath@00682f30", "--item", "3339", "--name", "path_finder_find_upath_00682f30", "--win", "16", "--own", "6", "--tib", "--rows", "6000", "--max-insns", "400000", "--cell-win", "0xcae5fc:4096", "--cell-win", "0xc06188:64", "--cell-win", "0xc0618c:112", "--cell-win", "0x7c01002c:16", "--at", "0xe85e40:152", "--at", "0xc0aec0:112", "--at", "0xee14fc:20", "--ptrarg", "0:16", "--stub", "0x46f960:0", "--stub", "0x6b52e0:2", "--stub", "0x687c80:3", "--stub", "0x683770:3", "--stub", "0x687ae0:0", "--stub", "0x6179d0:0", "--stub", "0x5e2cb0:1", "--stub", "0x46e8c0:1", "--stub", "slot:0x14:0", "--stub", "slot:0x40:0", "--place", "p0+0:0:512", "--place", "c2+0x14+0x1c*a[3]:0:12", "--place", "p2+4*a[4]:0:32", "--place", "c5+0x1c*a[3]:0:16", "--place", "p4+4*a[4]:0:16", "--place", "p3+0x18:0:560", "--place", "c6+4:0:512", "--answer-win", "5:32", "--answer-win", "9:88", "--vtat-also", "p8+0", "--alias", "c3+0:c3!", "--fill", "c0:i//3-2", "--wbias", "c1+0:2..14", "--wbias", "c1+4:2..14", "--wbias", "c3+8:5,5,1000", "--wbias", "c6+0:0,0,5", "--wbias", "c6+8:0,1,2,3,8,64", "--wbias", "c6+16:-1,4", "--wbias", "c4+132:0,0,1", "--wbias", "p0+4:0,1,2,3,5,8,16,40", "--wbias", "p0+8:-1,0,1,2,3,4,4,4,5,5,6,7,8,9,12", "--wbias", "p0+12:-1,2,4", "--wbias", "p3+16:407167,407167,407167,407167,407119,407199", "--wbias", "p3+20:407167,407167,407167,407167,407119,407199", "--wbias", "p6+536:0,1,2,2,3", "--wbias", "p9+28:0,0,1,3", "--argrange", "3:0:2", "--argrange", "4:0:2", "--argset", "1!:30,47,48,48,96,96,144,144,192,192,240,288,336,71,73,95,119,121,167,169,215,217,100,130,300,600,624,576,1000,1200,1250,2400,2424,2376", "--argset", "2!:30,47,48,48,96,96,144,144,192,192,240,288,336,71,73,95,119,121,167,169,215,217,100,130,300,600,624,576,1000,1200,1250,2400,2424,2376", "--argset", "5:0,1", "--answers", "0:0,0,1", "--answers", "2:0,1,1,1", "--answer-range", "3:-1:4", "--answers", "8:0,1,1", "--answers", "1:0,1,2,3", "--seq", "1:24", "--seq", "2:12:1", "--deref", "1:0:4", "--deref", "1:1:4", "--record", "0", "--record", "1", "--record", "2", "--record", "3", "--record", "4", "--record", "5", "--record", "6", "--record", "7", "--record", "8", "--record", "9", "--record-order", "--init-call", "0xa46980", "--port", "{\n                // **The port is the crate's own steps**,\n                // `crate::upath::find_upath_steps`, which `Sim::upath` runs over\n                // its `SimUpath`; here the view is the emulated original's\n                // windows.\n                //\n                // `a`: the six stack arguments (the `Stack<PathData>*`, the\n                // from-position `x`, `y`, the unit class and slot, `anti`), the\n                // ten stubs' single answers (`a[6..16]`: 0 `can_transport`, 3\n                // `astar_path`, 8 the slot `+0x14`; 5 and 9 the order windows'\n                // addresses; the others unused), stub 1's twenty-four\n                // `get_tregion` answers (`a[16..40]`) and stub 2's twelve\n                // `valid_ucoord` answers (`a[40..52]`). Stubs: 0\n                // `UnitData::can_transport`, 1 `get_tregion` (its two arguments\n                // are locals; the cell pair is printed after the record), 2\n                // `valid_ucoord`, 3 `astar_path`, 4 `kill_lists`, 5\n                // `Unit::update_order`, 6 `kill_current_order`, 7\n                // `Stack<PathData>::increase_size` (the native push and pop run\n                // in the emulator; this view writes the same bytes), 8/9 the\n                // slots `+0x14`, `+0x40`. Windows: `c0` `div_3_table`, `c1` the\n                // world (`+0/+4` its extent in 16-tile blocks), `c2` the\n                // objects singleton, `c3` the thread-local block (`+8` the init\n                // epoch), `c4` the pathfinder (`+0x54` the unit, `+0x58/+0x5c`\n                // the start tile, `+0x64`, `+0x68/+0x6c`, `+0x74`, `+0x84` the\n                // `saving` flag, `+0x94`), `c5` the units singleton, `c6` the\n                // function-static `keep` (`+0` its init guard held at or below\n                // the epoch, so the init arm is not reached; `+4` list, `+8`\n                // size, `+0xc` length, `+0x10` increment); `p0` the stack\n                // (`+0` list, `+4` size, `+8` length, `+0xc` increment), `p1`\n                // its list, `p2` the objects row, `p3` the object (`+0x10/+0x14`\n                // the position xor `0x63637`, `+0x18` the type), `p4` the units\n                // row, `p5` the unit, `p6` the type (`+0x218` domain), `p7`\n                // `keep`'s list, `p8` the order `update_order` answers, `p9` the\n                // `MoveOrder` its slot `+0x40` answers (`+0x1c` retry).\n                use crate::Pos;\n                use crate::orders::PathData;\n                use crate::upath::{UpathView, find_upath_steps};\n                fn rd32(w: &[u8], o: usize) -> i32 {\n                    i32::from_le_bytes(w[o..o + 4].try_into().expect(\"four bytes\"))\n                }\n                fn wr32(w: &mut [u8], o: usize, v: i32) {\n                    w[o..o + 4].copy_from_slice(&v.to_le_bytes());\n                }\n                fn hex(v: i32) -> String {\n                    v.to_le_bytes().iter().map(|b| format!(\"{b:02x}\")).collect()\n                }\n                fn ent(l: &[u8], k: i32) -> PathData {\n                    let o = 16 * k as usize;\n                    PathData {\n                        to: Pos::new(rd32(l, o), rd32(l, o + 4)),\n                        tolerance: rd32(l, o + 8),\n                        flags: rd32(l, o + 12) as u8,\n                    }\n                }\n                fn s_pop(h: &mut [u8], b: usize, l: &[u8]) -> PathData {\n                    let len = rd32(h, b + 8).max(1) - 1;\n                    wr32(h, b + 8, len);\n                    ent(l, len)\n                }\n                fn s_peek(h: &[u8], b: usize, l: &[u8]) -> PathData {\n                    let len = rd32(h, b + 8);\n                    ent(l, if len < 1 { 0 } else { len - 1 })\n                }\n                fn s_push(\n                    h: &mut [u8],\n                    b: usize,\n                    l: &mut [u8],\n                    e: PathData,\n                    log: &mut Vec<String>,\n                    name: &str,\n                ) {\n                    let len = rd32(h, b + 8);\n                    if len >= rd32(h, b + 4) {\n                        log.push(format!(\"s7:{name},{}\", h[b + 12] as i8));\n                    }\n                    let o = 16 * len as usize;\n                    wr32(l, o, e.to.x);\n                    wr32(l, o + 4, e.to.y);\n                    wr32(l, o + 8, e.tolerance);\n                    wr32(l, o + 12, i32::from(e.flags));\n                    wr32(h, b + 8, len + 1);\n                }\n                struct V<'a> {\n                    a: &'a [i32],\n                    c: &'a mut [Vec<u8>],\n                    p: &'a mut [Vec<u8>],\n                    log: &'a mut Vec<String>,\n                    tiles_x: Vec<String>,\n                    tiles_y: Vec<String>,\n                    tregions: usize,\n                    valids: usize,\n                }\n                impl UpathView for V<'_> {\n                    fn begin(&mut self, here: Pos) {\n                        let unit = self.p[4][4 * self.a[4] as usize..][..4].to_vec();\n                        self.c[4][0x54..0x58].copy_from_slice(&unit);\n                        let sx = rd32(&self.c[0], 4 * (here.x >> 6) as usize);\n                        let sy = rd32(&self.c[0], 4 * (here.y >> 6) as usize);\n                        wr32(&mut self.c[4], 0x58, sx);\n                        wr32(&mut self.c[4], 0x5c, sy);\n                        wr32(&mut self.c[4], 0x68, 0);\n                        wr32(&mut self.c[4], 0x6c, 0);\n                        wr32(&mut self.c[4], 0x94, 0);\n                    }\n                    fn saving(&self) -> bool {\n                        rd32(&self.c[4], 0x84) != 0\n                    }\n                    fn cell48(&self, c: i32) -> i32 {\n                        rd32(&self.c[0], 4 * (c >> 4) as usize)\n                    }\n                    fn world_dims(&self) -> (i32, i32) {\n                        (rd32(&self.c[1], 0), rd32(&self.c[1], 4))\n                    }\n                    fn pop(&mut self) -> Option<PathData> {\n                        let (h, l) = self.p.split_at_mut(1);\n                        Some(s_pop(&mut h[0], 0, &l[0]))\n                    }\n                    fn peek(&mut self) -> PathData {\n                        s_peek(&self.p[0], 0, &self.p[1])\n                    }\n                    fn push(&mut self, e: PathData) {\n                        let (h, l) = self.p.split_at_mut(1);\n                        s_push(&mut h[0], 0, &mut l[0], e, self.log, \"p0+0x0\");\n                    }\n                    fn clear(&mut self) {\n                        wr32(&mut self.p[0], 8, 0);\n                    }\n                    fn depth(&self) -> i32 {\n                        rd32(&self.p[0], 8)\n                    }\n                    fn domain_below_2(&self) -> bool {\n                        rd32(&self.p[6], 0x218) < 2\n                    }\n                    fn can_transport(&mut self) -> bool {\n                        self.log.push(\"s0:p5+0x0\".to_string());\n                        self.a[6] != 0\n                    }\n                    fn tregion(&mut self, cx4: i32, cy4: i32) -> i32 {\n                        // The goal's pair first, then the start's: two different\n                        // locals.\n                        if self.tregions.is_multiple_of(2) {\n                            self.log.push(\"s1:c1+0x0,sp-0x48,sp-0x44\".to_string());\n                        } else {\n                            self.log.push(\"s1:c1+0x0,sp-0x50,sp-0x4c\".to_string());\n                        }\n                        let v = self.a[16 + self.tregions.min(23)];\n                        self.tregions += 1;\n                        self.tiles_x.push(hex(cx4));\n                        self.tiles_y.push(hex(cy4));\n                        v\n                    }\n                    fn valid_ucoord(&mut self, p: Pos, metric: i32) -> bool {\n                        self.log\n                            .push(format!(\"s2:c4+0x0,{},{},{}\", p.x, p.y, metric));\n                        let v = self.a[40 + self.valids.min(11)];\n                        self.valids += 1;\n                        v != 0\n                    }\n                    fn search(&mut self, anti: i32) -> i32 {\n                        wr32(&mut self.c[4], 0x68, 0);\n                        wr32(&mut self.c[4], 0x6c, 0);\n                        wr32(&mut self.c[4], 0x64, 1);\n                        self.log.push(format!(\"s3:c4+0x0,p0+0x0,48,{anti}\"));\n                        wr32(&mut self.c[4], 0x74, 0);\n                        wr32(&mut self.c[4], 0x64, 0);\n                        self.a[9]\n                    }\n                    fn suspended(&self, _r: i32) -> bool {\n                        rd32(&self.c[4], 0x84) != 0\n                    }\n                    fn drop_unfinal_top(&mut self) {\n                        let len = rd32(&self.p[0], 8);\n                        let k = if len >= 1 { (len - 1) as usize * 16 } else { 0 };\n                        if rd32(&self.p[1], k + 12) & 1 == 0 {\n                            wr32(&mut self.p[0], 8, len.max(1) - 1);\n                        }\n                    }\n                    fn order_is_move(&mut self) -> bool {\n                        self.log.push(\"s5:p5+0x0\".to_string());\n                        self.log.push(\"s8:p8+0x0\".to_string());\n                        self.a[14] != 0\n                    }\n                    fn move_retry(&mut self) -> i32 {\n                        self.log.push(\"s5:p5+0x0\".to_string());\n                        self.log.push(\"s9:p8+0x0\".to_string());\n                        rd32(&self.p[9], 0x1c)\n                    }\n                    fn kill_current_order(&mut self) {\n                        self.log.push(\"s6:p5+0x0,0\".to_string());\n                    }\n                    fn kill_lists(&mut self) {\n                        self.log.push(\"s4:c4+0x0\".to_string());\n                    }\n                    fn unit_pos(&self) -> Pos {\n                        Pos::new(\n                            rd32(&self.p[3], 0x10) ^ 0x63637,\n                            rd32(&self.p[3], 0x14) ^ 0x63637,\n                        )\n                    }\n                    fn keep_begin(&mut self) {\n                        wr32(&mut self.c[6], 0xc, 0);\n                    }\n                    fn keep_push(&mut self, e: PathData) {\n                        s_push(\n                            &mut self.c[6],\n                            4,\n                            &mut self.p[7],\n                            e,\n                            self.log,\n                            \"c6+0x4\",\n                        );\n                    }\n                    fn keep_pop(&mut self) -> Option<PathData> {\n                        if rd32(&self.c[6], 0xc) == 0 {\n                            return None;\n                        }\n                        Some(s_pop(&mut self.c[6], 4, &self.p[7]))\n                    }\n                }\n                assert_eq!(a.len(), 52, \"arguments and answers\");\n                let mut v = V {\n                    a,\n                    c,\n                    p,\n                    log: &mut log,\n                    tiles_x: Vec::new(),\n                    tiles_y: Vec::new(),\n                    tregions: 0,\n                    valids: 0,\n                };\n                let r = find_upath_steps(&mut v, Pos::new(a[1], a[2]), a[5]);\n                let col = |w: &Vec<String>| {\n                    if w.is_empty() {\n                        \"-\".to_string()\n                    } else {\n                        w.join(\"/\")\n                    }\n                };\n                let (x, y) = (col(&v.tiles_x), col(&v.tiles_y));\n                // No call recorded at all is a `-` column of its own.\n                if v.log.is_empty() {\n                    v.log.push(\"-\".to_string());\n                }\n                v.log.push(x);\n                v.log.push(y);\n                i64::from(r)\n            }", "--wbias", "p1+0:24,72,72,120,120,168,168,216,264,312,600,1200,2400,72,72,72,72,696,72,72,72", "--wbias", "p1+4:24,72,72,120,120,168,168,216,264,312,600,1200,2400,72,72,696,72,72,72,72", "--wbias", "p1+8:0,0,24,96,384", "--wbias", "p1+12:0,0,1,2,2,2,2,3,6,6,7", "--wbias", "p1+16:24,72,72,120,120,168,168,216,264,312,600,1200,2400,120,120,120,120,648,72,72,72", "--wbias", "p1+20:24,72,72,120,120,168,168,216,264,312,600,1200,2400,120,120,648,72,72,72,72", "--wbias", "p1+24:0,0,24,96,384", "--wbias", "p1+28:0,0,1,2,2,2,2,3,6,6,7", "--wbias", "p1+32:24,72,72,120,120,168,168,216,264,312,600,1200,2400,168,168,168,168,600,72,72,72", "--wbias", "p1+36:24,72,72,120,120,168,168,216,264,312,600,1200,2400,168,168,600,72,72,72,72", "--wbias", "p1+40:0,0,24,96,384", "--wbias", "p1+44:0,0,1,2,2,2,2,3,6,6,7", "--wbias", "p1+48:24,72,72,120,120,168,168,216,264,312,600,1200,2400,216,216,216,216,552,72,72,72", "--wbias", "p1+52:24,72,72,120,120,168,168,216,264,312,600,1200,2400,216,216,552,72,72,72,72", "--wbias", "p1+56:0,0,24,96,384", "--wbias", "p1+60:0,0,1,2,2,2,2,3,6,6,7", "--wbias", "p1+64:24,72,72,120,120,168,168,216,264,312,600,1200,2400,264,264,264,264,504,72,72,72", "--wbias", "p1+68:24,72,72,120,120,168,168,216,264,312,600,1200,2400,264,264,504,72,72,72,72", "--wbias", "p1+72:0,0,24,96,384", "--wbias", "p1+76:0,0,1,2,2,2,2,3,6,6,7", "--wbias", "p1+80:24,72,72,120,120,168,168,216,264,312,600,1200,2400,312,312,312,312,456,72,72,72", "--wbias", "p1+84:24,72,72,120,120,168,168,216,264,312,600,1200,2400,312,312,456,72,72,72,72", "--wbias", "p1+88:0,0,24,96,384", "--wbias", "p1+92:0,0,1,2,2,2,2,3,6,6,7", "--wbias", "p1+96:24,72,72,120,120,168,168,216,264,312,600,1200,2400,360,360,360,360,408,72,72,72", "--wbias", "p1+100:24,72,72,120,120,168,168,216,264,312,600,1200,2400,360,360,408,72,72,72,72", "--wbias", "p1+104:0,0,24,96,384", "--wbias", "p1+108:0,0,1,2,2,2,2,3,6,6,7", "--wbias", "p1+112:24,72,72,120,120,168,168,216,264,312,600,1200,2400,408,408,408,408,360,72,72,72", "--wbias", "p1+116:24,72,72,120,120,168,168,216,264,312,600,1200,2400,408,408,360,72,72,72,72", "--wbias", "p1+120:0,0,24,96,384", "--wbias", "p1+124:0,0,1,2,2,2,2,3,6,6,7", "--wbias", "p1+128:24,72,72,120,120,168,168,216,264,312,600,1200,2400,456,456,456,456,312,72,72,72", "--wbias", "p1+132:24,72,72,120,120,168,168,216,264,312,600,1200,2400,456,456,312,72,72,72,72", "--wbias", "p1+136:0,0,24,96,384", "--wbias", "p1+140:0,0,1,2,2,2,2,3,6,6,7", "--wbias", "p1+144:24,72,72,120,120,168,168,216,264,312,600,1200,2400,504,504,504,504,264,72,72,72", "--wbias", "p1+148:24,72,72,120,120,168,168,216,264,312,600,1200,2400,504,504,264,72,72,72,72", "--wbias", "p1+152:0,0,24,96,384", "--wbias", "p1+156:0,0,1,2,2,2,2,3,6,6,7", "--wbias", "p1+160:24,72,72,120,120,168,168,216,264,312,600,1200,2400,552,552,552,552,216,72,72,72", "--wbias", "p1+164:24,72,72,120,120,168,168,216,264,312,600,1200,2400,552,552,216,72,72,72,72", "--wbias", "p1+168:0,0,24,96,384", "--wbias", "p1+172:0,0,1,2,2,2,2,3,6,6,7", "--wbias", "p1+176:24,72,72,120,120,168,168,216,264,312,600,1200,2400,600,600,600,600,168,72,72,72", "--wbias", "p1+180:24,72,72,120,120,168,168,216,264,312,600,1200,2400,600,600,168,72,72,72,72", "--wbias", "p1+184:0,0,24,96,384", "--wbias", "p1+188:0,0,1,2,2,2,2,3,6,6,7", "--wbias", "p1+192:24,72,72,120,120,168,168,216,264,312,600,1200,2400,648,648,648,648,120,72,72,72", "--wbias", "p1+196:24,72,72,120,120,168,168,216,264,312,600,1200,2400,648,648,120,72,72,72,72", "--wbias", "p1+200:0,0,24,96,384", "--wbias", "p1+204:0,0,1,2,2,2,2,3,6,6,7", "--wbias", "p1+208:24,72,72,120,120,168,168,216,264,312,600,1200,2400,696,696,696,696,72,72,72,72", "--wbias", "p1+212:24,72,72,120,120,168,168,216,264,312,600,1200,2400,696,696,72,72,72,72,72", "--wbias", "p1+216:0,0,24,96,384", "--wbias", "p1+220:0,0,1,2,2,2,2,3,6,6,7"]
