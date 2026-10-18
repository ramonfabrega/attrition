#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_data_construct_time.py — `BuildData::construct_time@0062d5c0` under unicorn (item 3261, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_build_data_construct_time.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 1024-byte object of random bytes (no bias), 1 stack argument(s);
`p` windows in order: p0..p31 sparse table 0 (8 rows × 4 slice(s), row-major), p32..p33 sparse table 1 (2 rows × 1 slice(s), row-major), p34..p35 sparse table 2 (2 rows × 1 slice(s), row-major), p36..p37 sparse table 3 (2 rows × 1 slice(s), row-major), p38..p39 sparse table 4 (2 rows × 1 slice(s), row-major), p40..p41 sparse table 5 (2 rows × 1 slice(s), row-major), p42..p43 sparse table 6 (2 rows × 1 slice(s), row-major), p44..p45 sparse table 7 (2 rows × 1 slice(s), row-major), p46..p47 sparse table 8 (2 rows × 1 slice(s), row-major), p48 sparse table 9 (1 rows × 1 slice(s), row-major), p49 sparse table 10 (1 rows × 1 slice(s), row-major), p50 sparse table 11 (1 rows × 1 slice(s), row-major), p51 sparse table 12 (1 rows × 1 slice(s), row-major), p52 sparse table 13 (1 rows × 1 slice(s), row-major), p53 sparse table 14 (1 rows × 1 slice(s), row-major), p54 sparse table 15 (1 rows × 1 slice(s), row-major), p55 sparse table 16 (1 rows × 1 slice(s), row-major), p56..p184 the place(s), in flag order, p185..p188 the stubs' answer window(s); the global cell 0x00c061e4 points at 3224 random bytes; 32 random bytes at 0x00c0acf4 (each word in 2000..2002); vtable slot +0x2c (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x3c (0 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x006db680 (0 args) stubbed; callee 0x006ebc10 (1 args) stubbed; callee 0x00646b00 (2 args) stubbed; the word at `m+0x18` points at 16 random bytes less `0` (placed); the word at `p32+0` points at 32 random bytes less `0` (placed); the word at `p57+0x18` points at 16 random bytes less `0` (placed); the word at `p32+4` points at 32 random bytes less `0` (placed); the word at `p59+0x18` points at 16 random bytes less `0` (placed); the word at `p32+8` points at 32 random bytes less `0` (placed); the word at `p61+0x18` points at 16 random bytes less `0` (placed); the word at `p32+12` points at 32 random bytes less `0` (placed); the word at `p63+0x18` points at 16 random bytes less `0` (placed); the word at `p34+0` points at 32 random bytes less `0` (placed); the word at `p65+0x18` points at 16 random bytes less `0` (placed); the word at `p34+4` points at 32 random bytes less `0` (placed); the word at `p67+0x18` points at 16 random bytes less `0` (placed); the word at `p34+8` points at 32 random bytes less `0` (placed); the word at `p69+0x18` points at 16 random bytes less `0` (placed); the word at `p34+12` points at 32 random bytes less `0` (placed); the word at `p71+0x18` points at 16 random bytes less `0` (placed); the word at `p36+0` points at 32 random bytes less `0` (placed); the word at `p73+0x18` points at 16 random bytes less `0` (placed); the word at `p36+4` points at 32 random bytes less `0` (placed); the word at `p75+0x18` points at 16 random bytes less `0` (placed); the word at `p36+8` points at 32 random bytes less `0` (placed); the word at `p77+0x18` points at 16 random bytes less `0` (placed); the word at `p36+12` points at 32 random bytes less `0` (placed); the word at `p79+0x18` points at 16 random bytes less `0` (placed); the word at `p38+0` points at 32 random bytes less `0` (placed); the word at `p81+0x18` points at 16 random bytes less `0` (placed); the word at `p38+4` points at 32 random bytes less `0` (placed); the word at `p83+0x18` points at 16 random bytes less `0` (placed); the word at `p38+8` points at 32 random bytes less `0` (placed); the word at `p85+0x18` points at 16 random bytes less `0` (placed); the word at `p38+12` points at 32 random bytes less `0` (placed); the word at `p87+0x18` points at 16 random bytes less `0` (placed); the word at `p40+0` points at 32 random bytes less `0` (placed); the word at `p89+0x18` points at 16 random bytes less `0` (placed); the word at `p40+4` points at 32 random bytes less `0` (placed); the word at `p91+0x18` points at 16 random bytes less `0` (placed); the word at `p40+8` points at 32 random bytes less `0` (placed); the word at `p93+0x18` points at 16 random bytes less `0` (placed); the word at `p40+12` points at 32 random bytes less `0` (placed); the word at `p95+0x18` points at 16 random bytes less `0` (placed); the word at `p42+0` points at 32 random bytes less `0` (placed); the word at `p97+0x18` points at 16 random bytes less `0` (placed); the word at `p42+4` points at 32 random bytes less `0` (placed); the word at `p99+0x18` points at 16 random bytes less `0` (placed); the word at `p42+8` points at 32 random bytes less `0` (placed); the word at `p101+0x18` points at 16 random bytes less `0` (placed); the word at `p42+12` points at 32 random bytes less `0` (placed); the word at `p103+0x18` points at 16 random bytes less `0` (placed); the word at `p44+0` points at 32 random bytes less `0` (placed); the word at `p105+0x18` points at 16 random bytes less `0` (placed); the word at `p44+4` points at 32 random bytes less `0` (placed); the word at `p107+0x18` points at 16 random bytes less `0` (placed); the word at `p44+8` points at 32 random bytes less `0` (placed); the word at `p109+0x18` points at 16 random bytes less `0` (placed); the word at `p44+12` points at 32 random bytes less `0` (placed); the word at `p111+0x18` points at 16 random bytes less `0` (placed); the word at `p46+0` points at 32 random bytes less `0` (placed); the word at `p113+0x18` points at 16 random bytes less `0` (placed); the word at `p46+4` points at 32 random bytes less `0` (placed); the word at `p115+0x18` points at 16 random bytes less `0` (placed); the word at `p46+8` points at 32 random bytes less `0` (placed); the word at `p117+0x18` points at 16 random bytes less `0` (placed); the word at `p46+12` points at 32 random bytes less `0` (placed); the word at `p119+0x18` points at 16 random bytes less `0` (placed); the word at `p33+0` points at 32 random bytes less `0` (placed); the word at `p121+0x18` points at 16 random bytes less `0` (placed); the word at `p33+4` points at 32 random bytes less `0` (placed); the word at `p123+0x18` points at 16 random bytes less `0` (placed); the word at `p35+0` points at 32 random bytes less `0` (placed); the word at `p125+0x18` points at 16 random bytes less `0` (placed); the word at `p35+4` points at 32 random bytes less `0` (placed); the word at `p127+0x18` points at 16 random bytes less `0` (placed); the word at `p37+0` points at 32 random bytes less `0` (placed); the word at `p129+0x18` points at 16 random bytes less `0` (placed); the word at `p37+4` points at 32 random bytes less `0` (placed); the word at `p131+0x18` points at 16 random bytes less `0` (placed); the word at `p39+0` points at 32 random bytes less `0` (placed); the word at `p133+0x18` points at 16 random bytes less `0` (placed); the word at `p39+4` points at 32 random bytes less `0` (placed); the word at `p135+0x18` points at 16 random bytes less `0` (placed); the word at `p41+0` points at 32 random bytes less `0` (placed); the word at `p137+0x18` points at 16 random bytes less `0` (placed); the word at `p41+4` points at 32 random bytes less `0` (placed); the word at `p139+0x18` points at 16 random bytes less `0` (placed); the word at `p43+0` points at 32 random bytes less `0` (placed); the word at `p141+0x18` points at 16 random bytes less `0` (placed); the word at `p43+4` points at 32 random bytes less `0` (placed); the word at `p143+0x18` points at 16 random bytes less `0` (placed); the word at `p45+0` points at 32 random bytes less `0` (placed); the word at `p145+0x18` points at 16 random bytes less `0` (placed); the word at `p45+4` points at 32 random bytes less `0` (placed); the word at `p147+0x18` points at 16 random bytes less `0` (placed); the word at `p47+0` points at 32 random bytes less `0` (placed); the word at `p149+0x18` points at 16 random bytes less `0` (placed); the word at `p47+4` points at 32 random bytes less `0` (placed); the word at `p151+0x18` points at 16 random bytes less `0` (placed); the word at `p48+0` points at 16 random bytes less `0` (placed); the word at `p48+4` points at 16 random bytes less `0` (placed); the word at `p48+8` points at 16 random bytes less `0` (placed); the word at `p48+12` points at 16 random bytes less `0` (placed); the word at `p49+0` points at 16 random bytes less `0` (placed); the word at `p49+4` points at 16 random bytes less `0` (placed); the word at `p49+8` points at 16 random bytes less `0` (placed); the word at `p49+12` points at 16 random bytes less `0` (placed); the word at `p50+0` points at 16 random bytes less `0` (placed); the word at `p50+4` points at 16 random bytes less `0` (placed); the word at `p50+8` points at 16 random bytes less `0` (placed); the word at `p50+12` points at 16 random bytes less `0` (placed); the word at `p51+0` points at 16 random bytes less `0` (placed); the word at `p51+4` points at 16 random bytes less `0` (placed); the word at `p51+8` points at 16 random bytes less `0` (placed); the word at `p51+12` points at 16 random bytes less `0` (placed); the word at `p52+0` points at 16 random bytes less `0` (placed); the word at `p52+4` points at 16 random bytes less `0` (placed); the word at `p52+8` points at 16 random bytes less `0` (placed); the word at `p52+12` points at 16 random bytes less `0` (placed); the word at `p53+0` points at 16 random bytes less `0` (placed); the word at `p53+4` points at 16 random bytes less `0` (placed); the word at `p53+8` points at 16 random bytes less `0` (placed); the word at `p53+12` points at 16 random bytes less `0` (placed); the word at `p54+0` points at 16 random bytes less `0` (placed); the word at `p54+4` points at 16 random bytes less `0` (placed); the word at `p54+8` points at 16 random bytes less `0` (placed); the word at `p54+12` points at 16 random bytes less `0` (placed); the word at `p55+0` points at 16 random bytes less `0` (placed); the word at `p55+4` points at 16 random bytes less `0` (placed); the word at `p55+8` points at 16 random bytes less `0` (placed); the word at `p55+12` points at 16 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0x2c laid as 0x00472320 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 0, 1] (90%); stub 6 answered from [0, 1, 1, 1] (90%); stub 7 answered from [0, 0, 0, 1, 2] (90%); stub 8 answered from [0, 1, 1, 1, 2, 3] (90%); stub 9 answered from [-1, -1, 0, 7, 3] (90%); stub 5 answers a window of its own to each call, in call order (128 bytes, ...); the word at `p56+0` laid as the image vtable at 0xb42b94 (the stub slots copied there) half the rows; stub 4 answers 14 values in call order, printed after the answers; stub 6 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4, +0x424:0x4, +0x854:0x4, +0x5720:0x48 (`p` windows after the pointed buffers, before the places); 2 rows 0x1f40 apart at `0xc0ab84`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 2 rows 0x1f40 apart at `0xc0aba0`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 2 rows 0x1f40 apart at `0xc0abbc`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 2 rows 0x1f40 apart at `0xc0abd8`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 2 rows 0x1f40 apart at `0xc0abf4`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 2 rows 0x1f40 apart at `0xc0ac10`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 2 rows 0x1f40 apart at `0xc0ac2c`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 2 rows 0x1f40 apart at `0xc0ac48`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc0a390`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc0a3ac`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc0a3c8`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc0a3e4`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc0a400`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc0a41c`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc0a438`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc0a454`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p57+0` as well as at `this`; the stubbed vtable laid at `p59+0` as well as at `this`; the stubbed vtable laid at `p61+0` as well as at `this`; the stubbed vtable laid at `p63+0` as well as at `this`; the stubbed vtable laid at `p65+0` as well as at `this`; the stubbed vtable laid at `p67+0` as well as at `this`; the stubbed vtable laid at `p69+0` as well as at `this`; the stubbed vtable laid at `p71+0` as well as at `this`; the stubbed vtable laid at `p73+0` as well as at `this`; the stubbed vtable laid at `p75+0` as well as at `this`; the stubbed vtable laid at `p77+0` as well as at `this`; the stubbed vtable laid at `p79+0` as well as at `this`; the stubbed vtable laid at `p81+0` as well as at `this`; the stubbed vtable laid at `p83+0` as well as at `this`; the stubbed vtable laid at `p85+0` as well as at `this`; the stubbed vtable laid at `p87+0` as well as at `this`; the stubbed vtable laid at `p89+0` as well as at `this`; the stubbed vtable laid at `p91+0` as well as at `this`; the stubbed vtable laid at `p93+0` as well as at `this`; the stubbed vtable laid at `p95+0` as well as at `this`; the stubbed vtable laid at `p97+0` as well as at `this`; the stubbed vtable laid at `p99+0` as well as at `this`; the stubbed vtable laid at `p101+0` as well as at `this`; the stubbed vtable laid at `p103+0` as well as at `this`; the stubbed vtable laid at `p105+0` as well as at `this`; the stubbed vtable laid at `p107+0` as well as at `this`; the stubbed vtable laid at `p109+0` as well as at `this`; the stubbed vtable laid at `p111+0` as well as at `this`; the stubbed vtable laid at `p113+0` as well as at `this`; the stubbed vtable laid at `p115+0` as well as at `this`; the stubbed vtable laid at `p117+0` as well as at `this`; the stubbed vtable laid at `p119+0` as well as at `this`; the stubbed vtable laid at `p121+0` as well as at `this`; the stubbed vtable laid at `p123+0` as well as at `this`; the stubbed vtable laid at `p125+0` as well as at `this`; the stubbed vtable laid at `p127+0` as well as at `this`; the stubbed vtable laid at `p129+0` as well as at `this`; the stubbed vtable laid at `p131+0` as well as at `this`; the stubbed vtable laid at `p133+0` as well as at `this`; the stubbed vtable laid at `p135+0` as well as at `this`; the stubbed vtable laid at `p137+0` as well as at `this`; the stubbed vtable laid at `p139+0` as well as at `this`; the stubbed vtable laid at `p141+0` as well as at `this`; the stubbed vtable laid at `p143+0` as well as at `this`; the stubbed vtable laid at `p145+0` as well as at `this`; the stubbed vtable laid at `p147+0` as well as at `this`; the stubbed vtable laid at `p149+0` as well as at `this`; the stubbed vtable laid at `p151+0` as well as at `this`.
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

