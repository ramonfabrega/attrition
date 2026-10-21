#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_do_attack.py — `Build::do_attack@006228f0` under unicorn (item 3397, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_build_do_attack.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 144-byte object of random bytes (no bias), 0 stack argument(s);
`p` windows in order: p0..p42 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00c061ec points at 1376 random bytes; vtable slot +0x148 (1 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x11c (1 args) stubbed; vtable slot +0x114 (0 args) stubbed; vtable slot +0xe0 (1 args) stubbed; vtable slot +0x1b8 (0 args) stubbed; vtable slot +0x164 (0 args) stubbed; vtable slot +0x134 (1 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xd8 (0 args) stubbed; callee 0x00622c80 (0 args) stubbed; callee 0x00648ba0 (3 args) stubbed; callee 0x0060a850 (0 args) stubbed; callee 0x00648d70 (4 args) stubbed; callee 0x0064c8b0 (2 args) stubbed; callee 0x0097f340 (3 args) stubbed; the word at `0xc0aec0+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `p0+4` points at 128 random bytes less `0` (placed); the word at `p0+8` points at 128 random bytes less `0` (placed); the word at `p0+12` points at 128 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 128 random bytes less `0` (placed); the word at `p5+4` points at 128 random bytes less `0` (placed); the word at `p5+8` points at 128 random bytes less `0` (placed); the word at `p5+12` points at 128 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*2` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 128 random bytes less `0` (placed); the word at `p10+4` points at 128 random bytes less `0` (placed); the word at `p10+8` points at 128 random bytes less `0` (placed); the word at `p10+12` points at 128 random bytes less `0` (placed); the word at `c0+0x14+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 32 random bytes less `0` (placed); the word at `p15+4` points at 32 random bytes less `0` (placed); the word at `p15+8` points at 32 random bytes less `0` (placed); the word at `p15+12` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p20+0` points at 32 random bytes less `0` (placed); the word at `p20+4` points at 32 random bytes less `0` (placed); the word at `p20+8` points at 32 random bytes less `0` (placed); the word at `p20+12` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*2` points at 16 random bytes less `0` (placed); the word at `p25+0` points at 32 random bytes less `0` (placed); the word at `p25+4` points at 32 random bytes less `0` (placed); the word at `p25+8` points at 32 random bytes less `0` (placed); the word at `p25+12` points at 32 random bytes less `0` (placed); the word at `m+0x18` points at 512 random bytes less `0` (placed); the word at `p1+0x18` points at 720 random bytes less `0` (placed); the word at `p2+0x18` points at 720 random bytes less `0` (placed); the word at `p3+0x18` points at 720 random bytes less `0` (placed); the word at `p4+0x18` points at 720 random bytes less `0` (placed); the word at `p6+0x18` points at 720 random bytes less `0` (placed); the word at `p7+0x18` points at 720 random bytes less `0` (placed); the word at `p8+0x18` points at 720 random bytes less `0` (placed); the word at `p9+0x18` points at 720 random bytes less `0` (placed); the word at `p11+0x18` points at 720 random bytes less `0` (placed); the word at `p12+0x18` points at 720 random bytes less `0` (placed); the word at `p13+0x18` points at 720 random bytes less `0` (placed); the word at `p14+0x18` points at 720 random bytes less `0` (placed); stub 9 answered from [1, 1, 1, 0] (90%); stub 10 answered from [0, 0, 0, 0, 1] (90%); stub 12 answered from [1, 1, 1, 0] (90%); stub 13 answered from [14, 14, 3, 0, 1, 15] (90%); stub 14 answered from [1, 1, 1, 0] (90%); the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 0 answers 2 values in call order, printed after the answers; stub 1 answers 4 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 3 answers 2 values in call order, printed after the answers; stub 0 answers 0..1 every row; stub 1 answers 0..1 every row; stub 2 answers 0..1 every row; stub 3 answers -3..40 every row; stub 4 answers -2..5 every row; stub 5 answers -10..3 every row; stub 6 answers -2..5 every row; stub 8 answers -70000..70000 every row; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 5 writes a value in 0..2 through its argument 0, printed after the answers (`o` in a place); the stack 1024 bytes under the call is zeroed before each row; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`.
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

ENTRY = 0x006228f0
WIN = 144
OWN = 0
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607884, 128), (12607980, 1376)]
TLS = []
OUTRET = []
TABLEARG = None
AT = []
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('slot', 328, 1), ('slot', 96, 2), ('slot', 184, 2), ('slot', 284, 1), ('slot', 276, 0), ('slot', 224, 1), ('slot', 440, 0), ('slot', 356, 0), ('slot', 308, 1), ('slot', 24, 0), ('slot', 216, 0), ('call', 6433920, 0), ('call', 6589344, 3), ('call', 6334544, 0), ('call', 6589808, 4), ('call', 6604976, 2), ('call', 9958208, 3)]
PLACES = [(None, '0xc0aec0+0x1c*0', '0', 16), ('p0', '0', '0', 128), ('p0', '4', '0', 128), ('p0', '8', '0', 128), ('p0', '12', '0', 128), (None, '0xc0aec0+0x1c*1', '0', 16), ('p5', '0', '0', 128), ('p5', '4', '0', 128), ('p5', '8', '0', 128), ('p5', '12', '0', 128), (None, '0xc0aec0+0x1c*2', '0', 16), ('p10', '0', '0', 128), ('p10', '4', '0', 128), ('p10', '8', '0', 128), ('p10', '12', '0', 128), ('c0', '0x14+0x1c*0', '0', 16), ('p15', '0', '0', 32), ('p15', '4', '0', 32), ('p15', '8', '0', 32), ('p15', '12', '0', 32), ('c0', '0x14+0x1c*1', '0', 16), ('p20', '0', '0', 32), ('p20', '4', '0', 32), ('p20', '8', '0', 32), ('p20', '12', '0', 32), ('c0', '0x14+0x1c*2', '0', 16), ('p25', '0', '0', 32), ('p25', '4', '0', 32), ('p25', '8', '0', 32), ('p25', '12', '0', 32), ('m', '0x18', '0', 512), ('p1', '0x18', '0', 720), ('p2', '0x18', '0', 720), ('p3', '0x18', '0', 720), ('p4', '0x18', '0', 720), ('p6', '0x18', '0', 720), ('p7', '0x18', '0', 720), ('p8', '0x18', '0', 720), ('p9', '0x18', '0', 720), ('p11', '0x18', '0', 720), ('p12', '0x18', '0', 720), ('p13', '0x18', '0', 720), ('p14', '0x18', '0', 720)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 8, ('set', [4292345856, 4292345984, 4292345983, 4292345857, 4292411392, 4292411520, 4292411519, 4292411393, 4292476928, 4292477056, 4292477055, 4292476929, 4292542464, 4292542592, 4292542591, 4292542465, 4292608000, 4292608128, 4292608127, 4292608001, 4292673536, 4292673664, 4292673663, 4292673537, 4292739072, 4292739200, 4292739199, 4292739073, 4292804608, 4292804736, 4292804735, 4292804609, 4292870144, 4292870272, 4292870271, 4292870145, 4292935680, 4292935808, 4292935807, 4292935681, 4293001216, 4293001344, 4293001343, 4293001217, 4293066752, 4293066880, 4293066879, 4293066753, 4293132288, 4293132416, 4293132415, 4293132289, 4293197824, 4293197952, 4293197951, 4293197825, 4293263360, 4293263488, 4293263487, 4293263361, 4293328896, 4293329024, 4293329023, 4293328897, 4293394432, 4293394560, 4293394559, 4293394433, 4293459968, 4293460096, 4293460095, 4293459969, 4293525504, 4293525632, 4293525631, 4293525505, 4293591040, 4293591168, 4293591167, 4293591041, 4293656576, 4293656704, 4293656703, 4293656577, 4293722112, 4293722240, 4293722239, 4293722113, 4293787648, 4293787776, 4293787775, 4293787649, 4293853184, 4293853312, 4293853311, 4293853185, 4293918720, 4293918848, 4293918847, 4293918721, 4293984256, 4293984384, 4293984383, 4293984257, 4294049792, 4294049920, 4294049919, 4294049793, 4294115328, 4294115456, 4294115455, 4294115329, 4294180864, 4294180992, 4294180991, 4294180865, 4294246400, 4294246528, 4294246527, 4294246401, 4294311936, 4294312064, 4294312063, 4294311937, 4294377472, 4294377600, 4294377599, 4294377473, 4294443008, 4294443136, 4294443135, 4294443009, 4294508544, 4294508672, 4294508671, 4294508545, 4294574080, 4294574208, 4294574207, 4294574081, 4294639616, 4294639744, 4294639743, 4294639617, 4294705152, 4294705280, 4294705279, 4294705153, 4294770688, 4294770816, 4294770815, 4294770689, 4294836224, 4294836352, 4294836351, 4294836225, 4294901760, 4294901888, 4294901887, 4294901761, 0, 128, 127, 1, 65536, 65664, 65663, 65537, 131072, 131200, 131199, 131073, 196608, 196736, 196735, 196609, 262144, 262272, 262271, 262145, 327680, 327808, 327807, 327681, 393216, 393344, 393343, 393217, 458752, 458880, 458879, 458753, 524288, 524416, 524415, 524289, 589824, 589952, 589951, 589825, 655360, 655488, 655487, 655361, 720896, 721024, 721023, 720897, 786432, 786560, 786559, 786433, 851968, 852096, 852095, 851969, 917504, 917632, 917631, 917505, 983040, 983168, 983167, 983041, 1048576, 1048704, 1048703, 1048577, 1114112, 1114240, 1114239, 1114113, 1179648, 1179776, 1179775, 1179649, 1245184, 1245312, 1245311, 1245185, 1310720, 1310848, 1310847, 1310721, 1376256, 1376384, 1376383, 1376257, 1441792, 1441920, 1441919, 1441793, 1507328, 1507456, 1507455, 1507329, 1572864, 1572992, 1572991, 1572865, 1638400, 1638528, 1638527, 1638401, 1703936, 1704064, 1704063, 1703937, 1769472, 1769600, 1769599, 1769473, 1835008, 1835136, 1835135, 1835009, 1900544, 1900672, 1900671, 1900545, 1966080, 1966208, 1966207, 1966081, 2031616, 2031744, 2031743, 2031617, 2097152, 2097280, 2097279, 2097153, 2162688, 2162816, 2162815, 2162689, 2228224, 2228352, 2228351, 2228225, 2293760, 2293888, 2293887, 2293761, 2359296, 2359424, 2359423, 2359297, 2424832, 2424960, 2424959, 2424833, 2490368, 2490496, 2490495, 2490369, 2555904, 2556032, 2556031, 2555905, 2621440, 2621568, 2621567, 2621441])), ('m', 36, ('range', 0, 30)), ('m', 64, ('set', [0, 1, 2, 3, 4, 5, 6, 7, 248, 255, 128])), ('m', 96, ('set', [0, 65536, 131072, 262144, 16711680, 393216, 4, 65540, 131076, 262148, 16711684, 393220, 32768, 98304, 163840, 294912, 16744448, 425984, 32772, 98308, 163844, 294916, 16744452, 425988, 32763, 98299, 163835, 294907, 16744443, 425979])), ('m', 120, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 65536, 131072, 327680, 4294901760, 2147418112, 2147483648])), ('m', 124, ('set', [0, 1, 2, 3, 65535, 4294967295, 65534, 32768, 0, 1, 2, 3])), ('m', 128, ('set', [0, 256, 512, 256, 0, 512, 65280, 32768, 33024])), ('c1', 1360, ('range', 0, 70)), ('p30', 500, ('set', [0, 1, 2, 3, 10, 100, 300, 32767, 32768, 65535, 74565, 65539])), ('p31', 712, ('set', [0, 65536, 131072, 65537, 32768])), ('p32', 712, ('set', [0, 65536, 131072, 65537, 32768])), ('p33', 712, ('set', [0, 65536, 131072, 65537, 32768])), ('p34', 712, ('set', [0, 65536, 131072, 65537, 32768])), ('p35', 712, ('set', [0, 65536, 131072, 65537, 32768])), ('p36', 712, ('set', [0, 65536, 131072, 65537, 32768])), ('p37', 712, ('set', [0, 65536, 131072, 65537, 32768])), ('p38', 712, ('set', [0, 65536, 131072, 65537, 32768])), ('p39', 712, ('set', [0, 65536, 131072, 65537, 32768])), ('p40', 712, ('set', [0, 65536, 131072, 65537, 32768])), ('p41', 712, ('set', [0, 65536, 131072, 65537, 32768])), ('p42', 712, ('set', [0, 65536, 131072, 65537, 32768]))]
FILLS = []
ARGRANGE = []
SLOTBASE = []
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(9, [1, 1, 1, 0]), (10, [0, 0, 0, 0, 1]), (12, [1, 1, 1, 0]), (13, [14, 14, 3, 0, 1, 15]), (14, [1, 1, 1, 0])]
VTAT = None
VTATHALF = []
VTIMAGE = [('m', 0, 11805044)]
ALIAS = []
ANSWERBY = []
ANSWERWIN = []
CDECL = False
TIB = False
ARGEQ = []
SEQ = [(0, 2, None), (1, 4, None), (2, 4, None), (3, 2, None)]
ANSRANGE = [(0, 0, 1), (1, 0, 1), (2, 0, 1), (3, -3, 40), (4, -2, 5), (5, -10, 3), (6, -2, 5), (8, -70000, 70000)]
CELLTHIS = []
VTALSO = [('p30', 0), ('p1', 0), ('p2', 0), ('p3', 0), ('p4', 0), ('p6', 0), ('p7', 0), ('p8', 0), ('p9', 0), ('p11', 0), ('p12', 0), ('p13', 0), ('p14', 0), ('p16', 0), ('p17', 0), ('p18', 0), ('p19', 0), ('p21', 0), ('p22', 0), ('p23', 0), ('p24', 0), ('p26', 0), ('p27', 0), ('p28', 0), ('p29', 0)]
SPARSE = []
GTAB = []
RECORD = [7, 11, 12, 13, 14, 15, 16]
RECORDORDER = True
RECORDEDX = False
ROWSPLIT = {}
LINKS = []
LINKHI = []
OUTARG = [(5, 0, ('range', 0, 2))]
CDECLSTUBS = []
STUBWRITE = []
STUBCOPY = []
STUBALLOC = []
STACKCLEAN = 1024
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
    rng = random.Random(3397)
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

