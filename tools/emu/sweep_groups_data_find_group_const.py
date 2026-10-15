#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_groups_data_find_group_const.py — `GroupsData::find_group_const@007084c0` under unicorn (item 2743, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_groups_data_find_group_const.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 16-byte object of random bytes (no bias), 3 stack argument(s);
the word at `0xe85f20` points at 276760 random bytes less `0` (placed); the word at `0xc0ab84` points at 12 random bytes less `0` (placed); the word at `0xc0aba0` points at 12 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `p2+4` points at 16 random bytes less `0` (placed); the word at `p2+8` points at 16 random bytes less `0` (placed).
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
from sweeplib import draw_reg  # noqa: E402

ENTRY = 0x007084c0
WIN = 16
OWN = 3
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = []
AT = []
PTRARGS = []
PNULLS = []
STUBS = []
PLACES = [(None, '0xe85f20', '0', 276760), (None, '0xc0ab84', '0', 12), (None, '0xc0aba0', '0', 12), ('p1', '0', '0', 16), ('p1', '4', '0', 16), ('p1', '8', '0', 16), ('p2', '0', '0', 16), ('p2', '4', '0', 16), ('p2', '8', '0', 16)]
NULLS = []
WBIAS = [('p3', 8, ('set', [0, 1, 1])), ('p4', 8, ('set', [0, 1, 1])), ('p5', 8, ('set', [0, 1, 1])), ('p6', 8, ('set', [0, 1, 1])), ('p7', 8, ('set', [0, 1, 1])), ('p8', 8, ('set', [0, 1, 1])), ('p0', 72, ('set', [0, 65536, 65536, 131072])), ('p0', 12, ('range', 0, 3)), ('p0', 2252, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 2256, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 2588, ('set', [0, 65536, 65536, 131072])), ('p0', 2528, ('range', 0, 3)), ('p0', 4768, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 4772, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 5104, ('set', [0, 65536, 65536, 131072])), ('p0', 5044, ('range', 0, 3)), ('p0', 7284, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 7288, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 7620, ('set', [0, 65536, 65536, 131072])), ('p0', 7560, ('range', 0, 3)), ('p0', 9800, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 9804, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 10136, ('set', [0, 65536, 65536, 131072])), ('p0', 10076, ('range', 0, 3)), ('p0', 12316, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 12320, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 12652, ('set', [0, 65536, 65536, 131072])), ('p0', 12592, ('range', 0, 3)), ('p0', 14832, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 14836, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 15168, ('set', [0, 65536, 65536, 131072])), ('p0', 15108, ('range', 0, 3)), ('p0', 17348, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 17352, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 17684, ('set', [0, 65536, 65536, 131072])), ('p0', 17624, ('range', 0, 3)), ('p0', 19864, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 19868, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 20200, ('set', [0, 65536, 65536, 131072])), ('p0', 20140, ('range', 0, 3)), ('p0', 22380, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 22384, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 22716, ('set', [0, 65536, 65536, 131072])), ('p0', 22656, ('range', 0, 3)), ('p0', 24896, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 24900, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 25232, ('set', [0, 65536, 65536, 131072])), ('p0', 25172, ('range', 0, 3)), ('p0', 27412, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 27416, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 27748, ('set', [0, 65536, 65536, 131072])), ('p0', 27688, ('range', 0, 3)), ('p0', 29928, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 29932, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 30264, ('set', [0, 65536, 65536, 131072])), ('p0', 30204, ('range', 0, 3)), ('p0', 32444, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 32448, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 32780, ('set', [0, 65536, 65536, 131072])), ('p0', 32720, ('range', 0, 3)), ('p0', 34960, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 34964, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 35296, ('set', [0, 65536, 65536, 131072])), ('p0', 35236, ('range', 0, 3)), ('p0', 37476, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 37480, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 37812, ('set', [0, 65536, 65536, 131072])), ('p0', 37752, ('range', 0, 3)), ('p0', 39992, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 39996, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 40328, ('set', [0, 65536, 65536, 131072])), ('p0', 40268, ('range', 0, 3)), ('p0', 42508, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 42512, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 42844, ('set', [0, 65536, 65536, 131072])), ('p0', 42784, ('range', 0, 3)), ('p0', 45024, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 45028, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 45360, ('set', [0, 65536, 65536, 131072])), ('p0', 45300, ('range', 0, 3)), ('p0', 47540, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 47544, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 47876, ('set', [0, 65536, 65536, 131072])), ('p0', 47816, ('range', 0, 3)), ('p0', 50056, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 50060, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 50392, ('set', [0, 65536, 65536, 131072])), ('p0', 50332, ('range', 0, 3)), ('p0', 52572, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 52576, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 52908, ('set', [0, 65536, 65536, 131072])), ('p0', 52848, ('range', 0, 3)), ('p0', 55088, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 55092, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 55424, ('set', [0, 65536, 65536, 131072])), ('p0', 55364, ('range', 0, 3)), ('p0', 57604, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 57608, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 57940, ('set', [0, 65536, 65536, 131072])), ('p0', 57880, ('range', 0, 3)), ('p0', 60120, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 60124, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 60456, ('set', [0, 65536, 65536, 131072])), ('p0', 60396, ('range', 0, 3)), ('p0', 62636, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 62640, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 62972, ('set', [0, 65536, 65536, 131072])), ('p0', 62912, ('range', 0, 3)), ('p0', 65152, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 65156, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 65488, ('set', [0, 65536, 65536, 131072])), ('p0', 65428, ('range', 0, 3)), ('p0', 67668, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 67672, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 68004, ('set', [0, 65536, 65536, 131072])), ('p0', 67944, ('range', 0, 3)), ('p0', 70184, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 70188, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 70520, ('set', [0, 65536, 65536, 131072])), ('p0', 70460, ('range', 0, 3)), ('p0', 72700, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 72704, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 73036, ('set', [0, 65536, 65536, 131072])), ('p0', 72976, ('range', 0, 3)), ('p0', 75216, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 75220, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 75552, ('set', [0, 65536, 65536, 131072])), ('p0', 75492, ('range', 0, 3)), ('p0', 77732, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 77736, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 78068, ('set', [0, 65536, 65536, 131072])), ('p0', 78008, ('range', 0, 3)), ('p0', 80248, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 80252, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 80584, ('set', [0, 65536, 65536, 131072])), ('p0', 80524, ('range', 0, 3)), ('p0', 82764, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 82768, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 83100, ('set', [0, 65536, 65536, 131072])), ('p0', 83040, ('range', 0, 3)), ('p0', 85280, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 85284, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 85616, ('set', [0, 65536, 65536, 131072])), ('p0', 85556, ('range', 0, 3)), ('p0', 87796, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 87800, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 88132, ('set', [0, 65536, 65536, 131072])), ('p0', 88072, ('range', 0, 3)), ('p0', 90312, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 90316, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 90648, ('set', [0, 65536, 65536, 131072])), ('p0', 90588, ('range', 0, 3)), ('p0', 92828, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 92832, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 93164, ('set', [0, 65536, 65536, 131072])), ('p0', 93104, ('range', 0, 3)), ('p0', 95344, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 95348, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 95680, ('set', [0, 65536, 65536, 131072])), ('p0', 95620, ('range', 0, 3)), ('p0', 97860, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 97864, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 98196, ('set', [0, 65536, 65536, 131072])), ('p0', 98136, ('range', 0, 3)), ('p0', 100376, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 100380, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 100712, ('set', [0, 65536, 65536, 131072])), ('p0', 100652, ('range', 0, 3)), ('p0', 102892, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 102896, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 103228, ('set', [0, 65536, 65536, 131072])), ('p0', 103168, ('range', 0, 3)), ('p0', 105408, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 105412, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 105744, ('set', [0, 65536, 65536, 131072])), ('p0', 105684, ('range', 0, 3)), ('p0', 107924, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 107928, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 108260, ('set', [0, 65536, 65536, 131072])), ('p0', 108200, ('range', 0, 3)), ('p0', 110440, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 110444, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 110776, ('set', [0, 65536, 65536, 131072])), ('p0', 110716, ('range', 0, 3)), ('p0', 112956, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 112960, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 113292, ('set', [0, 65536, 65536, 131072])), ('p0', 113232, ('range', 0, 3)), ('p0', 115472, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 115476, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 161096, ('set', [0, 65536, 65536, 131072])), ('p0', 161036, ('range', 0, 3)), ('p0', 163276, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 163280, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 163612, ('set', [0, 65536, 65536, 131072])), ('p0', 163552, ('range', 0, 3)), ('p0', 165792, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 165796, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 166128, ('set', [0, 65536, 65536, 131072])), ('p0', 166068, ('range', 0, 3)), ('p0', 168308, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 168312, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 168644, ('set', [0, 65536, 65536, 131072])), ('p0', 168584, ('range', 0, 3)), ('p0', 170824, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 170828, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 171160, ('set', [0, 65536, 65536, 131072])), ('p0', 171100, ('range', 0, 3)), ('p0', 173340, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 173344, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 173676, ('set', [0, 65536, 65536, 131072])), ('p0', 173616, ('range', 0, 3)), ('p0', 175856, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 175860, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 176192, ('set', [0, 65536, 65536, 131072])), ('p0', 176132, ('range', 0, 3)), ('p0', 178372, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 178376, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 178708, ('set', [0, 65536, 65536, 131072])), ('p0', 178648, ('range', 0, 3)), ('p0', 180888, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 180892, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 181224, ('set', [0, 65536, 65536, 131072])), ('p0', 181164, ('range', 0, 3)), ('p0', 183404, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 183408, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 183740, ('set', [0, 65536, 65536, 131072])), ('p0', 183680, ('range', 0, 3)), ('p0', 185920, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 185924, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 186256, ('set', [0, 65536, 65536, 131072])), ('p0', 186196, ('range', 0, 3)), ('p0', 188436, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 188440, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 188772, ('set', [0, 65536, 65536, 131072])), ('p0', 188712, ('range', 0, 3)), ('p0', 190952, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 190956, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 191288, ('set', [0, 65536, 65536, 131072])), ('p0', 191228, ('range', 0, 3)), ('p0', 193468, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 193472, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 193804, ('set', [0, 65536, 65536, 131072])), ('p0', 193744, ('range', 0, 3)), ('p0', 195984, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 195988, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 196320, ('set', [0, 65536, 65536, 131072])), ('p0', 196260, ('range', 0, 3)), ('p0', 198500, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 198504, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 198836, ('set', [0, 65536, 65536, 131072])), ('p0', 198776, ('range', 0, 3)), ('p0', 201016, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 201020, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 201352, ('set', [0, 65536, 65536, 131072])), ('p0', 201292, ('range', 0, 3)), ('p0', 203532, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 203536, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 203868, ('set', [0, 65536, 65536, 131072])), ('p0', 203808, ('range', 0, 3)), ('p0', 206048, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 206052, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 206384, ('set', [0, 65536, 65536, 131072])), ('p0', 206324, ('range', 0, 3)), ('p0', 208564, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 208568, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 208900, ('set', [0, 65536, 65536, 131072])), ('p0', 208840, ('range', 0, 3)), ('p0', 211080, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 211084, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 211416, ('set', [0, 65536, 65536, 131072])), ('p0', 211356, ('range', 0, 3)), ('p0', 213596, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 213600, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 213932, ('set', [0, 65536, 65536, 131072])), ('p0', 213872, ('range', 0, 3)), ('p0', 216112, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 216116, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 216448, ('set', [0, 65536, 65536, 131072])), ('p0', 216388, ('range', 0, 3)), ('p0', 218628, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 218632, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 218964, ('set', [0, 65536, 65536, 131072])), ('p0', 218904, ('range', 0, 3)), ('p0', 221144, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 221148, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 221480, ('set', [0, 65536, 65536, 131072])), ('p0', 221420, ('range', 0, 3)), ('p0', 223660, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 223664, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 223996, ('set', [0, 65536, 65536, 131072])), ('p0', 223936, ('range', 0, 3)), ('p0', 226176, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 226180, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 226512, ('set', [0, 65536, 65536, 131072])), ('p0', 226452, ('range', 0, 3)), ('p0', 228692, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 228696, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 229028, ('set', [0, 65536, 65536, 131072])), ('p0', 228968, ('range', 0, 3)), ('p0', 231208, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 231212, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 231544, ('set', [0, 65536, 65536, 131072])), ('p0', 231484, ('range', 0, 3)), ('p0', 233724, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 233728, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 234060, ('set', [0, 65536, 65536, 131072])), ('p0', 234000, ('range', 0, 3)), ('p0', 236240, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 236244, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 236576, ('set', [0, 65536, 65536, 131072])), ('p0', 236516, ('range', 0, 3)), ('p0', 238756, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 238760, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 239092, ('set', [0, 65536, 65536, 131072])), ('p0', 239032, ('range', 0, 3)), ('p0', 241272, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 241276, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 241608, ('set', [0, 65536, 65536, 131072])), ('p0', 241548, ('range', 0, 3)), ('p0', 243788, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 243792, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 244124, ('set', [0, 65536, 65536, 131072])), ('p0', 244064, ('range', 0, 3)), ('p0', 246304, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 246308, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 246640, ('set', [0, 65536, 65536, 131072])), ('p0', 246580, ('range', 0, 3)), ('p0', 248820, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 248824, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 249156, ('set', [0, 65536, 65536, 131072])), ('p0', 249096, ('range', 0, 3)), ('p0', 251336, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 251340, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 251672, ('set', [0, 65536, 65536, 131072])), ('p0', 251612, ('range', 0, 3)), ('p0', 253852, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 253856, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 254188, ('set', [0, 65536, 65536, 131072])), ('p0', 254128, ('range', 0, 3)), ('p0', 256368, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 256372, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 256704, ('set', [0, 65536, 65536, 131072])), ('p0', 256644, ('range', 0, 3)), ('p0', 258884, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 258888, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 259220, ('set', [0, 65536, 65536, 131072])), ('p0', 259160, ('range', 0, 3)), ('p0', 261400, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 261404, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 261736, ('set', [0, 65536, 65536, 131072])), ('p0', 261676, ('range', 0, 3)), ('p0', 263916, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 263920, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 264252, ('set', [0, 65536, 65536, 131072])), ('p0', 264192, ('range', 0, 3)), ('p0', 266432, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 266436, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 266768, ('set', [0, 65536, 65536, 131072])), ('p0', 266708, ('range', 0, 3)), ('p0', 268948, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 268952, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 269284, ('set', [0, 65536, 65536, 131072])), ('p0', 269224, ('range', 0, 3)), ('p0', 271464, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 271468, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 271800, ('set', [0, 65536, 65536, 131072])), ('p0', 271740, ('range', 0, 3)), ('p0', 273980, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 273984, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 274316, ('set', [0, 65536, 65536, 131072])), ('p0', 274256, ('range', 0, 3)), ('p0', 276496, ('set', [0, 1, 2, 65536, 65538, 131073])), ('p0', 276500, ('set', [0, 1, 2, 65536, 65538, 131073]))]
FILLS = []
ARGRANGE = [(0, 0, 1), (1, 0, 2)]
SLOTBASE = []
ECXINT = None
EDX = None
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = []
VTAT = None
VTATHALF = []
ANSWERWIN = []
CDECL = False
ARGEQ = []
SEQ = []
ANSRANGE = []
CELLTHIS = []
VTALSO = []
SPARSE = []
GTAB = []
RECORD = []
RECORDORDER = False
RECORDEDX = False
ROWSPLIT = {}
LINKS = []
OUTARG = []
CDECLSTUBS = []
STUBWRITE = []
ECXANSWER = []
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
    offset (`m+0x0`, `p0+0x10`), a local by its depth (`sp-0x14`), anything
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