ENTRY = 0x0062d5c0
WIN = 1024
OWN = 1
BIAS = []
ARGSET = [(0, [0, 0, 0, 0, 0, 1, -1])]
ARGFROM = []
CELLS = [(12607972, 3224)]
AT = [(12627188, 32)]
ATRANGE = {12627188: (2000, 2002, 4)}
PTRARGS = []
PNULLS = []
STUBS = [('slot', 44, 0), ('slot', 184, 2), ('slot', 28, 0), ('slot', 96, 2), ('slot', 76, 0), ('slot', 60, 0), ('call', 7213936, 1), ('call', 7190144, 0), ('call', 7257104, 1), ('call', 6580992, 2)]
PLACES = [('m', '0x18', '0', 16), ('p32', '0', '0', 32), ('p57', '0x18', '0', 16), ('p32', '4', '0', 32), ('p59', '0x18', '0', 16), ('p32', '8', '0', 32), ('p61', '0x18', '0', 16), ('p32', '12', '0', 32), ('p63', '0x18', '0', 16), ('p34', '0', '0', 32), ('p65', '0x18', '0', 16), ('p34', '4', '0', 32), ('p67', '0x18', '0', 16), ('p34', '8', '0', 32), ('p69', '0x18', '0', 16), ('p34', '12', '0', 32), ('p71', '0x18', '0', 16), ('p36', '0', '0', 32), ('p73', '0x18', '0', 16), ('p36', '4', '0', 32), ('p75', '0x18', '0', 16), ('p36', '8', '0', 32), ('p77', '0x18', '0', 16), ('p36', '12', '0', 32), ('p79', '0x18', '0', 16), ('p38', '0', '0', 32), ('p81', '0x18', '0', 16), ('p38', '4', '0', 32), ('p83', '0x18', '0', 16), ('p38', '8', '0', 32), ('p85', '0x18', '0', 16), ('p38', '12', '0', 32), ('p87', '0x18', '0', 16), ('p40', '0', '0', 32), ('p89', '0x18', '0', 16), ('p40', '4', '0', 32), ('p91', '0x18', '0', 16), ('p40', '8', '0', 32), ('p93', '0x18', '0', 16), ('p40', '12', '0', 32), ('p95', '0x18', '0', 16), ('p42', '0', '0', 32), ('p97', '0x18', '0', 16), ('p42', '4', '0', 32), ('p99', '0x18', '0', 16), ('p42', '8', '0', 32), ('p101', '0x18', '0', 16), ('p42', '12', '0', 32), ('p103', '0x18', '0', 16), ('p44', '0', '0', 32), ('p105', '0x18', '0', 16), ('p44', '4', '0', 32), ('p107', '0x18', '0', 16), ('p44', '8', '0', 32), ('p109', '0x18', '0', 16), ('p44', '12', '0', 32), ('p111', '0x18', '0', 16), ('p46', '0', '0', 32), ('p113', '0x18', '0', 16), ('p46', '4', '0', 32), ('p115', '0x18', '0', 16), ('p46', '8', '0', 32), ('p117', '0x18', '0', 16), ('p46', '12', '0', 32), ('p119', '0x18', '0', 16), ('p33', '0', '0', 32), ('p121', '0x18', '0', 16), ('p33', '4', '0', 32), ('p123', '0x18', '0', 16), ('p35', '0', '0', 32), ('p125', '0x18', '0', 16), ('p35', '4', '0', 32), ('p127', '0x18', '0', 16), ('p37', '0', '0', 32), ('p129', '0x18', '0', 16), ('p37', '4', '0', 32), ('p131', '0x18', '0', 16), ('p39', '0', '0', 32), ('p133', '0x18', '0', 16), ('p39', '4', '0', 32), ('p135', '0x18', '0', 16), ('p41', '0', '0', 32), ('p137', '0x18', '0', 16), ('p41', '4', '0', 32), ('p139', '0x18', '0', 16), ('p43', '0', '0', 32), ('p141', '0x18', '0', 16), ('p43', '4', '0', 32), ('p143', '0x18', '0', 16), ('p45', '0', '0', 32), ('p145', '0x18', '0', 16), ('p45', '4', '0', 32), ('p147', '0x18', '0', 16), ('p47', '0', '0', 32), ('p149', '0x18', '0', 16), ('p47', '4', '0', 32), ('p151', '0x18', '0', 16), ('p48', '0', '0', 16), ('p48', '4', '0', 16), ('p48', '8', '0', 16), ('p48', '12', '0', 16), ('p49', '0', '0', 16), ('p49', '4', '0', 16), ('p49', '8', '0', 16), ('p49', '12', '0', 16), ('p50', '0', '0', 16), ('p50', '4', '0', 16), ('p50', '8', '0', 16), ('p50', '12', '0', 16), ('p51', '0', '0', 16), ('p51', '4', '0', 16), ('p51', '8', '0', 16), ('p51', '12', '0', 16), ('p52', '0', '0', 16), ('p52', '4', '0', 16), ('p52', '8', '0', 16), ('p52', '12', '0', 16), ('p53', '0', '0', 16), ('p53', '4', '0', 16), ('p53', '8', '0', 16), ('p53', '12', '0', 16), ('p54', '0', '0', 16), ('p54', '4', '0', 16), ('p54', '8', '0', 16), ('p54', '12', '0', 16), ('p55', '0', '0', 16), ('p55', '4', '0', 16), ('p55', '8', '0', 16), ('p55', '12', '0', 16)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 8, ('set', [131072000, 131137536, 0, 327680, 131072256, 131137792, 256, 327936, 131072512, 131138048, 512, 328192, 131072768, 131138304, 768, 328448, 131073024, 131138560, 1024, 328704, 131073280, 131138816, 1280, 328960, 131073536, 131139072, 1536, 329216, 131073792, 131139328, 1792, 329472])), ('m', 112, ('set', [-65536, 0, 65536, 131072, 196608])), ('m', 80, ('set', [0, 1, 2, 3, 99, 100, 101, 1000, 12345, 100000, 5000000, 20000000])), ('p56', 4, ('set', [528, 528, 528, 526, 541, 542, 510, 543, 525])), ('p58', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p60', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p62', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p64', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p66', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p68', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p70', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p72', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p74', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p76', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p78', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p80', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p82', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p84', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p86', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p88', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p90', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p92', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p94', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p96', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p98', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p100', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p102', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p104', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p106', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p108', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p110', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p112', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p114', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p116', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p118', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p120', 4, ('set', [528, 528, 528, 528, 528, 526, 529, 541, 5])), ('p122', 4, ('set', [528, 528, 528, 526, 541, 542, 510, 543, 525])), ('p124', 4, ('set', [528, 528, 528, 526, 541, 542, 510, 543, 525])), ('p126', 4, ('set', [528, 528, 528, 526, 541, 542, 510, 543, 525])), ('p128', 4, ('set', [528, 528, 528, 526, 541, 542, 510, 543, 525])), ('p130', 4, ('set', [528, 528, 528, 526, 541, 542, 510, 543, 525])), ('p132', 4, ('set', [528, 528, 528, 526, 541, 542, 510, 543, 525])), ('p134', 4, ('set', [528, 528, 528, 526, 541, 542, 510, 543, 525])), ('p136', 4, ('set', [528, 528, 528, 526, 541, 542, 510, 543, 525])), ('p138', 4, ('set', [528, 528, 528, 526, 541, 542, 510, 543, 525])), ('p140', 4, ('set', [528, 528, 528, 526, 541, 542, 510, 543, 525])), ('p142', 4, ('set', [528, 528, 528, 526, 541, 542, 510, 543, 525])), ('p144', 4, ('set', [528, 528, 528, 526, 541, 542, 510, 543, 525])), ('p146', 4, ('set', [528, 528, 528, 526, 541, 542, 510, 543, 525])), ('p148', 4, ('set', [528, 528, 528, 526, 541, 542, 510, 543, 525])), ('p150', 4, ('set', [528, 528, 528, 526, 541, 542, 510, 543, 525])), ('p152', 4, ('set', [528, 528, 528, 526, 541, 542, 510, 543, 525])), ('p153', 0, ('set', [0, 65536, 131072, 196608])), ('p154', 0, ('set', [0, 65536, 131072, 196608])), ('p155', 0, ('set', [0, 65536, 131072, 196608])), ('p156', 0, ('set', [0, 65536, 131072, 196608])), ('p157', 0, ('set', [0, 65536, 131072, 196608])), ('p158', 0, ('set', [0, 65536, 131072, 196608])), ('p159', 0, ('set', [0, 65536, 131072, 196608])), ('p160', 0, ('set', [0, 65536, 131072, 196608])), ('p161', 0, ('set', [0, 65536, 131072, 196608])), ('p162', 0, ('set', [0, 65536, 131072, 196608])), ('p163', 0, ('set', [0, 65536, 131072, 196608])), ('p164', 0, ('set', [0, 65536, 131072, 196608])), ('p165', 0, ('set', [0, 65536, 131072, 196608])), ('p166', 0, ('set', [0, 65536, 131072, 196608])), ('p167', 0, ('set', [0, 65536, 131072, 196608])), ('p168', 0, ('set', [0, 65536, 131072, 196608])), ('p169', 0, ('set', [0, 65536, 131072, 196608])), ('p170', 0, ('set', [0, 65536, 131072, 196608])), ('p171', 0, ('set', [0, 65536, 131072, 196608])), ('p172', 0, ('set', [0, 65536, 131072, 196608])), ('p173', 0, ('set', [0, 65536, 131072, 196608])), ('p174', 0, ('set', [0, 65536, 131072, 196608])), ('p175', 0, ('set', [0, 65536, 131072, 196608])), ('p176', 0, ('set', [0, 65536, 131072, 196608])), ('p177', 0, ('set', [0, 65536, 131072, 196608])), ('p178', 0, ('set', [0, 65536, 131072, 196608])), ('p179', 0, ('set', [0, 65536, 131072, 196608])), ('p180', 0, ('set', [0, 65536, 131072, 196608])), ('p181', 0, ('set', [0, 65536, 131072, 196608])), ('p182', 0, ('set', [0, 65536, 131072, 196608])), ('p183', 0, ('set', [0, 65536, 131072, 196608])), ('p184', 0, ('set', [0, 65536, 131072, 196608])), ('p185', 112, ('set', [-65536, 0, 65536, 131072, 196608])), ('p186', 112, ('set', [-65536, 0, 65536, 131072, 196608])), ('p187', 112, ('set', [-65536, 0, 65536, 131072, 196608])), ('p188', 112, ('set', [-65536, 0, 65536, 131072, 196608])), ('p1', 0, ('range', 0, 4)), ('p2', 0, ('set', [0, 0, 0, 1, 5])), ('p3', None, ('range', 0, 1)), ('p5', 0, ('range', 0, 4)), ('p6', 0, ('set', [0, 0, 0, 1, 5])), ('p7', None, ('range', 0, 1)), ('p9', 0, ('range', 0, 4)), ('p10', 0, ('set', [0, 0, 0, 1, 5])), ('p11', None, ('range', 0, 1)), ('p13', 0, ('range', 0, 4)), ('p14', 0, ('set', [0, 0, 0, 1, 5])), ('p15', None, ('range', 0, 1)), ('p17', 0, ('range', 0, 4)), ('p18', 0, ('set', [0, 0, 0, 1, 5])), ('p19', None, ('range', 0, 1)), ('p21', 0, ('range', 0, 4)), ('p22', 0, ('set', [0, 0, 0, 1, 5])), ('p23', None, ('range', 0, 1)), ('p25', 0, ('range', 0, 4)), ('p26', 0, ('set', [0, 0, 0, 1, 5])), ('p27', None, ('range', 0, 1)), ('p29', 0, ('range', 0, 4)), ('p30', 0, ('set', [0, 0, 0, 1, 5])), ('p31', None, ('range', 0, 1)), ('c0', 1100, ('range', 0, 100)), ('c0', 3216, ('range', 0, 200)), ('c0', 2100, ('set', [0, 1, 2]))]
FILLS = []
ARGRANGE = []
SLOTBASE = [(184, 6633360), (44, 4662048)]
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [0, 1]), (1, [0, 1]), (2, [0, 1]), (3, [0, 1]), (4, [0, 0, 1]), (6, [0, 1, 1, 1]), (7, [0, 0, 0, 1, 2]), (8, [0, 1, 1, 1, 2, 3]), (9, [-1, -1, 0, 7, 3])]
VTAT = None
VTATHALF = []
VTIMAGE = [('p56', 0, 11807636)]
ALIAS = []
ANSWERBY = []
ANSWERWIN = [(5, 128), (5, 128), (5, 128), (5, 128)]
CDECL = False
TIB = False
ARGEQ = []
SEQ = [(4, 14, None), (6, 2, None)]
ANSRANGE = []
CELLTHIS = []
VTALSO = [('p57', 0), ('p59', 0), ('p61', 0), ('p63', 0), ('p65', 0), ('p67', 0), ('p69', 0), ('p71', 0), ('p73', 0), ('p75', 0), ('p77', 0), ('p79', 0), ('p81', 0), ('p83', 0), ('p85', 0), ('p87', 0), ('p89', 0), ('p91', 0), ('p93', 0), ('p95', 0), ('p97', 0), ('p99', 0), ('p101', 0), ('p103', 0), ('p105', 0), ('p107', 0), ('p109', 0), ('p111', 0), ('p113', 0), ('p115', 0), ('p117', 0), ('p119', 0), ('p121', 0), ('p123', 0), ('p125', 0), ('p127', 0), ('p129', 0), ('p131', 0), ('p133', 0), ('p135', 0), ('p137', 0), ('p139', 0), ('p141', 0), ('p143', 0), ('p145', 0), ('p147', 0), ('p149', 0), ('p151', 0)]
SPARSE = [(None, '0xc061e0', 28396, 8, [(0, 4), (1060, 4), (2132, 4), (22304, 72)]), (None, '0xc0ab84', 8000, 2, [(0, 16)]), (None, '0xc0aba0', 8000, 2, [(0, 16)]), (None, '0xc0abbc', 8000, 2, [(0, 16)]), (None, '0xc0abd8', 8000, 2, [(0, 16)]), (None, '0xc0abf4', 8000, 2, [(0, 16)]), (None, '0xc0ac10', 8000, 2, [(0, 16)]), (None, '0xc0ac2c', 8000, 2, [(0, 16)]), (None, '0xc0ac48', 8000, 2, [(0, 16)]), (None, '0xc0a390', 16, 1, [(0, 16)]), (None, '0xc0a3ac', 16, 1, [(0, 16)]), (None, '0xc0a3c8', 16, 1, [(0, 16)]), (None, '0xc0a3e4', 16, 1, [(0, 16)]), (None, '0xc0a400', 16, 1, [(0, 16)]), (None, '0xc0a41c', 16, 1, [(0, 16)]), (None, '0xc0a438', 16, 1, [(0, 16)]), (None, '0xc0a454', 16, 1, [(0, 16)])]
GTAB = []
RECORD = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
RECORDORDER = False
RECORDEDX = False
ROWSPLIT = {6: 0, 7: 0, 8: 0}
LINKS = []
LINKHI = []
OUTARG = []
CDECLSTUBS = []
STUBWRITE = []
STUBCOPY = []
STACKCLEAN = 0
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
    rng = random.Random(3261)
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

