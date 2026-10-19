#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_calc_gather.py — `UnitData::calc_gather@00609180` under unicorn (item 3330, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_unit_data_calc_gather.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 176-byte object of random bytes (no bias), 10 stack argument(s);
`p` windows in order: p0..p5 pointed argument(s), p6..p13 sparse table 0 (4 rows × 2 slice(s), row-major), p14..p38 the place(s), in flag order; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; argument 0 names 8 random bytes (null a third of the rows); argument 1 names 8 random bytes (null a third of the rows); argument 2 names 8 random bytes (null a third of the rows); argument 3 names 32 random bytes; argument 4 names 64 random bytes (null a third of the rows); argument 5 names 256 random bytes (null a third of the rows); vtable slot +0x60 (2 args) stubbed; vtable slot +0x124 (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; callee 0x0065bec0 (5 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x006e08d0 (3 args) stubbed; callee 0x0060a4b0 (0 args) stubbed; the word at `0xc0ab84+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p14+0` points at 64 random bytes less `0` (placed); the word at `p14+4` points at 64 random bytes less `0` (placed); the word at `p14+8` points at 64 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p18+0` points at 64 random bytes less `0` (placed); the word at `p18+4` points at 64 random bytes less `0` (placed); the word at `p18+8` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p22+0` points at 112 random bytes less `0` (placed); the word at `p22+4` points at 112 random bytes less `0` (placed); the word at `p22+8` points at 112 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p26+0` points at 112 random bytes less `0` (placed); the word at `p26+4` points at 112 random bytes less `0` (placed); the word at `p26+8` points at 112 random bytes less `0` (placed); the word at `p23+0x18` points at 64 random bytes less `0` (placed); the word at `p24+0x18` points at 64 random bytes less `0` (placed); the word at `p25+0x18` points at 64 random bytes less `0` (placed); the word at `p27+0x18` points at 64 random bytes less `0` (placed); the word at `p28+0x18` points at 64 random bytes less `0` (placed); the word at `p29+0x18` points at 64 random bytes less `0` (placed); the word at `m+0x18` points at 64 random bytes less `0` (placed); the word at `c0+0x134` points at 1024 random bytes less `0` (placed); the word at `c0+0x138` points at 1280 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1, 0] (90%); stub 2 answered from [1, 1, 1, 0] (90%); stub 3 answered from [1, 1, 1, 0] (90%); stub 4 answered from [1, 1, 0] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [0] (90%); stub 8 answered from [0, 1] (90%); stub 0 answers 40 values in call order, printed after the answers; stub 1 answers 40 values in call order, printed after the answers; stub 2 answers 40 values in call order, printed after the answers; stub 3 answers 40 values in call order, printed after the answers; stub 4 answers 40 values in call order, printed after the answers; stub 5 answers 80 values in call order, printed after the answers; stub 6 answers 10 values in call order, printed after the answers; stub 8 answers 10 values in call order, printed after the answers; stub 1 answers 0..4 every row; stub 5 answers -20..40 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `0x006817f0` run once before the rows (a table builder the startup would have run); stub 7 writes 1000 (4 bytes) at `p3+0x0` each call; stub 7 writes -77 (4 bytes) at `p3+0x4` each call; stub 7 writes 12345 (4 bytes) at `p3+0x8` each call; stub 7 writes 3 (4 bytes) at `p3+0xc` each call; stub 7 writes 999999 (4 bytes) at `p3+0x10` each call; stub 7 writes 0 (4 bytes) at `p3+0x14` each call; 4 rows 0x6eec apart at `0xc061e0`, slices +0x8:0x4, +0x74:0x10 (`p` windows after the pointed buffers, before the places); figures p15..p17 link at +0x2c (2 bytes) below their own index or to -1; figures p19..p21 link at +0x2c (2 bytes) below their own index or to -1; figures p15..p17 hold a short at +0x2e drawn from [0, 1], after the links; figures p19..p21 hold a short at +0x2e drawn from [0, 1], after the links; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`.
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

ENTRY = 0x00609180
WIN = 176
OWN = 10
BIAS = []
ARGSET = [(-7, [0, 1, 1]), (-8, [0, 1]), (-9, [-1, -1, -1, -1, -1, -1, 1500, 1536, 1560, 1600, 1700, 1800, 1850, 1900, 2000, 2100, 2200, 2300, 2304, 2400, 2500, 2600, 2700, 2800, 2900, 3000, 1500, 1536, 1560, 1600, 1700, 1800, 1850, 1900, 2000, 2100, 2200, 2300, 2304, 2400, 2500, 2600, 2700, 2800, 2900, 3000, 0, 40, 96, 150, 191, 192, 300, 383, 384, 500, 576, 700, 767, 768, 800, 1000, 1150, 1300, 1536, 1600, 1800, 2000, 2200, 2304, 2500, 2800, 3000, 3072, 3300, 3600, 3840, 4000, 4300, 4500, 4607]), (-10, [1500, 1536, 1560, 1600, 1700, 1800, 1850, 1900, 2000, 2100, 2200, 2300, 2304, 2400, 2500, 2600, 2700, 2800, 2900, 3000, 1500, 1536, 1560, 1600, 1700, 1800, 1850, 1900, 2000, 2100, 2200, 2300, 2304, 2400, 2500, 2600, 2700, 2800, 2900, 3000, 0, 40, 96, 150, 191, 192, 300, 383, 384, 500, 576, 700, 767, 768, 800, 1000, 1150, 1300, 1536, 1600, 1800, 2000, 2200, 2304, 2500, 2800, 3000, 3072, 3300, 3600, 3840, 4000, 4300, 4500, 4607])]
ARGFROM = []
CELLS = [(12607952, 320), (13297148, 4096)]
AT = []
ATRANGE = {}
PTRARGS = [(0, 8), (1, 8), (2, 8), (3, 32), (4, 64), (5, 256)]
PNULLS = [0, 1, 2, 4, 5]
STUBS = [('slot', 96, 2), ('slot', 292, 0), ('slot', 8, 0), ('slot', 188, 0), ('slot', 184, 2), ('call', 6667968, 5), ('call', 7265104, 1), ('call', 7211216, 3), ('call', 6333616, 0)]
PLACES = [(None, '0xc0ab84+0x1c*0', '0', 12), ('p14', '0', '0', 64), ('p14', '4', '0', 64), ('p14', '8', '0', 64), (None, '0xc0ab84+0x1c*1', '0', 12), ('p18', '0', '0', 64), ('p18', '4', '0', 64), ('p18', '8', '0', 64), (None, '0xc0aec0+0x1c*0', '0', 12), ('p22', '0', '0', 112), ('p22', '4', '0', 112), ('p22', '8', '0', 112), (None, '0xc0aec0+0x1c*1', '0', 12), ('p26', '0', '0', 112), ('p26', '4', '0', 112), ('p26', '8', '0', 112), ('p23', '0x18', '0', 64), ('p24', '0x18', '0', 64), ('p25', '0x18', '0', 64), ('p27', '0x18', '0', 64), ('p28', '0x18', '0', 64), ('p29', '0x18', '0', 64), ('m', '0x18', '0', 64), ('c0', '0x134', '0', 1024), ('c0', '0x138', '0', 1280)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 8, ('set', [4294901760, 0, 65536, 131072, 196608, 262144, 327680, 4294902016, 256, 65792, 131328, 196864, 262400, 327936, 4294902272, 512, 66048, 131584, 197120, 262656, 328192])), ('m', 16, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('m', 20, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('m', 104, ('set', [0, 0, 0, 524288, 1, 524289])), ('m', 148, ('set', [-1, -1, 0, 1, 2, 3, 5, 8, 12, 20, 40, 100, 1000])), ('p30', 4, ('set', [61, 62, 400, 5, 7, 317, 5, 9, 11, 60])), ('p31', 4, ('set', [61, 62, 400, 5, 7, 317, 5, 9, 11, 60])), ('p32', 4, ('set', [61, 62, 400, 5, 7, 317, 5, 9, 11, 60])), ('p33', 4, ('set', [61, 62, 400, 5, 7, 317, 5, 9, 11, 60])), ('p34', 4, ('set', [61, 62, 400, 5, 7, 317, 5, 9, 11, 60])), ('p35', 4, ('set', [61, 62, 400, 5, 7, 317, 5, 9, 11, 60])), ('p36', 4, ('set', [61, 62, 400, 5, 7, 317, 5, 9, 11, 60])), ('c0', 0, ('set', [5, 6, 6])), ('c0', 4, ('set', [5, 6, 6])), ('c0', 24, ('set', [16, 20, 24, 24])), ('c0', 28, ('set', [16, 20, 24, 24])), ('p23', 16, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p24', 16, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p25', 16, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p23', 20, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p24', 20, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p25', 20, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p23', 104, ('set', [0, 0, 0, 524288, 1, 524289])), ('p24', 104, ('set', [0, 0, 0, 524288, 1, 524289])), ('p25', 104, ('set', [0, 0, 0, 524288, 1, 524289])), ('p27', 16, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p28', 16, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p29', 16, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p27', 20, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p28', 20, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p29', 20, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p27', 104, ('set', [0, 0, 0, 524288, 1, 524289])), ('p28', 104, ('set', [0, 0, 0, 524288, 1, 524289])), ('p29', 104, ('set', [0, 0, 0, 524288, 1, 524289])), ('p15', 16, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p16', 16, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p17', 16, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p15', 20, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p16', 20, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p17', 20, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p19', 16, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p20', 16, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p21', 16, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p19', 20, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p20', 20, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p21', 20, ('set', [406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 406507, 405559, 405551, 405623, 405651, 405823, 405773, 405851, 405991, 409091, 409263, 409291, 409399, 409431, 409587, 408607, 408763, 408775, 408931, 408975, 407095, 407071, 407127, 407201, 407176, 407287, 407323, 407368, 407479, 407491, 406647, 406667, 406728, 406839, 406807, 407007, 406089, 406307, 405559, 405623, 405823, 405991, 409263, 409399, 409587, 408775, 408975, 408119, 408275, 407591, 407863, 407959, 403195, 403363, 403400])), ('p6', 0, ('set', [0, 1, 2, 3])), ('p7', None, ('set', [0, 2, 2, 1])), ('p8', 0, ('set', [0, 1, 2, 3])), ('p9', None, ('set', [0, 2, 2, 1])), ('p10', 0, ('set', [0, 1, 2, 3])), ('p11', None, ('set', [0, 2, 2, 1])), ('p12', 0, ('set', [0, 1, 2, 3])), ('p13', None, ('set', [0, 2, 2, 1])), ('p37', None, ('set', [0, 1, 2, 65536, 65537, 65538, 65535, -16711681, -16711681, 16842751, 33619967, 50397183, 16842751, 65535])), ('p38', None, ('set', [35652128, 35652096, 35652112, 35652144, 35651616, 35651584, 33554976, 33554944, 33554960, 33554992, 33554464, 33554432, 34603552, 34603520, 34603536, 34603568, 34603040, 34603008, 36700704, 36700672, 36700688, 36700720, 36700192, 36700160, 2097696, 2097664, 2097680, 2097712, 2097184, 2097152, 544, 512, 528, 560, 32, 0, 35652128, 35652128, 35652128, 35652128, 35652128, 35652128, 33554944, 33554944, 33554944, 33554944, 33554944, 33554944, 35652096, 35652096, 33554976, 33554976]))]
FILLS = [('c1', 'i//3')]
ARGRANGE = []
SLOTBASE = [(184, 6633360)]
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [0, 1, 1, 0]), (2, [1, 1, 1, 0]), (3, [1, 1, 1, 0]), (4, [1, 1, 0]), (6, [0, 1]), (7, [0]), (8, [0, 1])]
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = []
ANSWERBY = []
ANSWERWIN = []
CDECL = False
TIB = False
ARGEQ = []
SEQ = [(0, 40, None), (1, 40, None), (2, 40, None), (3, 40, None), (4, 40, None), (5, 80, None), (6, 10, None), (8, 10, None)]
ANSRANGE = [(1, 0, 4), (5, -20, 40)]
CELLTHIS = []
VTALSO = [('p30', 0), ('p31', 0), ('p32', 0), ('p33', 0), ('p34', 0), ('p35', 0), ('p36', 0), ('p15', 0), ('p16', 0), ('p17', 0), ('p19', 0), ('p20', 0), ('p21', 0), ('p23', 0), ('p24', 0), ('p25', 0), ('p27', 0), ('p28', 0), ('p29', 0)]
SPARSE = [(None, '0xc061e0', 28396, 4, [(8, 4), (116, 16)])]
GTAB = []
RECORD = [0, 1, 2, 3, 4, 5, 6, 7, 8]
RECORDORDER = True
RECORDEDX = False
ROWSPLIT = {6: 0, 7: 0}
LINKS = [(15, 17, 44, 2, '<', [-1]), (19, 21, 44, 2, '<', [-1])]
LINKHI = [(15, 17, 46, ('set', [0, 1])), (19, 21, 46, ('set', [0, 1]))]
OUTARG = []
CDECLSTUBS = []
STUBWRITE = [(7, 'p3', 0, 4, 1000), (7, 'p3', 4, 4, -77), (7, 'p3', 8, 4, 12345), (7, 'p3', 12, 4, 3), (7, 'p3', 16, 4, 999999), (7, 'p3', 20, 4, 0)]
STUBCOPY = []
STACKCLEAN = 0
INITCALLS = [6821872]
MAXINSNS = 3000000
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
    for i in range(2500):
        h.brk = base  # the bump heap is reused a row; 2500 rows of windows outgrow it
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

# sweep_gen: ["--item", "3330", "obj", "UnitData::calc_gather@00609180", "--name", "unit_data_calc_gather", "--win", "0xb0", "--own", "10", "--rows", "2500", "--ret", "eax", "--max-insns", "3000000", "--init-call", "0x6817f0", "--cell-win", "0xc061d0:0x140", "--cell-win", "0xcae5fc:0x1000", "--fill", "c1:i//3", "--ptrarg", "0:0x8", "--ptrarg", "1:0x8", "--ptrarg", "2:0x8", "--ptrarg", "3:0x20", "--ptrarg", "4:0x40", "--ptrarg", "5:0x100", "--ptrarg-null", "0", "--ptrarg-null", "1", "--ptrarg-null", "2", "--ptrarg-null", "4", "--ptrarg-null", "5", "--ptrtab", "0xc0ab84+0x1c*0:3:0x40:vt", "--ptrtab", "0xc0ab84+0x1c*1:3:0x40:vt", "--ptrtab", "0xc0aec0+0x1c*0:3:0x70:vt", "--ptrtab", "0xc0aec0+0x1c*1:3:0x70:vt", "--sparse", "0xc061e0:0x6eec:4:0x8,4/0x74,16", "--place", "p23+0x18:0:0x40", "--place", "p24+0x18:0:0x40", "--place", "p25+0x18:0:0x40", "--place", "p27+0x18:0:0x40", "--place", "p28+0x18:0:0x40", "--place", "p29+0x18:0:0x40", "--place", "m+0x18:0:0x40", "--place", "c0+0x134:0:0x400", "--place", "c0+0x138:0:0x500", "--vtat-also", "p30+0", "--vtat-also", "p31+0", "--vtat-also", "p32+0", "--vtat-also", "p33+0", "--vtat-also", "p34+0", "--vtat-also", "p35+0", "--vtat-also", "p36+0", "--stub", "slot:0x60:2", "--stub", "slot:0x124:0", "--stub", "slot:0x8:0", "--stub", "slot:0xbc:0", "--stub", "slot:0xb8:2", "--stub", "0x65bec0:5", "--stub", "0x6edb50:1", "--stub", "0x6e08d0:3", "--stub", "0x60a4b0:0", "--vtat-keep", "0xb8:0x653790", "--record", "0", "--record", "1", "--record", "2", "--record", "3", "--record", "4", "--record", "5", "--record", "6@0", "--record", "7@0", "--record", "8", "--record-order", "--seq", "0:40", "--answers", "0:0,1,1,0", "--seq", "1:40", "--answer-range", "1:0:4", "--seq", "2:40", "--answers", "2:1,1,1,0", "--seq", "3:40", "--answers", "3:1,1,1,0", "--seq", "4:40", "--answers", "4:1,1,0", "--seq", "5:80", "--answer-range", "5:-20:40", "--seq", "6:10", "--answers", "6:0,1", "--answers", "7:0", "--seq", "8:10", "--answers", "8:0,1", "--stub-write", "7:p3+0x0:4:1000", "--stub-write", "7:p3+0x4:4:-77", "--stub-write", "7:p3+0x8:4:12345", "--stub-write", "7:p3+0xc:4:3", "--stub-write", "7:p3+0x10:4:999999", "--stub-write", "7:p3+0x14:4:0", "--argset", "6!:0,1,1", "--argset", "7!:0,1", "--argset", "8!:-1,-1,-1,-1,-1,-1,1500,1536,1560,1600,1700,1800,1850,1900,2000,2100,2200,2300,2304,2400,2500,2600,2700,2800,2900,3000,1500,1536,1560,1600,1700,1800,1850,1900,2000,2100,2200,2300,2304,2400,2500,2600,2700,2800,2900,3000,0,40,96,150,191,192,300,383,384,500,576,700,767,768,800,1000,1150,1300,1536,1600,1800,2000,2200,2304,2500,2800,3000,3072,3300,3600,3840,4000,4300,4500,4607", "--argset", "9!:1500,1536,1560,1600,1700,1800,1850,1900,2000,2100,2200,2300,2304,2400,2500,2600,2700,2800,2900,3000,1500,1536,1560,1600,1700,1800,1850,1900,2000,2100,2200,2300,2304,2400,2500,2600,2700,2800,2900,3000,0,40,96,150,191,192,300,383,384,500,576,700,767,768,800,1000,1150,1300,1536,1600,1800,2000,2200,2304,2500,2800,3000,3072,3300,3600,3840,4000,4300,4500,4607", "--wbias", "m+0x8:4294901760,0,65536,131072,196608,262144,327680,4294902016,256,65792,131328,196864,262400,327936,4294902272,512,66048,131584,197120,262656,328192", "--wbias", "m+0x10:406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,407095,407071,407127,407201,407176,407287,407323,407368,407479,407491,406647,406667,406728,406839,406807,407007,406089,406307,405559,405623,405823,405991,409263,409399,409587,408775,408975,408119,408275,407591,407863,407959,403195,403363,403400", "--wbias", "m+0x14:406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,407095,407071,407127,407201,407176,407287,407323,407368,407479,407491,406647,406667,406728,406839,406807,407007,406089,406307,405559,405623,405823,405991,409263,409399,409587,408775,408975,408119,408275,407591,407863,407959,403195,403363,403400", "--wbias", "m+0x68:0,0,0,0x80000,1,0x80001", "--wbias", "m+0x94:-1,-1,0,1,2,3,5,8,12,20,40,100,1000", "--wbias", "p30+0x4:0x3d,0x3e,0x190,5,7,0x13d,5,9,11,0x3c", "--wbias", "p31+0x4:0x3d,0x3e,0x190,5,7,0x13d,5,9,11,0x3c", "--wbias", "p32+0x4:0x3d,0x3e,0x190,5,7,0x13d,5,9,11,0x3c", "--wbias", "p33+0x4:0x3d,0x3e,0x190,5,7,0x13d,5,9,11,0x3c", "--wbias", "p34+0x4:0x3d,0x3e,0x190,5,7,0x13d,5,9,11,0x3c", "--wbias", "p35+0x4:0x3d,0x3e,0x190,5,7,0x13d,5,9,11,0x3c", "--wbias", "p36+0x4:0x3d,0x3e,0x190,5,7,0x13d,5,9,11,0x3c", "--wbias", "c0+0x0:5,6,6", "--wbias", "c0+0x4:5,6,6", "--wbias", "c0+0x18:16,20,24,24", "--wbias", "c0+0x1c:16,20,24,24", "--wbias", "p23..25+0x10:406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,407095,407071,407127,407201,407176,407287,407323,407368,407479,407491,406647,406667,406728,406839,406807,407007,406089,406307,405559,405623,405823,405991,409263,409399,409587,408775,408975,408119,408275,407591,407863,407959,403195,403363,403400", "--wbias", "p23..25+0x14:406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,407095,407071,407127,407201,407176,407287,407323,407368,407479,407491,406647,406667,406728,406839,406807,407007,406089,406307,405559,405623,405823,405991,409263,409399,409587,408775,408975,408119,408275,407591,407863,407959,403195,403363,403400", "--wbias", "p23..25+0x68:0,0,0,0x80000,1,0x80001", "--wbias", "p27..29+0x10:406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,407095,407071,407127,407201,407176,407287,407323,407368,407479,407491,406647,406667,406728,406839,406807,407007,406089,406307,405559,405623,405823,405991,409263,409399,409587,408775,408975,408119,408275,407591,407863,407959,403195,403363,403400", "--wbias", "p27..29+0x14:406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,407095,407071,407127,407201,407176,407287,407323,407368,407479,407491,406647,406667,406728,406839,406807,407007,406089,406307,405559,405623,405823,405991,409263,409399,409587,408775,408975,408119,408275,407591,407863,407959,403195,403363,403400", "--wbias", "p27..29+0x68:0,0,0,0x80000,1,0x80001", "--wbias", "p15..17+0x10:406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,407095,407071,407127,407201,407176,407287,407323,407368,407479,407491,406647,406667,406728,406839,406807,407007,406089,406307,405559,405623,405823,405991,409263,409399,409587,408775,408975,408119,408275,407591,407863,407959,403195,403363,403400", "--wbias", "p15..17+0x14:406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,407095,407071,407127,407201,407176,407287,407323,407368,407479,407491,406647,406667,406728,406839,406807,407007,406089,406307,405559,405623,405823,405991,409263,409399,409587,408775,408975,408119,408275,407591,407863,407959,403195,403363,403400", "--link", "p15..17+0x2c:2:<:-1", "--link-hi", "p15..17+0x2e:0,1", "--wbias", "p19..21+0x10:406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,407095,407071,407127,407201,407176,407287,407323,407368,407479,407491,406647,406667,406728,406839,406807,407007,406089,406307,405559,405623,405823,405991,409263,409399,409587,408775,408975,408119,408275,407591,407863,407959,403195,403363,403400", "--wbias", "p19..21+0x14:406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,406507,405559,405551,405623,405651,405823,405773,405851,405991,409091,409263,409291,409399,409431,409587,408607,408763,408775,408931,408975,407095,407071,407127,407201,407176,407287,407323,407368,407479,407491,406647,406667,406728,406839,406807,407007,406089,406307,405559,405623,405823,405991,409263,409399,409587,408775,408975,408119,408275,407591,407863,407959,403195,403363,403400", "--link", "p19..21+0x2c:2:<:-1", "--link-hi", "p19..21+0x2e:0,1", "--wbias", "p6+0:0,1,2,3", "--wbias", "p7+*:0,2,2,1", "--wbias", "p8+0:0,1,2,3", "--wbias", "p9+*:0,2,2,1", "--wbias", "p10+0:0,1,2,3", "--wbias", "p11+*:0,2,2,1", "--wbias", "p12+0:0,1,2,3", "--wbias", "p13+*:0,2,2,1", "--wbias", "p37+*:0,1,2,65536,65537,65538,65535,-16711681,-16711681,16842751,33619967,50397183,16842751,65535", "--wbias", "p38+*:35652128,35652096,35652112,35652144,35651616,35651584,33554976,33554944,33554960,33554992,33554464,33554432,34603552,34603520,34603536,34603568,34603040,34603008,36700704,36700672,36700688,36700720,36700192,36700160,2097696,2097664,2097680,2097712,2097184,2097152,544,512,528,560,32,0,35652128,35652128,35652128,35652128,35652128,35652128,33554944,33554944,33554944,33554944,33554944,33554944,35652096,35652096,33554976,33554976", "--port", "{\n                // **The port is the crate's own steps**, `crate::calc_gather::unit_calc_gather_steps`, which\n                // `Sim::calc_gather_at` and `Sim::do_gather_rates` run over their `SimGather`; here the view is the\n                // emulated original's windows.\n                //\n                // `a`: the ten stack arguments (`param_1` `rare`, `param_2` the flag, `param_3` `good_obj`, `param_4`\n                // the six rates, `param_5` the mask, `param_6` the tally \u2014 a pointer argument is 0 when the row laid\n                // it null \u2014 then `param_7`, `param_8` and the position `param_9`, `param_10`), the nine stubs' single\n                // answers (`a[10 + k]`; only stub 7's is read, and it is ignored), then the call-order answers: stub 0\n                // (a type's `is`) from `a[19]`, 1 (its `get_upgrade_level`) `a[59]`, 2 (an object's `+0x8`) `a[99]`,\n                // 3 (`+0xbc`) `a[139]`, 4 (a unit's `+0xb8`) `a[179]`, 5 (`find_good_at`) `a[219]`, 6 (`is_ally`)\n                // `a[299]`, 8 (`is_unpacking`) `a[309]`, and the vtable flag `a[319]` (slot `+0xb8` laid as\n                // `ObjectData::is`: the unit's `is` is its type's slot `+0x60`, stub 0, not stub 4).\n                // Windows: `c[0]` the world, `c[1]` `div_3_table`; `p0..p5` the pointed arguments, `p6 + 2r` and\n                // `p7 + 2r` leader `r`'s `+0x8` and `+0x74..`, `p15 + 4L + i` the objects of list `L`, `p23 + 4L + i`\n                // the units, `p30 + 3L + i` their types, `p36` this's type, `p37` the cells, `p38` the tile masks.\n                use crate::calc_gather::{GatherArgs, GatherView, Who, unit_calc_gather_steps};\n                type Obj = (i32, i32);\n                struct V<'a> {\n                    a: &'a [i32],\n                    m: &'a mut [u8],\n                    c: &'a mut [Vec<u8>],\n                    p: &'a mut [Vec<u8>],\n                    log: &'a mut Vec<String>,\n                    turn: [usize; 9],\n                }\n                fn rd32(w: &[u8], o: usize) -> i32 {\n                    i32::from_le_bytes(w[o..o + 4].try_into().expect(\"four bytes\"))\n                }\n                fn wr32(w: &mut [u8], o: usize, v: i32) {\n                    w[o..o + 4].copy_from_slice(&v.to_le_bytes());\n                }\n                fn rd16(w: &[u8], o: usize) -> i32 {\n                    i32::from(i16::from_le_bytes(w[o..o + 2].try_into().expect(\"two bytes\")))\n                }\n                impl V<'_> {\n                    fn seq(&mut self, k: usize, base: usize, n: usize) -> i32 {\n                        let i = self.turn[k].min(n - 1);\n                        self.turn[k] += 1;\n                        self.a[base + i]\n                    }\n                    fn obj(&self, h: Obj) -> usize {\n                        15 + 4 * h.0 as usize + h.1 as usize\n                    }\n                    fn unit(&self, h: Obj) -> usize {\n                        23 + 4 * h.0 as usize + h.1 as usize\n                    }\n                    fn ty(&self, w: Who<Obj>) -> usize {\n                        match w {\n                            Who::Me => 36,\n                            Who::Other(h) => 30 + 3 * h.0 as usize + h.1 as usize,\n                        }\n                    }\n                    fn rec(&self, w: Who<Obj>) -> &[u8] {\n                        match w {\n                            Who::Me => &*self.m,\n                            Who::Other(h) => &self.p[self.unit(h)],\n                        }\n                    }\n                    fn cell_slot(&self, cx: i32, cy: i32) -> usize {\n                        (rd32(&self.c[0], 0) * cy + cx) as usize * 0x1c\n                    }\n                    fn link(&self, o: usize) -> Option<Obj> {\n                        let (idx, list) = (rd16(&self.p[o], 0x2c), rd16(&self.p[o], 0x2e));\n                        (idx >= 0).then_some((list, idx))\n                    }\n                }\n                impl GatherView for V<'_> {\n                    type H = Obj;\n                    fn type_index(&self, w: Who<Obj>) -> i32 {\n                        rd32(&self.p[self.ty(w)], 4)\n                    }\n                    fn type_is_fisher(&mut self, w: Who<Obj>) -> bool {\n                        self.log.push(format!(\"s0:p{}+0x0,317,0\", self.ty(w)));\n                        self.seq(0, 19, 40) != 0\n                    }\n                    fn type_radius_base(&mut self, w: Who<Obj>) -> i32 {\n                        self.log.push(format!(\"s1:p{}+0x0\", self.ty(w)));\n                        self.seq(1, 59, 40)\n                    }\n                    fn packed(&self, w: Who<Obj>) -> bool {\n                        rd32(self.rec(w), 0x68) & 0x80000 != 0\n                    }\n                    fn pos(&self, w: Who<Obj>) -> (i32, i32) {\n                        // The place is read off the **objects** list's element, the mask and the type off the units'.\n                        let r = match w {\n                            Who::Me => &*self.m,\n                            Who::Other(h) => &self.p[self.obj(h)],\n                        };\n                        (rd32(r, 0x10) ^ 0x63637, rd32(r, 0x14) ^ 0x63637)\n                    }\n                    fn me_owner(&self) -> i32 {\n                        i32::from(self.m[9])\n                    }\n                    fn good_obj(&self) -> i32 {\n                        rd16(self.m, 0x94)\n                    }\n                    fn div3(&self, i: i32) -> i32 {\n                        rd32(&self.c[1], 4 * i as usize)\n                    }\n                    fn tile_dims(&self) -> (i32, i32) {\n                        (rd32(&self.c[0], 0x18), rd32(&self.c[0], 0x1c))\n                    }\n                    fn cell_dims(&self) -> (i32, i32) {\n                        (rd32(&self.c[0], 0), rd32(&self.c[0], 4))\n                    }\n                    fn tile_mask(&self, x: i32, y: i32) -> u16 {\n                        let o = 2 * (rd32(&self.c[0], 0x18) * y + x) as usize;\n                        u16::from_le_bytes(self.p[38][o..o + 2].try_into().expect(\"two bytes\"))\n                    }\n                    fn cell_owner(&self, cx: i32, cy: i32) -> i32 {\n                        i32::from(self.p[37][self.cell_slot(cx, cy) + 0xf] as i8)\n                    }\n                    fn cell_head(&self, cx: i32, cy: i32) -> Option<Obj> {\n                        let o = self.cell_slot(cx, cy);\n                        let (idx, list) = (rd16(&self.p[37], o + 8), rd16(&self.p[37], o + 0xa));\n                        (idx >= 0).then_some((list, idx))\n                    }\n                    fn next(&self, h: Obj) -> Option<Obj> {\n                        self.link(self.obj(h))\n                    }\n                    fn list_of(&self, h: Obj) -> i32 {\n                        h.0\n                    }\n                    fn active(&mut self, h: Obj) -> bool {\n                        self.log.push(format!(\"s2:p{}+0x0\", self.obj(h)));\n                        self.seq(2, 99, 40) != 0\n                    }\n                    fn on_map(&mut self, h: Obj) -> bool {\n                        self.log.push(format!(\"s3:p{}+0x0\", self.obj(h)));\n                        self.seq(3, 139, 40) != 0\n                    }\n                    fn is_line(&mut self, h: Obj, ty: i32) -> bool {\n                        if self.a[319] != 0 {\n                            self.log\n                                .push(format!(\"s0:p{}+0x0,{ty},0\", self.ty(Who::Other(h))));\n                            self.seq(0, 19, 40) != 0\n                        } else {\n                            self.log.push(format!(\"s4:p{}+0x0,{ty},0\", self.unit(h)));\n                            self.seq(4, 179, 40) != 0\n                        }\n                    }\n                    fn unpacking(&mut self, h: Obj) -> bool {\n                        self.log.push(format!(\"s8:p{}+0x0\", self.unit(h)));\n                        self.seq(8, 309, 10) != 0\n                    }\n                    fn is_self(&self, h: Obj) -> bool {\n                        h.1 == rd16(self.m, 0xa)\n                    }\n                    fn vector_dist(&mut self, dx: i32, dy: i32) -> i32 {\n                        crate::world::vector_dist(dx, dy)\n                    }\n                    fn find_good(&mut self, cx: i32, cy: i32, who: i32, ecx: i32) -> i32 {\n                        self.log.push(format!(\"s5:{ecx},{cx},{cy},{who},0,0\"));\n                        self.seq(5, 219, 80)\n                    }\n                    fn is_ally(&mut self, who: i32, owner: i32) -> bool {\n                        self.log.push(format!(\"s6:r{who}+0x0,{owner}\"));\n                        self.seq(6, 299, 10) != 0\n                    }\n                    fn leader_id(&self, who: i32) -> i32 {\n                        rd32(&self.p[6 + 2 * who as usize], 0)\n                    }\n                    fn relation(&self, a: i32, b: i32) -> i32 {\n                        rd32(&self.p[7 + 2 * a as usize], 4 * b as usize)\n                    }\n                    fn calc_rare(&mut self, good: i32, ally: bool) {\n                        let who = self.me_owner();\n                        self.log\n                            .push(format!(\"s7:r{who}+0x0,{good},p3+0x0,{}\", i32::from(ally)));\n                        for (k, v) in [1000, -77, 12345, 3, 999999, 0].into_iter().enumerate() {\n                            wr32(&mut self.p[3], 4 * k, v);\n                        }\n                    }\n                    fn rates_zero(&mut self) {\n                        for k in 0..6 {\n                            wr32(&mut self.p[3], 4 * k, 0);\n                        }\n                    }\n                    fn rates_div(&mut self, n: i32) {\n                        for k in 0..6 {\n                            let v = rd32(&self.p[3], 4 * k);\n                            wr32(&mut self.p[3], 4 * k, v.wrapping_div(n));\n                        }\n                    }\n                    fn mask_set(&mut self, good: i32) {\n                        let i = good.wrapping_sub(6);\n                        let at = (0xc + (i >> 3)) as usize;\n                        self.p[4][at] |= 1 << (i & 7);\n                        wr32(&mut self.p[4], 8, 0);\n                    }\n                    fn tally_inc(&mut self, good: i32) {\n                        // `tally[good - 6]`: below the buffer for `good < 6`, which lands in the allocator's slack.\n                        let off = 4 * i64::from(good) - 0x18;\n                        if off >= 0 {\n                            let v = rd32(&self.p[5], off as usize);\n                            wr32(&mut self.p[5], off as usize, v.wrapping_add(1));\n                        }\n                    }\n                    fn out_rare(&mut self, v: i32) {\n                        wr32(&mut self.p[0], 0, v);\n                    }\n                    fn out_flag(&mut self, v: i32) {\n                        wr32(&mut self.p[1], 0, v);\n                    }\n                    fn out_good(&mut self, v: i32) {\n                        self.p[2][0..2].copy_from_slice(&(v as i16).to_le_bytes());\n                    }\n                }\n                let args = GatherArgs {\n                    rare: a[0] != 0,\n                    flag: a[1] != 0,\n                    good: a[2] != 0,\n                    mask: a[4] != 0,\n                    tally: a[5] != 0,\n                    p7: a[6],\n                    p8: a[7],\n                    x: a[8],\n                    y: a[9],\n                };\n                let mut v = V {\n                    a,\n                    m,\n                    c,\n                    p,\n                    log: &mut log,\n                    turn: [0; 9],\n                };\n                let r = unit_calc_gather_steps(&mut v, &args);\n                if v.log.is_empty() {\n                    v.log.push(\"-\".to_string());\n                }\n                i64::from(r)\n            }"]
