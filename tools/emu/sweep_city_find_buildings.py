#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_city_find_buildings.py — `City::find_buildings@007384c0` under unicorn (item 3304, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_city_find_buildings.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 128-byte object of random bytes (no bias), 0 stack argument(s);
`p` windows in order: p0..p7 sparse table 0 (8 rows × 1 slice(s), row-major), p8..p73 the place(s), in flag order, p74 stub 9's answer window; the global cell 0x00c0618c points at 504 random bytes; the global cell 0x00c06198 points at 16 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c06210 points at 672 random bytes; vtable slot +0xac (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; callee 0x006db790 (1 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; callee 0x00627fe0 (0 args) stubbed; callee 0x00622380 (1 args) stubbed; vtable slot +0x94 (1 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0x10c (1 args) stubbed; callee 0x00a1cd60 (2 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; vtable slot +0x1a8 (3 args) stubbed; vtable slot +0x150 (3 args) stubbed; callee 0x00642fc0 (2 args) stubbed; callee 0x00738b20 (0 args) stubbed; callee 0x00647080 (3 args) stubbed; vtable slot +0x158 (3 args) stubbed; the word at `c0+0x14+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p8+0` points at 36 random bytes less `0` (placed); the word at `p8+4` points at 36 random bytes less `0` (placed); the word at `p8+8` points at 36 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p12+0` points at 36 random bytes less `0` (placed); the word at `p12+4` points at 36 random bytes less `0` (placed); the word at `p12+8` points at 36 random bytes less `0` (placed); the word at `c0+0x14+0x1c*2` points at 12 random bytes less `0` (placed); the word at `p16+0` points at 36 random bytes less `0` (placed); the word at `p16+4` points at 36 random bytes less `0` (placed); the word at `p16+8` points at 36 random bytes less `0` (placed); the word at `c0+0x14+0x1c*3` points at 12 random bytes less `0` (placed); the word at `p20+0` points at 36 random bytes less `0` (placed); the word at `p20+4` points at 36 random bytes less `0` (placed); the word at `p20+8` points at 36 random bytes less `0` (placed); the word at `c0+0x14+0x1c*4` points at 12 random bytes less `0` (placed); the word at `p24+0` points at 36 random bytes less `0` (placed); the word at `p24+4` points at 36 random bytes less `0` (placed); the word at `p24+8` points at 36 random bytes less `0` (placed); the word at `c0+0x14+0x1c*5` points at 12 random bytes less `0` (placed); the word at `p28+0` points at 36 random bytes less `0` (placed); the word at `p28+4` points at 36 random bytes less `0` (placed); the word at `p28+8` points at 36 random bytes less `0` (placed); the word at `c0+0x14+0x1c*6` points at 12 random bytes less `0` (placed); the word at `p32+0` points at 36 random bytes less `0` (placed); the word at `p32+4` points at 36 random bytes less `0` (placed); the word at `p32+8` points at 36 random bytes less `0` (placed); the word at `c0+0x14+0x1c*7` points at 12 random bytes less `0` (placed); the word at `p36+0` points at 36 random bytes less `0` (placed); the word at `p36+4` points at 36 random bytes less `0` (placed); the word at `p36+8` points at 36 random bytes less `0` (placed); the word at `0xc0ab84+u8(m,0x5e)*0x1c` points at 12 random bytes less `0` (placed); the word at `p40+0` points at 32 random bytes less `0` (placed); the word at `p40+4` points at 32 random bytes less `0` (placed); the word at `p40+8` points at 32 random bytes less `0` (placed); the word at `p9+0x20` points at 128 random bytes less `0` (placed); the word at `p10+0x20` points at 128 random bytes less `0` (placed); the word at `p11+0x20` points at 128 random bytes less `0` (placed); the word at `p13+0x20` points at 128 random bytes less `0` (placed); the word at `p14+0x20` points at 128 random bytes less `0` (placed); the word at `p15+0x20` points at 128 random bytes less `0` (placed); the word at `p17+0x20` points at 128 random bytes less `0` (placed); the word at `p18+0x20` points at 128 random bytes less `0` (placed); the word at `p19+0x20` points at 128 random bytes less `0` (placed); the word at `p21+0x20` points at 128 random bytes less `0` (placed); the word at `p22+0x20` points at 128 random bytes less `0` (placed); the word at `p23+0x20` points at 128 random bytes less `0` (placed); the word at `p25+0x20` points at 128 random bytes less `0` (placed); the word at `p26+0x20` points at 128 random bytes less `0` (placed); the word at `p27+0x20` points at 128 random bytes less `0` (placed); the word at `p29+0x20` points at 128 random bytes less `0` (placed); the word at `p30+0x20` points at 128 random bytes less `0` (placed); the word at `p31+0x20` points at 128 random bytes less `0` (placed); the word at `p33+0x20` points at 128 random bytes less `0` (placed); the word at `p34+0x20` points at 128 random bytes less `0` (placed); the word at `p35+0x20` points at 128 random bytes less `0` (placed); the word at `p37+0x20` points at 128 random bytes less `0` (placed); the word at `p38+0x20` points at 128 random bytes less `0` (placed); the word at `p39+0x20` points at 128 random bytes less `0` (placed); the word at `m+0x70` points at 712 random bytes less `0` (placed); the word at `m+0x74` points at 712 random bytes less `0` (placed); the word at `m+0x78` points at 712 random bytes less `0` (placed); the word at `m+0x7c` points at 712 random bytes less `0` (placed); the word at `c0+0x1ec` points at 32 random bytes less `0` (placed); the word at `c0+0x1f0` points at 32 random bytes less `0` (placed); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 answered from [0, 1, 2, 3, 5, 8, 10, 15, 20, 30, 40, 63, 64, 65, 100, 64, 65, 66, 100, 100] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 9 answers the address of 128 random bytes, a `p` window after the places; the word at `p44+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p45+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p46+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p47+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p48+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p49+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p50+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p51+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p52+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p53+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p54+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p55+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p56+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p57+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p58+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p59+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p60+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p61+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p62+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p63+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p64+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p65+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p66+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p67+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p44+0x18` laid as the address of `p68` every row; the word at `p45+0x18` laid as the address of `p69` every row; the word at `p46+0x18` laid as the address of `p70` every row; the word at `p47+0x18` laid as the address of `p71` every row; the word at `p48+0x18` laid as the address of `p68` every row; the word at `p49+0x18` laid as the address of `p69` every row; the word at `p50+0x18` laid as the address of `p70` every row; the word at `p51+0x18` laid as the address of `p71` every row; the word at `p52+0x18` laid as the address of `p68` every row; the word at `p53+0x18` laid as the address of `p69` every row; the word at `p54+0x18` laid as the address of `p70` every row; the word at `p55+0x18` laid as the address of `p71` every row; the word at `p56+0x18` laid as the address of `p68` every row; the word at `p57+0x18` laid as the address of `p69` every row; the word at `p58+0x18` laid as the address of `p70` every row; the word at `p59+0x18` laid as the address of `p71` every row; the word at `p60+0x18` laid as the address of `p68` every row; the word at `p61+0x18` laid as the address of `p69` every row; the word at `p62+0x18` laid as the address of `p70` every row; the word at `p63+0x18` laid as the address of `p71` every row; the word at `p64+0x18` laid as the address of `p68` every row; the word at `p65+0x18` laid as the address of `p69` every row; the word at `p66+0x18` laid as the address of `p70` every row; the word at `p67+0x18` laid as the address of `p71` every row; the word at `p41+0x18` laid as the address of `p68` every row; the word at `p42+0x18` laid as the address of `p68` every row; the word at `p43+0x18` laid as the address of `p68` every row; stub 1 answers 12 values in call order, printed after the answers; stub 7 answers 8 values in call order, printed after the answers; stub 7 answers -2..2 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 answers from its own `ecx`: the word at `+0x20`; 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p44+0` as well as at `this`; the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p46+0` as well as at `this`; the stubbed vtable laid at `p47+0` as well as at `this`; the stubbed vtable laid at `p48+0` as well as at `this`; the stubbed vtable laid at `p49+0` as well as at `this`; the stubbed vtable laid at `p50+0` as well as at `this`; the stubbed vtable laid at `p51+0` as well as at `this`; the stubbed vtable laid at `p52+0` as well as at `this`; the stubbed vtable laid at `p53+0` as well as at `this`; the stubbed vtable laid at `p54+0` as well as at `this`; the stubbed vtable laid at `p55+0` as well as at `this`; the stubbed vtable laid at `p56+0` as well as at `this`; the stubbed vtable laid at `p57+0` as well as at `this`; the stubbed vtable laid at `p58+0` as well as at `this`; the stubbed vtable laid at `p59+0` as well as at `this`; the stubbed vtable laid at `p60+0` as well as at `this`; the stubbed vtable laid at `p61+0` as well as at `this`; the stubbed vtable laid at `p62+0` as well as at `this`; the stubbed vtable laid at `p63+0` as well as at `this`; the stubbed vtable laid at `p64+0` as well as at `this`; the stubbed vtable laid at `p65+0` as well as at `this`; the stubbed vtable laid at `p66+0` as well as at `this`; the stubbed vtable laid at `p67+0` as well as at `this`; the stubbed vtable laid at `p68+0` as well as at `this`; the stubbed vtable laid at `p69+0` as well as at `this`; the stubbed vtable laid at `p70+0` as well as at `this`; the stubbed vtable laid at `p71+0` as well as at `this`; the stubbed vtable laid at `p74+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`.
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

