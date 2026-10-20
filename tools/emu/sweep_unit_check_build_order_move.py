#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_check_build_order_move.py — `Unit::check_build_order@00603470` under unicorn (item 3372, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_unit_check_build_order_move.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 256-byte object of random bytes (no bias), 0 stack argument(s);
`p` windows in order: p0..p25 the place(s), in flag order, p26..p36 the stubs' answer window(s); the global cell 0x00c0618c points at 768 random bytes; the global cell 0x00c061f0 points at 256 random bytes; vtable slot +0x10 (0 args) stubbed; vtable slot +0xb4 (0 args) stubbed; vtable slot +0xb8 (0 args) stubbed; vtable slot +0x0 (0 args) stubbed; vtable slot +0x4 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0x100 (0 args) stubbed; vtable slot +0xa8 (0 args) stubbed; vtable slot +0xd8 (0 args) stubbed; callee 0x005e29b0 (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x005e5210 (4 args) stubbed; callee 0x0065a620 (11 args) stubbed; callee 0x00608450 (0 args) stubbed; the import cell 0x00ac54f0 holds a stub (1 args); the import cell 0x00ac5500 holds a stub (1 args); callee 0x0055e0ac (3 args) stubbed; the word at `c0+0x14` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p0+8` points at 64 random bytes less `0` (placed); the word at `c0+0x214` points at 12 random bytes less `0` (placed); the word at `p4+0` points at 32 random bytes less `0` (placed); the word at `p4+4` points at 32 random bytes less `0` (placed); the word at `p4+8` points at 32 random bytes less `0` (placed); the word at `m+0xf8` points at 12 random bytes less `0` (placed); the word at `p8+0` points at 32 random bytes less `0` (placed); the word at `p8+4` points at 32 random bytes less `0` (placed); the word at `p8+8` points at 32 random bytes less `0` (placed); the word at `m+0xfc` points at 8 random bytes less `0` (placed); the word at `p12+0` points at 32 random bytes less `0` (placed); the word at `p12+4` points at 32 random bytes less `0` (placed); the word at `m+0xdc` points at 32 random bytes less `0` (placed); the word at `p15+4` points at 32 random bytes less `0` (placed); the word at `p16+4` points at 32 random bytes less `0` (placed); the word at `p17+4` points at 32 random bytes less `0` (placed); the word at `p15+8` points at 64 random bytes less `0` (placed); the word at `p16+8` points at 64 random bytes less `0` (placed); the word at `p17+8` points at 64 random bytes less `0` (placed); the word at `p18+8` points at 64 random bytes less `0` (placed); the word at `m+0xf0` points at 16 random bytes less `0` (placed); the word at `m+0xf4` points at 32 random bytes less `0` (placed); stub 3's answer points at 64 random bytes less `0` (placed); slot +0x0 laid as 0x0072ff60 or its stub, a flag after the answers; slot +0x4 laid as 0x0072ff10 or its stub, a flag after the answers; stub 0 answered from [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 6, 6, 3, 0] (90%); stub 4 answered from [1] (90%); stub 5 answered from [0] (90%); stub 6 answered from [0] (90%); stub 7 answered from [0] (90%); stub 2 answers the address of 32 random bytes, a `p` window after the places; stub 1 answers a window of its own to each call, in call order (64 bytes, ...); stub 9 answers a window of its own to each call, in call order (32 bytes, ...); the word at `p18+4` laid as the address of `p15` every row; the word at `p1+0x18` laid as the address of `p24` every row; the word at `p2+0x18` laid as the address of `p24` every row; the word at `p3+0x18` laid as the address of `p24` every row; the word at `p25+0x18` laid as the address of `p24` every row; the word at `p32+0` laid as the address of `p23` every row; stub 8 answers `p9+j`, the data of the object its `ecx` is in `p5+j` (j < 3), linked at `+0x1c`; stub 14 answers `p13+j`, the data of the object its `ecx` is in `p9+j` (j < 2), linked at `+0x1c`; stub 0 answers 60 values in call order, the last from [0, 2], printed after the answers; stub 4 answers 12 values in call order, printed after the answers; stub 5 answers 12 values in call order, printed after the answers; stub 6 answers 12 values in call order, printed after the answers; stub 7 answers 12 values in call order, printed after the answers; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 is `__cdecl`: the caller pops its arguments; stub 16 is `__cdecl`: the caller pops its arguments; stub 17 is `__cdecl`: the caller pops its arguments; stub 15 answers a fresh bump-heap address of its argument 0 bytes (a `malloc`); stub 17 copies its argument 2 bytes from argument 1 to argument 0 (a `memcpy`); the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`.
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

ENTRY = 0x00603470
WIN = 256
OWN = 0
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607884, 768), (12607984, 256)]
AT = []
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('slot', 16, 0), ('slot', 180, 0), ('slot', 184, 0), ('slot', 0, 0), ('slot', 4, 0), ('slot', 76, 0), ('slot', 252, 0), ('slot', 256, 0), ('slot', 168, 0), ('slot', 216, 0), ('call', 6171056, 0), ('call', 6171824, 1), ('call', 6181392, 4), ('call', 6661664, 11), ('call', 6325328, 0), ('iat', 11293936, 1), ('iat', 11293952, 1), ('call', 5628076, 3)]
PLACES = [('c0', '0x14', '0', 12), ('p0', '0', '0', 64), ('p0', '4', '0', 64), ('p0', '8', '0', 64), ('c0', '0x214', '0', 12), ('p4', '0', '0', 32), ('p4', '4', '0', 32), ('p4', '8', '0', 32), ('m', '0xf8', '0', 12), ('p8', '0', '0', 32), ('p8', '4', '0', 32), ('p8', '8', '0', 32), ('m', '0xfc', '0', 8), ('p12', '0', '0', 32), ('p12', '4', '0', 32), ('m', '0xdc', '0', 32), ('p15', '4', '0', 32), ('p16', '4', '0', 32), ('p17', '4', '0', 32), ('p15', '8', '0', 64), ('p16', '8', '0', 64), ('p17', '8', '0', 64), ('p18', '8', '0', 64), ('m', '0xf0', '0', 16), ('m', '0xf4', '0', 32), ('s3', '0', '0', 64)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 216, ('set', [1, 2, 3, 4, 5, 6])), ('m', 8, ('set', [65536, 65792, 66048, 131072, 131328, 131584])), ('p23', 4, ('set', [0, 4])), ('p1', 8, ('set', [6553601, 13107201, 19660801])), ('p1', 48, ('set', [0])), ('p2', 8, ('set', [6553601, 13107201, 19660801])), ('p2', 48, ('set', [0])), ('p3', 8, ('set', [6553601, 13107201, 19660801])), ('p3', 48, ('set', [0])), ('p25', 8, ('set', [6553601, 13107201, 19660801])), ('p25', 48, ('set', [0])), ('p26', 8, ('set', [0, 1, 2])), ('p26', 12, ('set', [0])), ('p26', 16, ('set', [0])), ('p27', 8, ('set', [0, 1, 2])), ('p27', 12, ('set', [0])), ('p27', 16, ('set', [0])), ('p28', 8, ('set', [0, 1, 2])), ('p28', 12, ('set', [0])), ('p28', 16, ('set', [0])), ('p29', 8, ('set', [0, 1, 2])), ('p29', 12, ('set', [0])), ('p29', 16, ('set', [0])), ('p30', 8, ('set', [0, 1, 2])), ('p30', 12, ('set', [0])), ('p30', 16, ('set', [0])), ('p31', 8, ('set', [0, 1, 2])), ('p31', 12, ('set', [0])), ('p31', 16, ('set', [0])), ('c0', 520, ('set', [1, 2, 3])), ('p9', 8, ('set', [65536, 131072, 196608])), ('p10', 8, ('set', [65536, 131072, 196608])), ('p11', 8, ('set', [65536, 131072, 196608])), ('p33', 8, ('set', [100, 200, 300])), ('p33', 12, ('set', [0, 1, 2])), ('p34', 8, ('set', [100, 200, 300])), ('p34', 12, ('set', [0, 1, 2])), ('p35', 8, ('set', [100, 200, 300])), ('p35', 12, ('set', [0, 1, 2])), ('p36', 8, ('set', [100, 200, 300])), ('p36', 12, ('set', [0, 1, 2]))]
FILLS = []
ARGRANGE = []
SLOTBASE = [(0, 7536480), (4, 7536400)]
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 6, 6, 3, 0]), (4, [1]), (5, [0]), (6, [0]), (7, [0])]
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = [('p18', 4, 'p15', 0, True), ('p1', 24, 'p24', 0, True), ('p2', 24, 'p24', 0, True), ('p3', 24, 'p24', 0, True), ('p25', 24, 'p24', 0, True), ('p32', 0, 'p23', 0, True)]
ANSWERBY = [(8, 5, 9, 3, 28), (14, 9, 13, 2, 28)]
ANSWERWIN = [(1, 64), (1, 64), (1, 64), (1, 64), (1, 64), (1, 64), (2, 32), (9, 32), (9, 32), (9, 32), (9, 32)]
CDECL = False
TIB = True
ARGEQ = []
SEQ = [(0, 60, [0, 2]), (4, 12, None), (5, 12, None), (6, 12, None), (7, 12, None)]
ANSRANGE = []
CELLTHIS = []
VTALSO = [('p19', 0), ('p20', 0), ('p21', 0), ('p22', 0), ('p24', 0), ('p25', 0), ('p26', 0), ('p27', 0), ('p28', 0), ('p29', 0), ('p30', 0), ('p31', 0), ('p1', 0), ('p2', 0), ('p3', 0), ('p5', 0), ('p6', 0), ('p7', 0), ('p13', 0), ('p14', 0)]
SPARSE = []
GTAB = []
RECORD = [10, 11, 12, 13]
RECORDORDER = False
RECORDEDX = False
ROWSPLIT = {}
LINKS = []
LINKHI = []
OUTARG = []
CDECLSTUBS = [15, 16, 17]
STUBWRITE = []
STUBCOPY = [17]
STUBALLOC = [15]
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

# sweep_gen: ["obj", "Unit::check_build_order@00603470", "--item", "3372", "--win", "0x100", "--ret", "none", "--tib", "--rows", "2000", "--name", "unit_check_build_order_move", "--cell-win", "0xc0618c:0x300", "--cell-win", "0xc061f0:0x100", "--stub", "slot:0x10:0", "--stub", "slot:0xb4:0", "--stub", "slot:0xb8:0", "--stub", "slot:0x0:0", "--stub", "slot:0x4:0", "--stub", "slot:0x4c:0", "--stub", "slot:0xfc:0", "--stub", "slot:0x100:0", "--stub", "slot:0xa8:0", "--stub", "slot:0xd8:0", "--stub", "0x5e29b0:0", "--stub", "0x5e2cb0:1", "--stub", "0x5e5210:4", "--stub", "0x65a620:11", "--stub", "0x608450:0", "--stub", "iat:0xac54f0:1", "--stub", "iat:0xac5500:1", "--stub", "0x55e0ac:3", "--stub-cdecl", "15", "--stub-cdecl", "16", "--stub-cdecl", "17", "--stub-alloc", "15", "--stub-copy", "17", "--record", "10", "--record", "11", "--record", "12", "--record", "13", "--vtat-keep", "0:0x72ff60", "--vtat-keep", "4:0x72ff10", "--ptrtab", "c0+0x14:3:0x40:vt", "--ptrtab", "c0+0x214:3:0x20:vt", "--ptrtab", "m+0xf8:3:0x20", "--ptrtab", "m+0xfc:2:0x20:vt", "--answer-by", "8:5:9:3:0x1c", "--answer-by", "14:9:13:2:0x1c", "--place", "m+0xdc:0:0x20", "--place", "p15+4:0:0x20", "--place", "p16+4:0:0x20", "--place", "p17+4:0:0x20", "--place", "p15+8:0:0x40", "--place", "p16+8:0:0x40", "--place", "p17+8:0:0x40", "--place", "p18+8:0:0x40", "--place", "m+0xf0:0:0x10", "--place", "m+0xf4:0:0x20", "--place", "s3:0:0x40", "--alias", "p18+4:p15!", "--vtat-also", "p19+0", "--vtat-also", "p20+0", "--vtat-also", "p21+0", "--vtat-also", "p22+0", "--vtat-also", "p24+0", "--vtat-also", "p25+0", "--alias", "p1+0x18:p24!", "--alias", "p2+0x18:p24!", "--alias", "p3+0x18:p24!", "--alias", "p25+0x18:p24!", "--alias", "p32+0:p23!", "--answer-win", "1:0x40", "--answer-win", "1:0x40", "--answer-win", "1:0x40", "--answer-win", "1:0x40", "--answer-win", "1:0x40", "--answer-win", "1:0x40", "--answer-win", "2:0x20", "--answer-win", "9:0x20", "--answer-win", "9:0x20", "--answer-win", "9:0x20", "--answer-win", "9:0x20", "--vtat-also", "p26+0", "--vtat-also", "p27+0", "--vtat-also", "p28+0", "--vtat-also", "p29+0", "--vtat-also", "p30+0", "--vtat-also", "p31+0", "--wbias", "m+0xd8:1,2,3,4,5,6", "--wbias", "m+8:0x10000,0x10100,0x10200,0x20000,0x20100,0x20200", "--wbias", "p23+4:0,4", "--wbias", "p1+8:0x640001,0xc80001,0x12c0001", "--wbias", "p1+0x30:0", "--wbias", "p2+8:0x640001,0xc80001,0x12c0001", "--wbias", "p2+0x30:0", "--wbias", "p3+8:0x640001,0xc80001,0x12c0001", "--wbias", "p3+0x30:0", "--wbias", "p25+8:0x640001,0xc80001,0x12c0001", "--wbias", "p25+0x30:0", "--wbias", "p26+8:0,1,2", "--wbias", "p26+0xc:0", "--wbias", "p26+0x10:0", "--wbias", "p27+8:0,1,2", "--wbias", "p27+0xc:0", "--wbias", "p27+0x10:0", "--wbias", "p28+8:0,1,2", "--wbias", "p28+0xc:0", "--wbias", "p28+0x10:0", "--wbias", "p29+8:0,1,2", "--wbias", "p29+0xc:0", "--wbias", "p29+0x10:0", "--wbias", "p30+8:0,1,2", "--wbias", "p30+0xc:0", "--wbias", "p30+0x10:0", "--wbias", "p31+8:0,1,2", "--wbias", "p31+0xc:0", "--wbias", "p31+0x10:0", "--wbias", "c0+0x208:1,2,3", "--wbias", "p9+8:0x10000,0x20000,0x30000", "--wbias", "p10+8:0x10000,0x20000,0x30000", "--wbias", "p11+8:0x10000,0x20000,0x30000", "--wbias", "p33+8:100,200,300", "--wbias", "p33+0xc:0,1,2", "--wbias", "p34+8:100,200,300", "--wbias", "p34+0xc:0,1,2", "--wbias", "p35+8:100,200,300", "--wbias", "p35+0xc:0,1,2", "--wbias", "p36+8:100,200,300", "--wbias", "p36+0xc:0,1,2", "--seq", "0:60:0,2", "--answers", "0:1,1,1,1,1,1,1,1,1,1,6,6,3,0", "--seq", "4:12", "--answers", "4:1", "--seq", "5:12", "--answers", "5:0", "--seq", "6:12", "--answers", "6:0", "--seq", "7:12", "--answers", "7:0", "--port", "{\n                // Windows (sweep_gen's flag order): `m` the unit; `c0` the objects record (`0xc0618c`: `+0x14` the\n                // list of uid 0's objects, `+0x208` the found count, `+0x214` the found list), `c1` the constants\n                // (`+0x24` `unit_build_respond_range`). `p0` the uid-0 list, `p1..p3` its objects; `p4` the found\n                // list, `p5..p7` its elements; `p8` the unit-data list, `p9..p11` the unit data (`answer-by` of the\n                // elements); `p12` the action list, `p13..p14` the actions of unit data 0 and 1 (the third has none);\n                // `p15..p18` the order list's end node and three more, in `next` (`+4`) order, ring-closed by an\n                // alias; `p19..p22` their orders; `p23` the vbase record (`+4` the action flag's offset), `p24` the\n                // site type, `p25` the object `get_target`'s slot answers; `p26..p31` the six target orders a call\n                // answers in turn (`get_order_target`, the last repeated); `p32` the object slot `+0xb8` answers;\n                // `p33..p36` the records slot `+0xd8` answers in turn. The answers: `a[0..18]` the stubs' single\n                // answers, `a[18..78]` stub 0's (`get_type`) sixty by call order, `a[78..90]` stub 4's\n                // (`target_exists`), `a[90..102]` stub 5's (`+0x4c`), `a[102..114]` stub 6's (`+0xfc`),\n                // `a[114..126]` stub 7's (`+0x100`); `a[126]` 1 when the image's `TargetOrder::get_target` stands in\n                // slot 0, `a[127]` 1 when its `target_exists` stands in slot 4.\n                use crate::check_build_view::{CheckBuildView, check_build_order_steps};\n                fn r32(w: &[u8], o: usize) -> i32 {\n                    i32::from_le_bytes(w[o..o + 4].try_into().expect(\"four bytes\"))\n                }\n                fn w32(w: &mut [u8], o: usize, v: i32) {\n                    w[o..o + 4].copy_from_slice(&v.to_le_bytes());\n                }\n                fn r16(w: &[u8], o: usize) -> i32 {\n                    i32::from(i16::from_le_bytes([w[o], w[o + 1]]))\n                }\n                struct V<'a> {\n                    m: &'a mut [u8],\n                    c: &'a mut [Vec<u8>],\n                    p: &'a mut [Vec<u8>],\n                    a: &'a [i32],\n                    /// The recorded calls: repath, kill, add_build_order, find_units.\n                    t: Vec<Vec<String>>,\n                    /// The cursor's node (0 the end node, 1.. the front on).\n                    node: usize,\n                    /// Calls made so far of stubs 0, 4, 5, 6, 7 (by `--seq`), of slot `+0xb4` and of slot `+0xd8`.\n                    seq_at: [usize; 8],\n                    /// The object the last `target_site` named (a `p` window).\n                    obj: usize,\n                    /// The unit data `found_other` last asked (a `p` window).\n                    ud: usize,\n                }\n                impl V<'_> {\n                    /// The next call-order answer of stub `s` (`--seq`), the last repeated.\n                    fn seq(&mut self, s: usize) -> i32 {\n                        let (at, k) = match s {\n                            0 => (18, 0),\n                            4 => (78, 1),\n                            5 => (90, 2),\n                            6 => (102, 3),\n                            7 => (114, 4),\n                            _ => unreachable!(),\n                        };\n                        let len = if s == 0 { 60 } else { 12 };\n                        let n = self.seq_at[k].min(len - 1);\n                        self.seq_at[k] += 1;\n                        self.a[at + n]\n                    }\n                    /// The next target order (slot `+0xb4`): `p26 + call`, the last repeated.\n                    fn target_order(&mut self) -> usize {\n                        let n = self.seq_at[5].min(5);\n                        self.seq_at[5] += 1;\n                        26 + n\n                    }\n                    /// `get_target` of the order window `t`: inline (`TargetOrder::get_target`'s table walk) or the\n                    /// stub's one object.\n                    fn get_target(&mut self, t: usize) -> usize {\n                        if self.a[126] != 0 {\n                            let (idx, uid) = (r32(&self.p[t], 8), r32(&self.p[t], 0xc));\n                            assert!(uid == 0 && (0..3).contains(&idx), \"only uid 0's three objects are laid\");\n                            1 + idx as usize\n                        } else {\n                            25\n                        }\n                    }\n                    fn rec(&mut self, col: usize, s: String) {\n                        self.t[col].push(s);\n                    }\n                    fn node_addr(&self, k: usize) -> i32 {\n                        match k {\n                            0 => r32(self.m, 0xdc),\n                            _ => r32(&self.p[14 + k], 4),\n                        }\n                    }\n                    fn store_cursor(&mut self) {\n                        let k = self.node;\n                        let addr = self.node_addr(k);\n                        w32(self.m, 0xd4, addr);\n                        let ord = r32(&self.p[15 + k], 8);\n                        w32(self.m, 0xcc, ord);\n                        self.m[0xd0] = self.p[15 + k][0xc];\n                    }\n                }\n                impl CheckBuildView for V<'_> {\n                    fn orders_present(&mut self) -> bool {\n                        r32(self.m, 0xd8) != 0\n                    }\n                    fn list_present(&mut self) -> bool {\n                        r32(self.m, 0xdc) != 0\n                    }\n                    fn cursor_to_front(&mut self) {\n                        self.node = 1;\n                        self.store_cursor();\n                    }\n                    fn cursor_advance(&mut self) {\n                        let next = r32(&self.p[15 + self.node], 4);\n                        self.node = (0..4)\n                            .find(|&k| self.node_addr(k) == next)\n                            .expect(\"a node of the ring\");\n                        self.store_cursor();\n                    }\n                    fn has_order(&mut self) -> bool {\n                        r32(self.m, 0xcc) != 0\n                    }\n                    fn order_type(&mut self) -> i32 {\n                        self.seq(0)\n                    }\n                    fn order_is_action(&mut self) -> bool {\n                        let off = r32(&self.p[23], 4) as usize;\n                        self.p[32][off + 4] & 4 != 0\n                    }\n                    fn cursor_at_end(&mut self) -> bool {\n                        r32(self.m, 0xd4) == r32(self.m, 0xdc)\n                    }\n                    fn target_exists(&mut self) -> bool {\n                        let t = self.target_order();\n                        if self.a[127] != 0 {\n                            let (idx, uid) = (r32(&self.p[t], 8), r32(&self.p[t], 0xc));\n                            if idx < 0 || uid < 0 {\n                                return false;\n                            }\n                            assert!(uid == 0 && idx < 3, \"only uid 0's three objects are laid\");\n                            let o = &self.p[1 + idx as usize];\n                            o[8] & 1 != 0 && r16(o, 0x30) == r16(&self.p[t], 0x10)\n                        } else {\n                            self.seq(4) != 0\n                        }\n                    }\n                    fn target_finished(&mut self) -> bool {\n                        let t = self.target_order();\n                        let _ = self.get_target(t);\n                        self.seq(5) != 0\n                    }\n                    fn target_site(&mut self) -> i32 {\n                        let t = self.target_order();\n                        self.obj = self.get_target(t);\n                        r16(&self.p[self.obj], 0xa)\n                    }\n                    fn target_blocks(&mut self) -> bool {\n                        self.p[self.obj][8] & 0x20 != 0 || self.seq(6) != 0 || self.seq(7) != 0\n                    }\n                    fn repath(&mut self) {\n                        self.rec(0, \"m+0x0\".into());\n                    }\n                    fn kill_current_order(&mut self) {\n                        self.rec(1, \"m+0x0,0\".into());\n                    }\n                    fn add_build_order(&mut self, site: i32) {\n                        let owner = i32::from(self.m[9]);\n                        self.rec(2, format!(\"m+0x0,{site},{owner},0,1\"));\n                    }\n                    fn owner(&mut self) -> i32 {\n                        i32::from(self.m[9])\n                    }\n                    fn find_units(&mut self) {\n                        let (x, y) = (r32(self.m, 0x10) ^ 0x63637, r32(self.m, 0x14) ^ 0x63637);\n                        let range = r32(&self.c[1], 0x24).wrapping_mul(3).wrapping_shl(6);\n                        let owner = i32::from(self.m[9]);\n                        self.rec(3, format!(\"{x},{x},{y},1,{owner},{range},512,10,1,0,{x},1\"));\n                    }\n                    fn found_count(&mut self) -> i32 {\n                        r32(&self.c[0], 0x208)\n                    }\n                    fn found_other(&mut self, j: i32) -> bool {\n                        self.ud = 9 + j as usize;\n                        r16(&self.p[self.ud], 0xa) != r16(self.m, 0xa)\n                    }\n                    fn found_builds(&mut self, j: i32) -> bool {\n                        // `answer-by`: unit data 0 and 1 have an action, the third none.\n                        j < 2 && self.seq(0) == 6\n                    }\n                    fn found_target(&mut self, j: i32) -> (i32, i32) {\n                        assert!(j < 2);\n                        let n = self.seq_at[6].min(3);\n                        self.seq_at[6] += 1;\n                        let r = &self.p[33 + n];\n                        (r32(r, 0xc), r32(r, 8))\n                    }\n                }\n                let mut v = V {\n                    m,\n                    c,\n                    p,\n                    a,\n                    t: vec![Vec::new(); 4],\n                    node: 0,\n                    seq_at: [0; 8],\n                    obj: 0,\n                    ud: 0,\n                };\n                check_build_order_steps(&mut v);\n                for col in v.t {\n                    log.push(if col.is_empty() { \"-\".to_string() } else { col.join(\"/\") });\n                }\n                0\n            }"]