def win_write(inner, at, size, v, write):
    """`--stub-write IDX:WIN+OFF:SIZE:VAL` (item 2346): a stub that writes VAL
    at a window's word each call — a callee that unlinks the head of the
    container its caller loops over (`Object::kill_contents`) — then answers
    as `inner` would."""
    def answer(ecx, args):
        write(at, (v & ((1 << 8 * size) - 1)).to_bytes(size, "little"))
        return inner(ecx, args) if callable(inner) else inner
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
    rng = random.Random(2743)
    # `--stub-cdecl IDX`: the callee's arguments are the caller's to pop (item 2325).
    h = Harness(argv[1], {a: (f"{a:x}", n, False, i in CDECLSTUBS) for i, (k, a, n) in enumerate(STUBS)
                          if k == "call"})
    uc = h.m.uc
    slots = {off: (f"slot{off:x}", n, 0) for k, off, n in STUBS if k == "slot"}
    vt = h.vtable(slots) if slots else None
    names = [f"slot{a:x}" if k == "slot" else f"iat{a:x}" if k == "iat" else f"{a:x}" for k, a, _ in STUBS]
    # `--stub iat:VA:N`: a stub whose address is laid in an import's IAT cell,
    # which the emulator resolves to nothing, so `call [VA]` lands on it
    # (item 2336: `timeGetTime`, parked 2327).
    iats = [(a, h.stub(f"iat{a:x}", n, 0)) for k, a, n in STUBS if k == "iat"]
    # `--stub-cdecl IDX` on an `iat` stub (`malloc`: `call [VA]; add esp, 4`).
    for (_, sa), (i, (k, _, _)) in zip(iats, [e for e in enumerate(STUBS) if e[1][0] == "iat"]):
        if i in CDECLSTUBS:
            h.hooked[sa] = h.hooked[sa][:3] + (True,)
    stubbed = {off: h.r32(vt + off) & 0xFFFFFFFF for off, _ in SLOTBASE}
    base = h.brk
    for i in range(220):
        h.brk = base  # the bump heap is reused a row; 220 rows of windows outgrow it
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
        for win, off in VTATHALF:
            # The stubbed vtable laid half the rows at a record the function
            # compares with a class's vtable, the other half left as drawn
            # (a `--wbias` of the class's address; item 2223).
            if rng.random() < 0.5:
                w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
                w[off:off + 4] = vt.to_bytes(4, "little")
        for lo, hi, off, size, d, terms in LINKS:
            # A link field (`o_up`, a type's upgrade) of figures p(lo)..p(hi),
            # drawn from the figures on one side of each one's own index or a
            # terminator, so a walk through them ends (item 2276).
            n = hi - lo + 1
            for k in range(n):
                v = link_value(rng, n, k, d, terms)
                p[lo + k][off:off + size] = (v & ((1 << 8 * size) - 1)).to_bytes(size, "little")
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
            h.answers[names[idx]] = win_write(h.answers[names[idx]], at + off, size, v, uc.mem_write)
        derefs = [[] for _ in DEREF]
        for (idx, k, n, wr), sink in zip(DEREF, derefs):
            h.answers[names[idx]] = read_arg(h.answers[names[idx]], k, n,
                                             lambda a_, n_: bytes(uc.mem_read(a_, n_)), sink, wr)
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
        h.calls.clear()
        h.edxs.clear()
        r = h.call(ENTRY, tuple(args), ecx=this if ECXINT is None else ecx,
                   edx=0 if EDX is None else edx, ret=RET)  # Harness.call signs eax; signed() again would double a negative
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
                 + ([] if ECXINT is None else [str(ecx)]) + ([] if EDX is None else [str(edx)]))
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