ENTRY = 0x007384c0
WIN = 128
OWN = 0
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607884, 504), (12607896, 16), (13297148, 1024), (12608016, 672)]
AT = []
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('slot', 172, 0), ('slot', 16, 0), ('call', 7190416, 1), ('slot', 96, 2), ('slot', 184, 2), ('call', 6455264, 0), ('call', 6431616, 1), ('slot', 148, 1), ('call', 10612400, 1), ('slot', 176, 0), ('slot', 268, 1), ('call', 10603872, 2), ('call', 10604352, 0), ('call', 8298416, 10), ('slot', 424, 3), ('slot', 336, 3), ('call', 6565824, 2), ('call', 7572256, 0), ('call', 6582400, 3), ('slot', 344, 3)]
PLACES = [('c0', '0x14+0x1c*0', '0', 12), ('p8', '0', '0', 36), ('p8', '4', '0', 36), ('p8', '8', '0', 36), ('c0', '0x14+0x1c*1', '0', 12), ('p12', '0', '0', 36), ('p12', '4', '0', 36), ('p12', '8', '0', 36), ('c0', '0x14+0x1c*2', '0', 12), ('p16', '0', '0', 36), ('p16', '4', '0', 36), ('p16', '8', '0', 36), ('c0', '0x14+0x1c*3', '0', 12), ('p20', '0', '0', 36), ('p20', '4', '0', 36), ('p20', '8', '0', 36), ('c0', '0x14+0x1c*4', '0', 12), ('p24', '0', '0', 36), ('p24', '4', '0', 36), ('p24', '8', '0', 36), ('c0', '0x14+0x1c*5', '0', 12), ('p28', '0', '0', 36), ('p28', '4', '0', 36), ('p28', '8', '0', 36), ('c0', '0x14+0x1c*6', '0', 12), ('p32', '0', '0', 36), ('p32', '4', '0', 36), ('p32', '8', '0', 36), ('c0', '0x14+0x1c*7', '0', 12), ('p36', '0', '0', 36), ('p36', '4', '0', 36), ('p36', '8', '0', 36), (None, '0xc0ab84+u8(m,0x5e)*0x1c', '0', 12), ('p40', '0', '0', 32), ('p40', '4', '0', 32), ('p40', '8', '0', 32), ('p9', '0x20', '0', 128), ('p10', '0x20', '0', 128), ('p11', '0x20', '0', 128), ('p13', '0x20', '0', 128), ('p14', '0x20', '0', 128), ('p15', '0x20', '0', 128), ('p17', '0x20', '0', 128), ('p18', '0x20', '0', 128), ('p19', '0x20', '0', 128), ('p21', '0x20', '0', 128), ('p22', '0x20', '0', 128), ('p23', '0x20', '0', 128), ('p25', '0x20', '0', 128), ('p26', '0x20', '0', 128), ('p27', '0x20', '0', 128), ('p29', '0x20', '0', 128), ('p30', '0x20', '0', 128), ('p31', '0x20', '0', 128), ('p33', '0x20', '0', 128), ('p34', '0x20', '0', 128), ('p35', '0x20', '0', 128), ('p37', '0x20', '0', 128), ('p38', '0x20', '0', 128), ('p39', '0x20', '0', 128), ('m', '0x70', '0', 712), ('m', '0x74', '0', 712), ('m', '0x78', '0', 712), ('m', '0x7c', '0', 712), ('c0', '0x1ec', '0', 32), ('c0', '0x1f0', '0', 32)]
NULLS = []
NULLFORCE = {}
WBIAS = [('p9', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p9', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p9', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p44', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p10', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p10', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p10', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p45', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p11', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p11', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p11', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p46', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p13', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p13', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p13', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p47', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p14', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p14', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p14', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p48', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p15', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p15', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p15', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p49', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p17', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p17', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p17', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p50', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p18', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p18', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p18', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p51', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p19', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p19', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p19', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p52', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p21', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p21', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p21', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p53', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p22', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p22', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p22', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p54', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p23', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p23', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p23', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p55', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p25', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p25', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p25', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p56', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p26', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p26', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p26', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p57', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p27', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p27', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p27', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p58', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p29', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p29', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p29', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p59', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p30', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p30', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p30', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p60', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p31', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p31', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p31', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p61', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p33', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p33', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p33', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p62', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p34', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p34', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p34', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p63', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p35', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p35', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p35', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p64', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p37', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p37', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p37', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p65', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p38', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p38', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p38', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p66', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p39', 16, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p39', 20, ('set', [407095, 407123, 407176, 407287, 407315, 407368, 407479, 407507, 406536, 406647, 406675, 406728, 407031, 406035, 406088, 405559, 405587, 405640, 409399, 409427, 409480, 408119, 408147, 408200, 407863, 407891, 407944, 402679, 402707, 402760, 401591, 401619, 401672, 403511, 403539, 403592, 398263, 398291, 397320, 400183, 400211, 400264, 394807, 394835, 394888, 394999, 395027, 395080, 395191, 395219, 394248, 393911, 393939, 393992, 395831, 395859, 395912])), ('p39', 8, ('set', [0, 0, 0, 32, 4096, 0])), ('p67', 112, ('set', [4294901760, 4294901760, 2147483648, 65536, 0, 327680])), ('p68', 704, ('set', [16, 0, 16, 0, 31, 32, 48])), ('p69', 704, ('set', [16, 0, 16, 0, 31, 32, 48])), ('p70', 704, ('set', [16, 0, 16, 0, 31, 32, 48])), ('p71', 704, ('set', [16, 0, 16, 0, 31, 32, 48])), ('p0', 0, ('set', [0, 1, 1, 1, 3, 257])), ('p1', 0, ('set', [0, 1, 1, 1, 3, 257])), ('p2', 0, ('set', [0, 1, 1, 1, 3, 257])), ('p72', None, ('set', [0, 1, 2, 3, 3])), ('p73', None, ('set', [0, 1, 2, 3, 3])), ('c1', 4, ('set', [0, 0, 1, 2])), ('c1', 8, ('set', [0, 0, 1, 2])), ('m', 92, ('set', [0, 65536, 131072])), ('m', 8, ('set', [0, 1, 2])), ('c3', 664, ('set', [0, 1, 2, 3, -1]))]
FILLS = [('c2', 'i//3')]
ARGRANGE = []
SLOTBASE = []
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(1, [1, 1, 1, 0]), (2, [0, 1, 2, 3, 5, 8, 10, 15, 20, 30, 40, 63, 64, 65, 100, 64, 65, 66, 100, 100]), (3, [0, 1]), (4, [0, 1])]
VTAT = None
VTATHALF = []
VTIMAGE = [('p44', 0, 11805044), ('p45', 0, 11805044), ('p46', 0, 11805044), ('p47', 0, 11805044), ('p48', 0, 11805044), ('p49', 0, 11805044), ('p50', 0, 11805044), ('p51', 0, 11805044), ('p52', 0, 11805044), ('p53', 0, 11805044), ('p54', 0, 11805044), ('p55', 0, 11805044), ('p56', 0, 11805044), ('p57', 0, 11805044), ('p58', 0, 11805044), ('p59', 0, 11805044), ('p60', 0, 11805044), ('p61', 0, 11805044), ('p62', 0, 11805044), ('p63', 0, 11805044), ('p64', 0, 11805044), ('p65', 0, 11805044), ('p66', 0, 11805044), ('p67', 0, 11805044)]
ALIAS = [('p44', 24, 'p68', 0, True), ('p45', 24, 'p69', 0, True), ('p46', 24, 'p70', 0, True), ('p47', 24, 'p71', 0, True), ('p48', 24, 'p68', 0, True), ('p49', 24, 'p69', 0, True), ('p50', 24, 'p70', 0, True), ('p51', 24, 'p71', 0, True), ('p52', 24, 'p68', 0, True), ('p53', 24, 'p69', 0, True), ('p54', 24, 'p70', 0, True), ('p55', 24, 'p71', 0, True), ('p56', 24, 'p68', 0, True), ('p57', 24, 'p69', 0, True), ('p58', 24, 'p70', 0, True), ('p59', 24, 'p71', 0, True), ('p60', 24, 'p68', 0, True), ('p61', 24, 'p69', 0, True), ('p62', 24, 'p70', 0, True), ('p63', 24, 'p71', 0, True), ('p64', 24, 'p68', 0, True), ('p65', 24, 'p69', 0, True), ('p66', 24, 'p70', 0, True), ('p67', 24, 'p71', 0, True), ('p41', 24, 'p68', 0, True), ('p42', 24, 'p68', 0, True), ('p43', 24, 'p68', 0, True)]
ANSWERBY = []
ANSWERWIN = [(9, 128)]
CDECL = False
TIB = True
ARGEQ = []
SEQ = [(1, 12, None), (7, 8, None)]
ANSRANGE = [(7, -2, 2)]
CELLTHIS = []
VTALSO = [('p44', 0), ('p45', 0), ('p46', 0), ('p47', 0), ('p48', 0), ('p49', 0), ('p50', 0), ('p51', 0), ('p52', 0), ('p53', 0), ('p54', 0), ('p55', 0), ('p56', 0), ('p57', 0), ('p58', 0), ('p59', 0), ('p60', 0), ('p61', 0), ('p62', 0), ('p63', 0), ('p64', 0), ('p65', 0), ('p66', 0), ('p67', 0), ('p68', 0), ('p69', 0), ('p70', 0), ('p71', 0), ('p74', 0), ('p9', 0), ('p10', 0), ('p11', 0), ('p13', 0), ('p14', 0), ('p15', 0), ('p17', 0), ('p18', 0), ('p19', 0), ('p21', 0), ('p22', 0), ('p23', 0), ('p25', 0), ('p26', 0), ('p27', 0), ('p29', 0), ('p30', 0), ('p31', 0), ('p33', 0), ('p34', 0), ('p35', 0), ('p37', 0), ('p38', 0), ('p39', 0)]
SPARSE = [('@', '0xe3a390', 28396, 8, [(0, 4)])]
GTAB = []
RECORD = [0, 1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]
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
INITCALLS = []
MAXINSNS = 200000
ECXANSWER = [(0, 32, None)]
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
    rng = random.Random(3304)
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
    for i in range(1500):
        h.brk = base  # the bump heap is reused a row; 1500 rows of windows outgrow it
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

# sweep_gen: ["--item", "3304", "obj", "City::find_buildings@007384c0", "--name", "city_find_buildings", "--win", "0x80", "--own", "0", "--ret", "none", "--tib", "--cell-win", "0xc0618c:0x1f8", "--cell-win", "0xc06198:0x10", "--cell-win", "0xcae5fc:0x400", "--cell-win", "0xc06210:0x2a0", "--fill", "c2:i//3", "--sparse", "@0xe3a390:0x6eec:8:0,4", "--ptrtab", "c0+0x14+0x1c*0:3:0x24:vt", "--ptrtab", "c0+0x14+0x1c*1:3:0x24:vt", "--ptrtab", "c0+0x14+0x1c*2:3:0x24:vt", "--ptrtab", "c0+0x14+0x1c*3:3:0x24:vt", "--ptrtab", "c0+0x14+0x1c*4:3:0x24:vt", "--ptrtab", "c0+0x14+0x1c*5:3:0x24:vt", "--ptrtab", "c0+0x14+0x1c*6:3:0x24:vt", "--ptrtab", "c0+0x14+0x1c*7:3:0x24:vt", "--ptrtab", "0xc0ab84+u8(m,0x5e)*0x1c:3:0x20", "--stub", "slot:0xac:0", "--stub", "slot:0x10:0", "--stub", "0x6db790:1", "--stub", "slot:0x60:2", "--stub", "slot:0xb8:2", "--stub", "0x627fe0:0", "--stub", "0x622380:1", "--stub", "slot:0x94:1", "--stub", "0xa1eeb0:1", "--stub", "slot:0xb0:0", "--stub", "slot:0x10c:1", "--stub", "0xa1cd60:2", "--stub", "0xa1cf40:0", "--stub", "0x7e9fb0:10", "--stub", "slot:0x1a8:3", "--stub", "slot:0x150:3", "--stub", "0x642fc0:2", "--stub", "0x738b20:0", "--stub", "0x647080:3", "--stub", "slot:0x158:3", "--place", "p9+0x20:0:0x80", "--place", "p10+0x20:0:0x80", "--place", "p11+0x20:0:0x80", "--place", "p13+0x20:0:0x80", "--place", "p14+0x20:0:0x80", "--place", "p15+0x20:0:0x80", "--place", "p17+0x20:0:0x80", "--place", "p18+0x20:0:0x80", "--place", "p19+0x20:0:0x80", "--place", "p21+0x20:0:0x80", "--place", "p22+0x20:0:0x80", "--place", "p23+0x20:0:0x80", "--place", "p25+0x20:0:0x80", "--place", "p26+0x20:0:0x80", "--place", "p27+0x20:0:0x80", "--place", "p29+0x20:0:0x80", "--place", "p30+0x20:0:0x80", "--place", "p31+0x20:0:0x80", "--place", "p33+0x20:0:0x80", "--place", "p34+0x20:0:0x80", "--place", "p35+0x20:0:0x80", "--place", "p37+0x20:0:0x80", "--place", "p38+0x20:0:0x80", "--place", "p39+0x20:0:0x80", "--place", "m+0x70:0:0x2c8", "--place", "m+0x74:0:0x2c8", "--place", "m+0x78:0:0x2c8", "--place", "m+0x7c:0:0x2c8", "--place", "c0+0x1ec:0:0x20", "--place", "c0+0x1f0:0:0x20", "--answer-win", "9:0x80", "--alias", "p44+0x18:p68!", "--vtat-also", "p44+0", "--vtat-image", "p44+0:0xb42174", "--alias", "p45+0x18:p69!", "--vtat-also", "p45+0", "--vtat-image", "p45+0:0xb42174", "--alias", "p46+0x18:p70!", "--vtat-also", "p46+0", "--vtat-image", "p46+0:0xb42174", "--alias", "p47+0x18:p71!", "--vtat-also", "p47+0", "--vtat-image", "p47+0:0xb42174", "--alias", "p48+0x18:p68!", "--vtat-also", "p48+0", "--vtat-image", "p48+0:0xb42174", "--alias", "p49+0x18:p69!", "--vtat-also", "p49+0", "--vtat-image", "p49+0:0xb42174", "--alias", "p50+0x18:p70!", "--vtat-also", "p50+0", "--vtat-image", "p50+0:0xb42174", "--alias", "p51+0x18:p71!", "--vtat-also", "p51+0", "--vtat-image", "p51+0:0xb42174", "--alias", "p52+0x18:p68!", "--vtat-also", "p52+0", "--vtat-image", "p52+0:0xb42174", "--alias", "p53+0x18:p69!", "--vtat-also", "p53+0", "--vtat-image", "p53+0:0xb42174", "--alias", "p54+0x18:p70!", "--vtat-also", "p54+0", "--vtat-image", "p54+0:0xb42174", "--alias", "p55+0x18:p71!", "--vtat-also", "p55+0", "--vtat-image", "p55+0:0xb42174", "--alias", "p56+0x18:p68!", "--vtat-also", "p56+0", "--vtat-image", "p56+0:0xb42174", "--alias", "p57+0x18:p69!", "--vtat-also", "p57+0", "--vtat-image", "p57+0:0xb42174", "--alias", "p58+0x18:p70!", "--vtat-also", "p58+0", "--vtat-image", "p58+0:0xb42174", "--alias", "p59+0x18:p71!", "--vtat-also", "p59+0", "--vtat-image", "p59+0:0xb42174", "--alias", "p60+0x18:p68!", "--vtat-also", "p60+0", "--vtat-image", "p60+0:0xb42174", "--alias", "p61+0x18:p69!", "--vtat-also", "p61+0", "--vtat-image", "p61+0:0xb42174", "--alias", "p62+0x18:p70!", "--vtat-also", "p62+0", "--vtat-image", "p62+0:0xb42174", "--alias", "p63+0x18:p71!", "--vtat-also", "p63+0", "--vtat-image", "p63+0:0xb42174", "--alias", "p64+0x18:p68!", "--vtat-also", "p64+0", "--vtat-image", "p64+0:0xb42174", "--alias", "p65+0x18:p69!", "--vtat-also", "p65+0", "--vtat-image", "p65+0:0xb42174", "--alias", "p66+0x18:p70!", "--vtat-also", "p66+0", "--vtat-image", "p66+0:0xb42174", "--alias", "p67+0x18:p71!", "--vtat-also", "p67+0", "--vtat-image", "p67+0:0xb42174", "--alias", "p41+0x18:p68!", "--alias", "p42+0x18:p68!", "--alias", "p43+0x18:p68!", "--vtat-also", "p68+0", "--vtat-also", "p69+0", "--vtat-also", "p70+0", "--vtat-also", "p71+0", "--vtat-also", "p74+0", "--ecx-answer", "0:0x20", "--wbias", "p9+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p9+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p9+8:0,0,0,0x20,0x1000,0", "--wbias", "p44+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p10+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p10+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p10+8:0,0,0,0x20,0x1000,0", "--wbias", "p45+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p11+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p11+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p11+8:0,0,0,0x20,0x1000,0", "--wbias", "p46+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p13+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p13+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p13+8:0,0,0,0x20,0x1000,0", "--wbias", "p47+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p14+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p14+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p14+8:0,0,0,0x20,0x1000,0", "--wbias", "p48+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p15+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p15+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p15+8:0,0,0,0x20,0x1000,0", "--wbias", "p49+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p17+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p17+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p17+8:0,0,0,0x20,0x1000,0", "--wbias", "p50+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p18+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p18+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p18+8:0,0,0,0x20,0x1000,0", "--wbias", "p51+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p19+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p19+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p19+8:0,0,0,0x20,0x1000,0", "--wbias", "p52+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p21+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p21+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p21+8:0,0,0,0x20,0x1000,0", "--wbias", "p53+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p22+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p22+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p22+8:0,0,0,0x20,0x1000,0", "--wbias", "p54+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p23+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p23+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p23+8:0,0,0,0x20,0x1000,0", "--wbias", "p55+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p25+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p25+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p25+8:0,0,0,0x20,0x1000,0", "--wbias", "p56+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p26+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p26+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p26+8:0,0,0,0x20,0x1000,0", "--wbias", "p57+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p27+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p27+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p27+8:0,0,0,0x20,0x1000,0", "--wbias", "p58+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p29+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p29+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p29+8:0,0,0,0x20,0x1000,0", "--wbias", "p59+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p30+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p30+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p30+8:0,0,0,0x20,0x1000,0", "--wbias", "p60+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p31+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p31+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p31+8:0,0,0,0x20,0x1000,0", "--wbias", "p61+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p33+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p33+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p33+8:0,0,0,0x20,0x1000,0", "--wbias", "p62+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p34+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p34+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p34+8:0,0,0,0x20,0x1000,0", "--wbias", "p63+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p35+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p35+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p35+8:0,0,0,0x20,0x1000,0", "--wbias", "p64+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p37+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p37+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p37+8:0,0,0,0x20,0x1000,0", "--wbias", "p65+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p38+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p38+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p38+8:0,0,0,0x20,0x1000,0", "--wbias", "p66+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p39+0x10:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p39+0x14:0x63637,0x63653,0x63688,0x636f7,0x63713,0x63748,0x637b7,0x637d3,0x63408,0x63477,0x63493,0x634c8,0x635f7,0x63213,0x63248,0x63037,0x63053,0x63088,0x63f37,0x63f53,0x63f88,0x63a37,0x63a53,0x63a88,0x63937,0x63953,0x63988,0x624f7,0x62513,0x62548,0x620b7,0x620d3,0x62108,0x62837,0x62853,0x62888,0x613b7,0x613d3,0x61008,0x61b37,0x61b53,0x61b88,0x60637,0x60653,0x60688,0x606f7,0x60713,0x60748,0x607b7,0x607d3,0x60408,0x602b7,0x602d3,0x60308,0x60a37,0x60a53,0x60a88", "--wbias", "p39+8:0,0,0,0x20,0x1000,0", "--wbias", "p67+0x70:0xffff0000,0xffff0000,0x80000000,0x00010000,0,0x00050000", "--wbias", "p68..71+0x2c0:0x10,0,0x10,0,0x1f,0x20,0x30", "--wbias", "p0..2+0:0,1,1,1,3,0x101", "--wbias", "p72+*:0,1,2,3,3", "--wbias", "p73+*:0,1,2,3,3", "--wbias", "c1+4:0,0,1,2", "--wbias", "c1+8:0,0,1,2", "--wbias", "m+0x5c:0,0x10000,0x20000", "--wbias", "m+8:0,1,2", "--wbias", "c3+0x298:0,1,2,3,-1", "--answers", "1:1,1,1,0", "--answers", "2:0,1,2,3,5,8,10,15,20,30,40,63,64,65,100,64,65,66,100,100", "--answers", "3:0,1", "--answers", "4:0,1", "--seq", "1:12", "--seq", "7:8", "--answer-range", "7:-2:2", "--record", "0", "--record", "1", "--record", "2", "--record", "3", "--record", "4", "--record", "5", "--record", "6", "--record", "7", "--record", "9", "--record", "10", "--record", "11", "--record", "12", "--record", "13", "--record", "14", "--record", "15", "--record", "16", "--record", "17", "--record", "18", "--record", "19", "--record-order", "--rows", "1500"]
