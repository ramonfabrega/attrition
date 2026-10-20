#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_check_capture.py — `Build::check_capture@006276a0` under unicorn (item 3356, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_build_check_capture.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 256-byte object of random bytes (no bias), 2 stack argument(s);
`p` windows in order: p0..p23 sparse table 0 (8 rows × 3 slice(s), row-major), p24..p26 sparse table 1 (3 rows × 1 slice(s), row-major), p27..p87 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00cae5fc points at 256 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c061f0 points at 312 random bytes; the global cell 0x00c061ec points at 1364 random bytes; the global cell 0x00c06210 points at 672 random bytes; the global cell 0x00c06208 points at 64 random bytes; 4 random bytes at 0x00c8cd00; callee 0x0062d1d0 (0 args) stubbed; callee 0x00461420 (2 args) stubbed; callee 0x004613d0 (4 args) stubbed; callee 0x0067dbb0 (4 args) stubbed; callee 0x00646d50 (1 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x006ebaa0 (1 args) stubbed; callee 0x00733380 (3 args) stubbed; callee 0x00647080 (3 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x00a1ee30 (0 args) stubbed; callee 0x00a1d440 (1 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1cd60 (2 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x00642fc0 (2 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x120 (0 args) stubbed; vtable slot +0xec (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0x164 (0 args) stubbed; vtable slot +0x1a8 (3 args) stubbed; vtable slot +0x10c (1 args) stubbed; vtable slot +0x94 (1 args) stubbed; vtable slot +0x150 (3 args) stubbed; vtable slot +0x158 (3 args) stubbed; the word at `c0+0x14+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p27+0` points at 96 random bytes less `0` (placed); the word at `p27+4` points at 96 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p30+0` points at 96 random bytes less `0` (placed); the word at `p30+4` points at 96 random bytes less `0` (placed); the word at `c0+0x14+0x1c*2` points at 8 random bytes less `0` (placed); the word at `p33+0` points at 96 random bytes less `0` (placed); the word at `p33+4` points at 96 random bytes less `0` (placed); the word at `0xc09970+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p36+0` points at 128 random bytes less `0` (placed); the word at `p36+4` points at 128 random bytes less `0` (placed); the word at `p36+8` points at 128 random bytes less `0` (placed); the word at `0xc09970+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p40+0` points at 128 random bytes less `0` (placed); the word at `p40+4` points at 128 random bytes less `0` (placed); the word at `p40+8` points at 128 random bytes less `0` (placed); the word at `0xc09970+0x1c*2` points at 12 random bytes less `0` (placed); the word at `p44+0` points at 128 random bytes less `0` (placed); the word at `p44+4` points at 128 random bytes less `0` (placed); the word at `p44+8` points at 128 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p48+0` points at 128 random bytes less `0` (placed); the word at `p48+4` points at 128 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p51+0` points at 128 random bytes less `0` (placed); the word at `p51+4` points at 128 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*2` points at 8 random bytes less `0` (placed); the word at `p54+0` points at 128 random bytes less `0` (placed); the word at `p54+4` points at 128 random bytes less `0` (placed); the word at `c2+0x134` points at 2816 random bytes less `0` (placed); the word at `p28+0x18` points at 56 random bytes less `0x1e4` (placed); the word at `p28+0x20` points at 112 random bytes less `0` (placed); the word at `p28+0x28` points at 32 random bytes less `0` (placed); the word at `p29+0x18` points at 56 random bytes less `0x1e4` (placed); the word at `p29+0x20` points at 112 random bytes less `0` (placed); the word at `p29+0x28` points at 32 random bytes less `0` (placed); the word at `p31+0x18` points at 56 random bytes less `0x1e4` (placed); the word at `p31+0x20` points at 112 random bytes less `0` (placed); the word at `p31+0x28` points at 32 random bytes less `0` (placed); the word at `p32+0x18` points at 56 random bytes less `0x1e4` (placed); the word at `p32+0x20` points at 112 random bytes less `0` (placed); the word at `p32+0x28` points at 32 random bytes less `0` (placed); the word at `p34+0x18` points at 56 random bytes less `0x1e4` (placed); the word at `p34+0x20` points at 112 random bytes less `0` (placed); the word at `p34+0x28` points at 32 random bytes less `0` (placed); the word at `p35+0x18` points at 56 random bytes less `0x1e4` (placed); the word at `p35+0x20` points at 112 random bytes less `0` (placed); the word at `p35+0x28` points at 32 random bytes less `0` (placed); the word at `p59+0x18` points at 16 random bytes less `0` (placed); the word at `p62+0x18` points at 16 random bytes less `0` (placed); the word at `p65+0x18` points at 16 random bytes less `0` (placed); the word at `p68+0x18` points at 16 random bytes less `0` (placed); the word at `p71+0x18` points at 16 random bytes less `0` (placed); the word at `p74+0x18` points at 16 random bytes less `0` (placed); the word at `p60+0x18` points at 8 random bytes less `0x74` (placed); the word at `p63+0x18` points at 8 random bytes less `0x74` (placed); the word at `p66+0x18` points at 8 random bytes less `0x74` (placed); the word at `p69+0x18` points at 8 random bytes less `0x74` (placed); the word at `p72+0x18` points at 8 random bytes less `0x74` (placed); the word at `p75+0x18` points at 8 random bytes less `0x74` (placed); slot +0x10c laid as 0x00645310 or its stub, a flag after the answers; stub 0 answered from [1, 1, 1, 1, 1, 1, 1, 1, 1, 0] (90%); the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 1 answers 21 values in call order, printed after the answers; stub 2 answers 21 values in call order, printed after the answers; stub 3 answers 10 values in call order, printed after the answers; stub 5 answers 12 values in call order, printed after the answers; stub 6 answers 12 values in call order, printed after the answers; stub 1 answers 0..3 every row; stub 2 answers 0..3 every row; stub 3 answers 0..4 every row; stub 4 answers 0..5 every row; stub 5 answers 0..2 every row; stub 6 answers 0..2 every row; stub 7 answers 0..2 every row; stub 23 answers 0..1 every row; stub 27 answers -1..1 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 2 writes a value in [0, 0, 0, 1, 1, 1, -1] through its argument 2, printed after the answers (`o` in a place); stub 2 writes a value in [0, 0, 0, 0, 0, 1, 2] through its argument 3, printed after the answers (`o` in a place); the stack 1024 bytes under the call is zeroed before each row; `0x006817f0` run once before the rows (a table builder the startup would have run); stub 17 answers from its own `ecx`: the word at `+0x30`; stub 18 answers from its own `ecx`: the word at `+0x34`; stub 19 answers from its own `ecx`: the word at `+0x38`; stub 20 answers from its own `ecx`: the word at `+0x3c`; stub 21 answers from its own `ecx`: the word at `+0x20`; stub 22 answers from its own `ecx`: the word at `+0x28`; 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x8:0x4, +0x74:0x20 (`p` windows after the pointed buffers, before the places); 3 rows 0x6eec apart at `0xc061e0`, slices +0x74:0xc (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p59+0` as well as at `this`; the stubbed vtable laid at `p76+0` as well as at `this`; the stubbed vtable laid at `p60+0` as well as at `this`; the stubbed vtable laid at `p62+0` as well as at `this`; the stubbed vtable laid at `p77+0` as well as at `this`; the stubbed vtable laid at `p63+0` as well as at `this`; the stubbed vtable laid at `p65+0` as well as at `this`; the stubbed vtable laid at `p78+0` as well as at `this`; the stubbed vtable laid at `p66+0` as well as at `this`; the stubbed vtable laid at `p68+0` as well as at `this`; the stubbed vtable laid at `p79+0` as well as at `this`; the stubbed vtable laid at `p69+0` as well as at `this`; the stubbed vtable laid at `p71+0` as well as at `this`; the stubbed vtable laid at `p80+0` as well as at `this`; the stubbed vtable laid at `p72+0` as well as at `this`; the stubbed vtable laid at `p74+0` as well as at `this`; the stubbed vtable laid at `p81+0` as well as at `this`; the stubbed vtable laid at `p75+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`.
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

ENTRY = 0x006276a0
WIN = 256
OWN = 2
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607884, 128), (13297148, 256), (12607880, 320), (12607984, 312), (12607980, 1364), (12608016, 672), (12608008, 64)]
AT = [(13159680, 4)]
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('call', 6476240, 0), ('call', 4592672, 2), ('call', 4592592, 4), ('call', 6806448, 4), ('call', 6581584, 1), ('call', 7265104, 1), ('call', 7256736, 1), ('call', 7549824, 3), ('call', 6582400, 3), ('call', 9959280, 1), ('call', 10612272, 0), ('call', 10605632, 1), ('call', 10612400, 1), ('call', 10603872, 2), ('call', 10604352, 0), ('call', 8298416, 10), ('call', 6565824, 2), ('slot', 24, 0), ('slot', 288, 0), ('slot', 236, 0), ('slot', 32, 0), ('slot', 172, 0), ('slot', 176, 0), ('slot', 252, 0), ('slot', 356, 0), ('slot', 424, 3), ('slot', 268, 1), ('slot', 148, 1), ('slot', 336, 3), ('slot', 344, 3)]
PLACES = [('c0', '0x14+0x1c*0', '0', 8), ('p27', '0', '0', 96), ('p27', '4', '0', 96), ('c0', '0x14+0x1c*1', '0', 8), ('p30', '0', '0', 96), ('p30', '4', '0', 96), ('c0', '0x14+0x1c*2', '0', 8), ('p33', '0', '0', 96), ('p33', '4', '0', 96), (None, '0xc09970+0x1c*0', '0', 12), ('p36', '0', '0', 128), ('p36', '4', '0', 128), ('p36', '8', '0', 128), (None, '0xc09970+0x1c*1', '0', 12), ('p40', '0', '0', 128), ('p40', '4', '0', 128), ('p40', '8', '0', 128), (None, '0xc09970+0x1c*2', '0', 12), ('p44', '0', '0', 128), ('p44', '4', '0', 128), ('p44', '8', '0', 128), (None, '0xc0aec0+0x1c*0', '0', 8), ('p48', '0', '0', 128), ('p48', '4', '0', 128), (None, '0xc0aec0+0x1c*1', '0', 8), ('p51', '0', '0', 128), ('p51', '4', '0', 128), (None, '0xc0aec0+0x1c*2', '0', 8), ('p54', '0', '0', 128), ('p54', '4', '0', 128), ('c2', '0x134', '0', 2816), ('p28', '0x18', '0x1e4', 56), ('p28', '0x20', '0', 112), ('p28', '0x28', '0', 32), ('p29', '0x18', '0x1e4', 56), ('p29', '0x20', '0', 112), ('p29', '0x28', '0', 32), ('p31', '0x18', '0x1e4', 56), ('p31', '0x20', '0', 112), ('p31', '0x28', '0', 32), ('p32', '0x18', '0x1e4', 56), ('p32', '0x20', '0', 112), ('p32', '0x28', '0', 32), ('p34', '0x18', '0x1e4', 56), ('p34', '0x20', '0', 112), ('p34', '0x28', '0', 32), ('p35', '0x18', '0x1e4', 56), ('p35', '0x20', '0', 112), ('p35', '0x28', '0', 32), ('p59', '0x18', '0', 16), ('p62', '0x18', '0', 16), ('p65', '0x18', '0', 16), ('p68', '0x18', '0', 16), ('p71', '0x18', '0', 16), ('p74', '0x18', '0', 16), ('p60', '0x18', '0x74', 8), ('p63', '0x18', '0x74', 8), ('p66', '0x18', '0x74', 8), ('p69', '0x18', '0x74', 8), ('p72', '0x18', '0x74', 8), ('p75', '0x18', '0x74', 8)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 8, ('set', [32, 65568, 32, 65568, 32, 65568, 32, 65568, 32, 65568, 288, 65824, 544, 66080, 32, 65568, 32, 65568, 32, 65568, 32, 65568, 32, 65568, 288, 65824, 544, 66080, 32, 65568, 32, 65568, 32, 65568, 32, 65568, 32, 65568, 288, 65824, 544, 66080, 33, 65569, 33, 65569, 33, 65569, 33, 65569, 33, 65569, 289, 65825, 545, 66081, 96, 65632, 96, 65632, 96, 65632, 96, 65632, 96, 65632, 352, 65888, 608, 66144, 0, 65536, 0, 65536, 0, 65536, 0, 65536, 0, 65536, 256, 65792, 512, 66048, 0, 65536, 0, 65536, 0, 65536, 0, 65536, 0, 65536, 256, 65792, 512, 66048, 1, 65537, 1, 65537, 1, 65537, 1, 65537, 1, 65537, 257, 65793, 513, 66049, 64, 65600, 64, 65600, 64, 65600, 64, 65600, 64, 65600, 320, 65856, 576, 66112])), ('m', 16, ('set', [407819, 408011, 403083, 403275, 402443, 402635, 402827, 401995])), ('m', 20, ('set', [407819, 408011, 403083, 403275, 402443, 402635, 402827, 401995])), ('m', 40, ('set', [65535, 65535, 65535, -1, -1, -1, 393214, -65536, -327679, 0])), ('m', 112, ('set', [0, 65536, 131072])), ('c2', 0, ('set', [9, 10])), ('c3', 24, ('set', [3, 4, 5, 6, 6, 8, 8, 8, 12])), ('c3', 308, ('set', [3, 4, 5, 6, 6, 8, 8, 8, 12])), ('c4', 1360, ('set', [5000])), ('c5', 664, ('set', [0, 1, 2, -1, 3])), ('p57', None, ('set', [0, 0, 0, 0, 0, 0, 1])), ('p0', 0, ('set', [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 1, 7])), ('p1', 0, ('set', [0, 0, 0, 1, 0, 1, 2])), ('p2', None, ('set', [0, 0, 1, 2])), ('p3', 0, ('set', [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 1, 7])), ('p4', 0, ('set', [1, 1, 1, 2, 0, 1, 2])), ('p5', None, ('set', [0, 0, 1, 2])), ('p6', 0, ('set', [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 1, 7])), ('p7', 0, ('set', [2, 2, 2, 0, 0, 1, 2])), ('p8', None, ('set', [0, 0, 1, 2])), ('p9', 0, ('set', [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 1, 7])), ('p10', 0, ('set', [0, 0, 0, 1, 0, 1, 2])), ('p11', None, ('set', [0, 0, 1, 2])), ('p12', 0, ('set', [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 1, 7])), ('p13', 0, ('set', [1, 1, 1, 2, 0, 1, 2])), ('p14', None, ('set', [0, 0, 1, 2])), ('p15', 0, ('set', [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 1, 7])), ('p16', 0, ('set', [2, 2, 2, 0, 0, 1, 2])), ('p17', None, ('set', [0, 0, 1, 2])), ('p18', 0, ('set', [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 1, 7])), ('p19', 0, ('set', [0, 0, 0, 1, 0, 1, 2])), ('p20', None, ('set', [0, 0, 1, 2])), ('p21', 0, ('set', [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 1, 7])), ('p22', 0, ('set', [1, 1, 1, 2, 0, 1, 2])), ('p23', None, ('set', [0, 0, 1, 2])), ('p24', None, ('set', [0, 0, 1, 2])), ('p25', None, ('set', [0, 0, 1, 2])), ('p26', None, ('set', [0, 0, 1, 2])), ('p37', 32, ('set', [5000, 4990, 4930, 4926, 4925, 4924, 4925, 4500, 4101, 4100, 4099, 3000, 0, 4500, 3000])), ('p37', 40, ('set', [0, 1, 2, 3, 5, 8, 12])), ('p37', 92, ('set', [0, 16777216, 33554432])), ('p37', 96, ('set', [0, 1, 2])), ('p38', 32, ('set', [5000, 4990, 4930, 4926, 4925, 4924, 4925, 4500, 4101, 4100, 4099, 3000, 0, 4500, 3000])), ('p38', 40, ('set', [0, 1, 2, 3, 5, 8, 12])), ('p38', 92, ('set', [0, 16777216, 33554432])), ('p38', 96, ('set', [0, 1, 2])), ('p39', 32, ('set', [5000, 4990, 4930, 4926, 4925, 4924, 4925, 4500, 4101, 4100, 4099, 3000, 0, 4500, 3000])), ('p39', 40, ('set', [0, 1, 2, 3, 5, 8, 12])), ('p39', 92, ('set', [0, 16777216, 33554432])), ('p39', 96, ('set', [0, 1, 2])), ('p41', 32, ('set', [5000, 4990, 4930, 4926, 4925, 4924, 4925, 4500, 4101, 4100, 4099, 3000, 0, 4500, 3000])), ('p41', 40, ('set', [0, 1, 2, 3, 5, 8, 12])), ('p41', 92, ('set', [0, 16777216, 33554432])), ('p41', 96, ('set', [0, 1, 2])), ('p42', 32, ('set', [5000, 4990, 4930, 4926, 4925, 4924, 4925, 4500, 4101, 4100, 4099, 3000, 0, 4500, 3000])), ('p42', 40, ('set', [0, 1, 2, 3, 5, 8, 12])), ('p42', 92, ('set', [0, 16777216, 33554432])), ('p42', 96, ('set', [0, 1, 2])), ('p43', 32, ('set', [5000, 4990, 4930, 4926, 4925, 4924, 4925, 4500, 4101, 4100, 4099, 3000, 0, 4500, 3000])), ('p43', 40, ('set', [0, 1, 2, 3, 5, 8, 12])), ('p43', 92, ('set', [0, 16777216, 33554432])), ('p43', 96, ('set', [0, 1, 2])), ('p45', 32, ('set', [5000, 4990, 4930, 4926, 4925, 4924, 4925, 4500, 4101, 4100, 4099, 3000, 0, 4500, 3000])), ('p45', 40, ('set', [0, 1, 2, 3, 5, 8, 12])), ('p45', 92, ('set', [0, 16777216, 33554432])), ('p45', 96, ('set', [0, 1, 2])), ('p46', 32, ('set', [5000, 4990, 4930, 4926, 4925, 4924, 4925, 4500, 4101, 4100, 4099, 3000, 0, 4500, 3000])), ('p46', 40, ('set', [0, 1, 2, 3, 5, 8, 12])), ('p46', 92, ('set', [0, 16777216, 33554432])), ('p46', 96, ('set', [0, 1, 2])), ('p47', 32, ('set', [5000, 4990, 4930, 4926, 4925, 4924, 4925, 4500, 4101, 4100, 4099, 3000, 0, 4500, 3000])), ('p47', 40, ('set', [0, 1, 2, 3, 5, 8, 12])), ('p47', 92, ('set', [0, 16777216, 33554432])), ('p47', 96, ('set', [0, 1, 2])), ('p49', 104, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 256])), ('p50', 104, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 256])), ('p52', 104, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 256])), ('p53', 104, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 256])), ('p55', 104, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 256])), ('p56', 104, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 256])), ('p28', 8, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 0, 3])), ('p28', 16, ('set', [407819, 408011, 403083, 403275, 402443, 402635, 402827, 401995])), ('p28', 20, ('set', [407819, 408011, 403083, 403275, 402443, 402635, 402827, 401995])), ('p28', 44, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65536, 131072, 1, 1, 1, 65537, 131073])), ('p28', 48, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0])), ('p28', 52, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0])), ('p28', 56, ('set', [1, 3, 5, 8, 10, 12, 16, 0, 6])), ('p28', 60, ('set', [0, 1, 1, 1])), ('p58', 0, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 134217728])), ('p58', 52, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2])), ('p59', 64, ('set', [0, 1, 2, 4, 3, 6, 7, 255])), ('p29', 8, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 0, 3])), ('p29', 16, ('set', [407819, 408011, 403083, 403275, 402443, 402635, 402827, 401995])), ('p29', 20, ('set', [407819, 408011, 403083, 403275, 402443, 402635, 402827, 401995])), ('p29', 44, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65537, 131073])), ('p29', 48, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0])), ('p29', 52, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0])), ('p29', 56, ('set', [1, 3, 5, 8, 10, 12, 16, 0, 6])), ('p29', 60, ('set', [0, 1, 1, 1])), ('p61', 0, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 134217728])), ('p61', 52, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2])), ('p62', 64, ('set', [0, 1, 2, 4, 3, 6, 7, 255])), ('p31', 8, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 0, 3])), ('p31', 16, ('set', [407819, 408011, 403083, 403275, 402443, 402635, 402827, 401995])), ('p31', 20, ('set', [407819, 408011, 403083, 403275, 402443, 402635, 402827, 401995])), ('p31', 44, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 131072, 1, 1, 1, 65537, 131073])), ('p31', 48, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0])), ('p31', 52, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0])), ('p31', 56, ('set', [1, 3, 5, 8, 10, 12, 16, 0, 6])), ('p31', 60, ('set', [0, 1, 1, 1])), ('p64', 0, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 134217728])), ('p64', 52, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2])), ('p65', 64, ('set', [0, 1, 2, 4, 3, 6, 7, 255])), ('p32', 8, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 0, 3])), ('p32', 16, ('set', [407819, 408011, 403083, 403275, 402443, 402635, 402827, 401995])), ('p32', 20, ('set', [407819, 408011, 403083, 403275, 402443, 402635, 402827, 401995])), ('p32', 44, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 131073])), ('p32', 48, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0])), ('p32', 52, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0])), ('p32', 56, ('set', [1, 3, 5, 8, 10, 12, 16, 0, 6])), ('p32', 60, ('set', [0, 1, 1, 1])), ('p67', 0, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 134217728])), ('p67', 52, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2])), ('p68', 64, ('set', [0, 1, 2, 4, 3, 6, 7, 255])), ('p34', 8, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 0, 3])), ('p34', 16, ('set', [407819, 408011, 403083, 403275, 402443, 402635, 402827, 401995])), ('p34', 20, ('set', [407819, 408011, 403083, 403275, 402443, 402635, 402827, 401995])), ('p34', 44, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 1, 1, 1, 65537, 131073])), ('p34', 48, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0])), ('p34', 52, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0])), ('p34', 56, ('set', [1, 3, 5, 8, 10, 12, 16, 0, 6])), ('p34', 60, ('set', [0, 1, 1, 1])), ('p70', 0, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 134217728])), ('p70', 52, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2])), ('p71', 64, ('set', [0, 1, 2, 4, 3, 6, 7, 255])), ('p35', 8, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 0, 3])), ('p35', 16, ('set', [407819, 408011, 403083, 403275, 402443, 402635, 402827, 401995])), ('p35', 20, ('set', [407819, 408011, 403083, 403275, 402443, 402635, 402827, 401995])), ('p35', 44, ('set', [65535, 65535, 65535, 65535, 65535, 65535])), ('p35', 48, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0])), ('p35', 52, ('set', [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0])), ('p35', 56, ('set', [1, 3, 5, 8, 10, 12, 16, 0, 6])), ('p35', 60, ('set', [0, 1, 1, 1])), ('p73', 0, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 134217728])), ('p73', 52, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2])), ('p74', 64, ('set', [0, 1, 2, 4, 3, 6, 7, 255]))]
FILLS = [('c1', 'i//3')]
ARGRANGE = [(0, 0, 1), (1, 0, 2)]
SLOTBASE = [(268, 6574864)]
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [1, 1, 1, 1, 1, 1, 1, 1, 1, 0])]
VTAT = None
VTATHALF = []
VTIMAGE = [('m', 0, 11805044)]
ALIAS = []
ANSWERBY = []
ANSWERWIN = []
CDECL = False
TIB = True
ARGEQ = []
SEQ = [(1, 21, None), (2, 21, None), (3, 10, None), (5, 12, None), (6, 12, None)]
ANSRANGE = [(1, 0, 3), (2, 0, 3), (3, 0, 4), (4, 0, 5), (5, 0, 2), (6, 0, 2), (7, 0, 2), (23, 0, 1), (27, -1, 1)]
CELLTHIS = []
VTALSO = [('p59', 0), ('p76', 0), ('p60', 0), ('p62', 0), ('p77', 0), ('p63', 0), ('p65', 0), ('p78', 0), ('p66', 0), ('p68', 0), ('p79', 0), ('p69', 0), ('p71', 0), ('p80', 0), ('p72', 0), ('p74', 0), ('p81', 0), ('p75', 0), ('p28', 0), ('p29', 0), ('p31', 0), ('p32', 0), ('p34', 0), ('p35', 0)]
SPARSE = [('@', '0xe3a390', 28396, 8, [(0, 4), (8, 4), (116, 32)]), (None, '0xc061e0', 28396, 3, [(116, 12)])]
GTAB = []
RECORD = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29]
RECORDORDER = True
RECORDEDX = False
ROWSPLIT = {}
LINKS = []
LINKHI = []
OUTARG = [(2, 2, ('set', [0, 0, 0, 1, 1, 1, -1])), (2, 3, ('set', [0, 0, 0, 0, 0, 1, 2]))]
CDECLSTUBS = []
STUBWRITE = []
STUBCOPY = []
STACKCLEAN = 1024
INITCALLS = [6821872]
MAXINSNS = 200000
ECXANSWER = [(17, 48, None), (18, 52, None), (19, 56, None), (20, 60, None), (21, 32, None), (22, 40, None)]
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
    for i in range(3000):
        h.brk = base  # the bump heap is reused a row; 3000 rows of windows outgrow it
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

# sweep_gen: ["obj", "Build::check_capture@006276a0", "--item=3356", "--name=build_check_capture", "--rows=3000", "--own=2", "--win=0x100", "--clean-stack=0x400", "--tib", "--init-call=0x6817f0", "--argrange=0:0:1", "--argrange=1:0:2", "--cell-win=0xc0618c:0x80", "--cell-win=0xcae5fc:0x100", "--cell-win=0xc06188:0x140", "--cell-win=0xc061f0:0x138", "--cell-win=0xc061ec:0x554", "--cell-win=0xc06210:0x2a0", "--cell-win=0xc06208:0x40", "--at=0xc8cd00:4", "--fill=c1:i//3", "--sparse=@0xe3a390:0x6eec:8:0,4/8,4/0x74,0x20", "--sparse=0xc061e0:0x6eec:3:0x74,0xc", "--ptrtab=c0+0x14+0x1c*0:2:0x60:vt", "--ptrtab=c0+0x14+0x1c*1:2:0x60:vt", "--ptrtab=c0+0x14+0x1c*2:2:0x60:vt", "--ptrtab=0xc09970+0x1c*0:3:0x80", "--ptrtab=0xc09970+0x1c*1:3:0x80", "--ptrtab=0xc09970+0x1c*2:3:0x80", "--ptrtab=0xc0aec0+0x1c*0:2:0x80", "--ptrtab=0xc0aec0+0x1c*1:2:0x80", "--ptrtab=0xc0aec0+0x1c*2:2:0x80", "--place=c2+0x134:0:0xb00", "--place=p28+0x18:0x1e4:0x38", "--place=p28+0x20:0:0x70", "--place=p28+0x28:0:0x20", "--place=p29+0x18:0x1e4:0x38", "--place=p29+0x20:0:0x70", "--place=p29+0x28:0:0x20", "--place=p31+0x18:0x1e4:0x38", "--place=p31+0x20:0:0x70", "--place=p31+0x28:0:0x20", "--place=p32+0x18:0x1e4:0x38", "--place=p32+0x20:0:0x70", "--place=p32+0x28:0:0x20", "--place=p34+0x18:0x1e4:0x38", "--place=p34+0x20:0:0x70", "--place=p34+0x28:0:0x20", "--place=p35+0x18:0x1e4:0x38", "--place=p35+0x20:0:0x70", "--place=p35+0x28:0:0x20", "--place=p59+0x18:0:0x10", "--place=p62+0x18:0:0x10", "--place=p65+0x18:0:0x10", "--place=p68+0x18:0:0x10", "--place=p71+0x18:0:0x10", "--place=p74+0x18:0:0x10", "--place=p60+0x18:0x74:8", "--place=p63+0x18:0x74:8", "--place=p66+0x18:0x74:8", "--place=p69+0x18:0x74:8", "--place=p72+0x18:0x74:8", "--place=p75+0x18:0x74:8", "--stub=0x62d1d0:0", "--stub=0x461420:2", "--stub=0x4613d0:4", "--stub=0x67dbb0:4", "--stub=0x646d50:1", "--stub=0x6edb50:1", "--stub=0x6ebaa0:1", "--stub=0x733380:3", "--stub=0x647080:3", "--stub=0x97f770:1", "--stub=0xa1ee30:0", "--stub=0xa1d440:1", "--stub=0xa1eeb0:1", "--stub=0xa1cd60:2", "--stub=0xa1cf40:0", "--stub=0x7e9fb0:10", "--stub=0x642fc0:2", "--stub=slot:0x18:0", "--stub=slot:0x120:0", "--stub=slot:0xec:0", "--stub=slot:0x20:0", "--stub=slot:0xac:0", "--stub=slot:0xb0:0", "--stub=slot:0xfc:0", "--stub=slot:0x164:0", "--stub=slot:0x1a8:3", "--stub=slot:0x10c:1", "--stub=slot:0x94:1", "--stub=slot:0x150:3", "--stub=slot:0x158:3", "--answers=0:1,1,1,1,1,1,1,1,1,0", "--answer-range=1:0:3", "--seq=1:21", "--answer-range=2:0:3", "--seq=2:21", "--outarg=2:2:0,0,0,1,1,1,-1", "--outarg=2:3:0,0,0,0,0,1,2", "--answer-range=3:0:4", "--seq=3:10", "--answer-range=4:0:5", "--answer-range=5:0:2", "--seq=5:12", "--answer-range=6:0:2", "--seq=6:12", "--answer-range=7:0:2", "--answer-range=23:0:1", "--answer-range=27:-1:1", "--ecx-answer=17:0x30", "--ecx-answer=18:0x34", "--ecx-answer=19:0x38", "--ecx-answer=20:0x3c", "--ecx-answer=21:0x20", "--ecx-answer=22:0x28", "--vtat-keep=0x10c:0x645310", "--vtat-image=m+0:0xb42174", "--record=0", "--record=1", "--record=2", "--record=3", "--record=4", "--record=5", "--record=6", "--record=7", "--record=8", "--record=9", "--record=10", "--record=11", "--record=12", "--record=13", "--record=14", "--record=15", "--record=16", "--record=17", "--record=18", "--record=19", "--record=20", "--record=21", "--record=22", "--record=23", "--record=24", "--record=25", "--record=26", "--record=27", "--record=28", "--record=29", "--record-order", "--vtat-also=p59+0", "--vtat-also=p76+0", "--vtat-also=p60+0", "--vtat-also=p62+0", "--vtat-also=p77+0", "--vtat-also=p63+0", "--vtat-also=p65+0", "--vtat-also=p78+0", "--vtat-also=p66+0", "--vtat-also=p68+0", "--vtat-also=p79+0", "--vtat-also=p69+0", "--vtat-also=p71+0", "--vtat-also=p80+0", "--vtat-also=p72+0", "--vtat-also=p74+0", "--vtat-also=p81+0", "--vtat-also=p75+0", "--wbias=m+0x8:32,65568,32,65568,32,65568,32,65568,32,65568,288,65824,544,66080,32,65568,32,65568,32,65568,32,65568,32,65568,288,65824,544,66080,32,65568,32,65568,32,65568,32,65568,32,65568,288,65824,544,66080,33,65569,33,65569,33,65569,33,65569,33,65569,289,65825,545,66081,96,65632,96,65632,96,65632,96,65632,96,65632,352,65888,608,66144,0,65536,0,65536,0,65536,0,65536,0,65536,256,65792,512,66048,0,65536,0,65536,0,65536,0,65536,0,65536,256,65792,512,66048,1,65537,1,65537,1,65537,1,65537,1,65537,257,65793,513,66049,64,65600,64,65600,64,65600,64,65600,64,65600,320,65856,576,66112", "--wbias=m+0x10:407819,408011,403083,403275,402443,402635,402827,401995", "--wbias=m+0x14:407819,408011,403083,403275,402443,402635,402827,401995", "--wbias=m+0x28:65535,65535,65535,-1,-1,-1,393214,-65536,-327679,0", "--wbias=m+0x70:0,65536,131072", "--wbias=c2+0:9,10", "--wbias=c3+0x18:3,4,5,6,6,8,8,8,12", "--wbias=c3+0x134:3,4,5,6,6,8,8,8,12", "--wbias=c4+0x550:5000", "--wbias=c5+0x298:0,1,2,-1,3", "--wbias=p57+*:0,0,0,0,0,0,1", "--wbias=p0+0:3,3,3,3,3,3,3,3,3,3,2,1,7", "--wbias=p1+0:0,0,0,1,0,1,2", "--wbias=p2+*:0,0,1,2", "--wbias=p3+0:3,3,3,3,3,3,3,3,3,3,2,1,7", "--wbias=p4+0:1,1,1,2,0,1,2", "--wbias=p5+*:0,0,1,2", "--wbias=p6+0:3,3,3,3,3,3,3,3,3,3,2,1,7", "--wbias=p7+0:2,2,2,0,0,1,2", "--wbias=p8+*:0,0,1,2", "--wbias=p9+0:3,3,3,3,3,3,3,3,3,3,2,1,7", "--wbias=p10+0:0,0,0,1,0,1,2", "--wbias=p11+*:0,0,1,2", "--wbias=p12+0:3,3,3,3,3,3,3,3,3,3,2,1,7", "--wbias=p13+0:1,1,1,2,0,1,2", "--wbias=p14+*:0,0,1,2", "--wbias=p15+0:3,3,3,3,3,3,3,3,3,3,2,1,7", "--wbias=p16+0:2,2,2,0,0,1,2", "--wbias=p17+*:0,0,1,2", "--wbias=p18+0:3,3,3,3,3,3,3,3,3,3,2,1,7", "--wbias=p19+0:0,0,0,1,0,1,2", "--wbias=p20+*:0,0,1,2", "--wbias=p21+0:3,3,3,3,3,3,3,3,3,3,2,1,7", "--wbias=p22+0:1,1,1,2,0,1,2", "--wbias=p23+*:0,0,1,2", "--wbias=p24+*:0,0,1,2", "--wbias=p25+*:0,0,1,2", "--wbias=p26+*:0,0,1,2", "--wbias=p37+0x20:5000,4990,4930,4926,4925,4924,4925,4500,4101,4100,4099,3000,0,4500,3000", "--wbias=p37+0x28:0,1,2,3,5,8,12", "--wbias=p37+0x5c:0,16777216,33554432", "--wbias=p37+0x60:0,1,2", "--wbias=p38+0x20:5000,4990,4930,4926,4925,4924,4925,4500,4101,4100,4099,3000,0,4500,3000", "--wbias=p38+0x28:0,1,2,3,5,8,12", "--wbias=p38+0x5c:0,16777216,33554432", "--wbias=p38+0x60:0,1,2", "--wbias=p39+0x20:5000,4990,4930,4926,4925,4924,4925,4500,4101,4100,4099,3000,0,4500,3000", "--wbias=p39+0x28:0,1,2,3,5,8,12", "--wbias=p39+0x5c:0,16777216,33554432", "--wbias=p39+0x60:0,1,2", "--wbias=p41+0x20:5000,4990,4930,4926,4925,4924,4925,4500,4101,4100,4099,3000,0,4500,3000", "--wbias=p41+0x28:0,1,2,3,5,8,12", "--wbias=p41+0x5c:0,16777216,33554432", "--wbias=p41+0x60:0,1,2", "--wbias=p42+0x20:5000,4990,4930,4926,4925,4924,4925,4500,4101,4100,4099,3000,0,4500,3000", "--wbias=p42+0x28:0,1,2,3,5,8,12", "--wbias=p42+0x5c:0,16777216,33554432", "--wbias=p42+0x60:0,1,2", "--wbias=p43+0x20:5000,4990,4930,4926,4925,4924,4925,4500,4101,4100,4099,3000,0,4500,3000", "--wbias=p43+0x28:0,1,2,3,5,8,12", "--wbias=p43+0x5c:0,16777216,33554432", "--wbias=p43+0x60:0,1,2", "--wbias=p45+0x20:5000,4990,4930,4926,4925,4924,4925,4500,4101,4100,4099,3000,0,4500,3000", "--wbias=p45+0x28:0,1,2,3,5,8,12", "--wbias=p45+0x5c:0,16777216,33554432", "--wbias=p45+0x60:0,1,2", "--wbias=p46+0x20:5000,4990,4930,4926,4925,4924,4925,4500,4101,4100,4099,3000,0,4500,3000", "--wbias=p46+0x28:0,1,2,3,5,8,12", "--wbias=p46+0x5c:0,16777216,33554432", "--wbias=p46+0x60:0,1,2", "--wbias=p47+0x20:5000,4990,4930,4926,4925,4924,4925,4500,4101,4100,4099,3000,0,4500,3000", "--wbias=p47+0x28:0,1,2,3,5,8,12", "--wbias=p47+0x5c:0,16777216,33554432", "--wbias=p47+0x60:0,1,2", "--wbias=p49+0x68:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,2,256", "--wbias=p50+0x68:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,2,256", "--wbias=p52+0x68:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,2,256", "--wbias=p53+0x68:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,2,256", "--wbias=p55+0x68:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,2,256", "--wbias=p56+0x68:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,2,256", "--wbias=p28+0x8:1,1,1,1,1,1,1,1,0,3", "--wbias=p28+0x10:407819,408011,403083,403275,402443,402635,402827,401995", "--wbias=p28+0x14:407819,408011,403083,403275,402443,402635,402827,401995", "--wbias=p28+0x2c:65535,65535,65535,65535,65535,65535,65536,131072,1,1,1,65537,131073", "--wbias=p28+0x30:1,1,1,1,1,1,1,1,1,1,1,1,1,1,0", "--wbias=p28+0x34:1,1,1,1,1,1,1,1,1,1,1,1,1,1,0", "--wbias=p28+0x38:1,3,5,8,10,12,16,0,6", "--wbias=p28+0x3c:0,1,1,1", "--wbias=p58+0:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,134217728", "--wbias=p58+0x34:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,2", "--wbias=p59+0x40:0,1,2,4,3,6,7,255", "--wbias=p29+0x8:1,1,1,1,1,1,1,1,0,3", "--wbias=p29+0x10:407819,408011,403083,403275,402443,402635,402827,401995", "--wbias=p29+0x14:407819,408011,403083,403275,402443,402635,402827,401995", "--wbias=p29+0x2c:65535,65535,65535,65535,65535,65535,65537,131073", "--wbias=p29+0x30:1,1,1,1,1,1,1,1,1,1,1,1,1,1,0", "--wbias=p29+0x34:1,1,1,1,1,1,1,1,1,1,1,1,1,1,0", "--wbias=p29+0x38:1,3,5,8,10,12,16,0,6", "--wbias=p29+0x3c:0,1,1,1", "--wbias=p61+0:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,134217728", "--wbias=p61+0x34:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,2", "--wbias=p62+0x40:0,1,2,4,3,6,7,255", "--wbias=p31+0x8:1,1,1,1,1,1,1,1,0,3", "--wbias=p31+0x10:407819,408011,403083,403275,402443,402635,402827,401995", "--wbias=p31+0x14:407819,408011,403083,403275,402443,402635,402827,401995", "--wbias=p31+0x2c:65535,65535,65535,65535,65535,65535,131072,1,1,1,65537,131073", "--wbias=p31+0x30:1,1,1,1,1,1,1,1,1,1,1,1,1,1,0", "--wbias=p31+0x34:1,1,1,1,1,1,1,1,1,1,1,1,1,1,0", "--wbias=p31+0x38:1,3,5,8,10,12,16,0,6", "--wbias=p31+0x3c:0,1,1,1", "--wbias=p64+0:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,134217728", "--wbias=p64+0x34:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,2", "--wbias=p65+0x40:0,1,2,4,3,6,7,255", "--wbias=p32+0x8:1,1,1,1,1,1,1,1,0,3", "--wbias=p32+0x10:407819,408011,403083,403275,402443,402635,402827,401995", "--wbias=p32+0x14:407819,408011,403083,403275,402443,402635,402827,401995", "--wbias=p32+0x2c:65535,65535,65535,65535,65535,65535,131073", "--wbias=p32+0x30:1,1,1,1,1,1,1,1,1,1,1,1,1,1,0", "--wbias=p32+0x34:1,1,1,1,1,1,1,1,1,1,1,1,1,1,0", "--wbias=p32+0x38:1,3,5,8,10,12,16,0,6", "--wbias=p32+0x3c:0,1,1,1", "--wbias=p67+0:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,134217728", "--wbias=p67+0x34:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,2", "--wbias=p68+0x40:0,1,2,4,3,6,7,255", "--wbias=p34+0x8:1,1,1,1,1,1,1,1,0,3", "--wbias=p34+0x10:407819,408011,403083,403275,402443,402635,402827,401995", "--wbias=p34+0x14:407819,408011,403083,403275,402443,402635,402827,401995", "--wbias=p34+0x2c:65535,65535,65535,65535,65535,65535,1,1,1,65537,131073", "--wbias=p34+0x30:1,1,1,1,1,1,1,1,1,1,1,1,1,1,0", "--wbias=p34+0x34:1,1,1,1,1,1,1,1,1,1,1,1,1,1,0", "--wbias=p34+0x38:1,3,5,8,10,12,16,0,6", "--wbias=p34+0x3c:0,1,1,1", "--wbias=p70+0:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,134217728", "--wbias=p70+0x34:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,2", "--wbias=p71+0x40:0,1,2,4,3,6,7,255", "--wbias=p35+0x8:1,1,1,1,1,1,1,1,0,3", "--wbias=p35+0x10:407819,408011,403083,403275,402443,402635,402827,401995", "--wbias=p35+0x14:407819,408011,403083,403275,402443,402635,402827,401995", "--wbias=p35+0x2c:65535,65535,65535,65535,65535,65535", "--wbias=p35+0x30:1,1,1,1,1,1,1,1,1,1,1,1,1,1,0", "--wbias=p35+0x34:1,1,1,1,1,1,1,1,1,1,1,1,1,1,0", "--wbias=p35+0x38:1,3,5,8,10,12,16,0,6", "--wbias=p35+0x3c:0,1,1,1", "--wbias=p73+0:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,134217728", "--wbias=p73+0x34:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,2", "--wbias=p74+0x40:0,1,2,4,3,6,7,255", "--port", "// Row: `a[0]`, `a[1]` the arguments (the captor's index in `objects[who]`, and its player);\n// `a[2 + i]` stub `i`'s answer; `a[32]`, `a[33]` the index and player `get_down` writes; then the seq\n// values in flag order \u2014 `a[34..55]` `valid`'s 21, `a[55..76]` `get_down`'s 21, `a[76..86]`\n// `valid_filter`'s 10, `a[86..98]` `is_ally`'s 12, `a[98..110]` `is_enemy`'s 12; then `a[110]`, whether\n// slot `+0x10c` held `ObjectData::say_name` (a native read, the caller inlines it).\n// Stubs: 0 `check_capture_eligible`, 1 `WorldData::valid`, 2 `get_down`, 3 `Search::valid_filter`, 4\n// `ObjectData::num_inside`, 5 `is_ally`, 6 `is_enemy`, 7 `Cities::capture_city`, 8 `Object::die`, 9\n// `SoundGlobal::play`, 10 `String::String`, 11 `String::operator+=`, 12 `operator=`, 13 `String::parse`, 14\n// `String::close`, 15 `MessageWin::add_message`, 16 `Wall::mask_me`, then the slots `+0x18`, `+0x120`, `+0xec`,\n// `+0x20`, `+0xac`, `+0xb0`, `+0xfc`, `+0x164`, `+0x1a8`, `+0x10c`, `+0x94`, `+0x150`, `+0x158` (17 to 29).\n// Windows: `m` the building, `c0` the objects cell, `c1` `div_3_table`, `c2` the world, `c3` `constants`,\n// `c4` `game`, `c5` the console, `c6` the message window, `c7` the word at `0xc8cd00`; `p3r + s` leader row\n// `r`'s `+0`, `+8`, `+0x74`; `p24 + r` the constants leaders' `+0x74`; `p27 + 3k` the objects of player\n// `k`, `p28 + 3k + j` object `j`; `p36 + 4k` the cities of `k`, `p37 + 4k + i` city `i`; `p48 + 3k` the units\n// of `k`, `p49 + 3k + j` unit `j`; `p57` the tile array; for object `n = 2k + j`: `p58 + 3n` its type, `p59 +\n// 3n` its `+0xac` record, `p60 + 3n` its `+0xb0` record; `p76 + n` the `+0xac` record's type, `p82 + n` the\n// `+0xb0` record's name.\n//\n// **The port is the crate's own steps**, `crate::capture::check_capture_steps`, which `Sim::check_capture`\n// calls over its `SimCapture`; here the view is the emulated original's windows.\nuse crate::capture::{CaptureView, check_capture_steps};\nconst ROW: i32 = 0x00e3_a390;\nconst MSG: i32 = 0x00ed_6730;\nfn r16(m: &[u8], off: usize) -> i32 {\n    i32::from(i16::from_le_bytes([m[off], m[off + 1]]))\n}\nfn pw(win: usize, off: usize) -> String {\n    format!(\"p{win}+{off:#x}\")\n}\nstruct V<'a> {\n    a: &'a [i32],\n    m: &'a mut [u8],\n    c: &'a [Vec<u8>],\n    p: &'a mut [Vec<u8>],\n    log: &'a mut Vec<String>,\n    nv: usize,\n    ng: usize,\n    nf: usize,\n    na: usize,\n    ne: usize,\n}\nimpl V<'_> {\n    fn obj(h: (i32, i32)) -> usize {\n        28 + 3 * h.0 as usize + h.1 as usize\n    }\n    fn n(h: (i32, i32)) -> usize {\n        2 * h.0 as usize + h.1 as usize\n    }\n    fn seq(&self, base: usize, len: usize, used: usize) -> i32 {\n        self.a[base + used.min(len - 1)]\n    }\n    fn city_win(&self, who: i32, i: i32) -> usize {\n        37 + 4 * who as usize + i as usize\n    }\n    fn my_city(&self) -> usize {\n        self.city_win(i32::from(self.m[9]), r16(self.m, 0x72))\n    }\n}\nimpl CaptureView for V<'_> {\n    type H = (i32, i32);\n    fn eligible(&mut self) -> bool {\n        self.log.push(\"s0:m+0x0\".into());\n        self.a[2] != 0\n    }\n    fn inside_down(&mut self) -> i32 {\n        r16(self.m, 0x28)\n    }\n    fn eject_pending(&mut self) {\n        let h = (i32::from(self.m[9]), r16(self.m, 0xa));\n        self.log.push(format!(\"s21:{}\", pw(Self::obj(h), 0)));\n        let ac = 59 + 3 * Self::n(h);\n        let v = u16::from_le_bytes([self.p[ac][0x60], self.p[ac][0x61]]) | 0x4000;\n        self.p[ac][0x60..0x62].copy_from_slice(&v.to_le_bytes());\n    }\n    fn owner(&mut self) -> i32 {\n        i32::from(self.m[9])\n    }\n    fn building(&mut self) -> (i32, i32) {\n        (i32::from(self.m[9]), r16(self.m, 0xa))\n    }\n    fn centre(&mut self) -> bool {\n        self.m[8] & 0x20 != 0\n    }\n    fn building_pos(&mut self) -> (i32, i32) {\n        (rd(self.m, 0x10) ^ 0x63637, rd(self.m, 0x14) ^ 0x63637)\n    }\n    fn div3(&mut self, i: i32) -> i32 {\n        rd(&self.c[1], 4 * i as usize)\n    }\n    fn leader_id(&mut self, who: i32) -> i32 {\n        rd(&self.p[3 * who as usize + 1], 0)\n    }\n    fn leader_flags(&mut self, who: i32) -> i32 {\n        rd(&self.p[3 * who as usize], 0)\n    }\n    fn relation(&mut self, who: i32, other: i32) -> i32 {\n        rd(&self.p[3 * who as usize + 2], 4 * other as usize)\n    }\n    fn relation_const(&mut self, who: i32, leader: i32) -> i32 {\n        rd(&self.p[24 + who as usize], 4 * leader as usize)\n    }\n    fn is_ally(&mut self, who: i32, other: i32) -> bool {\n        self.log.push(format!(\"s5:{},{other}\", pw(3 * who as usize, 0)));\n        let r = self.seq(86, 12, self.na);\n        self.na += 1;\n        r != 0\n    }\n    fn is_enemy(&mut self, who: i32, other: i32) -> bool {\n        self.log.push(format!(\"s6:{},{other}\", pw(3 * who as usize, 0)));\n        let r = self.seq(98, 12, self.ne);\n        self.ne += 1;\n        r != 0\n    }\n    fn handle(&mut self, who: i32, idx: i32) -> (i32, i32) {\n        (who, idx)\n    }\n    fn kind(&mut self, h: (i32, i32)) -> i32 {\n        rd(&self.p[58 + 3 * Self::n(h)], 0x34)\n    }\n    fn is_unit(&mut self, h: (i32, i32)) -> bool {\n        self.log.push(format!(\"s17:{}\", pw(Self::obj(h), 0)));\n        rd(&self.p[Self::obj(h)], 0x30) != 0\n    }\n    fn decoy(&mut self, h: (i32, i32)) -> bool {\n        rd(&self.p[49 + 3 * h.0 as usize + h.1 as usize], 0x68) & 1 != 0\n    }\n    fn can_capture(&mut self, h: (i32, i32)) -> bool {\n        self.log.push(format!(\"s18:{}\", pw(Self::obj(h), 0)));\n        rd(&self.p[Self::obj(h)], 0x34) != 0\n    }\n    fn missile(&mut self, h: (i32, i32)) -> bool {\n        rd(&self.p[58 + 3 * Self::n(h)], 0) & 0x800_0000 != 0\n    }\n    fn in_use(&mut self, h: (i32, i32)) -> bool {\n        rd(&self.p[Self::obj(h)], 8) & 1 != 0\n    }\n    fn player(&mut self, h: (i32, i32)) -> i32 {\n        h.0\n    }\n    fn valid_filter(&mut self, h: (i32, i32)) -> bool {\n        self.log.push(format!(\"s3:{},{},-1,-1,8\", h.0, h.1));\n        let r = self.seq(76, 10, self.nf);\n        self.nf += 1;\n        r != 0\n    }\n    fn pos(&mut self, h: (i32, i32)) -> (i32, i32) {\n        let o = &self.p[Self::obj(h)];\n        (rd(o, 0x10) ^ 0x63637, rd(o, 0x14) ^ 0x63637)\n    }\n    fn value(&mut self, h: (i32, i32)) -> i32 {\n        self.log.push(format!(\"s19:{}\", pw(Self::obj(h), 0)));\n        rd(&self.p[Self::obj(h)], 0x38)\n    }\n    fn is_building(&mut self, h: (i32, i32)) -> bool {\n        self.log.push(format!(\"s20:{}\", pw(Self::obj(h), 0)));\n        rd(&self.p[Self::obj(h)], 0x3c) != 0\n    }\n    fn garrison_weight(&mut self, h: (i32, i32)) -> i32 {\n        let (o, n) = (Self::obj(h), Self::n(h));\n        self.log.push(format!(\"s21:{}\", pw(o, 0)));\n        self.log.push(format!(\"s23:{}\", pw(76 + n, 0)));\n        let fort = self.a[2 + 23] != 0;\n        self.log.push(format!(\"s21:{}\", pw(o, 0)));\n        self.log.push(format!(\"s4:{},1\", pw(59 + 3 * n, 0)));\n        self.a[2 + 4] + if fort { 12 } else { 6 }\n    }\n    fn reveal(&mut self, h: (i32, i32), who: i32) {\n        let (o, n) = (Self::obj(h), Self::n(h));\n        let ac = 59 + 3 * n;\n        self.log.push(format!(\"s21:{}\", pw(o, 0)));\n        let seen = i32::from(self.p[ac][0x40] as i8);\n        if (1i32.wrapping_shl(who as u32 & 31)) & seen == 0 {\n            self.log.push(format!(\"s21:{}\", pw(o, 0)));\n            self.p[ac][0x40] = (seen | 1i32.wrapping_shl(who as u32 & 31)) as u8;\n            self.log.push(format!(\"s21:{}\", pw(o, 0)));\n            self.log.push(format!(\"s24:{}\", pw(ac, 0)));\n        }\n    }\n    fn city_stamp(&mut self) -> i32 {\n        rd(&self.p[self.my_city()], 0x20)\n    }\n    fn city_strength(&mut self) -> i32 {\n        rd(&self.p[self.my_city()], 0x28)\n    }\n    fn city_index(&mut self) -> i32 {\n        r16(self.m, 0x72)\n    }\n    fn city_founder(&mut self) -> i32 {\n        i32::from(self.p[self.my_city()][0x60] as i8)\n    }\n    fn city_race(&mut self) -> i32 {\n        i32::from(self.p[self.my_city()][0x5f] as i8)\n    }\n    fn frame(&mut self) -> i32 {\n        rd(&self.c[4], 0x550)\n    }\n    fn city_capture_radius(&mut self) -> i32 {\n        rd(&self.c[3], 0x134)\n    }\n    fn unit_respond_range(&mut self) -> i32 {\n        rd(&self.c[3], 0x18)\n    }\n    fn valid_tile(&mut self, _x: i32, _y: i32) -> bool {\n        self.log.push(\"s1:c2+0x0,sp-0x28,sp-0x2c\".into());\n        let r = self.seq(34, 21, self.nv);\n        self.nv += 1;\n        r != 0\n    }\n    fn get_down(&mut self, _x: i32, _y: i32) -> (bool, Option<(i32, i32)>) {\n        self.log\n            .push(\"s2:c2+0x0,sp-0x28,sp-0x2c,sp-0x38,sp-0x48\".into());\n        let r = self.seq(55, 21, self.ng);\n        self.ng += 1;\n        (r != 0, (self.a[32] >= 0).then_some((self.a[33], self.a[32])))\n    }\n    fn tile_region(&mut self, x: i32, y: i32) -> u16 {\n        let at = (rd(&self.c[2], 0).wrapping_mul(y).wrapping_add(x)) as usize * 0x1c + 4;\n        u16::from_le_bytes([self.p[57][at], self.p[57][at + 1]])\n    }\n    fn next(&mut self, h: (i32, i32)) -> Option<(i32, i32)> {\n        let o = &self.p[Self::obj(h)];\n        (r16(o, 0x2c) >= 0).then(|| (r16(o, 0x2e), r16(o, 0x2c)))\n    }\n    fn capture_city(&mut self, taker: i32, city: i32, owner: i32, ecx: i32) -> i32 {\n        // The stale `ecx`: a leaders row (laid in the sparse table's window) or the city's number.\n        let ecx = if ecx >= ROW && (ecx - ROW) % 0x6eec == 0 {\n            pw(3 * ((ecx - ROW) / 0x6eec) as usize, 0)\n        } else {\n            ecx.to_string()\n        };\n        self.log.push(format!(\"s7:{ecx},{taker},{city},{owner}\"));\n        self.a[2 + 7]\n    }\n    fn set_city_strength(&mut self, taker: i32, city: i32, v: i32) {\n        let w = self.city_win(taker, city);\n        wr(&mut self.p[w], 0x28, v);\n    }\n    fn swap_team(&mut self, who: i32) -> i32 {\n        self.log.push(format!(\"s27:m+0x0,{who}\"));\n        self.a[2 + 27]\n    }\n    fn die(&mut self) {\n        if rd(self.m, 0) == 0xb4_2174 {\n            self.log.push(\"s8:m+0x0,0,-1,0\".into());\n        } else {\n            self.log.push(\"s29:m+0x0,0,-1,0\".into());\n        }\n    }\n    fn console_who(&mut self) -> i32 {\n        rd(&self.c[5], 0x298)\n    }\n    fn play(&mut self, sound: i32) {\n        self.log.push(format!(\"s9:c5+0x0,{sound}\"));\n    }\n    fn announce(&mut self, new: i32, who: i32) {\n        let h = (who, new);\n        let (o, n) = (Self::obj(h), Self::n(h));\n        self.log.push(\"s10:sp-0x78\".into());\n        self.log.push(format!(\"s22:{}\", pw(o, 0)));\n        let data = 60 + 3 * n;\n        if self.a[110] != 0 {\n            self.log.push(format!(\"s11:sp-0x78,{}\", pw(82 + n, 0)));\n        } else {\n            self.log.push(format!(\"s26:{},sp-0x78\", pw(data, 0)));\n        }\n        self.log\n            .push(format!(\"s12:{MSG},{}\", rd(&self.c[7], 0).wrapping_add(0x76c)));\n        self.log.push(format!(\"s13:{MSG},sp-0x8c,sp-0x78\"));\n        self.log.push(format!(\"s12:{MSG},{}\", self.a[2 + 13]));\n        self.log.push(\"s14:sp-0x8c\".into());\n        let ob = &self.p[o];\n        let (x, y) = (rd(ob, 0x10) ^ 0x63637, rd(ob, 0x14) ^ 0x63637);\n        let list = pw(27 + 3 * who as usize, 0);\n        self.log.push(format!(\n            \"s15:c6+0x0,{MSG},{x},{y},0,{},-1,{list},0,0,{list}\",\n            0x00c8_d248\n        ));\n        self.log.push(\"s14:sp-0x78\".into());\n    }\n    fn activate_new(&mut self, new: i32, who: i32) {\n        let h = (who, new);\n        self.log.push(format!(\"s22:{}\", pw(Self::obj(h), 0)));\n        self.log\n            .push(format!(\"s25:{},0,1,0\", pw(60 + 3 * Self::n(h), 0)));\n    }\n    fn close_old(&mut self) {\n        self.log.push(\"s28:m+0x0,0,-1,0\".into());\n    }\n    fn mask_new(&mut self, new: i32, who: i32) {\n        let h = (who, new);\n        self.log.push(format!(\"s22:{}\", pw(Self::obj(h), 0)));\n        self.log\n            .push(format!(\"s16:{},1,2\", pw(60 + 3 * Self::n(h), 0)));\n    }\n}\nlet mut v = V {\n    a,\n    m,\n    c,\n    p,\n    log: &mut log,\n    nv: 0,\n    ng: 0,\n    nf: 0,\n    na: 0,\n    ne: 0,\n};\ni64::from(check_capture_steps(&mut v, a[0], a[1]))"]
