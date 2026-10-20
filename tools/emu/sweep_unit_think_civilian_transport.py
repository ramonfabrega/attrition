#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_think_civilian_transport.py — `Unit::think_civilian_transport@005f40d0` under unicorn (item 3376, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_unit_think_civilian_transport.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 224-byte object of random bytes (no bias), 1 stack argument(s);
`p` windows in order: p0..p72 the place(s), in flag order; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c06188 points at 352 random bytes; the global cell 0x00cae5fc points at 2560 random bytes; the global cell 0x00c061b8 points at 64 random bytes; 4 random bytes at 0x00e3a390; 4 random bytes at 0x00e3a788 (each word in 0..4); 4 random bytes at 0x00e3ad50 (each word in -2..2); 264 random bytes at 0x00e3b46e (each word in 0..1); 268 random bytes at 0x00e3b5ec (each word in -1..1); 4 random bytes at 0x00e4127c; 4 random bytes at 0x00e41674 (each word in 0..4); 4 random bytes at 0x00e41c3c (each word in -2..2); 264 random bytes at 0x00e4235a (each word in 0..1); 268 random bytes at 0x00e424d8 (each word in -1..1); 4 random bytes at 0x00e48168; 4 random bytes at 0x00e48560 (each word in 0..4); 4 random bytes at 0x00e48b28 (each word in -2..2); 264 random bytes at 0x00e49246 (each word in 0..1); 268 random bytes at 0x00e493c4 (each word in -1..1); 4 random bytes at 0x00e4f054; 4 random bytes at 0x00e4f44c (each word in 0..4); 4 random bytes at 0x00e4fa14 (each word in -2..2); 264 random bytes at 0x00e50132 (each word in 0..1); 268 random bytes at 0x00e502b0 (each word in -1..1); callee 0x0046f790 (0 args) stubbed; vtable slot +0x110 (0 args) stubbed; callee 0x005f6010 (1 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x006810f0 (1 args) stubbed; callee 0x00680f90 (1 args) stubbed; callee 0x00681020 (3 args) stubbed; callee 0x006b4db0 (2 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x0070f9e0 (3 args) stubbed; callee 0x00704990 (11 args) stubbed; the word at `0xc0aec0+m[9]*0x1c` points at 4 random bytes less `max(i16(m,0xa),0)*4` (placed); the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `p1+0x18` points at 704 random bytes less `0` (placed); the word at `m+0x18` points at 704 random bytes less `0` (placed); the word at `c3+0x10` points at 17408 random bytes less `0` (placed); the word at `c1+0x134` points at 3400 random bytes less `0` (placed); the word at `c1+0x13c` points at 256 random bytes less `0` (placed); the word at `c1+0x140` points at 256 random bytes less `0` (placed); the word at `c1+0x144` points at 256 random bytes less `0` (placed); the word at `c1+0x148` points at 256 random bytes less `0` (placed); the word at `p4+0x104` points at 16384 random bytes less `0` (placed); the word at `p4+0x18c` points at 16384 random bytes less `0` (placed); the word at `p4+0x214` points at 320 random bytes less `0` (placed); the word at `p4+0x29c` points at 320 random bytes less `0` (placed); the word at `p4+0x324` points at 320 random bytes less `0` (placed); the word at `p4+0x3ac` points at 320 random bytes less `0` (placed); the word at `p4+0x434` points at 320 random bytes less `0` (placed); the word at `p4+0x4bc` points at 320 random bytes less `0` (placed); the word at `p4+0x544` points at 320 random bytes less `0` (placed); the word at `p4+0x5cc` points at 320 random bytes less `0` (placed); the word at `p4+0x654` points at 320 random bytes less `0` (placed); the word at `p4+0x6dc` points at 320 random bytes less `0` (placed); the word at `p4+0x764` points at 320 random bytes less `0` (placed); the word at `p4+0x7ec` points at 320 random bytes less `0` (placed); the word at `p4+0x874` points at 320 random bytes less `0` (placed); the word at `p4+0x8fc` points at 320 random bytes less `0` (placed); the word at `p4+0x984` points at 320 random bytes less `0` (placed); the word at `p4+0xa0c` points at 320 random bytes less `0` (placed); the word at `p4+0xa94` points at 320 random bytes less `0` (placed); the word at `p4+0xb1c` points at 320 random bytes less `0` (placed); the word at `p4+0xba4` points at 320 random bytes less `0` (placed); the word at `p4+0xc2c` points at 320 random bytes less `0` (placed); the word at `p4+0xcb4` points at 320 random bytes less `0` (placed); the word at `p4+0xd3c` points at 320 random bytes less `0` (placed); the word at `p4+0xdc4` points at 320 random bytes less `0` (placed); the word at `p4+0xe4c` points at 320 random bytes less `0` (placed); the word at `p4+0xed4` points at 320 random bytes less `0` (placed); the word at `p4+0xf5c` points at 320 random bytes less `0` (placed); the word at `p4+0xfe4` points at 320 random bytes less `0` (placed); the word at `p4+0x106c` points at 320 random bytes less `0` (placed); the word at `p4+0x10f4` points at 320 random bytes less `0` (placed); the word at `p4+0x117c` points at 320 random bytes less `0` (placed); the word at `p4+0x1204` points at 320 random bytes less `0` (placed); the word at `p4+0x128c` points at 320 random bytes less `0` (placed); the word at `p4+0x1314` points at 320 random bytes less `0` (placed); the word at `p4+0x139c` points at 320 random bytes less `0` (placed); the word at `p4+0x1424` points at 320 random bytes less `0` (placed); the word at `p4+0x14ac` points at 320 random bytes less `0` (placed); the word at `p4+0x1534` points at 320 random bytes less `0` (placed); the word at `p4+0x15bc` points at 320 random bytes less `0` (placed); the word at `p4+0x1644` points at 320 random bytes less `0` (placed); the word at `p4+0x16cc` points at 320 random bytes less `0` (placed); the word at `p4+0x1754` points at 320 random bytes less `0` (placed); the word at `p4+0x17dc` points at 320 random bytes less `0` (placed); the word at `p4+0x1864` points at 320 random bytes less `0` (placed); the word at `p4+0x18ec` points at 320 random bytes less `0` (placed); the word at `p4+0x1974` points at 320 random bytes less `0` (placed); the word at `p4+0x19fc` points at 320 random bytes less `0` (placed); the word at `p4+0x1a84` points at 320 random bytes less `0` (placed); the word at `p4+0x1b0c` points at 320 random bytes less `0` (placed); the word at `p4+0x1b94` points at 320 random bytes less `0` (placed); the word at `p4+0x1c1c` points at 320 random bytes less `0` (placed); the word at `p4+0x1ca4` points at 320 random bytes less `0` (placed); the word at `p4+0x1d2c` points at 320 random bytes less `0` (placed); the word at `p4+0x1db4` points at 320 random bytes less `0` (placed); the word at `p4+0x1e3c` points at 320 random bytes less `0` (placed); the word at `p4+0x1ec4` points at 320 random bytes less `0` (placed); the word at `p4+0x1f4c` points at 320 random bytes less `0` (placed); the word at `p4+0x1fd4` points at 320 random bytes less `0` (placed); the word at `p4+0x205c` points at 320 random bytes less `0` (placed); the word at `p4+0x20e4` points at 320 random bytes less `0` (placed); the word at `p4+0x216c` points at 320 random bytes less `0` (placed); the word at `p4+0x21f4` points at 320 random bytes less `0` (placed); stub 0 answered from [0, 1, 2, 3] (90%); stub 1 answered from [0, 1, 1, 1] (90%); stub 4 answered from [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3] (90%); stub 5 answered from [0, 0, 0, 0, 0, 0, 0, 1] (90%); stub 6 answered from [0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8] (90%); stub 7 answered from [0, 1, 1] (90%); stub 4 answers 130 values in call order, printed after the answers; stub 5 answers 400 values in call order, printed after the answers; stub 6 answers 400 values in call order, printed after the answers; stub 7 answers 64 values in call order, printed after the answers; stub 3 answers -1..100 every row; stub 10 answers 0..40 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee).
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

ENTRY = 0x005f40d0
WIN = 224
OWN = 1
BIAS = []
ARGSET = [(0, [0, 1])]
ARGFROM = []
CELLS = [(12607980, 1376), (12607880, 352), (13297148, 2560), (12607928, 64)]
AT = [(14918544, 4), (14919560, 4), (14921040, 4), (14922862, 264), (14923244, 268), (14946940, 4), (14947956, 4), (14949436, 4), (14951258, 264), (14951640, 268), (14975336, 4), (14976352, 4), (14977832, 4), (14979654, 264), (14980036, 268), (15003732, 4), (15004748, 4), (15006228, 4), (15008050, 264), (15008432, 268)]
ATRANGE = {14919560: (0, 4, 4), 14921040: (-2, 2, 4), 14922862: (0, 1, 4), 14923244: (-1, 1, 4), 14947956: (0, 4, 4), 14949436: (-2, 2, 4), 14951258: (0, 1, 4), 14951640: (-1, 1, 4), 14976352: (0, 4, 4), 14977832: (-2, 2, 4), 14979654: (0, 1, 4), 14980036: (-1, 1, 4), 15004748: (0, 4, 4), 15006228: (-2, 2, 4), 15008050: (0, 1, 4), 15008432: (-1, 1, 4)}
PTRARGS = []
PNULLS = []
STUBS = [('call', 4650896, 0), ('slot', 272, 0), ('call', 6250512, 1), ('call', 7033568, 2), ('call', 6820080, 1), ('call', 6819728, 1), ('call', 6819872, 3), ('call', 7032240, 2), ('call', 7421568, 1), ('call', 7422800, 4), ('call', 7404000, 3), ('call', 7358864, 11)]
PLACES = [(None, '0xc0aec0+m[9]*0x1c', 'max(i16(m,0xa),0)*4', 4), ('p0', '0', '0', 128), ('p1', '0x18', '0', 704), ('m', '0x18', '0', 704), ('c3', '0x10', '0', 17408), ('c1', '0x134', '0', 3400), ('c1', '0x13c', '0', 256), ('c1', '0x140', '0', 256), ('c1', '0x144', '0', 256), ('c1', '0x148', '0', 256), ('p4', '0x104', '0', 16384), ('p4', '0x18c', '0', 16384), ('p4', '0x214', '0', 320), ('p4', '0x29c', '0', 320), ('p4', '0x324', '0', 320), ('p4', '0x3ac', '0', 320), ('p4', '0x434', '0', 320), ('p4', '0x4bc', '0', 320), ('p4', '0x544', '0', 320), ('p4', '0x5cc', '0', 320), ('p4', '0x654', '0', 320), ('p4', '0x6dc', '0', 320), ('p4', '0x764', '0', 320), ('p4', '0x7ec', '0', 320), ('p4', '0x874', '0', 320), ('p4', '0x8fc', '0', 320), ('p4', '0x984', '0', 320), ('p4', '0xa0c', '0', 320), ('p4', '0xa94', '0', 320), ('p4', '0xb1c', '0', 320), ('p4', '0xba4', '0', 320), ('p4', '0xc2c', '0', 320), ('p4', '0xcb4', '0', 320), ('p4', '0xd3c', '0', 320), ('p4', '0xdc4', '0', 320), ('p4', '0xe4c', '0', 320), ('p4', '0xed4', '0', 320), ('p4', '0xf5c', '0', 320), ('p4', '0xfe4', '0', 320), ('p4', '0x106c', '0', 320), ('p4', '0x10f4', '0', 320), ('p4', '0x117c', '0', 320), ('p4', '0x1204', '0', 320), ('p4', '0x128c', '0', 320), ('p4', '0x1314', '0', 320), ('p4', '0x139c', '0', 320), ('p4', '0x1424', '0', 320), ('p4', '0x14ac', '0', 320), ('p4', '0x1534', '0', 320), ('p4', '0x15bc', '0', 320), ('p4', '0x1644', '0', 320), ('p4', '0x16cc', '0', 320), ('p4', '0x1754', '0', 320), ('p4', '0x17dc', '0', 320), ('p4', '0x1864', '0', 320), ('p4', '0x18ec', '0', 320), ('p4', '0x1974', '0', 320), ('p4', '0x19fc', '0', 320), ('p4', '0x1a84', '0', 320), ('p4', '0x1b0c', '0', 320), ('p4', '0x1b94', '0', 320), ('p4', '0x1c1c', '0', 320), ('p4', '0x1ca4', '0', 320), ('p4', '0x1d2c', '0', 320), ('p4', '0x1db4', '0', 320), ('p4', '0x1e3c', '0', 320), ('p4', '0x1ec4', '0', 320), ('p4', '0x1f4c', '0', 320), ('p4', '0x1fd4', '0', 320), ('p4', '0x205c', '0', 320), ('p4', '0x20e4', '0', 320), ('p4', '0x216c', '0', 320), ('p4', '0x21f4', '0', 320)]
NULLS = []
NULLFORCE = {}
WBIAS = [('c1', 0, ('range', 3, 9)), ('c1', 36, ('range', 2, 5)), ('c0', 1360, ('set', [0, 1, 2, 3, 7, 32, 33, 256, 4660, 2147418112])), ('m', 8, ('set', [0, 65536, 131072, 327680, 2162688, 0, 65536, 131072, 327680, 2162688, 0, 65536, 131072, 327680, 2162688, 256, 65792, 131328, 327936, 2162944, 256, 65792, 131328, 327936, 2162944, 512, 66048, 131584, 328192, 2163200, 768, 66304, 131840, 328448, 2163456])), ('m', 16, ('range', 401408, 409599)), ('m', 20, ('range', 401408, 409599)), ('m', 104, ('set', [0, 0, 256, 67108864, 524288])), ('p3', 536, ('set', [0, 0, 0, 0, 0, 1, 2, 1])), ('p3', 692, ('set', [0, 16, 17, 255, 1])), ('p1', 104, ('set', [0, 8388608, 8388609, 8388608, 8388608, 16777216])), ('p1', 108, ('set', [0, 0, 0, 8192, 256, 8193])), ('p2', 692, ('set', [0, 16, 17, 1])), ('p5', None, ('set', [0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 65539, 42, 63, 131076, 16777216, 33554432, 50331648, 16777219])), ('p6', None, ('set', [0, 1, 2, 3, 5, 7, 100, 12345, 65536, 2147483647, -1, 131072])), ('p7', None, ('set', [0, 1, 2, 3, 5, 7, 100, 12345, 65536, 2147483647, -1, 131072])), ('p8', None, ('set', [0, 1, 2, 3, 5, 7, 100, 12345, 65536, 2147483647, -1, 131072])), ('p9', None, ('set', [0, 1, 2, 3, 5, 7, 100, 12345, 65536, 2147483647, -1, 131072])), ('p4', 156, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 16, 17, 33, 40, 512, 832, 1024, 1536, 2048])), ('p4', 200, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p10', None, ('range', 1, 8)), ('p4', 292, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 16, 17, 33, 40, 512, 832, 1024, 1536, 2048])), ('p4', 336, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p11', None, ('range', 1, 8)), ('p4', 428, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 472, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p12', None, ('range', 1, 8)), ('p4', 564, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 608, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p13', None, ('range', 1, 8)), ('p4', 700, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 744, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p14', None, ('range', 1, 8)), ('p4', 836, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 880, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p15', None, ('range', 1, 8)), ('p4', 972, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 1016, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p16', None, ('range', 1, 8)), ('p4', 1108, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 1152, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p17', None, ('range', 1, 8)), ('p4', 1244, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 1288, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p18', None, ('range', 1, 8)), ('p4', 1380, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 1424, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p19', None, ('range', 1, 8)), ('p4', 1516, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 1560, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p20', None, ('range', 1, 8)), ('p4', 1652, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 1696, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p21', None, ('range', 1, 8)), ('p4', 1788, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 1832, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p22', None, ('range', 1, 8)), ('p4', 1924, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 1968, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p23', None, ('range', 1, 8)), ('p4', 2060, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 2104, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p24', None, ('range', 1, 8)), ('p4', 2196, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 2240, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p25', None, ('range', 1, 8)), ('p4', 2332, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 2376, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p26', None, ('range', 1, 8)), ('p4', 2468, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 2512, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p27', None, ('range', 1, 8)), ('p4', 2604, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 2648, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p28', None, ('range', 1, 8)), ('p4', 2740, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 2784, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p29', None, ('range', 1, 8)), ('p4', 2876, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 2920, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p30', None, ('range', 1, 8)), ('p4', 3012, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 3056, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p31', None, ('range', 1, 8)), ('p4', 3148, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 3192, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p32', None, ('range', 1, 8)), ('p4', 3284, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 3328, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p33', None, ('range', 1, 8)), ('p4', 3420, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 3464, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p34', None, ('range', 1, 8)), ('p4', 3556, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 3600, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p35', None, ('range', 1, 8)), ('p4', 3692, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 3736, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p36', None, ('range', 1, 8)), ('p4', 3828, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 3872, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p37', None, ('range', 1, 8)), ('p4', 3964, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 4008, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p38', None, ('range', 1, 8)), ('p4', 4100, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 4144, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p39', None, ('range', 1, 8)), ('p4', 4236, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 4280, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p40', None, ('range', 1, 8)), ('p4', 4372, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 4416, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p41', None, ('range', 1, 8)), ('p4', 4508, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 4552, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p42', None, ('range', 1, 8)), ('p4', 4644, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 4688, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p43', None, ('range', 1, 8)), ('p4', 4780, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 4824, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p44', None, ('range', 1, 8)), ('p4', 4916, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 4960, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p45', None, ('range', 1, 8)), ('p4', 5052, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 5096, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p46', None, ('range', 1, 8)), ('p4', 5188, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 5232, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p47', None, ('range', 1, 8)), ('p4', 5324, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 5368, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p48', None, ('range', 1, 8)), ('p4', 5460, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 5504, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p49', None, ('range', 1, 8)), ('p4', 5596, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 5640, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p50', None, ('range', 1, 8)), ('p4', 5732, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 5776, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p51', None, ('range', 1, 8)), ('p4', 5868, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 5912, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p52', None, ('range', 1, 8)), ('p4', 6004, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 6048, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p53', None, ('range', 1, 8)), ('p4', 6140, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 6184, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p54', None, ('range', 1, 8)), ('p4', 6276, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 6320, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p55', None, ('range', 1, 8)), ('p4', 6412, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 6456, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p56', None, ('range', 1, 8)), ('p4', 6548, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 6592, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p57', None, ('range', 1, 8)), ('p4', 6684, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 6728, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p58', None, ('range', 1, 8)), ('p4', 6820, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 6864, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p59', None, ('range', 1, 8)), ('p4', 6956, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 7000, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p60', None, ('range', 1, 8)), ('p4', 7092, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 7136, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p61', None, ('range', 1, 8)), ('p4', 7228, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 7272, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p62', None, ('range', 1, 8)), ('p4', 7364, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 7408, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p63', None, ('range', 1, 8)), ('p4', 7500, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 7544, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p64', None, ('range', 1, 8)), ('p4', 7636, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 7680, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p65', None, ('range', 1, 8)), ('p4', 7772, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 7816, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p66', None, ('range', 1, 8)), ('p4', 7908, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 7952, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p67', None, ('range', 1, 8)), ('p4', 8044, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 8088, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p68', None, ('range', 1, 8)), ('p4', 8180, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 8224, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p69', None, ('range', 1, 8)), ('p4', 8316, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 8360, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p70', None, ('range', 1, 8)), ('p4', 8452, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 8496, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p71', None, ('range', 1, 8)), ('p4', 8588, ('set', [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 16, 17, 33, 40])), ('p4', 8632, ('set', [0, 0, 0, 1, 2, 4, 8, 3, 12, 15, 255])), ('p72', None, ('range', 1, 8)), ('p4', 8860, ('set', [0, 0, 1, 3, 40])), ('p4', 8996, ('set', [0, 0, 1, 3, 40])), ('p4', 9132, ('set', [0, 0, 1, 3, 40])), ('p4', 9268, ('set', [0, 0, 1, 3, 40])), ('p4', 9404, ('set', [0, 0, 1, 3, 40])), ('p4', 9540, ('set', [0, 0, 1, 3, 40])), ('p4', 9676, ('set', [0, 0, 1, 3, 40])), ('p4', 9812, ('set', [0, 0, 1, 3, 40])), ('p4', 9948, ('set', [0, 0, 1, 3, 40])), ('p4', 10084, ('set', [0, 0, 1, 3, 40])), ('p4', 10220, ('set', [0, 0, 1, 3, 40])), ('p4', 10356, ('set', [0, 0, 1, 3, 40])), ('p4', 10492, ('set', [0, 0, 1, 3, 40])), ('p4', 10628, ('set', [0, 0, 1, 3, 40])), ('p4', 10764, ('set', [0, 0, 1, 3, 40])), ('p4', 10900, ('set', [0, 0, 1, 3, 40])), ('p4', 11036, ('set', [0, 0, 1, 3, 40])), ('p4', 11172, ('set', [0, 0, 1, 3, 40])), ('p4', 11308, ('set', [0, 0, 1, 3, 40])), ('p4', 11444, ('set', [0, 0, 1, 3, 40])), ('p4', 11580, ('set', [0, 0, 1, 3, 40])), ('p4', 11716, ('set', [0, 0, 1, 3, 40])), ('p4', 11852, ('set', [0, 0, 1, 3, 40])), ('p4', 11988, ('set', [0, 0, 1, 3, 40])), ('p4', 12124, ('set', [0, 0, 1, 3, 40])), ('p4', 12260, ('set', [0, 0, 1, 3, 40])), ('p4', 12396, ('set', [0, 0, 1, 3, 40])), ('p4', 12532, ('set', [0, 0, 1, 3, 40])), ('p4', 12668, ('set', [0, 0, 1, 3, 40])), ('p4', 12804, ('set', [0, 0, 1, 3, 40])), ('p4', 12940, ('set', [0, 0, 1, 3, 40])), ('p4', 13076, ('set', [0, 0, 1, 3, 40])), ('p4', 13212, ('set', [0, 0, 1, 3, 40])), ('p4', 13348, ('set', [0, 0, 1, 3, 40])), ('p4', 13484, ('set', [0, 0, 1, 3, 40])), ('p4', 13620, ('set', [0, 0, 1, 3, 40])), ('p4', 13756, ('set', [0, 0, 1, 3, 40])), ('p4', 13892, ('set', [0, 0, 1, 3, 40])), ('p4', 14028, ('set', [0, 0, 1, 3, 40])), ('p4', 14164, ('set', [0, 0, 1, 3, 40])), ('p4', 14300, ('set', [0, 0, 1, 3, 40])), ('p4', 14436, ('set', [0, 0, 1, 3, 40])), ('p4', 14572, ('set', [0, 0, 1, 3, 40])), ('p4', 14708, ('set', [0, 0, 1, 3, 40])), ('p4', 14844, ('set', [0, 0, 1, 3, 40])), ('p4', 14980, ('set', [0, 0, 1, 3, 40])), ('p4', 15116, ('set', [0, 0, 1, 3, 40])), ('p4', 15252, ('set', [0, 0, 1, 3, 40])), ('p4', 15388, ('set', [0, 0, 1, 3, 40])), ('p4', 15524, ('set', [0, 0, 1, 3, 40])), ('p4', 15660, ('set', [0, 0, 1, 3, 40])), ('p4', 15796, ('set', [0, 0, 1, 3, 40])), ('p4', 15932, ('set', [0, 0, 1, 3, 40])), ('p4', 16068, ('set', [0, 0, 1, 3, 40])), ('p4', 16204, ('set', [0, 0, 1, 3, 40])), ('p4', 16340, ('set', [0, 0, 1, 3, 40])), ('p4', 16476, ('set', [0, 0, 1, 3, 40])), ('p4', 16612, ('set', [0, 0, 1, 3, 40])), ('p4', 16748, ('set', [0, 0, 1, 3, 40])), ('p4', 16884, ('set', [0, 0, 1, 3, 40])), ('p4', 17020, ('set', [0, 0, 1, 3, 40])), ('p4', 17156, ('set', [0, 0, 1, 3, 40]))]
FILLS = [('c2', 'i//3')]
ARGRANGE = []
SLOTBASE = []
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [0, 1, 2, 3]), (1, [0, 1, 1, 1]), (4, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3]), (5, [0, 0, 0, 0, 0, 0, 0, 1]), (6, [0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8]), (7, [0, 1, 1])]
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = []
ANSWERBY = []
ANSWERWIN = []
CDECL = False
TIB = False
ARGEQ = []
SEQ = [(4, 130, None), (5, 400, None), (6, 400, None), (7, 64, None)]
ANSRANGE = [(3, -1, 100), (10, 0, 40)]
CELLTHIS = []
VTALSO = []
SPARSE = []
GTAB = []
RECORD = [0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11]
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
STOPSTUB = None
INITCALLS = []
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
    rng = random.Random(3376)
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

