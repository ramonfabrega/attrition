#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_ammo_do_damage.py — `Ammo::do_damage@00678060` under unicorn (item 3356, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_ammo_do_damage.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 96-byte object of random bytes (no bias), 0 stack argument(s);
`p` windows in order: p0..p11 sparse table 0 (3 rows × 4 slice(s), row-major), p12..p14 sparse table 1 (3 rows × 1 slice(s), row-major), p15..p41 the place(s), in flag order, p42..p43 the stubs' answer window(s); the global cell 0x00c0618c points at 256 random bytes; the global cell 0x00cae5fc points at 256 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c06210 points at 688 random bytes; the global cell 0x00c061ec points at 2096 random bytes; the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00c06208 points at 16 random bytes; the global cell 0x00c06214 points at 16 random bytes; the global cell 0x00c06184 points at 16 random bytes; callee 0x00a3ab00 (10 args) stubbed; callee 0x009096e0 (1 args) stubbed; callee 0x00678f90 (0 args) stubbed; callee 0x00678d90 (1 args) stubbed; callee 0x0064a480 (8 args) stubbed; callee 0x006791a0 (0 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x006b53a0 (2 args) stubbed; callee 0x009072e0 (2 args) stubbed; callee 0x00679240 (0 args) stubbed; callee 0x0043f360 (2 args) stubbed; callee 0x006b4830 (2 args) stubbed; callee 0x006db810 (1 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x007af660 (3 args) stubbed; callee 0x006625c0 (2 args) stubbed; callee 0x0092ba30 (6 args) stubbed; callee 0x006b4bb0 (4 args) stubbed; callee 0x007e9b70 (9 args) stubbed; callee 0x0092bc80 (0 args) stubbed; callee 0x0097f340 (3 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0x34 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `c0+0x14+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p15+0` points at 64 random bytes less `0` (placed); the word at `p15+4` points at 64 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p18+0` points at 64 random bytes less `0` (placed); the word at `p18+4` points at 64 random bytes less `0` (placed); the word at `c0+0x14+0x1c*2` points at 8 random bytes less `0` (placed); the word at `p21+0` points at 64 random bytes less `0` (placed); the word at `p21+4` points at 64 random bytes less `0` (placed); the word at `c0+0x14+0x1c*8` points at 8 random bytes less `0` (placed); the word at `p24+0` points at 64 random bytes less `0` (placed); the word at `p24+4` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p27+0` points at 32 random bytes less `0` (placed); the word at `p27+4` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p30+0` points at 32 random bytes less `0` (placed); the word at `p30+4` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*2` points at 8 random bytes less `0` (placed); the word at `p33+0` points at 32 random bytes less `0` (placed); the word at `p33+4` points at 32 random bytes less `0` (placed); the word at `p16+0x18` points at 776 random bytes less `0` (placed); the word at `p17+0x18` points at 776 random bytes less `0` (placed); the word at `p22+0x18` points at 776 random bytes less `0` (placed); the word at `p23+0x18` points at 776 random bytes less `0` (placed); the word at `c2+0x134` points at 252 random bytes less `0` (placed); the word at `c2+0x138` points at 288 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 1 answered from [9, 0, 0, 1, 2, 3, 4, 5] (90%); stub 2 answered from [0, 0, 1] (90%); stub 8 answered from [0, 1] (90%); stub 10 answered from [0, 1] (90%); stub 11 answered from [0, 1, 0, 0] (90%); stub 12 answered from [0, 1] (90%); stub 23 answered from [0, 1, 1] (90%); stub 24 answered from [0, 1] (90%); stub 25 answered from [0, 1] (90%); stub 26 answered from [0, 1] (90%); stub 29 answered from [0, 1] (90%); stub 30 answered from [0, 1] (90%); stub 27 answers a window of its own to each call, in call order (32 bytes, ...); the word at `p42+0x18` laid as the address of `p36` every row; the word at `p43+0x18` laid as the address of `p37` every row; the word at `p19+0x18` laid as the address of `p36` every row; the word at `p20+0x18` laid as the address of `p37` every row; the word at `p25+0x18` laid as the address of `p39` every row; the word at `p26+0x18` laid as the address of `p38` every row; the word at `p28+0x18` laid as the address of `p36` every row; the word at `p29+0x18` laid as the address of `p37` every row; the word at `p31+0x18` laid as the address of `p36` every row; the word at `p32+0x18` laid as the address of `p37` every row; the word at `p34+0x18` laid as the address of `p38` every row; the word at `p35+0x18` laid as the address of `p39` every row; stub 6 answers 2 values in call order, printed after the answers; stub 11 answers 8 values in call order, printed after the answers; stub 24 answers 8 values in call order, printed after the answers; stub 25 answers 8 values in call order, printed after the answers; stub 26 answers 8 values in call order, printed after the answers; stub 6 answers 0..65535 every row; stub 28 answers 0..9 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 30 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 7 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 7 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 10 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 10 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 11 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 11 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 19 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 19 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 7 writes a value in 0..2047 through its argument 0, printed after the answers (`o` in a place); stub 7 writes a value in 0..2047 through its argument 1, printed after the answers (`o` in a place); the stack 1024 bytes under the call is zeroed before each row; `0x006817f0` run once before the rows (a table builder the startup would have run); stub 3 writes 1 (4 bytes) at `m+0x4c` each call; stub 3 writes 1 (4 bytes) at `m+0x48` each call; 3 rows 0x6eec apart at `@0xe3a390`, slices +0x8:0x4, +0x74:0x20, +0x7b4:0x10, +0x6928:0x4 (`p` windows after the pointed buffers, before the places); 3 rows 0x6eec apart at `0xc061e0`, slices +0x74:0x20 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`.
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

