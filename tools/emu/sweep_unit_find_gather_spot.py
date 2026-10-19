#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_find_gather_spot.py — `Unit::find_gather_spot@005f5170` under unicorn (item 3320, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_unit_find_gather_spot.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 64-byte object of random bytes (no bias), 3 stack argument(s);
`p` windows in order: p0..p39 the place(s), in flag order; the global cell 0x00c0618c points at 496 random bytes; the global cell 0x00c06198 points at 16 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c061ac points at 32 random bytes; callee 0x006b52e0 (2 args) stubbed; callee 0x0065b870 (5 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x180 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; callee 0x0046d330 (0 args) run, not hooked; callee 0x00630450 (2 args) stubbed; callee 0x00608880 (3 args) stubbed; callee 0x0046cff0 (0 args) run, not hooked; callee 0x006e33a0 (2 args) stubbed; callee 0x00670230 (0 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x0061a5c0 (3 args) stubbed; the word at `c0+0x14+0x1c*m[9]` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 40 random bytes less `0` (placed); the word at `p0+4` points at 40 random bytes less `0` (placed); the word at `p0+8` points at 40 random bytes less `0` (placed); the word at `p0+12` points at 40 random bytes less `0` (placed); the word at `p0+16` points at 40 random bytes less `0` (placed); the word at `p0+20` points at 40 random bytes less `0` (placed); the word at `0xc09970+0x1c*m[9]` points at 16 random bytes less `0` (placed); the word at `p7+0` points at 96 random bytes less `0` (placed); the word at `p7+4` points at 96 random bytes less `0` (placed); the word at `p7+8` points at 96 random bytes less `0` (placed); the word at `p7+12` points at 96 random bytes less `0` (placed); the word at `c0+0x1ec` points at 16 random bytes less `0` (placed); the word at `0xe41248+0x6eec*m[9]` points at 176 random bytes less `0` (placed); the word at `m+0x18` points at 16 random bytes less `0` (placed); the word at `c3+0x10` points at 24 random bytes less `0` (placed); the word at `p1+0x20` points at 32 random bytes less `0` (placed); the word at `p1+0x24` points at 136 random bytes less `0` (placed); the word at `p16+0x18` points at 16 random bytes less `0` (placed); the word at `p17+0x18` points at 16 random bytes less `0` (placed); the word at `p2+0x20` points at 32 random bytes less `0` (placed); the word at `p2+0x24` points at 136 random bytes less `0` (placed); the word at `p20+0x18` points at 16 random bytes less `0` (placed); the word at `p21+0x18` points at 16 random bytes less `0` (placed); the word at `p3+0x20` points at 32 random bytes less `0` (placed); the word at `p3+0x24` points at 136 random bytes less `0` (placed); the word at `p24+0x18` points at 16 random bytes less `0` (placed); the word at `p25+0x18` points at 16 random bytes less `0` (placed); the word at `p4+0x20` points at 32 random bytes less `0` (placed); the word at `p4+0x24` points at 136 random bytes less `0` (placed); the word at `p28+0x18` points at 16 random bytes less `0` (placed); the word at `p29+0x18` points at 16 random bytes less `0` (placed); the word at `p5+0x20` points at 32 random bytes less `0` (placed); the word at `p5+0x24` points at 136 random bytes less `0` (placed); the word at `p32+0x18` points at 16 random bytes less `0` (placed); the word at `p33+0x18` points at 16 random bytes less `0` (placed); the word at `p6+0x20` points at 32 random bytes less `0` (placed); the word at `p6+0x24` points at 136 random bytes less `0` (placed); the word at `p36+0x18` points at 16 random bytes less `0` (placed); the word at `p37+0x18` points at 16 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0x180 laid as 0x0063ed30 or its stub, a flag after the answers; stub 0 answered from [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2] (90%); stub 2 answered from [1, 1, 1, 1, 1, 1, 1, 1, 1, 0] (90%); stub 3 answered from [1, 1, 1, 1, 1, 1, 1, 1, 1, 0] (90%); stub 6 answered from [1, 1, 1, 1, 1, 1, 1, 0] (90%); stub 7 answered from [0, 0, 0, 0, 0, 0, 0, 1] (90%); stub 8 answered from [1, 1, 1, 0, 0] (90%); stub 9 answered from [0, 0, 0, 0, 1] (90%); stub 11 answered from [0, 0, 0, 1, 1, 2, 3] (90%); stub 12 answered from [0, 0, 1, 1] (90%); stub 14 answered from [1, 1, 1, 1, 1, 1, 0] (90%); stub 1 answered from [-1, -1, -1, 0, 1, 2, 3] (90%); stub 16 answered from [0, 1] (90%); the word at `p17+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p21+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p25+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p29+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p33+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p37+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 0 answers 7 values in call order, printed after the answers; stub 2 answers 6 values in call order, printed after the answers; stub 3 answers 6 values in call order, printed after the answers; stub 6 answers 6 values in call order, printed after the answers; stub 7 answers 6 values in call order, printed after the answers; stub 8 answers 14 values in call order, printed after the answers; stub 9 answers 14 values in call order, printed after the answers; stub 11 answers 6 values in call order, printed after the answers; stub 12 answers 7 values in call order, printed after the answers; stub 14 answers 36 values in call order, printed after the answers; stub 1 answers -9..3 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 4 answers from its own `ecx`: the word at `+0x20`; stub 5 answers from its own `ecx`: the word at `+0x24`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`.
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

ENTRY = 0x005f5170
WIN = 64
OWN = 3
BIAS = []
ARGSET = [(0, [-1, 0, 1, 100, 500, 1000, 1500, 2000, 3000, 5000, 30000, 50000, 100000, 2147483647, 2147483647])]
ARGFROM = []
CELLS = [(12607884, 496), (12607896, 16), (13297148, 4096), (12607916, 32)]
AT = []
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('call', 7033568, 2), ('call', 6666352, 5), ('slot', 12, 0), ('slot', 76, 0), ('slot', 176, 0), ('slot', 172, 0), ('slot', 144, 0), ('slot', 384, 0), ('slot', 184, 2), ('slot', 96, 2), ('run', 4641584, 0), ('call', 6489168, 2), ('call', 6326400, 3), ('run', 4640752, 0), ('call', 7222176, 2), ('call', 6750768, 0), ('call', 7213936, 1), ('call', 6399424, 3)]
PLACES = [('c0', '0x14+0x1c*m[9]', '0', 24), ('p0', '0', '0', 40), ('p0', '4', '0', 40), ('p0', '8', '0', 40), ('p0', '12', '0', 40), ('p0', '16', '0', 40), ('p0', '20', '0', 40), (None, '0xc09970+0x1c*m[9]', '0', 16), ('p7', '0', '0', 96), ('p7', '4', '0', 96), ('p7', '8', '0', 96), ('p7', '12', '0', 96), ('c0', '0x1ec', '0', 16), (None, '0xe41248+0x6eec*m[9]', '0', 176), ('m', '0x18', '0', 16), ('c3', '0x10', '0', 24), ('p1', '0x20', '0', 32), ('p1', '0x24', '0', 136), ('p16', '0x18', '0', 16), ('p17', '0x18', '0', 16), ('p2', '0x20', '0', 32), ('p2', '0x24', '0', 136), ('p20', '0x18', '0', 16), ('p21', '0x18', '0', 16), ('p3', '0x20', '0', 32), ('p3', '0x24', '0', 136), ('p24', '0x18', '0', 16), ('p25', '0x18', '0', 16), ('p4', '0x20', '0', 32), ('p4', '0x24', '0', 136), ('p28', '0x18', '0', 16), ('p29', '0x18', '0', 16), ('p5', '0x20', '0', 32), ('p5', '0x24', '0', 136), ('p32', '0x18', '0', 16), ('p33', '0x18', '0', 16), ('p6', '0x20', '0', 32), ('p6', '0x24', '0', 136), ('p36', '0x18', '0', 16), ('p37', '0x18', '0', 16)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 8, ('set', [0, 256, 512, 768])), ('m', 16, ('set', [407095, 407123, 407287, 407491, 407007, 407007, 405991, 408975, 423959, 436855, 449623])), ('m', 20, ('set', [407095, 407123, 407287, 407491, 407007, 407007, 405991, 408975, 423959, 436855, 449623])), ('p14', 4, ('set', [52, 53, 0, 1, 51, 54, 0, 0, 5, 16])), ('c1', 4, ('set', [0, 0, 0, 1, 2])), ('p12', 0, ('set', [2, 3, 4, 5, 6, 6, 1, 0])), ('p12', 4, ('set', [2, 3, 4, 5, 6, 6, 1, 0])), ('p12', 8, ('set', [2, 3, 4, 5, 6, 6, 1, 0])), ('p12', 12, ('set', [2, 3, 4, 5, 6, 6, 1, 0])), ('p8', 88, ('set', [0, 65536, 131072, 196608, 327680])), ('p9', 88, ('set', [0, 65536, 131072, 196608, 327680])), ('p10', 88, ('set', [0, 65536, 131072, 196608, 327680])), ('p11', 88, ('set', [0, 65536, 131072, 196608, 327680])), ('p8', 92, ('set', [0, 1, 2, 3, 5])), ('p9', 92, ('set', [0, 1, 2, 3, 5])), ('p10', 92, ('set', [0, 1, 2, 3, 5])), ('p11', 92, ('set', [0, 1, 2, 3, 5])), ('p1', 16, ('set', [407095, 407123, 407287, 407491, 407007, 407007, 405991, 408975, 423959, 436855, 449623])), ('p1', 20, ('set', [407095, 407123, 407287, 407491, 407007, 407007, 405991, 408975, 423959, 436855, 449623])), ('p17', 112, ('set', [0, 65536, 131072, 196608, -65536, 65536, 131072])), ('p17', 128, ('set', [1, 2, 3, 4, 5, 5, 255, 128, 0])), ('p2', 16, ('set', [407095, 407123, 407287, 407491, 407007, 407007, 405991, 408975, 423959, 436855, 449623])), ('p2', 20, ('set', [407095, 407123, 407287, 407491, 407007, 407007, 405991, 408975, 423959, 436855, 449623])), ('p21', 112, ('set', [0, 65536, 131072, 196608, -65536, 65536, 131072])), ('p21', 128, ('set', [1, 2, 3, 4, 5, 5, 255, 128, 0])), ('p3', 16, ('set', [407095, 407123, 407287, 407491, 407007, 407007, 405991, 408975, 423959, 436855, 449623])), ('p3', 20, ('set', [407095, 407123, 407287, 407491, 407007, 407007, 405991, 408975, 423959, 436855, 449623])), ('p25', 112, ('set', [0, 65536, 131072, 196608, -65536, 65536, 131072])), ('p25', 128, ('set', [1, 2, 3, 4, 5, 5, 255, 128, 0])), ('p4', 16, ('set', [407095, 407123, 407287, 407491, 407007, 407007, 405991, 408975, 423959, 436855, 449623])), ('p4', 20, ('set', [407095, 407123, 407287, 407491, 407007, 407007, 405991, 408975, 423959, 436855, 449623])), ('p29', 112, ('set', [0, 65536, 131072, 196608, -65536, 65536, 131072])), ('p29', 128, ('set', [1, 2, 3, 4, 5, 5, 255, 128, 0])), ('p5', 16, ('set', [407095, 407123, 407287, 407491, 407007, 407007, 405991, 408975, 423959, 436855, 449623])), ('p5', 20, ('set', [407095, 407123, 407287, 407491, 407007, 407007, 405991, 408975, 423959, 436855, 449623])), ('p33', 112, ('set', [0, 65536, 131072, 196608, -65536, 65536, 131072])), ('p33', 128, ('set', [1, 2, 3, 4, 5, 5, 255, 128, 0])), ('p6', 16, ('set', [407095, 407123, 407287, 407491, 407007, 407007, 405991, 408975, 423959, 436855, 449623])), ('p6', 20, ('set', [407095, 407123, 407287, 407491, 407007, 407007, 405991, 408975, 423959, 436855, 449623])), ('p37', 112, ('set', [0, 65536, 131072, 196608, -65536, 65536, 131072])), ('p37', 128, ('set', [1, 2, 3, 4, 5, 5, 255, 128, 0])), ('p13', 76, ('set', [35122, 35122, 35122, 35122, 35122, 35123, 0])), ('p13', 48, ('set', [4837, 4457, 5981, 5457, 6457, 6457, 265, 23713, 4737, 4736, 4739, 4742, 6072, 6454])), ('p13', 148, ('set', [590390, 590786, 590302, 591334, 592882, 592270, 593302, 591850, 590390, 590390, 590390, 590391])), ('p13', 80, ('set', [35122, 35122, 35122, 35122, 35122, 35123, 0])), ('p13', 52, ('set', [4837, 4457, 5981, 5457, 6457, 6457, 265, 23713, 4737, 4736, 4739, 4742, 6072, 6454])), ('p13', 152, ('set', [590390, 590786, 590302, 591334, 592882, 592270, 593302, 591850, 590390, 590390, 590390, 590391])), ('p13', 84, ('set', [35122, 35122, 35122, 35122, 35122, 35123, 0])), ('p13', 56, ('set', [4837, 4457, 5981, 5457, 6457, 6457, 265, 23713, 4737, 4736, 4739, 4742, 6072, 6454])), ('p13', 156, ('set', [590390, 590786, 590302, 591334, 592882, 592270, 593302, 591850, 590390, 590390, 590390, 590391])), ('p13', 88, ('set', [35122, 35122, 35122, 35122, 35122, 35123, 0])), ('p13', 60, ('set', [4837, 4457, 5981, 5457, 6457, 6457, 265, 23713, 4737, 4736, 4739, 4742, 6072, 6454])), ('p13', 160, ('set', [590390, 590786, 590302, 591334, 592882, 592270, 593302, 591850, 590390, 590390, 590390, 590391])), ('p13', 92, ('set', [35122, 35122, 35122, 35122, 35122, 35123, 0])), ('p13', 64, ('set', [4837, 4457, 5981, 5457, 6457, 6457, 265, 23713, 4737, 4736, 4739, 4742, 6072, 6454])), ('p13', 164, ('set', [590390, 590786, 590302, 591334, 592882, 592270, 593302, 591850, 590390, 590390, 590390, 590391])), ('p13', 96, ('set', [35122, 35122, 35122, 35122, 35122, 35123, 0])), ('p13', 68, ('set', [4837, 4457, 5981, 5457, 6457, 6457, 265, 23713, 4737, 4736, 4739, 4742, 6072, 6454])), ('p13', 168, ('set', [590390, 590786, 590302, 591334, 592882, 592270, 593302, 591850, 590390, 590390, 590390, 590391]))]
FILLS = []
ARGRANGE = []
SLOTBASE = [(184, 6633360), (384, 6548784)]
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2]), (2, [1, 1, 1, 1, 1, 1, 1, 1, 1, 0]), (3, [1, 1, 1, 1, 1, 1, 1, 1, 1, 0]), (6, [1, 1, 1, 1, 1, 1, 1, 0]), (7, [0, 0, 0, 0, 0, 0, 0, 1]), (8, [1, 1, 1, 0, 0]), (9, [0, 0, 0, 0, 1]), (11, [0, 0, 0, 1, 1, 2, 3]), (12, [0, 0, 1, 1]), (14, [1, 1, 1, 1, 1, 1, 0]), (1, [-1, -1, -1, 0, 1, 2, 3]), (16, [0, 1])]
VTAT = None
VTATHALF = []
VTIMAGE = [('p17', 0, 11805044), ('p21', 0, 11805044), ('p25', 0, 11805044), ('p29', 0, 11805044), ('p33', 0, 11805044), ('p37', 0, 11805044)]
ALIAS = []
ANSWERBY = []
ANSWERWIN = []
CDECL = False
TIB = False
ARGEQ = []
SEQ = [(0, 7, None), (2, 6, None), (3, 6, None), (6, 6, None), (7, 6, None), (8, 14, None), (9, 14, None), (11, 6, None), (12, 7, None), (14, 36, None)]
ANSRANGE = [(1, -9, 3)]
CELLTHIS = []
VTALSO = [('p16', 0), ('p18', 0), ('p19', 0), ('p20', 0), ('p22', 0), ('p23', 0), ('p24', 0), ('p26', 0), ('p27', 0), ('p28', 0), ('p30', 0), ('p31', 0), ('p32', 0), ('p34', 0), ('p35', 0), ('p36', 0), ('p38', 0), ('p39', 0), ('p1', 0), ('p2', 0), ('p3', 0), ('p4', 0), ('p5', 0), ('p6', 0)]
SPARSE = []
GTAB = []
RECORD = [1, 12, 17]
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
ECXANSWER = [(4, 32, None), (5, 36, None)]
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
    rng = random.Random(3320)
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
    for i in range(10000):
        h.brk = base  # the bump heap is reused a row; 10000 rows of windows outgrow it
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

