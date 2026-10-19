#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_objects_data_find_unit.py — `ObjectsData::find_unit@0065ca80` under unicorn (item 3339, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_objects_data_find_unit.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 1024-byte object of random bytes (no bias), 13 stack argument(s);
`p` windows in order: p0..p7 sparse table 0 (8 rows × 1 slice(s), row-major), p8..p15 sparse table 1 (8 rows × 1 slice(s), row-major), p16..p23 sparse table 2 (8 rows × 1 slice(s), row-major), p24..p88 the place(s), in flag order; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00cae5fc points at 2048 random bytes; the global cell 0x00c061e8 points at 1760 random bytes; 40 random bytes at 0x00cbe330 (each word in 0..14); 16 random bytes at 0x00cb7e90 (each byte in -3..3); 16 random bytes at 0x00cbb0e0 (each byte in -3..3); vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x0067daa0 (2 args) stubbed; callee 0x0067dbb0 (4 args) stubbed; vtable slot +0xe8 (0 args) stubbed; the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `p3+0` points at 16 random bytes less `0` (placed); the word at `p4+0` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 16 random bytes less `0` (placed); the word at `p7+0` points at 16 random bytes less `0` (placed); the word at `p8+0` points at 16 random bytes less `0` (placed); the word at `p9+0` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 16 random bytes less `0` (placed); the word at `p11+0` points at 16 random bytes less `0` (placed); the word at `p12+0` points at 16 random bytes less `0` (placed); the word at `p13+0` points at 16 random bytes less `0` (placed); the word at `p14+0` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 16 random bytes less `0` (placed); the word at `p24+0` points at 64 random bytes less `0` (placed); the word at `p24+4` points at 64 random bytes less `0` (placed); the word at `p24+8` points at 64 random bytes less `0` (placed); the word at `p25+0` points at 64 random bytes less `0` (placed); the word at `p25+4` points at 64 random bytes less `0` (placed); the word at `p25+8` points at 64 random bytes less `0` (placed); the word at `p26+0` points at 64 random bytes less `0` (placed); the word at `p26+4` points at 64 random bytes less `0` (placed); the word at `p26+8` points at 64 random bytes less `0` (placed); the word at `p27+0` points at 64 random bytes less `0` (placed); the word at `p27+4` points at 64 random bytes less `0` (placed); the word at `p27+8` points at 64 random bytes less `0` (placed); the word at `p28+0` points at 64 random bytes less `0` (placed); the word at `p28+4` points at 64 random bytes less `0` (placed); the word at `p28+8` points at 64 random bytes less `0` (placed); the word at `p29+0` points at 64 random bytes less `0` (placed); the word at `p29+4` points at 64 random bytes less `0` (placed); the word at `p29+8` points at 64 random bytes less `0` (placed); the word at `p30+0` points at 64 random bytes less `0` (placed); the word at `p30+4` points at 64 random bytes less `0` (placed); the word at `p30+8` points at 64 random bytes less `0` (placed); the word at `p31+0` points at 64 random bytes less `0` (placed); the word at `p31+4` points at 64 random bytes less `0` (placed); the word at `p31+8` points at 64 random bytes less `0` (placed); the word at `p32+0` points at 160 random bytes less `0` (placed); the word at `p32+4` points at 260 random bytes less `0` (placed); the word at `p32+8` points at 160 random bytes less `0` (placed); the word at `p33+0` points at 160 random bytes less `0` (placed); the word at `p33+4` points at 260 random bytes less `0` (placed); the word at `p33+8` points at 160 random bytes less `0` (placed); the word at `p34+0` points at 160 random bytes less `0` (placed); the word at `p34+4` points at 260 random bytes less `0` (placed); the word at `p34+8` points at 160 random bytes less `0` (placed); the word at `p35+0` points at 160 random bytes less `0` (placed); the word at `p35+4` points at 260 random bytes less `0` (placed); the word at `p35+8` points at 160 random bytes less `0` (placed); the word at `p36+0` points at 160 random bytes less `0` (placed); the word at `p36+4` points at 260 random bytes less `0` (placed); the word at `p36+8` points at 160 random bytes less `0` (placed); the word at `p37+0` points at 160 random bytes less `0` (placed); the word at `p37+4` points at 260 random bytes less `0` (placed); the word at `p37+8` points at 160 random bytes less `0` (placed); the word at `p38+0` points at 160 random bytes less `0` (placed); the word at `p38+4` points at 260 random bytes less `0` (placed); the word at `p38+8` points at 160 random bytes less `0` (placed); the word at `p39+0` points at 160 random bytes less `0` (placed); the word at `p39+4` points at 260 random bytes less `0` (placed); the word at `p39+8` points at 160 random bytes less `0` (placed); the word at `c0+0x134` points at 2024 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; the stubbed vtable laid at `p40+0`; stub 2 answers 8 values in call order, printed after the answers; stub 3 answers 8 values in call order, printed after the answers; stub 4 answers 8 values in call order, printed after the answers; stub 2 answers 0..3 every row; stub 3 answers 0..3 every row; stub 4 answers 0..2 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 answers from its own `ecx`: the word at `+0x20`; stub 1 answers from its own `ecx`: the word at `+0x24`; 8 rows 0x1c apart at `@0xc0ab84`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 8 rows 0x1c apart at `@0xc0aec0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); figures p40..p42 link at +0x2c (2 bytes) above their own index or to -1; figures p43..p45 link at +0x2c (2 bytes) above their own index or to -1; figures p46..p48 link at +0x2c (2 bytes) above their own index or to -1; figures p49..p51 link at +0x2c (2 bytes) above their own index or to -1; figures p52..p54 link at +0x2c (2 bytes) above their own index or to -1; figures p55..p57 link at +0x2c (2 bytes) above their own index or to -1; figures p58..p60 link at +0x2c (2 bytes) above their own index or to -1; figures p61..p63 link at +0x2c (2 bytes) above their own index or to -1; figures p40..p42 hold a short at +0x2e drawn from 0..7, after the links; figures p43..p45 hold a short at +0x2e drawn from 0..7, after the links; figures p46..p48 hold a short at +0x2e drawn from 0..7, after the links; figures p49..p51 hold a short at +0x2e drawn from 0..7, after the links; figures p52..p54 hold a short at +0x2e drawn from 0..7, after the links; figures p55..p57 hold a short at +0x2e drawn from 0..7, after the links; figures p58..p60 hold a short at +0x2e drawn from 0..7, after the links; figures p61..p63 hold a short at +0x2e drawn from 0..7, after the links; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p43+0` as well as at `this`; the stubbed vtable laid at `p44+0` as well as at `this`; the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p46+0` as well as at `this`; the stubbed vtable laid at `p47+0` as well as at `this`; the stubbed vtable laid at `p48+0` as well as at `this`; the stubbed vtable laid at `p49+0` as well as at `this`; the stubbed vtable laid at `p50+0` as well as at `this`; the stubbed vtable laid at `p51+0` as well as at `this`; the stubbed vtable laid at `p52+0` as well as at `this`; the stubbed vtable laid at `p53+0` as well as at `this`; the stubbed vtable laid at `p54+0` as well as at `this`; the stubbed vtable laid at `p55+0` as well as at `this`; the stubbed vtable laid at `p56+0` as well as at `this`; the stubbed vtable laid at `p57+0` as well as at `this`; the stubbed vtable laid at `p58+0` as well as at `this`; the stubbed vtable laid at `p59+0` as well as at `this`; the stubbed vtable laid at `p60+0` as well as at `this`; the stubbed vtable laid at `p61+0` as well as at `this`; the stubbed vtable laid at `p62+0` as well as at `this`; the stubbed vtable laid at `p63+0` as well as at `this`; the stubbed vtable laid at `p64+0` as well as at `this`; the stubbed vtable laid at `p65+0` as well as at `this`; the stubbed vtable laid at `p66+0` as well as at `this`; the stubbed vtable laid at `p67+0` as well as at `this`; the stubbed vtable laid at `p68+0` as well as at `this`; the stubbed vtable laid at `p69+0` as well as at `this`; the stubbed vtable laid at `p70+0` as well as at `this`; the stubbed vtable laid at `p71+0` as well as at `this`; the stubbed vtable laid at `p72+0` as well as at `this`; the stubbed vtable laid at `p73+0` as well as at `this`; the stubbed vtable laid at `p74+0` as well as at `this`; the stubbed vtable laid at `p75+0` as well as at `this`; the stubbed vtable laid at `p76+0` as well as at `this`; the stubbed vtable laid at `p77+0` as well as at `this`; the stubbed vtable laid at `p78+0` as well as at `this`; the stubbed vtable laid at `p79+0` as well as at `this`; the stubbed vtable laid at `p80+0` as well as at `this`; the stubbed vtable laid at `p81+0` as well as at `this`; the stubbed vtable laid at `p82+0` as well as at `this`; the stubbed vtable laid at `p83+0` as well as at `this`; the stubbed vtable laid at `p84+0` as well as at `this`; the stubbed vtable laid at `p85+0` as well as at `this`; the stubbed vtable laid at `p86+0` as well as at `this`; the stubbed vtable laid at `p87+0` as well as at `this`.
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

ENTRY = 0x0065ca80
WIN = 1024
OWN = 13
BIAS = []
ARGSET = [(-5, [-1, -1, -1, -1, -1, -1, -1, -1, 0, 1, 50, 100, 200, 300, 768, 769, 1500, 3000, 5000, 6144, 6145, 6911, 6912, 769, 1500, 3000, 5000, 6144, 6145, 6911, 6912]), (-6, [0, 512, 65536, 131072, 66048, 131584, 196608, 197120])]
ARGFROM = []
CELLS = [(12607952, 320), (13297148, 2048), (12607976, 1760)]
AT = [(13361968, 40), (13336208, 16), (13349088, 16)]
ATRANGE = {13361968: (0, 14, 4), 13336208: (-3, 3, 1), 13349088: (-3, 3, 1)}
PTRARGS = []
PNULLS = []
STUBS = [('slot', 8, 0), ('slot', 188, 0), ('call', 6806176, 2), ('call', 6806448, 4), ('slot', 232, 0)]
PLACES = [('p0', '0', '0', 16), ('p1', '0', '0', 16), ('p2', '0', '0', 16), ('p3', '0', '0', 16), ('p4', '0', '0', 16), ('p5', '0', '0', 16), ('p6', '0', '0', 16), ('p7', '0', '0', 16), ('p8', '0', '0', 16), ('p9', '0', '0', 16), ('p10', '0', '0', 16), ('p11', '0', '0', 16), ('p12', '0', '0', 16), ('p13', '0', '0', 16), ('p14', '0', '0', 16), ('p15', '0', '0', 16), ('p24', '0', '0', 64), ('p24', '4', '0', 64), ('p24', '8', '0', 64), ('p25', '0', '0', 64), ('p25', '4', '0', 64), ('p25', '8', '0', 64), ('p26', '0', '0', 64), ('p26', '4', '0', 64), ('p26', '8', '0', 64), ('p27', '0', '0', 64), ('p27', '4', '0', 64), ('p27', '8', '0', 64), ('p28', '0', '0', 64), ('p28', '4', '0', 64), ('p28', '8', '0', 64), ('p29', '0', '0', 64), ('p29', '4', '0', 64), ('p29', '8', '0', 64), ('p30', '0', '0', 64), ('p30', '4', '0', 64), ('p30', '8', '0', 64), ('p31', '0', '0', 64), ('p31', '4', '0', 64), ('p31', '8', '0', 64), ('p32', '0', '0', 160), ('p32', '4', '0', 260), ('p32', '8', '0', 160), ('p33', '0', '0', 160), ('p33', '4', '0', 260), ('p33', '8', '0', 160), ('p34', '0', '0', 160), ('p34', '4', '0', 260), ('p34', '8', '0', 160), ('p35', '0', '0', 160), ('p35', '4', '0', 260), ('p35', '8', '0', 160), ('p36', '0', '0', 160), ('p36', '4', '0', 260), ('p36', '8', '0', 160), ('p37', '0', '0', 160), ('p37', '4', '0', 260), ('p37', '8', '0', 160), ('p38', '0', '0', 160), ('p38', '4', '0', 260), ('p38', '8', '0', 160), ('p39', '0', '0', 160), ('p39', '4', '0', 260), ('p39', '8', '0', 160), ('c0', '0x134', '0', 2024)]
NULLS = []
NULLFORCE = {}
WBIAS = [('p16', 0, ('set', [1, 1, 3, 0, 2])), ('p17', 0, ('set', [1, 1, 3, 0, 2])), ('p18', 0, ('set', [1, 1, 3, 0, 2])), ('p19', 0, ('set', [1, 1, 3, 0, 2])), ('p20', 0, ('set', [1, 1, 3, 0, 2])), ('p21', 0, ('set', [1, 1, 3, 0, 2])), ('p22', 0, ('set', [1, 1, 3, 0, 2])), ('p23', 0, ('set', [1, 1, 3, 0, 2])), ('m', 348, ('range', 0, 3)), ('m', 352, ('range', 0, 3)), ('m', 356, ('range', 0, 3)), ('m', 360, ('range', 0, 3)), ('m', 364, ('range', 0, 3)), ('m', 368, ('range', 0, 3)), ('m', 372, ('range', 0, 3)), ('m', 376, ('range', 0, 3)), ('c0', 0, ('range', 3, 6)), ('c0', 4, ('range', 3, 6)), ('c2', 1752, ('range', 0, 14)), ('p40', 16, ('range', 405504, 407551)), ('p40', 20, ('range', 405504, 407551)), ('p40', 32, ('set', [1, 1, 1, 0])), ('p40', 36, ('set', [1, 1, 1, 0])), ('p41', 16, ('range', 405504, 407551)), ('p41', 20, ('range', 405504, 407551)), ('p41', 32, ('set', [1, 1, 1, 0])), ('p41', 36, ('set', [1, 1, 1, 0])), ('p42', 16, ('range', 405504, 407551)), ('p42', 20, ('range', 405504, 407551)), ('p42', 32, ('set', [1, 1, 1, 0])), ('p42', 36, ('set', [1, 1, 1, 0])), ('p43', 16, ('range', 405504, 407551)), ('p43', 20, ('range', 405504, 407551)), ('p43', 32, ('set', [1, 1, 1, 0])), ('p43', 36, ('set', [1, 1, 1, 0])), ('p44', 16, ('range', 405504, 407551)), ('p44', 20, ('range', 405504, 407551)), ('p44', 32, ('set', [1, 1, 1, 0])), ('p44', 36, ('set', [1, 1, 1, 0])), ('p45', 16, ('range', 405504, 407551)), ('p45', 20, ('range', 405504, 407551)), ('p45', 32, ('set', [1, 1, 1, 0])), ('p45', 36, ('set', [1, 1, 1, 0])), ('p46', 16, ('range', 405504, 407551)), ('p46', 20, ('range', 405504, 407551)), ('p46', 32, ('set', [1, 1, 1, 0])), ('p46', 36, ('set', [1, 1, 1, 0])), ('p47', 16, ('range', 405504, 407551)), ('p47', 20, ('range', 405504, 407551)), ('p47', 32, ('set', [1, 1, 1, 0])), ('p47', 36, ('set', [1, 1, 1, 0])), ('p48', 16, ('range', 405504, 407551)), ('p48', 20, ('range', 405504, 407551)), ('p48', 32, ('set', [1, 1, 1, 0])), ('p48', 36, ('set', [1, 1, 1, 0])), ('p49', 16, ('range', 405504, 407551)), ('p49', 20, ('range', 405504, 407551)), ('p49', 32, ('set', [1, 1, 1, 0])), ('p49', 36, ('set', [1, 1, 1, 0])), ('p50', 16, ('range', 405504, 407551)), ('p50', 20, ('range', 405504, 407551)), ('p50', 32, ('set', [1, 1, 1, 0])), ('p50', 36, ('set', [1, 1, 1, 0])), ('p51', 16, ('range', 405504, 407551)), ('p51', 20, ('range', 405504, 407551)), ('p51', 32, ('set', [1, 1, 1, 0])), ('p51', 36, ('set', [1, 1, 1, 0])), ('p52', 16, ('range', 405504, 407551)), ('p52', 20, ('range', 405504, 407551)), ('p52', 32, ('set', [1, 1, 1, 0])), ('p52', 36, ('set', [1, 1, 1, 0])), ('p53', 16, ('range', 405504, 407551)), ('p53', 20, ('range', 405504, 407551)), ('p53', 32, ('set', [1, 1, 1, 0])), ('p53', 36, ('set', [1, 1, 1, 0])), ('p54', 16, ('range', 405504, 407551)), ('p54', 20, ('range', 405504, 407551)), ('p54', 32, ('set', [1, 1, 1, 0])), ('p54', 36, ('set', [1, 1, 1, 0])), ('p55', 16, ('range', 405504, 407551)), ('p55', 20, ('range', 405504, 407551)), ('p55', 32, ('set', [1, 1, 1, 0])), ('p55', 36, ('set', [1, 1, 1, 0])), ('p56', 16, ('range', 405504, 407551)), ('p56', 20, ('range', 405504, 407551)), ('p56', 32, ('set', [1, 1, 1, 0])), ('p56', 36, ('set', [1, 1, 1, 0])), ('p57', 16, ('range', 405504, 407551)), ('p57', 20, ('range', 405504, 407551)), ('p57', 32, ('set', [1, 1, 1, 0])), ('p57', 36, ('set', [1, 1, 1, 0])), ('p58', 16, ('range', 405504, 407551)), ('p58', 20, ('range', 405504, 407551)), ('p58', 32, ('set', [1, 1, 1, 0])), ('p58', 36, ('set', [1, 1, 1, 0])), ('p59', 16, ('range', 405504, 407551)), ('p59', 20, ('range', 405504, 407551)), ('p59', 32, ('set', [1, 1, 1, 0])), ('p59', 36, ('set', [1, 1, 1, 0])), ('p60', 16, ('range', 405504, 407551)), ('p60', 20, ('range', 405504, 407551)), ('p60', 32, ('set', [1, 1, 1, 0])), ('p60', 36, ('set', [1, 1, 1, 0])), ('p61', 16, ('range', 405504, 407551)), ('p61', 20, ('range', 405504, 407551)), ('p61', 32, ('set', [1, 1, 1, 0])), ('p61', 36, ('set', [1, 1, 1, 0])), ('p62', 16, ('range', 405504, 407551)), ('p62', 20, ('range', 405504, 407551)), ('p62', 32, ('set', [1, 1, 1, 0])), ('p62', 36, ('set', [1, 1, 1, 0])), ('p63', 16, ('range', 405504, 407551)), ('p63', 20, ('range', 405504, 407551)), ('p63', 32, ('set', [1, 1, 1, 0])), ('p63', 36, ('set', [1, 1, 1, 0])), ('p88', 8, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 4, ('set', [0, 1, 2, 3])), ('p88', 36, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 32, ('set', [0, 1, 2, 3])), ('p88', 64, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 60, ('set', [0, 1, 2, 3])), ('p88', 92, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 88, ('set', [0, 1, 2, 3])), ('p88', 120, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 116, ('set', [0, 1, 2, 3])), ('p88', 148, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 144, ('set', [0, 1, 2, 3])), ('p88', 176, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 172, ('set', [0, 1, 2, 3])), ('p88', 204, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 200, ('set', [0, 1, 2, 3])), ('p88', 232, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 228, ('set', [0, 1, 2, 3])), ('p88', 260, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 256, ('set', [0, 1, 2, 3])), ('p88', 288, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 284, ('set', [0, 1, 2, 3])), ('p88', 316, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 312, ('set', [0, 1, 2, 3])), ('p88', 344, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 340, ('set', [0, 1, 2, 3])), ('p88', 372, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 368, ('set', [0, 1, 2, 3])), ('p88', 400, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 396, ('set', [0, 1, 2, 3])), ('p88', 428, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 424, ('set', [0, 1, 2, 3])), ('p88', 456, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 452, ('set', [0, 1, 2, 3])), ('p88', 484, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 480, ('set', [0, 1, 2, 3])), ('p88', 512, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 508, ('set', [0, 1, 2, 3])), ('p88', 540, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 536, ('set', [0, 1, 2, 3])), ('p88', 568, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 564, ('set', [0, 1, 2, 3])), ('p88', 596, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 592, ('set', [0, 1, 2, 3])), ('p88', 624, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 620, ('set', [0, 1, 2, 3])), ('p88', 652, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 648, ('set', [0, 1, 2, 3])), ('p88', 680, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 676, ('set', [0, 1, 2, 3])), ('p88', 708, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 704, ('set', [0, 1, 2, 3])), ('p88', 736, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 732, ('set', [0, 1, 2, 3])), ('p88', 764, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 760, ('set', [0, 1, 2, 3])), ('p88', 792, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 788, ('set', [0, 1, 2, 3])), ('p88', 820, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 816, ('set', [0, 1, 2, 3])), ('p88', 848, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 844, ('set', [0, 1, 2, 3])), ('p88', 876, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 872, ('set', [0, 1, 2, 3])), ('p88', 904, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 900, ('set', [0, 1, 2, 3])), ('p88', 932, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 928, ('set', [0, 1, 2, 3])), ('p88', 960, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 956, ('set', [0, 1, 2, 3])), ('p88', 988, ('set', [65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 0, 1, 2, 65536, 65537, 65538, 131072, 131073, 131074, 196608, 196609, 196610, 262144, 262145, 262146, 327680, 327681, 327682, 393216, 393217, 393218, 458752, 458753, 458754])), ('p88', 984, ('set', [0, 1, 2, 3]))]
FILLS = [('c1', 'i//3')]
ARGRANGE = [(0, 0, 2047), (1, 0, 2047), (2, -1, 7), (3, -1, 7), (6, 0, 25), (9, 0, 25), (7, -1, 7), (8, -1, 7), (10, -1, 7), (11, -1, 7)]
SLOTBASE = [(232, 4640432)]
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = []
VTAT = ('p40', 0)
VTATHALF = []
VTIMAGE = []
ALIAS = []
ANSWERBY = []
ANSWERWIN = []
CDECL = False
TIB = False
ARGEQ = []
SEQ = [(2, 8, None), (3, 8, None), (4, 8, None)]
ANSRANGE = [(2, 0, 3), (3, 0, 3), (4, 0, 2)]
CELLTHIS = []
VTALSO = [('p41', 0), ('p42', 0), ('p43', 0), ('p44', 0), ('p45', 0), ('p46', 0), ('p47', 0), ('p48', 0), ('p49', 0), ('p50', 0), ('p51', 0), ('p52', 0), ('p53', 0), ('p54', 0), ('p55', 0), ('p56', 0), ('p57', 0), ('p58', 0), ('p59', 0), ('p60', 0), ('p61', 0), ('p62', 0), ('p63', 0), ('p64', 0), ('p65', 0), ('p66', 0), ('p67', 0), ('p68', 0), ('p69', 0), ('p70', 0), ('p71', 0), ('p72', 0), ('p73', 0), ('p74', 0), ('p75', 0), ('p76', 0), ('p77', 0), ('p78', 0), ('p79', 0), ('p80', 0), ('p81', 0), ('p82', 0), ('p83', 0), ('p84', 0), ('p85', 0), ('p86', 0), ('p87', 0)]
SPARSE = [('@', '0xc0ab84', 28, 8, [(0, 4)]), ('@', '0xc0aec0', 28, 8, [(0, 4)]), (None, '0xc061e0', 28396, 8, [(0, 4)])]
GTAB = []
RECORD = [0, 1, 2, 3, 4]
RECORDORDER = True
RECORDEDX = False
ROWSPLIT = {}
LINKS = [(40, 42, 44, 2, '>', [-1]), (43, 45, 44, 2, '>', [-1]), (46, 48, 44, 2, '>', [-1]), (49, 51, 44, 2, '>', [-1]), (52, 54, 44, 2, '>', [-1]), (55, 57, 44, 2, '>', [-1]), (58, 60, 44, 2, '>', [-1]), (61, 63, 44, 2, '>', [-1])]
LINKHI = [(40, 42, 46, ('range', 0, 7)), (43, 45, 46, ('range', 0, 7)), (46, 48, 46, ('range', 0, 7)), (49, 51, 46, ('range', 0, 7)), (52, 54, 46, ('range', 0, 7)), (55, 57, 46, ('range', 0, 7)), (58, 60, 46, ('range', 0, 7)), (61, 63, 46, ('range', 0, 7))]
OUTARG = []
CDECLSTUBS = []
STUBWRITE = []
STUBCOPY = []
STACKCLEAN = 0
INITCALLS = []
MAXINSNS = 200000
ECXANSWER = [(0, 32, None), (1, 36, None)]
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
    rng = random.Random(3339)
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

# sweep_gen: ["obj", "ObjectsData::find_unit@0065ca80", "--win", "1024", "--own", "13", "--cell-win", "0xc061d0:320", "--cell-win", "0xcae5fc:2048", "--cell-win", "0xc061e8:1760", "--at", "0xcbe330:40:0..14", "--at", "0xcb7e90:16:-3..3/b", "--at", "0xcbb0e0:16:-3..3/b", "--sparse", "@0xc0ab84:28:8:0,4", "--sparse", "@0xc0aec0:28:8:0,4", "--sparse", "0xc061e0:28396:8:0,4", "--stub", "slot:8:0", "--stub", "slot:0xbc:0", "--stub", "0x67daa0:2", "--stub", "0x67dbb0:4", "--stub", "slot:0xe8:0", "--place", "p0+0:0:16", "--place", "p1+0:0:16", "--place", "p2+0:0:16", "--place", "p3+0:0:16", "--place", "p4+0:0:16", "--place", "p5+0:0:16", "--place", "p6+0:0:16", "--place", "p7+0:0:16", "--place", "p8+0:0:16", "--place", "p9+0:0:16", "--place", "p10+0:0:16", "--place", "p11+0:0:16", "--place", "p12+0:0:16", "--place", "p13+0:0:16", "--place", "p14+0:0:16", "--place", "p15+0:0:16", "--place", "p24+0:0:64", "--place", "p24+4:0:64", "--place", "p24+8:0:64", "--place", "p25+0:0:64", "--place", "p25+4:0:64", "--place", "p25+8:0:64", "--place", "p26+0:0:64", "--place", "p26+4:0:64", "--place", "p26+8:0:64", "--place", "p27+0:0:64", "--place", "p27+4:0:64", "--place", "p27+8:0:64", "--place", "p28+0:0:64", "--place", "p28+4:0:64", "--place", "p28+8:0:64", "--place", "p29+0:0:64", "--place", "p29+4:0:64", "--place", "p29+8:0:64", "--place", "p30+0:0:64", "--place", "p30+4:0:64", "--place", "p30+8:0:64", "--place", "p31+0:0:64", "--place", "p31+4:0:64", "--place", "p31+8:0:64", "--place", "p32+0:0:160", "--place", "p32+4:0:260", "--place", "p32+8:0:160", "--place", "p33+0:0:160", "--place", "p33+4:0:260", "--place", "p33+8:0:160", "--place", "p34+0:0:160", "--place", "p34+4:0:260", "--place", "p34+8:0:160", "--place", "p35+0:0:160", "--place", "p35+4:0:260", "--place", "p35+8:0:160", "--place", "p36+0:0:160", "--place", "p36+4:0:260", "--place", "p36+8:0:160", "--place", "p37+0:0:160", "--place", "p37+4:0:260", "--place", "p37+8:0:160", "--place", "p38+0:0:160", "--place", "p38+4:0:260", "--place", "p38+8:0:160", "--place", "p39+0:0:160", "--place", "p39+4:0:260", "--place", "p39+8:0:160", "--place", "c0+0x134:0:2024", "--wbias", "p16+0:1,1,3,0,2", "--wbias", "p17+0:1,1,3,0,2", "--wbias", "p18+0:1,1,3,0,2", "--wbias", "p19+0:1,1,3,0,2", "--wbias", "p20+0:1,1,3,0,2", "--wbias", "p21+0:1,1,3,0,2", "--wbias", "p22+0:1,1,3,0,2", "--wbias", "p23+0:1,1,3,0,2", "--wbias", "m+348:0..3", "--wbias", "m+352:0..3", "--wbias", "m+356:0..3", "--wbias", "m+360:0..3", "--wbias", "m+364:0..3", "--wbias", "m+368:0..3", "--wbias", "m+372:0..3", "--wbias", "m+376:0..3", "--wbias", "c0+0:3..6", "--wbias", "c0+4:3..6", "--wbias", "c2+1752:0..14", "--wbias", "p40+16:405504..407551", "--wbias", "p40+20:405504..407551", "--wbias", "p40+32:1,1,1,0", "--wbias", "p40+36:1,1,1,0", "--wbias", "p41+16:405504..407551", "--wbias", "p41+20:405504..407551", "--wbias", "p41+32:1,1,1,0", "--wbias", "p41+36:1,1,1,0", "--wbias", "p42+16:405504..407551", "--wbias", "p42+20:405504..407551", "--wbias", "p42+32:1,1,1,0", "--wbias", "p42+36:1,1,1,0", "--wbias", "p43+16:405504..407551", "--wbias", "p43+20:405504..407551", "--wbias", "p43+32:1,1,1,0", "--wbias", "p43+36:1,1,1,0", "--wbias", "p44+16:405504..407551", "--wbias", "p44+20:405504..407551", "--wbias", "p44+32:1,1,1,0", "--wbias", "p44+36:1,1,1,0", "--wbias", "p45+16:405504..407551", "--wbias", "p45+20:405504..407551", "--wbias", "p45+32:1,1,1,0", "--wbias", "p45+36:1,1,1,0", "--wbias", "p46+16:405504..407551", "--wbias", "p46+20:405504..407551", "--wbias", "p46+32:1,1,1,0", "--wbias", "p46+36:1,1,1,0", "--wbias", "p47+16:405504..407551", "--wbias", "p47+20:405504..407551", "--wbias", "p47+32:1,1,1,0", "--wbias", "p47+36:1,1,1,0", "--wbias", "p48+16:405504..407551", "--wbias", "p48+20:405504..407551", "--wbias", "p48+32:1,1,1,0", "--wbias", "p48+36:1,1,1,0", "--wbias", "p49+16:405504..407551", "--wbias", "p49+20:405504..407551", "--wbias", "p49+32:1,1,1,0", "--wbias", "p49+36:1,1,1,0", "--wbias", "p50+16:405504..407551", "--wbias", "p50+20:405504..407551", "--wbias", "p50+32:1,1,1,0", "--wbias", "p50+36:1,1,1,0", "--wbias", "p51+16:405504..407551", "--wbias", "p51+20:405504..407551", "--wbias", "p51+32:1,1,1,0", "--wbias", "p51+36:1,1,1,0", "--wbias", "p52+16:405504..407551", "--wbias", "p52+20:405504..407551", "--wbias", "p52+32:1,1,1,0", "--wbias", "p52+36:1,1,1,0", "--wbias", "p53+16:405504..407551", "--wbias", "p53+20:405504..407551", "--wbias", "p53+32:1,1,1,0", "--wbias", "p53+36:1,1,1,0", "--wbias", "p54+16:405504..407551", "--wbias", "p54+20:405504..407551", "--wbias", "p54+32:1,1,1,0", "--wbias", "p54+36:1,1,1,0", "--wbias", "p55+16:405504..407551", "--wbias", "p55+20:405504..407551", "--wbias", "p55+32:1,1,1,0", "--wbias", "p55+36:1,1,1,0", "--wbias", "p56+16:405504..407551", "--wbias", "p56+20:405504..407551", "--wbias", "p56+32:1,1,1,0", "--wbias", "p56+36:1,1,1,0", "--wbias", "p57+16:405504..407551", "--wbias", "p57+20:405504..407551", "--wbias", "p57+32:1,1,1,0", "--wbias", "p57+36:1,1,1,0", "--wbias", "p58+16:405504..407551", "--wbias", "p58+20:405504..407551", "--wbias", "p58+32:1,1,1,0", "--wbias", "p58+36:1,1,1,0", "--wbias", "p59+16:405504..407551", "--wbias", "p59+20:405504..407551", "--wbias", "p59+32:1,1,1,0", "--wbias", "p59+36:1,1,1,0", "--wbias", "p60+16:405504..407551", "--wbias", "p60+20:405504..407551", "--wbias", "p60+32:1,1,1,0", "--wbias", "p60+36:1,1,1,0", "--wbias", "p61+16:405504..407551", "--wbias", "p61+20:405504..407551", "--wbias", "p61+32:1,1,1,0", "--wbias", "p61+36:1,1,1,0", "--wbias", "p62+16:405504..407551", "--wbias", "p62+20:405504..407551", "--wbias", "p62+32:1,1,1,0", "--wbias", "p62+36:1,1,1,0", "--wbias", "p63+16:405504..407551", "--wbias", "p63+20:405504..407551", "--wbias", "p63+32:1,1,1,0", "--wbias", "p63+36:1,1,1,0", "--wbias", "p88+8:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+4:0,1,2,3", "--wbias", "p88+36:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+32:0,1,2,3", "--wbias", "p88+64:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+60:0,1,2,3", "--wbias", "p88+92:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+88:0,1,2,3", "--wbias", "p88+120:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+116:0,1,2,3", "--wbias", "p88+148:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+144:0,1,2,3", "--wbias", "p88+176:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+172:0,1,2,3", "--wbias", "p88+204:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+200:0,1,2,3", "--wbias", "p88+232:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+228:0,1,2,3", "--wbias", "p88+260:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+256:0,1,2,3", "--wbias", "p88+288:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+284:0,1,2,3", "--wbias", "p88+316:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+312:0,1,2,3", "--wbias", "p88+344:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+340:0,1,2,3", "--wbias", "p88+372:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+368:0,1,2,3", "--wbias", "p88+400:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+396:0,1,2,3", "--wbias", "p88+428:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+424:0,1,2,3", "--wbias", "p88+456:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+452:0,1,2,3", "--wbias", "p88+484:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+480:0,1,2,3", "--wbias", "p88+512:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+508:0,1,2,3", "--wbias", "p88+540:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+536:0,1,2,3", "--wbias", "p88+568:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+564:0,1,2,3", "--wbias", "p88+596:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+592:0,1,2,3", "--wbias", "p88+624:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+620:0,1,2,3", "--wbias", "p88+652:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+648:0,1,2,3", "--wbias", "p88+680:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+676:0,1,2,3", "--wbias", "p88+708:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+704:0,1,2,3", "--wbias", "p88+736:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+732:0,1,2,3", "--wbias", "p88+764:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+760:0,1,2,3", "--wbias", "p88+792:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+788:0,1,2,3", "--wbias", "p88+820:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+816:0,1,2,3", "--wbias", "p88+848:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+844:0,1,2,3", "--wbias", "p88+876:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+872:0,1,2,3", "--wbias", "p88+904:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+900:0,1,2,3", "--wbias", "p88+932:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+928:0,1,2,3", "--wbias", "p88+960:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+956:0,1,2,3", "--wbias", "p88+988:65535,65535,65535,65535,65535,65535,65535,65535,0,1,2,65536,65537,65538,131072,131073,131074,196608,196609,196610,262144,262145,262146,327680,327681,327682,393216,393217,393218,458752,458753,458754", "--wbias", "p88+984:0,1,2,3", "--fill", "c1:i//3", "--argrange", "0:0:2047", "--argrange", "1:0:2047", "--argrange", "2:-1:7", "--argrange", "3:-1:7", "--argrange", "6:0:25", "--argrange", "9:0:25", "--argrange", "7:-1:7", "--argrange", "8:-1:7", "--argrange", "10:-1:7", "--argrange", "11:-1:7", "--argset", "4!:-1,-1,-1,-1,-1,-1,-1,-1,0,1,50,100,200,300,768,769,1500,3000,5000,6144,6145,6911,6912,769,1500,3000,5000,6144,6145,6911,6912", "--argset", "5!:0,512,65536,131072,66048,131584,196608,197120", "--ecx-answer", "0:0x20", "--ecx-answer", "1:0x24", "--seq", "2:8", "--answer-range", "2:0:3", "--seq", "3:8", "--answer-range", "3:0:3", "--seq", "4:8", "--answer-range", "4:0:2", "--record", "0", "--record", "1", "--record", "2", "--record", "3", "--record", "4", "--record-order", "--vtat", "p40+0", "--vtat-also", "p41+0", "--vtat-also", "p42+0", "--vtat-also", "p43+0", "--vtat-also", "p44+0", "--vtat-also", "p45+0", "--vtat-also", "p46+0", "--vtat-also", "p47+0", "--vtat-also", "p48+0", "--vtat-also", "p49+0", "--vtat-also", "p50+0", "--vtat-also", "p51+0", "--vtat-also", "p52+0", "--vtat-also", "p53+0", "--vtat-also", "p54+0", "--vtat-also", "p55+0", "--vtat-also", "p56+0", "--vtat-also", "p57+0", "--vtat-also", "p58+0", "--vtat-also", "p59+0", "--vtat-also", "p60+0", "--vtat-also", "p61+0", "--vtat-also", "p62+0", "--vtat-also", "p63+0", "--vtat-also", "p64+0", "--vtat-also", "p65+0", "--vtat-also", "p66+0", "--vtat-also", "p67+0", "--vtat-also", "p68+0", "--vtat-also", "p69+0", "--vtat-also", "p70+0", "--vtat-also", "p71+0", "--vtat-also", "p72+0", "--vtat-also", "p73+0", "--vtat-also", "p74+0", "--vtat-also", "p75+0", "--vtat-also", "p76+0", "--vtat-also", "p77+0", "--vtat-also", "p78+0", "--vtat-also", "p79+0", "--vtat-also", "p80+0", "--vtat-also", "p81+0", "--vtat-also", "p82+0", "--vtat-also", "p83+0", "--vtat-also", "p84+0", "--vtat-also", "p85+0", "--vtat-also", "p86+0", "--vtat-also", "p87+0", "--vtat-keep", "0xe8:0x46ceb0", "--link", "p40..42+44:2:>:-1", "--link-hi", "p40..42+46:0..7", "--link", "p43..45+44:2:>:-1", "--link-hi", "p43..45+46:0..7", "--link", "p46..48+44:2:>:-1", "--link-hi", "p46..48+46:0..7", "--link", "p49..51+44:2:>:-1", "--link-hi", "p49..51+46:0..7", "--link", "p52..54+44:2:>:-1", "--link-hi", "p52..54+46:0..7", "--link", "p55..57+44:2:>:-1", "--link-hi", "p55..57+46:0..7", "--link", "p58..60+44:2:>:-1", "--link-hi", "p58..60+46:0..7", "--link", "p61..63+44:2:>:-1", "--link-hi", "p61..63+46:0..7", "--rows", "3000", "--item", "3339"]
