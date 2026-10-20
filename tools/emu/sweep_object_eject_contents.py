#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_eject_contents.py — `Object::eject_contents@0064cd20` under unicorn (item 3356, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_object_eject_contents.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 256-byte object of random bytes (no bias), 4 stack argument(s);
`p` windows in order: p0..p5 sparse table 0 (2 rows × 3 slice(s), row-major), p6..p51 the place(s), in flag order, p52..p54 the stubs' answer window(s); the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00c061ec points at 2304 random bytes; the global cell 0x00c061a8 points at 32 random bytes; the global cell 0x00e85ddc points at 272 random bytes; the global cell 0x00c06210 points at 688 random bytes; the global cell 0x00c061b4 points at 64 random bytes; the global cell 0x00e8d444 points at 5088 random bytes; the global cell 0x00e85f20 points at 7260 random bytes; vtable slot +0xbc (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x158 (3 args) stubbed; vtable slot +0x80 (2 args) stubbed; vtable slot +0x84 (2 args) stubbed; vtable slot +0x70 (0 args) stubbed; vtable slot +0x48 (2 args) stubbed; vtable slot +0xd0 (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0x68 (1 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x10 (4 args) stubbed; callee 0x00646c40 (1 args) stubbed; callee 0x00608450 (0 args) stubbed; callee 0x005e29b0 (0 args) stubbed; callee 0x00675bc0 (3 args) stubbed; callee 0x005e4a60 (7 args) stubbed; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00617c10 (1 args) stubbed; callee 0x00647080 (3 args) stubbed; callee 0x006d9380 (1 args) stubbed; callee 0x006e3140 (1 args) stubbed; callee 0x005e3860 (0 args) stubbed; callee 0x0065ba90 (9 args) stubbed; callee 0x00616ed0 (9 args) stubbed; callee 0x005e48c0 (7 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x005f9400 (2 args) stubbed; callee 0x0046d620 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; callee 0x0046d9a0 (0 args) stubbed; callee 0x0046e8c0 (1 args) stubbed; callee 0x0046d820 (1 args) stubbed; callee 0x005fd080 (0 args) stubbed; callee 0x00649d70 (0 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x0070e520 (1 args) stubbed; callee 0x00714110 (4 args) stubbed; callee 0x0070f9e0 (3 args) stubbed; callee 0x0070e620 (1 args) stubbed; callee 0x0046f600 (0 args) stubbed; callee 0x0046f790 (0 args) stubbed; callee 0x0070f8f0 (3 args) stubbed; callee 0x00643b70 (2 args) stubbed; the word at `0xc0aec0+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 256 random bytes less `0` (placed); the word at `p6+4` points at 256 random bytes less `0` (placed); the word at `p6+8` points at 256 random bytes less `0` (placed); the word at `p6+12` points at 256 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p11+0` points at 256 random bytes less `0` (placed); the word at `p11+4` points at 256 random bytes less `0` (placed); the word at `p11+8` points at 256 random bytes less `0` (placed); the word at `p11+12` points at 256 random bytes less `0` (placed); the word at `0xc09970+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p16+0` points at 32 random bytes less `0` (placed); the word at `p16+4` points at 32 random bytes less `0` (placed); the word at `0xc09970+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p19+0` points at 32 random bytes less `0` (placed); the word at `p19+4` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*0` points at 16 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 16 random bytes less `0` (placed); the word at `c2+0x10` points at 2656 random bytes less `0` (placed); the word at `c3+0x0` points at 16 random bytes less `0` (placed); the word at `c3+0x4` points at 16 random bytes less `0` (placed); the word at `c3+0x8` points at 16 random bytes less `0` (placed); the word at `c3+0xc` points at 16 random bytes less `0` (placed); the word at `c3+0x108` points at 16 random bytes less `0` (placed); the word at `p7+0x18` points at 784 random bytes less `0` (placed); the word at `p8+0x18` points at 784 random bytes less `0` (placed); the word at `p9+0x18` points at 784 random bytes less `0` (placed); the word at `p10+0x18` points at 784 random bytes less `0` (placed); the word at `p12+0x18` points at 784 random bytes less `0` (placed); the word at `p13+0x18` points at 784 random bytes less `0` (placed); the word at `p14+0x18` points at 784 random bytes less `0` (placed); the word at `p15+0x18` points at 784 random bytes less `0` (placed); the word at `m+0x18` points at 784 random bytes less `0` (placed); the word at `m+0xb8` points at 128 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p40+4` points at 16 random bytes less `0` (placed); the word at `p7+0xb8` points at 128 random bytes less `0` (placed); the word at `p8+0xb8` points at 128 random bytes less `0` (placed); the word at `p9+0xb8` points at 128 random bytes less `0` (placed); the word at `p12+0xb8` points at 128 random bytes less `0` (placed); the word at `p13+0xb8` points at 128 random bytes less `0` (placed); the word at `p14+0xb8` points at 128 random bytes less `0` (placed); the word at `0xe3a2a0+0x1c*0` points at 12 random bytes less `-4` (placed); the word at `p48+0` points at 16 random bytes less `0` (placed); the word at `p48+4` points at 16 random bytes less `0` (placed); the word at `p48+8` points at 16 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0x70 laid as 0x00662670 or its stub, a flag after the answers; slot +0xd0 laid as 0x0046ce90 or its stub, a flag after the answers; slot +0x158 laid as 0x0060eda0 or its stub, a flag after the answers; slot +0x80 laid as 0x0046ce10 or its stub, a flag after the answers; stub 0 answered from [1, 1, 1, 1, 1, 1, 1, 1, 0] (90%); stub 1 answered from [0, 1, 1, 1, 1] (90%); stub 3 answered from [0, 1, 1, 0] (90%); stub 7 answered from [0, 1, 1] (90%); stub 8 answered from [0, 1, 1] (90%); stub 9 answered from [0, 1, 1] (90%); stub 10 answered from [0, 1, 1] (90%); stub 12 answered from [0, 1, 1, 0] (90%); stub 13 answered from [1, 1, 1, 0] (90%); stub 15 answered from [0, 1] (90%); stub 16 answered from [0, 1] (90%); stub 18 answered from [0, 1] (90%); stub 23 answered from [0, 0, 0, 0, 0, 1] (90%); stub 25 answered from [0, 1] (90%); stub 39 answered from [-1, 0, 3] (90%); stub 48 answered from [0, 1] (90%); stub 2 answers a window of its own to each call, in call order (128 bytes, ...); the word at `p6+12` laid as the address of `m` every row; the word at `p22+0` laid as the address of `p7` every row; the word at `p22+4` laid as the address of `p8` every row; the word at `p22+8` laid as the address of `p9` every row; the word at `p22+12` laid as the address of `m` every row; the word at `p23+0` laid as the address of `p12` every row; the word at `p23+4` laid as the address of `p13` every row; the word at `p23+8` laid as the address of `p14` every row; the word at `p23+12` laid as the address of `p15` every row; stub 1 answers 4 values in call order, printed after the answers; stub 3 answers 16 values in call order, printed after the answers; stub 12 answers 16 values in call order, printed after the answers; stub 23 answers 4 values in call order, printed after the answers; stub 25 answers 4 values in call order, printed after the answers; stub 28 answers 3 values in call order, printed after the answers; stub 11 answers 0..127 every row; stub 28 answers -1..1 every row; stub 44 answers 0..2 every row; stub 47 answers 0..4 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 30 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 31 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 32 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 33 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 34 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 35 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 36 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 37 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 38 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 39 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 40 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 41 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 42 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 43 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 44 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 45 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 46 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 47 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 48 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 49 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 37 prints the 16 bytes its argument 0 points at, hex, after the record (`/` a call); the stack 1024 bytes under the call is zeroed before each row; stub 23 lowers `m+0x28` (2 bytes) by one each call; stub 23 lowers `p7+0x28` (2 bytes) by one each call; stub 23 lowers `p8+0x28` (2 bytes) by one each call; stub 23 lowers `p9+0x28` (2 bytes) by one each call; stub 23 lowers `p12+0x28` (2 bytes) by one each call; stub 23 lowers `p13+0x28` (2 bytes) by one each call; stub 23 lowers `p14+0x28` (2 bytes) by one each call; stub 24 lowers `m+0x28` (2 bytes) by one each call; stub 24 lowers `p7+0x28` (2 bytes) by one each call; stub 24 lowers `p8+0x28` (2 bytes) by one each call; stub 24 lowers `p9+0x28` (2 bytes) by one each call; stub 24 lowers `p12+0x28` (2 bytes) by one each call; stub 24 lowers `p13+0x28` (2 bytes) by one each call; stub 24 lowers `p14+0x28` (2 bytes) by one each call; stub 4 lowers `m+0x28` (2 bytes) by one each call; stub 4 lowers `p7+0x28` (2 bytes) by one each call; stub 4 lowers `p8+0x28` (2 bytes) by one each call; stub 4 lowers `p9+0x28` (2 bytes) by one each call; stub 4 lowers `p12+0x28` (2 bytes) by one each call; stub 4 lowers `p13+0x28` (2 bytes) by one each call; stub 4 lowers `p14+0x28` (2 bytes) by one each call; stub 33 lowers `m+0xd8` (4 bytes) by one each call; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x8:0x4, +0x6c18:0x10 (`p` windows after the pointed buffers, before the places); figures p7..p10 link at +0x28 (2 bytes) below their own index or to -1; figures p12..p15 link at +0x28 (2 bytes) below their own index or to -1; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `c6+0` as well as at `this`; the stubbed vtable laid at `c6+0x9f0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`.
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

