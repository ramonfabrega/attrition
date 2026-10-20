#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_data_get_damage.py — `ObjectData::get_damage@00644130` under unicorn (item 3356, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_object_data_get_damage.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 96-byte object of random bytes (no bias), 6 stack argument(s);
`p` windows in order: p0 argument 5, p1..p6 sparse table 0 (2 rows × 3 slice(s), row-major), p7..p43 the place(s), in flag order, p44..p52 the stubs' answer window(s); the global cell 0x00e85ddc points at 2192 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00c061e4 points at 3072 random bytes; the global cell 0x00c061e8 points at 2096 random bytes; the global cell 0x00c061d4 points at 96 random bytes; 2112 random bytes at 0x00c06afc; argument 5 names 16 random bytes (null a third of the rows); callee 0x006db810 (1 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x00646b00 (2 args) stubbed; callee 0x0062dd90 (0 args) stubbed; callee 0x006469f0 (0 args) stubbed; callee 0x006da000 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x3c (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0xd0 (0 args) stubbed; vtable slot +0xd8 (0 args) stubbed; vtable slot +0xe4 (0 args) stubbed; vtable slot +0xec (0 args) stubbed; vtable slot +0x10c (0 args) stubbed; vtable slot +0x120 (0 args) stubbed; vtable slot +0x124 (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0x184 (0 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; the word at `0xc0ab84` points at 8 random bytes less `0` (placed); the word at `p7+0` points at 64 random bytes less `0` (placed); the word at `p7+4` points at 64 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 8 random bytes less `0` (placed); the word at `p10+0` points at 64 random bytes less `0` (placed); the word at `p10+4` points at 64 random bytes less `0` (placed); the word at `0xc0aec0` points at 8 random bytes less `0` (placed); the word at `p13+0` points at 256 random bytes less `0` (placed); the word at `p13+4` points at 256 random bytes less `0` (placed); the word at `0xc0aec0+0x1c` points at 8 random bytes less `0` (placed); the word at `p16+0` points at 256 random bytes less `0` (placed); the word at `p16+4` points at 256 random bytes less `0` (placed); the word at `0xc0aaf4` points at 16 random bytes less `0` (placed); the word at `p19+0` points at 32 random bytes less `0` (placed); the word at `p19+4` points at 32 random bytes less `0` (placed); the word at `p19+8` points at 32 random bytes less `0` (placed); the word at `p19+12` points at 32 random bytes less `0` (placed); the word at `c5+0x10` points at 8 random bytes less `0` (placed); the word at `p24+0` points at 128 random bytes less `0` (placed); the word at `p24+4` points at 128 random bytes less `0` (placed); the word at `c5+0x2c` points at 8 random bytes less `0` (placed); the word at `p27+0` points at 128 random bytes less `0` (placed); the word at `p27+4` points at 128 random bytes less `0` (placed); the word at `c0+0x888` points at 64 random bytes less `0` (placed); the word at `c2+0x134` points at 2560 random bytes less `0` (placed); the word at `m+0x18` points at 784 random bytes less `0` (placed); the word at `p3+0` points at 256 random bytes less `0` (placed); the word at `p6+0` points at 256 random bytes less `0` (placed); the word at `p8+0x18` points at 784 random bytes less `0` (placed); the word at `p9+0x18` points at 784 random bytes less `0` (placed); the word at `p11+0x18` points at 784 random bytes less `0` (placed); the word at `p12+0x18` points at 784 random bytes less `0` (placed); the word at `p14+0x18` points at 784 random bytes less `0` (placed); the word at `p15+0x18` points at 784 random bytes less `0` (placed); the word at `p17+0x18` points at 784 random bytes less `0` (placed); the word at `p18+0x18` points at 784 random bytes less `0` (placed); the word at `m+0x30` points at 512 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [-2, -1, 0, 1, 2] (90%); stub 3 answered from [4, 4, 0, 1, 2, 3] (90%); stub 4 answered from [0, 10, 50, 100, 500, 1000, -5, 3000] (90%); stub 5 answered from [0, 1, 2, 0, 1] (90%); stub 6 answered from [1, 1, 1, 0] (90%); stub 7 answered from [0, 1] (90%); stub 8 answered from [0, 1, 1] (90%); stub 11 answered from [0, 1] (90%); stub 12 answered from [0, 1] (90%); stub 13 answered from [0, 1] (90%); stub 14 answered from [0, 0, 1] (90%); stub 15 answered from [0, 0, 1] (90%); stub 16 answered from [0, 1, 1] (90%); stub 17 answered from [0, 1, 2, 3, 5, -1] (90%); stub 19 answered from [0, 1] (90%); stub 20 answered from [0, 10, 50, 100, 500, 1000, -3, 3000] (90%); stub 21 answered from [0, 1, 2, 5, 10, 50, -3, 200] (90%); stub 22 answered from [0, 1] (90%); stub 23 answered from [0, 1] (90%); stub 24 answered from [0, 1] (90%); stub 25 answered from [0, 1] (90%); stub 9 answers a window of its own to each call, in call order (128 bytes, ...); stub 10 answers a window of its own to each call, in call order (64 bytes, ...); the word at `p44+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p45+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p46+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p47+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p48+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p49+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p50+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p44+0x18` laid as the address of `p43` every row; the word at `p45+0x18` laid as the address of `p43` every row; the word at `p46+0x18` laid as the address of `p43` every row; the word at `p47+0x18` laid as the address of `p43` every row; the word at `p48+0x18` laid as the address of `p43` every row; the word at `p49+0x18` laid as the address of `p43` every row; the word at `p50+0x18` laid as the address of `p43` every row; stub 0 answers 2 values in call order, printed after the answers; stub 1 answers 2 values in call order, printed after the answers; stub 6 answers 17 values in call order, printed after the answers; stub 7 answers 2 values in call order, printed after the answers; stub 8 answers 9 values in call order, printed after the answers; stub 11 answers 3 values in call order, printed after the answers; stub 12 answers 7 values in call order, printed after the answers; stub 13 answers 6 values in call order, printed after the answers; stub 14 answers 2 values in call order, printed after the answers; stub 19 answers 5 values in call order, printed after the answers; stub 20 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 1024 bytes under the call is zeroed before each row; 2 rows 0x6eec apart at `0xc061e0`, slices +0x59de:0x4, +0x6c5c:0x4, +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p40+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p51+0` as well as at `this`; the stubbed vtable laid at `p52+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`.
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