# sweep_gen: ["obj", "Build::do_attack@006228f0", "--item", "3397", "--win", "0x90", "--own", "0", "--ret", "none", "--rows", "4000", "--clean-stack", "0x400", "--cell-win", "0xc0618c:0x80", "--cell-win", "0xc061ec:0x560", "--ptrtab", "0xc0aec0+0x1c*0:4:0x80:vt", "--ptrtab", "0xc0aec0+0x1c*1:4:0x80:vt", "--ptrtab", "0xc0aec0+0x1c*2:4:0x80:vt", "--ptrtab", "c0+0x14+0x1c*0:4:0x20:vt", "--ptrtab", "c0+0x14+0x1c*1:4:0x20:vt", "--ptrtab", "c0+0x14+0x1c*2:4:0x20:vt", "--place", "m+0x18:0:0x200", "--place", "p1+0x18:0:0x2d0", "--place", "p2+0x18:0:0x2d0", "--place", "p3+0x18:0:0x2d0", "--place", "p4+0x18:0:0x2d0", "--place", "p6+0x18:0:0x2d0", "--place", "p7+0x18:0:0x2d0", "--place", "p8+0x18:0:0x2d0", "--place", "p9+0x18:0:0x2d0", "--place", "p11+0x18:0:0x2d0", "--place", "p12+0x18:0:0x2d0", "--place", "p13+0x18:0:0x2d0", "--place", "p14+0x18:0:0x2d0", "--vtat-image", "m+0:0xb42174", "--vtat-also", "p30+0", "--stub", "slot:0x148:1", "--stub", "slot:0x60:2", "--stub", "slot:0xb8:2", "--stub", "slot:0x11c:1", "--stub", "slot:0x114:0", "--stub", "slot:0xe0:1", "--stub", "slot:0x1b8:0", "--stub", "slot:0x164:0", "--stub", "slot:0x134:1", "--stub", "slot:0x18:0", "--stub", "slot:0xd8:0", "--stub", "0x622c80:0", "--stub", "0x648ba0:3", "--stub", "0x60a850:0", "--stub", "0x648d70:4", "--stub", "0x64c8b0:2", "--stub", "0x97f340:3", "--seq", "0:2", "--answer-range", "0:0:1", "--seq", "1:4", "--answer-range", "1:0:1", "--seq", "2:4", "--answer-range", "2:0:1", "--seq", "3:2", "--answer-range", "3:-3:40", "--answer-range", "4:-2:5", "--answer-range", "5:-10:3", "--answer-range", "6:-2:5", "--answer-range", "8:-70000:70000", "--answers", "9:1,1,1,0", "--answers", "10:0,0,0,0,1", "--answers", "12:1,1,1,0", "--answers", "13:14,14,3,0,1,15", "--answers", "14:1,1,1,0", "--outarg", "5:0:0..2", "--record", "7", "--record", "11", "--record", "12", "--record", "13", "--record", "14", "--record", "15", "--record", "16", "--record-order", "--wbias", "m+0x8:4292345856,4292345984,4292345983,4292345857,4292411392,4292411520,4292411519,4292411393,4292476928,4292477056,4292477055,4292476929,4292542464,4292542592,4292542591,4292542465,4292608000,4292608128,4292608127,4292608001,4292673536,4292673664,4292673663,4292673537,4292739072,4292739200,4292739199,4292739073,4292804608,4292804736,4292804735,4292804609,4292870144,4292870272,4292870271,4292870145,4292935680,4292935808,4292935807,4292935681,4293001216,4293001344,4293001343,4293001217,4293066752,4293066880,4293066879,4293066753,4293132288,4293132416,4293132415,4293132289,4293197824,4293197952,4293197951,4293197825,4293263360,4293263488,4293263487,4293263361,4293328896,4293329024,4293329023,4293328897,4293394432,4293394560,4293394559,4293394433,4293459968,4293460096,4293460095,4293459969,4293525504,4293525632,4293525631,4293525505,4293591040,4293591168,4293591167,4293591041,4293656576,4293656704,4293656703,4293656577,4293722112,4293722240,4293722239,4293722113,4293787648,4293787776,4293787775,4293787649,4293853184,4293853312,4293853311,4293853185,4293918720,4293918848,4293918847,4293918721,4293984256,4293984384,4293984383,4293984257,4294049792,4294049920,4294049919,4294049793,4294115328,4294115456,4294115455,4294115329,4294180864,4294180992,4294180991,4294180865,4294246400,4294246528,4294246527,4294246401,4294311936,4294312064,4294312063,4294311937,4294377472,4294377600,4294377599,4294377473,4294443008,4294443136,4294443135,4294443009,4294508544,4294508672,4294508671,4294508545,4294574080,4294574208,4294574207,4294574081,4294639616,4294639744,4294639743,4294639617,4294705152,4294705280,4294705279,4294705153,4294770688,4294770816,4294770815,4294770689,4294836224,4294836352,4294836351,4294836225,4294901760,4294901888,4294901887,4294901761,0,128,127,1,65536,65664,65663,65537,131072,131200,131199,131073,196608,196736,196735,196609,262144,262272,262271,262145,327680,327808,327807,327681,393216,393344,393343,393217,458752,458880,458879,458753,524288,524416,524415,524289,589824,589952,589951,589825,655360,655488,655487,655361,720896,721024,721023,720897,786432,786560,786559,786433,851968,852096,852095,851969,917504,917632,917631,917505,983040,983168,983167,983041,1048576,1048704,1048703,1048577,1114112,1114240,1114239,1114113,1179648,1179776,1179775,1179649,1245184,1245312,1245311,1245185,1310720,1310848,1310847,1310721,1376256,1376384,1376383,1376257,1441792,1441920,1441919,1441793,1507328,1507456,1507455,1507329,1572864,1572992,1572991,1572865,1638400,1638528,1638527,1638401,1703936,1704064,1704063,1703937,1769472,1769600,1769599,1769473,1835008,1835136,1835135,1835009,1900544,1900672,1900671,1900545,1966080,1966208,1966207,1966081,2031616,2031744,2031743,2031617,2097152,2097280,2097279,2097153,2162688,2162816,2162815,2162689,2228224,2228352,2228351,2228225,2293760,2293888,2293887,2293761,2359296,2359424,2359423,2359297,2424832,2424960,2424959,2424833,2490368,2490496,2490495,2490369,2555904,2556032,2556031,2555905,2621440,2621568,2621567,2621441", "--wbias", "m+0x24:0..30", "--wbias", "m+0x40:0,1,2,3,4,5,6,7,0xf8,0xff,0x80", "--wbias", "m+0x60:0,65536,131072,262144,16711680,393216,4,65540,131076,262148,16711684,393220,32768,98304,163840,294912,16744448,425984,32772,98308,163844,294916,16744452,425988,32763,98299,163835,294907,16744443,425979", "--wbias", "m+0x78:0,0,0,0,0,0,0,0,65536,131072,327680,4294901760,2147418112,2147483648", "--wbias", "m+0x7c:0,1,2,3,0xffff,0xffffffff,0xfffe,0x8000,0,1,2,3", "--wbias", "m+0x80:0,0x100,0x200,0x100,0,0x200,0xff00,0x8000,0x8100", "--wbias", "c1+0x550:0..70", "--wbias", "p30+0x1f4:0,1,2,3,10,100,300,32767,0x8000,0xffff,0x12345,0x10003", "--wbias", "p31..42+0x2c8:0,0x10000,0x20000,0x10001,0x8000", "--port", "{\n                // hand: port\n                let ret: i64 = 'done: {\n                // Window map: m the building; c[1] the game (frame at +0x550); p[5w] the\n                // units-table array of player w and p[5w+1+ox] its units; p[30] the building's\n                // type; p[31+4w+ox] unit (w, ox)'s type. Stub answers: a[0] `+0x148`,\n                // a[3] `+0x11c`, a[4] `+0x114`, a[5] `+0xe0` (a[17] its out-parameter), a[6]\n                // `+0x1b8`, a[8] `+0x134`, a[9] `+0x18`, a[10] `+0xd8`, a[12] `valid_target`,\n                // a[13] `action_type`, a[14] `is_in_range`; the `--seq` runs a[18..20] (`+0x148`),\n                // a[20..24] (`+0x60`, the type's `is`), a[24..28] (`+0xb8`, this's `is`),\n                // a[28..30] (`+0x11c`).\n                let vf = rd(m, 0) as u32 == 0xb42174;\n                let seq = |base: usize, len: usize, k: usize| a[base + k.min(len - 1)];\n                let mut n_g = 0usize; // `+0x148` calls so far\n                let mut n_is = 0usize; // `is` calls so far\n                let is = |n_is: &mut usize| -> bool {\n                    let v = if vf { seq(20, 4, *n_is) } else { seq(24, 4, *n_is) };\n                    *n_is += 1;\n                    v != 0\n                };\n                let frame = rd(&c[1], 0x550);\n                let o = i16::from_le_bytes([m[0xa], m[0xb]]) as i32;\n                let rech = u16::from_le_bytes([m[0x7a], m[0x7b]]);\n                // The head: a held-fire building (`+0x148` answers non-zero and neither `is`\n                // holds) skips the reload countdown.\n                let g = seq(18, 2, n_g) != 0;\n                n_g += 1;\n                let mut countdown = !g;\n                if g {\n                    countdown = is(&mut n_is) || is(&mut n_is);\n                }\n                if countdown && rech != 0 {\n                    m[0x7a..0x7c].copy_from_slice(&(rech - 1).to_le_bytes());\n                    break 'done 0;\n                }\n                if frame.wrapping_add(o) & 0x1f == 0 && (m[8] as i8) >= 0 {\n                    m[0x40] = 0;\n                }\n                m[8] &= 0x7f;\n                let n = if vf {\n                    let max = a[28];\n                    let d = a[29].wrapping_sub(rd(m, 0x24));\n                    if d < 0 || max < 0 { 0 } else { d.min(max) }\n                } else {\n                    a[4]\n                };\n                if n == 0 {\n                    break 'done 0;\n                }\n                let flags = |m: &[u8]| u16::from_le_bytes([m[0x60], m[0x61]]);\n                let put_flags = |m: &mut [u8], f: u16| m[0x60..0x62].copy_from_slice(&f.to_le_bytes());\n                if a[5] >= 0 {\n                    // Jammed: the unit the slot names is marked, the building cries misfire.\n                    let (w, ox) = (a[17] as usize, a[5] as usize);\n                    let u = &mut p[5 * w + 1 + ox];\n                    let v = rd(u, 0x6c) | 8;\n                    wr(u, 0x6c, v);\n                    log.push(format!(\n                        \"s16:{},110,{},{}\",\n                        rd(m, 0x14) ^ 0x63637,\n                        rd(m, 0x10) ^ 0x63637,\n                        rd(m, 0x14) ^ 0x63637\n                    ));\n                    put_flags(m, flags(m) | 0x8000);\n                    break 'done 0;\n                }\n                put_flags(m, flags(m) & 0x7fff);\n                let arrows = a[6];\n                if arrows == 0 {\n                    break 'done 0;\n                }\n                let find = |log: &mut Vec<String>| log.push(\"s11:m+0x0\".to_string());\n                let clear = |m: &mut [u8]| {\n                    let f = u16::from_le_bytes([m[0x60], m[0x61]]) & 0xfffb;\n                    m[0x60..0x62].copy_from_slice(&f.to_le_bytes());\n                    m[0x81] = 0xff;\n                    m[0x7c..0x7e].copy_from_slice(&0xffffu16.to_le_bytes());\n                };\n                if flags(m) & 4 == 0 {\n                    find(&mut log);\n                }\n                let who = m[0x81] as i8 as i32;\n                let ox = i16::from_le_bytes([m[0x7c], m[0x7d]]) as i32;\n                if ox < 0 || who < 0 {\n                    clear(m);\n                    break 'done 0;\n                }\n                log.push(format!(\"s12:m+0x0,{ox},{who},0\"));\n                if a[12] == 0 {\n                    find(&mut log);\n                    break 'done 0;\n                }\n                let (w, x) = (who as usize, ox as usize);\n                let slot = format!(\"p{}+{:#x}\", 5 * w, 4 * x);\n                let ecx: String;\n                if flags(m) & 4 != 0 {\n                    ecx = \"m+0x0\".to_string();\n                } else {\n                    let r = frame.wrapping_add(o).wrapping_add(14) & 0x8000001f_u32 as i32;\n                    let r = if r < 0 { ((r - 1) | -32) + 1 } else { r };\n                    if frame != 0 && r != 0 {\n                        ecx = frame.to_string();\n                    } else if a[9] == 0 || a[10] != 0 {\n                        find(&mut log);\n                        break 'done 0;\n                    } else {\n                        if rd(&p[31 + 4 * w + x], 0x2c8) & 0x10000 != 0 {\n                            ecx = slot;\n                        } else if {\n                            log.push(format!(\"s13:p{}+0x0\", 5 * w + 1 + x));\n                            a[13] != 0xe\n                        } {\n                            find(&mut log);\n                            break 'done 0;\n                        } else {\n                            ecx = format!(\"p{}+0x0\", 5 * w + 1 + x);\n                        }\n                    }\n                }\n                log.push(format!(\"s14:m+0x0,{ox},{who},{ecx},0\"));\n                if a[14] == 0 {\n                    clear(m);\n                    break 'done 0;\n                }\n                if seq(18, 2, n_g) != 0 {\n                    let fire = is(&mut n_is) || is(&mut n_is);\n                    if !fire {\n                        break 'done 0;\n                    }\n                }\n                log.push(format!(\"s15:m+0x0,{ox},{who}\"));\n                m[8] |= 0x80;\n                let bit = 1i32.wrapping_shl(who as u32 & 31);\n                let b40 = m[0x40] as i8 as i32;\n                if bit & b40 == 0 {\n                    m[0x40] = (b40 | bit) as u8;\n                    // A group the building has not shouted for yet: slot `+0x164`.\n                    if m[0x62] as i32 & (bit & 0xff) == 0 {\n                        log.push(\"s7:m+0x0\".to_string());\n                    }\n                }\n                let num = if vf { rd(&p[30], 0x1f4) as i16 as i32 } else { a[8] as i16 as i32 };\n                let v = (num / arrows) as i16;\n                m[0x7a..0x7c].copy_from_slice(&v.to_le_bytes());\n                0\n                };\n                if log.is_empty() {\n                    log.push(\"-\".to_string());\n                }\n                ret\n                // end hand\n            }"]