ENTRY = 0x0064cd20
WIN = 256
OWN = 4
BIAS = []
ARGSET = [(-1, [0, 1, 1, 1]), (-2, [-1, -1, -1, -1, -1, 0, 1, 2, 3]), (-3, [0, 1, 1]), (-4, [0, 0, 1])]
ARGFROM = []
CELLS = [(12607884, 128), (12607980, 2304), (12607912, 32), (15228380, 272), (12608016, 688), (12607924, 64), (15258692, 5088), (15228704, 7260)]
AT = []
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('slot', 188, 0), ('slot', 32, 0), ('slot', 172, 0), ('slot', 184, 2), ('slot', 344, 3), ('slot', 128, 2), ('slot', 132, 2), ('slot', 112, 0), ('slot', 72, 2), ('slot', 208, 0), ('slot', 304, 0), ('slot', 104, 1), ('slot', 96, 2), ('slot', 24, 0), ('slot', 16, 4), ('call', 6581312, 1), ('call', 6325328, 0), ('call', 6171056, 0), ('call', 6773696, 3), ('call', 6179424, 7), ('call', 6174704, 1), ('call', 6175008, 0), ('call', 6334576, 0), ('call', 6388752, 1), ('call', 6582400, 3), ('call', 7181184, 1), ('call', 7221568, 1), ('call', 6174816, 0), ('call', 6666896, 9), ('call', 6385360, 9), ('call', 6179008, 7), ('call', 9959280, 1), ('call', 6263808, 2), ('call', 4642336, 0), ('call', 4642208, 1), ('call', 4643232, 0), ('call', 4647104, 1), ('call', 4642848, 1), ('call', 6279296, 0), ('call', 6593904, 0), ('call', 7421568, 1), ('call', 7422800, 4), ('call', 7398688, 1), ('call', 7422224, 4), ('call', 7404000, 3), ('call', 7398944, 1), ('call', 4650496, 0), ('call', 4650896, 0), ('call', 7403760, 3), ('call', 6568816, 2)]
PLACES = [(None, '0xc0aec0+0x1c*0', '0', 16), ('p6', '0', '0', 256), ('p6', '4', '0', 256), ('p6', '8', '0', 256), ('p6', '12', '0', 256), (None, '0xc0aec0+0x1c*1', '0', 16), ('p11', '0', '0', 256), ('p11', '4', '0', 256), ('p11', '8', '0', 256), ('p11', '12', '0', 256), (None, '0xc09970+0x1c*0', '0', 8), ('p16', '0', '0', 32), ('p16', '4', '0', 32), (None, '0xc09970+0x1c*1', '0', 8), ('p19', '0', '0', 32), ('p19', '4', '0', 32), ('c0', '0x14+0x1c*0', '0', 16), ('c0', '0x14+0x1c*1', '0', 16), ('c2', '0x10', '0', 2656), ('c3', '0x0', '0', 16), ('c3', '0x4', '0', 16), ('c3', '0x8', '0', 16), ('c3', '0xc', '0', 16), ('c3', '0x108', '0', 16), ('p7', '0x18', '0', 784), ('p8', '0x18', '0', 784), ('p9', '0x18', '0', 784), ('p10', '0x18', '0', 784), ('p12', '0x18', '0', 784), ('p13', '0x18', '0', 784), ('p14', '0x18', '0', 784), ('p15', '0x18', '0', 784), ('m', '0x18', '0', 784), ('m', '0xb8', '0', 128), ('m', '0xdc', '0', 16), ('p40', '4', '0', 16), ('p7', '0xb8', '0', 128), ('p8', '0xb8', '0', 128), ('p9', '0xb8', '0', 128), ('p12', '0xb8', '0', 128), ('p13', '0xb8', '0', 128), ('p14', '0xb8', '0', 128), (None, '0xe3a2a0+0x1c*0', '-4', 12), ('p48', '0', '0', 16), ('p48', '4', '0', 16), ('p48', '8', '0', 16)]
NULLS = [34]
NULLFORCE = {}
WBIAS = [('m', 8, ('set', [196608])), ('m', 40, ('set', [0, 1, 2, 65535, 2, 2, 2, 2])), ('m', 60, ('set', [0, 65536])), ('m', 104, ('set', [0, 512, 262144, 67108864, 262656])), ('m', 132, ('set', [0, 65536])), ('m', 192, ('set', [0, 1, 2, 3, -5])), ('m', 216, ('set', [0, 1, 2])), ('c4', 664, ('set', [0, 1])), ('c4', 672, ('set', [0, 1])), ('p0', 0, ('set', [256, 512, 1024, 0, 1280])), ('p3', 0, ('set', [256, 512, 1024, 0, 1280])), ('p30', 4, ('set', [50, 51, 52, 53, 49, 64])), ('p30', 776, ('set', [0, 1, 1, 2])), ('p31', 4, ('set', [50, 51, 52, 53, 49, 64])), ('p31', 776, ('set', [0, 1, 1, 2])), ('p32', 4, ('set', [50, 51, 52, 53, 49, 64])), ('p32', 776, ('set', [0, 1, 1, 2])), ('p33', 4, ('set', [50, 51, 52, 53, 49, 64])), ('p33', 776, ('set', [0, 1, 1, 2])), ('p34', 4, ('set', [50, 51, 52, 53, 49, 64])), ('p34', 776, ('set', [0, 1, 1, 2])), ('p35', 4, ('set', [50, 51, 52, 53, 49, 64])), ('p35', 776, ('set', [0, 1, 1, 2])), ('p36', 4, ('set', [50, 51, 52, 53, 49, 64])), ('p36', 776, ('set', [0, 1, 1, 2])), ('p37', 4, ('set', [50, 51, 52, 53, 49, 64])), ('p37', 776, ('set', [0, 1, 1, 2])), ('p38', 4, ('set', [50, 51, 52, 53, 49, 64])), ('p38', 776, ('set', [0, 1, 1, 2])), ('p25', 4, ('set', [50, 51, 49, 52])), ('p26', 4, ('set', [50, 51, 49, 52])), ('p27', 4, ('set', [50, 51, 49, 52])), ('p28', 4, ('set', [50, 51, 49, 52])), ('p7', 60, ('set', [0, 65536])), ('p8', 60, ('set', [0, 65536])), ('p9', 60, ('set', [0, 65536])), ('p10', 60, ('set', [0, 65536])), ('p12', 60, ('set', [0, 65536])), ('p13', 60, ('set', [0, 65536])), ('p14', 60, ('set', [0, 65536])), ('p15', 60, ('set', [0, 65536])), ('p7', 104, ('set', [0, 512, 262144, 67108864, 262656])), ('p7', 132, ('set', [0, 65536])), ('p7', 188, ('set', [0, 1, 2, 4])), ('p7', 192, ('set', [0, 1, 2])), ('p8', 104, ('set', [0, 512, 262144, 67108864, 262656])), ('p8', 132, ('set', [0, 65536])), ('p8', 188, ('set', [0, 1, 2, 4])), ('p8', 192, ('set', [0, 1, 2])), ('p9', 104, ('set', [0, 512, 262144, 67108864, 262656])), ('p9', 132, ('set', [0, 65536])), ('p9', 188, ('set', [0, 1, 2, 4])), ('p9', 192, ('set', [0, 1, 2])), ('p12', 104, ('set', [0, 512, 262144, 67108864, 262656])), ('p12', 132, ('set', [0, 65536])), ('p12', 188, ('set', [0, 1, 2, 4])), ('p12', 192, ('set', [0, 1, 2])), ('p13', 104, ('set', [0, 512, 262144, 67108864, 262656])), ('p13', 132, ('set', [0, 65536])), ('p13', 188, ('set', [0, 1, 2, 4])), ('p13', 192, ('set', [0, 1, 2])), ('p14', 104, ('set', [0, 512, 262144, 67108864, 262656])), ('p14', 132, ('set', [0, 65536])), ('p14', 188, ('set', [0, 1, 2, 4])), ('p14', 192, ('set', [0, 1, 2])), ('p52', 112, ('set', [4294901760, 0, 65536, 0, 65536])), ('p53', 112, ('set', [4294901760, 0, 65536, 0, 65536])), ('p54', 112, ('set', [4294901760, 0, 65536, 0, 65536]))]
FILLS = []
ARGRANGE = []
SLOTBASE = [(184, 6633360), (112, 6694512), (208, 4640400), (344, 6352288), (128, 4640272)]
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [1, 1, 1, 1, 1, 1, 1, 1, 0]), (1, [0, 1, 1, 1, 1]), (3, [0, 1, 1, 0]), (7, [0, 1, 1]), (8, [0, 1, 1]), (9, [0, 1, 1]), (10, [0, 1, 1]), (12, [0, 1, 1, 0]), (13, [1, 1, 1, 0]), (15, [0, 1]), (16, [0, 1]), (18, [0, 1]), (23, [0, 0, 0, 0, 0, 1]), (25, [0, 1]), (39, [-1, 0, 3]), (48, [0, 1])]
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = [('p6', 12, 'm', 0, True), ('p22', 0, 'p7', 0, True), ('p22', 4, 'p8', 0, True), ('p22', 8, 'p9', 0, True), ('p22', 12, 'm', 0, True), ('p23', 0, 'p12', 0, True), ('p23', 4, 'p13', 0, True), ('p23', 8, 'p14', 0, True), ('p23', 12, 'p15', 0, True)]
ANSWERBY = []
ANSWERWIN = [(2, 128), (2, 128), (2, 128)]
CDECL = False
TIB = True
ARGEQ = []
SEQ = [(1, 4, None), (3, 16, None), (12, 16, None), (23, 4, None), (25, 4, None), (28, 3, None)]
ANSRANGE = [(11, 0, 127), (28, -1, 1), (44, 0, 2), (47, 0, 4)]
CELLTHIS = []
VTALSO = [('p29', 0), ('p30', 0), ('p31', 0), ('p32', 0), ('p33', 0), ('p34', 0), ('p35', 0), ('p36', 0), ('p37', 0), ('p38', 0), ('c6', 0), ('c6', 2544), ('p7', 0), ('p8', 0), ('p9', 0), ('p10', 0), ('p12', 0), ('p13', 0), ('p14', 0), ('p15', 0)]
SPARSE = [('@', '0xe3a390', 28396, 2, [(0, 4), (8, 4), (27672, 16)])]
GTAB = []
RECORD = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49]
RECORDORDER = True
RECORDEDX = False
ROWSPLIT = {}
LINKS = [(7, 10, 40, 2, '<', [-1]), (12, 15, 40, 2, '<', [-1])]
LINKHI = []
OUTARG = []
CDECLSTUBS = []
STUBWRITE = [(23, 'm', 40, 2, 'dec'), (23, 'p7', 40, 2, 'dec'), (23, 'p8', 40, 2, 'dec'), (23, 'p9', 40, 2, 'dec'), (23, 'p12', 40, 2, 'dec'), (23, 'p13', 40, 2, 'dec'), (23, 'p14', 40, 2, 'dec'), (24, 'm', 40, 2, 'dec'), (24, 'p7', 40, 2, 'dec'), (24, 'p8', 40, 2, 'dec'), (24, 'p9', 40, 2, 'dec'), (24, 'p12', 40, 2, 'dec'), (24, 'p13', 40, 2, 'dec'), (24, 'p14', 40, 2, 'dec'), (4, 'm', 40, 2, 'dec'), (4, 'p7', 40, 2, 'dec'), (4, 'p8', 40, 2, 'dec'), (4, 'p9', 40, 2, 'dec'), (4, 'p12', 40, 2, 'dec'), (4, 'p13', 40, 2, 'dec'), (4, 'p14', 40, 2, 'dec'), (33, 'm', 216, 4, 'dec')]
STUBCOPY = []
STACKCLEAN = 1024
INITCALLS = []
MAXINSNS = 200000
ECXANSWER = []
ECXOUT = []
RING = []
DEREF = [(37, 0, 16, None)]
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

