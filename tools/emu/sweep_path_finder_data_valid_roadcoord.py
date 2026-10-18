#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_path_finder_data_valid_roadcoord.py — `PathFinderData::valid_roadcoord@00688740` under unicorn (item 3238, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_path_finder_data_valid_roadcoord.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 112-byte object of random bytes (no bias), 8 stack argument(s);
`p` windows in order: p0..p14 the place(s), in flag order; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; vtable slot +0x3c (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x64 (0 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `m+0x68` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 48 random bytes less `0` (placed); the word at `p5+4` points at 48 random bytes less `0` (placed); the word at `p5+8` points at 48 random bytes less `0` (placed); the word at `p5+12` points at 48 random bytes less `0` (placed); the word at `c0+0x138` points at 288 random bytes less `0` (placed); the word at `p6+0x18` points at 576 random bytes less `0` (placed); the word at `p7+0x18` points at 576 random bytes less `0` (placed); the word at `p8+0x18` points at 576 random bytes less `0` (placed); the word at `p9+0x18` points at 576 random bytes less `0` (placed); slot +0x64 laid as 0x00472bc0 or its stub, a flag after the answers; stub 0 answers `p6+j`, the data of the object its `ecx` is in `p1+j` (j < 4), linked at `+0x1c`; stub 1 answers from its own `ecx`: the word at `+0x200`; stub 2 answers from its own `ecx`: the word at `+0x204`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`.
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

ENTRY = 0x00688740
WIN = 112
OWN = 8
BIAS = []
ARGSET = [(0, [-1, 0, 1, 63, 64, 95, 96, 191, 192, 193, 287, 288, 383, 384, 575, 576, 960, 1535, 1536, 1537, 2303, 2304, 2305]), (1, [-1, 0, 1, 63, 64, 95, 96, 191, 192, 193, 287, 288, 383, 384, 575, 576, 960, 1535, 1536, 1537, 2303, 2304, 2305]), (2, [-1, 0, 1, 63, 64, 95, 96, 191, 192, 193, 287, 288, 383, 384, 575, 576, 960, 1535, 1536, 1537, 2303, 2304, 2305]), (3, [-1, 0, 1, 63, 64, 95, 96, 191, 192, 193, 287, 288, 383, 384, 575, 576, 960, 1535, 1536, 1537, 2303, 2304, 2305])]
ARGFROM = []
CELLS = [(12607952, 320), (13297148, 4096)]
AT = []
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('slot', 60, 0), ('slot', 96, 2), ('slot', 100, 0)]
PLACES = [(None, '0xc0ab84', '0', 16), ('p0', '0', '0', 32), ('p0', '4', '0', 32), ('p0', '8', '0', 32), ('p0', '12', '0', 32), ('m', '0x68', '0', 16), ('p5', '0', '0', 48), ('p5', '4', '0', 48), ('p5', '8', '0', 48), ('p5', '12', '0', 48), ('c0', '0x138', '0', 288), ('p6', '0x18', '0', 576), ('p7', '0x18', '0', 576), ('p8', '0x18', '0', 576), ('p9', '0x18', '0', 576)]
NULLS = []
NULLFORCE = {}
WBIAS = [('c0', 24, ('set', [8, 12])), ('c0', 28, ('set', [8, 12])), ('m', 88, ('set', [0, 0, 1])), ('p10', None, ('set', [0, 3, 16, 19, 32, 35, 48, 51, 127, 128, 131, 144, 147, 16384, 16387, 16400, 16416, 16528, 196608, 196611, 196624, 196627, 196640, 196643, 196656, 196659, 196735, 196736, 196739, 196752, 196755, 212992, 212995, 213008, 213024, 213136, 1048576, 1048579, 1048592, 1048595, 1048608, 1048611, 1048624, 1048627, 1048703, 1048704, 1048707, 1048720, 1048723, 1064960, 1064963, 1064976, 1064992, 1065104, 1245184, 1245187, 1245200, 1245203, 1245216, 1245219, 1245232, 1245235, 1245311, 1245312, 1245315, 1245328, 1245331, 1261568, 1261571, 1261584, 1261600, 1261712, 2097152, 2097155, 2097168, 2097171, 2097184, 2097187, 2097200, 2097203, 2097279, 2097280, 2097283, 2097296, 2097299, 2113536, 2113539, 2113552, 2113568, 2113680, 2293760, 2293763, 2293776, 2293779, 2293792, 2293795, 2293808, 2293811, 2293887, 2293888, 2293891, 2293904, 2293907, 2310144, 2310147, 2310160, 2310176, 2310288, 3145728, 3145731, 3145744, 3145747, 3145760, 3145763, 3145776, 3145779, 3145855, 3145856, 3145859, 3145872, 3145875, 3162112, 3162115, 3162128, 3162144, 3162256, 3342336, 3342339, 3342352, 3342355, 3342368, 3342371, 3342384, 3342387, 3342463, 3342464, 3342467, 3342480, 3342483, 3358720, 3358723, 3358736, 3358752, 3358864, 8323072, 8323075, 8323088, 8323091, 8323104, 8323107, 8323120, 8323123, 8323199, 8323200, 8323203, 8323216, 8323219, 8339456, 8339459, 8339472, 8339488, 8339600, 8388608, 8388611, 8388624, 8388627, 8388640, 8388643, 8388656, 8388659, 8388735, 8388736, 8388739, 8388752, 8388755, 8404992, 8404995, 8405008, 8405024, 8405136, 8585216, 8585219, 8585232, 8585235, 8585248, 8585251, 8585264, 8585267, 8585343, 8585344, 8585347, 8585360, 8585363, 8601600, 8601603, 8601616, 8601632, 8601744, 9437184, 9437187, 9437200, 9437203, 9437216, 9437219, 9437232, 9437235, 9437311, 9437312, 9437315, 9437328, 9437331, 9453568, 9453571, 9453584, 9453600, 9453712, 9633792, 9633795, 9633808, 9633811, 9633824, 9633827, 9633840, 9633843, 9633919, 9633920, 9633923, 9633936, 9633939, 9650176, 9650179, 9650192, 9650208, 9650320, 1073741824, 1073741827, 1073741840, 1073741843, 1073741856, 1073741859, 1073741872, 1073741875, 1073741951, 1073741952, 1073741955, 1073741968, 1073741971, 1073758208, 1073758211, 1073758224, 1073758240, 1073758352, 1073938432, 1073938435, 1073938448, 1073938451, 1073938464, 1073938467, 1073938480, 1073938483, 1073938559, 1073938560, 1073938563, 1073938576, 1073938579, 1073954816, 1073954819, 1073954832, 1073954848, 1073954960, 1074790400, 1074790403, 1074790416, 1074790419, 1074790432, 1074790435, 1074790448, 1074790451, 1074790527, 1074790528, 1074790531, 1074790544, 1074790547, 1074806784, 1074806787, 1074806800, 1074806816, 1074806928, 1075838976, 1075838979, 1075838992, 1075838995, 1075839008, 1075839011, 1075839024, 1075839027, 1075839103, 1075839104, 1075839107, 1075839120, 1075839123, 1075855360, 1075855363, 1075855376, 1075855392, 1075855504, 1083179008, 1083179011, 1083179024, 1083179027, 1083179040, 1083179043, 1083179056, 1083179059, 1083179135, 1083179136, 1083179139, 1083179152, 1083179155, 1083195392, 1083195395, 1083195408, 1083195424, 1083195536])), ('p6', 16, ('set', [407095, 407094, 407048, 407159, 407144, 407127, 407176, 407287, 407286, 407336, 407319, 407368, 407479, 406536, 406647, 407031, 406472, 405559, 405558, 409288, 409399, 409398, 407095, 405991])), ('p6', 20, ('set', [407095, 407094, 407048, 407159, 407144, 407127, 407176, 407287, 407286, 407336, 407319, 407368, 407479, 406536, 406647, 407031, 406472, 405559, 405558, 409288, 409399, 409398, 407095, 405991])), ('p11', 564, ('set', [0, 1, 2, 3, 4, 5, 6, -1])), ('p11', 568, ('set', [0, 1, 2, 3, 4, 5, 6, -1])), ('p11', 512, ('set', [0, 1])), ('p11', 516, ('set', [0, 1])), ('p7', 16, ('set', [407095, 407094, 407048, 407159, 407144, 407127, 407176, 407287, 407286, 407336, 407319, 407368, 407479, 406536, 406647, 407031, 406472, 405559, 405558, 409288, 409399, 409398, 407095, 405991])), ('p7', 20, ('set', [407095, 407094, 407048, 407159, 407144, 407127, 407176, 407287, 407286, 407336, 407319, 407368, 407479, 406536, 406647, 407031, 406472, 405559, 405558, 409288, 409399, 409398, 407095, 405991])), ('p12', 564, ('set', [0, 1, 2, 3, 4, 5, 6, -1])), ('p12', 568, ('set', [0, 1, 2, 3, 4, 5, 6, -1])), ('p12', 512, ('set', [0, 1])), ('p12', 516, ('set', [0, 1])), ('p8', 16, ('set', [407095, 407094, 407048, 407159, 407144, 407127, 407176, 407287, 407286, 407336, 407319, 407368, 407479, 406536, 406647, 407031, 406472, 405559, 405558, 409288, 409399, 409398, 407095, 405991])), ('p8', 20, ('set', [407095, 407094, 407048, 407159, 407144, 407127, 407176, 407287, 407286, 407336, 407319, 407368, 407479, 406536, 406647, 407031, 406472, 405559, 405558, 409288, 409399, 409398, 407095, 405991])), ('p13', 564, ('set', [0, 1, 2, 3, 4, 5, 6, -1])), ('p13', 568, ('set', [0, 1, 2, 3, 4, 5, 6, -1])), ('p13', 512, ('set', [0, 1])), ('p13', 516, ('set', [0, 1])), ('p9', 16, ('set', [407095, 407094, 407048, 407159, 407144, 407127, 407176, 407287, 407286, 407336, 407319, 407368, 407479, 406536, 406647, 407031, 406472, 405559, 405558, 409288, 409399, 409398, 407095, 405991])), ('p9', 20, ('set', [407095, 407094, 407048, 407159, 407144, 407127, 407176, 407287, 407286, 407336, 407319, 407368, 407479, 406536, 406647, 407031, 406472, 405559, 405558, 409288, 409399, 409398, 407095, 405991])), ('p14', 564, ('set', [0, 1, 2, 3, 4, 5, 6, -1])), ('p14', 568, ('set', [0, 1, 2, 3, 4, 5, 6, -1])), ('p14', 512, ('set', [0, 1])), ('p14', 516, ('set', [0, 1]))]
FILLS = [('c1', 'i//3')]
ARGRANGE = [(4, 0, 3), (5, 0, 0), (6, 0, 3), (7, 0, 0)]
SLOTBASE = [(100, 4664256)]
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
ALIAS = []
ANSWERBY = [(0, 1, 6, 4, 28)]
ANSWERWIN = []
CDECL = False
TIB = False
ARGEQ = []
SEQ = []
ANSRANGE = []
CELLTHIS = []
VTALSO = [('p11', 0), ('p12', 0), ('p13', 0), ('p14', 0), ('p1', 0), ('p2', 0), ('p3', 0), ('p4', 0)]
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
ECXANSWER = [(1, 512, None), (2, 516, None)]
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
    rng = random.Random(3238)
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
    for i in range(4000):
        h.brk = base  # the bump heap is reused a row; 4000 rows of windows outgrow it
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

# sweep_gen: ["obj", "PathFinderData::valid_roadcoord@00688740", "--item", "3238", "--name", "path_finder_data_valid_roadcoord", "--item", "3238", "--win", "0x70", "--own", "8", "--cell-win", "0xc061d0:0x140", "--cell-win", "0xcae5fc:4096", "--fill", "c1:i//3", "--ptrtab", "0xc0ab84:4:0x20:vt", "--ptrtab", "m+0x68:4:0x30", "--place", "c0+0x138:0:288", "--place", "p6+0x18:0:0x240", "--place", "p7+0x18:0:0x240", "--place", "p8+0x18:0:0x240", "--place", "p9+0x18:0:0x240", "--vtat-also", "p11+0", "--vtat-also", "p12+0", "--vtat-also", "p13+0", "--vtat-also", "p14+0", "--stub", "slot:0x3c:0", "--stub", "slot:0x60:2", "--stub", "slot:0x64:0", "--answer-by", "0:1:6:4:0x1c", "--ecx-answer", "1:0x200", "--ecx-answer", "2:0x204", "--slotbase", "0x64:0x472bc0", "--argset", "0:-1,0,1,63,64,95,96,191,192,193,287,288,383,384,575,576,960,1535,1536,1537,2303,2304,2305", "--argset", "1:-1,0,1,63,64,95,96,191,192,193,287,288,383,384,575,576,960,1535,1536,1537,2303,2304,2305", "--argset", "2:-1,0,1,63,64,95,96,191,192,193,287,288,383,384,575,576,960,1535,1536,1537,2303,2304,2305", "--argset", "3:-1,0,1,63,64,95,96,191,192,193,287,288,383,384,575,576,960,1535,1536,1537,2303,2304,2305", "--argrange", "4:0:3", "--argrange", "5:0:0", "--argrange", "6:0:3", "--argrange", "7:0:0", "--wbias", "c0+0x18:8,12", "--wbias", "c0+0x1c:8,12", "--wbias", "m+0x58:0,0,1", "--wbias", "p10+*:0,3,16,19,32,35,48,51,127,128,131,144,147,16384,16387,16400,16416,16528,196608,196611,196624,196627,196640,196643,196656,196659,196735,196736,196739,196752,196755,212992,212995,213008,213024,213136,1048576,1048579,1048592,1048595,1048608,1048611,1048624,1048627,1048703,1048704,1048707,1048720,1048723,1064960,1064963,1064976,1064992,1065104,1245184,1245187,1245200,1245203,1245216,1245219,1245232,1245235,1245311,1245312,1245315,1245328,1245331,1261568,1261571,1261584,1261600,1261712,2097152,2097155,2097168,2097171,2097184,2097187,2097200,2097203,2097279,2097280,2097283,2097296,2097299,2113536,2113539,2113552,2113568,2113680,2293760,2293763,2293776,2293779,2293792,2293795,2293808,2293811,2293887,2293888,2293891,2293904,2293907,2310144,2310147,2310160,2310176,2310288,3145728,3145731,3145744,3145747,3145760,3145763,3145776,3145779,3145855,3145856,3145859,3145872,3145875,3162112,3162115,3162128,3162144,3162256,3342336,3342339,3342352,3342355,3342368,3342371,3342384,3342387,3342463,3342464,3342467,3342480,3342483,3358720,3358723,3358736,3358752,3358864,8323072,8323075,8323088,8323091,8323104,8323107,8323120,8323123,8323199,8323200,8323203,8323216,8323219,8339456,8339459,8339472,8339488,8339600,8388608,8388611,8388624,8388627,8388640,8388643,8388656,8388659,8388735,8388736,8388739,8388752,8388755,8404992,8404995,8405008,8405024,8405136,8585216,8585219,8585232,8585235,8585248,8585251,8585264,8585267,8585343,8585344,8585347,8585360,8585363,8601600,8601603,8601616,8601632,8601744,9437184,9437187,9437200,9437203,9437216,9437219,9437232,9437235,9437311,9437312,9437315,9437328,9437331,9453568,9453571,9453584,9453600,9453712,9633792,9633795,9633808,9633811,9633824,9633827,9633840,9633843,9633919,9633920,9633923,9633936,9633939,9650176,9650179,9650192,9650208,9650320,1073741824,1073741827,1073741840,1073741843,1073741856,1073741859,1073741872,1073741875,1073741951,1073741952,1073741955,1073741968,1073741971,1073758208,1073758211,1073758224,1073758240,1073758352,1073938432,1073938435,1073938448,1073938451,1073938464,1073938467,1073938480,1073938483,1073938559,1073938560,1073938563,1073938576,1073938579,1073954816,1073954819,1073954832,1073954848,1073954960,1074790400,1074790403,1074790416,1074790419,1074790432,1074790435,1074790448,1074790451,1074790527,1074790528,1074790531,1074790544,1074790547,1074806784,1074806787,1074806800,1074806816,1074806928,1075838976,1075838979,1075838992,1075838995,1075839008,1075839011,1075839024,1075839027,1075839103,1075839104,1075839107,1075839120,1075839123,1075855360,1075855363,1075855376,1075855392,1075855504,1083179008,1083179011,1083179024,1083179027,1083179040,1083179043,1083179056,1083179059,1083179135,1083179136,1083179139,1083179152,1083179155,1083195392,1083195395,1083195408,1083195424,1083195536", "--wbias", "p6+0x10:407095,407094,407048,407159,407144,407127,407176,407287,407286,407336,407319,407368,407479,406536,406647,407031,406472,405559,405558,409288,409399,409398,407095,405991", "--wbias", "p6+0x14:407095,407094,407048,407159,407144,407127,407176,407287,407286,407336,407319,407368,407479,406536,406647,407031,406472,405559,405558,409288,409399,409398,407095,405991", "--wbias", "p11+0x234:0,1,2,3,4,5,6,-1", "--wbias", "p11+0x238:0,1,2,3,4,5,6,-1", "--wbias", "p11+0x200:0,1", "--wbias", "p11+0x204:0,1", "--wbias", "p7+0x10:407095,407094,407048,407159,407144,407127,407176,407287,407286,407336,407319,407368,407479,406536,406647,407031,406472,405559,405558,409288,409399,409398,407095,405991", "--wbias", "p7+0x14:407095,407094,407048,407159,407144,407127,407176,407287,407286,407336,407319,407368,407479,406536,406647,407031,406472,405559,405558,409288,409399,409398,407095,405991", "--wbias", "p12+0x234:0,1,2,3,4,5,6,-1", "--wbias", "p12+0x238:0,1,2,3,4,5,6,-1", "--wbias", "p12+0x200:0,1", "--wbias", "p12+0x204:0,1", "--wbias", "p8+0x10:407095,407094,407048,407159,407144,407127,407176,407287,407286,407336,407319,407368,407479,406536,406647,407031,406472,405559,405558,409288,409399,409398,407095,405991", "--wbias", "p8+0x14:407095,407094,407048,407159,407144,407127,407176,407287,407286,407336,407319,407368,407479,406536,406647,407031,406472,405559,405558,409288,409399,409398,407095,405991", "--wbias", "p13+0x234:0,1,2,3,4,5,6,-1", "--wbias", "p13+0x238:0,1,2,3,4,5,6,-1", "--wbias", "p13+0x200:0,1", "--wbias", "p13+0x204:0,1", "--wbias", "p9+0x10:407095,407094,407048,407159,407144,407127,407176,407287,407286,407336,407319,407368,407479,406536,406647,407031,406472,405559,405558,409288,409399,409398,407095,405991", "--wbias", "p9+0x14:407095,407094,407048,407159,407144,407127,407176,407287,407286,407336,407319,407368,407479,406536,406647,407031,406472,405559,405558,409288,409399,409398,407095,405991", "--wbias", "p14+0x234:0,1,2,3,4,5,6,-1", "--wbias", "p14+0x238:0,1,2,3,4,5,6,-1", "--wbias", "p14+0x200:0,1", "--wbias", "p14+0x204:0,1", "--rows", "4000", "--port", "{\n                // Windows: m the pathfinder (`can_transport` at +0x58); c0 `worldc` (+0x18 / +0x1c the tile width and\n                // height, +0x138 the mask array p10); p0 the object array of player 0, p1..p4 its objects, whose\n                // slot +0x3c answers the wall p6..p9 (`+0x10` / `+0x14` the position xor 0x63637, `+0x18` the type\n                // p11..p14: `x_size` +0x234, `y_size` +0x238, the `is(VILLAGE)` answer +0x200, slot +0x64's +0x204).\n                // `a[0..4]` the coordinates, `a[4]` / `a[6]` the two ends' object index, `a[11]` whether slot +0x64\n                // was the real `is_city` (then slot +0x60's answer, else slot +0x64's own). The crate's\n                // `valid_roadcoord` is driven on a `Sim` rebuilt from these windows; `covers_tile` is the original's own.\n                use crate::build::{BuildType, Ident};\n                use crate::world::{Pos, World};\n                use crate::{Sim, Tuning};\n                const K: i32 = 0x63637;\n                let (wt, ht) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));\n                let mut sim = Sim::new(Tuning::RON, World::new(wt / 4, ht / 4), 2);\n                for y in 0..ht {\n                    for x in 0..wt {\n                        let o = 2 * (y * wt + x) as usize;\n                        let mask = u16::from_le_bytes([p[10][o], p[10][o + 1]]);\n                        sim.world.set_tile_mask(Pos::new(x, y), mask);\n                    }\n                }\n                let mut end = |idx: i32| -> usize {\n                    let j = idx as usize;\n                    let ty_w = &p[11 + j];\n                    let city = if a[11] != 0 { rd(ty_w, 0x200) } else { rd(ty_w, 0x204) } != 0;\n                    let ty = sim.add_build_type(BuildType {\n                        ident: if city { Ident::Village } else { Ident::Other },\n                        x_size: rd(ty_w, 0x234),\n                        y_size: rd(ty_w, 0x238),\n                        ..BuildType::default()\n                    });\n                    let wall = &p[6 + j];\n                    let pos = Pos::new(rd(wall, 0x10) ^ K, rd(wall, 0x14) ^ K);\n                    let b = sim.add_building(0, pos, 1);\n                    sim.buildings[b].pos = pos;\n                    sim.buildings[b].ty = Some(ty);\n                    b\n                };\n                let ends = (end(a[4]), end(a[6]));\n                i64::from(sim.valid_roadcoord(\n                    Pos::new(a[0], a[1]),\n                    Pos::new(a[2], a[3]),\n                    ends,\n                    rd(m, 0x58) != 0,\n                ))\n            }"]