# sweep_gen: ["obj", "Unit::think_civilian_transport@005f40d0", "--item", "3376", "--win", "0xe0", "--own", "1", "--argset", "0:0,1", "--name", "unit_think_civilian_transport", "--rows", "2500", "--max-insns", "3000000", "--cell-win", "0xc061ec:0x560", "--cell-win", "0xc06188:0x160", "--cell-win", "0xcae5fc:0xa00", "--cell-win", "0xc061b8:0x40", "--at", "0xe3a390:4", "--at", "0xe3a788:4:0..4", "--at", "0xe3ad50:4:-2..2", "--at", "0xe3b46e:264:0..1", "--at", "0xe3b5ec:268:-1..1", "--at", "0xe4127c:4", "--at", "0xe41674:4:0..4", "--at", "0xe41c3c:4:-2..2", "--at", "0xe4235a:264:0..1", "--at", "0xe424d8:268:-1..1", "--at", "0xe48168:4", "--at", "0xe48560:4:0..4", "--at", "0xe48b28:4:-2..2", "--at", "0xe49246:264:0..1", "--at", "0xe493c4:268:-1..1", "--at", "0xe4f054:4", "--at", "0xe4f44c:4:0..4", "--at", "0xe4fa14:4:-2..2", "--at", "0xe50132:264:0..1", "--at", "0xe502b0:268:-1..1", "--stub", "0x46f790:0", "--stub", "slot:0x110:0", "--stub", "0x5f6010:1", "--stub", "0x6b52e0:2", "--stub", "0x6810f0:1", "--stub", "0x680f90:1", "--stub", "0x681020:3", "--stub", "0x6b4db0:2", "--stub", "0x713e80:1", "--stub", "0x714350:4", "--stub", "0x70f9e0:3", "--stub", "0x704990:11", "--place", "0xc0aec0+m[9]*0x1c:max(i16(m,0xa),0)*4:4", "--place", "p0+0:0:0x80", "--place", "p1+0x18:0:0x2c0", "--place", "m+0x18:0:0x2c0", "--place", "c3+0x10:0:0x4400", "--place", "c1+0x134:0:3400", "--place", "c1+0x13c:0:0x100", "--place", "c1+0x140:0:0x100", "--place", "c1+0x144:0:0x100", "--place", "c1+0x148:0:0x100", "--place", "p4+0x104:0:16384", "--place", "p4+0x18c:0:16384", "--place", "p4+0x214:0:320", "--place", "p4+0x29c:0:320", "--place", "p4+0x324:0:320", "--place", "p4+0x3ac:0:320", "--place", "p4+0x434:0:320", "--place", "p4+0x4bc:0:320", "--place", "p4+0x544:0:320", "--place", "p4+0x5cc:0:320", "--place", "p4+0x654:0:320", "--place", "p4+0x6dc:0:320", "--place", "p4+0x764:0:320", "--place", "p4+0x7ec:0:320", "--place", "p4+0x874:0:320", "--place", "p4+0x8fc:0:320", "--place", "p4+0x984:0:320", "--place", "p4+0xa0c:0:320", "--place", "p4+0xa94:0:320", "--place", "p4+0xb1c:0:320", "--place", "p4+0xba4:0:320", "--place", "p4+0xc2c:0:320", "--place", "p4+0xcb4:0:320", "--place", "p4+0xd3c:0:320", "--place", "p4+0xdc4:0:320", "--place", "p4+0xe4c:0:320", "--place", "p4+0xed4:0:320", "--place", "p4+0xf5c:0:320", "--place", "p4+0xfe4:0:320", "--place", "p4+0x106c:0:320", "--place", "p4+0x10f4:0:320", "--place", "p4+0x117c:0:320", "--place", "p4+0x1204:0:320", "--place", "p4+0x128c:0:320", "--place", "p4+0x1314:0:320", "--place", "p4+0x139c:0:320", "--place", "p4+0x1424:0:320", "--place", "p4+0x14ac:0:320", "--place", "p4+0x1534:0:320", "--place", "p4+0x15bc:0:320", "--place", "p4+0x1644:0:320", "--place", "p4+0x16cc:0:320", "--place", "p4+0x1754:0:320", "--place", "p4+0x17dc:0:320", "--place", "p4+0x1864:0:320", "--place", "p4+0x18ec:0:320", "--place", "p4+0x1974:0:320", "--place", "p4+0x19fc:0:320", "--place", "p4+0x1a84:0:320", "--place", "p4+0x1b0c:0:320", "--place", "p4+0x1b94:0:320", "--place", "p4+0x1c1c:0:320", "--place", "p4+0x1ca4:0:320", "--place", "p4+0x1d2c:0:320", "--place", "p4+0x1db4:0:320", "--place", "p4+0x1e3c:0:320", "--place", "p4+0x1ec4:0:320", "--place", "p4+0x1f4c:0:320", "--place", "p4+0x1fd4:0:320", "--place", "p4+0x205c:0:320", "--place", "p4+0x20e4:0:320", "--place", "p4+0x216c:0:320", "--place", "p4+0x21f4:0:320", "--fill", "c2:i//3", "--wbias", "c1+0:3..9", "--wbias", "c1+0x24:2..5", "--wbias", "c0+0x550:0,1,2,3,7,0x20,0x21,0x100,0x1234,0x7fff0000", "--wbias", "m+8:0x0,0x10000,0x20000,0x50000,0x210000,0x0,0x10000,0x20000,0x50000,0x210000,0x0,0x10000,0x20000,0x50000,0x210000,0x100,0x10100,0x20100,0x50100,0x210100,0x100,0x10100,0x20100,0x50100,0x210100,0x200,0x10200,0x20200,0x50200,0x210200,0x300,0x10300,0x20300,0x50300,0x210300", "--wbias", "m+0x10:0x62000..0x63fff", "--wbias", "m+0x14:0x62000..0x63fff", "--wbias", "m+0x68:0,0,0x100,0x4000000,0x80000", "--wbias", "p3+0x218:0,0,0,0,0,1,2,1", "--wbias", "p3+0x2b4:0,0x10,0x11,0xff,0x1", "--wbias", "p1+0x68:0,0x800000,0x800001,0x800000,0x800000,0x1000000", "--wbias", "p1+0x6c:0,0,0,0x2000,0x100,0x2001", "--wbias", "p2+0x2b4:0,0x10,0x11,0x1", "--wbias", "p5+*:0,0,0,1,2,3,4,5,6,7,8,9,0x10003,0x2a,0x3f,0x20004,0x1000000,0x2000000,0x3000000,0x1000003", "--wbias", "p6+*:0,1,2,3,5,7,100,12345,0x10000,0x7fffffff,-1,0x20000", "--wbias", "p7+*:0,1,2,3,5,7,100,12345,0x10000,0x7fffffff,-1,0x20000", "--wbias", "p8+*:0,1,2,3,5,7,100,12345,0x10000,0x7fffffff,-1,0x20000", "--wbias", "p9+*:0,1,2,3,5,7,100,12345,0x10000,0x7fffffff,-1,0x20000", "--wbias", "p4+0x9c:0,0,0,0,0,0,0,0,0,0,3,16,17,33,40,512,832,1024,1536,2048", "--wbias", "p4+0xc8:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p10+*:1..8", "--wbias", "p4+0x124:0,0,0,0,0,0,0,0,0,0,3,16,17,33,40,512,832,1024,1536,2048", "--wbias", "p4+0x150:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p11+*:1..8", "--wbias", "p4+0x1ac:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1d8:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p12+*:1..8", "--wbias", "p4+0x234:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x260:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p13+*:1..8", "--wbias", "p4+0x2bc:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x2e8:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p14+*:1..8", "--wbias", "p4+0x344:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x370:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p15+*:1..8", "--wbias", "p4+0x3cc:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x3f8:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p16+*:1..8", "--wbias", "p4+0x454:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x480:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p17+*:1..8", "--wbias", "p4+0x4dc:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x508:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p18+*:1..8", "--wbias", "p4+0x564:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x590:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p19+*:1..8", "--wbias", "p4+0x5ec:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x618:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p20+*:1..8", "--wbias", "p4+0x674:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x6a0:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p21+*:1..8", "--wbias", "p4+0x6fc:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x728:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p22+*:1..8", "--wbias", "p4+0x784:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x7b0:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p23+*:1..8", "--wbias", "p4+0x80c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x838:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p24+*:1..8", "--wbias", "p4+0x894:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x8c0:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p25+*:1..8", "--wbias", "p4+0x91c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x948:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p26+*:1..8", "--wbias", "p4+0x9a4:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x9d0:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p27+*:1..8", "--wbias", "p4+0xa2c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0xa58:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p28+*:1..8", "--wbias", "p4+0xab4:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0xae0:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p29+*:1..8", "--wbias", "p4+0xb3c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0xb68:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p30+*:1..8", "--wbias", "p4+0xbc4:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0xbf0:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p31+*:1..8", "--wbias", "p4+0xc4c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0xc78:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p32+*:1..8", "--wbias", "p4+0xcd4:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0xd00:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p33+*:1..8", "--wbias", "p4+0xd5c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0xd88:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p34+*:1..8", "--wbias", "p4+0xde4:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0xe10:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p35+*:1..8", "--wbias", "p4+0xe6c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0xe98:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p36+*:1..8", "--wbias", "p4+0xef4:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0xf20:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p37+*:1..8", "--wbias", "p4+0xf7c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0xfa8:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p38+*:1..8", "--wbias", "p4+0x1004:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1030:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p39+*:1..8", "--wbias", "p4+0x108c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x10b8:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p40+*:1..8", "--wbias", "p4+0x1114:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1140:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p41+*:1..8", "--wbias", "p4+0x119c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x11c8:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p42+*:1..8", "--wbias", "p4+0x1224:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1250:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p43+*:1..8", "--wbias", "p4+0x12ac:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x12d8:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p44+*:1..8", "--wbias", "p4+0x1334:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1360:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p45+*:1..8", "--wbias", "p4+0x13bc:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x13e8:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p46+*:1..8", "--wbias", "p4+0x1444:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1470:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p47+*:1..8", "--wbias", "p4+0x14cc:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x14f8:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p48+*:1..8", "--wbias", "p4+0x1554:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1580:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p49+*:1..8", "--wbias", "p4+0x15dc:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1608:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p50+*:1..8", "--wbias", "p4+0x1664:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1690:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p51+*:1..8", "--wbias", "p4+0x16ec:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1718:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p52+*:1..8", "--wbias", "p4+0x1774:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x17a0:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p53+*:1..8", "--wbias", "p4+0x17fc:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1828:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p54+*:1..8", "--wbias", "p4+0x1884:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x18b0:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p55+*:1..8", "--wbias", "p4+0x190c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1938:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p56+*:1..8", "--wbias", "p4+0x1994:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x19c0:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p57+*:1..8", "--wbias", "p4+0x1a1c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1a48:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p58+*:1..8", "--wbias", "p4+0x1aa4:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1ad0:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p59+*:1..8", "--wbias", "p4+0x1b2c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1b58:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p60+*:1..8", "--wbias", "p4+0x1bb4:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1be0:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p61+*:1..8", "--wbias", "p4+0x1c3c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1c68:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p62+*:1..8", "--wbias", "p4+0x1cc4:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1cf0:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p63+*:1..8", "--wbias", "p4+0x1d4c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1d78:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p64+*:1..8", "--wbias", "p4+0x1dd4:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1e00:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p65+*:1..8", "--wbias", "p4+0x1e5c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1e88:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p66+*:1..8", "--wbias", "p4+0x1ee4:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1f10:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p67+*:1..8", "--wbias", "p4+0x1f6c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x1f98:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p68+*:1..8", "--wbias", "p4+0x1ff4:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x2020:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p69+*:1..8", "--wbias", "p4+0x207c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x20a8:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p70+*:1..8", "--wbias", "p4+0x2104:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x2130:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p71+*:1..8", "--wbias", "p4+0x218c:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,16,17,33,40", "--wbias", "p4+0x21b8:0,0,0,1,2,4,8,3,0xc,0xf,0xff", "--wbias", "p72+*:1..8", "--wbias", "p4+0x229c:0,0,1,3,40", "--wbias", "p4+0x2324:0,0,1,3,40", "--wbias", "p4+0x23ac:0,0,1,3,40", "--wbias", "p4+0x2434:0,0,1,3,40", "--wbias", "p4+0x24bc:0,0,1,3,40", "--wbias", "p4+0x2544:0,0,1,3,40", "--wbias", "p4+0x25cc:0,0,1,3,40", "--wbias", "p4+0x2654:0,0,1,3,40", "--wbias", "p4+0x26dc:0,0,1,3,40", "--wbias", "p4+0x2764:0,0,1,3,40", "--wbias", "p4+0x27ec:0,0,1,3,40", "--wbias", "p4+0x2874:0,0,1,3,40", "--wbias", "p4+0x28fc:0,0,1,3,40", "--wbias", "p4+0x2984:0,0,1,3,40", "--wbias", "p4+0x2a0c:0,0,1,3,40", "--wbias", "p4+0x2a94:0,0,1,3,40", "--wbias", "p4+0x2b1c:0,0,1,3,40", "--wbias", "p4+0x2ba4:0,0,1,3,40", "--wbias", "p4+0x2c2c:0,0,1,3,40", "--wbias", "p4+0x2cb4:0,0,1,3,40", "--wbias", "p4+0x2d3c:0,0,1,3,40", "--wbias", "p4+0x2dc4:0,0,1,3,40", "--wbias", "p4+0x2e4c:0,0,1,3,40", "--wbias", "p4+0x2ed4:0,0,1,3,40", "--wbias", "p4+0x2f5c:0,0,1,3,40", "--wbias", "p4+0x2fe4:0,0,1,3,40", "--wbias", "p4+0x306c:0,0,1,3,40", "--wbias", "p4+0x30f4:0,0,1,3,40", "--wbias", "p4+0x317c:0,0,1,3,40", "--wbias", "p4+0x3204:0,0,1,3,40", "--wbias", "p4+0x328c:0,0,1,3,40", "--wbias", "p4+0x3314:0,0,1,3,40", "--wbias", "p4+0x339c:0,0,1,3,40", "--wbias", "p4+0x3424:0,0,1,3,40", "--wbias", "p4+0x34ac:0,0,1,3,40", "--wbias", "p4+0x3534:0,0,1,3,40", "--wbias", "p4+0x35bc:0,0,1,3,40", "--wbias", "p4+0x3644:0,0,1,3,40", "--wbias", "p4+0x36cc:0,0,1,3,40", "--wbias", "p4+0x3754:0,0,1,3,40", "--wbias", "p4+0x37dc:0,0,1,3,40", "--wbias", "p4+0x3864:0,0,1,3,40", "--wbias", "p4+0x38ec:0,0,1,3,40", "--wbias", "p4+0x3974:0,0,1,3,40", "--wbias", "p4+0x39fc:0,0,1,3,40", "--wbias", "p4+0x3a84:0,0,1,3,40", "--wbias", "p4+0x3b0c:0,0,1,3,40", "--wbias", "p4+0x3b94:0,0,1,3,40", "--wbias", "p4+0x3c1c:0,0,1,3,40", "--wbias", "p4+0x3ca4:0,0,1,3,40", "--wbias", "p4+0x3d2c:0,0,1,3,40", "--wbias", "p4+0x3db4:0,0,1,3,40", "--wbias", "p4+0x3e3c:0,0,1,3,40", "--wbias", "p4+0x3ec4:0,0,1,3,40", "--wbias", "p4+0x3f4c:0,0,1,3,40", "--wbias", "p4+0x3fd4:0,0,1,3,40", "--wbias", "p4+0x405c:0,0,1,3,40", "--wbias", "p4+0x40e4:0,0,1,3,40", "--wbias", "p4+0x416c:0,0,1,3,40", "--wbias", "p4+0x41f4:0,0,1,3,40", "--wbias", "p4+0x427c:0,0,1,3,40", "--wbias", "p4+0x4304:0,0,1,3,40", "--answers", "0:0,1,2,3", "--answers", "1:0,1,1,1", "--answer-range", "3:-1:100", "--answers", "4:0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,2,3", "--answers", "5:0,0,0,0,0,0,0,1", "--answers", "6:0,0,0,0,0,0,0,0,1,2,3,4,5,6,7,8", "--answers", "7:0,1,1", "--answer-range", "10:0:40", "--seq", "4:130", "--seq", "5:400", "--seq", "6:400", "--seq", "7:64", "--record", "0", "--record", "1", "--record", "2", "--record", "4", "--record", "5", "--record", "6", "--record", "7", "--record", "8", "--record", "9", "--record", "10", "--record", "11", "--record-order", "--port", "let mut v = RowView {\n    a: &a,\n    m: &mut m[..],\n    c: &mut c[..],\n    p: &mut p[..],\n    log: &mut log,\n    goes: 0,\n    coasts: 0,\n    hits: 0,\n    waters: 0,\n};\ni64::from(think_civilian_transport_steps(&mut v, a[0] != 0))"]