# sweep_gen: ["--item", "2743", "obj", "GroupsData::find_group_const@007084c0", "--own", "3", "--win", "0x10", "--argrange", "0:0:1", "--argrange", "1:0:2", "--place", "0xe85f20:0:276760", "--place", "0xc0ab84:0:12", "--place", "0xc0aba0:0:12", "--place", "p1+0:0:0x10", "--place", "p1+4:0:0x10", "--place", "p1+8:0:0x10", "--place", "p2+0:0:0x10", "--place", "p2+4:0:0x10", "--place", "p2+8:0:0x10", "--wbias", "p3+8:0,1,1", "--wbias", "p4+8:0,1,1", "--wbias", "p5+8:0,1,1", "--wbias", "p6+8:0,1,1", "--wbias", "p7+8:0,1,1", "--wbias", "p8+8:0,1,1", "--wbias", "p0+72:0,0x10000,0x10000,0x20000", "--wbias", "p0+12:0..3", "--wbias", "p0+2252:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+2256:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+2588:0,0x10000,0x10000,0x20000", "--wbias", "p0+2528:0..3", "--wbias", "p0+4768:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+4772:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+5104:0,0x10000,0x10000,0x20000", "--wbias", "p0+5044:0..3", "--wbias", "p0+7284:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+7288:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+7620:0,0x10000,0x10000,0x20000", "--wbias", "p0+7560:0..3", "--wbias", "p0+9800:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+9804:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+10136:0,0x10000,0x10000,0x20000", "--wbias", "p0+10076:0..3", "--wbias", "p0+12316:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+12320:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+12652:0,0x10000,0x10000,0x20000", "--wbias", "p0+12592:0..3", "--wbias", "p0+14832:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+14836:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+15168:0,0x10000,0x10000,0x20000", "--wbias", "p0+15108:0..3", "--wbias", "p0+17348:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+17352:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+17684:0,0x10000,0x10000,0x20000", "--wbias", "p0+17624:0..3", "--wbias", "p0+19864:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+19868:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+20200:0,0x10000,0x10000,0x20000", "--wbias", "p0+20140:0..3", "--wbias", "p0+22380:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+22384:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+22716:0,0x10000,0x10000,0x20000", "--wbias", "p0+22656:0..3", "--wbias", "p0+24896:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+24900:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+25232:0,0x10000,0x10000,0x20000", "--wbias", "p0+25172:0..3", "--wbias", "p0+27412:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+27416:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+27748:0,0x10000,0x10000,0x20000", "--wbias", "p0+27688:0..3", "--wbias", "p0+29928:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+29932:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+30264:0,0x10000,0x10000,0x20000", "--wbias", "p0+30204:0..3", "--wbias", "p0+32444:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+32448:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+32780:0,0x10000,0x10000,0x20000", "--wbias", "p0+32720:0..3", "--wbias", "p0+34960:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+34964:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+35296:0,0x10000,0x10000,0x20000", "--wbias", "p0+35236:0..3", "--wbias", "p0+37476:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+37480:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+37812:0,0x10000,0x10000,0x20000", "--wbias", "p0+37752:0..3", "--wbias", "p0+39992:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+39996:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+40328:0,0x10000,0x10000,0x20000", "--wbias", "p0+40268:0..3", "--wbias", "p0+42508:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+42512:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+42844:0,0x10000,0x10000,0x20000", "--wbias", "p0+42784:0..3", "--wbias", "p0+45024:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+45028:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+45360:0,0x10000,0x10000,0x20000", "--wbias", "p0+45300:0..3", "--wbias", "p0+47540:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+47544:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+47876:0,0x10000,0x10000,0x20000", "--wbias", "p0+47816:0..3", "--wbias", "p0+50056:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+50060:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+50392:0,0x10000,0x10000,0x20000", "--wbias", "p0+50332:0..3", "--wbias", "p0+52572:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+52576:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+52908:0,0x10000,0x10000,0x20000", "--wbias", "p0+52848:0..3", "--wbias", "p0+55088:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+55092:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+55424:0,0x10000,0x10000,0x20000", "--wbias", "p0+55364:0..3", "--wbias", "p0+57604:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+57608:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+57940:0,0x10000,0x10000,0x20000", "--wbias", "p0+57880:0..3", "--wbias", "p0+60120:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+60124:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+60456:0,0x10000,0x10000,0x20000", "--wbias", "p0+60396:0..3", "--wbias", "p0+62636:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+62640:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+62972:0,0x10000,0x10000,0x20000", "--wbias", "p0+62912:0..3", "--wbias", "p0+65152:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+65156:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+65488:0,0x10000,0x10000,0x20000", "--wbias", "p0+65428:0..3", "--wbias", "p0+67668:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+67672:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+68004:0,0x10000,0x10000,0x20000", "--wbias", "p0+67944:0..3", "--wbias", "p0+70184:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+70188:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+70520:0,0x10000,0x10000,0x20000", "--wbias", "p0+70460:0..3", "--wbias", "p0+72700:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+72704:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+73036:0,0x10000,0x10000,0x20000", "--wbias", "p0+72976:0..3", "--wbias", "p0+75216:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+75220:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+75552:0,0x10000,0x10000,0x20000", "--wbias", "p0+75492:0..3", "--wbias", "p0+77732:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+77736:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+78068:0,0x10000,0x10000,0x20000", "--wbias", "p0+78008:0..3", "--wbias", "p0+80248:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+80252:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+80584:0,0x10000,0x10000,0x20000", "--wbias", "p0+80524:0..3", "--wbias", "p0+82764:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+82768:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+83100:0,0x10000,0x10000,0x20000", "--wbias", "p0+83040:0..3", "--wbias", "p0+85280:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+85284:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+85616:0,0x10000,0x10000,0x20000", "--wbias", "p0+85556:0..3", "--wbias", "p0+87796:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+87800:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+88132:0,0x10000,0x10000,0x20000", "--wbias", "p0+88072:0..3", "--wbias", "p0+90312:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+90316:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+90648:0,0x10000,0x10000,0x20000", "--wbias", "p0+90588:0..3", "--wbias", "p0+92828:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+92832:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+93164:0,0x10000,0x10000,0x20000", "--wbias", "p0+93104:0..3", "--wbias", "p0+95344:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+95348:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+95680:0,0x10000,0x10000,0x20000", "--wbias", "p0+95620:0..3", "--wbias", "p0+97860:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+97864:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+98196:0,0x10000,0x10000,0x20000", "--wbias", "p0+98136:0..3", "--wbias", "p0+100376:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+100380:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+100712:0,0x10000,0x10000,0x20000", "--wbias", "p0+100652:0..3", "--wbias", "p0+102892:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+102896:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+103228:0,0x10000,0x10000,0x20000", "--wbias", "p0+103168:0..3", "--wbias", "p0+105408:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+105412:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+105744:0,0x10000,0x10000,0x20000", "--wbias", "p0+105684:0..3", "--wbias", "p0+107924:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+107928:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+108260:0,0x10000,0x10000,0x20000", "--wbias", "p0+108200:0..3", "--wbias", "p0+110440:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+110444:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+110776:0,0x10000,0x10000,0x20000", "--wbias", "p0+110716:0..3", "--wbias", "p0+112956:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+112960:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+113292:0,0x10000,0x10000,0x20000", "--wbias", "p0+113232:0..3", "--wbias", "p0+115472:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+115476:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+161096:0,0x10000,0x10000,0x20000", "--wbias", "p0+161036:0..3", "--wbias", "p0+163276:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+163280:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+163612:0,0x10000,0x10000,0x20000", "--wbias", "p0+163552:0..3", "--wbias", "p0+165792:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+165796:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+166128:0,0x10000,0x10000,0x20000", "--wbias", "p0+166068:0..3", "--wbias", "p0+168308:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+168312:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+168644:0,0x10000,0x10000,0x20000", "--wbias", "p0+168584:0..3", "--wbias", "p0+170824:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+170828:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+171160:0,0x10000,0x10000,0x20000", "--wbias", "p0+171100:0..3", "--wbias", "p0+173340:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+173344:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+173676:0,0x10000,0x10000,0x20000", "--wbias", "p0+173616:0..3", "--wbias", "p0+175856:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+175860:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+176192:0,0x10000,0x10000,0x20000", "--wbias", "p0+176132:0..3", "--wbias", "p0+178372:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+178376:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+178708:0,0x10000,0x10000,0x20000", "--wbias", "p0+178648:0..3", "--wbias", "p0+180888:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+180892:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+181224:0,0x10000,0x10000,0x20000", "--wbias", "p0+181164:0..3", "--wbias", "p0+183404:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+183408:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+183740:0,0x10000,0x10000,0x20000", "--wbias", "p0+183680:0..3", "--wbias", "p0+185920:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+185924:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+186256:0,0x10000,0x10000,0x20000", "--wbias", "p0+186196:0..3", "--wbias", "p0+188436:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+188440:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+188772:0,0x10000,0x10000,0x20000", "--wbias", "p0+188712:0..3", "--wbias", "p0+190952:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+190956:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+191288:0,0x10000,0x10000,0x20000", "--wbias", "p0+191228:0..3", "--wbias", "p0+193468:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+193472:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+193804:0,0x10000,0x10000,0x20000", "--wbias", "p0+193744:0..3", "--wbias", "p0+195984:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+195988:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+196320:0,0x10000,0x10000,0x20000", "--wbias", "p0+196260:0..3", "--wbias", "p0+198500:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+198504:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+198836:0,0x10000,0x10000,0x20000", "--wbias", "p0+198776:0..3", "--wbias", "p0+201016:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+201020:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+201352:0,0x10000,0x10000,0x20000", "--wbias", "p0+201292:0..3", "--wbias", "p0+203532:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+203536:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+203868:0,0x10000,0x10000,0x20000", "--wbias", "p0+203808:0..3", "--wbias", "p0+206048:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+206052:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+206384:0,0x10000,0x10000,0x20000", "--wbias", "p0+206324:0..3", "--wbias", "p0+208564:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+208568:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+208900:0,0x10000,0x10000,0x20000", "--wbias", "p0+208840:0..3", "--wbias", "p0+211080:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+211084:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+211416:0,0x10000,0x10000,0x20000", "--wbias", "p0+211356:0..3", "--wbias", "p0+213596:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+213600:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+213932:0,0x10000,0x10000,0x20000", "--wbias", "p0+213872:0..3", "--wbias", "p0+216112:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+216116:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+216448:0,0x10000,0x10000,0x20000", "--wbias", "p0+216388:0..3", "--wbias", "p0+218628:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+218632:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+218964:0,0x10000,0x10000,0x20000", "--wbias", "p0+218904:0..3", "--wbias", "p0+221144:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+221148:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+221480:0,0x10000,0x10000,0x20000", "--wbias", "p0+221420:0..3", "--wbias", "p0+223660:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+223664:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+223996:0,0x10000,0x10000,0x20000", "--wbias", "p0+223936:0..3", "--wbias", "p0+226176:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+226180:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+226512:0,0x10000,0x10000,0x20000", "--wbias", "p0+226452:0..3", "--wbias", "p0+228692:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+228696:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+229028:0,0x10000,0x10000,0x20000", "--wbias", "p0+228968:0..3", "--wbias", "p0+231208:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+231212:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+231544:0,0x10000,0x10000,0x20000", "--wbias", "p0+231484:0..3", "--wbias", "p0+233724:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+233728:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+234060:0,0x10000,0x10000,0x20000", "--wbias", "p0+234000:0..3", "--wbias", "p0+236240:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+236244:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+236576:0,0x10000,0x10000,0x20000", "--wbias", "p0+236516:0..3", "--wbias", "p0+238756:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+238760:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+239092:0,0x10000,0x10000,0x20000", "--wbias", "p0+239032:0..3", "--wbias", "p0+241272:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+241276:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+241608:0,0x10000,0x10000,0x20000", "--wbias", "p0+241548:0..3", "--wbias", "p0+243788:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+243792:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+244124:0,0x10000,0x10000,0x20000", "--wbias", "p0+244064:0..3", "--wbias", "p0+246304:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+246308:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+246640:0,0x10000,0x10000,0x20000", "--wbias", "p0+246580:0..3", "--wbias", "p0+248820:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+248824:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+249156:0,0x10000,0x10000,0x20000", "--wbias", "p0+249096:0..3", "--wbias", "p0+251336:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+251340:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+251672:0,0x10000,0x10000,0x20000", "--wbias", "p0+251612:0..3", "--wbias", "p0+253852:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+253856:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+254188:0,0x10000,0x10000,0x20000", "--wbias", "p0+254128:0..3", "--wbias", "p0+256368:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+256372:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+256704:0,0x10000,0x10000,0x20000", "--wbias", "p0+256644:0..3", "--wbias", "p0+258884:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+258888:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+259220:0,0x10000,0x10000,0x20000", "--wbias", "p0+259160:0..3", "--wbias", "p0+261400:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+261404:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+261736:0,0x10000,0x10000,0x20000", "--wbias", "p0+261676:0..3", "--wbias", "p0+263916:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+263920:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+264252:0,0x10000,0x10000,0x20000", "--wbias", "p0+264192:0..3", "--wbias", "p0+266432:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+266436:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+266768:0,0x10000,0x10000,0x20000", "--wbias", "p0+266708:0..3", "--wbias", "p0+268948:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+268952:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+269284:0,0x10000,0x10000,0x20000", "--wbias", "p0+269224:0..3", "--wbias", "p0+271464:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+271468:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+271800:0,0x10000,0x10000,0x20000", "--wbias", "p0+271740:0..3", "--wbias", "p0+273980:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+273984:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+274316:0,0x10000,0x10000,0x20000", "--wbias", "p0+274256:0..3", "--wbias", "p0+276496:0,1,2,0x10000,0x10002,0x20001", "--wbias", "p0+276500:0,1,2,0x10000,0x10002,0x20001", "--port", "todo!()"]