ENTRY = 0x00644130
WIN = 96
OWN = 6
BIAS = []
ARGSET = [(3, [0, 1]), (4, [0, 1, 1]), (2, [0, -1431655764, -1431655765, -1431655766, 1431655767, 1431655766, 1431655765, -536870911, -536870912, 1431655767, 1431655766, 536870913, 536870912, -536870912, -536870913, -1431655765, -1431655766])]
ARGFROM = []
CELLS = [(15228380, 2192), (13297148, 1024), (12607952, 320), (12607972, 3072), (12607976, 2096), (12607956, 96)]
AT = [(12610300, 2112)]
ATRANGE = {}
PTRARGS = [(5, 16)]
PNULLS = [5]
STUBS = [('call', 7190544, 1), ('call', 7213936, 1), ('call', 6580992, 2), ('call', 6479248, 0), ('call', 6580720, 0), ('call', 7184384, 0), ('slot', 24, 0), ('slot', 28, 0), ('slot', 32, 0), ('slot', 60, 0), ('slot', 64, 0), ('slot', 76, 0), ('slot', 96, 2), ('slot', 184, 2), ('slot', 204, 0), ('slot', 208, 0), ('slot', 216, 0), ('slot', 228, 0), ('slot', 236, 0), ('slot', 268, 0), ('slot', 288, 0), ('slot', 292, 0), ('slot', 304, 0), ('slot', 388, 0), ('slot', 12, 0), ('slot', 20, 0)]
PLACES = [(None, '0xc0ab84', '0', 8), ('p7', '0', '0', 64), ('p7', '4', '0', 64), (None, '0xc0ab84+0x1c', '0', 8), ('p10', '0', '0', 64), ('p10', '4', '0', 64), (None, '0xc0aec0', '0', 8), ('p13', '0', '0', 256), ('p13', '4', '0', 256), (None, '0xc0aec0+0x1c', '0', 8), ('p16', '0', '0', 256), ('p16', '4', '0', 256), (None, '0xc0aaf4', '0', 16), ('p19', '0', '0', 32), ('p19', '4', '0', 32), ('p19', '8', '0', 32), ('p19', '12', '0', 32), ('c5', '0x10', '0', 8), ('p24', '0', '0', 128), ('p24', '4', '0', 128), ('c5', '0x2c', '0', 8), ('p27', '0', '0', 128), ('p27', '4', '0', 128), ('c0', '0x888', '0', 64), ('c2', '0x134', '0', 2560), ('m', '0x18', '0', 784), ('p3', '0', '0', 256), ('p6', '0', '0', 256), ('p8', '0x18', '0', 784), ('p9', '0x18', '0', 784), ('p11', '0x18', '0', 784), ('p12', '0x18', '0', 784), ('p14', '0x18', '0', 784), ('p15', '0x18', '0', 784), ('p17', '0x18', '0', 784), ('p18', '0x18', '0', 784), ('m', '0x30', '0', 512)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 8, ('set', [0, 32, 65536, 65568, 256, 288])), ('m', 12, ('set', [-407325, -407096, 407095, 407094, 407093, 407045, 407323])), ('p32', 4, ('set', [443, 444, 50, 51, 52, 53, 54, 0, 5, 66, 7])), ('p32', 64, ('set', [428, 5, 3, 428])), ('p32', 484, ('set', [0, 8, 2, 4, 8388608, 131072, 134217728, 268435456, 8192, 2147483648, 262144, 32, 2097152, 4096, 65800, 65536, 256, 131080, 8388610, 8224, 262176, 268443648, 40, 65832, 12288, 134348800, 266240])), ('p32', 536, ('set', [0, 0, 1, 1, 2, 2])), ('p35', 4, ('set', [0, 1])), ('p36', 4, ('set', [0, 1])), ('p37', 4, ('set', [0, 1])), ('p38', 4, ('set', [0, 1])), ('p38', 4, ('set', [0, 1, 541, 541])), ('p35', 64, ('set', [-3, -1, 0, 1, 2, 3])), ('p36', 64, ('set', [-3, -1, 0, 1, 2, 3])), ('p37', 64, ('set', [-3, -1, 0, 1, 2, 3])), ('p38', 64, ('set', [-3, -1, 0, 1, 2, 3])), ('p35', 484, ('set', [0, 8, 2, 4, 8388608, 131072, 134217728, 268435456, 8192, 2147483648, 262144, 32, 2097152, 4096, 65800, 65536, 256, 131080, 8388610, 8224, 262176, 268443648, 40, 65832, 12288, 134348800, 266240])), ('p36', 484, ('set', [0, 8, 2, 4, 8388608, 131072, 134217728, 268435456, 8192, 2147483648, 262144, 32, 2097152, 4096, 65800, 65536, 256, 131080, 8388610, 8224, 262176, 268443648, 40, 65832, 12288, 134348800, 266240])), ('p37', 484, ('set', [0, 8, 2, 4, 8388608, 131072, 134217728, 268435456, 8192, 2147483648, 262144, 32, 2097152, 4096, 65800, 65536, 256, 131080, 8388610, 8224, 262176, 268443648, 40, 65832, 12288, 134348800, 266240])), ('p38', 484, ('set', [0, 8, 2, 4, 8388608, 131072, 134217728, 268435456, 8192, 2147483648, 262144, 32, 2097152, 4096, 65800, 65536, 256, 131080, 8388610, 8224, 262176, 268443648, 40, 65832, 12288, 134348800, 266240])), ('p35', 516, ('set', [0, 25, 50, 100, 200, -20])), ('p36', 516, ('set', [0, 25, 50, 100, 200, -20])), ('p37', 516, ('set', [0, 25, 50, 100, 200, -20])), ('p38', 516, ('set', [0, 25, 50, 100, 200, -20])), ('p35', 536, ('set', [0, 0, 1, 1, 2, 2])), ('p36', 536, ('set', [0, 0, 1, 1, 2, 2])), ('p37', 536, ('set', [0, 0, 1, 1, 2, 2])), ('p38', 536, ('set', [0, 0, 1, 1, 2, 2])), ('p39', 64, ('set', [427, 5, 427, 3])), ('p40', 64, ('set', [427, 5, 427, 3])), ('p41', 64, ('set', [427, 5, 427, 3])), ('p42', 64, ('set', [427, 5, 427, 3])), ('p39', 536, ('set', [0, 0, 1, 1, 2, 2])), ('p40', 536, ('set', [0, 0, 1, 1, 2, 2])), ('p41', 536, ('set', [0, 0, 1, 1, 2, 2])), ('p42', 536, ('set', [0, 0, 1, 1, 2, 2])), ('p39', 696, ('set', [0, 4, 255, 256, 260])), ('p40', 696, ('set', [0, 4, 255, 256, 260])), ('p41', 696, ('set', [0, 4, 255, 256, 260])), ('p42', 696, ('set', [0, 4, 255, 256, 260])), ('p39', 776, ('set', [1, 2, 3, 5, 7, 10])), ('p40', 776, ('set', [1, 2, 3, 5, 7, 10])), ('p41', 776, ('set', [1, 2, 3, 5, 7, 10])), ('p42', 776, ('set', [1, 2, 3, 5, 7, 10])), ('p43', 488, ('set', [0, 0, 1, 5])), ('p30', 4, ('set', [49, 50, 256, 413, 414, 415, 5, 79])), ('p8', 12, ('set', [-407325, -407096, 407095, 407094, 407093, 407045, 407323])), ('p9', 12, ('set', [-407325, -407096, 407095, 407094, 407093, 407045, 407323])), ('p11', 12, ('set', [-407325, -407096, 407095, 407094, 407093, 407045, 407323])), ('p12', 12, ('set', [-407325, -407096, 407095, 407094, 407093, 407045, 407323])), ('p8', 16, ('set', [407095, 407112, 407351, 407368, 406583, 406600, 406839, 406856, 406327, 406344, 409143, 409160, 408631, 408648, 407863, 407880, 401975, 401992, 404279, 404296])), ('p8', 20, ('set', [407095, 407112, 407351, 407368, 406583, 406600, 406839, 406856, 406327, 406344, 409143, 409160, 408631, 408648, 407863, 407880, 401975, 401992, 404279, 404296])), ('p9', 16, ('set', [407095, 407112, 407351, 407368, 406583, 406600, 406839, 406856, 406327, 406344, 409143, 409160, 408631, 408648, 407863, 407880, 401975, 401992, 404279, 404296])), ('p9', 20, ('set', [407095, 407112, 407351, 407368, 406583, 406600, 406839, 406856, 406327, 406344, 409143, 409160, 408631, 408648, 407863, 407880, 401975, 401992, 404279, 404296])), ('p11', 16, ('set', [407095, 407112, 407351, 407368, 406583, 406600, 406839, 406856, 406327, 406344, 409143, 409160, 408631, 408648, 407863, 407880, 401975, 401992, 404279, 404296])), ('p11', 20, ('set', [407095, 407112, 407351, 407368, 406583, 406600, 406839, 406856, 406327, 406344, 409143, 409160, 408631, 408648, 407863, 407880, 401975, 401992, 404279, 404296])), ('p12', 16, ('set', [407095, 407112, 407351, 407368, 406583, 406600, 406839, 406856, 406327, 406344, 409143, 409160, 408631, 408648, 407863, 407880, 401975, 401992, 404279, 404296])), ('p12', 20, ('set', [407095, 407112, 407351, 407368, 406583, 406600, 406839, 406856, 406327, 406344, 409143, 409160, 408631, 408648, 407863, 407880, 401975, 401992, 404279, 404296])), ('p14', 12, ('set', [-407325, -407096, 407095, 407094, 407093, 407045, 407323])), ('p14', 104, ('set', [0, 1, 524288, 4194304, 33554432, 524289, 34078720, 33554433, 37748736, 38273025])), ('p14', 108, ('set', [0, 4096])), ('p14', 76, ('set', [0, 1, 3, 50, 90, 95, 100, 990, 1000, -7])), ('p14', 80, ('set', [0, 0, 0, 324508639, 4041331395])), ('p14', 92, ('set', [0, 0, 0, 324508639, 4041331395])), ('p14', 164, ('set', [0, 1, 2, 3, 5, 65535])), ('p15', 12, ('set', [-407325, -407096, 407095, 407094, 407093, 407045, 407323])), ('p15', 104, ('set', [0, 1, 524288, 4194304, 33554432, 524289, 34078720, 33554433, 37748736, 38273025])), ('p15', 108, ('set', [0, 4096])), ('p15', 76, ('set', [0, 1, 3, 50, 90, 95, 100, 990, 1000, -7])), ('p15', 80, ('set', [0, 0, 0, 324508639, 4041331395])), ('p15', 92, ('set', [0, 0, 0, 324508639, 4041331395])), ('p15', 164, ('set', [0, 1, 2, 3, 5, 65535])), ('p17', 12, ('set', [-407325, -407096, 407095, 407094, 407093, 407045, 407323])), ('p17', 104, ('set', [0, 1, 524288, 4194304, 33554432, 524289, 34078720, 33554433, 37748736, 38273025])), ('p17', 108, ('set', [0, 4096])), ('p17', 76, ('set', [0, 1, 3, 50, 90, 95, 100, 990, 1000, -7])), ('p17', 80, ('set', [0, 0, 0, 324508639, 4041331395])), ('p17', 92, ('set', [0, 0, 0, 324508639, 4041331395])), ('p17', 164, ('set', [0, 1, 2, 3, 5, 65535])), ('p18', 12, ('set', [-407325, -407096, 407095, 407094, 407093, 407045, 407323])), ('p18', 104, ('set', [0, 1, 524288, 4194304, 33554432, 524289, 34078720, 33554433, 37748736, 38273025])), ('p18', 108, ('set', [0, 4096])), ('p18', 76, ('set', [0, 1, 3, 50, 90, 95, 100, 990, 1000, -7])), ('p18', 80, ('set', [0, 0, 0, 324508639, 4041331395])), ('p18', 92, ('set', [0, 0, 0, 324508639, 4041331395])), ('p18', 164, ('set', [0, 1, 2, 3, 5, 65535])), ('c3', 68, ('set', [1, 2, 3, 5, 10])), ('c3', 72, ('set', [0, 1, 2, 5, 100, -3])), ('c3', 76, ('set', [0, 5, 25, 50, 100, 256, -20])), ('c3', 80, ('set', [0, 64, 128, 256, 300])), ('c3', 84, ('set', [0, 64, 128, 256, 300])), ('c3', 88, ('set', [0, 128, 256, 200])), ('c3', 92, ('set', [0, 1, 2, 10, 100, 1000, 50])), ('c3', 96, ('set', [0, 64, 85, 128, 256, -50])), ('c3', 100, ('set', [0, 64, 128, 256, 300])), ('c3', 104, ('set', [0, 128, 256, 64])), ('c3', 108, ('set', [0, 128, 256, 300, 1000])), ('c3', 2968, ('set', [0, 128, 256, 400])), ('c3', 1220, ('set', [0, 25, 50, 100, 150, -10])), ('c3', 1368, ('set', [0, 1, 2, 0])), ('c3', 1900, ('set', [0, 25, 50, 100, -30])), ('c3', 1940, ('set', [-5, -10, 0, 10, 20, -1])), ('c3', 3004, ('set', [0, 1, 5, 10, -3])), ('c4', 36, ('set', [0, 1, 2, 2, 256, 258, 770])), ('c4', 1360, ('set', [0, 1, 3, 100, 1000, 1003, 2000, -5])), ('c4', 2080, ('set', [0, 512, 768, 256, 65280, 1024])), ('c2', 0, ('set', [1, 2, 3, 4, 5, 8])), ('p31', None, ('set', [0, 8, 16777216, 16777224, 4278190080, 4278190088, 134217728])), ('p25', 92, ('set', [0, 16777216, 4278190080, 17970262, 16777215])), ('p26', 92, ('set', [0, 16777216, 4278190080, 17970262, 16777215])), ('p28', 92, ('set', [0, 16777216, 4278190080, 17970262, 16777215])), ('p29', 92, ('set', [0, 16777216, 4278190080, 17970262, 16777215])), ('p44', 112, ('set', [0, 65536, 4294901760, 65536, 0])), ('p45', 112, ('set', [0, 65536, 4294901760, 65536, 0])), ('p46', 112, ('set', [0, 65536, 4294901760, 65536, 0])), ('p47', 112, ('set', [0, 65536, 4294901760, 65536, 0])), ('p48', 112, ('set', [0, 65536, 4294901760, 65536, 0])), ('p49', 112, ('set', [0, 65536, 4294901760, 65536, 0])), ('p50', 112, ('set', [0, 65536, 4294901760, 65536, 0])), ('p1', 0, ('set', [0, 1, 65535, 5, 65536])), ('p2', 0, ('set', [0, 4, 255, 1, 251])), ('p4', 0, ('set', [0, 1, 65535, 5, 65536])), ('p5', 0, ('set', [0, 4, 255, 1, 251])), ('p33', 220, ('set', [403302, 403303, 403300, 403301, 403310])), ('p34', 220, ('set', [403302, 403303, 403300, 403301, 403310])), ('p33', 232, ('set', [405895, 405894, 405893, 405892, 405903])), ('p34', 232, ('set', [405895, 405894, 405893, 405892, 405903]))]
FILLS = [('c1', 'i//3'), ('c6', '(((2*i)*37+11)%251)|((((2*i+1)*37+11)%251)<<16)')]
ARGRANGE = [(0, 0, 1), (1, 0, 1)]
SLOTBASE = [(184, 6633360), (12, 4655072)]
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [0, 1]), (1, [0, 1]), (2, [-2, -1, 0, 1, 2]), (3, [4, 4, 0, 1, 2, 3]), (4, [0, 10, 50, 100, 500, 1000, -5, 3000]), (5, [0, 1, 2, 0, 1]), (6, [1, 1, 1, 0]), (7, [0, 1]), (8, [0, 1, 1]), (11, [0, 1]), (12, [0, 1]), (13, [0, 1]), (14, [0, 0, 1]), (15, [0, 0, 1]), (16, [0, 1, 1]), (17, [0, 1, 2, 3, 5, -1]), (19, [0, 1]), (20, [0, 10, 50, 100, 500, 1000, -3, 3000]), (21, [0, 1, 2, 5, 10, 50, -3, 200]), (22, [0, 1]), (23, [0, 1]), (24, [0, 1]), (25, [0, 1])]
VTAT = None
VTATHALF = []
VTIMAGE = [('p44', 0, 11805044), ('p45', 0, 11805044), ('p46', 0, 11805044), ('p47', 0, 11805044), ('p48', 0, 11805044), ('p49', 0, 11805044), ('p50', 0, 11805044)]
ALIAS = [('p44', 24, 'p43', 0, True), ('p45', 24, 'p43', 0, True), ('p46', 24, 'p43', 0, True), ('p47', 24, 'p43', 0, True), ('p48', 24, 'p43', 0, True), ('p49', 24, 'p43', 0, True), ('p50', 24, 'p43', 0, True)]
ANSWERBY = []
ANSWERWIN = [(9, 128), (9, 128), (9, 128), (9, 128), (9, 128), (9, 128), (9, 128), (10, 64), (10, 64)]
CDECL = False
TIB = False
ARGEQ = []
SEQ = [(0, 2, None), (1, 2, None), (6, 17, None), (7, 2, None), (8, 9, None), (11, 3, None), (12, 7, None), (13, 6, None), (14, 2, None), (19, 5, None), (20, 2, None)]
ANSRANGE = []
CELLTHIS = []
VTALSO = [('p30', 0), ('p32', 0), ('p35', 0), ('p36', 0), ('p37', 0), ('p38', 0), ('p39', 0), ('p40', 0), ('p41', 0), ('p42', 0), ('p51', 0), ('p52', 0), ('p8', 0), ('p9', 0), ('p11', 0), ('p12', 0), ('p14', 0), ('p15', 0), ('p17', 0), ('p18', 0), ('p20', 0), ('p21', 0), ('p22', 0), ('p23', 0)]
SPARSE = [(None, '0xc061e0', 28396, 2, [(23006, 4), (27740, 4), (28344, 4)])]
GTAB = []
RECORD = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 19, 20, 21, 22, 23, 24, 25]
RECORDORDER = True
RECORDEDX = False
ROWSPLIT = {0: 0, 1: 0, 5: 0}
LINKS = []
LINKHI = []
OUTARG = []
CDECLSTUBS = []
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

