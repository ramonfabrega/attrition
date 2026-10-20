#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_army_find_target_run.py — `Army::find_target@006f69b0` under unicorn (item 3362, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_army_find_target_run.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 152-byte object of random bytes (no bias), 0 stack argument(s);
`p` windows in order: p0..p119 sparse table 0 (8 rows × 15 slice(s), row-major), p120..p135 sparse table 1 (8 rows × 2 slice(s), row-major), p136..p288 the place(s), in flag order; the global cell 0x00c061e8 points at 2112 random bytes; the global cell 0x00c061ec points at 1536 random bytes; the global cell 0x00c061b8 points at 32 random bytes; the global cell 0x00c06188 points at 312 random bytes; the global cell 0x00c0618c points at 528 random bytes; callee 0x006f9120 (2 args) stubbed; callee 0x006e33a0 (2 args) stubbed; callee 0x006eb930 (4 args) stubbed; callee 0x006d5b90 (1 args) stubbed; callee 0x006f2e10 (1 args) stubbed; callee 0x006ec000 (0 args) stubbed; callee 0x00680f90 (1 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x0062d470 (0 args) stubbed; callee 0x00739340 (0 args) stubbed; callee 0x007382b0 (1 args) stubbed; callee 0x005948a0 (0 args) stubbed; callee 0x006db810 (1 args) stubbed; callee 0x006f2c90 (5 args) stubbed; callee 0x006f5cc0 (3 args) stubbed; callee 0x006f8ea0 (0 args) stubbed; callee 0x006f9df0 (1 args) stubbed; vtable slot +0x3c (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x11c (1 args) stubbed; the word at `0xc09970+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p136+0` points at 112 random bytes less `0` (placed); the word at `p136+4` points at 112 random bytes less `0` (placed); the word at `p136+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p140+0` points at 112 random bytes less `0` (placed); the word at `p140+4` points at 112 random bytes less `0` (placed); the word at `p140+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*2` points at 12 random bytes less `0` (placed); the word at `p144+0` points at 112 random bytes less `0` (placed); the word at `p144+4` points at 112 random bytes less `0` (placed); the word at `p144+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*3` points at 12 random bytes less `0` (placed); the word at `p148+0` points at 112 random bytes less `0` (placed); the word at `p148+4` points at 112 random bytes less `0` (placed); the word at `p148+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*4` points at 12 random bytes less `0` (placed); the word at `p152+0` points at 112 random bytes less `0` (placed); the word at `p152+4` points at 112 random bytes less `0` (placed); the word at `p152+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*5` points at 12 random bytes less `0` (placed); the word at `p156+0` points at 112 random bytes less `0` (placed); the word at `p156+4` points at 112 random bytes less `0` (placed); the word at `p156+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*6` points at 12 random bytes less `0` (placed); the word at `p160+0` points at 112 random bytes less `0` (placed); the word at `p160+4` points at 112 random bytes less `0` (placed); the word at `p160+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*7` points at 12 random bytes less `0` (placed); the word at `p164+0` points at 112 random bytes less `0` (placed); the word at `p164+4` points at 112 random bytes less `0` (placed); the word at `p164+8` points at 112 random bytes less `0` (placed); the word at `c4+0x14+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p168+0` points at 64 random bytes less `0` (placed); the word at `p168+4` points at 64 random bytes less `0` (placed); the word at `p168+8` points at 64 random bytes less `0` (placed); the word at `c4+0x14+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p172+0` points at 64 random bytes less `0` (placed); the word at `p172+4` points at 64 random bytes less `0` (placed); the word at `p172+8` points at 64 random bytes less `0` (placed); the word at `c4+0x14+0x1c*2` points at 12 random bytes less `0` (placed); the word at `p176+0` points at 64 random bytes less `0` (placed); the word at `p176+4` points at 64 random bytes less `0` (placed); the word at `p176+8` points at 64 random bytes less `0` (placed); the word at `c4+0x14+0x1c*3` points at 12 random bytes less `0` (placed); the word at `p180+0` points at 64 random bytes less `0` (placed); the word at `p180+4` points at 64 random bytes less `0` (placed); the word at `p180+8` points at 64 random bytes less `0` (placed); the word at `c4+0x14+0x1c*4` points at 12 random bytes less `0` (placed); the word at `p184+0` points at 64 random bytes less `0` (placed); the word at `p184+4` points at 64 random bytes less `0` (placed); the word at `p184+8` points at 64 random bytes less `0` (placed); the word at `c4+0x14+0x1c*5` points at 12 random bytes less `0` (placed); the word at `p188+0` points at 64 random bytes less `0` (placed); the word at `p188+4` points at 64 random bytes less `0` (placed); the word at `p188+8` points at 64 random bytes less `0` (placed); the word at `c4+0x14+0x1c*6` points at 12 random bytes less `0` (placed); the word at `p192+0` points at 64 random bytes less `0` (placed); the word at `p192+4` points at 64 random bytes less `0` (placed); the word at `p192+8` points at 64 random bytes less `0` (placed); the word at `c4+0x14+0x1c*7` points at 12 random bytes less `0` (placed); the word at `p196+0` points at 64 random bytes less `0` (placed); the word at `p196+4` points at 64 random bytes less `0` (placed); the word at `p196+8` points at 64 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p200+0` points at 64 random bytes less `0` (placed); the word at `p200+4` points at 64 random bytes less `0` (placed); the word at `p200+8` points at 64 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p204+0` points at 64 random bytes less `0` (placed); the word at `p204+4` points at 64 random bytes less `0` (placed); the word at `p204+8` points at 64 random bytes less `0` (placed); the word at `0xc09710+0x1c*0` points at 64 random bytes less `0` (placed); the word at `p208+0` points at 80 random bytes less `0` (placed); the word at `p208+4` points at 80 random bytes less `0` (placed); the word at `p208+8` points at 80 random bytes less `0` (placed); the word at `p208+12` points at 80 random bytes less `0` (placed); the word at `p208+16` points at 80 random bytes less `0` (placed); the word at `p208+20` points at 80 random bytes less `0` (placed); the word at `p208+24` points at 80 random bytes less `0` (placed); the word at `p208+28` points at 80 random bytes less `0` (placed); the word at `p208+32` points at 80 random bytes less `0` (placed); the word at `p208+36` points at 80 random bytes less `0` (placed); the word at `p208+40` points at 80 random bytes less `0` (placed); the word at `p208+44` points at 80 random bytes less `0` (placed); the word at `p208+48` points at 80 random bytes less `0` (placed); the word at `p208+52` points at 80 random bytes less `0` (placed); the word at `p208+56` points at 80 random bytes less `0` (placed); the word at `p208+60` points at 80 random bytes less `0` (placed); the word at `0xc09710+0x1c*1` points at 64 random bytes less `0` (placed); the word at `p225+0` points at 80 random bytes less `0` (placed); the word at `p225+4` points at 80 random bytes less `0` (placed); the word at `p225+8` points at 80 random bytes less `0` (placed); the word at `p225+12` points at 80 random bytes less `0` (placed); the word at `p225+16` points at 80 random bytes less `0` (placed); the word at `p225+20` points at 80 random bytes less `0` (placed); the word at `p225+24` points at 80 random bytes less `0` (placed); the word at `p225+28` points at 80 random bytes less `0` (placed); the word at `p225+32` points at 80 random bytes less `0` (placed); the word at `p225+36` points at 80 random bytes less `0` (placed); the word at `p225+40` points at 80 random bytes less `0` (placed); the word at `p225+44` points at 80 random bytes less `0` (placed); the word at `p225+48` points at 80 random bytes less `0` (placed); the word at `p225+52` points at 80 random bytes less `0` (placed); the word at `p225+56` points at 80 random bytes less `0` (placed); the word at `p225+60` points at 80 random bytes less `0` (placed); the word at `0xc09710+0x1c*2` points at 64 random bytes less `0` (placed); the word at `p242+0` points at 80 random bytes less `0` (placed); the word at `p242+4` points at 80 random bytes less `0` (placed); the word at `p242+8` points at 80 random bytes less `0` (placed); the word at `p242+12` points at 80 random bytes less `0` (placed); the word at `p242+16` points at 80 random bytes less `0` (placed); the word at `p242+20` points at 80 random bytes less `0` (placed); the word at `p242+24` points at 80 random bytes less `0` (placed); the word at `p242+28` points at 80 random bytes less `0` (placed); the word at `p242+32` points at 80 random bytes less `0` (placed); the word at `p242+36` points at 80 random bytes less `0` (placed); the word at `p242+40` points at 80 random bytes less `0` (placed); the word at `p242+44` points at 80 random bytes less `0` (placed); the word at `p242+48` points at 80 random bytes less `0` (placed); the word at `p242+52` points at 80 random bytes less `0` (placed); the word at `p242+56` points at 80 random bytes less `0` (placed); the word at `p242+60` points at 80 random bytes less `0` (placed); the word at `0xc09710+0x1c*3` points at 64 random bytes less `0` (placed); the word at `p259+0` points at 80 random bytes less `0` (placed); the word at `p259+4` points at 80 random bytes less `0` (placed); the word at `p259+8` points at 80 random bytes less `0` (placed); the word at `p259+12` points at 80 random bytes less `0` (placed); the word at `p259+16` points at 80 random bytes less `0` (placed); the word at `p259+20` points at 80 random bytes less `0` (placed); the word at `p259+24` points at 80 random bytes less `0` (placed); the word at `p259+28` points at 80 random bytes less `0` (placed); the word at `p259+32` points at 80 random bytes less `0` (placed); the word at `p259+36` points at 80 random bytes less `0` (placed); the word at `p259+40` points at 80 random bytes less `0` (placed); the word at `p259+44` points at 80 random bytes less `0` (placed); the word at `p259+48` points at 80 random bytes less `0` (placed); the word at `p259+52` points at 80 random bytes less `0` (placed); the word at `p259+56` points at 80 random bytes less `0` (placed); the word at `p259+60` points at 80 random bytes less `0` (placed); the word at `0xcae5fc` points at 1024 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*0+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*1+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*2+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*3+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*4+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*5+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*6+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*7+0x6eb8` points at 256 random bytes less `0` (placed); the word at `c3+0x134` points at 12288 random bytes less `0` (placed); stub 17's answer points at 256 random bytes less `0` (placed); the word at `p286+0x18` points at 32 random bytes less `0` (placed); stub 18's answer points at 256 random bytes less `0` (placed); stub 2 answered from [0] (90%); the stubbed vtable laid at `m+0x90`; stub 0 answers 6 values in call order, printed after the answers; stub 3 answers 16 values in call order, printed after the answers; stub 4 answers 16 values in call order, printed after the answers; stub 5 answers 16 values in call order, printed after the answers; stub 6 answers 48 values in call order, printed after the answers; stub 7 answers 40 values in call order, printed after the answers; stub 8 answers 8 values in call order, printed after the answers; stub 9 answers 6 values in call order, printed after the answers; stub 10 answers 4 values in call order, printed after the answers; stub 19 answers 2 values in call order, printed after the answers; stub 0 answers 0..6 every row; stub 1 answers 0..1 every row; stub 3 answers 0..1 every row; stub 4 answers -1..7 every row; stub 5 answers 0..5 every row; stub 6 answers 0..1 every row; stub 7 answers 0..65535 every row; stub 8 answers 0..1 every row; stub 9 answers 1..3 every row; stub 10 answers 0..1 every row; stub 11 answers 0..7 every row; stub 12 answers 0..1 every row; stub 14 answers 0..1 every row; stub 16 answers -1..-1 every row; stub 19 answers 0..100 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 2 writes a value in [-1, 0, 1, 2] through its argument 0, printed after the answers (`o` in a place); stub 2 writes a value in [0, 1, 2, 3, 4, 5, 6, 7] through its argument 1, printed after the answers (`o` in a place); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x8:0x4, +0x18:0x4, +0x50:0x4, +0x74:0x20, +0x230:0x20, +0x250:0x20, +0x408:0x4, +0x444:0xc, +0x7a8:0x4, +0x7e8:0x4, +0x944:0xc, +0xa40:0x8, +0x6dec:0x4, +0x6e20:0x4 (`p` windows after the pointed buffers, before the places); 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4, +0x74:0x20 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p288+0` as well as at `this`; the stubbed vtable laid at `p169+0` as well as at `this`; the stubbed vtable laid at `p170+0` as well as at `this`; the stubbed vtable laid at `p171+0` as well as at `this`; the stubbed vtable laid at `p173+0` as well as at `this`; the stubbed vtable laid at `p174+0` as well as at `this`; the stubbed vtable laid at `p175+0` as well as at `this`; the stubbed vtable laid at `p177+0` as well as at `this`; the stubbed vtable laid at `p178+0` as well as at `this`; the stubbed vtable laid at `p179+0` as well as at `this`; the stubbed vtable laid at `p181+0` as well as at `this`; the stubbed vtable laid at `p182+0` as well as at `this`; the stubbed vtable laid at `p183+0` as well as at `this`; the stubbed vtable laid at `p185+0` as well as at `this`; the stubbed vtable laid at `p186+0` as well as at `this`; the stubbed vtable laid at `p187+0` as well as at `this`; the stubbed vtable laid at `p189+0` as well as at `this`; the stubbed vtable laid at `p190+0` as well as at `this`; the stubbed vtable laid at `p191+0` as well as at `this`; the stubbed vtable laid at `p193+0` as well as at `this`; the stubbed vtable laid at `p194+0` as well as at `this`; the stubbed vtable laid at `p195+0` as well as at `this`; the stubbed vtable laid at `p197+0` as well as at `this`; the stubbed vtable laid at `p198+0` as well as at `this`; the stubbed vtable laid at `p199+0` as well as at `this`; the stubbed vtable laid at `p201+0` as well as at `this`; the stubbed vtable laid at `p202+0` as well as at `this`; the stubbed vtable laid at `p203+0` as well as at `this`; the stubbed vtable laid at `p205+0` as well as at `this`; the stubbed vtable laid at `p206+0` as well as at `this`; the stubbed vtable laid at `p207+0` as well as at `this`.
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

ENTRY = 0x006f69b0
WIN = 152
OWN = 0
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607976, 2112), (12607980, 1536), (12607928, 32), (12607880, 312), (12607884, 528)]
AT = []
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('call', 7311648, 2), ('call', 7222176, 2), ('call', 7256368, 4), ('call', 7166864, 1), ('call', 7286288, 1), ('call', 7258112, 0), ('call', 6819728, 1), ('call', 10722672, 2), ('call', 6476912, 0), ('call', 7574336, 0), ('call', 7570096, 1), ('call', 5851296, 0), ('call', 7190544, 1), ('call', 7285904, 5), ('call', 7298240, 3), ('call', 7311008, 0), ('call', 7314928, 1), ('slot', 60, 0), ('slot', 172, 0), ('slot', 284, 1)]
PLACES = [(None, '0xc09970+0x1c*0', '0', 12), ('p136', '0', '0', 112), ('p136', '4', '0', 112), ('p136', '8', '0', 112), (None, '0xc09970+0x1c*1', '0', 12), ('p140', '0', '0', 112), ('p140', '4', '0', 112), ('p140', '8', '0', 112), (None, '0xc09970+0x1c*2', '0', 12), ('p144', '0', '0', 112), ('p144', '4', '0', 112), ('p144', '8', '0', 112), (None, '0xc09970+0x1c*3', '0', 12), ('p148', '0', '0', 112), ('p148', '4', '0', 112), ('p148', '8', '0', 112), (None, '0xc09970+0x1c*4', '0', 12), ('p152', '0', '0', 112), ('p152', '4', '0', 112), ('p152', '8', '0', 112), (None, '0xc09970+0x1c*5', '0', 12), ('p156', '0', '0', 112), ('p156', '4', '0', 112), ('p156', '8', '0', 112), (None, '0xc09970+0x1c*6', '0', 12), ('p160', '0', '0', 112), ('p160', '4', '0', 112), ('p160', '8', '0', 112), (None, '0xc09970+0x1c*7', '0', 12), ('p164', '0', '0', 112), ('p164', '4', '0', 112), ('p164', '8', '0', 112), ('c4', '0x14+0x1c*0', '0', 12), ('p168', '0', '0', 64), ('p168', '4', '0', 64), ('p168', '8', '0', 64), ('c4', '0x14+0x1c*1', '0', 12), ('p172', '0', '0', 64), ('p172', '4', '0', 64), ('p172', '8', '0', 64), ('c4', '0x14+0x1c*2', '0', 12), ('p176', '0', '0', 64), ('p176', '4', '0', 64), ('p176', '8', '0', 64), ('c4', '0x14+0x1c*3', '0', 12), ('p180', '0', '0', 64), ('p180', '4', '0', 64), ('p180', '8', '0', 64), ('c4', '0x14+0x1c*4', '0', 12), ('p184', '0', '0', 64), ('p184', '4', '0', 64), ('p184', '8', '0', 64), ('c4', '0x14+0x1c*5', '0', 12), ('p188', '0', '0', 64), ('p188', '4', '0', 64), ('p188', '8', '0', 64), ('c4', '0x14+0x1c*6', '0', 12), ('p192', '0', '0', 64), ('p192', '4', '0', 64), ('p192', '8', '0', 64), ('c4', '0x14+0x1c*7', '0', 12), ('p196', '0', '0', 64), ('p196', '4', '0', 64), ('p196', '8', '0', 64), (None, '0xc0ab84+0x1c*0', '0', 12), ('p200', '0', '0', 64), ('p200', '4', '0', 64), ('p200', '8', '0', 64), (None, '0xc0ab84+0x1c*1', '0', 12), ('p204', '0', '0', 64), ('p204', '4', '0', 64), ('p204', '8', '0', 64), (None, '0xc09710+0x1c*0', '0', 64), ('p208', '0', '0', 80), ('p208', '4', '0', 80), ('p208', '8', '0', 80), ('p208', '12', '0', 80), ('p208', '16', '0', 80), ('p208', '20', '0', 80), ('p208', '24', '0', 80), ('p208', '28', '0', 80), ('p208', '32', '0', 80), ('p208', '36', '0', 80), ('p208', '40', '0', 80), ('p208', '44', '0', 80), ('p208', '48', '0', 80), ('p208', '52', '0', 80), ('p208', '56', '0', 80), ('p208', '60', '0', 80), (None, '0xc09710+0x1c*1', '0', 64), ('p225', '0', '0', 80), ('p225', '4', '0', 80), ('p225', '8', '0', 80), ('p225', '12', '0', 80), ('p225', '16', '0', 80), ('p225', '20', '0', 80), ('p225', '24', '0', 80), ('p225', '28', '0', 80), ('p225', '32', '0', 80), ('p225', '36', '0', 80), ('p225', '40', '0', 80), ('p225', '44', '0', 80), ('p225', '48', '0', 80), ('p225', '52', '0', 80), ('p225', '56', '0', 80), ('p225', '60', '0', 80), (None, '0xc09710+0x1c*2', '0', 64), ('p242', '0', '0', 80), ('p242', '4', '0', 80), ('p242', '8', '0', 80), ('p242', '12', '0', 80), ('p242', '16', '0', 80), ('p242', '20', '0', 80), ('p242', '24', '0', 80), ('p242', '28', '0', 80), ('p242', '32', '0', 80), ('p242', '36', '0', 80), ('p242', '40', '0', 80), ('p242', '44', '0', 80), ('p242', '48', '0', 80), ('p242', '52', '0', 80), ('p242', '56', '0', 80), ('p242', '60', '0', 80), (None, '0xc09710+0x1c*3', '0', 64), ('p259', '0', '0', 80), ('p259', '4', '0', 80), ('p259', '8', '0', 80), ('p259', '12', '0', 80), ('p259', '16', '0', 80), ('p259', '20', '0', 80), ('p259', '24', '0', 80), ('p259', '28', '0', 80), ('p259', '32', '0', 80), ('p259', '36', '0', 80), ('p259', '40', '0', 80), ('p259', '44', '0', 80), ('p259', '48', '0', 80), ('p259', '52', '0', 80), ('p259', '56', '0', 80), ('p259', '60', '0', 80), (None, '0xcae5fc', '0', 1024), (None, '0xe3a390+0x6eec*0+0x6eb8', '0', 256), (None, '0xe3a390+0x6eec*1+0x6eb8', '0', 256), (None, '0xe3a390+0x6eec*2+0x6eb8', '0', 256), (None, '0xe3a390+0x6eec*3+0x6eb8', '0', 256), (None, '0xe3a390+0x6eec*4+0x6eb8', '0', 256), (None, '0xe3a390+0x6eec*5+0x6eb8', '0', 256), (None, '0xe3a390+0x6eec*6+0x6eb8', '0', 256), (None, '0xe3a390+0x6eec*7+0x6eb8', '0', 256), ('c3', '0x134', '0', 12288), ('s17', '0', '0', 256), ('p286', '0x18', '0', 32), ('s18', '0', '0', 256)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 0, ('set', [0, 65536, 131072, 196608, 262144, 327680, 393216, 458752])), ('m', 8, ('set', [0, 4, 8, 32, 60, 1, 2])), ('m', 20, ('range', 0, 12)), ('m', 24, ('range', 0, 9)), ('m', 36, ('set', [0, 0, 0, 1])), ('m', 148, ('range', 0, 3)), ('m', 48, ('set', [0, 1, 2, -1])), ('m', 52, ('range', 0, 7)), ('m', 56, ('range', 0, 65535)), ('m', 60, ('range', 0, 65535)), ('p0', 0, ('set', [3, 3, 3, 7, 259, 771, 1795, 263, 6, 1, 2, 11])), ('p1', 0, ('set', [0])), ('p2', 0, ('range', 0, 30)), ('p3', 0, ('set', [-1, 0, 1, 2, 3, 4, 5])), ('p4', None, ('set', [0, 1, 2, 2])), ('p5', None, ('set', [0, 1, 1])), ('p6', None, ('set', [0, 0, 0, 1])), ('p7', 0, ('range', 0, 2)), ('p8', None, ('set', [0, 0, 1])), ('p9', 0, ('set', [0, 128, 255, 256, 257, 512])), ('p10', 0, ('set', [0, 0, 1])), ('p11', 0, ('range', 0, 24)), ('p11', 8, ('range', 0, 40)), ('p12', 0, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p12', 4, ('set', [-1, 0, 1, 2, 3, 4, 5, 6, 7])), ('p13', 0, ('set', [-1, 0, 1])), ('p14', 0, ('set', [0, 1])), ('p120', 0, ('set', [0, 1, 2, 3, 3, 3])), ('p121', None, ('set', [0, 1, 2, 2])), ('p15', 0, ('set', [3, 3, 3, 7, 259, 771, 1795, 263, 6, 1, 2, 11])), ('p16', 0, ('set', [1])), ('p17', 0, ('range', 0, 30)), ('p18', 0, ('set', [-1, 0, 1, 2, 3, 4, 5])), ('p19', None, ('set', [0, 1, 2, 2])), ('p20', None, ('set', [0, 1, 1])), ('p21', None, ('set', [0, 0, 0, 1])), ('p22', 0, ('range', 0, 2)), ('p23', None, ('set', [0, 0, 1])), ('p24', 0, ('set', [0, 128, 255, 256, 257, 512])), ('p25', 0, ('set', [0, 0, 1])), ('p26', 0, ('range', 0, 24)), ('p26', 8, ('range', 0, 40)), ('p27', 0, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p27', 4, ('set', [-1, 0, 1, 2, 3, 4, 5, 6, 7])), ('p28', 0, ('set', [-1, 0, 1])), ('p29', 0, ('set', [0, 1])), ('p122', 0, ('set', [0, 1, 2, 3, 3, 3])), ('p123', None, ('set', [0, 1, 2, 2])), ('p30', 0, ('set', [3, 3, 3, 7, 259, 771, 1795, 263, 6, 1, 2, 11])), ('p31', 0, ('set', [2])), ('p32', 0, ('range', 0, 30)), ('p33', 0, ('set', [-1, 0, 1, 2, 3, 4, 5])), ('p34', None, ('set', [0, 1, 2, 2])), ('p35', None, ('set', [0, 1, 1])), ('p36', None, ('set', [0, 0, 0, 1])), ('p37', 0, ('range', 0, 2)), ('p38', None, ('set', [0, 0, 1])), ('p39', 0, ('set', [0, 128, 255, 256, 257, 512])), ('p40', 0, ('set', [0, 0, 1])), ('p41', 0, ('range', 0, 24)), ('p41', 8, ('range', 0, 40)), ('p42', 0, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p42', 4, ('set', [-1, 0, 1, 2, 3, 4, 5, 6, 7])), ('p43', 0, ('set', [-1, 0, 1])), ('p44', 0, ('set', [0, 1])), ('p124', 0, ('set', [0, 1, 2, 3, 3, 3])), ('p125', None, ('set', [0, 1, 2, 2])), ('p45', 0, ('set', [3, 3, 3, 7, 259, 771, 1795, 263, 6, 1, 2, 11])), ('p46', 0, ('set', [3])), ('p47', 0, ('range', 0, 30)), ('p48', 0, ('set', [-1, 0, 1, 2, 3, 4, 5])), ('p49', None, ('set', [0, 1, 2, 2])), ('p50', None, ('set', [0, 1, 1])), ('p51', None, ('set', [0, 0, 0, 1])), ('p52', 0, ('range', 0, 2)), ('p53', None, ('set', [0, 0, 1])), ('p54', 0, ('set', [0, 128, 255, 256, 257, 512])), ('p55', 0, ('set', [0, 0, 1])), ('p56', 0, ('range', 0, 24)), ('p56', 8, ('range', 0, 40)), ('p57', 0, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p57', 4, ('set', [-1, 0, 1, 2, 3, 4, 5, 6, 7])), ('p58', 0, ('set', [-1, 0, 1])), ('p59', 0, ('set', [0, 1])), ('p126', 0, ('set', [0, 1, 2, 3, 3, 3])), ('p127', None, ('set', [0, 1, 2, 2])), ('p60', 0, ('set', [3, 3, 3, 7, 259, 771, 1795, 263, 6, 1, 2, 11])), ('p61', 0, ('set', [4])), ('p62', 0, ('range', 0, 30)), ('p63', 0, ('set', [-1, 0, 1, 2, 3, 4, 5])), ('p64', None, ('set', [0, 1, 2, 2])), ('p65', None, ('set', [0, 1, 1])), ('p66', None, ('set', [0, 0, 0, 1])), ('p67', 0, ('range', 0, 2)), ('p68', None, ('set', [0, 0, 1])), ('p69', 0, ('set', [0, 128, 255, 256, 257, 512])), ('p70', 0, ('set', [0, 0, 1])), ('p71', 0, ('range', 0, 24)), ('p71', 8, ('range', 0, 40)), ('p72', 0, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p72', 4, ('set', [-1, 0, 1, 2, 3, 4, 5, 6, 7])), ('p73', 0, ('set', [-1, 0, 1])), ('p74', 0, ('set', [0, 1])), ('p128', 0, ('set', [0, 1, 2, 3, 3, 3])), ('p129', None, ('set', [0, 1, 2, 2])), ('p75', 0, ('set', [3, 3, 3, 7, 259, 771, 1795, 263, 6, 1, 2, 11])), ('p76', 0, ('set', [5])), ('p77', 0, ('range', 0, 30)), ('p78', 0, ('set', [-1, 0, 1, 2, 3, 4, 5])), ('p79', None, ('set', [0, 1, 2, 2])), ('p80', None, ('set', [0, 1, 1])), ('p81', None, ('set', [0, 0, 0, 1])), ('p82', 0, ('range', 0, 2)), ('p83', None, ('set', [0, 0, 1])), ('p84', 0, ('set', [0, 128, 255, 256, 257, 512])), ('p85', 0, ('set', [0, 0, 1])), ('p86', 0, ('range', 0, 24)), ('p86', 8, ('range', 0, 40)), ('p87', 0, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p87', 4, ('set', [-1, 0, 1, 2, 3, 4, 5, 6, 7])), ('p88', 0, ('set', [-1, 0, 1])), ('p89', 0, ('set', [0, 1])), ('p130', 0, ('set', [0, 1, 2, 3, 3, 3])), ('p131', None, ('set', [0, 1, 2, 2])), ('p90', 0, ('set', [3, 3, 3, 7, 259, 771, 1795, 263, 6, 1, 2, 11])), ('p91', 0, ('set', [6])), ('p92', 0, ('range', 0, 30)), ('p93', 0, ('set', [-1, 0, 1, 2, 3, 4, 5])), ('p94', None, ('set', [0, 1, 2, 2])), ('p95', None, ('set', [0, 1, 1])), ('p96', None, ('set', [0, 0, 0, 1])), ('p97', 0, ('range', 0, 2)), ('p98', None, ('set', [0, 0, 1])), ('p99', 0, ('set', [0, 128, 255, 256, 257, 512])), ('p100', 0, ('set', [0, 0, 1])), ('p101', 0, ('range', 0, 24)), ('p101', 8, ('range', 0, 40)), ('p102', 0, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p102', 4, ('set', [-1, 0, 1, 2, 3, 4, 5, 6, 7])), ('p103', 0, ('set', [-1, 0, 1])), ('p104', 0, ('set', [0, 1])), ('p132', 0, ('set', [0, 1, 2, 3, 3, 3])), ('p133', None, ('set', [0, 1, 2, 2])), ('p105', 0, ('set', [3, 3, 3, 7, 259, 771, 1795, 263, 6, 1, 2, 11])), ('p106', 0, ('set', [7])), ('p107', 0, ('range', 0, 30)), ('p108', 0, ('set', [-1, 0, 1, 2, 3, 4, 5])), ('p109', None, ('set', [0, 1, 2, 2])), ('p110', None, ('set', [0, 1, 1])), ('p111', None, ('set', [0, 0, 0, 1])), ('p112', 0, ('range', 0, 2)), ('p113', None, ('set', [0, 0, 1])), ('p114', 0, ('set', [0, 128, 255, 256, 257, 512])), ('p115', 0, ('set', [0, 0, 1])), ('p116', 0, ('range', 0, 24)), ('p116', 8, ('range', 0, 40)), ('p117', 0, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p117', 4, ('set', [-1, 0, 1, 2, 3, 4, 5, 6, 7])), ('p118', 0, ('set', [-1, 0, 1])), ('p119', 0, ('set', [0, 1])), ('p134', 0, ('set', [0, 1, 2, 3, 3, 3])), ('p135', None, ('set', [0, 1, 2, 2])), ('c0', 2080, ('set', [0, 4, 4096, 65536, 131072, 135168, 131076, 4100])), ('c0', 40, ('set', [0, 16777216, 33554432, 50331648, 67108864, 83886080])), ('c0', 1628, ('range', 0, 7)), ('c0', 1660, ('range', 0, 7)), ('c0', 1632, ('range', 0, 7)), ('c0', 1664, ('range', 0, 7)), ('c0', 1636, ('range', 0, 7)), ('c0', 1668, ('range', 0, 7)), ('c0', 1640, ('range', 0, 7)), ('c0', 1672, ('range', 0, 7)), ('c0', 1644, ('range', 0, 7)), ('c0', 1676, ('range', 0, 7)), ('c0', 1648, ('range', 0, 7)), ('c0', 1680, ('range', 0, 7)), ('c0', 1652, ('range', 0, 7)), ('c0', 1684, ('range', 0, 7)), ('c0', 1656, ('range', 0, 7)), ('c0', 1688, ('range', 0, 7)), ('c1', 36, ('set', [0, 1, 2, 2, 2, 3])), ('c1', 1360, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('c3', 0, ('range', 1, 4)), ('c3', 4, ('range', 1, 4)), ('p137', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p137', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p137', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p137', 92, ('set', [0, 65536])), ('p137', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p137', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p137', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p137', 12, ('range', 0, 65535)), ('p137', 16, ('range', 0, 65535)), ('p138', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p138', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p138', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p138', 92, ('set', [0, 65536])), ('p138', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p138', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p138', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p138', 12, ('range', 0, 65535)), ('p138', 16, ('range', 0, 65535)), ('p139', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p139', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p139', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p139', 92, ('set', [0, 65536])), ('p139', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p139', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p139', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p139', 12, ('range', 0, 65535)), ('p139', 16, ('range', 0, 65535)), ('p141', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p141', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p141', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p141', 92, ('set', [0, 65536])), ('p141', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p141', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p141', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p141', 12, ('range', 0, 65535)), ('p141', 16, ('range', 0, 65535)), ('p142', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p142', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p142', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p142', 92, ('set', [0, 65536])), ('p142', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p142', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p142', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p142', 12, ('range', 0, 65535)), ('p142', 16, ('range', 0, 65535)), ('p143', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p143', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p143', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p143', 92, ('set', [0, 65536])), ('p143', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p143', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p143', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p143', 12, ('range', 0, 65535)), ('p143', 16, ('range', 0, 65535)), ('p145', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p145', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p145', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p145', 92, ('set', [0, 65536])), ('p145', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p145', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p145', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p145', 12, ('range', 0, 65535)), ('p145', 16, ('range', 0, 65535)), ('p146', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p146', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p146', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p146', 92, ('set', [0, 65536])), ('p146', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p146', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p146', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p146', 12, ('range', 0, 65535)), ('p146', 16, ('range', 0, 65535)), ('p147', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p147', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p147', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p147', 92, ('set', [0, 65536])), ('p147', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p147', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p147', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p147', 12, ('range', 0, 65535)), ('p147', 16, ('range', 0, 65535)), ('p149', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p149', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p149', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p149', 92, ('set', [0, 65536])), ('p149', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p149', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p149', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p149', 12, ('range', 0, 65535)), ('p149', 16, ('range', 0, 65535)), ('p150', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p150', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p150', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p150', 92, ('set', [0, 65536])), ('p150', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p150', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p150', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p150', 12, ('range', 0, 65535)), ('p150', 16, ('range', 0, 65535)), ('p151', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p151', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p151', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p151', 92, ('set', [0, 65536])), ('p151', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p151', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p151', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p151', 12, ('range', 0, 65535)), ('p151', 16, ('range', 0, 65535)), ('p153', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p153', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p153', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p153', 92, ('set', [0, 65536])), ('p153', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p153', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p153', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p153', 12, ('range', 0, 65535)), ('p153', 16, ('range', 0, 65535)), ('p154', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p154', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p154', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p154', 92, ('set', [0, 65536])), ('p154', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p154', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p154', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p154', 12, ('range', 0, 65535)), ('p154', 16, ('range', 0, 65535)), ('p155', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p155', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p155', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p155', 92, ('set', [0, 65536])), ('p155', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p155', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p155', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p155', 12, ('range', 0, 65535)), ('p155', 16, ('range', 0, 65535)), ('p157', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p157', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p157', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p157', 92, ('set', [0, 65536])), ('p157', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p157', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p157', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p157', 12, ('range', 0, 65535)), ('p157', 16, ('range', 0, 65535)), ('p158', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p158', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p158', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p158', 92, ('set', [0, 65536])), ('p158', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p158', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p158', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p158', 12, ('range', 0, 65535)), ('p158', 16, ('range', 0, 65535)), ('p159', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p159', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p159', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p159', 92, ('set', [0, 65536])), ('p159', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p159', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p159', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p159', 12, ('range', 0, 65535)), ('p159', 16, ('range', 0, 65535)), ('p161', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p161', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p161', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p161', 92, ('set', [0, 65536])), ('p161', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p161', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p161', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p161', 12, ('range', 0, 65535)), ('p161', 16, ('range', 0, 65535)), ('p162', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p162', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p162', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p162', 92, ('set', [0, 65536])), ('p162', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p162', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p162', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p162', 12, ('range', 0, 65535)), ('p162', 16, ('range', 0, 65535)), ('p163', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p163', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p163', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p163', 92, ('set', [0, 65536])), ('p163', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p163', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p163', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p163', 12, ('range', 0, 65535)), ('p163', 16, ('range', 0, 65535)), ('p165', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p165', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p165', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p165', 92, ('set', [0, 65536])), ('p165', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p165', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p165', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p165', 12, ('range', 0, 65535)), ('p165', 16, ('range', 0, 65535)), ('p166', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p166', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p166', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p166', 92, ('set', [0, 65536])), ('p166', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p166', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p166', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p166', 12, ('range', 0, 65535)), ('p166', 16, ('range', 0, 65535)), ('p167', 4, ('set', [0, 1, 1, 3, 17, 19, 8193, 8195, 8209, 0])), ('p167', 8, ('set', [0, 262144, 524288, 2097152, 3932160, 1, 262145, 524289, 2097153, 3932161, 2, 262146, 524290, 2097154, 3932162])), ('p167', 20, ('set', [0, 900, 1800, 2700, 3600, 4500, 5400, 6300, 7200, 8100, 9000, 9900, 10800, 11700, 12600, 13500, 14400, 15300, 16200, 17100, 18000, 18900, 19800, 20700, 21600, 22500, 23400, 24300, 25200, 26100])), ('p167', 92, ('set', [0, 65536])), ('p167', 96, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539])), ('p167', 100, ('set', [0, 256, 512, 1024, 2048, 768, 3840])), ('p167', 104, ('set', [0, 0, 1, 2, 4, 8, 3])), ('p167', 12, ('range', 0, 65535)), ('p167', 16, ('range', 0, 65535)), ('p277', 232, ('set', [405895, 405894, 405893, 405892, 405891, 405890, 405889])), ('p278', 232, ('set', [405895, 405894, 405893, 405892, 405891, 405890, 405889])), ('p279', 232, ('set', [405895, 405894, 405893, 405892, 405891, 405890, 405889])), ('p280', 232, ('set', [405895, 405894, 405893, 405892, 405891, 405890, 405889])), ('p281', 232, ('set', [405895, 405894, 405893, 405892, 405891, 405890, 405889])), ('p282', 232, ('set', [405895, 405894, 405893, 405892, 405891, 405890, 405889])), ('p283', 232, ('set', [405895, 405894, 405893, 405892, 405891, 405890, 405889])), ('p284', 232, ('set', [405895, 405894, 405893, 405892, 405891, 405890, 405889])), ('p169', 16, ('range', 393216, 458751)), ('p169', 20, ('range', 393216, 458751)), ('p170', 16, ('range', 393216, 458751)), ('p170', 20, ('range', 393216, 458751)), ('p171', 16, ('range', 393216, 458751)), ('p171', 20, ('range', 393216, 458751)), ('p173', 16, ('range', 393216, 458751)), ('p173', 20, ('range', 393216, 458751)), ('p174', 16, ('range', 393216, 458751)), ('p174', 20, ('range', 393216, 458751)), ('p175', 16, ('range', 393216, 458751)), ('p175', 20, ('range', 393216, 458751)), ('p177', 16, ('range', 393216, 458751)), ('p177', 20, ('range', 393216, 458751)), ('p178', 16, ('range', 393216, 458751)), ('p178', 20, ('range', 393216, 458751)), ('p179', 16, ('range', 393216, 458751)), ('p179', 20, ('range', 393216, 458751)), ('p181', 16, ('range', 393216, 458751)), ('p181', 20, ('range', 393216, 458751)), ('p182', 16, ('range', 393216, 458751)), ('p182', 20, ('range', 393216, 458751)), ('p183', 16, ('range', 393216, 458751)), ('p183', 20, ('range', 393216, 458751)), ('p185', 16, ('range', 393216, 458751)), ('p185', 20, ('range', 393216, 458751)), ('p186', 16, ('range', 393216, 458751)), ('p186', 20, ('range', 393216, 458751)), ('p187', 16, ('range', 393216, 458751)), ('p187', 20, ('range', 393216, 458751)), ('p189', 16, ('range', 393216, 458751)), ('p189', 20, ('range', 393216, 458751)), ('p190', 16, ('range', 393216, 458751)), ('p190', 20, ('range', 393216, 458751)), ('p191', 16, ('range', 393216, 458751)), ('p191', 20, ('range', 393216, 458751)), ('p193', 16, ('range', 393216, 458751)), ('p193', 20, ('range', 393216, 458751)), ('p194', 16, ('range', 393216, 458751)), ('p194', 20, ('range', 393216, 458751)), ('p195', 16, ('range', 393216, 458751)), ('p195', 20, ('range', 393216, 458751)), ('p197', 16, ('range', 393216, 458751)), ('p197', 20, ('range', 393216, 458751)), ('p198', 16, ('range', 393216, 458751)), ('p198', 20, ('range', 393216, 458751)), ('p199', 16, ('range', 393216, 458751)), ('p199', 20, ('range', 393216, 458751)), ('p209', 0, ('set', [0, 1, 1, 2])), ('p209', 4, ('set', [0, 1, 2, 3])), ('p209', 48, ('set', [0, 1, 2, -1])), ('p209', 52, ('range', 0, 7)), ('p210', 0, ('set', [0, 1, 1, 2])), ('p210', 4, ('set', [0, 1, 2, 3])), ('p210', 48, ('set', [0, 1, 2, -1])), ('p210', 52, ('range', 0, 7)), ('p211', 0, ('set', [0, 1, 1, 2])), ('p211', 4, ('set', [0, 1, 2, 3])), ('p211', 48, ('set', [0, 1, 2, -1])), ('p211', 52, ('range', 0, 7)), ('p212', 0, ('set', [0, 1, 1, 2])), ('p212', 4, ('set', [0, 1, 2, 3])), ('p212', 48, ('set', [0, 1, 2, -1])), ('p212', 52, ('range', 0, 7)), ('p213', 0, ('set', [0, 1, 1, 2])), ('p213', 4, ('set', [0, 1, 2, 3])), ('p213', 48, ('set', [0, 1, 2, -1])), ('p213', 52, ('range', 0, 7)), ('p214', 0, ('set', [0, 1, 1, 2])), ('p214', 4, ('set', [0, 1, 2, 3])), ('p214', 48, ('set', [0, 1, 2, -1])), ('p214', 52, ('range', 0, 7)), ('p215', 0, ('set', [0, 1, 1, 2])), ('p215', 4, ('set', [0, 1, 2, 3])), ('p215', 48, ('set', [0, 1, 2, -1])), ('p215', 52, ('range', 0, 7)), ('p216', 0, ('set', [0, 1, 1, 2])), ('p216', 4, ('set', [0, 1, 2, 3])), ('p216', 48, ('set', [0, 1, 2, -1])), ('p216', 52, ('range', 0, 7)), ('p217', 0, ('set', [0, 1, 1, 2])), ('p217', 4, ('set', [0, 1, 2, 3])), ('p217', 48, ('set', [0, 1, 2, -1])), ('p217', 52, ('range', 0, 7)), ('p218', 0, ('set', [0, 1, 1, 2])), ('p218', 4, ('set', [0, 1, 2, 3])), ('p218', 48, ('set', [0, 1, 2, -1])), ('p218', 52, ('range', 0, 7)), ('p219', 0, ('set', [0, 1, 1, 2])), ('p219', 4, ('set', [0, 1, 2, 3])), ('p219', 48, ('set', [0, 1, 2, -1])), ('p219', 52, ('range', 0, 7)), ('p220', 0, ('set', [0, 1, 1, 2])), ('p220', 4, ('set', [0, 1, 2, 3])), ('p220', 48, ('set', [0, 1, 2, -1])), ('p220', 52, ('range', 0, 7)), ('p221', 0, ('set', [0, 1, 1, 2])), ('p221', 4, ('set', [0, 1, 2, 3])), ('p221', 48, ('set', [0, 1, 2, -1])), ('p221', 52, ('range', 0, 7)), ('p222', 0, ('set', [0, 1, 1, 2])), ('p222', 4, ('set', [0, 1, 2, 3])), ('p222', 48, ('set', [0, 1, 2, -1])), ('p222', 52, ('range', 0, 7)), ('p223', 0, ('set', [0, 1, 1, 2])), ('p223', 4, ('set', [0, 1, 2, 3])), ('p223', 48, ('set', [0, 1, 2, -1])), ('p223', 52, ('range', 0, 7)), ('p224', 0, ('set', [0, 1, 1, 2])), ('p224', 4, ('set', [0, 1, 2, 3])), ('p224', 48, ('set', [0, 1, 2, -1])), ('p224', 52, ('range', 0, 7)), ('p226', 0, ('set', [0, 1, 1, 2])), ('p226', 4, ('set', [0, 1, 2, 3])), ('p226', 48, ('set', [0, 1, 2, -1])), ('p226', 52, ('range', 0, 7)), ('p227', 0, ('set', [0, 1, 1, 2])), ('p227', 4, ('set', [0, 1, 2, 3])), ('p227', 48, ('set', [0, 1, 2, -1])), ('p227', 52, ('range', 0, 7)), ('p228', 0, ('set', [0, 1, 1, 2])), ('p228', 4, ('set', [0, 1, 2, 3])), ('p228', 48, ('set', [0, 1, 2, -1])), ('p228', 52, ('range', 0, 7)), ('p229', 0, ('set', [0, 1, 1, 2])), ('p229', 4, ('set', [0, 1, 2, 3])), ('p229', 48, ('set', [0, 1, 2, -1])), ('p229', 52, ('range', 0, 7)), ('p230', 0, ('set', [0, 1, 1, 2])), ('p230', 4, ('set', [0, 1, 2, 3])), ('p230', 48, ('set', [0, 1, 2, -1])), ('p230', 52, ('range', 0, 7)), ('p231', 0, ('set', [0, 1, 1, 2])), ('p231', 4, ('set', [0, 1, 2, 3])), ('p231', 48, ('set', [0, 1, 2, -1])), ('p231', 52, ('range', 0, 7)), ('p232', 0, ('set', [0, 1, 1, 2])), ('p232', 4, ('set', [0, 1, 2, 3])), ('p232', 48, ('set', [0, 1, 2, -1])), ('p232', 52, ('range', 0, 7)), ('p233', 0, ('set', [0, 1, 1, 2])), ('p233', 4, ('set', [0, 1, 2, 3])), ('p233', 48, ('set', [0, 1, 2, -1])), ('p233', 52, ('range', 0, 7)), ('p234', 0, ('set', [0, 1, 1, 2])), ('p234', 4, ('set', [0, 1, 2, 3])), ('p234', 48, ('set', [0, 1, 2, -1])), ('p234', 52, ('range', 0, 7)), ('p235', 0, ('set', [0, 1, 1, 2])), ('p235', 4, ('set', [0, 1, 2, 3])), ('p235', 48, ('set', [0, 1, 2, -1])), ('p235', 52, ('range', 0, 7)), ('p236', 0, ('set', [0, 1, 1, 2])), ('p236', 4, ('set', [0, 1, 2, 3])), ('p236', 48, ('set', [0, 1, 2, -1])), ('p236', 52, ('range', 0, 7)), ('p237', 0, ('set', [0, 1, 1, 2])), ('p237', 4, ('set', [0, 1, 2, 3])), ('p237', 48, ('set', [0, 1, 2, -1])), ('p237', 52, ('range', 0, 7)), ('p238', 0, ('set', [0, 1, 1, 2])), ('p238', 4, ('set', [0, 1, 2, 3])), ('p238', 48, ('set', [0, 1, 2, -1])), ('p238', 52, ('range', 0, 7)), ('p239', 0, ('set', [0, 1, 1, 2])), ('p239', 4, ('set', [0, 1, 2, 3])), ('p239', 48, ('set', [0, 1, 2, -1])), ('p239', 52, ('range', 0, 7)), ('p240', 0, ('set', [0, 1, 1, 2])), ('p240', 4, ('set', [0, 1, 2, 3])), ('p240', 48, ('set', [0, 1, 2, -1])), ('p240', 52, ('range', 0, 7)), ('p241', 0, ('set', [0, 1, 1, 2])), ('p241', 4, ('set', [0, 1, 2, 3])), ('p241', 48, ('set', [0, 1, 2, -1])), ('p241', 52, ('range', 0, 7)), ('p243', 0, ('set', [0, 1, 1, 2])), ('p243', 4, ('set', [0, 1, 2, 3])), ('p243', 48, ('set', [0, 1, 2, -1])), ('p243', 52, ('range', 0, 7)), ('p244', 0, ('set', [0, 1, 1, 2])), ('p244', 4, ('set', [0, 1, 2, 3])), ('p244', 48, ('set', [0, 1, 2, -1])), ('p244', 52, ('range', 0, 7)), ('p245', 0, ('set', [0, 1, 1, 2])), ('p245', 4, ('set', [0, 1, 2, 3])), ('p245', 48, ('set', [0, 1, 2, -1])), ('p245', 52, ('range', 0, 7)), ('p246', 0, ('set', [0, 1, 1, 2])), ('p246', 4, ('set', [0, 1, 2, 3])), ('p246', 48, ('set', [0, 1, 2, -1])), ('p246', 52, ('range', 0, 7)), ('p247', 0, ('set', [0, 1, 1, 2])), ('p247', 4, ('set', [0, 1, 2, 3])), ('p247', 48, ('set', [0, 1, 2, -1])), ('p247', 52, ('range', 0, 7)), ('p248', 0, ('set', [0, 1, 1, 2])), ('p248', 4, ('set', [0, 1, 2, 3])), ('p248', 48, ('set', [0, 1, 2, -1])), ('p248', 52, ('range', 0, 7)), ('p249', 0, ('set', [0, 1, 1, 2])), ('p249', 4, ('set', [0, 1, 2, 3])), ('p249', 48, ('set', [0, 1, 2, -1])), ('p249', 52, ('range', 0, 7)), ('p250', 0, ('set', [0, 1, 1, 2])), ('p250', 4, ('set', [0, 1, 2, 3])), ('p250', 48, ('set', [0, 1, 2, -1])), ('p250', 52, ('range', 0, 7)), ('p251', 0, ('set', [0, 1, 1, 2])), ('p251', 4, ('set', [0, 1, 2, 3])), ('p251', 48, ('set', [0, 1, 2, -1])), ('p251', 52, ('range', 0, 7)), ('p252', 0, ('set', [0, 1, 1, 2])), ('p252', 4, ('set', [0, 1, 2, 3])), ('p252', 48, ('set', [0, 1, 2, -1])), ('p252', 52, ('range', 0, 7)), ('p253', 0, ('set', [0, 1, 1, 2])), ('p253', 4, ('set', [0, 1, 2, 3])), ('p253', 48, ('set', [0, 1, 2, -1])), ('p253', 52, ('range', 0, 7)), ('p254', 0, ('set', [0, 1, 1, 2])), ('p254', 4, ('set', [0, 1, 2, 3])), ('p254', 48, ('set', [0, 1, 2, -1])), ('p254', 52, ('range', 0, 7)), ('p255', 0, ('set', [0, 1, 1, 2])), ('p255', 4, ('set', [0, 1, 2, 3])), ('p255', 48, ('set', [0, 1, 2, -1])), ('p255', 52, ('range', 0, 7)), ('p256', 0, ('set', [0, 1, 1, 2])), ('p256', 4, ('set', [0, 1, 2, 3])), ('p256', 48, ('set', [0, 1, 2, -1])), ('p256', 52, ('range', 0, 7)), ('p257', 0, ('set', [0, 1, 1, 2])), ('p257', 4, ('set', [0, 1, 2, 3])), ('p257', 48, ('set', [0, 1, 2, -1])), ('p257', 52, ('range', 0, 7)), ('p258', 0, ('set', [0, 1, 1, 2])), ('p258', 4, ('set', [0, 1, 2, 3])), ('p258', 48, ('set', [0, 1, 2, -1])), ('p258', 52, ('range', 0, 7)), ('p260', 0, ('set', [0, 1, 1, 2])), ('p260', 4, ('set', [0, 1, 2, 3])), ('p260', 48, ('set', [0, 1, 2, -1])), ('p260', 52, ('range', 0, 7)), ('p261', 0, ('set', [0, 1, 1, 2])), ('p261', 4, ('set', [0, 1, 2, 3])), ('p261', 48, ('set', [0, 1, 2, -1])), ('p261', 52, ('range', 0, 7)), ('p262', 0, ('set', [0, 1, 1, 2])), ('p262', 4, ('set', [0, 1, 2, 3])), ('p262', 48, ('set', [0, 1, 2, -1])), ('p262', 52, ('range', 0, 7)), ('p263', 0, ('set', [0, 1, 1, 2])), ('p263', 4, ('set', [0, 1, 2, 3])), ('p263', 48, ('set', [0, 1, 2, -1])), ('p263', 52, ('range', 0, 7)), ('p264', 0, ('set', [0, 1, 1, 2])), ('p264', 4, ('set', [0, 1, 2, 3])), ('p264', 48, ('set', [0, 1, 2, -1])), ('p264', 52, ('range', 0, 7)), ('p265', 0, ('set', [0, 1, 1, 2])), ('p265', 4, ('set', [0, 1, 2, 3])), ('p265', 48, ('set', [0, 1, 2, -1])), ('p265', 52, ('range', 0, 7)), ('p266', 0, ('set', [0, 1, 1, 2])), ('p266', 4, ('set', [0, 1, 2, 3])), ('p266', 48, ('set', [0, 1, 2, -1])), ('p266', 52, ('range', 0, 7)), ('p267', 0, ('set', [0, 1, 1, 2])), ('p267', 4, ('set', [0, 1, 2, 3])), ('p267', 48, ('set', [0, 1, 2, -1])), ('p267', 52, ('range', 0, 7)), ('p268', 0, ('set', [0, 1, 1, 2])), ('p268', 4, ('set', [0, 1, 2, 3])), ('p268', 48, ('set', [0, 1, 2, -1])), ('p268', 52, ('range', 0, 7)), ('p269', 0, ('set', [0, 1, 1, 2])), ('p269', 4, ('set', [0, 1, 2, 3])), ('p269', 48, ('set', [0, 1, 2, -1])), ('p269', 52, ('range', 0, 7)), ('p270', 0, ('set', [0, 1, 1, 2])), ('p270', 4, ('set', [0, 1, 2, 3])), ('p270', 48, ('set', [0, 1, 2, -1])), ('p270', 52, ('range', 0, 7)), ('p271', 0, ('set', [0, 1, 1, 2])), ('p271', 4, ('set', [0, 1, 2, 3])), ('p271', 48, ('set', [0, 1, 2, -1])), ('p271', 52, ('range', 0, 7)), ('p272', 0, ('set', [0, 1, 1, 2])), ('p272', 4, ('set', [0, 1, 2, 3])), ('p272', 48, ('set', [0, 1, 2, -1])), ('p272', 52, ('range', 0, 7)), ('p273', 0, ('set', [0, 1, 1, 2])), ('p273', 4, ('set', [0, 1, 2, 3])), ('p273', 48, ('set', [0, 1, 2, -1])), ('p273', 52, ('range', 0, 7)), ('p274', 0, ('set', [0, 1, 1, 2])), ('p274', 4, ('set', [0, 1, 2, 3])), ('p274', 48, ('set', [0, 1, 2, -1])), ('p274', 52, ('range', 0, 7)), ('p275', 0, ('set', [0, 1, 1, 2])), ('p275', 4, ('set', [0, 1, 2, 3])), ('p275', 48, ('set', [0, 1, 2, -1])), ('p275', 52, ('range', 0, 7)), ('p287', 4, ('set', [415, 416, 531, 256, 257])), ('p288', 36, ('set', [0, 0, 5, 10, 100]))]
FILLS = [('p276', 'i//3')]
ARGRANGE = []
SLOTBASE = []
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(2, [0])]
VTAT = ('m', 144)
VTATHALF = []
VTIMAGE = []
ALIAS = []
ANSWERBY = []
ANSWERWIN = []
CDECL = False
TIB = False
ARGEQ = []
SEQ = [(0, 6, None), (3, 16, None), (4, 16, None), (5, 16, None), (6, 48, None), (7, 40, None), (8, 8, None), (9, 6, None), (10, 4, None), (19, 2, None)]
ANSRANGE = [(0, 0, 6), (1, 0, 1), (3, 0, 1), (4, -1, 7), (5, 0, 5), (6, 0, 1), (7, 0, 65535), (8, 0, 1), (9, 1, 3), (10, 0, 1), (11, 0, 7), (12, 0, 1), (14, 0, 1), (16, -1, -1), (19, 0, 100)]
CELLTHIS = []
VTALSO = [('p288', 0), ('p169', 0), ('p170', 0), ('p171', 0), ('p173', 0), ('p174', 0), ('p175', 0), ('p177', 0), ('p178', 0), ('p179', 0), ('p181', 0), ('p182', 0), ('p183', 0), ('p185', 0), ('p186', 0), ('p187', 0), ('p189', 0), ('p190', 0), ('p191', 0), ('p193', 0), ('p194', 0), ('p195', 0), ('p197', 0), ('p198', 0), ('p199', 0), ('p201', 0), ('p202', 0), ('p203', 0), ('p205', 0), ('p206', 0), ('p207', 0)]
SPARSE = [('@', '0xe3a390', 28396, 8, [(0, 4), (8, 4), (24, 4), (80, 4), (116, 32), (560, 32), (592, 32), (1032, 4), (1092, 12), (1960, 4), (2024, 4), (2372, 12), (2624, 8), (28140, 4), (28192, 4)]), (None, '0xc061e0', 28396, 8, [(0, 4), (116, 32)])]
GTAB = []
RECORD = [0, 1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 7]
RECORDORDER = True
RECORDEDX = False
ROWSPLIT = {}
LINKS = []
LINKHI = []
OUTARG = [(2, 0, ('set', [-1, 0, 1, 2])), (2, 1, ('set', [0, 1, 2, 3, 4, 5, 6, 7]))]
CDECLSTUBS = []
STUBWRITE = []
STUBCOPY = []
STACKCLEAN = 0
STOPSTUB = None
INITCALLS = []
MAXINSNS = 6000000
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
    rng = random.Random(3362)
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