# sweep_gen: ["--item", "3261", "obj", "BuildData::construct_time@0062d5c0", "--own", "1", "--argset", "0:0,0,0,0,0,1,-1", "--sparse", "0xc061e0:0x6eec:8:0,4/0x424,4/0x854,4/0x5720,0x48", "--sparse", "0xc0ab84:8000:2:0,16", "--sparse", "0xc0aba0:8000:2:0,16", "--sparse", "0xc0abbc:8000:2:0,16", "--sparse", "0xc0abd8:8000:2:0,16", "--sparse", "0xc0abf4:8000:2:0,16", "--sparse", "0xc0ac10:8000:2:0,16", "--sparse", "0xc0ac2c:8000:2:0,16", "--sparse", "0xc0ac48:8000:2:0,16", "--sparse", "0xc0a390:16:1:0,16", "--sparse", "0xc0a3ac:16:1:0,16", "--sparse", "0xc0a3c8:16:1:0,16", "--sparse", "0xc0a3e4:16:1:0,16", "--sparse", "0xc0a400:16:1:0,16", "--sparse", "0xc0a41c:16:1:0,16", "--sparse", "0xc0a438:16:1:0,16", "--sparse", "0xc0a454:16:1:0,16", "--cell-win", "0xc061e4:0xc98", "--at", "0xc0acf4:0x20:2000..2002", "--place", "m+0x18:0:0x10", "--place", "p32+0:0:0x20", "--place", "p57+0x18:0:0x10", "--place", "p32+4:0:0x20", "--place", "p59+0x18:0:0x10", "--place", "p32+8:0:0x20", "--place", "p61+0x18:0:0x10", "--place", "p32+12:0:0x20", "--place", "p63+0x18:0:0x10", "--place", "p34+0:0:0x20", "--place", "p65+0x18:0:0x10", "--place", "p34+4:0:0x20", "--place", "p67+0x18:0:0x10", "--place", "p34+8:0:0x20", "--place", "p69+0x18:0:0x10", "--place", "p34+12:0:0x20", "--place", "p71+0x18:0:0x10", "--place", "p36+0:0:0x20", "--place", "p73+0x18:0:0x10", "--place", "p36+4:0:0x20", "--place", "p75+0x18:0:0x10", "--place", "p36+8:0:0x20", "--place", "p77+0x18:0:0x10", "--place", "p36+12:0:0x20", "--place", "p79+0x18:0:0x10", "--place", "p38+0:0:0x20", "--place", "p81+0x18:0:0x10", "--place", "p38+4:0:0x20", "--place", "p83+0x18:0:0x10", "--place", "p38+8:0:0x20", "--place", "p85+0x18:0:0x10", "--place", "p38+12:0:0x20", "--place", "p87+0x18:0:0x10", "--place", "p40+0:0:0x20", "--place", "p89+0x18:0:0x10", "--place", "p40+4:0:0x20", "--place", "p91+0x18:0:0x10", "--place", "p40+8:0:0x20", "--place", "p93+0x18:0:0x10", "--place", "p40+12:0:0x20", "--place", "p95+0x18:0:0x10", "--place", "p42+0:0:0x20", "--place", "p97+0x18:0:0x10", "--place", "p42+4:0:0x20", "--place", "p99+0x18:0:0x10", "--place", "p42+8:0:0x20", "--place", "p101+0x18:0:0x10", "--place", "p42+12:0:0x20", "--place", "p103+0x18:0:0x10", "--place", "p44+0:0:0x20", "--place", "p105+0x18:0:0x10", "--place", "p44+4:0:0x20", "--place", "p107+0x18:0:0x10", "--place", "p44+8:0:0x20", "--place", "p109+0x18:0:0x10", "--place", "p44+12:0:0x20", "--place", "p111+0x18:0:0x10", "--place", "p46+0:0:0x20", "--place", "p113+0x18:0:0x10", "--place", "p46+4:0:0x20", "--place", "p115+0x18:0:0x10", "--place", "p46+8:0:0x20", "--place", "p117+0x18:0:0x10", "--place", "p46+12:0:0x20", "--place", "p119+0x18:0:0x10", "--place", "p33+0:0:0x20", "--place", "p121+0x18:0:0x10", "--place", "p33+4:0:0x20", "--place", "p123+0x18:0:0x10", "--place", "p35+0:0:0x20", "--place", "p125+0x18:0:0x10", "--place", "p35+4:0:0x20", "--place", "p127+0x18:0:0x10", "--place", "p37+0:0:0x20", "--place", "p129+0x18:0:0x10", "--place", "p37+4:0:0x20", "--place", "p131+0x18:0:0x10", "--place", "p39+0:0:0x20", "--place", "p133+0x18:0:0x10", "--place", "p39+4:0:0x20", "--place", "p135+0x18:0:0x10", "--place", "p41+0:0:0x20", "--place", "p137+0x18:0:0x10", "--place", "p41+4:0:0x20", "--place", "p139+0x18:0:0x10", "--place", "p43+0:0:0x20", "--place", "p141+0x18:0:0x10", "--place", "p43+4:0:0x20", "--place", "p143+0x18:0:0x10", "--place", "p45+0:0:0x20", "--place", "p145+0x18:0:0x10", "--place", "p45+4:0:0x20", "--place", "p147+0x18:0:0x10", "--place", "p47+0:0:0x20", "--place", "p149+0x18:0:0x10", "--place", "p47+4:0:0x20", "--place", "p151+0x18:0:0x10", "--place", "p48+0:0:0x10", "--place", "p48+4:0:0x10", "--place", "p48+8:0:0x10", "--place", "p48+12:0:0x10", "--place", "p49+0:0:0x10", "--place", "p49+4:0:0x10", "--place", "p49+8:0:0x10", "--place", "p49+12:0:0x10", "--place", "p50+0:0:0x10", "--place", "p50+4:0:0x10", "--place", "p50+8:0:0x10", "--place", "p50+12:0:0x10", "--place", "p51+0:0:0x10", "--place", "p51+4:0:0x10", "--place", "p51+8:0:0x10", "--place", "p51+12:0:0x10", "--place", "p52+0:0:0x10", "--place", "p52+4:0:0x10", "--place", "p52+8:0:0x10", "--place", "p52+12:0:0x10", "--place", "p53+0:0:0x10", "--place", "p53+4:0:0x10", "--place", "p53+8:0:0x10", "--place", "p53+12:0:0x10", "--place", "p54+0:0:0x10", "--place", "p54+4:0:0x10", "--place", "p54+8:0:0x10", "--place", "p54+12:0:0x10", "--place", "p55+0:0:0x10", "--place", "p55+4:0:0x10", "--place", "p55+8:0:0x10", "--place", "p55+12:0:0x10", "--stub", "slot:0x2c:0", "--stub", "slot:0xb8:2", "--stub", "slot:0x1c:0", "--stub", "slot:0x60:2", "--stub", "slot:0x4c:0", "--stub", "slot:0x3c:0", "--stub", "0x6e1370:1", "--stub", "0x6db680:0", "--stub", "0x6ebc10:1", "--stub", "0x646b00:2", "--answer-win", "5:0x80", "--answer-win", "5:0x80", "--answer-win", "5:0x80", "--answer-win", "5:0x80", "--seq", "4:14", "--seq", "6:2", "--answers", "0:0,1", "--answers", "1:0,1", "--answers", "2:0,1", "--answers", "3:0,1", "--answers", "4:0,0,1", "--answers", "6:0,1,1,1", "--answers", "7:0,0,0,1,2", "--answers", "8:0,1,1,1,2,3", "--answers", "9:-1,-1,0,7,3", "--record", "0", "--record", "1", "--record", "2", "--record", "3", "--record", "4", "--record", "5", "--record", "6@0", "--record", "7@0", "--record", "8@0", "--record", "9", "--vtat-also", "p57+0", "--vtat-also", "p59+0", "--vtat-also", "p61+0", "--vtat-also", "p63+0", "--vtat-also", "p65+0", "--vtat-also", "p67+0", "--vtat-also", "p69+0", "--vtat-also", "p71+0", "--vtat-also", "p73+0", "--vtat-also", "p75+0", "--vtat-also", "p77+0", "--vtat-also", "p79+0", "--vtat-also", "p81+0", "--vtat-also", "p83+0", "--vtat-also", "p85+0", "--vtat-also", "p87+0", "--vtat-also", "p89+0", "--vtat-also", "p91+0", "--vtat-also", "p93+0", "--vtat-also", "p95+0", "--vtat-also", "p97+0", "--vtat-also", "p99+0", "--vtat-also", "p101+0", "--vtat-also", "p103+0", "--vtat-also", "p105+0", "--vtat-also", "p107+0", "--vtat-also", "p109+0", "--vtat-also", "p111+0", "--vtat-also", "p113+0", "--vtat-also", "p115+0", "--vtat-also", "p117+0", "--vtat-also", "p119+0", "--vtat-also", "p121+0", "--vtat-also", "p123+0", "--vtat-also", "p125+0", "--vtat-also", "p127+0", "--vtat-also", "p129+0", "--vtat-also", "p131+0", "--vtat-also", "p133+0", "--vtat-also", "p135+0", "--vtat-also", "p137+0", "--vtat-also", "p139+0", "--vtat-also", "p141+0", "--vtat-also", "p143+0", "--vtat-also", "p145+0", "--vtat-also", "p147+0", "--vtat-also", "p149+0", "--vtat-also", "p151+0", "--vtat-image", "p56+0:0xb42b94", "--vtat-keep", "0x2c:0x472320", "--slotbase", "0xb8:0x653790", "--wbias", "m+8:131072000,131137536,0,327680,131072256,131137792,256,327936,131072512,131138048,512,328192,131072768,131138304,768,328448,131073024,131138560,1024,328704,131073280,131138816,1280,328960,131073536,131139072,1536,329216,131073792,131139328,1792,329472", "--wbias", "m+0x70:-65536,0,65536,131072,196608", "--wbias", "m+0x50:0,1,2,3,99,100,101,1000,12345,100000,5000000,20000000", "--wbias", "p56+4:528,528,528,526,541,542,510,543,525", "--wbias", "p58+4:528,528,528,528,528,526,529,541,5", "--wbias", "p60+4:528,528,528,528,528,526,529,541,5", "--wbias", "p62+4:528,528,528,528,528,526,529,541,5", "--wbias", "p64+4:528,528,528,528,528,526,529,541,5", "--wbias", "p66+4:528,528,528,528,528,526,529,541,5", "--wbias", "p68+4:528,528,528,528,528,526,529,541,5", "--wbias", "p70+4:528,528,528,528,528,526,529,541,5", "--wbias", "p72+4:528,528,528,528,528,526,529,541,5", "--wbias", "p74+4:528,528,528,528,528,526,529,541,5", "--wbias", "p76+4:528,528,528,528,528,526,529,541,5", "--wbias", "p78+4:528,528,528,528,528,526,529,541,5", "--wbias", "p80+4:528,528,528,528,528,526,529,541,5", "--wbias", "p82+4:528,528,528,528,528,526,529,541,5", "--wbias", "p84+4:528,528,528,528,528,526,529,541,5", "--wbias", "p86+4:528,528,528,528,528,526,529,541,5", "--wbias", "p88+4:528,528,528,528,528,526,529,541,5", "--wbias", "p90+4:528,528,528,528,528,526,529,541,5", "--wbias", "p92+4:528,528,528,528,528,526,529,541,5", "--wbias", "p94+4:528,528,528,528,528,526,529,541,5", "--wbias", "p96+4:528,528,528,528,528,526,529,541,5", "--wbias", "p98+4:528,528,528,528,528,526,529,541,5", "--wbias", "p100+4:528,528,528,528,528,526,529,541,5", "--wbias", "p102+4:528,528,528,528,528,526,529,541,5", "--wbias", "p104+4:528,528,528,528,528,526,529,541,5", "--wbias", "p106+4:528,528,528,528,528,526,529,541,5", "--wbias", "p108+4:528,528,528,528,528,526,529,541,5", "--wbias", "p110+4:528,528,528,528,528,526,529,541,5", "--wbias", "p112+4:528,528,528,528,528,526,529,541,5", "--wbias", "p114+4:528,528,528,528,528,526,529,541,5", "--wbias", "p116+4:528,528,528,528,528,526,529,541,5", "--wbias", "p118+4:528,528,528,528,528,526,529,541,5", "--wbias", "p120+4:528,528,528,528,528,526,529,541,5", "--wbias", "p122+4:528,528,528,526,541,542,510,543,525", "--wbias", "p124+4:528,528,528,526,541,542,510,543,525", "--wbias", "p126+4:528,528,528,526,541,542,510,543,525", "--wbias", "p128+4:528,528,528,526,541,542,510,543,525", "--wbias", "p130+4:528,528,528,526,541,542,510,543,525", "--wbias", "p132+4:528,528,528,526,541,542,510,543,525", "--wbias", "p134+4:528,528,528,526,541,542,510,543,525", "--wbias", "p136+4:528,528,528,526,541,542,510,543,525", "--wbias", "p138+4:528,528,528,526,541,542,510,543,525", "--wbias", "p140+4:528,528,528,526,541,542,510,543,525", "--wbias", "p142+4:528,528,528,526,541,542,510,543,525", "--wbias", "p144+4:528,528,528,526,541,542,510,543,525", "--wbias", "p146+4:528,528,528,526,541,542,510,543,525", "--wbias", "p148+4:528,528,528,526,541,542,510,543,525", "--wbias", "p150+4:528,528,528,526,541,542,510,543,525", "--wbias", "p152+4:528,528,528,526,541,542,510,543,525", "--wbias", "p153..184+0:0,65536,131072,196608", "--wbias", "p185..188+0x70:-65536,0,65536,131072,196608", "--wbias", "p1+0:0..4", "--wbias", "p2+0:0,0,0,1,5", "--wbias", "p3+*:0..1", "--wbias", "p5+0:0..4", "--wbias", "p6+0:0,0,0,1,5", "--wbias", "p7+*:0..1", "--wbias", "p9+0:0..4", "--wbias", "p10+0:0,0,0,1,5", "--wbias", "p11+*:0..1", "--wbias", "p13+0:0..4", "--wbias", "p14+0:0,0,0,1,5", "--wbias", "p15+*:0..1", "--wbias", "p17+0:0..4", "--wbias", "p18+0:0,0,0,1,5", "--wbias", "p19+*:0..1", "--wbias", "p21+0:0..4", "--wbias", "p22+0:0,0,0,1,5", "--wbias", "p23+*:0..1", "--wbias", "p25+0:0..4", "--wbias", "p26+0:0,0,0,1,5", "--wbias", "p27+*:0..1", "--wbias", "p29+0:0..4", "--wbias", "p30+0:0,0,0,1,5", "--wbias", "p31+*:0..1", "--wbias", "c0+0x44c:0..100", "--wbias", "c0+0xc90:0..200", "--wbias", "c0+0x834:0,1,2", "--rows", "3000"]
