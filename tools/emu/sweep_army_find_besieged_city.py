#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_army_find_besieged_city.py — `Army::find_besieged_city@006f58b0` under unicorn (item 3281, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_army_find_besieged_city.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 176-byte object of random bytes (no bias), 0 stack argument(s);
`p` windows in order: p0..p23 sparse table 0 (8 rows × 3 slice(s), row-major), p24..p31 sparse table 1 (8 rows × 1 slice(s), row-major), p32..p215 the place(s), in flag order; the global cell 0x00c06188 points at 16 random bytes; the global cell 0x00c061b8 points at 32 random bytes; the global cell 0x00c0618c points at 32 random bytes; vtable slot +0x3c (0 args) stubbed; callee 0x00680f90 (1 args) stubbed; callee 0x0065ca80 (13 args) stubbed; callee 0x0092d130 (0 args) stubbed; callee 0x006f5cc0 (3 args) stubbed; callee 0x0046cff0 (0 args) run, not hooked; the word at `0xc09970+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p32+0` points at 112 random bytes less `0` (placed); the word at `p32+4` points at 112 random bytes less `0` (placed); the word at `p32+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p36+0` points at 112 random bytes less `0` (placed); the word at `p36+4` points at 112 random bytes less `0` (placed); the word at `p36+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*2` points at 12 random bytes less `0` (placed); the word at `p40+0` points at 112 random bytes less `0` (placed); the word at `p40+4` points at 112 random bytes less `0` (placed); the word at `p40+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*3` points at 12 random bytes less `0` (placed); the word at `p44+0` points at 112 random bytes less `0` (placed); the word at `p44+4` points at 112 random bytes less `0` (placed); the word at `p44+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*4` points at 12 random bytes less `0` (placed); the word at `p48+0` points at 112 random bytes less `0` (placed); the word at `p48+4` points at 112 random bytes less `0` (placed); the word at `p48+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*5` points at 12 random bytes less `0` (placed); the word at `p52+0` points at 112 random bytes less `0` (placed); the word at `p52+4` points at 112 random bytes less `0` (placed); the word at `p52+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*6` points at 12 random bytes less `0` (placed); the word at `p56+0` points at 112 random bytes less `0` (placed); the word at `p56+4` points at 112 random bytes less `0` (placed); the word at `p56+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*7` points at 12 random bytes less `0` (placed); the word at `p60+0` points at 112 random bytes less `0` (placed); the word at `p60+4` points at 112 random bytes less `0` (placed); the word at `p60+8` points at 112 random bytes less `0` (placed); the word at `0xc09710+0x1c*0` points at 64 random bytes less `0` (placed); the word at `p64+0` points at 36 random bytes less `0` (placed); the word at `p64+4` points at 36 random bytes less `0` (placed); the word at `p64+8` points at 36 random bytes less `0` (placed); the word at `p64+12` points at 36 random bytes less `0` (placed); the word at `p64+16` points at 36 random bytes less `0` (placed); the word at `p64+20` points at 36 random bytes less `0` (placed); the word at `p64+24` points at 36 random bytes less `0` (placed); the word at `p64+28` points at 36 random bytes less `0` (placed); the word at `p64+32` points at 36 random bytes less `0` (placed); the word at `p64+36` points at 36 random bytes less `0` (placed); the word at `p64+40` points at 36 random bytes less `0` (placed); the word at `p64+44` points at 36 random bytes less `0` (placed); the word at `p64+48` points at 36 random bytes less `0` (placed); the word at `p64+52` points at 36 random bytes less `0` (placed); the word at `p64+56` points at 36 random bytes less `0` (placed); the word at `p64+60` points at 36 random bytes less `0` (placed); the word at `0xc09710+0x1c*1` points at 64 random bytes less `0` (placed); the word at `p81+0` points at 36 random bytes less `0` (placed); the word at `p81+4` points at 36 random bytes less `0` (placed); the word at `p81+8` points at 36 random bytes less `0` (placed); the word at `p81+12` points at 36 random bytes less `0` (placed); the word at `p81+16` points at 36 random bytes less `0` (placed); the word at `p81+20` points at 36 random bytes less `0` (placed); the word at `p81+24` points at 36 random bytes less `0` (placed); the word at `p81+28` points at 36 random bytes less `0` (placed); the word at `p81+32` points at 36 random bytes less `0` (placed); the word at `p81+36` points at 36 random bytes less `0` (placed); the word at `p81+40` points at 36 random bytes less `0` (placed); the word at `p81+44` points at 36 random bytes less `0` (placed); the word at `p81+48` points at 36 random bytes less `0` (placed); the word at `p81+52` points at 36 random bytes less `0` (placed); the word at `p81+56` points at 36 random bytes less `0` (placed); the word at `p81+60` points at 36 random bytes less `0` (placed); the word at `0xc09710+0x1c*2` points at 64 random bytes less `0` (placed); the word at `p98+0` points at 36 random bytes less `0` (placed); the word at `p98+4` points at 36 random bytes less `0` (placed); the word at `p98+8` points at 36 random bytes less `0` (placed); the word at `p98+12` points at 36 random bytes less `0` (placed); the word at `p98+16` points at 36 random bytes less `0` (placed); the word at `p98+20` points at 36 random bytes less `0` (placed); the word at `p98+24` points at 36 random bytes less `0` (placed); the word at `p98+28` points at 36 random bytes less `0` (placed); the word at `p98+32` points at 36 random bytes less `0` (placed); the word at `p98+36` points at 36 random bytes less `0` (placed); the word at `p98+40` points at 36 random bytes less `0` (placed); the word at `p98+44` points at 36 random bytes less `0` (placed); the word at `p98+48` points at 36 random bytes less `0` (placed); the word at `p98+52` points at 36 random bytes less `0` (placed); the word at `p98+56` points at 36 random bytes less `0` (placed); the word at `p98+60` points at 36 random bytes less `0` (placed); the word at `0xc09710+0x1c*3` points at 64 random bytes less `0` (placed); the word at `p115+0` points at 36 random bytes less `0` (placed); the word at `p115+4` points at 36 random bytes less `0` (placed); the word at `p115+8` points at 36 random bytes less `0` (placed); the word at `p115+12` points at 36 random bytes less `0` (placed); the word at `p115+16` points at 36 random bytes less `0` (placed); the word at `p115+20` points at 36 random bytes less `0` (placed); the word at `p115+24` points at 36 random bytes less `0` (placed); the word at `p115+28` points at 36 random bytes less `0` (placed); the word at `p115+32` points at 36 random bytes less `0` (placed); the word at `p115+36` points at 36 random bytes less `0` (placed); the word at `p115+40` points at 36 random bytes less `0` (placed); the word at `p115+44` points at 36 random bytes less `0` (placed); the word at `p115+48` points at 36 random bytes less `0` (placed); the word at `p115+52` points at 36 random bytes less `0` (placed); the word at `p115+56` points at 36 random bytes less `0` (placed); the word at `p115+60` points at 36 random bytes less `0` (placed); the word at `0xc09710+0x1c*4` points at 64 random bytes less `0` (placed); the word at `p132+0` points at 36 random bytes less `0` (placed); the word at `p132+4` points at 36 random bytes less `0` (placed); the word at `p132+8` points at 36 random bytes less `0` (placed); the word at `p132+12` points at 36 random bytes less `0` (placed); the word at `p132+16` points at 36 random bytes less `0` (placed); the word at `p132+20` points at 36 random bytes less `0` (placed); the word at `p132+24` points at 36 random bytes less `0` (placed); the word at `p132+28` points at 36 random bytes less `0` (placed); the word at `p132+32` points at 36 random bytes less `0` (placed); the word at `p132+36` points at 36 random bytes less `0` (placed); the word at `p132+40` points at 36 random bytes less `0` (placed); the word at `p132+44` points at 36 random bytes less `0` (placed); the word at `p132+48` points at 36 random bytes less `0` (placed); the word at `p132+52` points at 36 random bytes less `0` (placed); the word at `p132+56` points at 36 random bytes less `0` (placed); the word at `p132+60` points at 36 random bytes less `0` (placed); the word at `0xc09710+0x1c*5` points at 64 random bytes less `0` (placed); the word at `p149+0` points at 36 random bytes less `0` (placed); the word at `p149+4` points at 36 random bytes less `0` (placed); the word at `p149+8` points at 36 random bytes less `0` (placed); the word at `p149+12` points at 36 random bytes less `0` (placed); the word at `p149+16` points at 36 random bytes less `0` (placed); the word at `p149+20` points at 36 random bytes less `0` (placed); the word at `p149+24` points at 36 random bytes less `0` (placed); the word at `p149+28` points at 36 random bytes less `0` (placed); the word at `p149+32` points at 36 random bytes less `0` (placed); the word at `p149+36` points at 36 random bytes less `0` (placed); the word at `p149+40` points at 36 random bytes less `0` (placed); the word at `p149+44` points at 36 random bytes less `0` (placed); the word at `p149+48` points at 36 random bytes less `0` (placed); the word at `p149+52` points at 36 random bytes less `0` (placed); the word at `p149+56` points at 36 random bytes less `0` (placed); the word at `p149+60` points at 36 random bytes less `0` (placed); the word at `0xc09710+0x1c*6` points at 64 random bytes less `0` (placed); the word at `p166+0` points at 36 random bytes less `0` (placed); the word at `p166+4` points at 36 random bytes less `0` (placed); the word at `p166+8` points at 36 random bytes less `0` (placed); the word at `p166+12` points at 36 random bytes less `0` (placed); the word at `p166+16` points at 36 random bytes less `0` (placed); the word at `p166+20` points at 36 random bytes less `0` (placed); the word at `p166+24` points at 36 random bytes less `0` (placed); the word at `p166+28` points at 36 random bytes less `0` (placed); the word at `p166+32` points at 36 random bytes less `0` (placed); the word at `p166+36` points at 36 random bytes less `0` (placed); the word at `p166+40` points at 36 random bytes less `0` (placed); the word at `p166+44` points at 36 random bytes less `0` (placed); the word at `p166+48` points at 36 random bytes less `0` (placed); the word at `p166+52` points at 36 random bytes less `0` (placed); the word at `p166+56` points at 36 random bytes less `0` (placed); the word at `p166+60` points at 36 random bytes less `0` (placed); the word at `0xc09710+0x1c*7` points at 64 random bytes less `0` (placed); the word at `p183+0` points at 36 random bytes less `0` (placed); the word at `p183+4` points at 36 random bytes less `0` (placed); the word at `p183+8` points at 36 random bytes less `0` (placed); the word at `p183+12` points at 36 random bytes less `0` (placed); the word at `p183+16` points at 36 random bytes less `0` (placed); the word at `p183+20` points at 36 random bytes less `0` (placed); the word at `p183+24` points at 36 random bytes less `0` (placed); the word at `p183+28` points at 36 random bytes less `0` (placed); the word at `p183+32` points at 36 random bytes less `0` (placed); the word at `p183+36` points at 36 random bytes less `0` (placed); the word at `p183+40` points at 36 random bytes less `0` (placed); the word at `p183+44` points at 36 random bytes less `0` (placed); the word at `p183+48` points at 36 random bytes less `0` (placed); the word at `p183+52` points at 36 random bytes less `0` (placed); the word at `p183+56` points at 36 random bytes less `0` (placed); the word at `p183+60` points at 36 random bytes less `0` (placed); the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p200+0` points at 16 random bytes less `0` (placed); the word at `p200+4` points at 16 random bytes less `0` (placed); the word at `p200+8` points at 16 random bytes less `0` (placed); the word at `p200+12` points at 16 random bytes less `0` (placed); the word at `m+0x98` points at 16 random bytes less `0` (placed); the word at `p205+0` points at 32 random bytes less `0` (placed); the word at `p205+4` points at 32 random bytes less `0` (placed); the word at `p205+8` points at 32 random bytes less `0` (placed); the word at `p205+12` points at 32 random bytes less `0` (placed); the word at `m+0xa0` points at 16 random bytes less `0` (placed); the word at `p210+0` points at 16 random bytes less `0` (placed); the word at `p210+4` points at 16 random bytes less `0` (placed); the word at `p210+8` points at 16 random bytes less `0` (placed); the word at `p210+12` points at 16 random bytes less `0` (placed); the word at `0xcae5fc` points at 1024 random bytes less `0` (placed); stub 1 answered from [0, 1] (90%); stub 2 answered from [-1, 0, 5, -1] (90%); the word at `p206+0x18` laid as the address of `p211` every row; the word at `p207+0x18` laid as the address of `p212` every row; the word at `p208+0x18` laid as the address of `p213` every row; the word at `p209+0x18` laid as the address of `p214` every row; stub 0 answers `p206+j`, the data of the object its `ecx` is in `p201+j` (j < 4), linked at `+0x1c`; stub 1 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0xc, +0x74:0x20, +0x408:0x4 (`p` windows after the pointed buffers, before the places); 8 rows 0x6eec apart at `0xc061e0`, slices +0x74:0x20 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p201+0` as well as at `this`; the stubbed vtable laid at `p202+0` as well as at `this`; the stubbed vtable laid at `p203+0` as well as at `this`; the stubbed vtable laid at `p204+0` as well as at `this`.
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