# sweep_gen: ["obj", "Unit::find_gather_spot@005f5170", "--item", "3320", "--win", "0x40", "--own", "3", "--cell-win", "0xc0618c:0x1f0", "--cell-win", "0xc06198:0x10", "--cell-win", "0xcae5fc:0x1000", "--cell-win", "0xc061ac:0x20", "--stub", "0x6b52e0:2", "--stub", "0x65b870:5", "--stub", "slot:0xc:0", "--stub", "slot:0x4c:0", "--stub", "slot:0xb0:0", "--stub", "slot:0xac:0", "--stub", "slot:0x90:0", "--stub", "slot:0x180:0", "--stub", "slot:0xb8:2", "--stub", "slot:0x60:2", "--stub", "run:0x46d330:0", "--stub", "0x630450:2", "--stub", "0x608880:3", "--stub", "run:0x46cff0:0", "--stub", "0x6e33a0:2", "--stub", "0x670230:0", "--stub", "0x6e1370:1", "--stub", "0x61a5c0:3", "--ptrtab", "c0+0x14+0x1c*m[9]:6:0x28:vt", "--ptrtab", "0xc09970+0x1c*m[9]:4:0x60", "--place", "c0+0x1ec:0:16", "--place", "0xe41248+0x6eec*m[9]:0:0xb0", "--place", "m+0x18:0:16", "--place", "c3+0x10:0:24", "--place", "p1+0x20:0:0x20", "--place", "p1+0x24:0:0x88", "--place", "p16+0x18:0:0x10", "--place", "p17+0x18:0:0x10", "--vtat-also", "p16+0", "--vtat-also", "p18+0", "--vtat-also", "p19+0", "--vtat-image", "p17+0:0xb42174", "--place", "p2+0x20:0:0x20", "--place", "p2+0x24:0:0x88", "--place", "p20+0x18:0:0x10", "--place", "p21+0x18:0:0x10", "--vtat-also", "p20+0", "--vtat-also", "p22+0", "--vtat-also", "p23+0", "--vtat-image", "p21+0:0xb42174", "--place", "p3+0x20:0:0x20", "--place", "p3+0x24:0:0x88", "--place", "p24+0x18:0:0x10", "--place", "p25+0x18:0:0x10", "--vtat-also", "p24+0", "--vtat-also", "p26+0", "--vtat-also", "p27+0", "--vtat-image", "p25+0:0xb42174", "--place", "p4+0x20:0:0x20", "--place", "p4+0x24:0:0x88", "--place", "p28+0x18:0:0x10", "--place", "p29+0x18:0:0x10", "--vtat-also", "p28+0", "--vtat-also", "p30+0", "--vtat-also", "p31+0", "--vtat-image", "p29+0:0xb42174", "--place", "p5+0x20:0:0x20", "--place", "p5+0x24:0:0x88", "--place", "p32+0x18:0:0x10", "--place", "p33+0x18:0:0x10", "--vtat-also", "p32+0", "--vtat-also", "p34+0", "--vtat-also", "p35+0", "--vtat-image", "p33+0:0xb42174", "--place", "p6+0x20:0:0x20", "--place", "p6+0x24:0:0x88", "--place", "p36+0x18:0:0x10", "--place", "p37+0x18:0:0x10", "--vtat-also", "p36+0", "--vtat-also", "p38+0", "--vtat-also", "p39+0", "--vtat-image", "p37+0:0xb42174", "--ecx-answer", "4:0x20", "--ecx-answer", "5:0x24", "--vtat-keep", "0xb8:0x653790", "--vtat-keep", "0x180:0x63ed30", "--wbias", "m+8:0,0x100,0x200,0x300", "--wbias", "m+0x10:407095,407123,407287,407491,407007,407007,405991,408975,423959,436855,449623", "--wbias", "m+0x14:407095,407123,407287,407491,407007,407007,405991,408975,423959,436855,449623", "--wbias", "p14+4:0x34,0x35,0,1,0x33,0x36,0,0,5,0x10", "--argset", "0:-1,0,1,100,500,1000,1500,2000,3000,5000,30000,50000,100000,0x7fffffff,0x7fffffff", "--wbias", "c1+4:0,0,0,1,2", "--wbias", "p12+0:2,3,4,5,6,6,1,0", "--wbias", "p12+4:2,3,4,5,6,6,1,0", "--wbias", "p12+8:2,3,4,5,6,6,1,0", "--wbias", "p12+12:2,3,4,5,6,6,1,0", "--wbias", "p8..11+0x58:0,0x10000,0x20000,0x30000,0x50000", "--wbias", "p8..11+0x5c:0,1,2,3,5", "--wbias", "p1+0x10:407095,407123,407287,407491,407007,407007,405991,408975,423959,436855,449623", "--wbias", "p1+0x14:407095,407123,407287,407491,407007,407007,405991,408975,423959,436855,449623", "--wbias", "p17+0x70:0,0x10000,0x20000,0x30000,-65536,0x10000,0x20000", "--wbias", "p17+0x80:1,2,3,4,5,5,0xff,0x80,0", "--wbias", "p2+0x10:407095,407123,407287,407491,407007,407007,405991,408975,423959,436855,449623", "--wbias", "p2+0x14:407095,407123,407287,407491,407007,407007,405991,408975,423959,436855,449623", "--wbias", "p21+0x70:0,0x10000,0x20000,0x30000,-65536,0x10000,0x20000", "--wbias", "p21+0x80:1,2,3,4,5,5,0xff,0x80,0", "--wbias", "p3+0x10:407095,407123,407287,407491,407007,407007,405991,408975,423959,436855,449623", "--wbias", "p3+0x14:407095,407123,407287,407491,407007,407007,405991,408975,423959,436855,449623", "--wbias", "p25+0x70:0,0x10000,0x20000,0x30000,-65536,0x10000,0x20000", "--wbias", "p25+0x80:1,2,3,4,5,5,0xff,0x80,0", "--wbias", "p4+0x10:407095,407123,407287,407491,407007,407007,405991,408975,423959,436855,449623", "--wbias", "p4+0x14:407095,407123,407287,407491,407007,407007,405991,408975,423959,436855,449623", "--wbias", "p29+0x70:0,0x10000,0x20000,0x30000,-65536,0x10000,0x20000", "--wbias", "p29+0x80:1,2,3,4,5,5,0xff,0x80,0", "--wbias", "p5+0x10:407095,407123,407287,407491,407007,407007,405991,408975,423959,436855,449623", "--wbias", "p5+0x14:407095,407123,407287,407491,407007,407007,405991,408975,423959,436855,449623", "--wbias", "p33+0x70:0,0x10000,0x20000,0x30000,-65536,0x10000,0x20000", "--wbias", "p33+0x80:1,2,3,4,5,5,0xff,0x80,0", "--wbias", "p6+0x10:407095,407123,407287,407491,407007,407007,405991,408975,423959,436855,449623", "--wbias", "p6+0x14:407095,407123,407287,407491,407007,407007,405991,408975,423959,436855,449623", "--wbias", "p37+0x70:0,0x10000,0x20000,0x30000,-65536,0x10000,0x20000", "--wbias", "p37+0x80:1,2,3,4,5,5,0xff,0x80,0", "--wbias", "p13+76:0x8932,0x8932,0x8932,0x8932,0x8932,0x8933,0", "--wbias", "p13+48:4837,4457,5981,5457,6457,6457,265,23713,4737,4736,4739,4742,6072,6454", "--wbias", "p13+148:590390,590786,590302,591334,592882,592270,593302,591850,590390,590390,590390,590391", "--wbias", "p13+80:0x8932,0x8932,0x8932,0x8932,0x8932,0x8933,0", "--wbias", "p13+52:4837,4457,5981,5457,6457,6457,265,23713,4737,4736,4739,4742,6072,6454", "--wbias", "p13+152:590390,590786,590302,591334,592882,592270,593302,591850,590390,590390,590390,590391", "--wbias", "p13+84:0x8932,0x8932,0x8932,0x8932,0x8932,0x8933,0", "--wbias", "p13+56:4837,4457,5981,5457,6457,6457,265,23713,4737,4736,4739,4742,6072,6454", "--wbias", "p13+156:590390,590786,590302,591334,592882,592270,593302,591850,590390,590390,590390,590391", "--wbias", "p13+88:0x8932,0x8932,0x8932,0x8932,0x8932,0x8933,0", "--wbias", "p13+60:4837,4457,5981,5457,6457,6457,265,23713,4737,4736,4739,4742,6072,6454", "--wbias", "p13+160:590390,590786,590302,591334,592882,592270,593302,591850,590390,590390,590390,590391", "--wbias", "p13+92:0x8932,0x8932,0x8932,0x8932,0x8932,0x8933,0", "--wbias", "p13+64:4837,4457,5981,5457,6457,6457,265,23713,4737,4736,4739,4742,6072,6454", "--wbias", "p13+164:590390,590786,590302,591334,592882,592270,593302,591850,590390,590390,590390,590391", "--wbias", "p13+96:0x8932,0x8932,0x8932,0x8932,0x8932,0x8933,0", "--wbias", "p13+68:4837,4457,5981,5457,6457,6457,265,23713,4737,4736,4739,4742,6072,6454", "--wbias", "p13+168:590390,590786,590302,591334,592882,592270,593302,591850,590390,590390,590390,590391", "--seq", "0:7", "--answers", "0:0,0,0,0,0,0,0,0,0,0,0,0,0,1,2", "--seq", "2:6", "--answers", "2:1,1,1,1,1,1,1,1,1,0", "--seq", "3:6", "--answers", "3:1,1,1,1,1,1,1,1,1,0", "--seq", "6:6", "--answers", "6:1,1,1,1,1,1,1,0", "--seq", "7:6", "--answers", "7:0,0,0,0,0,0,0,1", "--seq", "8:14", "--answers", "8:1,1,1,0,0", "--seq", "9:14", "--answers", "9:0,0,0,0,1", "--seq", "11:6", "--answers", "11:0,0,0,1,1,2,3", "--seq", "12:7", "--answers", "12:0,0,1,1", "--seq", "14:36", "--answers", "14:1,1,1,1,1,1,0", "--answer-range", "1:-9:3", "--answers", "1:-1,-1,-1,0,1,2,3", "--answers", "16:0,1", "--record", "1", "--record", "12", "--record", "17", "--record-order", "--rows", "10000", "--port", "{\n                // `a`: the three stack arguments (the first is the range), the eighteen stubs' single answers in the order\n                // the line names them (0 `get_tregion`, 1 `find_city_at`, 2 slot +0xc, 3 slot +0x4c, 4 slot +0xb0 and 5 slot\n                // +0xac (both answered from the object's `ecx`, not read), 6 slot +0x90, 7 slot +0x180, 8 slot +0xb8, 9 slot\n                // +0x60, 10 `is_scholar` and 13 `vector_dist` (run), 11 `num_gatherers`, 12 `is_gathering_at`, 14 `type_avail`,\n                // 15 `best_gather_type`, 16 `has_tribe_bonus`, 17 `add_gather_order`), then the `--seq` runs, then the two\n                // `--vtat-keep` flags (slot +0xb8 is `ObjectData::is`, slot +0x180 `WallData::is_neutralized`).\n                // Windows: `c[0]` the objects singleton (the owner's list at +0x14 + 0x1c * owner), `c[1]` `obj_base`\n                // (start at +4), `p[1 + k]` object k (data at +0x20, build data at +0x24, position xor 0x63637 at +0x10,\n                // +0x14), `p[8 + c]` city c (`free` +0x5a, `gatherers` +0x5c), `p[12]` the owners' object counts, `p[13]` the\n                // ledger (cap +0x30, state +0x4c, income +0x94, a word a good), `p[14]` the unit's type record (+4),\n                // `p[16 + 4k]` object k's data record, `p[17 + 4k]` its build data (vtable word either `Build::vftable` or\n                // the stubbed table; city the short at +0x72, limit the byte at +0x80).\n                //\n                // **The port is the crate's own steps**, `crate::orders::find_gather_spot_steps`, which `Sim::find_gather_spot`\n                // calls over its `GatherSpotSim`; here the view is the emulated original's windows.\n                use crate::orders::{GatherSpotView, find_gather_spot_steps};\n                const K: i32 = 0x63637;\n                // (stub, first answer in `a`, run length): the `--seq` stubs.\n                const SEQS: [(usize, usize, usize); 10] = [\n                    (0, 21, 7),\n                    (2, 28, 6),\n                    (3, 34, 6),\n                    (6, 40, 6),\n                    (7, 46, 6),\n                    (8, 52, 14),\n                    (9, 66, 14),\n                    (11, 80, 6),\n                    (12, 86, 7),\n                    (14, 93, 36),\n                ];\n                struct V<'a> {\n                    a: &'a [i32],\n                    m: &'a [u8],\n                    c: &'a [Vec<u8>],\n                    p: &'a [Vec<u8>],\n                    turn: [usize; 20],\n                    log: &'a mut Vec<String>,\n                }\n                fn rd32(w: &[u8], o: usize) -> i32 {\n                    i32::from_le_bytes(w[o..o + 4].try_into().expect(\"four bytes\"))\n                }\n                fn rd16(w: &[u8], o: usize) -> i32 {\n                    i32::from(i16::from_le_bytes(w[o..o + 2].try_into().expect(\"two bytes\")))\n                }\n                impl V<'_> {\n                    fn owner(&self) -> usize {\n                        usize::from(self.m[9])\n                    }\n                    /// The stub's next answer, the last repeated.\n                    fn next(&mut self, stub: usize) -> i32 {\n                        let &(_, base, n) = SEQS.iter().find(|s| s.0 == stub).expect(\"a seq stub\");\n                        let k = self.turn[stub].min(n - 1);\n                        self.turn[stub] += 1;\n                        self.a[base + k]\n                    }\n                    /// Slot +0xb8 or +0x60 on a record: `ObjectData::is` answers through +0x60.\n                    fn is_slot(&mut self) -> bool {\n                        if self.a[129] != 0 {\n                            self.next(9) != 0\n                        } else {\n                            self.next(8) != 0\n                        }\n                    }\n                }\n                impl GatherSpotView for V<'_> {\n                    fn first(&mut self) -> i32 {\n                        rd32(&self.c[1], 4)\n                    }\n                    fn count(&mut self) -> i32 {\n                        rd32(&self.p[12], 4 * self.owner())\n                    }\n                    fn scholar(&mut self) -> bool {\n                        matches!(rd32(&self.p[14], 4), 0x34 | 0x35)\n                    }\n                    fn region_of_unit(&mut self) -> i32 {\n                        self.next(0)\n                    }\n                    fn city_of_unit(&mut self) -> i32 {\n                        let (x, y) = self.unit_pos();\n                        let o = self.owner();\n                        self.log.push(format!(\"s1:c0+0x0,{x},{y},{o},-1,0\"));\n                        self.a[3 + 1]\n                    }\n                    fn unit_pos(&mut self) -> (i32, i32) {\n                        (rd32(self.m, 0x10) ^ K, rd32(self.m, 0x14) ^ K)\n                    }\n                    fn alive(&mut self, _i: i32) -> bool {\n                        self.next(2) != 0\n                    }\n                    fn active(&mut self, _i: i32) -> bool {\n                        self.next(3) != 0\n                    }\n                    fn gather_type(&mut self, _i: i32) -> bool {\n                        self.next(6) != 0\n                    }\n                    fn neutralized(&mut self, _i: i32) -> bool {\n                        // The kept slot is `WallData::is_neutralized`, which answers no.\n                        self.a[130] == 0 && self.next(7) != 0\n                    }\n                    fn is_university(&mut self, i: i32) -> bool {\n                        if rd32(&self.p[17 + 4 * i as usize], 0) as u32 == 0x00b4_2174 {\n                            self.next(9) != 0\n                        } else {\n                            self.is_slot()\n                        }\n                    }\n                    fn num_gatherers(&mut self, _i: i32) -> i32 {\n                        self.next(11)\n                    }\n                    fn max_gatherers(&mut self, i: i32) -> i32 {\n                        i32::from(self.p[17 + 4 * i as usize][0x80] as i8)\n                    }\n                    fn gathering_at(&mut self, i: i32) -> bool {\n                        let o = self.owner();\n                        self.log.push(format!(\"s12:m+0x0,{i},{o},0\"));\n                        self.next(12) != 0\n                    }\n                    fn region_of(&mut self, _i: i32) -> i32 {\n                        self.next(0)\n                    }\n                    fn city_of(&mut self, i: i32) -> i32 {\n                        rd16(&self.p[17 + 4 * i as usize], 0x72)\n                    }\n                    fn city_pop(&mut self, c: i32) -> i32 {\n                        let w = &self.p[8 + c as usize];\n                        i32::from(w[0x5c]) + i32::from(w[0x5a])\n                    }\n                    fn pos_of(&mut self, i: i32) -> (i32, i32) {\n                        let w = &self.p[1 + i as usize];\n                        (rd32(w, 0x10) ^ K, rd32(w, 0x14) ^ K)\n                    }\n                    fn good_avail(&mut self, _g: usize) -> bool {\n                        self.next(14) != 0\n                    }\n                    fn gathers(&mut self, _i: i32, _g: usize) -> bool {\n                        self.is_slot()\n                    }\n                    fn under_cap(&mut self, g: usize) -> bool {\n                        rd32(&self.p[13], 0x4c + 4 * g) == 0x8932\n                    }\n                    fn headroom(&mut self, g: usize) -> i32 {\n                        (rd32(&self.p[13], 0x30 + 4 * g) ^ 0x1281)\n                            .wrapping_sub(rd32(&self.p[13], 0x94 + 4 * g) ^ 0x90236)\n                    }\n                    fn dutch(&mut self) -> bool {\n                        self.a[3 + 16] != 0\n                    }\n                    fn add_gather_order(&mut self, i: i32) {\n                        let o = self.owner();\n                        let _ = o;\n                        self.log.push(format!(\"s17:m+0x0,{i},1,0\"));\n                    }\n                }\n                let mut v = V {\n                    a,\n                    m: &*m,\n                    c: &*c,\n                    p: &*p,\n                    turn: [0; 20],\n                    log: &mut log,\n                };\n                i64::from(find_gather_spot_steps(&mut v, a[0]))\n            }"]