# sweep_gen: ["--item", "3362", "obj", "Army::find_target@006f69b0", "--name", "army_find_target_run", "--win", "0x98", "--own", "0", "--ret", "none", "--rows", "2500", "--max-insns", "6000000", "--cell-win", "0xc061e8:0x840", "--cell-win", "0xc061ec:0x600", "--cell-win", "0xc061b8:0x20", "--cell-win", "0xc06188:0x138", "--cell-win", "0xc0618c:0x210", "--sparse", "@0xe3a390:0x6eec:8:0x0,0x4/0x8,0x4/0x18,0x4/0x50,0x4/0x74,0x20/0x230,0x20/0x250,0x20/0x408,0x4/0x444,0xc/0x7a8,0x4/0x7e8,0x4/0x944,0xc/0xa40,0x8/0x6dec,0x4/0x6e20,0x4", "--sparse", "0xc061e0:0x6eec:8:0,4/0x74,0x20", "--ptrtab", "0xc09970+0x1c*0:3:0x70", "--ptrtab", "0xc09970+0x1c*1:3:0x70", "--ptrtab", "0xc09970+0x1c*2:3:0x70", "--ptrtab", "0xc09970+0x1c*3:3:0x70", "--ptrtab", "0xc09970+0x1c*4:3:0x70", "--ptrtab", "0xc09970+0x1c*5:3:0x70", "--ptrtab", "0xc09970+0x1c*6:3:0x70", "--ptrtab", "0xc09970+0x1c*7:3:0x70", "--ptrtab", "c4+0x14+0x1c*0:3:0x40:vt", "--ptrtab", "c4+0x14+0x1c*1:3:0x40:vt", "--ptrtab", "c4+0x14+0x1c*2:3:0x40:vt", "--ptrtab", "c4+0x14+0x1c*3:3:0x40:vt", "--ptrtab", "c4+0x14+0x1c*4:3:0x40:vt", "--ptrtab", "c4+0x14+0x1c*5:3:0x40:vt", "--ptrtab", "c4+0x14+0x1c*6:3:0x40:vt", "--ptrtab", "c4+0x14+0x1c*7:3:0x40:vt", "--ptrtab", "0xc0ab84+0x1c*0:3:0x40:vt", "--ptrtab", "0xc0ab84+0x1c*1:3:0x40:vt", "--ptrtab", "0xc09710+0x1c*0:16:0x50", "--ptrtab", "0xc09710+0x1c*1:16:0x50", "--ptrtab", "0xc09710+0x1c*2:16:0x50", "--ptrtab", "0xc09710+0x1c*3:16:0x50", "--place", "0xcae5fc:0:0x400", "--place", "0xe3a390+0x6eec*0+0x6eb8:0:0x100", "--place", "0xe3a390+0x6eec*1+0x6eb8:0:0x100", "--place", "0xe3a390+0x6eec*2+0x6eb8:0:0x100", "--place", "0xe3a390+0x6eec*3+0x6eb8:0:0x100", "--place", "0xe3a390+0x6eec*4+0x6eb8:0:0x100", "--place", "0xe3a390+0x6eec*5+0x6eb8:0:0x100", "--place", "0xe3a390+0x6eec*6+0x6eb8:0:0x100", "--place", "0xe3a390+0x6eec*7+0x6eb8:0:0x100", "--place", "c3+0x134:0:0x3000", "--place", "s17:0:0x100", "--place", "p286+0x18:0:0x20", "--place", "s18:0:0x100", "--fill", "p276:i//3", "--wbias", "m+0:0,0x10000,0x20000,0x30000,0x40000,0x50000,0x60000,0x70000", "--wbias", "m+8:0,4,8,32,60,1,2", "--wbias", "m+0x14:0..12", "--wbias", "m+0x18:0..9", "--wbias", "m+0x24:0,0,0,1", "--wbias", "m+0x94:0..3", "--wbias", "m+0x30:0,1,2,-1", "--wbias", "m+0x34:0..7", "--wbias", "m+0x38:0..65535", "--wbias", "m+0x3c:0..65535", "--wbias", "p0+0:3,3,3,7,0x103,0x303,0x703,0x107,6,1,2,11", "--wbias", "p1+0:0", "--wbias", "p2+0:0..30", "--wbias", "p3+0:-1,0,1,2,3,4,5", "--wbias", "p4+*:0,1,2,2", "--wbias", "p5+*:0,1,1", "--wbias", "p6+*:0,0,0,1", "--wbias", "p7+0:0..2", "--wbias", "p8+*:0,0,1", "--wbias", "p9+0:0,0x80,0xff,0x100,0x101,0x200", "--wbias", "p10+0:0,0,1", "--wbias", "p11+0:0..24", "--wbias", "p11+8:0..40", "--wbias", "p12+0:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p12+4:-1,0,1,2,3,4,5,6,7", "--wbias", "p13+0:-1,0,1", "--wbias", "p14+0:0,1", "--wbias", "p120+0:0,1,2,3,3,3", "--wbias", "p121+*:0,1,2,2", "--wbias", "p15+0:3,3,3,7,0x103,0x303,0x703,0x107,6,1,2,11", "--wbias", "p16+0:1", "--wbias", "p17+0:0..30", "--wbias", "p18+0:-1,0,1,2,3,4,5", "--wbias", "p19+*:0,1,2,2", "--wbias", "p20+*:0,1,1", "--wbias", "p21+*:0,0,0,1", "--wbias", "p22+0:0..2", "--wbias", "p23+*:0,0,1", "--wbias", "p24+0:0,0x80,0xff,0x100,0x101,0x200", "--wbias", "p25+0:0,0,1", "--wbias", "p26+0:0..24", "--wbias", "p26+8:0..40", "--wbias", "p27+0:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p27+4:-1,0,1,2,3,4,5,6,7", "--wbias", "p28+0:-1,0,1", "--wbias", "p29+0:0,1", "--wbias", "p122+0:0,1,2,3,3,3", "--wbias", "p123+*:0,1,2,2", "--wbias", "p30+0:3,3,3,7,0x103,0x303,0x703,0x107,6,1,2,11", "--wbias", "p31+0:2", "--wbias", "p32+0:0..30", "--wbias", "p33+0:-1,0,1,2,3,4,5", "--wbias", "p34+*:0,1,2,2", "--wbias", "p35+*:0,1,1", "--wbias", "p36+*:0,0,0,1", "--wbias", "p37+0:0..2", "--wbias", "p38+*:0,0,1", "--wbias", "p39+0:0,0x80,0xff,0x100,0x101,0x200", "--wbias", "p40+0:0,0,1", "--wbias", "p41+0:0..24", "--wbias", "p41+8:0..40", "--wbias", "p42+0:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p42+4:-1,0,1,2,3,4,5,6,7", "--wbias", "p43+0:-1,0,1", "--wbias", "p44+0:0,1", "--wbias", "p124+0:0,1,2,3,3,3", "--wbias", "p125+*:0,1,2,2", "--wbias", "p45+0:3,3,3,7,0x103,0x303,0x703,0x107,6,1,2,11", "--wbias", "p46+0:3", "--wbias", "p47+0:0..30", "--wbias", "p48+0:-1,0,1,2,3,4,5", "--wbias", "p49+*:0,1,2,2", "--wbias", "p50+*:0,1,1", "--wbias", "p51+*:0,0,0,1", "--wbias", "p52+0:0..2", "--wbias", "p53+*:0,0,1", "--wbias", "p54+0:0,0x80,0xff,0x100,0x101,0x200", "--wbias", "p55+0:0,0,1", "--wbias", "p56+0:0..24", "--wbias", "p56+8:0..40", "--wbias", "p57+0:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p57+4:-1,0,1,2,3,4,5,6,7", "--wbias", "p58+0:-1,0,1", "--wbias", "p59+0:0,1", "--wbias", "p126+0:0,1,2,3,3,3", "--wbias", "p127+*:0,1,2,2", "--wbias", "p60+0:3,3,3,7,0x103,0x303,0x703,0x107,6,1,2,11", "--wbias", "p61+0:4", "--wbias", "p62+0:0..30", "--wbias", "p63+0:-1,0,1,2,3,4,5", "--wbias", "p64+*:0,1,2,2", "--wbias", "p65+*:0,1,1", "--wbias", "p66+*:0,0,0,1", "--wbias", "p67+0:0..2", "--wbias", "p68+*:0,0,1", "--wbias", "p69+0:0,0x80,0xff,0x100,0x101,0x200", "--wbias", "p70+0:0,0,1", "--wbias", "p71+0:0..24", "--wbias", "p71+8:0..40", "--wbias", "p72+0:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p72+4:-1,0,1,2,3,4,5,6,7", "--wbias", "p73+0:-1,0,1", "--wbias", "p74+0:0,1", "--wbias", "p128+0:0,1,2,3,3,3", "--wbias", "p129+*:0,1,2,2", "--wbias", "p75+0:3,3,3,7,0x103,0x303,0x703,0x107,6,1,2,11", "--wbias", "p76+0:5", "--wbias", "p77+0:0..30", "--wbias", "p78+0:-1,0,1,2,3,4,5", "--wbias", "p79+*:0,1,2,2", "--wbias", "p80+*:0,1,1", "--wbias", "p81+*:0,0,0,1", "--wbias", "p82+0:0..2", "--wbias", "p83+*:0,0,1", "--wbias", "p84+0:0,0x80,0xff,0x100,0x101,0x200", "--wbias", "p85+0:0,0,1", "--wbias", "p86+0:0..24", "--wbias", "p86+8:0..40", "--wbias", "p87+0:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p87+4:-1,0,1,2,3,4,5,6,7", "--wbias", "p88+0:-1,0,1", "--wbias", "p89+0:0,1", "--wbias", "p130+0:0,1,2,3,3,3", "--wbias", "p131+*:0,1,2,2", "--wbias", "p90+0:3,3,3,7,0x103,0x303,0x703,0x107,6,1,2,11", "--wbias", "p91+0:6", "--wbias", "p92+0:0..30", "--wbias", "p93+0:-1,0,1,2,3,4,5", "--wbias", "p94+*:0,1,2,2", "--wbias", "p95+*:0,1,1", "--wbias", "p96+*:0,0,0,1", "--wbias", "p97+0:0..2", "--wbias", "p98+*:0,0,1", "--wbias", "p99+0:0,0x80,0xff,0x100,0x101,0x200", "--wbias", "p100+0:0,0,1", "--wbias", "p101+0:0..24", "--wbias", "p101+8:0..40", "--wbias", "p102+0:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p102+4:-1,0,1,2,3,4,5,6,7", "--wbias", "p103+0:-1,0,1", "--wbias", "p104+0:0,1", "--wbias", "p132+0:0,1,2,3,3,3", "--wbias", "p133+*:0,1,2,2", "--wbias", "p105+0:3,3,3,7,0x103,0x303,0x703,0x107,6,1,2,11", "--wbias", "p106+0:7", "--wbias", "p107+0:0..30", "--wbias", "p108+0:-1,0,1,2,3,4,5", "--wbias", "p109+*:0,1,2,2", "--wbias", "p110+*:0,1,1", "--wbias", "p111+*:0,0,0,1", "--wbias", "p112+0:0..2", "--wbias", "p113+*:0,0,1", "--wbias", "p114+0:0,0x80,0xff,0x100,0x101,0x200", "--wbias", "p115+0:0,0,1", "--wbias", "p116+0:0..24", "--wbias", "p116+8:0..40", "--wbias", "p117+0:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p117+4:-1,0,1,2,3,4,5,6,7", "--wbias", "p118+0:-1,0,1", "--wbias", "p119+0:0,1", "--wbias", "p134+0:0,1,2,3,3,3", "--wbias", "p135+*:0,1,2,2", "--wbias", "c0+0x820:0,4,0x1000,0x10000,0x20000,0x21000,0x20004,0x1004", "--wbias", "c0+0x28:0,0x1000000,0x2000000,0x3000000,0x4000000,0x5000000", "--wbias", "c0+1628:0..7", "--wbias", "c0+1660:0..7", "--wbias", "c0+1632:0..7", "--wbias", "c0+1664:0..7", "--wbias", "c0+1636:0..7", "--wbias", "c0+1668:0..7", "--wbias", "c0+1640:0..7", "--wbias", "c0+1672:0..7", "--wbias", "c0+1644:0..7", "--wbias", "c0+1676:0..7", "--wbias", "c0+1648:0..7", "--wbias", "c0+1680:0..7", "--wbias", "c0+1652:0..7", "--wbias", "c0+1684:0..7", "--wbias", "c0+1656:0..7", "--wbias", "c0+1688:0..7", "--wbias", "c1+0x24:0,1,2,2,2,3", "--wbias", "c1+0x550:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "c3+0:1..4", "--wbias", "c3+4:1..4", "--wbias", "p137+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p137+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p137+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p137+0x5c:0,0x10000", "--wbias", "p137+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p137+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p137+0x68:0,0,1,2,4,8,3", "--wbias", "p137+0xc:0..65535", "--wbias", "p137+0x10:0..65535", "--wbias", "p138+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p138+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p138+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p138+0x5c:0,0x10000", "--wbias", "p138+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p138+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p138+0x68:0,0,1,2,4,8,3", "--wbias", "p138+0xc:0..65535", "--wbias", "p138+0x10:0..65535", "--wbias", "p139+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p139+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p139+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p139+0x5c:0,0x10000", "--wbias", "p139+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p139+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p139+0x68:0,0,1,2,4,8,3", "--wbias", "p139+0xc:0..65535", "--wbias", "p139+0x10:0..65535", "--wbias", "p141+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p141+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p141+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p141+0x5c:0,0x10000", "--wbias", "p141+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p141+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p141+0x68:0,0,1,2,4,8,3", "--wbias", "p141+0xc:0..65535", "--wbias", "p141+0x10:0..65535", "--wbias", "p142+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p142+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p142+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p142+0x5c:0,0x10000", "--wbias", "p142+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p142+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p142+0x68:0,0,1,2,4,8,3", "--wbias", "p142+0xc:0..65535", "--wbias", "p142+0x10:0..65535", "--wbias", "p143+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p143+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p143+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p143+0x5c:0,0x10000", "--wbias", "p143+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p143+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p143+0x68:0,0,1,2,4,8,3", "--wbias", "p143+0xc:0..65535", "--wbias", "p143+0x10:0..65535", "--wbias", "p145+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p145+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p145+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p145+0x5c:0,0x10000", "--wbias", "p145+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p145+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p145+0x68:0,0,1,2,4,8,3", "--wbias", "p145+0xc:0..65535", "--wbias", "p145+0x10:0..65535", "--wbias", "p146+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p146+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p146+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p146+0x5c:0,0x10000", "--wbias", "p146+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p146+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p146+0x68:0,0,1,2,4,8,3", "--wbias", "p146+0xc:0..65535", "--wbias", "p146+0x10:0..65535", "--wbias", "p147+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p147+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p147+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p147+0x5c:0,0x10000", "--wbias", "p147+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p147+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p147+0x68:0,0,1,2,4,8,3", "--wbias", "p147+0xc:0..65535", "--wbias", "p147+0x10:0..65535", "--wbias", "p149+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p149+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p149+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p149+0x5c:0,0x10000", "--wbias", "p149+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p149+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p149+0x68:0,0,1,2,4,8,3", "--wbias", "p149+0xc:0..65535", "--wbias", "p149+0x10:0..65535", "--wbias", "p150+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p150+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p150+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p150+0x5c:0,0x10000", "--wbias", "p150+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p150+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p150+0x68:0,0,1,2,4,8,3", "--wbias", "p150+0xc:0..65535", "--wbias", "p150+0x10:0..65535", "--wbias", "p151+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p151+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p151+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p151+0x5c:0,0x10000", "--wbias", "p151+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p151+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p151+0x68:0,0,1,2,4,8,3", "--wbias", "p151+0xc:0..65535", "--wbias", "p151+0x10:0..65535", "--wbias", "p153+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p153+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p153+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p153+0x5c:0,0x10000", "--wbias", "p153+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p153+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p153+0x68:0,0,1,2,4,8,3", "--wbias", "p153+0xc:0..65535", "--wbias", "p153+0x10:0..65535", "--wbias", "p154+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p154+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p154+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p154+0x5c:0,0x10000", "--wbias", "p154+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p154+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p154+0x68:0,0,1,2,4,8,3", "--wbias", "p154+0xc:0..65535", "--wbias", "p154+0x10:0..65535", "--wbias", "p155+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p155+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p155+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p155+0x5c:0,0x10000", "--wbias", "p155+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p155+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p155+0x68:0,0,1,2,4,8,3", "--wbias", "p155+0xc:0..65535", "--wbias", "p155+0x10:0..65535", "--wbias", "p157+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p157+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p157+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p157+0x5c:0,0x10000", "--wbias", "p157+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p157+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p157+0x68:0,0,1,2,4,8,3", "--wbias", "p157+0xc:0..65535", "--wbias", "p157+0x10:0..65535", "--wbias", "p158+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p158+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p158+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p158+0x5c:0,0x10000", "--wbias", "p158+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p158+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p158+0x68:0,0,1,2,4,8,3", "--wbias", "p158+0xc:0..65535", "--wbias", "p158+0x10:0..65535", "--wbias", "p159+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p159+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p159+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p159+0x5c:0,0x10000", "--wbias", "p159+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p159+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p159+0x68:0,0,1,2,4,8,3", "--wbias", "p159+0xc:0..65535", "--wbias", "p159+0x10:0..65535", "--wbias", "p161+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p161+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p161+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p161+0x5c:0,0x10000", "--wbias", "p161+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p161+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p161+0x68:0,0,1,2,4,8,3", "--wbias", "p161+0xc:0..65535", "--wbias", "p161+0x10:0..65535", "--wbias", "p162+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p162+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p162+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p162+0x5c:0,0x10000", "--wbias", "p162+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p162+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p162+0x68:0,0,1,2,4,8,3", "--wbias", "p162+0xc:0..65535", "--wbias", "p162+0x10:0..65535", "--wbias", "p163+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p163+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p163+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p163+0x5c:0,0x10000", "--wbias", "p163+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p163+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p163+0x68:0,0,1,2,4,8,3", "--wbias", "p163+0xc:0..65535", "--wbias", "p163+0x10:0..65535", "--wbias", "p165+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p165+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p165+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p165+0x5c:0,0x10000", "--wbias", "p165+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p165+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p165+0x68:0,0,1,2,4,8,3", "--wbias", "p165+0xc:0..65535", "--wbias", "p165+0x10:0..65535", "--wbias", "p166+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p166+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p166+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p166+0x5c:0,0x10000", "--wbias", "p166+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p166+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p166+0x68:0,0,1,2,4,8,3", "--wbias", "p166+0xc:0..65535", "--wbias", "p166+0x10:0..65535", "--wbias", "p167+4:0,1,1,3,0x11,0x13,0x2001,0x2003,0x2011,0", "--wbias", "p167+8:0x0,0x40000,0x80000,0x200000,0x3c0000,0x1,0x40001,0x80001,0x200001,0x3c0001,0x2,0x40002,0x80002,0x200002,0x3c0002", "--wbias", "p167+0x14:0,900,1800,2700,3600,4500,5400,6300,7200,8100,9000,9900,10800,11700,12600,13500,14400,15300,16200,17100,18000,18900,19800,20700,21600,22500,23400,24300,25200,26100", "--wbias", "p167+0x5c:0,0x10000", "--wbias", "p167+0x60:0,1,2,3,0x10000,0x10001,0x10002,0x10003", "--wbias", "p167+0x64:0,0x100,0x200,0x400,0x800,0x300,0xf00", "--wbias", "p167+0x68:0,0,1,2,4,8,3", "--wbias", "p167+0xc:0..65535", "--wbias", "p167+0x10:0..65535", "--wbias", "p277+0xe8:0x63187,0x63186,0x63185,0x63184,0x63183,0x63182,0x63181", "--wbias", "p278+0xe8:0x63187,0x63186,0x63185,0x63184,0x63183,0x63182,0x63181", "--wbias", "p279+0xe8:0x63187,0x63186,0x63185,0x63184,0x63183,0x63182,0x63181", "--wbias", "p280+0xe8:0x63187,0x63186,0x63185,0x63184,0x63183,0x63182,0x63181", "--wbias", "p281+0xe8:0x63187,0x63186,0x63185,0x63184,0x63183,0x63182,0x63181", "--wbias", "p282+0xe8:0x63187,0x63186,0x63185,0x63184,0x63183,0x63182,0x63181", "--wbias", "p283+0xe8:0x63187,0x63186,0x63185,0x63184,0x63183,0x63182,0x63181", "--wbias", "p284+0xe8:0x63187,0x63186,0x63185,0x63184,0x63183,0x63182,0x63181", "--wbias", "p169+0x10:0x60000..0x6ffff", "--wbias", "p169+0x14:0x60000..0x6ffff", "--wbias", "p170+0x10:0x60000..0x6ffff", "--wbias", "p170+0x14:0x60000..0x6ffff", "--wbias", "p171+0x10:0x60000..0x6ffff", "--wbias", "p171+0x14:0x60000..0x6ffff", "--wbias", "p173+0x10:0x60000..0x6ffff", "--wbias", "p173+0x14:0x60000..0x6ffff", "--wbias", "p174+0x10:0x60000..0x6ffff", "--wbias", "p174+0x14:0x60000..0x6ffff", "--wbias", "p175+0x10:0x60000..0x6ffff", "--wbias", "p175+0x14:0x60000..0x6ffff", "--wbias", "p177+0x10:0x60000..0x6ffff", "--wbias", "p177+0x14:0x60000..0x6ffff", "--wbias", "p178+0x10:0x60000..0x6ffff", "--wbias", "p178+0x14:0x60000..0x6ffff", "--wbias", "p179+0x10:0x60000..0x6ffff", "--wbias", "p179+0x14:0x60000..0x6ffff", "--wbias", "p181+0x10:0x60000..0x6ffff", "--wbias", "p181+0x14:0x60000..0x6ffff", "--wbias", "p182+0x10:0x60000..0x6ffff", "--wbias", "p182+0x14:0x60000..0x6ffff", "--wbias", "p183+0x10:0x60000..0x6ffff", "--wbias", "p183+0x14:0x60000..0x6ffff", "--wbias", "p185+0x10:0x60000..0x6ffff", "--wbias", "p185+0x14:0x60000..0x6ffff", "--wbias", "p186+0x10:0x60000..0x6ffff", "--wbias", "p186+0x14:0x60000..0x6ffff", "--wbias", "p187+0x10:0x60000..0x6ffff", "--wbias", "p187+0x14:0x60000..0x6ffff", "--wbias", "p189+0x10:0x60000..0x6ffff", "--wbias", "p189+0x14:0x60000..0x6ffff", "--wbias", "p190+0x10:0x60000..0x6ffff", "--wbias", "p190+0x14:0x60000..0x6ffff", "--wbias", "p191+0x10:0x60000..0x6ffff", "--wbias", "p191+0x14:0x60000..0x6ffff", "--wbias", "p193+0x10:0x60000..0x6ffff", "--wbias", "p193+0x14:0x60000..0x6ffff", "--wbias", "p194+0x10:0x60000..0x6ffff", "--wbias", "p194+0x14:0x60000..0x6ffff", "--wbias", "p195+0x10:0x60000..0x6ffff", "--wbias", "p195+0x14:0x60000..0x6ffff", "--wbias", "p197+0x10:0x60000..0x6ffff", "--wbias", "p197+0x14:0x60000..0x6ffff", "--wbias", "p198+0x10:0x60000..0x6ffff", "--wbias", "p198+0x14:0x60000..0x6ffff", "--wbias", "p199+0x10:0x60000..0x6ffff", "--wbias", "p199+0x14:0x60000..0x6ffff", "--wbias", "p209+0:0,1,1,2", "--wbias", "p209+4:0,1,2,3", "--wbias", "p209+0x30:0,1,2,-1", "--wbias", "p209+0x34:0..7", "--wbias", "p210+0:0,1,1,2", "--wbias", "p210+4:0,1,2,3", "--wbias", "p210+0x30:0,1,2,-1", "--wbias", "p210+0x34:0..7", "--wbias", "p211+0:0,1,1,2", "--wbias", "p211+4:0,1,2,3", "--wbias", "p211+0x30:0,1,2,-1", "--wbias", "p211+0x34:0..7", "--wbias", "p212+0:0,1,1,2", "--wbias", "p212+4:0,1,2,3", "--wbias", "p212+0x30:0,1,2,-1", "--wbias", "p212+0x34:0..7", "--wbias", "p213+0:0,1,1,2", "--wbias", "p213+4:0,1,2,3", "--wbias", "p213+0x30:0,1,2,-1", "--wbias", "p213+0x34:0..7", "--wbias", "p214+0:0,1,1,2", "--wbias", "p214+4:0,1,2,3", "--wbias", "p214+0x30:0,1,2,-1", "--wbias", "p214+0x34:0..7", "--wbias", "p215+0:0,1,1,2", "--wbias", "p215+4:0,1,2,3", "--wbias", "p215+0x30:0,1,2,-1", "--wbias", "p215+0x34:0..7", "--wbias", "p216+0:0,1,1,2", "--wbias", "p216+4:0,1,2,3", "--wbias", "p216+0x30:0,1,2,-1", "--wbias", "p216+0x34:0..7", "--wbias", "p217+0:0,1,1,2", "--wbias", "p217+4:0,1,2,3", "--wbias", "p217+0x30:0,1,2,-1", "--wbias", "p217+0x34:0..7", "--wbias", "p218+0:0,1,1,2", "--wbias", "p218+4:0,1,2,3", "--wbias", "p218+0x30:0,1,2,-1", "--wbias", "p218+0x34:0..7", "--wbias", "p219+0:0,1,1,2", "--wbias", "p219+4:0,1,2,3", "--wbias", "p219+0x30:0,1,2,-1", "--wbias", "p219+0x34:0..7", "--wbias", "p220+0:0,1,1,2", "--wbias", "p220+4:0,1,2,3", "--wbias", "p220+0x30:0,1,2,-1", "--wbias", "p220+0x34:0..7", "--wbias", "p221+0:0,1,1,2", "--wbias", "p221+4:0,1,2,3", "--wbias", "p221+0x30:0,1,2,-1", "--wbias", "p221+0x34:0..7", "--wbias", "p222+0:0,1,1,2", "--wbias", "p222+4:0,1,2,3", "--wbias", "p222+0x30:0,1,2,-1", "--wbias", "p222+0x34:0..7", "--wbias", "p223+0:0,1,1,2", "--wbias", "p223+4:0,1,2,3", "--wbias", "p223+0x30:0,1,2,-1", "--wbias", "p223+0x34:0..7", "--wbias", "p224+0:0,1,1,2", "--wbias", "p224+4:0,1,2,3", "--wbias", "p224+0x30:0,1,2,-1", "--wbias", "p224+0x34:0..7", "--wbias", "p226+0:0,1,1,2", "--wbias", "p226+4:0,1,2,3", "--wbias", "p226+0x30:0,1,2,-1", "--wbias", "p226+0x34:0..7", "--wbias", "p227+0:0,1,1,2", "--wbias", "p227+4:0,1,2,3", "--wbias", "p227+0x30:0,1,2,-1", "--wbias", "p227+0x34:0..7", "--wbias", "p228+0:0,1,1,2", "--wbias", "p228+4:0,1,2,3", "--wbias", "p228+0x30:0,1,2,-1", "--wbias", "p228+0x34:0..7", "--wbias", "p229+0:0,1,1,2", "--wbias", "p229+4:0,1,2,3", "--wbias", "p229+0x30:0,1,2,-1", "--wbias", "p229+0x34:0..7", "--wbias", "p230+0:0,1,1,2", "--wbias", "p230+4:0,1,2,3", "--wbias", "p230+0x30:0,1,2,-1", "--wbias", "p230+0x34:0..7", "--wbias", "p231+0:0,1,1,2", "--wbias", "p231+4:0,1,2,3", "--wbias", "p231+0x30:0,1,2,-1", "--wbias", "p231+0x34:0..7", "--wbias", "p232+0:0,1,1,2", "--wbias", "p232+4:0,1,2,3", "--wbias", "p232+0x30:0,1,2,-1", "--wbias", "p232+0x34:0..7", "--wbias", "p233+0:0,1,1,2", "--wbias", "p233+4:0,1,2,3", "--wbias", "p233+0x30:0,1,2,-1", "--wbias", "p233+0x34:0..7", "--wbias", "p234+0:0,1,1,2", "--wbias", "p234+4:0,1,2,3", "--wbias", "p234+0x30:0,1,2,-1", "--wbias", "p234+0x34:0..7", "--wbias", "p235+0:0,1,1,2", "--wbias", "p235+4:0,1,2,3", "--wbias", "p235+0x30:0,1,2,-1", "--wbias", "p235+0x34:0..7", "--wbias", "p236+0:0,1,1,2", "--wbias", "p236+4:0,1,2,3", "--wbias", "p236+0x30:0,1,2,-1", "--wbias", "p236+0x34:0..7", "--wbias", "p237+0:0,1,1,2", "--wbias", "p237+4:0,1,2,3", "--wbias", "p237+0x30:0,1,2,-1", "--wbias", "p237+0x34:0..7", "--wbias", "p238+0:0,1,1,2", "--wbias", "p238+4:0,1,2,3", "--wbias", "p238+0x30:0,1,2,-1", "--wbias", "p238+0x34:0..7", "--wbias", "p239+0:0,1,1,2", "--wbias", "p239+4:0,1,2,3", "--wbias", "p239+0x30:0,1,2,-1", "--wbias", "p239+0x34:0..7", "--wbias", "p240+0:0,1,1,2", "--wbias", "p240+4:0,1,2,3", "--wbias", "p240+0x30:0,1,2,-1", "--wbias", "p240+0x34:0..7", "--wbias", "p241+0:0,1,1,2", "--wbias", "p241+4:0,1,2,3", "--wbias", "p241+0x30:0,1,2,-1", "--wbias", "p241+0x34:0..7", "--wbias", "p243+0:0,1,1,2", "--wbias", "p243+4:0,1,2,3", "--wbias", "p243+0x30:0,1,2,-1", "--wbias", "p243+0x34:0..7", "--wbias", "p244+0:0,1,1,2", "--wbias", "p244+4:0,1,2,3", "--wbias", "p244+0x30:0,1,2,-1", "--wbias", "p244+0x34:0..7", "--wbias", "p245+0:0,1,1,2", "--wbias", "p245+4:0,1,2,3", "--wbias", "p245+0x30:0,1,2,-1", "--wbias", "p245+0x34:0..7", "--wbias", "p246+0:0,1,1,2", "--wbias", "p246+4:0,1,2,3", "--wbias", "p246+0x30:0,1,2,-1", "--wbias", "p246+0x34:0..7", "--wbias", "p247+0:0,1,1,2", "--wbias", "p247+4:0,1,2,3", "--wbias", "p247+0x30:0,1,2,-1", "--wbias", "p247+0x34:0..7", "--wbias", "p248+0:0,1,1,2", "--wbias", "p248+4:0,1,2,3", "--wbias", "p248+0x30:0,1,2,-1", "--wbias", "p248+0x34:0..7", "--wbias", "p249+0:0,1,1,2", "--wbias", "p249+4:0,1,2,3", "--wbias", "p249+0x30:0,1,2,-1", "--wbias", "p249+0x34:0..7", "--wbias", "p250+0:0,1,1,2", "--wbias", "p250+4:0,1,2,3", "--wbias", "p250+0x30:0,1,2,-1", "--wbias", "p250+0x34:0..7", "--wbias", "p251+0:0,1,1,2", "--wbias", "p251+4:0,1,2,3", "--wbias", "p251+0x30:0,1,2,-1", "--wbias", "p251+0x34:0..7", "--wbias", "p252+0:0,1,1,2", "--wbias", "p252+4:0,1,2,3", "--wbias", "p252+0x30:0,1,2,-1", "--wbias", "p252+0x34:0..7", "--wbias", "p253+0:0,1,1,2", "--wbias", "p253+4:0,1,2,3", "--wbias", "p253+0x30:0,1,2,-1", "--wbias", "p253+0x34:0..7", "--wbias", "p254+0:0,1,1,2", "--wbias", "p254+4:0,1,2,3", "--wbias", "p254+0x30:0,1,2,-1", "--wbias", "p254+0x34:0..7", "--wbias", "p255+0:0,1,1,2", "--wbias", "p255+4:0,1,2,3", "--wbias", "p255+0x30:0,1,2,-1", "--wbias", "p255+0x34:0..7", "--wbias", "p256+0:0,1,1,2", "--wbias", "p256+4:0,1,2,3", "--wbias", "p256+0x30:0,1,2,-1", "--wbias", "p256+0x34:0..7", "--wbias", "p257+0:0,1,1,2", "--wbias", "p257+4:0,1,2,3", "--wbias", "p257+0x30:0,1,2,-1", "--wbias", "p257+0x34:0..7", "--wbias", "p258+0:0,1,1,2", "--wbias", "p258+4:0,1,2,3", "--wbias", "p258+0x30:0,1,2,-1", "--wbias", "p258+0x34:0..7", "--wbias", "p260+0:0,1,1,2", "--wbias", "p260+4:0,1,2,3", "--wbias", "p260+0x30:0,1,2,-1", "--wbias", "p260+0x34:0..7", "--wbias", "p261+0:0,1,1,2", "--wbias", "p261+4:0,1,2,3", "--wbias", "p261+0x30:0,1,2,-1", "--wbias", "p261+0x34:0..7", "--wbias", "p262+0:0,1,1,2", "--wbias", "p262+4:0,1,2,3", "--wbias", "p262+0x30:0,1,2,-1", "--wbias", "p262+0x34:0..7", "--wbias", "p263+0:0,1,1,2", "--wbias", "p263+4:0,1,2,3", "--wbias", "p263+0x30:0,1,2,-1", "--wbias", "p263+0x34:0..7", "--wbias", "p264+0:0,1,1,2", "--wbias", "p264+4:0,1,2,3", "--wbias", "p264+0x30:0,1,2,-1", "--wbias", "p264+0x34:0..7", "--wbias", "p265+0:0,1,1,2", "--wbias", "p265+4:0,1,2,3", "--wbias", "p265+0x30:0,1,2,-1", "--wbias", "p265+0x34:0..7", "--wbias", "p266+0:0,1,1,2", "--wbias", "p266+4:0,1,2,3", "--wbias", "p266+0x30:0,1,2,-1", "--wbias", "p266+0x34:0..7", "--wbias", "p267+0:0,1,1,2", "--wbias", "p267+4:0,1,2,3", "--wbias", "p267+0x30:0,1,2,-1", "--wbias", "p267+0x34:0..7", "--wbias", "p268+0:0,1,1,2", "--wbias", "p268+4:0,1,2,3", "--wbias", "p268+0x30:0,1,2,-1", "--wbias", "p268+0x34:0..7", "--wbias", "p269+0:0,1,1,2", "--wbias", "p269+4:0,1,2,3", "--wbias", "p269+0x30:0,1,2,-1", "--wbias", "p269+0x34:0..7", "--wbias", "p270+0:0,1,1,2", "--wbias", "p270+4:0,1,2,3", "--wbias", "p270+0x30:0,1,2,-1", "--wbias", "p270+0x34:0..7", "--wbias", "p271+0:0,1,1,2", "--wbias", "p271+4:0,1,2,3", "--wbias", "p271+0x30:0,1,2,-1", "--wbias", "p271+0x34:0..7", "--wbias", "p272+0:0,1,1,2", "--wbias", "p272+4:0,1,2,3", "--wbias", "p272+0x30:0,1,2,-1", "--wbias", "p272+0x34:0..7", "--wbias", "p273+0:0,1,1,2", "--wbias", "p273+4:0,1,2,3", "--wbias", "p273+0x30:0,1,2,-1", "--wbias", "p273+0x34:0..7", "--wbias", "p274+0:0,1,1,2", "--wbias", "p274+4:0,1,2,3", "--wbias", "p274+0x30:0,1,2,-1", "--wbias", "p274+0x34:0..7", "--wbias", "p275+0:0,1,1,2", "--wbias", "p275+4:0,1,2,3", "--wbias", "p275+0x30:0,1,2,-1", "--wbias", "p275+0x34:0..7", "--wbias", "p287+4:0x19f,0x1a0,0x213,0x100,0x101", "--wbias", "p288+0x24:0,0,5,10,100", "--vtat-also", "p288+0", "--vtat", "m+0x90", "--stub", "0x6f9120:2", "--stub", "0x6e33a0:2", "--stub", "0x6eb930:4", "--stub", "0x6d5b90:1", "--stub", "0x6f2e10:1", "--stub", "0x6ec000:0", "--stub", "0x680f90:1", "--stub", "0xa39d70:2", "--stub", "0x62d470:0", "--stub", "0x739340:0", "--stub", "0x7382b0:1", "--stub", "0x5948a0:0", "--stub", "0x6db810:1", "--stub", "0x6f2c90:5", "--stub", "0x6f5cc0:3", "--stub", "0x6f8ea0:0", "--stub", "0x6f9df0:1", "--stub", "slot:0x3c:0", "--stub", "slot:0xac:0", "--stub", "slot:0x11c:1", "--answer-range", "0:0:6", "--seq", "0:6", "--answer-range", "1:0:1", "--answers", "2:0", "--outarg", "2:0:-1,0,1,2", "--outarg", "2:1:0,1,2,3,4,5,6,7", "--answer-range", "3:0:1", "--seq", "3:16", "--answer-range", "4:-1:7", "--seq", "4:16", "--answer-range", "5:0:5", "--seq", "5:16", "--answer-range", "6:0:1", "--seq", "6:48", "--answer-range", "7:0:65535", "--seq", "7:40", "--answer-range", "8:0:1", "--seq", "8:8", "--answer-range", "9:1:3", "--seq", "9:6", "--answer-range", "10:0:1", "--seq", "10:4", "--answer-range", "11:0:7", "--answer-range", "12:0:1", "--answer-range", "14:0:1", "--answer-range", "16:-1:-1", "--answer-range", "19:0:100", "--seq", "19:2", "--record", "0", "--record", "1", "--record", "2", "--record", "3", "--record", "4", "--record", "5", "--record", "6", "--record", "8", "--record", "9", "--record", "10", "--record", "11", "--record", "12", "--record", "13", "--record", "14", "--record", "15", "--record", "16", "--record", "17", "--record", "18", "--record", "19", "--record", "7", "--record-order"]