ENTRY = 0x00678060
WIN = 96
OWN = 0
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607884, 256), (13297148, 256), (12607880, 320), (12608016, 688), (12607980, 2096), (12608376, 32), (12608008, 16), (12608020, 16), (12607876, 16)]
AT = []
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('call', 10726144, 10), ('call', 9475808, 1), ('call', 6786960, 0), ('call', 6786448, 1), ('call', 6595712, 8), ('call', 6787488, 0), ('call', 10722672, 2), ('call', 7033760, 2), ('call', 9466592, 2), ('call', 6787648, 0), ('call', 4453216, 2), ('call', 7030832, 2), ('call', 7190544, 1), ('call', 9186496, 1), ('call', 8298416, 10), ('call', 9959280, 1), ('call', 8058464, 3), ('call', 6694336, 2), ('call', 9615920, 6), ('call', 7031728, 4), ('call', 8297328, 9), ('call', 9616512, 0), ('call', 9958208, 3), ('slot', 24, 0), ('slot', 12, 0), ('slot', 8, 0), ('slot', 188, 0), ('slot', 176, 0), ('slot', 52, 0), ('slot', 184, 2), ('slot', 96, 2)]
PLACES = [('c0', '0x14+0x1c*0', '0', 8), ('p15', '0', '0', 64), ('p15', '4', '0', 64), ('c0', '0x14+0x1c*1', '0', 8), ('p18', '0', '0', 64), ('p18', '4', '0', 64), ('c0', '0x14+0x1c*2', '0', 8), ('p21', '0', '0', 64), ('p21', '4', '0', 64), ('c0', '0x14+0x1c*8', '0', 8), ('p24', '0', '0', 64), ('p24', '4', '0', 64), (None, '0xc0aec0+0x1c*0', '0', 8), ('p27', '0', '0', 32), ('p27', '4', '0', 32), (None, '0xc0aec0+0x1c*1', '0', 8), ('p30', '0', '0', 32), ('p30', '4', '0', 32), (None, '0xc0aec0+0x1c*2', '0', 8), ('p33', '0', '0', 32), ('p33', '4', '0', 32), ('p16', '0x18', '0', 776), ('p17', '0x18', '0', 776), ('p22', '0x18', '0', 776), ('p23', '0x18', '0', 776), ('c2', '0x134', '0', 252), ('c2', '0x138', '0', 288)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 60, ('set', [-1, 0, 0, 1, 1, 2, 2])), ('m', 64, ('set', [-1, 0, 1, 1, 0])), ('m', 72, ('set', [-1, 0, 1, 2, 8])), ('m', 76, ('set', [-1, 0, 1])), ('m', 48, ('set', [0, 0, 0, 1, 2, 3, 4, 5, 7, 8, 12, 20, 39, 40, 41])), ('m', 8, ('set', [0, 1, 2, 3, 4, 5, 7, 9])), ('m', 12, ('range', 0, 2047)), ('m', 16, ('range', 0, 2047)), ('m', 24, ('range', 0, 2047)), ('m', 28, ('range', 0, 2047)), ('c2', 0, ('set', [2, 3, 3])), ('c2', 4, ('set', [2, 3, 3])), ('c2', 24, ('set', [4, 8, 12])), ('c2', 28, ('set', [4, 8, 12])), ('c3', 664, ('set', [-1, 0, 1, 2])), ('p0', 0, ('set', [0, 1, 2])), ('p1', None, ('set', [0, 0, 1, 2])), ('p12', None, ('set', [0, 0, 1, 5])), ('p3', 0, ('range', 0, 20)), ('p4', 0, ('set', [0, 1, 2])), ('p5', None, ('set', [0, 0, 1, 2])), ('p13', None, ('set', [0, 0, 1, 5])), ('p7', 0, ('range', 0, 20)), ('p8', 0, ('set', [0, 1, 2])), ('p9', None, ('set', [0, 0, 1, 2])), ('p14', None, ('set', [0, 0, 1, 5])), ('p11', 0, ('range', 0, 20)), ('p16', 8, ('set', [0, 1, 2, 3, 32, 33, 256, 257])), ('p16', 16, ('set', [407095, 407063, 407223, 407351, 407479, 406583, 406839, 406071, 406327, 405559, 405815, 405960, 409399])), ('p16', 20, ('set', [407095, 407063, 407223, 407351, 407479, 406583, 406839, 406071, 406327, 405559, 405815, 405960, 409399])), ('p17', 8, ('set', [0, 1, 2, 3, 32, 33, 256, 257])), ('p17', 16, ('set', [407095, 407063, 407223, 407351, 407479, 406583, 406839, 406071, 406327, 405559, 405815, 405960, 409399])), ('p17', 20, ('set', [407095, 407063, 407223, 407351, 407479, 406583, 406839, 406071, 406327, 405559, 405815, 405960, 409399])), ('p19', 8, ('set', [0, 1, 2, 3, 32, 33, 256, 257])), ('p19', 16, ('set', [407095, 407063, 407223, 407351, 407479, 406583, 406839, 406071, 406327, 405559, 405815, 405960, 409399])), ('p19', 20, ('set', [407095, 407063, 407223, 407351, 407479, 406583, 406839, 406071, 406327, 405559, 405815, 405960, 409399])), ('p20', 8, ('set', [0, 1, 2, 3, 32, 33, 256, 257])), ('p20', 16, ('set', [407095, 407063, 407223, 407351, 407479, 406583, 406839, 406071, 406327, 405559, 405815, 405960, 409399])), ('p20', 20, ('set', [407095, 407063, 407223, 407351, 407479, 406583, 406839, 406071, 406327, 405559, 405815, 405960, 409399])), ('p22', 8, ('set', [0, 1, 2, 3, 32, 33, 256, 257])), ('p22', 16, ('set', [407095, 407063, 407223, 407351, 407479, 406583, 406839, 406071, 406327, 405559, 405815, 405960, 409399])), ('p22', 20, ('set', [407095, 407063, 407223, 407351, 407479, 406583, 406839, 406071, 406327, 405559, 405815, 405960, 409399])), ('p23', 8, ('set', [0, 1, 2, 3, 32, 33, 256, 257])), ('p23', 16, ('set', [407095, 407063, 407223, 407351, 407479, 406583, 406839, 406071, 406327, 405559, 405815, 405960, 409399])), ('p23', 20, ('set', [407095, 407063, 407223, 407351, 407479, 406583, 406839, 406071, 406327, 405559, 405815, 405960, 409399])), ('p25', 8, ('set', [0, 1, 2, 3, 32, 33, 256, 257])), ('p25', 16, ('set', [407095, 407063, 407223, 407351, 407479, 406583, 406839, 406071, 406327, 405559, 405815, 405960, 409399])), ('p25', 20, ('set', [407095, 407063, 407223, 407351, 407479, 406583, 406839, 406071, 406327, 405559, 405815, 405960, 409399])), ('p26', 8, ('set', [0, 1, 2, 3, 32, 33, 256, 257])), ('p26', 16, ('set', [407095, 407063, 407223, 407351, 407479, 406583, 406839, 406071, 406327, 405559, 405815, 405960, 409399])), ('p26', 20, ('set', [407095, 407063, 407223, 407351, 407479, 406583, 406839, 406071, 406327, 405559, 405815, 405960, 409399])), ('p16', 44, ('set', [65535, 65535, 65535, 1, 65536, 65537, 131072, 131073, 524288, 524289])), ('p17', 44, ('set', [65535, 65535, 65535, 65536, 65537, 131072, 131073, 524288, 524289])), ('p19', 44, ('set', [65535, 65535, 65535, 65537, 131072, 131073, 524288, 524289])), ('p20', 44, ('set', [65535, 65535, 65535, 131072, 131073, 524288, 524289])), ('p22', 44, ('set', [65535, 65535, 65535, 131073, 524288, 524289])), ('p23', 44, ('set', [65535, 65535, 65535, 524288, 524289])), ('p25', 44, ('set', [65535, 65535, 65535, 524289])), ('p26', 44, ('set', [65535, 65535, 65535])), ('p36', 484, ('set', [0, 134217728, 134217728, 1, 134217729])), ('p36', 512, ('set', [-3, -1, 0, 0, 1, 1, 2, 3, 4, 5, 6, 8, 64])), ('p36', 536, ('set', [0, 1, 2, 2, 3])), ('p36', 692, ('set', [0, 8192, 8192, 1, 8193])), ('p36', 564, ('set', [0, 1, 2, 3])), ('p36', 568, ('set', [0, 1, 2, 3])), ('p36', 572, ('set', [0, 5, 50, 100, 192])), ('p37', 484, ('set', [0, 134217728, 134217728, 1, 134217729])), ('p37', 512, ('set', [-3, -1, 0, 0, 1, 1, 2, 3, 4, 5, 6, 8, 64])), ('p37', 536, ('set', [0, 1, 2, 2, 3])), ('p37', 692, ('set', [0, 8192, 8192, 1, 8193])), ('p37', 564, ('set', [0, 1, 2, 3])), ('p37', 568, ('set', [0, 1, 2, 3])), ('p37', 572, ('set', [0, 5, 50, 100, 192])), ('p38', 484, ('set', [0, 134217728, 134217728, 1, 134217729])), ('p38', 512, ('set', [-3, -1, 0, 0, 1, 1, 2, 3, 4, 5, 6, 8, 64])), ('p38', 536, ('set', [0, 1, 2, 2, 3])), ('p38', 692, ('set', [0, 8192, 8192, 1, 8193])), ('p38', 564, ('set', [0, 1, 2, 3])), ('p38', 568, ('set', [0, 1, 2, 3])), ('p38', 572, ('set', [0, 5, 50, 100, 192])), ('p39', 484, ('set', [0, 134217728, 134217728, 1, 134217729])), ('p39', 512, ('set', [-3, -1, 0, 0, 1, 1, 2, 3, 4, 5, 6, 8, 64])), ('p39', 536, ('set', [0, 1, 2, 2, 3])), ('p39', 692, ('set', [0, 8192, 8192, 1, 8193])), ('p39', 564, ('set', [0, 1, 2, 3])), ('p39', 568, ('set', [0, 1, 2, 3])), ('p39', 572, ('set', [0, 5, 50, 100, 192])), ('p40', 8, ('set', [0, 1, 65536, 65537, 131072, 131073, 524288, 524289, 65535, 131071, 196607, 65535])), ('p40', 12, ('set', [4278190080, 0, 16777216, 33554432, 16777216, 50331648])), ('p40', 36, ('set', [0, 1, 65536, 65537, 131072, 131073, 524288, 524289, 65535, 131071, 196607, 65535])), ('p40', 40, ('set', [4278190080, 0, 16777216, 33554432, 16777216, 50331648])), ('p40', 64, ('set', [0, 1, 65536, 65537, 131072, 131073, 524288, 524289, 65535, 131071, 196607, 65535])), ('p40', 68, ('set', [4278190080, 0, 16777216, 33554432, 16777216, 50331648])), ('p40', 92, ('set', [0, 1, 65536, 65537, 131072, 131073, 524288, 524289, 65535, 131071, 196607, 65535])), ('p40', 96, ('set', [4278190080, 0, 16777216, 33554432, 16777216, 50331648])), ('p40', 120, ('set', [0, 1, 65536, 65537, 131072, 131073, 524288, 524289, 65535, 131071, 196607, 65535])), ('p40', 124, ('set', [4278190080, 0, 16777216, 33554432, 16777216, 50331648])), ('p40', 148, ('set', [0, 1, 65536, 65537, 131072, 131073, 524288, 524289, 65535, 131071, 196607, 65535])), ('p40', 152, ('set', [4278190080, 0, 16777216, 33554432, 16777216, 50331648])), ('p40', 176, ('set', [0, 1, 65536, 65537, 131072, 131073, 524288, 524289, 65535, 131071, 196607, 65535])), ('p40', 180, ('set', [4278190080, 0, 16777216, 33554432, 16777216, 50331648])), ('p40', 204, ('set', [0, 1, 65536, 65537, 131072, 131073, 524288, 524289, 65535, 131071, 196607, 65535])), ('p40', 208, ('set', [4278190080, 0, 16777216, 33554432, 16777216, 50331648])), ('p40', 232, ('set', [0, 1, 65536, 65537, 131072, 131073, 524288, 524289, 65535, 131071, 196607, 65535])), ('p40', 236, ('set', [4278190080, 0, 16777216, 33554432, 16777216, 50331648]))]
FILLS = [('c1', 'i//3')]
ARGRANGE = []
SLOTBASE = [(184, 6633360)]
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(1, [9, 0, 0, 1, 2, 3, 4, 5]), (2, [0, 0, 1]), (8, [0, 1]), (10, [0, 1]), (11, [0, 1, 0, 0]), (12, [0, 1]), (23, [0, 1, 1]), (24, [0, 1]), (25, [0, 1]), (26, [0, 1]), (29, [0, 1]), (30, [0, 1])]
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = [('p42', 24, 'p36', 0, True), ('p43', 24, 'p37', 0, True), ('p19', 24, 'p36', 0, True), ('p20', 24, 'p37', 0, True), ('p25', 24, 'p39', 0, True), ('p26', 24, 'p38', 0, True), ('p28', 24, 'p36', 0, True), ('p29', 24, 'p37', 0, True), ('p31', 24, 'p36', 0, True), ('p32', 24, 'p37', 0, True), ('p34', 24, 'p38', 0, True), ('p35', 24, 'p39', 0, True)]
ANSWERBY = []
ANSWERWIN = [(27, 32), (27, 32)]
CDECL = False
TIB = False
ARGEQ = []
SEQ = [(6, 2, None), (11, 8, None), (24, 8, None), (25, 8, None), (26, 8, None)]
ANSRANGE = [(6, 0, 65535), (28, 0, 9)]
CELLTHIS = []
VTALSO = [('p36', 0), ('p37', 0), ('p38', 0), ('p39', 0), ('p16', 0), ('p17', 0), ('p19', 0), ('p20', 0), ('p22', 0), ('p23', 0), ('p25', 0), ('p26', 0)]
SPARSE = [('@', '0xe3a390', 28396, 3, [(8, 4), (116, 32), (1972, 16), (26920, 4)]), (None, '0xc061e0', 28396, 3, [(116, 32)])]
GTAB = []
RECORD = [0, 1, 2, 3, 4, 5, 6, 8, 9, 12, 13, 14, 15, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30]
RECORDORDER = True
RECORDEDX = False
ROWSPLIT = {}
LINKS = []
LINKHI = []
OUTARG = [(7, 0, ('range', 0, 2047)), (7, 1, ('range', 0, 2047))]
CDECLSTUBS = []
STUBWRITE = [(3, 'm', 76, 4, 1), (3, 'm', 72, 4, 1)]
STUBCOPY = []
STACKCLEAN = 1024
INITCALLS = [6821872]
MAXINSNS = 700000
ECXANSWER = []
ECXOUT = []
RING = []
DEREF = [(7, 0, 4, None), (7, 1, 4, None), (10, 0, 4, None), (10, 1, 4, None), (11, 0, 4, None), (11, 1, 4, None), (19, 0, 4, None), (19, 1, 4, None)]
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
    for i in range(2400):
        h.brk = base  # the bump heap is reused a row; 2400 rows of windows outgrow it
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