# sweep_gen: ["obj", "ObjectData::get_damage@00644130", "--item=3356", "--name=object_data_get_damage", "--win=0x60", "--own=6", "--ret=eax", "--rows=2500", "--clean-stack=0x400", "--ptrarg=5:0x10", "--ptrarg-null=5", "--argrange=0:0:1", "--argrange=1:0:1", "--argset=3:0,1", "--argset=4:0,1,1", "--argset=2:0,-1431655764,-1431655765,-1431655766,1431655767,1431655766,1431655765,-536870911,-536870912,1431655767,1431655766,536870913,536870912,-536870912,-536870913,-1431655765,-1431655766", "--cell-win=0xe85ddc:0x890", "--cell-win=0xcae5fc:0x400", "--cell-win=0xc061d0:0x140", "--cell-win=0xc061e4:0xc00", "--cell-win=0xc061e8:0x830", "--cell-win=0xc061d4:0x60", "--at=0xc06afc:0x840", "--fill=c1:i//3", "--fill=c6:(((2*i)*37+11)%251)|((((2*i+1)*37+11)%251)<<16)", "--sparse=0xc061e0:0x6eec:2:0x59de,4/0x6c5c,4/0x6eb8,4", "--ptrtab=0xc0ab84:2:0x40:vt", "--ptrtab=0xc0ab84+0x1c:2:0x40:vt", "--ptrtab=0xc0aec0:2:0x100:vt", "--ptrtab=0xc0aec0+0x1c:2:0x100:vt", "--ptrtab=0xc0aaf4:4:0x20:vt", "--ptrtab=c5+0x10:2:0x80", "--ptrtab=c5+0x2c:2:0x80", "--place=c0+0x888:0:0x40", "--place=c2+0x134:0:0xa00", "--place=m+0x18:0:0x310", "--place=p3+0:0:0x100", "--place=p6+0:0:0x100", "--place=p8+0x18:0:0x310", "--place=p9+0x18:0:0x310", "--place=p11+0x18:0:0x310", "--place=p12+0x18:0:0x310", "--place=p14+0x18:0:0x310", "--place=p15+0x18:0:0x310", "--place=p17+0x18:0:0x310", "--place=p18+0x18:0:0x310", "--place=m+0x30:0:0x200", "--stub=0x6db810:1", "--stub=0x6e1370:1", "--stub=0x646b00:2", "--stub=0x62dd90:0", "--stub=0x6469f0:0", "--stub=0x6da000:0", "--stub=slot:0x18:0", "--stub=slot:0x1c:0", "--stub=slot:0x20:0", "--stub=slot:0x3c:0", "--stub=slot:0x40:0", "--stub=slot:0x4c:0", "--stub=slot:0x60:2", "--stub=slot:0xb8:2", "--stub=slot:0xcc:0", "--stub=slot:0xd0:0", "--stub=slot:0xd8:0", "--stub=slot:0xe4:0", "--stub=slot:0xec:0", "--stub=slot:0x10c:0", "--stub=slot:0x120:0", "--stub=slot:0x124:0", "--stub=slot:0x130:0", "--stub=slot:0x184:0", "--stub=slot:0xc:0", "--stub=slot:0x14:0", "--answers=0:0,1", "--answers=1:0,1", "--answers=2:-2,-1,0,1,2", "--answers=3:4,4,0,1,2,3", "--answers=4:0,10,50,100,500,1000,-5,3000", "--answers=5:0,1,2,0,1", "--answers=6:1,1,1,0", "--answers=7:0,1", "--answers=8:0,1,1", "--answers=11:0,1", "--answers=12:0,1", "--answers=13:0,1", "--answers=14:0,0,1", "--answers=15:0,0,1", "--answers=16:0,1,1", "--answers=17:0,1,2,3,5,-1", "--answers=19:0,1", "--answers=20:0,10,50,100,500,1000,-3,3000", "--answers=21:0,1,2,5,10,50,-3,200", "--answers=22:0,1", "--answers=23:0,1", "--answers=24:0,1", "--answers=25:0,1", "--seq=0:2", "--seq=1:2", "--seq=6:17", "--seq=7:2", "--seq=8:9", "--seq=11:3", "--seq=12:7", "--seq=13:6", "--seq=14:2", "--seq=19:5", "--seq=20:2", "--answer-win=9:0x80", "--answer-win=9:0x80", "--answer-win=9:0x80", "--answer-win=9:0x80", "--answer-win=9:0x80", "--answer-win=9:0x80", "--answer-win=9:0x80", "--answer-win=10:0x40", "--answer-win=10:0x40", "--slotbase=0xb8:0x653790", "--vtat-keep=0xc:0x4707e0", "--vtat-also=p30+0", "--vtat-also=p32+0", "--vtat-also=p35+0", "--vtat-also=p36+0", "--vtat-also=p37+0", "--vtat-also=p38+0", "--vtat-also=p39+0", "--vtat-also=p40+0", "--vtat-also=p41+0", "--vtat-also=p42+0", "--vtat-also=p51+0", "--vtat-also=p52+0", "--vtat-image=p44+0:0xb42174", "--alias=p44+0x18:p43!", "--vtat-image=p45+0:0xb42174", "--alias=p45+0x18:p43!", "--vtat-image=p46+0:0xb42174", "--alias=p46+0x18:p43!", "--vtat-image=p47+0:0xb42174", "--alias=p47+0x18:p43!", "--vtat-image=p48+0:0xb42174", "--alias=p48+0x18:p43!", "--vtat-image=p49+0:0xb42174", "--alias=p49+0x18:p43!", "--vtat-image=p50+0:0xb42174", "--alias=p50+0x18:p43!", "--wbias=m+8:0x0,0x20,0x10000,0x10020,0x100,0x120", "--wbias=m+0xc:-407325,-407096,407095,407094,407093,407045,407323", "--wbias=p32+4:0x1bb,0x1bc,0x32,0x33,0x34,0x35,0x36,0,5,0x42,7", "--wbias=p32+0x40:0x1ac,5,3,0x1ac", "--wbias=p32+0x1e4:0x0,0x8,0x2,0x4,0x800000,0x20000,0x8000000,0x10000000,0x2000,0x80000000,0x40000,0x20,0x200000,0x1000,0x10108,0x10000,0x100,0x20008,0x800002,0x2020,0x40020,0x10002000,0x28,0x10128,0x3000,0x8020000,0x41000", "--wbias=p32+0x218:0,0,1,1,2,2", "--wbias=p35..38+4:0,1", "--wbias=p38+4:0,1,0x21d,0x21d", "--wbias=p35..38+0x40:-3,-1,0,1,2,3", "--wbias=p35..38+0x1e4:0x0,0x8,0x2,0x4,0x800000,0x20000,0x8000000,0x10000000,0x2000,0x80000000,0x40000,0x20,0x200000,0x1000,0x10108,0x10000,0x100,0x20008,0x800002,0x2020,0x40020,0x10002000,0x28,0x10128,0x3000,0x8020000,0x41000", "--wbias=p35..38+0x204:0,25,50,100,200,-20", "--wbias=p35..38+0x218:0,0,1,1,2,2", "--wbias=p39..42+0x40:0x1ab,5,0x1ab,3", "--wbias=p39..42+0x218:0,0,1,1,2,2", "--wbias=p39..42+0x2b8:0,4,0xff,0x100,0x104", "--wbias=p39..42+0x308:1,2,3,5,7,10", "--wbias=p43+0x1e8:0,0,1,5", "--wbias=p30+4:0x31,0x32,0x100,0x19d,0x19e,0x19f,5,0x4f", "--wbias=p8..9+0xc:-407325,-407096,407095,407094,407093,407045,407323", "--wbias=p11..12+0xc:-407325,-407096,407095,407094,407093,407045,407323", "--wbias=p8+0x10:407095,407112,407351,407368,406583,406600,406839,406856,406327,406344,409143,409160,408631,408648,407863,407880,401975,401992,404279,404296", "--wbias=p8+0x14:407095,407112,407351,407368,406583,406600,406839,406856,406327,406344,409143,409160,408631,408648,407863,407880,401975,401992,404279,404296", "--wbias=p9+0x10:407095,407112,407351,407368,406583,406600,406839,406856,406327,406344,409143,409160,408631,408648,407863,407880,401975,401992,404279,404296", "--wbias=p9+0x14:407095,407112,407351,407368,406583,406600,406839,406856,406327,406344,409143,409160,408631,408648,407863,407880,401975,401992,404279,404296", "--wbias=p11+0x10:407095,407112,407351,407368,406583,406600,406839,406856,406327,406344,409143,409160,408631,408648,407863,407880,401975,401992,404279,404296", "--wbias=p11+0x14:407095,407112,407351,407368,406583,406600,406839,406856,406327,406344,409143,409160,408631,408648,407863,407880,401975,401992,404279,404296", "--wbias=p12+0x10:407095,407112,407351,407368,406583,406600,406839,406856,406327,406344,409143,409160,408631,408648,407863,407880,401975,401992,404279,404296", "--wbias=p12+0x14:407095,407112,407351,407368,406583,406600,406839,406856,406327,406344,409143,409160,408631,408648,407863,407880,401975,401992,404279,404296", "--wbias=p14+0xc:-407325,-407096,407095,407094,407093,407045,407323", "--wbias=p14+0x68:0x0,0x1,0x80000,0x400000,0x2000000,0x80001,0x2080000,0x2000001,0x2400000,0x2480001", "--wbias=p14+0x6c:0,0x1000", "--wbias=p14+0x4c:0,1,3,50,90,95,100,990,1000,-7", "--wbias=p14+0x50:0,0,0,0x13579bdf,0xf0e1d2c3", "--wbias=p14+0x5c:0,0,0,0x13579bdf,0xf0e1d2c3", "--wbias=p14+0xa4:0,1,2,3,5,0xffff", "--wbias=p15+0xc:-407325,-407096,407095,407094,407093,407045,407323", "--wbias=p15+0x68:0x0,0x1,0x80000,0x400000,0x2000000,0x80001,0x2080000,0x2000001,0x2400000,0x2480001", "--wbias=p15+0x6c:0,0x1000", "--wbias=p15+0x4c:0,1,3,50,90,95,100,990,1000,-7", "--wbias=p15+0x50:0,0,0,0x13579bdf,0xf0e1d2c3", "--wbias=p15+0x5c:0,0,0,0x13579bdf,0xf0e1d2c3", "--wbias=p15+0xa4:0,1,2,3,5,0xffff", "--wbias=p17+0xc:-407325,-407096,407095,407094,407093,407045,407323", "--wbias=p17+0x68:0x0,0x1,0x80000,0x400000,0x2000000,0x80001,0x2080000,0x2000001,0x2400000,0x2480001", "--wbias=p17+0x6c:0,0x1000", "--wbias=p17+0x4c:0,1,3,50,90,95,100,990,1000,-7", "--wbias=p17+0x50:0,0,0,0x13579bdf,0xf0e1d2c3", "--wbias=p17+0x5c:0,0,0,0x13579bdf,0xf0e1d2c3", "--wbias=p17+0xa4:0,1,2,3,5,0xffff", "--wbias=p18+0xc:-407325,-407096,407095,407094,407093,407045,407323", "--wbias=p18+0x68:0x0,0x1,0x80000,0x400000,0x2000000,0x80001,0x2080000,0x2000001,0x2400000,0x2480001", "--wbias=p18+0x6c:0,0x1000", "--wbias=p18+0x4c:0,1,3,50,90,95,100,990,1000,-7", "--wbias=p18+0x50:0,0,0,0x13579bdf,0xf0e1d2c3", "--wbias=p18+0x5c:0,0,0,0x13579bdf,0xf0e1d2c3", "--wbias=p18+0xa4:0,1,2,3,5,0xffff", "--wbias=c3+0x44:1,2,3,5,10", "--wbias=c3+0x48:0,1,2,5,100,-3", "--wbias=c3+0x4c:0,5,25,50,100,256,-20", "--wbias=c3+0x50:0,64,128,256,300", "--wbias=c3+0x54:0,64,128,256,300", "--wbias=c3+0x58:0,128,256,200", "--wbias=c3+0x5c:0,1,2,10,100,1000,50", "--wbias=c3+0x60:0,64,85,128,256,-50", "--wbias=c3+0x64:0,64,128,256,300", "--wbias=c3+0x68:0,128,256,64", "--wbias=c3+0x6c:0,128,256,300,1000", "--wbias=c3+0xb98:0,128,256,400", "--wbias=c3+0x4c4:0,25,50,100,150,-10", "--wbias=c3+0x558:0,1,2,0", "--wbias=c3+0x76c:0,25,50,100,-30", "--wbias=c3+0x794:-5,-10,0,10,20,-1", "--wbias=c3+0xbbc:0,1,5,10,-3", "--wbias=c4+0x24:0,1,2,2,0x100,0x102,0x302", "--wbias=c4+0x550:0,1,3,100,1000,1003,2000,-5", "--wbias=c4+0x820:0,0x200,0x300,0x100,0xff00,0x400", "--wbias=c2+0:1,2,3,4,5,8", "--wbias=p31+*:0,0x8,0x01000000,0x01000008,0xff000000,0xff000008,0x08000000", "--wbias=p25+0x5c:0,0x01000000,0xff000000,0x01123456,0x00ffffff", "--wbias=p26+0x5c:0,0x01000000,0xff000000,0x01123456,0x00ffffff", "--wbias=p28+0x5c:0,0x01000000,0xff000000,0x01123456,0x00ffffff", "--wbias=p29+0x5c:0,0x01000000,0xff000000,0x01123456,0x00ffffff", "--wbias=p44+0x70:0,0x10000,0xffff0000,0x10000,0", "--wbias=p45+0x70:0,0x10000,0xffff0000,0x10000,0", "--wbias=p46+0x70:0,0x10000,0xffff0000,0x10000,0", "--wbias=p47+0x70:0,0x10000,0xffff0000,0x10000,0", "--wbias=p48+0x70:0,0x10000,0xffff0000,0x10000,0", "--wbias=p49+0x70:0,0x10000,0xffff0000,0x10000,0", "--wbias=p50+0x70:0,0x10000,0xffff0000,0x10000,0", "--wbias=p1+0:0,1,0xffff,5,0x10000", "--wbias=p2+0:0,4,0xff,1,0xfb", "--wbias=p4+0:0,1,0xffff,5,0x10000", "--wbias=p5+0:0,4,0xff,1,0xfb", "--wbias=p33+0xdc:403302,403303,403300,403301,403310", "--wbias=p34+0xdc:403302,403303,403300,403301,403310", "--wbias=p33+0xe8:405895,405894,405893,405892,405903", "--wbias=p34+0xe8:405895,405894,405893,405892,405903", "--record=0@0", "--record=1@0", "--record=2", "--record=3", "--record=4", "--record=5@0", "--record=6", "--record=7", "--record=8", "--record=9", "--record=10", "--record=11", "--record=12", "--record=13", "--record=14", "--record=15", "--record=16", "--record=17", "--record=19", "--record=20", "--record=21", "--record=22", "--record=23", "--record=24", "--record=25", "--record-order"]