# sweep_gen: ["--item=3356", "obj", "Object::eject_contents@0064cd20", "--name=object_eject_contents", "--win=0x100", "--own=4", "--ret=none", "--tib", "--rows=4000", "--clean-stack=0x400", "--argset=0!:0,1,1,1", "--argset=1!:-1,-1,-1,-1,-1,0,1,2,3", "--argset=2!:0,1,1", "--argset=3!:0,0,1", "--cell-win=0xc0618c:0x80", "--cell-win=0xc061ec:0x900", "--cell-win=0xc061a8:0x20", "--cell-win=0xe85ddc:0x110", "--cell-win=0xc06210:0x2b0", "--cell-win=0xc061b4:0x40", "--cell-win=0xe8d444:0x13e0", "--cell-win=0xe85f20:0x1c5c", "--sparse=@0xe3a390:0x6eec:2:0,4/8,4/0x6c18,0x10", "--ptrtab=0xc0aec0+0x1c*0:4:0x100:vt", "--ptrtab=0xc0aec0+0x1c*1:4:0x100:vt", "--ptrtab=0xc09970+0x1c*0:2:0x20", "--ptrtab=0xc09970+0x1c*1:2:0x20", "--stub=slot:0xbc:0", "--stub=slot:0x20:0", "--stub=slot:0xac:0", "--stub=slot:0xb8:2", "--stub=slot:0x158:3", "--stub=slot:0x80:2", "--stub=slot:0x84:2", "--stub=slot:0x70:0", "--stub=slot:0x48:2", "--stub=slot:0xd0:0", "--stub=slot:0x130:0", "--stub=slot:0x68:1", "--stub=slot:0x60:2", "--stub=slot:0x18:0", "--stub=slot:0x10:4", "--stub=0x646c40:1", "--stub=0x608450:0", "--stub=0x5e29b0:0", "--stub=0x675bc0:3", "--stub=0x5e4a60:7", "--stub=0x5e37f0:1", "--stub=0x5e3920:0", "--stub=0x60a870:0", "--stub=0x617c10:1", "--stub=0x647080:3", "--stub=0x6d9380:1", "--stub=0x6e3140:1", "--stub=0x5e3860:0", "--stub=0x65ba90:9", "--stub=0x616ed0:9", "--stub=0x5e48c0:7", "--stub=0x97f770:1", "--stub=0x5f9400:2", "--stub=0x46d620:0", "--stub=0x46d5a0:1", "--stub=0x46d9a0:0", "--stub=0x46e8c0:1", "--stub=0x46d820:1", "--stub=0x5fd080:0", "--stub=0x649d70:0", "--stub=0x713e80:1", "--stub=0x714350:4", "--stub=0x70e520:1", "--stub=0x714110:4", "--stub=0x70f9e0:3", "--stub=0x70e620:1", "--stub=0x46f600:0", "--stub=0x46f790:0", "--stub=0x70f8f0:3", "--stub=0x643b70:2", "--place=c0+0x14+0x1c*0:0:0x10", "--place=c0+0x14+0x1c*1:0:0x10", "--place=c2+0x10:0:0xa60", "--place=c3+0x0:0:0x10", "--place=c3+0x4:0:0x10", "--place=c3+0x8:0:0x10", "--place=c3+0xc:0:0x10", "--place=c3+0x108:0:0x10", "--place=p7+0x18:0:0x310", "--place=p8+0x18:0:0x310", "--place=p9+0x18:0:0x310", "--place=p10+0x18:0:0x310", "--place=p12+0x18:0:0x310", "--place=p13+0x18:0:0x310", "--place=p14+0x18:0:0x310", "--place=p15+0x18:0:0x310", "--place=m+0x18:0:0x310", "--place=m+0xb8:0:0x80", "--place=m+0xdc:0:0x10", "--place=p40+4:0:0x10", "--place=p7+0xb8:0:0x80", "--place=p8+0xb8:0:0x80", "--place=p9+0xb8:0:0x80", "--place=p12+0xb8:0:0x80", "--place=p13+0xb8:0:0x80", "--place=p14+0xb8:0:0x80", "--place=0xe3a2a0+0x1c*0:-4:0xc", "--place=p48+0:0:0x10", "--place=p48+4:0:0x10", "--place=p48+8:0:0x10", "--place-null=34", "--answer-win=2:0x80", "--answer-win=2:0x80", "--answer-win=2:0x80", "--alias=p6+12:m!", "--alias=p22+0:p7!", "--alias=p22+4:p8!", "--alias=p22+8:p9!", "--alias=p22+12:m!", "--alias=p23+0:p12!", "--alias=p23+4:p13!", "--alias=p23+8:p14!", "--alias=p23+12:p15!", "--vtat-also=p29+0", "--vtat-also=p30+0", "--vtat-also=p31+0", "--vtat-also=p32+0", "--vtat-also=p33+0", "--vtat-also=p34+0", "--vtat-also=p35+0", "--vtat-also=p36+0", "--vtat-also=p37+0", "--vtat-also=p38+0", "--vtat-also=c6+0", "--vtat-also=c6+0x9f0", "--vtat-keep=0xb8:0x653790", "--vtat-keep=0x70:0x662670", "--vtat-keep=0xd0:0x46ce90", "--vtat-keep=0x158:0x60eda0", "--vtat-keep=0x80:0x46ce10", "--stub-write=23:m+0x28:2:dec", "--stub-write=23:p7+0x28:2:dec", "--stub-write=23:p8+0x28:2:dec", "--stub-write=23:p9+0x28:2:dec", "--stub-write=23:p12+0x28:2:dec", "--stub-write=23:p13+0x28:2:dec", "--stub-write=23:p14+0x28:2:dec", "--stub-write=24:m+0x28:2:dec", "--stub-write=24:p7+0x28:2:dec", "--stub-write=24:p8+0x28:2:dec", "--stub-write=24:p9+0x28:2:dec", "--stub-write=24:p12+0x28:2:dec", "--stub-write=24:p13+0x28:2:dec", "--stub-write=24:p14+0x28:2:dec", "--stub-write=4:m+0x28:2:dec", "--stub-write=4:p7+0x28:2:dec", "--stub-write=4:p8+0x28:2:dec", "--stub-write=4:p9+0x28:2:dec", "--stub-write=4:p12+0x28:2:dec", "--stub-write=4:p13+0x28:2:dec", "--stub-write=4:p14+0x28:2:dec", "--stub-write=33:m+0xd8:4:dec", "--wbias=m+8:0x30000", "--wbias=m+0x28:0,1,2,0xffff,2,2,2,2", "--wbias=m+0x3c:0,0x10000", "--wbias=m+0x68:0,0x200,0x40000,0x4000000,0x40200", "--wbias=m+0x84:0,0x10000", "--wbias=m+0xc0:0,1,2,3,-5", "--wbias=m+0xd8:0,1,2", "--wbias=c4+0x298:0,1", "--wbias=c4+0x2a0:0,1", "--wbias=p0+0:0x100,0x200,0x400,0,0x500", "--wbias=p3+0:0x100,0x200,0x400,0,0x500", "--wbias=p30+4:0x32,0x33,0x34,0x35,0x31,0x40", "--wbias=p30+0x308:0,1,1,2", "--wbias=p31+4:0x32,0x33,0x34,0x35,0x31,0x40", "--wbias=p31+0x308:0,1,1,2", "--wbias=p32+4:0x32,0x33,0x34,0x35,0x31,0x40", "--wbias=p32+0x308:0,1,1,2", "--wbias=p33+4:0x32,0x33,0x34,0x35,0x31,0x40", "--wbias=p33+0x308:0,1,1,2", "--wbias=p34+4:0x32,0x33,0x34,0x35,0x31,0x40", "--wbias=p34+0x308:0,1,1,2", "--wbias=p35+4:0x32,0x33,0x34,0x35,0x31,0x40", "--wbias=p35+0x308:0,1,1,2", "--wbias=p36+4:0x32,0x33,0x34,0x35,0x31,0x40", "--wbias=p36+0x308:0,1,1,2", "--wbias=p37+4:0x32,0x33,0x34,0x35,0x31,0x40", "--wbias=p37+0x308:0,1,1,2", "--wbias=p38+4:0x32,0x33,0x34,0x35,0x31,0x40", "--wbias=p38+0x308:0,1,1,2", "--wbias=p25+4:0x32,0x33,0x31,0x34", "--wbias=p26+4:0x32,0x33,0x31,0x34", "--wbias=p27+4:0x32,0x33,0x31,0x34", "--wbias=p28+4:0x32,0x33,0x31,0x34", "--wbias=p7+0x3c:0,0x10000", "--wbias=p8+0x3c:0,0x10000", "--wbias=p9+0x3c:0,0x10000", "--wbias=p10+0x3c:0,0x10000", "--wbias=p12+0x3c:0,0x10000", "--wbias=p13+0x3c:0,0x10000", "--wbias=p14+0x3c:0,0x10000", "--wbias=p15+0x3c:0,0x10000", "--wbias=p7+0x68:0,0x200,0x40000,0x4000000,0x40200", "--wbias=p7+0x84:0,0x10000", "--wbias=p7+0xbc:0,1,2,4", "--wbias=p7+0xc0:0,1,2", "--wbias=p8+0x68:0,0x200,0x40000,0x4000000,0x40200", "--wbias=p8+0x84:0,0x10000", "--wbias=p8+0xbc:0,1,2,4", "--wbias=p8+0xc0:0,1,2", "--wbias=p9+0x68:0,0x200,0x40000,0x4000000,0x40200", "--wbias=p9+0x84:0,0x10000", "--wbias=p9+0xbc:0,1,2,4", "--wbias=p9+0xc0:0,1,2", "--wbias=p12+0x68:0,0x200,0x40000,0x4000000,0x40200", "--wbias=p12+0x84:0,0x10000", "--wbias=p12+0xbc:0,1,2,4", "--wbias=p12+0xc0:0,1,2", "--wbias=p13+0x68:0,0x200,0x40000,0x4000000,0x40200", "--wbias=p13+0x84:0,0x10000", "--wbias=p13+0xbc:0,1,2,4", "--wbias=p13+0xc0:0,1,2", "--wbias=p14+0x68:0,0x200,0x40000,0x4000000,0x40200", "--wbias=p14+0x84:0,0x10000", "--wbias=p14+0xbc:0,1,2,4", "--wbias=p14+0xc0:0,1,2", "--wbias=p52+0x70:0xffff0000,0,0x10000,0,0x10000", "--wbias=p53+0x70:0xffff0000,0,0x10000,0,0x10000", "--wbias=p54+0x70:0xffff0000,0,0x10000,0,0x10000", "--link=p7..10+0x28:2:<:-1", "--link=p12..15+0x28:2:<:-1", "--record=0", "--record=1", "--record=2", "--record=3", "--record=4", "--record=5", "--record=6", "--record=7", "--record=8", "--record=9", "--record=10", "--record=11", "--record=12", "--record=13", "--record=14", "--record=15", "--record=16", "--record=17", "--record=18", "--record=19", "--record=20", "--record=21", "--record=22", "--record=23", "--record=24", "--record=25", "--record=26", "--record=27", "--record=28", "--record=29", "--record=30", "--record=31", "--record=32", "--record=33", "--record=34", "--record=35", "--record=36", "--record=37", "--record=38", "--record=39", "--record=40", "--record=41", "--record=42", "--record=43", "--record=44", "--record=45", "--record=46", "--record=47", "--record=48", "--record=49", "--record-order", "--deref=37:0:16", "--answers=0:1,1,1,1,1,1,1,1,0", "--answers=1:0,1,1,1,1", "--answers=3:0,1,1,0", "--answers=7:0,1,1", "--answers=8:0,1,1", "--answers=9:0,1,1", "--answers=10:0,1,1", "--answers=12:0,1,1,0", "--answers=13:1,1,1,0", "--answers=15:0,1", "--answers=16:0,1", "--answers=18:0,1", "--answers=23:0,0,0,0,0,1", "--answers=25:0,1", "--answers=39:-1,0,3", "--answers=48:0,1", "--answer-range=11:0:127", "--answer-range=28:-1:1", "--answer-range=44:0:2", "--answer-range=47:0:4", "--seq=1:4", "--seq=3:16", "--seq=12:16", "--seq=23:4", "--seq=25:4", "--seq=28:3"]