# sweep_gen: ["--item", "3356", "obj", "Ammo::do_damage@00678060", "--name=ammo_do_damage", "--win=0x60", "--own=0", "--rows=2400", "--clean-stack=0x400", "--max-insns=700000", "--init-call=0x6817f0", "--cell-win=0xc0618c:0x100", "--cell-win=0xcae5fc:0x100", "--cell-win=0xc06188:0x140", "--cell-win=0xc06210:0x2b0", "--cell-win=0xc061ec:0x830", "--cell-win=0xc06378:0x20", "--cell-win=0xc06208:0x10", "--cell-win=0xc06214:0x10", "--cell-win=0xc06184:0x10", "--fill=c1:i//3", "--sparse=@0xe3a390:0x6eec:3:0x8,4/0x74,0x20/0x6928,4/0x7b4,0x10", "--sparse=0xc061e0:0x6eec:3:0x74,0x20", "--ptrtab=c0+0x14+0x1c*0:2:0x40:vt", "--ptrtab=c0+0x14+0x1c*1:2:0x40:vt", "--ptrtab=c0+0x14+0x1c*2:2:0x40:vt", "--ptrtab=c0+0x14+0x1c*8:2:0x40:vt", "--ptrtab=0xc0aec0+0x1c*0:2:0x20", "--ptrtab=0xc0aec0+0x1c*1:2:0x20", "--ptrtab=0xc0aec0+0x1c*2:2:0x20", "--place=p16+0x18:0:0x308", "--place=p17+0x18:0:0x308", "--place=p22+0x18:0:0x308", "--place=p23+0x18:0:0x308", "--place=c2+0x134:0:0xfc", "--place=c2+0x138:0:0x120", "--alias=p42+0x18:p36!", "--alias=p43+0x18:p37!", "--vtat-also=p36+0", "--vtat-also=p37+0", "--vtat-also=p38+0", "--vtat-also=p39+0", "--alias=p19+0x18:p36!", "--alias=p20+0x18:p37!", "--alias=p25+0x18:p39!", "--alias=p26+0x18:p38!", "--alias=p28+0x18:p36!", "--alias=p29+0x18:p37!", "--alias=p31+0x18:p36!", "--alias=p32+0x18:p37!", "--alias=p34+0x18:p38!", "--alias=p35+0x18:p39!", "--stub=0xa3ab00:10", "--stub=0x9096e0:1", "--stub=0x678f90:0", "--stub=0x678d90:1", "--stub=0x64a480:8", "--stub=0x6791a0:0", "--stub=0xa39d70:2", "--stub=0x6b53a0:2", "--stub=0x9072e0:2", "--stub=0x679240:0", "--stub=0x43f360:2", "--stub=0x6b4830:2", "--stub=0x6db810:1", "--stub=0x8c2cc0:1", "--stub=0x7e9fb0:10", "--stub=0x97f770:1", "--stub=0x7af660:3", "--stub=0x6625c0:2", "--stub=0x92ba30:6", "--stub=0x6b4bb0:4", "--stub=0x7e9b70:9", "--stub=0x92bc80:0", "--stub=0x97f340:3", "--stub=slot:0x18:0", "--stub=slot:0xc:0", "--stub=slot:0x8:0", "--stub=slot:0xbc:0", "--stub=slot:0xb0:0", "--stub=slot:0x34:0", "--stub=slot:0xb8:2", "--stub=slot:0x60:2", "--stub-write=3:m+0x4c:4:1", "--stub-write=3:m+0x48:4:1", "--answer-win=27:0x20", "--answer-win=27:0x20", "--vtat-keep=0xb8:0x653790", "--answers=1:9,0,0,1,2,3,4,5", "--answers=2:0,0,1", "--answers=8:0,1", "--answers=10:0,1", "--answers=11:0,1,0,0", "--answers=12:0,1", "--answers=23:0,1,1", "--answers=24:0,1", "--answers=25:0,1", "--answers=26:0,1", "--answers=29:0,1", "--answers=30:0,1", "--seq=6:2", "--answer-range=6:0:0xffff", "--seq=11:8", "--seq=24:8", "--seq=25:8", "--seq=26:8", "--answer-range=28:0:9", "--outarg=7:0:0..0x7ff", "--outarg=7:1:0..0x7ff", "--record=0", "--record=1", "--record=2", "--record=3", "--record=4", "--record=5", "--record=6", "--record=8", "--record=9", "--record=12", "--record=13", "--record=14", "--record=15", "--record=18", "--record=19", "--record=20", "--record=21", "--record=22", "--record=23", "--record=24", "--record=25", "--record=26", "--record=27", "--record=28", "--record=29", "--record=30", "--record-order", "--deref=7:0:4", "--deref=7:1:4", "--deref=10:0:4", "--deref=10:1:4", "--deref=11:0:4", "--deref=11:1:4", "--deref=19:0:4", "--deref=19:1:4", "--wbias=m+0x3c:-1,0,0,1,1,2,2", "--wbias=m+0x40:-1,0,1,1,0", "--wbias=m+0x48:-1,0,1,2,8", "--wbias=m+0x4c:-1,0,1", "--wbias=m+0x30:0,0,0,1,2,3,4,5,7,8,12,20,39,40,41", "--wbias=m+0x8:0,1,2,3,4,5,7,9", "--wbias=m+0xc:0..0x7ff", "--wbias=m+0x10:0..0x7ff", "--wbias=m+0x18:0..0x7ff", "--wbias=m+0x1c:0..0x7ff", "--wbias=c2+0:2,3,3", "--wbias=c2+4:2,3,3", "--wbias=c2+0x18:4,8,12", "--wbias=c2+0x1c:4,8,12", "--wbias=c3+0x298:-1,0,1,2", "--wbias=p0+0:0,1,2", "--wbias=p1+*:0,0,1,2", "--wbias=p12+*:0,0,1,5", "--wbias=p3+0:0..20", "--wbias=p4+0:0,1,2", "--wbias=p5+*:0,0,1,2", "--wbias=p13+*:0,0,1,5", "--wbias=p7+0:0..20", "--wbias=p8+0:0,1,2", "--wbias=p9+*:0,0,1,2", "--wbias=p14+*:0,0,1,5", "--wbias=p11+0:0..20", "--wbias=p16+8:0,1,2,3,0x20,0x21,0x100,0x101", "--wbias=p16+0x10:0x63637,0x63617,0x636b7,0x63737,0x637b7,0x63437,0x63537,0x63237,0x63337,0x63037,0x63137,0x631c8,0x63f37", "--wbias=p16+0x14:0x63637,0x63617,0x636b7,0x63737,0x637b7,0x63437,0x63537,0x63237,0x63337,0x63037,0x63137,0x631c8,0x63f37", "--wbias=p17+8:0,1,2,3,0x20,0x21,0x100,0x101", "--wbias=p17+0x10:0x63637,0x63617,0x636b7,0x63737,0x637b7,0x63437,0x63537,0x63237,0x63337,0x63037,0x63137,0x631c8,0x63f37", "--wbias=p17+0x14:0x63637,0x63617,0x636b7,0x63737,0x637b7,0x63437,0x63537,0x63237,0x63337,0x63037,0x63137,0x631c8,0x63f37", "--wbias=p19+8:0,1,2,3,0x20,0x21,0x100,0x101", "--wbias=p19+0x10:0x63637,0x63617,0x636b7,0x63737,0x637b7,0x63437,0x63537,0x63237,0x63337,0x63037,0x63137,0x631c8,0x63f37", "--wbias=p19+0x14:0x63637,0x63617,0x636b7,0x63737,0x637b7,0x63437,0x63537,0x63237,0x63337,0x63037,0x63137,0x631c8,0x63f37", "--wbias=p20+8:0,1,2,3,0x20,0x21,0x100,0x101", "--wbias=p20+0x10:0x63637,0x63617,0x636b7,0x63737,0x637b7,0x63437,0x63537,0x63237,0x63337,0x63037,0x63137,0x631c8,0x63f37", "--wbias=p20+0x14:0x63637,0x63617,0x636b7,0x63737,0x637b7,0x63437,0x63537,0x63237,0x63337,0x63037,0x63137,0x631c8,0x63f37", "--wbias=p22+8:0,1,2,3,0x20,0x21,0x100,0x101", "--wbias=p22+0x10:0x63637,0x63617,0x636b7,0x63737,0x637b7,0x63437,0x63537,0x63237,0x63337,0x63037,0x63137,0x631c8,0x63f37", "--wbias=p22+0x14:0x63637,0x63617,0x636b7,0x63737,0x637b7,0x63437,0x63537,0x63237,0x63337,0x63037,0x63137,0x631c8,0x63f37", "--wbias=p23+8:0,1,2,3,0x20,0x21,0x100,0x101", "--wbias=p23+0x10:0x63637,0x63617,0x636b7,0x63737,0x637b7,0x63437,0x63537,0x63237,0x63337,0x63037,0x63137,0x631c8,0x63f37", "--wbias=p23+0x14:0x63637,0x63617,0x636b7,0x63737,0x637b7,0x63437,0x63537,0x63237,0x63337,0x63037,0x63137,0x631c8,0x63f37", "--wbias=p25+8:0,1,2,3,0x20,0x21,0x100,0x101", "--wbias=p25+0x10:0x63637,0x63617,0x636b7,0x63737,0x637b7,0x63437,0x63537,0x63237,0x63337,0x63037,0x63137,0x631c8,0x63f37", "--wbias=p25+0x14:0x63637,0x63617,0x636b7,0x63737,0x637b7,0x63437,0x63537,0x63237,0x63337,0x63037,0x63137,0x631c8,0x63f37", "--wbias=p26+8:0,1,2,3,0x20,0x21,0x100,0x101", "--wbias=p26+0x10:0x63637,0x63617,0x636b7,0x63737,0x637b7,0x63437,0x63537,0x63237,0x63337,0x63037,0x63137,0x631c8,0x63f37", "--wbias=p26+0x14:0x63637,0x63617,0x636b7,0x63737,0x637b7,0x63437,0x63537,0x63237,0x63337,0x63037,0x63137,0x631c8,0x63f37", "--wbias=p16+0x2c:0xffff,0xffff,0xffff,0x1,0x10000,0x10001,0x20000,0x20001,0x80000,0x80001", "--wbias=p17+0x2c:0xffff,0xffff,0xffff,0x10000,0x10001,0x20000,0x20001,0x80000,0x80001", "--wbias=p19+0x2c:0xffff,0xffff,0xffff,0x10001,0x20000,0x20001,0x80000,0x80001", "--wbias=p20+0x2c:0xffff,0xffff,0xffff,0x20000,0x20001,0x80000,0x80001", "--wbias=p22+0x2c:0xffff,0xffff,0xffff,0x20001,0x80000,0x80001", "--wbias=p23+0x2c:0xffff,0xffff,0xffff,0x80000,0x80001", "--wbias=p25+0x2c:0xffff,0xffff,0xffff,0x80001", "--wbias=p26+0x2c:0xffff,0xffff,0xffff", "--wbias=p36+0x1e4:0,0x8000000,0x8000000,1,0x8000001", "--wbias=p36+0x200:-3,-1,0,0,1,1,2,3,4,5,6,8,0x40", "--wbias=p36+0x218:0,1,2,2,3", "--wbias=p36+0x2b4:0,0x2000,0x2000,1,0x2001", "--wbias=p36+0x234:0,1,2,3", "--wbias=p36+0x238:0,1,2,3", "--wbias=p36+0x23c:0,5,50,100,192", "--wbias=p37+0x1e4:0,0x8000000,0x8000000,1,0x8000001", "--wbias=p37+0x200:-3,-1,0,0,1,1,2,3,4,5,6,8,0x40", "--wbias=p37+0x218:0,1,2,2,3", "--wbias=p37+0x2b4:0,0x2000,0x2000,1,0x2001", "--wbias=p37+0x234:0,1,2,3", "--wbias=p37+0x238:0,1,2,3", "--wbias=p37+0x23c:0,5,50,100,192", "--wbias=p38+0x1e4:0,0x8000000,0x8000000,1,0x8000001", "--wbias=p38+0x200:-3,-1,0,0,1,1,2,3,4,5,6,8,0x40", "--wbias=p38+0x218:0,1,2,2,3", "--wbias=p38+0x2b4:0,0x2000,0x2000,1,0x2001", "--wbias=p38+0x234:0,1,2,3", "--wbias=p38+0x238:0,1,2,3", "--wbias=p38+0x23c:0,5,50,100,192", "--wbias=p39+0x1e4:0,0x8000000,0x8000000,1,0x8000001", "--wbias=p39+0x200:-3,-1,0,0,1,1,2,3,4,5,6,8,0x40", "--wbias=p39+0x218:0,1,2,2,3", "--wbias=p39+0x2b4:0,0x2000,0x2000,1,0x2001", "--wbias=p39+0x234:0,1,2,3", "--wbias=p39+0x238:0,1,2,3", "--wbias=p39+0x23c:0,5,50,100,192", "--wbias=p40+0x8:0x0,0x1,0x10000,0x10001,0x20000,0x20001,0x80000,0x80001,0xffff,0x1ffff,0x2ffff,0xffff", "--wbias=p40+0xc:0xff000000,0,0x01000000,0x02000000,0x01000000,0x03000000", "--wbias=p40+0x24:0x0,0x1,0x10000,0x10001,0x20000,0x20001,0x80000,0x80001,0xffff,0x1ffff,0x2ffff,0xffff", "--wbias=p40+0x28:0xff000000,0,0x01000000,0x02000000,0x01000000,0x03000000", "--wbias=p40+0x40:0x0,0x1,0x10000,0x10001,0x20000,0x20001,0x80000,0x80001,0xffff,0x1ffff,0x2ffff,0xffff", "--wbias=p40+0x44:0xff000000,0,0x01000000,0x02000000,0x01000000,0x03000000", "--wbias=p40+0x5c:0x0,0x1,0x10000,0x10001,0x20000,0x20001,0x80000,0x80001,0xffff,0x1ffff,0x2ffff,0xffff", "--wbias=p40+0x60:0xff000000,0,0x01000000,0x02000000,0x01000000,0x03000000", "--wbias=p40+0x78:0x0,0x1,0x10000,0x10001,0x20000,0x20001,0x80000,0x80001,0xffff,0x1ffff,0x2ffff,0xffff", "--wbias=p40+0x7c:0xff000000,0,0x01000000,0x02000000,0x01000000,0x03000000", "--wbias=p40+0x94:0x0,0x1,0x10000,0x10001,0x20000,0x20001,0x80000,0x80001,0xffff,0x1ffff,0x2ffff,0xffff", "--wbias=p40+0x98:0xff000000,0,0x01000000,0x02000000,0x01000000,0x03000000", "--wbias=p40+0xb0:0x0,0x1,0x10000,0x10001,0x20000,0x20001,0x80000,0x80001,0xffff,0x1ffff,0x2ffff,0xffff", "--wbias=p40+0xb4:0xff000000,0,0x01000000,0x02000000,0x01000000,0x03000000", "--wbias=p40+0xcc:0x0,0x1,0x10000,0x10001,0x20000,0x20001,0x80000,0x80001,0xffff,0x1ffff,0x2ffff,0xffff", "--wbias=p40+0xd0:0xff000000,0,0x01000000,0x02000000,0x01000000,0x03000000", "--wbias=p40+0xe8:0x0,0x1,0x10000,0x10001,0x20000,0x20001,0x80000,0x80001,0xffff,0x1ffff,0x2ffff,0xffff", "--wbias=p40+0xec:0xff000000,0,0x01000000,0x02000000,0x01000000,0x03000000", "--port", "// Row: `a[i]` stub `i`'s answer (stubs 0 to 30, in the line's order: 0 `Log::say`, 1\n// `get_gpiece_type`, 2 `hit_target`, 3 `check_hit`, 4 `Object::do_damage`, 5 `close`, 6\n// `Random::get`, 7 `WorldData::restrict`, 8 `verify_ammo_flags`, 9 `puncture_ground`, 10\n// `is_valid`, 11 `is_ocean`, 12 `has_preq`, 13 `get_neon`, 14 `add_message`, 15 the shield\n// sound, 16 `Achieve::add_event`, 17 `play_sound`, 18 `Nuke::add_nuke`, 19 `set_seen2`, 20\n// `MessageWin::add_event`, 21 `Nuke::do_damage`, 22 the nuke-hit sound, then the vtable slots\n// `+0x18`, `+0xc`, `+0x8`, `+0xbc`, `+0xb0`, `+0x34`, `+0xb8`, `+0x60` (23 to 30)); `a[31]`,\n// `a[32]` the two values `restrict` writes through its arguments; then the `--seq` runs:\n// `Random::get` (2, `a[33..35]`), `is_ocean` (8, `a[35..43]`), slot `+0xc` (`a[43..51]`),\n// slot `+0x8` (`a[51..59]`), slot `+0xbc` (`a[59..67]`); `a[67]` whether slot `+0xb8` held\n// `ObjectData::is` (1) or its stub (0). Windows: `m` the round (`Ammo`), `c0` the objects, `c1`\n// `div_3_table`, `c2` the world, `c3` the console, `c4` the game, `c5` `int_str_array`, `c6`\n// the message window, `c7` the graphic pieces, `c8` `game_random`; `p4r..p4r+3` leader row `r`'s\n// `+0x8`, `+0x74`, `+0x7b4` and `+0x6928` slices, `p12 + r` the constants leaders' `+0x74`;\n// `p16 p17 p19 p20 p22 p23 p25 p26` the objects of (0,0) (0,1) (1,0) (1,1) (2,0) (2,1) (8,0)\n// (8,1), the units are unread (their type is the object's own, aliased), `p36..p39` the four\n// type records they share, `p40` the world's `+0x134` grid, `p41` its `+0x138` flags,\n// `p42`/`p43` the two records slot `+0xb0` answers.\n//\n// **The port is the sweep's own**: the crate has `Sim::land` (fight.rs) for this function, but\n// over the whole world and not a view, and it walks a different splash (a Chebyshev disc,\n// `combat::splash_cells`, where the original walks the spiral to `radius[k]` *inclusive*, one\n// cell past the disc). What it calls of the crate: `movement::find_angle`,\n// `world::vector_dist`, `collide::spiral` (`move_x`/`move_y`), `ai_place::circle`\n// (`circle_x`/`circle_y`).\nstruct Env<'a> {\n    a: &'a [i32],\n    m: &'a mut [u8],\n    c: &'a mut [Vec<u8>],\n    p: &'a mut [Vec<u8>],\n    toks: Vec<String>,\n    derefs: [Vec<String>; 8],\n    n_rand: usize,\n    n_ocean: usize,\n    n_c: usize,\n    n_8: usize,\n    n_bc: usize,\n    n_b0: usize,\n}\n\nfn n(v: i32) -> String {\n    v.to_string()\n}\n\nfn hex4(v: i32) -> String {\n    v.to_le_bytes().iter().map(|b| format!(\"{b:02x}\")).collect()\n}\n\nfn i16at(w: &[u8], off: usize) -> i32 {\n    i32::from(i16::from_le_bytes([w[off], w[off + 1]]))\n}\n\n/// `(x ^ s) - s`, the original's absolute value (`i32::MIN` stays negative).\nfn abs_sign(v: i32) -> i32 {\n    let s = v >> 31;\n    (v ^ s).wrapping_sub(s)\n}\n\n/// The eight objects the draw lays: (player, index, window).\nconst OBJS: [(i32, i32, usize); 8] = [\n    (0, 0, 16),\n    (0, 1, 17),\n    (1, 0, 19),\n    (1, 1, 20),\n    (2, 0, 22),\n    (2, 1, 23),\n    (8, 0, 25),\n    (8, 1, 26),\n];\n/// Their type records (a unit's is the same record).\nconst TYPE_OF: [usize; 8] = [36, 37, 36, 37, 38, 39, 39, 38];\n\nimpl Env<'_> {\n    fn f(&self, off: usize) -> i32 {\n        rd(self.m, off)\n    }\n    fn tok(&mut self, stub: usize, ecx: &str, args: &[String]) {\n        let mut s = format!(\"s{stub}:{ecx}\");\n        for v in args {\n            s.push(',');\n            s.push_str(v);\n        }\n        self.toks.push(s);\n    }\n    fn obj(&self, pl: i32, ix: i32) -> usize {\n        OBJS.iter()\n            .find(|o| o.0 == pl && o.1 == ix)\n            .map(|o| o.2)\n            .expect(\"an object the draw lays\")\n    }\n    fn ty(&self, pl: i32, ix: i32) -> usize {\n        let k = OBJS\n            .iter()\n            .position(|o| o.0 == pl && o.1 == ix)\n            .expect(\"an object the draw lays\");\n        TYPE_OF[k]\n    }\n    fn div3(&self, i: i32) -> i32 {\n        rd(&self.c[1], 4 * i as usize)\n    }\n    fn semaphore(&self) -> bool {\n        self.c[4][0x821] & 2 != 0\n    }\n    fn hits_inc(&mut self, who: i32) {\n        let at = 0x7fc + 4 * who as usize;\n        let v = rd(&self.c[4], at);\n        wr(&mut self.c[4], at, v.wrapping_add(1));\n    }\n    fn close(&mut self) {\n        self.tok(5, \"m+0x0\", &[]);\n    }\n    fn domain(&self) -> i32 {\n        let (pl, ix) = (self.f(0x48), self.f(0x4c));\n        if ix >= 0 && pl >= 0 {\n            rd(&self.p[self.ty(pl, ix)], 0x218)\n        } else {\n            0\n        }\n    }\n    /// `hit_target`, and `check_hit` when it answers 0.\n    fn hit_and_check(&mut self, dom: i32) {\n        self.tok(2, \"m+0x0\", &[]);\n        if self.a[2] == 0 {\n            self.tok(3, \"m+0x0\", &[n(dom)]);\n            // `check_hit`'s stub lays the target (1, 1).\n            wr(self.m, 0x4c, 1);\n            wr(self.m, 0x48, 1);\n        }\n    }\n    fn rand(&mut self) -> i32 {\n        self.tok(6, \"c8+0x0\", &[n(0), n(0xffff)]);\n        let r = self.a[33 + self.n_rand.min(1)];\n        self.n_rand += 1;\n        r\n    }\n    fn is_ocean(&mut self, x: i32, y: i32) -> bool {\n        self.derefs[4].push(hex4(x));\n        self.derefs[5].push(hex4(y));\n        let r = self.a[35 + self.n_ocean.min(7)];\n        self.n_ocean += 1;\n        r != 0\n    }\n    /// The owner byte of cell `(x, y)` of the `+0x134` grid.\n    fn owner_at(&self, x: i32, y: i32) -> i32 {\n        let w = rd(&self.c[2], 0);\n        let at = (w.wrapping_mul(y).wrapping_add(x)) as usize * 0x1c + 0xf;\n        i32::from(self.p[40][at] as i8)\n    }\n    fn do_damage_call(&mut self, tgt: (i32, i32), angle: i32, count: i32, flag: i32) {\n        let shooter = self.obj(self.f(0x3c), self.f(0x40));\n        let args = [\n            n(tgt.0),\n            n(tgt.1),\n            n(angle),\n            n(self.f(0x44)),\n            n(self.f(0x34)),\n            n(count),\n            n(flag),\n            n(0),\n        ];\n        self.tok(4, &format!(\"p{shooter}+0x0\"), &args);\n    }\n    /// One of the two missile-shield messages (`get_neon`, `add_message`, the sound).\n    fn shield_message(&mut self, str_off: i32, cx: i32, cy: i32) {\n        let team = i32::from(self.p[4 * self.f(0x3c) as usize + 3][0]);\n        self.tok(\n            13,\n            &n(0xe39ce0 + team * 0x64),\n            &[\"sp-0x4c\".to_string()],\n        );\n        let color = self.a[13];\n        let x = cx.wrapping_mul(3).wrapping_shl(8).wrapping_add(0x180);\n        let y = cy.wrapping_mul(3).wrapping_shl(8).wrapping_add(0x180);\n        self.tok(\n            14,\n            \"c6+0x0\",\n            &[\n                n(str_off),\n                n(x),\n                n(y),\n                n(1),\n                n(color),\n                n(-1),\n                \"c3+0x0\".to_string(),\n                n(0),\n                n(0),\n                \"c3+0x0\".to_string(),\n            ],\n        );\n        self.tok(15, \"c6+0x0\", &[n(0x9a)]);\n    }\n\n    fn run(&mut self) {\n        // Two log lines.\n        let s = rd(&self.c[5], 0x10);\n        let game_log = n(0xeb1360);\n        let (f0c, f10, f14) = (self.f(0xc), self.f(0x10), self.f(0x14));\n        self.tok(\n            0,\n            &game_log,\n            &[n(s.wrapping_add(0xa14)), n(0xeb437c), n(f0c), n(f10), n(f14), n(0), n(0), n(0), n(0), n(3)],\n        );\n        let (f18, f1c, f20) = (self.f(0x18), self.f(0x1c), self.f(0x20));\n        self.tok(\n            0,\n            &game_log,\n            &[n(s.wrapping_add(0xa28)), n(0xeb437c), n(f18), n(f1c), n(f20), n(0), n(0), n(0), n(0), n(3)],\n        );\n        // A target that is gone is forgotten.\n        if self.f(0x4c) >= 0 && self.f(0x48) >= 0 {\n            let w = self.obj(self.f(0x48), self.f(0x4c));\n            if self.p[w][8] & 1 == 0 {\n                wr(self.m, 0x48, -1);\n                wr(self.m, 0x4c, -1);\n            }\n        }\n        let gp = self.a[1];\n        let f8 = self.f(8);\n        self.tok(1, \"c7+0x0\", &[n(f8)]);\n        if self.f(0x40) < 0 || self.f(0x3c) < 0 || gp == 9 {\n            return self.close();\n        }\n        if self.f(0x30) == 0 {\n            self.arm_ground();\n        } else {\n            self.arm_splash();\n        }\n    }\n\n    /// `field_0x30 == 0`: the round has no splash.\n    fn arm_ground(&mut self) {\n        let dom = self.domain();\n        self.hit_and_check(dom);\n        let (ix, pl) = (self.f(0x4c), self.f(0x48));\n        if ix < 0 || pl < 0 || (ix == self.f(0x40) && pl == self.f(0x3c)) {\n            // Nothing struck: a puncture point near the landing.\n            let r1 = self.rand();\n            let x = self.f(0x18).wrapping_add(r1 % 0x29).wrapping_sub(0x14);\n            let r2 = self.rand();\n            let y = self.f(0x1c).wrapping_add(r2 % 0x29).wrapping_sub(0x14);\n            self.derefs[0].push(hex4(x));\n            self.derefs[1].push(hex4(y));\n            let (x, y) = (self.a[31], self.a[32]);\n            let f8 = self.f(8);\n            self.tok(8, \"c7+0x0\", &[n(f8), n(1)]);\n            if self.a[8] != 0 {\n                self.tok(9, \"m+0x0\", &[]);\n                self.derefs[2].push(hex4(x));\n                self.derefs[3].push(hex4(y));\n                if self.a[10] != 0 {\n                    let w3 = rd(&self.c[2], 0x18);\n                    let at = (self\n                        .div3(y >> 6)\n                        .wrapping_mul(w3)\n                        .wrapping_add(self.div3(x >> 6))) as usize\n                        * 2;\n                    if self.p[41][at] & 0x30 != 0x20 {\n                        self.m[4] = 1;\n                        wr(self.m, 0x24, 0);\n                        return;\n                    }\n                }\n            }\n            return self.close();\n        }\n        let w = self.obj(pl, ix);\n        if self.p[w][8] & 1 != 0 {\n            let angle = crate::movement::find_angle(\n                self.f(0x18).wrapping_sub(self.f(0xc)),\n                self.f(0x1c).wrapping_sub(self.f(0x10)),\n            )\n            .0;\n            self.do_damage_call((ix, pl), angle, 0x100, 0);\n            if self.semaphore() && pl != self.f(0x3c) {\n                self.hits_inc(self.f(0x3c));\n            }\n        }\n        self.close();\n    }\n\n    /// `field_0x30 != 0`: the round splashes.\n    fn arm_splash(&mut self) {\n        let dom = self.domain();\n        let (sp, si) = (self.f(0x3c), self.f(0x40));\n        let shooter = self.obj(sp, si);\n        let sty = self.ty(sp, si);\n        let (f18, f1c) = (self.f(0x18), self.f(0x1c));\n        if rd(&self.p[sty], 0x1e4) & 0x8000000 != 0 {\n            // `Object::is(0x13b)`: the shooter is a nuke.\n            let nuke = if self.a[67] != 0 {\n                self.tok(30, &format!(\"p{sty}+0x0\"), &[n(0x13b), n(0)]);\n                self.a[30]\n            } else {\n                self.tok(29, &format!(\"p{shooter}+0x0\"), &[n(0x13b), n(0)]);\n                self.a[29]\n            };\n            let cx = self.div3(f18 >> 8);\n            let cy = self.div3(f1c >> 8);\n            let owner = if !self.is_ocean(cx, cy) {\n                self.owner_at(cx, cy)\n            } else {\n                let mut owner = -1;\n                let w = rd(&self.c[2], 0);\n                let h = rd(&self.c[2], 4);\n                let ring = crate::collide::spiral(5);\n                for &(dx, dy) in &ring[1..121] {\n                    let (xx, yy) = (cx + dx, cy + dy);\n                    if xx >= 0 && yy >= 0 && xx < w && yy < h && !self.is_ocean(xx, yy) {\n                        owner = self.owner_at(xx, yy);\n                        if owner >= 0 {\n                            break;\n                        }\n                    }\n                }\n                owner\n            };\n            let mut shield = false;\n            if owner >= 0 {\n                let ecx = n(0xe3a390 + owner.wrapping_mul(0x6eec));\n                self.tok(12, &ecx, &[n(0x2b5)]);\n                if self.a[12] != 0 && sp != owner {\n                    shield = true;\n                    let who = rd(&self.c[3], 0x298);\n                    if sp == who {\n                        self.shield_message(0x690, cx, cy);\n                    }\n                    if owner == who {\n                        self.shield_message(0x6a4, cx, cy);\n                    }\n                }\n            }\n            if shield {\n                return self.close();\n            }\n            if nuke != 0 {\n                return self.nuke_blast(sp, si);\n            }\n            self.tok(22, \"sp-0x60\", &[n(0x50), n(f18), n(f1c)]);\n            let c = &mut self.p[4 * sp as usize + 2];\n            let v = rd(c, 0xc);\n            wr(c, 0xc, v.wrapping_add(1));\n        }\n        self.hit_and_check(dom);\n        let (sv_ix, sv_pl) = (self.f(0x4c), self.f(0x48));\n        self.tok(23, &format!(\"p{shooter}+0x0\"), &[]);\n        let (mut px, mut py) = (f18, f1c);\n        if self.a[23] != 0 && rd(&self.p[sty], 0x2b4) & 0x2000 != 0 {\n            px = self.f(0xc).wrapping_add(f18);\n            px = px.wrapping_sub(px >> 31) >> 1;\n            py = self.f(0x10).wrapping_add(f1c);\n            py = py.wrapping_sub(py >> 31) >> 1;\n        }\n        let angle = crate::movement::find_angle(\n            f18.wrapping_sub(self.f(0xc)),\n            f1c.wrapping_sub(self.f(0x10)),\n        )\n        .0;\n        let area = self.f(0x30);\n        let mut ridx = (area.wrapping_add((area >> 31) & 3) >> 2).wrapping_add(1);\n        if ridx >= 11 {\n            ridx = 10;\n        }\n        let limit = (2 * ridx + 1) * (2 * ridx + 1);\n        let (ccx, ccy) = (self.div3(px >> 8), self.div3(py >> 8));\n        let (w, h) = (rd(&self.c[2], 0), rd(&self.c[2], 4));\n        let spiral = crate::collide::spiral(11);\n        let mut hit = false;\n        for i in 0..=limit as usize {\n            let (cxx, cyy) = (ccx + spiral[i].0, ccy + spiral[i].1);\n            if cxx < 0 || cyy < 0 || cxx >= w || cyy >= h {\n                continue;\n            }\n            let rec = (cyy.wrapping_mul(w).wrapping_add(cxx)) as usize * 0x1c;\n            let first = i16at(&self.p[40], rec + 8);\n            let first_pl = i16at(&self.p[40], rec + 0xa);\n            if first < 0 {\n                continue;\n            }\n            let (mut pl, mut ix) = (first_pl, first);\n            loop {\n                wr(self.m, 0x48, pl);\n                wr(self.m, 0x4c, ix);\n                let ow = self.obj(pl, ix);\n                let (next_pl, next_ix) = (i16at(&self.p[ow], 0x2e), i16at(&self.p[ow], 0x2c));\n                self.splash_one(pl, ix, (sv_ix, sv_pl), (px, py), dom, angle, &mut hit);\n                if next_ix < 0 {\n                    break;\n                }\n                pl = next_pl;\n                ix = next_ix;\n            }\n        }\n        if !hit {\n            return self.close();\n        }\n        self.tok(28, &format!(\"p{shooter}+0x0\"), &[]);\n        if self.f(8) == self.a[28] {\n            return self.close();\n        }\n        self.hits_inc(sp);\n        self.close();\n    }\n\n    /// One object on the chain of a cell.\n    #[allow(clippy::too_many_arguments)]\n    fn splash_one(\n        &mut self,\n        pl: i32,\n        ix: i32,\n        sv: (i32, i32),\n        (px, py): (i32, i32),\n        dom: i32,\n        angle: i32,\n        hit: &mut bool,\n    ) {\n        let (sp, si) = (self.f(0x3c), self.f(0x40));\n        if ix == si && pl == sp {\n            return;\n        }\n        if pl >= 8 {\n            return;\n        }\n        let intended = ix == sv.0 && pl == sv.1;\n        if !intended {\n            let row = 4 * sp as usize;\n            let f8 = rd(&self.p[row], 0);\n            if pl == f8 {\n                return;\n            }\n            if rd(&self.p[row + 1], 4 * pl as usize) != 0\n                && rd(&self.p[12 + pl as usize], 4 * f8 as usize) != 0\n            {\n                return;\n            }\n        }\n        let tgt = self.obj(pl, ix);\n        let tgt_ecx = format!(\"p{tgt}+0x0\");\n        self.tok(24, &tgt_ecx, &[]);\n        let r = self.a[43 + self.n_c.min(7)];\n        self.n_c += 1;\n        let flag = i32::from(!intended);\n        let area = self.f(0x30);\n        let den = area.wrapping_mul(0xc0);\n        let k = 0x63637;\n        if r != 0 {\n            if dom == 2 {\n                return;\n            }\n            // The footprint arm.\n            self.tok(27, &tgt_ecx, &[]);\n            let wa = 42 + self.n_b0.min(1);\n            self.n_b0 += 1;\n            let ta = if wa == 42 { 36 } else { 37 };\n            let dx1 = abs_sign(px.wrapping_sub(rd(&self.p[tgt], 0x10) ^ k))\n                .wrapping_sub(rd(&self.p[ta], 0x234).wrapping_mul(0xc0));\n            self.tok(27, &tgt_ecx, &[]);\n            let wb = 42 + self.n_b0.min(1);\n            self.n_b0 += 1;\n            let tb = if wb == 42 { 36 } else { 37 };\n            let dy1 = abs_sign(py.wrapping_sub(rd(&self.p[tgt], 0x14) ^ k))\n                .wrapping_sub(rd(&self.p[tb], 0x238).wrapping_mul(0xc0));\n            let (ex, ey) = (dx1.max(0), dy1.max(0));\n            let dist = crate::world::vector_dist(ex, ey);\n            let cnt = 0x100 - dist.wrapping_shl(8).wrapping_div(den);\n            if cnt < 0 {\n                return;\n            }\n            self.do_damage_call((ix, pl), angle, cnt, flag);\n            if self.semaphore() && pl != sp {\n                *hit = true;\n            }\n            return;\n        }\n        self.tok(25, &tgt_ecx, &[]);\n        let r2 = self.a[51 + self.n_8.min(7)];\n        self.n_8 += 1;\n        if r2 == 0 {\n            return;\n        }\n        self.tok(26, &tgt_ecx, &[]);\n        let r3 = self.a[59 + self.n_bc.min(7)];\n        self.n_bc += 1;\n        if r3 == 0 {\n            return;\n        }\n        let tty = self.ty(pl, ix);\n        if (rd(&self.p[tty], 0x218) != 2) != (dom != 2) {\n            return;\n        }\n        if rd(&self.p[tty], 0x1e4) & 0x8000000 != 0 {\n            return;\n        }\n        let ey = abs_sign(py.wrapping_sub(rd(&self.p[tgt], 0x14) ^ k));\n        let ex = abs_sign(px.wrapping_sub(rd(&self.p[tgt], 0x10) ^ k));\n        let dist = crate::world::vector_dist(ex, ey);\n        let v = (-0xc0_i32).wrapping_sub(rd(&self.p[tty], 0x23c)).wrapping_add(dist);\n        let v = v.max(0);\n        let cnt = 0x100 - v.wrapping_shl(8).wrapping_div(den);\n        if cnt <= 0 {\n            return;\n        }\n        self.do_damage_call((ix, pl), angle, cnt, flag);\n        if self.semaphore() && pl != sp {\n            *hit = true;\n        }\n    }\n\n    /// The nuke's blast: the leader stamped, the effect laid, the ground lit.\n    fn nuke_blast(&mut self, sp: i32, si: i32) {\n        let row = 4 * sp as usize + 2;\n        let frame = rd(&self.c[4], 0x550);\n        wr(&mut self.p[row], 0, frame);\n        let v = rd(&self.p[row], 8);\n        wr(&mut self.p[row], 8, v.wrapping_add(1));\n        let nn = rd(&self.p[self.ty(sp, si)], 0x200);\n        let (f18, f1c) = (self.f(0x18), self.f(0x1c));\n        self.tok(\n            18,\n            \"sp-0x70\",\n            &[n(f18), n(f1c), n(0x64), n(nn.wrapping_mul(3).wrapping_shl(6)), n(sp), n(si)],\n        );\n        let ring = (nn.wrapping_add(1)).clamp(0, 0x40) as usize;\n        let tx = self.div3(f18 >> 6);\n        let ty = self.div3(f1c >> 6);\n        let circle = crate::ai_place::circle();\n        let (w3, h3) = (rd(&self.c[2], 0x18), rd(&self.c[2], 0x1c));\n        for i in 0..circle.radius[ring] {\n            let y = circle.y[i] + ty;\n            let x = circle.x[i] + tx;\n            if x >= 0 && y >= 0 && x < w3 && y < h3 {\n                self.derefs[6].push(hex4(x >> 1));\n                self.derefs[7].push(hex4(y >> 1));\n                self.tok(\n                    19,\n                    &n(x >> 1),\n                    &[\"sp-0x2c\".to_string(), \"sp-0x30\".to_string(), n(0xff), n(0)],\n                );\n            }\n        }\n        self.tok(\n            20,\n            \"c6+0x0\",\n            &[n(f18), n(f1c), n(-1), n(0xc8d26c), n(-1), n(0xc00), n(0), n(0), n(0)],\n        );\n        self.tok(21, \"c6+0x0\", &[]);\n        self.close();\n    }\n}\n\nlet mut env = Env {\n    a,\n    m,\n    c,\n    p,\n    toks: Vec::new(),\n    derefs: Default::default(),\n    n_rand: 0,\n    n_ocean: 0,\n    n_c: 0,\n    n_8: 0,\n    n_bc: 0,\n    n_b0: 0,\n};\nenv.run();\nlog.extend(env.toks);\nfor d in &env.derefs {\n    log.push(if d.is_empty() { \"-\".to_string() } else { d.join(\"/\") });\n}\n0"]