ENTRY = 0x006f58b0
WIN = 176
OWN = 0
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607880, 16), (12607928, 32), (12607884, 32)]
AT = []
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('slot', 60, 0), ('call', 6819728, 1), ('call', 6670976, 13), ('call', 9621808, 0), ('call', 7298240, 3), ('run', 4640752, 0)]
PLACES = [(None, '0xc09970+0x1c*0', '0', 12), ('p32', '0', '0', 112), ('p32', '4', '0', 112), ('p32', '8', '0', 112), (None, '0xc09970+0x1c*1', '0', 12), ('p36', '0', '0', 112), ('p36', '4', '0', 112), ('p36', '8', '0', 112), (None, '0xc09970+0x1c*2', '0', 12), ('p40', '0', '0', 112), ('p40', '4', '0', 112), ('p40', '8', '0', 112), (None, '0xc09970+0x1c*3', '0', 12), ('p44', '0', '0', 112), ('p44', '4', '0', 112), ('p44', '8', '0', 112), (None, '0xc09970+0x1c*4', '0', 12), ('p48', '0', '0', 112), ('p48', '4', '0', 112), ('p48', '8', '0', 112), (None, '0xc09970+0x1c*5', '0', 12), ('p52', '0', '0', 112), ('p52', '4', '0', 112), ('p52', '8', '0', 112), (None, '0xc09970+0x1c*6', '0', 12), ('p56', '0', '0', 112), ('p56', '4', '0', 112), ('p56', '8', '0', 112), (None, '0xc09970+0x1c*7', '0', 12), ('p60', '0', '0', 112), ('p60', '4', '0', 112), ('p60', '8', '0', 112), (None, '0xc09710+0x1c*0', '0', 64), ('p64', '0', '0', 36), ('p64', '4', '0', 36), ('p64', '8', '0', 36), ('p64', '12', '0', 36), ('p64', '16', '0', 36), ('p64', '20', '0', 36), ('p64', '24', '0', 36), ('p64', '28', '0', 36), ('p64', '32', '0', 36), ('p64', '36', '0', 36), ('p64', '40', '0', 36), ('p64', '44', '0', 36), ('p64', '48', '0', 36), ('p64', '52', '0', 36), ('p64', '56', '0', 36), ('p64', '60', '0', 36), (None, '0xc09710+0x1c*1', '0', 64), ('p81', '0', '0', 36), ('p81', '4', '0', 36), ('p81', '8', '0', 36), ('p81', '12', '0', 36), ('p81', '16', '0', 36), ('p81', '20', '0', 36), ('p81', '24', '0', 36), ('p81', '28', '0', 36), ('p81', '32', '0', 36), ('p81', '36', '0', 36), ('p81', '40', '0', 36), ('p81', '44', '0', 36), ('p81', '48', '0', 36), ('p81', '52', '0', 36), ('p81', '56', '0', 36), ('p81', '60', '0', 36), (None, '0xc09710+0x1c*2', '0', 64), ('p98', '0', '0', 36), ('p98', '4', '0', 36), ('p98', '8', '0', 36), ('p98', '12', '0', 36), ('p98', '16', '0', 36), ('p98', '20', '0', 36), ('p98', '24', '0', 36), ('p98', '28', '0', 36), ('p98', '32', '0', 36), ('p98', '36', '0', 36), ('p98', '40', '0', 36), ('p98', '44', '0', 36), ('p98', '48', '0', 36), ('p98', '52', '0', 36), ('p98', '56', '0', 36), ('p98', '60', '0', 36), (None, '0xc09710+0x1c*3', '0', 64), ('p115', '0', '0', 36), ('p115', '4', '0', 36), ('p115', '8', '0', 36), ('p115', '12', '0', 36), ('p115', '16', '0', 36), ('p115', '20', '0', 36), ('p115', '24', '0', 36), ('p115', '28', '0', 36), ('p115', '32', '0', 36), ('p115', '36', '0', 36), ('p115', '40', '0', 36), ('p115', '44', '0', 36), ('p115', '48', '0', 36), ('p115', '52', '0', 36), ('p115', '56', '0', 36), ('p115', '60', '0', 36), (None, '0xc09710+0x1c*4', '0', 64), ('p132', '0', '0', 36), ('p132', '4', '0', 36), ('p132', '8', '0', 36), ('p132', '12', '0', 36), ('p132', '16', '0', 36), ('p132', '20', '0', 36), ('p132', '24', '0', 36), ('p132', '28', '0', 36), ('p132', '32', '0', 36), ('p132', '36', '0', 36), ('p132', '40', '0', 36), ('p132', '44', '0', 36), ('p132', '48', '0', 36), ('p132', '52', '0', 36), ('p132', '56', '0', 36), ('p132', '60', '0', 36), (None, '0xc09710+0x1c*5', '0', 64), ('p149', '0', '0', 36), ('p149', '4', '0', 36), ('p149', '8', '0', 36), ('p149', '12', '0', 36), ('p149', '16', '0', 36), ('p149', '20', '0', 36), ('p149', '24', '0', 36), ('p149', '28', '0', 36), ('p149', '32', '0', 36), ('p149', '36', '0', 36), ('p149', '40', '0', 36), ('p149', '44', '0', 36), ('p149', '48', '0', 36), ('p149', '52', '0', 36), ('p149', '56', '0', 36), ('p149', '60', '0', 36), (None, '0xc09710+0x1c*6', '0', 64), ('p166', '0', '0', 36), ('p166', '4', '0', 36), ('p166', '8', '0', 36), ('p166', '12', '0', 36), ('p166', '16', '0', 36), ('p166', '20', '0', 36), ('p166', '24', '0', 36), ('p166', '28', '0', 36), ('p166', '32', '0', 36), ('p166', '36', '0', 36), ('p166', '40', '0', 36), ('p166', '44', '0', 36), ('p166', '48', '0', 36), ('p166', '52', '0', 36), ('p166', '56', '0', 36), ('p166', '60', '0', 36), (None, '0xc09710+0x1c*7', '0', 64), ('p183', '0', '0', 36), ('p183', '4', '0', 36), ('p183', '8', '0', 36), ('p183', '12', '0', 36), ('p183', '16', '0', 36), ('p183', '20', '0', 36), ('p183', '24', '0', 36), ('p183', '28', '0', 36), ('p183', '32', '0', 36), ('p183', '36', '0', 36), ('p183', '40', '0', 36), ('p183', '44', '0', 36), ('p183', '48', '0', 36), ('p183', '52', '0', 36), ('p183', '56', '0', 36), ('p183', '60', '0', 36), (None, '0xc0ab84', '0', 16), ('p200', '0', '0', 16), ('p200', '4', '0', 16), ('p200', '8', '0', 16), ('p200', '12', '0', 16), ('m', '0x98', '0', 16), ('p205', '0', '0', 32), ('p205', '4', '0', 32), ('p205', '8', '0', 32), ('p205', '12', '0', 32), ('m', '0xa0', '0', 16), ('p210', '0', '0', 16), ('p210', '4', '0', 16), ('p210', '8', '0', 16), ('p210', '12', '0', 16), (None, '0xcae5fc', '0', 1024)]
NULLS = []
NULLFORCE = {}
WBIAS = [('m', 0, ('set', [0, 65536, 131072, 196608, 262144, 327680, 393216, 458752, 524288, 589824, 655360, 720896, 786432, 851968, 917504, 983040])), ('m', 8, ('range', 0, 3)), ('m', 36, ('set', [0, 1])), ('m', 56, ('range', 0, 16383)), ('m', 60, ('range', 0, 16383)), ('m', 148, ('range', 0, 7)), ('c0', 0, ('range', 1, 24)), ('c0', 4, ('range', 1, 24)), ('c1', 16, ('set', [0, 4096, 20480])), ('p211', 4, ('set', [415, 416, 531, 415, 416, 256, 414, 7])), ('p212', 4, ('set', [415, 416, 531, 415, 416, 256, 414, 7])), ('p213', 4, ('set', [415, 416, 531, 415, 416, 256, 414, 7])), ('p214', 4, ('set', [415, 416, 531, 415, 416, 256, 414, 7])), ('p0', 0, ('set', [1, 1, 1, 0, 2, 3])), ('p0', 8, ('range', 0, 7)), ('p1', None, ('set', [2, 2, 2, 2, 0, 1, 3])), ('p24', None, ('set', [2, 2, 2, 2, 0, 1])), ('p2', 0, ('range', 0, 3)), ('p3', 0, ('set', [1, 1, 1, 0, 2, 3])), ('p3', 8, ('range', 0, 7)), ('p4', None, ('set', [2, 2, 2, 2, 0, 1, 3])), ('p25', None, ('set', [2, 2, 2, 2, 0, 1])), ('p5', 0, ('range', 0, 3)), ('p6', 0, ('set', [1, 1, 1, 0, 2, 3])), ('p6', 8, ('range', 0, 7)), ('p7', None, ('set', [2, 2, 2, 2, 0, 1, 3])), ('p26', None, ('set', [2, 2, 2, 2, 0, 1])), ('p8', 0, ('range', 0, 3)), ('p9', 0, ('set', [1, 1, 1, 0, 2, 3])), ('p9', 8, ('range', 0, 7)), ('p10', None, ('set', [2, 2, 2, 2, 0, 1, 3])), ('p27', None, ('set', [2, 2, 2, 2, 0, 1])), ('p11', 0, ('range', 0, 3)), ('p12', 0, ('set', [1, 1, 1, 0, 2, 3])), ('p12', 8, ('range', 0, 7)), ('p13', None, ('set', [2, 2, 2, 2, 0, 1, 3])), ('p28', None, ('set', [2, 2, 2, 2, 0, 1])), ('p14', 0, ('range', 0, 3)), ('p15', 0, ('set', [1, 1, 1, 0, 2, 3])), ('p15', 8, ('range', 0, 7)), ('p16', None, ('set', [2, 2, 2, 2, 0, 1, 3])), ('p29', None, ('set', [2, 2, 2, 2, 0, 1])), ('p17', 0, ('range', 0, 3)), ('p18', 0, ('set', [1, 1, 1, 0, 2, 3])), ('p18', 8, ('range', 0, 7)), ('p19', None, ('set', [2, 2, 2, 2, 0, 1, 3])), ('p30', None, ('set', [2, 2, 2, 2, 0, 1])), ('p20', 0, ('range', 0, 3)), ('p21', 0, ('set', [1, 1, 1, 0, 2, 3])), ('p21', 8, ('range', 0, 7)), ('p22', None, ('set', [2, 2, 2, 2, 0, 1, 3])), ('p31', None, ('set', [2, 2, 2, 2, 0, 1])), ('p23', 0, ('range', 0, 3)), ('p33', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p34', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p35', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p33', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p34', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p35', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p33', 12, ('range', -1024, 16383)), ('p34', 12, ('range', -1024, 16383)), ('p35', 12, ('range', -1024, 16383)), ('p33', 16, ('range', -1024, 16383)), ('p34', 16, ('range', -1024, 16383)), ('p35', 16, ('range', -1024, 16383)), ('p33', 92, ('range', 0, 65535)), ('p34', 92, ('range', 0, 65535)), ('p35', 92, ('range', 0, 65535)), ('p65', 0, ('set', [0, 1, 1, 2, 65536])), ('p66', 0, ('set', [0, 1, 1, 2, 65536])), ('p67', 0, ('set', [0, 1, 1, 2, 65536])), ('p68', 0, ('set', [0, 1, 1, 2, 65536])), ('p69', 0, ('set', [0, 1, 1, 2, 65536])), ('p70', 0, ('set', [0, 1, 1, 2, 65536])), ('p71', 0, ('set', [0, 1, 1, 2, 65536])), ('p72', 0, ('set', [0, 1, 1, 2, 65536])), ('p73', 0, ('set', [0, 1, 1, 2, 65536])), ('p74', 0, ('set', [0, 1, 1, 2, 65536])), ('p75', 0, ('set', [0, 1, 1, 2, 65536])), ('p76', 0, ('set', [0, 1, 1, 2, 65536])), ('p77', 0, ('set', [0, 1, 1, 2, 65536])), ('p78', 0, ('set', [0, 1, 1, 2, 65536])), ('p79', 0, ('set', [0, 1, 1, 2, 65536])), ('p80', 0, ('set', [0, 1, 1, 2, 65536])), ('p65', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p66', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p67', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p68', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p69', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p70', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p71', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p72', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p73', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p74', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p75', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p76', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p77', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p78', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p79', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p80', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p65', 32, ('set', [0, 1, 2, 3, -1])), ('p66', 32, ('set', [0, 1, 2, 3, -1])), ('p67', 32, ('set', [0, 1, 2, 3, -1])), ('p68', 32, ('set', [0, 1, 2, 3, -1])), ('p69', 32, ('set', [0, 1, 2, 3, -1])), ('p70', 32, ('set', [0, 1, 2, 3, -1])), ('p71', 32, ('set', [0, 1, 2, 3, -1])), ('p72', 32, ('set', [0, 1, 2, 3, -1])), ('p73', 32, ('set', [0, 1, 2, 3, -1])), ('p74', 32, ('set', [0, 1, 2, 3, -1])), ('p75', 32, ('set', [0, 1, 2, 3, -1])), ('p76', 32, ('set', [0, 1, 2, 3, -1])), ('p77', 32, ('set', [0, 1, 2, 3, -1])), ('p78', 32, ('set', [0, 1, 2, 3, -1])), ('p79', 32, ('set', [0, 1, 2, 3, -1])), ('p80', 32, ('set', [0, 1, 2, 3, -1])), ('p37', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p38', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p39', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p37', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p38', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p39', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p37', 12, ('range', -1024, 16383)), ('p38', 12, ('range', -1024, 16383)), ('p39', 12, ('range', -1024, 16383)), ('p37', 16, ('range', -1024, 16383)), ('p38', 16, ('range', -1024, 16383)), ('p39', 16, ('range', -1024, 16383)), ('p37', 92, ('range', 0, 65535)), ('p38', 92, ('range', 0, 65535)), ('p39', 92, ('range', 0, 65535)), ('p82', 0, ('set', [0, 1, 1, 2, 65536])), ('p83', 0, ('set', [0, 1, 1, 2, 65536])), ('p84', 0, ('set', [0, 1, 1, 2, 65536])), ('p85', 0, ('set', [0, 1, 1, 2, 65536])), ('p86', 0, ('set', [0, 1, 1, 2, 65536])), ('p87', 0, ('set', [0, 1, 1, 2, 65536])), ('p88', 0, ('set', [0, 1, 1, 2, 65536])), ('p89', 0, ('set', [0, 1, 1, 2, 65536])), ('p90', 0, ('set', [0, 1, 1, 2, 65536])), ('p91', 0, ('set', [0, 1, 1, 2, 65536])), ('p92', 0, ('set', [0, 1, 1, 2, 65536])), ('p93', 0, ('set', [0, 1, 1, 2, 65536])), ('p94', 0, ('set', [0, 1, 1, 2, 65536])), ('p95', 0, ('set', [0, 1, 1, 2, 65536])), ('p96', 0, ('set', [0, 1, 1, 2, 65536])), ('p97', 0, ('set', [0, 1, 1, 2, 65536])), ('p82', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p83', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p84', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p85', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p86', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p87', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p88', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p89', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p90', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p91', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p92', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p93', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p94', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p95', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p96', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p97', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p82', 32, ('set', [0, 1, 2, 3, -1])), ('p83', 32, ('set', [0, 1, 2, 3, -1])), ('p84', 32, ('set', [0, 1, 2, 3, -1])), ('p85', 32, ('set', [0, 1, 2, 3, -1])), ('p86', 32, ('set', [0, 1, 2, 3, -1])), ('p87', 32, ('set', [0, 1, 2, 3, -1])), ('p88', 32, ('set', [0, 1, 2, 3, -1])), ('p89', 32, ('set', [0, 1, 2, 3, -1])), ('p90', 32, ('set', [0, 1, 2, 3, -1])), ('p91', 32, ('set', [0, 1, 2, 3, -1])), ('p92', 32, ('set', [0, 1, 2, 3, -1])), ('p93', 32, ('set', [0, 1, 2, 3, -1])), ('p94', 32, ('set', [0, 1, 2, 3, -1])), ('p95', 32, ('set', [0, 1, 2, 3, -1])), ('p96', 32, ('set', [0, 1, 2, 3, -1])), ('p97', 32, ('set', [0, 1, 2, 3, -1])), ('p41', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p42', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p43', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p41', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p42', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p43', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p41', 12, ('range', -1024, 16383)), ('p42', 12, ('range', -1024, 16383)), ('p43', 12, ('range', -1024, 16383)), ('p41', 16, ('range', -1024, 16383)), ('p42', 16, ('range', -1024, 16383)), ('p43', 16, ('range', -1024, 16383)), ('p41', 92, ('range', 0, 65535)), ('p42', 92, ('range', 0, 65535)), ('p43', 92, ('range', 0, 65535)), ('p99', 0, ('set', [0, 1, 1, 2, 65536])), ('p100', 0, ('set', [0, 1, 1, 2, 65536])), ('p101', 0, ('set', [0, 1, 1, 2, 65536])), ('p102', 0, ('set', [0, 1, 1, 2, 65536])), ('p103', 0, ('set', [0, 1, 1, 2, 65536])), ('p104', 0, ('set', [0, 1, 1, 2, 65536])), ('p105', 0, ('set', [0, 1, 1, 2, 65536])), ('p106', 0, ('set', [0, 1, 1, 2, 65536])), ('p107', 0, ('set', [0, 1, 1, 2, 65536])), ('p108', 0, ('set', [0, 1, 1, 2, 65536])), ('p109', 0, ('set', [0, 1, 1, 2, 65536])), ('p110', 0, ('set', [0, 1, 1, 2, 65536])), ('p111', 0, ('set', [0, 1, 1, 2, 65536])), ('p112', 0, ('set', [0, 1, 1, 2, 65536])), ('p113', 0, ('set', [0, 1, 1, 2, 65536])), ('p114', 0, ('set', [0, 1, 1, 2, 65536])), ('p99', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p100', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p101', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p102', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p103', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p104', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p105', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p106', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p107', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p108', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p109', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p110', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p111', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p112', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p113', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p114', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p99', 32, ('set', [0, 1, 2, 3, -1])), ('p100', 32, ('set', [0, 1, 2, 3, -1])), ('p101', 32, ('set', [0, 1, 2, 3, -1])), ('p102', 32, ('set', [0, 1, 2, 3, -1])), ('p103', 32, ('set', [0, 1, 2, 3, -1])), ('p104', 32, ('set', [0, 1, 2, 3, -1])), ('p105', 32, ('set', [0, 1, 2, 3, -1])), ('p106', 32, ('set', [0, 1, 2, 3, -1])), ('p107', 32, ('set', [0, 1, 2, 3, -1])), ('p108', 32, ('set', [0, 1, 2, 3, -1])), ('p109', 32, ('set', [0, 1, 2, 3, -1])), ('p110', 32, ('set', [0, 1, 2, 3, -1])), ('p111', 32, ('set', [0, 1, 2, 3, -1])), ('p112', 32, ('set', [0, 1, 2, 3, -1])), ('p113', 32, ('set', [0, 1, 2, 3, -1])), ('p114', 32, ('set', [0, 1, 2, 3, -1])), ('p45', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p46', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p47', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p45', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p46', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p47', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p45', 12, ('range', -1024, 16383)), ('p46', 12, ('range', -1024, 16383)), ('p47', 12, ('range', -1024, 16383)), ('p45', 16, ('range', -1024, 16383)), ('p46', 16, ('range', -1024, 16383)), ('p47', 16, ('range', -1024, 16383)), ('p45', 92, ('range', 0, 65535)), ('p46', 92, ('range', 0, 65535)), ('p47', 92, ('range', 0, 65535)), ('p116', 0, ('set', [0, 1, 1, 2, 65536])), ('p117', 0, ('set', [0, 1, 1, 2, 65536])), ('p118', 0, ('set', [0, 1, 1, 2, 65536])), ('p119', 0, ('set', [0, 1, 1, 2, 65536])), ('p120', 0, ('set', [0, 1, 1, 2, 65536])), ('p121', 0, ('set', [0, 1, 1, 2, 65536])), ('p122', 0, ('set', [0, 1, 1, 2, 65536])), ('p123', 0, ('set', [0, 1, 1, 2, 65536])), ('p124', 0, ('set', [0, 1, 1, 2, 65536])), ('p125', 0, ('set', [0, 1, 1, 2, 65536])), ('p126', 0, ('set', [0, 1, 1, 2, 65536])), ('p127', 0, ('set', [0, 1, 1, 2, 65536])), ('p128', 0, ('set', [0, 1, 1, 2, 65536])), ('p129', 0, ('set', [0, 1, 1, 2, 65536])), ('p130', 0, ('set', [0, 1, 1, 2, 65536])), ('p131', 0, ('set', [0, 1, 1, 2, 65536])), ('p116', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p117', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p118', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p119', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p120', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p121', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p122', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p123', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p124', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p125', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p126', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p127', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p128', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p129', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p130', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p131', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p116', 32, ('set', [0, 1, 2, 3, -1])), ('p117', 32, ('set', [0, 1, 2, 3, -1])), ('p118', 32, ('set', [0, 1, 2, 3, -1])), ('p119', 32, ('set', [0, 1, 2, 3, -1])), ('p120', 32, ('set', [0, 1, 2, 3, -1])), ('p121', 32, ('set', [0, 1, 2, 3, -1])), ('p122', 32, ('set', [0, 1, 2, 3, -1])), ('p123', 32, ('set', [0, 1, 2, 3, -1])), ('p124', 32, ('set', [0, 1, 2, 3, -1])), ('p125', 32, ('set', [0, 1, 2, 3, -1])), ('p126', 32, ('set', [0, 1, 2, 3, -1])), ('p127', 32, ('set', [0, 1, 2, 3, -1])), ('p128', 32, ('set', [0, 1, 2, 3, -1])), ('p129', 32, ('set', [0, 1, 2, 3, -1])), ('p130', 32, ('set', [0, 1, 2, 3, -1])), ('p131', 32, ('set', [0, 1, 2, 3, -1])), ('p49', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p50', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p51', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p49', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p50', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p51', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p49', 12, ('range', -1024, 16383)), ('p50', 12, ('range', -1024, 16383)), ('p51', 12, ('range', -1024, 16383)), ('p49', 16, ('range', -1024, 16383)), ('p50', 16, ('range', -1024, 16383)), ('p51', 16, ('range', -1024, 16383)), ('p49', 92, ('range', 0, 65535)), ('p50', 92, ('range', 0, 65535)), ('p51', 92, ('range', 0, 65535)), ('p133', 0, ('set', [0, 1, 1, 2, 65536])), ('p134', 0, ('set', [0, 1, 1, 2, 65536])), ('p135', 0, ('set', [0, 1, 1, 2, 65536])), ('p136', 0, ('set', [0, 1, 1, 2, 65536])), ('p137', 0, ('set', [0, 1, 1, 2, 65536])), ('p138', 0, ('set', [0, 1, 1, 2, 65536])), ('p139', 0, ('set', [0, 1, 1, 2, 65536])), ('p140', 0, ('set', [0, 1, 1, 2, 65536])), ('p141', 0, ('set', [0, 1, 1, 2, 65536])), ('p142', 0, ('set', [0, 1, 1, 2, 65536])), ('p143', 0, ('set', [0, 1, 1, 2, 65536])), ('p144', 0, ('set', [0, 1, 1, 2, 65536])), ('p145', 0, ('set', [0, 1, 1, 2, 65536])), ('p146', 0, ('set', [0, 1, 1, 2, 65536])), ('p147', 0, ('set', [0, 1, 1, 2, 65536])), ('p148', 0, ('set', [0, 1, 1, 2, 65536])), ('p133', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p134', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p135', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p136', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p137', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p138', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p139', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p140', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p141', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p142', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p143', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p144', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p145', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p146', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p147', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p148', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p133', 32, ('set', [0, 1, 2, 3, -1])), ('p134', 32, ('set', [0, 1, 2, 3, -1])), ('p135', 32, ('set', [0, 1, 2, 3, -1])), ('p136', 32, ('set', [0, 1, 2, 3, -1])), ('p137', 32, ('set', [0, 1, 2, 3, -1])), ('p138', 32, ('set', [0, 1, 2, 3, -1])), ('p139', 32, ('set', [0, 1, 2, 3, -1])), ('p140', 32, ('set', [0, 1, 2, 3, -1])), ('p141', 32, ('set', [0, 1, 2, 3, -1])), ('p142', 32, ('set', [0, 1, 2, 3, -1])), ('p143', 32, ('set', [0, 1, 2, 3, -1])), ('p144', 32, ('set', [0, 1, 2, 3, -1])), ('p145', 32, ('set', [0, 1, 2, 3, -1])), ('p146', 32, ('set', [0, 1, 2, 3, -1])), ('p147', 32, ('set', [0, 1, 2, 3, -1])), ('p148', 32, ('set', [0, 1, 2, 3, -1])), ('p53', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p54', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p55', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p53', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p54', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p55', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p53', 12, ('range', -1024, 16383)), ('p54', 12, ('range', -1024, 16383)), ('p55', 12, ('range', -1024, 16383)), ('p53', 16, ('range', -1024, 16383)), ('p54', 16, ('range', -1024, 16383)), ('p55', 16, ('range', -1024, 16383)), ('p53', 92, ('range', 0, 65535)), ('p54', 92, ('range', 0, 65535)), ('p55', 92, ('range', 0, 65535)), ('p150', 0, ('set', [0, 1, 1, 2, 65536])), ('p151', 0, ('set', [0, 1, 1, 2, 65536])), ('p152', 0, ('set', [0, 1, 1, 2, 65536])), ('p153', 0, ('set', [0, 1, 1, 2, 65536])), ('p154', 0, ('set', [0, 1, 1, 2, 65536])), ('p155', 0, ('set', [0, 1, 1, 2, 65536])), ('p156', 0, ('set', [0, 1, 1, 2, 65536])), ('p157', 0, ('set', [0, 1, 1, 2, 65536])), ('p158', 0, ('set', [0, 1, 1, 2, 65536])), ('p159', 0, ('set', [0, 1, 1, 2, 65536])), ('p160', 0, ('set', [0, 1, 1, 2, 65536])), ('p161', 0, ('set', [0, 1, 1, 2, 65536])), ('p162', 0, ('set', [0, 1, 1, 2, 65536])), ('p163', 0, ('set', [0, 1, 1, 2, 65536])), ('p164', 0, ('set', [0, 1, 1, 2, 65536])), ('p165', 0, ('set', [0, 1, 1, 2, 65536])), ('p150', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p151', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p152', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p153', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p154', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p155', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p156', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p157', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p158', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p159', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p160', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p161', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p162', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p163', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p164', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p165', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p150', 32, ('set', [0, 1, 2, 3, -1])), ('p151', 32, ('set', [0, 1, 2, 3, -1])), ('p152', 32, ('set', [0, 1, 2, 3, -1])), ('p153', 32, ('set', [0, 1, 2, 3, -1])), ('p154', 32, ('set', [0, 1, 2, 3, -1])), ('p155', 32, ('set', [0, 1, 2, 3, -1])), ('p156', 32, ('set', [0, 1, 2, 3, -1])), ('p157', 32, ('set', [0, 1, 2, 3, -1])), ('p158', 32, ('set', [0, 1, 2, 3, -1])), ('p159', 32, ('set', [0, 1, 2, 3, -1])), ('p160', 32, ('set', [0, 1, 2, 3, -1])), ('p161', 32, ('set', [0, 1, 2, 3, -1])), ('p162', 32, ('set', [0, 1, 2, 3, -1])), ('p163', 32, ('set', [0, 1, 2, 3, -1])), ('p164', 32, ('set', [0, 1, 2, 3, -1])), ('p165', 32, ('set', [0, 1, 2, 3, -1])), ('p57', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p58', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p59', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p57', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p58', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p59', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p57', 12, ('range', -1024, 16383)), ('p58', 12, ('range', -1024, 16383)), ('p59', 12, ('range', -1024, 16383)), ('p57', 16, ('range', -1024, 16383)), ('p58', 16, ('range', -1024, 16383)), ('p59', 16, ('range', -1024, 16383)), ('p57', 92, ('range', 0, 65535)), ('p58', 92, ('range', 0, 65535)), ('p59', 92, ('range', 0, 65535)), ('p167', 0, ('set', [0, 1, 1, 2, 65536])), ('p168', 0, ('set', [0, 1, 1, 2, 65536])), ('p169', 0, ('set', [0, 1, 1, 2, 65536])), ('p170', 0, ('set', [0, 1, 1, 2, 65536])), ('p171', 0, ('set', [0, 1, 1, 2, 65536])), ('p172', 0, ('set', [0, 1, 1, 2, 65536])), ('p173', 0, ('set', [0, 1, 1, 2, 65536])), ('p174', 0, ('set', [0, 1, 1, 2, 65536])), ('p175', 0, ('set', [0, 1, 1, 2, 65536])), ('p176', 0, ('set', [0, 1, 1, 2, 65536])), ('p177', 0, ('set', [0, 1, 1, 2, 65536])), ('p178', 0, ('set', [0, 1, 1, 2, 65536])), ('p179', 0, ('set', [0, 1, 1, 2, 65536])), ('p180', 0, ('set', [0, 1, 1, 2, 65536])), ('p181', 0, ('set', [0, 1, 1, 2, 65536])), ('p182', 0, ('set', [0, 1, 1, 2, 65536])), ('p167', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p168', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p169', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p170', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p171', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p172', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p173', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p174', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p175', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p176', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p177', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p178', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p179', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p180', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p181', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p182', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p167', 32, ('set', [0, 1, 2, 3, -1])), ('p168', 32, ('set', [0, 1, 2, 3, -1])), ('p169', 32, ('set', [0, 1, 2, 3, -1])), ('p170', 32, ('set', [0, 1, 2, 3, -1])), ('p171', 32, ('set', [0, 1, 2, 3, -1])), ('p172', 32, ('set', [0, 1, 2, 3, -1])), ('p173', 32, ('set', [0, 1, 2, 3, -1])), ('p174', 32, ('set', [0, 1, 2, 3, -1])), ('p175', 32, ('set', [0, 1, 2, 3, -1])), ('p176', 32, ('set', [0, 1, 2, 3, -1])), ('p177', 32, ('set', [0, 1, 2, 3, -1])), ('p178', 32, ('set', [0, 1, 2, 3, -1])), ('p179', 32, ('set', [0, 1, 2, 3, -1])), ('p180', 32, ('set', [0, 1, 2, 3, -1])), ('p181', 32, ('set', [0, 1, 2, 3, -1])), ('p182', 32, ('set', [0, 1, 2, 3, -1])), ('p61', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p62', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p63', 4, ('set', [0, 1, 2, 3, 3, 3, 65539])), ('p61', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p62', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p63', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p61', 12, ('range', -1024, 16383)), ('p62', 12, ('range', -1024, 16383)), ('p63', 12, ('range', -1024, 16383)), ('p61', 16, ('range', -1024, 16383)), ('p62', 16, ('range', -1024, 16383)), ('p63', 16, ('range', -1024, 16383)), ('p61', 92, ('range', 0, 65535)), ('p62', 92, ('range', 0, 65535)), ('p63', 92, ('range', 0, 65535)), ('p184', 0, ('set', [0, 1, 1, 2, 65536])), ('p185', 0, ('set', [0, 1, 1, 2, 65536])), ('p186', 0, ('set', [0, 1, 1, 2, 65536])), ('p187', 0, ('set', [0, 1, 1, 2, 65536])), ('p188', 0, ('set', [0, 1, 1, 2, 65536])), ('p189', 0, ('set', [0, 1, 1, 2, 65536])), ('p190', 0, ('set', [0, 1, 1, 2, 65536])), ('p191', 0, ('set', [0, 1, 1, 2, 65536])), ('p192', 0, ('set', [0, 1, 1, 2, 65536])), ('p193', 0, ('set', [0, 1, 1, 2, 65536])), ('p194', 0, ('set', [0, 1, 1, 2, 65536])), ('p195', 0, ('set', [0, 1, 1, 2, 65536])), ('p196', 0, ('set', [0, 1, 1, 2, 65536])), ('p197', 0, ('set', [0, 1, 1, 2, 65536])), ('p198', 0, ('set', [0, 1, 1, 2, 65536])), ('p199', 0, ('set', [0, 1, 1, 2, 65536])), ('p184', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p185', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p186', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p187', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p188', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p189', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p190', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p191', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p192', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p193', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p194', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p195', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p196', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p197', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p198', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p199', 20, ('set', [0, 3, 4, 5, 6, 9])), ('p184', 32, ('set', [0, 1, 2, 3, -1])), ('p185', 32, ('set', [0, 1, 2, 3, -1])), ('p186', 32, ('set', [0, 1, 2, 3, -1])), ('p187', 32, ('set', [0, 1, 2, 3, -1])), ('p188', 32, ('set', [0, 1, 2, 3, -1])), ('p189', 32, ('set', [0, 1, 2, 3, -1])), ('p190', 32, ('set', [0, 1, 2, 3, -1])), ('p191', 32, ('set', [0, 1, 2, 3, -1])), ('p192', 32, ('set', [0, 1, 2, 3, -1])), ('p193', 32, ('set', [0, 1, 2, 3, -1])), ('p194', 32, ('set', [0, 1, 2, 3, -1])), ('p195', 32, ('set', [0, 1, 2, 3, -1])), ('p196', 32, ('set', [0, 1, 2, 3, -1])), ('p197', 32, ('set', [0, 1, 2, 3, -1])), ('p198', 32, ('set', [0, 1, 2, 3, -1])), ('p199', 32, ('set', [0, 1, 2, 3, -1]))]
FILLS = [('p215', 'i//3')]
ARGRANGE = []
SLOTBASE = []
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(1, [0, 1]), (2, [-1, 0, 5, -1])]
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = [('p206', 24, 'p211', 0, True), ('p207', 24, 'p212', 0, True), ('p208', 24, 'p213', 0, True), ('p209', 24, 'p214', 0, True)]
ANSWERBY = [(0, 201, 206, 4, 28)]
ANSWERWIN = []
CDECL = False
TIB = False
ARGEQ = []
SEQ = []
ANSRANGE = []
CELLTHIS = []
VTALSO = [('p201', 0), ('p202', 0), ('p203', 0), ('p204', 0)]
SPARSE = [('@', '0xe3a390', 28396, 8, [(0, 12), (116, 32), (1032, 4)]), (None, '0xc061e0', 28396, 8, [(116, 32)])]
GTAB = []
RECORD = [1, 2, 3, 4]
RECORDORDER = False
RECORDEDX = True
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
    rng = random.Random(3281)
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
    for i in range(2000):
        h.brk = base  # the bump heap is reused a row; 2000 rows of windows outgrow it
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

# sweep_gen: ["obj", "Army::find_besieged_city@006f58b0", "--item", "3281", "--name", "army_find_besieged_city", "--win", "0xb0", "--own", "0", "--cell-win", "0xc06188:0x10", "--cell-win", "0xc061b8:0x20", "--cell-win", "0xc0618c:0x20", "--sparse", "@0xe3a390:0x6eec:8:0,12/0x74,0x20/0x408,4", "--sparse", "0xc061e0:0x6eec:8:0x74,0x20", "--ptrtab", "0xc09970+0x1c*0:3:0x70", "--ptrtab", "0xc09970+0x1c*1:3:0x70", "--ptrtab", "0xc09970+0x1c*2:3:0x70", "--ptrtab", "0xc09970+0x1c*3:3:0x70", "--ptrtab", "0xc09970+0x1c*4:3:0x70", "--ptrtab", "0xc09970+0x1c*5:3:0x70", "--ptrtab", "0xc09970+0x1c*6:3:0x70", "--ptrtab", "0xc09970+0x1c*7:3:0x70", "--ptrtab", "0xc09710+0x1c*0:16:0x24", "--ptrtab", "0xc09710+0x1c*1:16:0x24", "--ptrtab", "0xc09710+0x1c*2:16:0x24", "--ptrtab", "0xc09710+0x1c*3:16:0x24", "--ptrtab", "0xc09710+0x1c*4:16:0x24", "--ptrtab", "0xc09710+0x1c*5:16:0x24", "--ptrtab", "0xc09710+0x1c*6:16:0x24", "--ptrtab", "0xc09710+0x1c*7:16:0x24", "--ptrtab", "0xc0ab84:4:0x10:vt", "--ptrtab", "m+0x98:4:0x20", "--ptrtab", "m+0xa0:4:0x10", "--place", "0xcae5fc:0:0x400", "--stub", "slot:0x3c:0", "--stub", "0x680f90:1", "--stub", "0x65ca80:13", "--stub", "0x92d130:0", "--stub", "0x6f5cc0:3", "--stub", "run:0x46cff0:0", "--wbias", "m+0:0,0x10000,0x20000,0x30000,0x40000,0x50000,0x60000,0x70000,0x80000,0x90000,0xa0000,0xb0000,0xc0000,0xd0000,0xe0000,0xf0000", "--wbias", "m+8:0..3", "--wbias", "m+0x24:0,1", "--wbias", "m+0x38:0..0x3fff", "--wbias", "m+0x3c:0..0x3fff", "--wbias", "m+0x94:0..7", "--wbias", "c0+0:1..24", "--wbias", "c0+4:1..24", "--wbias", "c1+0x10:0,0x1000,0x5000", "--wbias", "p211..214+4:0x19f,0x1a0,0x213,0x19f,0x1a0,0x100,0x19e,7", "--wbias", "p0+0:1,1,1,0,2,3", "--wbias", "p0+8:0..7", "--wbias", "p1+*:2,2,2,2,0,1,3", "--wbias", "p24+*:2,2,2,2,0,1", "--wbias", "p2+0:0..3", "--wbias", "p3+0:1,1,1,0,2,3", "--wbias", "p3+8:0..7", "--wbias", "p4+*:2,2,2,2,0,1,3", "--wbias", "p25+*:2,2,2,2,0,1", "--wbias", "p5+0:0..3", "--wbias", "p6+0:1,1,1,0,2,3", "--wbias", "p6+8:0..7", "--wbias", "p7+*:2,2,2,2,0,1,3", "--wbias", "p26+*:2,2,2,2,0,1", "--wbias", "p8+0:0..3", "--wbias", "p9+0:1,1,1,0,2,3", "--wbias", "p9+8:0..7", "--wbias", "p10+*:2,2,2,2,0,1,3", "--wbias", "p27+*:2,2,2,2,0,1", "--wbias", "p11+0:0..3", "--wbias", "p12+0:1,1,1,0,2,3", "--wbias", "p12+8:0..7", "--wbias", "p13+*:2,2,2,2,0,1,3", "--wbias", "p28+*:2,2,2,2,0,1", "--wbias", "p14+0:0..3", "--wbias", "p15+0:1,1,1,0,2,3", "--wbias", "p15+8:0..7", "--wbias", "p16+*:2,2,2,2,0,1,3", "--wbias", "p29+*:2,2,2,2,0,1", "--wbias", "p17+0:0..3", "--wbias", "p18+0:1,1,1,0,2,3", "--wbias", "p18+8:0..7", "--wbias", "p19+*:2,2,2,2,0,1,3", "--wbias", "p30+*:2,2,2,2,0,1", "--wbias", "p20+0:0..3", "--wbias", "p21+0:1,1,1,0,2,3", "--wbias", "p21+8:0..7", "--wbias", "p22+*:2,2,2,2,0,1,3", "--wbias", "p31+*:2,2,2,2,0,1", "--wbias", "p23+0:0..3", "--wbias", "p33..35+4:0,1,2,3,3,3,0x10003", "--wbias", "p33..35+8:0,1,2,3,0x10000,0x10001,0x10002,0x10003,0x20000,0x20001,0x20002,0x20003,0x30000,0x30001,0x30002,0x30003", "--wbias", "p33..35+0xc:-0x400..0x3fff", "--wbias", "p33..35+0x10:-0x400..0x3fff", "--wbias", "p33..35+0x5c:0..0xffff", "--wbias", "p65..80+0:0,1,1,2,0x10000", "--wbias", "p65..80+0x14:0,3,4,5,6,9", "--wbias", "p65..80+0x20:0,1,2,3,-1", "--wbias", "p37..39+4:0,1,2,3,3,3,0x10003", "--wbias", "p37..39+8:0,1,2,3,0x10000,0x10001,0x10002,0x10003,0x20000,0x20001,0x20002,0x20003,0x30000,0x30001,0x30002,0x30003", "--wbias", "p37..39+0xc:-0x400..0x3fff", "--wbias", "p37..39+0x10:-0x400..0x3fff", "--wbias", "p37..39+0x5c:0..0xffff", "--wbias", "p82..97+0:0,1,1,2,0x10000", "--wbias", "p82..97+0x14:0,3,4,5,6,9", "--wbias", "p82..97+0x20:0,1,2,3,-1", "--wbias", "p41..43+4:0,1,2,3,3,3,0x10003", "--wbias", "p41..43+8:0,1,2,3,0x10000,0x10001,0x10002,0x10003,0x20000,0x20001,0x20002,0x20003,0x30000,0x30001,0x30002,0x30003", "--wbias", "p41..43+0xc:-0x400..0x3fff", "--wbias", "p41..43+0x10:-0x400..0x3fff", "--wbias", "p41..43+0x5c:0..0xffff", "--wbias", "p99..114+0:0,1,1,2,0x10000", "--wbias", "p99..114+0x14:0,3,4,5,6,9", "--wbias", "p99..114+0x20:0,1,2,3,-1", "--wbias", "p45..47+4:0,1,2,3,3,3,0x10003", "--wbias", "p45..47+8:0,1,2,3,0x10000,0x10001,0x10002,0x10003,0x20000,0x20001,0x20002,0x20003,0x30000,0x30001,0x30002,0x30003", "--wbias", "p45..47+0xc:-0x400..0x3fff", "--wbias", "p45..47+0x10:-0x400..0x3fff", "--wbias", "p45..47+0x5c:0..0xffff", "--wbias", "p116..131+0:0,1,1,2,0x10000", "--wbias", "p116..131+0x14:0,3,4,5,6,9", "--wbias", "p116..131+0x20:0,1,2,3,-1", "--wbias", "p49..51+4:0,1,2,3,3,3,0x10003", "--wbias", "p49..51+8:0,1,2,3,0x10000,0x10001,0x10002,0x10003,0x20000,0x20001,0x20002,0x20003,0x30000,0x30001,0x30002,0x30003", "--wbias", "p49..51+0xc:-0x400..0x3fff", "--wbias", "p49..51+0x10:-0x400..0x3fff", "--wbias", "p49..51+0x5c:0..0xffff", "--wbias", "p133..148+0:0,1,1,2,0x10000", "--wbias", "p133..148+0x14:0,3,4,5,6,9", "--wbias", "p133..148+0x20:0,1,2,3,-1", "--wbias", "p53..55+4:0,1,2,3,3,3,0x10003", "--wbias", "p53..55+8:0,1,2,3,0x10000,0x10001,0x10002,0x10003,0x20000,0x20001,0x20002,0x20003,0x30000,0x30001,0x30002,0x30003", "--wbias", "p53..55+0xc:-0x400..0x3fff", "--wbias", "p53..55+0x10:-0x400..0x3fff", "--wbias", "p53..55+0x5c:0..0xffff", "--wbias", "p150..165+0:0,1,1,2,0x10000", "--wbias", "p150..165+0x14:0,3,4,5,6,9", "--wbias", "p150..165+0x20:0,1,2,3,-1", "--wbias", "p57..59+4:0,1,2,3,3,3,0x10003", "--wbias", "p57..59+8:0,1,2,3,0x10000,0x10001,0x10002,0x10003,0x20000,0x20001,0x20002,0x20003,0x30000,0x30001,0x30002,0x30003", "--wbias", "p57..59+0xc:-0x400..0x3fff", "--wbias", "p57..59+0x10:-0x400..0x3fff", "--wbias", "p57..59+0x5c:0..0xffff", "--wbias", "p167..182+0:0,1,1,2,0x10000", "--wbias", "p167..182+0x14:0,3,4,5,6,9", "--wbias", "p167..182+0x20:0,1,2,3,-1", "--wbias", "p61..63+4:0,1,2,3,3,3,0x10003", "--wbias", "p61..63+8:0,1,2,3,0x10000,0x10001,0x10002,0x10003,0x20000,0x20001,0x20002,0x20003,0x30000,0x30001,0x30002,0x30003", "--wbias", "p61..63+0xc:-0x400..0x3fff", "--wbias", "p61..63+0x10:-0x400..0x3fff", "--wbias", "p61..63+0x5c:0..0xffff", "--wbias", "p184..199+0:0,1,1,2,0x10000", "--wbias", "p184..199+0x14:0,3,4,5,6,9", "--wbias", "p184..199+0x20:0,1,2,3,-1", "--alias", "p206+0x18:p211!", "--alias", "p207+0x18:p212!", "--alias", "p208+0x18:p213!", "--alias", "p209+0x18:p214!", "--answer-by", "0:201:206:4:0x1c", "--answers", "1:0,1", "--answers", "2:-1,0,5,-1", "--record", "1", "--record", "2", "--record", "3", "--record", "4", "--record-edx", "--fill", "p215:i//3", "--ret", "eax", "--rows", "2000", "--port", "todo"]
