#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_balance_type_damage.py — `Balance::type_damage@0057fb50` under unicorn (item 3362, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_balance_type_damage.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 64-byte object of random bytes (no bias), 2 stack argument(s);
`p` windows in order: p0..p7 the place(s), in flag order; the global cell 0x00c061f0 points at 256 random bytes; vtable slot +0xc (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xec (0 args) stubbed; vtable slot +0x10c (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0x114 (0 args) stubbed; the word at `0xc061f8` points at 32 random bytes less `0` (placed); the word at `p0+0x10` points at 28 random bytes less `0` (placed); the word at `p1+20` points at 768 random bytes less `0` (placed); the word at `p1+24` points at 768 random bytes less `0` (placed); the word at `p2+0x28c` points at 16 random bytes less `0` (placed); the word at `p2+0x2a8` points at 16 random bytes less `0` (placed); the word at `p3+0x28c` points at 16 random bytes less `0` (placed); the word at `p3+0x2a8` points at 16 random bytes less `0` (placed); slot +0x60 laid as 0x0065f7d0 or its stub, a flag after the answers; stub 0 answers from its own `ecx`: the word at `+0x2b0`; stub 1 answers from its own `ecx`: the word at `+0x2b4`; stub 2 answers from its own `ecx`: the word at `+0x2b8`; stub 4 answers from its own `ecx`: the word at `+0x2bc`; stub 5 answers from its own `ecx`: the word at `+0x2c0`; stub 6 answers from its own `ecx`: the word at `+0x2c4`; stub 7 answers from its own `ecx`: the word at `+0x2c8`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`.
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

ENTRY = 0x0057fb50
WIN = 64
OWN = 2
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(12607984, 256)]
AT = []
ATRANGE = {}
PTRARGS = []
PNULLS = []
STUBS = [('slot', 12, 0), ('slot', 20, 0), ('slot', 28, 0), ('slot', 96, 2), ('slot', 236, 0), ('slot', 268, 0), ('slot', 304, 0), ('slot', 276, 0)]
PLACES = [(None, '0xc061f8', '0', 32), ('p0', '0x10', '0', 28), ('p1', '20', '0', 768), ('p1', '24', '0', 768), ('p2', '0x28c', '0', 16), ('p2', '0x2a8', '0', 16), ('p3', '0x28c', '0', 16), ('p3', '0x2a8', '0', 16)]
NULLS = []
NULLFORCE = {}
WBIAS = [('p2', 484, ('set', [0, 1, 2, 4, 8, 9, 10, 12, 16, 24, 32, 33, 34, 36, 40, 48, 64, 72, 73, 80, 96, 128, 129, 130, 132, 136, 144, 160, 192, 256, 260, 264, 288, 384, 512, 513, 520, 544, 640, 1024, 1025, 1026, 1028, 1032, 1040, 1056, 1088, 1152, 1280, 1536, 2048, 2056, 2080, 2176, 3072, 4096, 4097, 4098, 4100, 4104, 4112, 4128, 4160, 4224, 4352, 4608, 5120, 6144, 8192, 8196, 8200, 8210, 8224, 8320, 9216, 12288, 16384, 16385, 16386, 16388, 16392, 16400, 16416, 16448, 16512, 16640, 16896, 17408, 18432, 20480, 24576, 32768, 32776, 32800, 32896, 33024, 33792, 34816, 36864, 49152, 53248, 65536, 65537, 65538, 65540, 65544, 65552, 65568, 65600, 65664, 65792, 65800, 66048, 66560, 67584, 69632, 73728, 81920, 83080, 98304, 131072, 131080, 131104, 131200, 132096, 135168, 147456, 196608, 262144, 262152, 262160, 262176, 262272, 263168, 266240, 270336, 278528, 327680, 524288, 524296, 524320, 524322, 524416, 524832, 525312, 526336, 528384, 540672, 589824, 1048576, 1048584, 1048592, 1048608, 1048704, 1049088, 1049600, 1052672, 1064960, 1114112, 2097152, 2097160, 2097184, 2097280, 2098176, 2098304, 2101248, 2113536, 2162688, 2162752, 4194304, 4194306, 4194312, 4194336, 4194432, 4195328, 4198400, 4210688, 4259840, 6291456, 6307840, 8388608, 8388616, 8388640, 8388672, 8388736, 8389184, 8389632, 8392704, 8397312, 8404992, 8454144, 8455168, 8912896, 16777216, 16777217, 16777218, 16777220, 16777224, 16777232, 16777248, 16777280, 16777344, 16777472, 16777728, 16778240, 16779264, 16781312, 16785408, 16793600, 16809984, 16842752, 16843264, 16908288, 17039360, 17301504, 17825792, 18874368, 20971520, 25165824, 33554432, 33554440, 33554464, 33554560, 33555456, 33558528, 33558536, 33570816, 33574912, 33619968, 33685504, 50331648, 67108864, 67108868, 67108872, 67108880, 67108896, 67108992, 67109888, 67112960, 67125248, 67174400, 67239936, 67371008, 76546048, 83886080, 134217728, 134217736, 134217760, 134217856, 134217984, 134218752, 134221824, 134234112, 134283264, 135266304, 150994944, 268435456, 268435464, 268435488, 268435584, 268436480, 268439552, 268443664, 268451840, 268500992, 268959744, 270532608, 285212672, 536870912, 536870913, 536870914, 536870916, 536870920, 536870928, 536870944, 536870976, 536871040, 536871168, 536871176, 536871424, 536871936, 536872960, 536875008, 536879104, 536887296, 536903680, 536936448, 537001984, 537133056, 537395200, 537919488, 538968064, 541065216, 545259520, 553648128, 570425344, 603979776, 671088640, 805306368, 1073741824, 1073741832, 1073741856, 1073741952, 1073742848, 1073742850, 1073745920, 1073758208, 1073807360, 1073872896, 1073872898, 1086324736, 1090519040, 1107296320, 1610612736, -2147483648, -2147483640, -2147483616, -2147483520, -2147483136, -2147482624, -2147481600, -2147479552, -2147475456, -2147467264, -2147418112, -2146959360, -2143289342, -2130706432, -1879048192, -1879048191, -1879048190, -1879048188, -1879048184, -1879048176, -1879048160, -1879048128, -1879048064, -1879047936, -1879047680, -1879047168, -1879046144, -1879044096, -1879040000, -1879031808, -1879015424, -1878982656, -1878917120, -1878786048, -1878523904, -1877999616, -1876951040, -1874853888, -1870659584, -1862270976, -1845493760, -1811939328, -1744830464, -1610612736, -1342177280, -805306368, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576, 2097152, 4194304, 8388608, 16777216, 33554432, 67108864, 134217728, 268435456, 536870912, 1073741824, -2147483648, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576, 2097152, 4194304, 8388608, 16777216, 33554432, 67108864, 134217728, 268435456, 536870912, 1073741824, -2147483648, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576, 2097152, 4194304, 8388608, 16777216, 33554432, 67108864, 134217728, 268435456, 536870912, 1073741824, -2147483648, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576, 2097152, 4194304, 8388608, 16777216, 33554432, 67108864, 134217728, 268435456, 536870912, 1073741824, -2147483648])), ('p2', 536, ('set', [0, 1, 2])), ('p2', 4, ('set', [50, 51, 52, 53, 61, 62, 400, 64, 256, 414, 543, 216, 239, 123, 131, 66, 346, 323, 338, 334, 340, 304, 308, 310, 313, 63, 88, 174, 91, 191, 195, 199, 224, 177, 178, 179, 232, 146, 160, 149, 151, 144, 114, 130, 258, 259, 279, 276, 274, 447])), ('p2', 688, ('set', [0, 0, 1])), ('p2', 692, ('set', [0, 0, 1])), ('p2', 696, ('set', [0, 0, 1])), ('p2', 704, ('set', [0, 0, 1])), ('p2', 708, ('set', [0, 0, 1])), ('p2', 712, ('set', [0, 0, 1])), ('p2', 700, ('range', 0, 6)), ('p2', 640, ('range', 1, 4)), ('p2', 668, ('range', 1, 4)), ('p3', 484, ('set', [0, 1, 2, 4, 8, 9, 10, 12, 16, 24, 32, 33, 34, 36, 40, 48, 64, 72, 73, 80, 96, 128, 129, 130, 132, 136, 144, 160, 192, 256, 260, 264, 288, 384, 512, 513, 520, 544, 640, 1024, 1025, 1026, 1028, 1032, 1040, 1056, 1088, 1152, 1280, 1536, 2048, 2056, 2080, 2176, 3072, 4096, 4097, 4098, 4100, 4104, 4112, 4128, 4160, 4224, 4352, 4608, 5120, 6144, 8192, 8196, 8200, 8210, 8224, 8320, 9216, 12288, 16384, 16385, 16386, 16388, 16392, 16400, 16416, 16448, 16512, 16640, 16896, 17408, 18432, 20480, 24576, 32768, 32776, 32800, 32896, 33024, 33792, 34816, 36864, 49152, 53248, 65536, 65537, 65538, 65540, 65544, 65552, 65568, 65600, 65664, 65792, 65800, 66048, 66560, 67584, 69632, 73728, 81920, 83080, 98304, 131072, 131080, 131104, 131200, 132096, 135168, 147456, 196608, 262144, 262152, 262160, 262176, 262272, 263168, 266240, 270336, 278528, 327680, 524288, 524296, 524320, 524322, 524416, 524832, 525312, 526336, 528384, 540672, 589824, 1048576, 1048584, 1048592, 1048608, 1048704, 1049088, 1049600, 1052672, 1064960, 1114112, 2097152, 2097160, 2097184, 2097280, 2098176, 2098304, 2101248, 2113536, 2162688, 2162752, 4194304, 4194306, 4194312, 4194336, 4194432, 4195328, 4198400, 4210688, 4259840, 6291456, 6307840, 8388608, 8388616, 8388640, 8388672, 8388736, 8389184, 8389632, 8392704, 8397312, 8404992, 8454144, 8455168, 8912896, 16777216, 16777217, 16777218, 16777220, 16777224, 16777232, 16777248, 16777280, 16777344, 16777472, 16777728, 16778240, 16779264, 16781312, 16785408, 16793600, 16809984, 16842752, 16843264, 16908288, 17039360, 17301504, 17825792, 18874368, 20971520, 25165824, 33554432, 33554440, 33554464, 33554560, 33555456, 33558528, 33558536, 33570816, 33574912, 33619968, 33685504, 50331648, 67108864, 67108868, 67108872, 67108880, 67108896, 67108992, 67109888, 67112960, 67125248, 67174400, 67239936, 67371008, 76546048, 83886080, 134217728, 134217736, 134217760, 134217856, 134217984, 134218752, 134221824, 134234112, 134283264, 135266304, 150994944, 268435456, 268435464, 268435488, 268435584, 268436480, 268439552, 268443664, 268451840, 268500992, 268959744, 270532608, 285212672, 536870912, 536870913, 536870914, 536870916, 536870920, 536870928, 536870944, 536870976, 536871040, 536871168, 536871176, 536871424, 536871936, 536872960, 536875008, 536879104, 536887296, 536903680, 536936448, 537001984, 537133056, 537395200, 537919488, 538968064, 541065216, 545259520, 553648128, 570425344, 603979776, 671088640, 805306368, 1073741824, 1073741832, 1073741856, 1073741952, 1073742848, 1073742850, 1073745920, 1073758208, 1073807360, 1073872896, 1073872898, 1086324736, 1090519040, 1107296320, 1610612736, -2147483648, -2147483640, -2147483616, -2147483520, -2147483136, -2147482624, -2147481600, -2147479552, -2147475456, -2147467264, -2147418112, -2146959360, -2143289342, -2130706432, -1879048192, -1879048191, -1879048190, -1879048188, -1879048184, -1879048176, -1879048160, -1879048128, -1879048064, -1879047936, -1879047680, -1879047168, -1879046144, -1879044096, -1879040000, -1879031808, -1879015424, -1878982656, -1878917120, -1878786048, -1878523904, -1877999616, -1876951040, -1874853888, -1870659584, -1862270976, -1845493760, -1811939328, -1744830464, -1610612736, -1342177280, -805306368, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576, 2097152, 4194304, 8388608, 16777216, 33554432, 67108864, 134217728, 268435456, 536870912, 1073741824, -2147483648, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576, 2097152, 4194304, 8388608, 16777216, 33554432, 67108864, 134217728, 268435456, 536870912, 1073741824, -2147483648, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576, 2097152, 4194304, 8388608, 16777216, 33554432, 67108864, 134217728, 268435456, 536870912, 1073741824, -2147483648, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576, 2097152, 4194304, 8388608, 16777216, 33554432, 67108864, 134217728, 268435456, 536870912, 1073741824, -2147483648])), ('p3', 536, ('set', [0, 1, 2])), ('p3', 4, ('set', [50, 51, 52, 53, 61, 62, 400, 64, 256, 414, 543, 216, 239, 123, 131, 66, 346, 323, 338, 334, 340, 304, 308, 310, 313, 63, 88, 174, 91, 191, 195, 199, 224, 177, 178, 179, 232, 146, 160, 149, 151, 144, 114, 130, 258, 259, 279, 276, 274, 447])), ('p3', 688, ('set', [0, 0, 1])), ('p3', 692, ('set', [0, 0, 1])), ('p3', 696, ('set', [0, 0, 1])), ('p3', 704, ('set', [0, 0, 1])), ('p3', 708, ('set', [0, 0, 1])), ('p3', 712, ('set', [0, 0, 1])), ('p3', 700, ('range', 0, 6)), ('p3', 640, ('range', 1, 4)), ('p3', 668, ('range', 1, 4)), ('p4', None, ('set', [0, 91, 123, 258, 327746, 327810, 327811, 327875, 327919, 327990, 3211264, 3211269, 3211327, 3211604, 4128773, 4128817, 4128919, 4129215, 4325536, 4325555, 4325575, 4325615, 4325634, 4325686, 5767298, 5767312, 5767367, 5767426, 5767427, 5767615, 5963890, 5964080, 5964084, 5964086, 5964122, 7471255, 7471303, 8061019, 8061102, 8061119, 8061202, 8061262, 8061268, 8519680, 8519746, 8519768, 8519771, 8519826, 8519829, 8519840, 8519871, 8519912, 8519990, 8520003, 8520127, 8585360, 8585390, 8585393, 8585495, 8585520, 8585550, 9437247, 9437400, 9437443, 9437494, 9437497, 9437524, 9437530, 9568515, 9568535, 9765096, 9896087, 9896113, 9896152, 9896210, 9896215, 9896274, 10485809, 10485848, 10485934, 10486068, 10486094, 11403330, 11403378, 11403408, 11403463, 11403488, 11403522, 11403610, 11599921, 11600016, 11600021, 11600185, 11600319, 11665413, 11665582, 11665754, 11731168, 11731267, 11731282, 11731290, 11731391, 12517490, 12517506, 12517520, 12517634, 12517689, 12517716, 12779569, 12779643, 12779715, 13041778, 13041813, 13041841, 14155925, 14156015, 14156084, 14156110, 14156223, 14680069, 14680127, 14680243, 14680280, 14680296, 14680303, 14680374, 15204503, 15204526, 15204675, 15204686, 15204692, 15204698, 15663195, 15663234, 15663248, 15663253, 15663281, 15663320, 15663336, 15663417, 15663450, 16908483, 16908487, 16908562, 16908622, 16908628, 16973938, 16974158, 16974164, 17956952, 17957103, 17957210, 18087936, 18088066, 18284674, 18284693, 18284768, 18284802, 18284884, 19923007, 19923058, 19923282, 19923290, 20185219, 20185237, 20185239, 20185428, 20316209, 20316248, 20316306, 20316320, 20316334, 20316439, 20316468, 20512834, 20512856, 20512942, 20512963, 20513091, 21168177, 21168191, 21168272, 21168277, 21889147, 21889173, 21889198, 21889282, 21889303, 21889328, 22151168, 22151234, 22151314, 22151319, 22151363, 22282289, 22282418, 22282479, 22282498, 22282687, 22675461, 22675505, 22675630, 22675634, 22675651, 22675672, 22675715, 29294592])), ('p6', None, ('set', [0, 91, 123, 258, 327746, 327810, 327811, 327875, 327919, 327990, 3211264, 3211269, 3211327, 3211604, 4128773, 4128817, 4128919, 4129215, 4325536, 4325555, 4325575, 4325615, 4325634, 4325686, 5767298, 5767312, 5767367, 5767426, 5767427, 5767615, 5963890, 5964080, 5964084, 5964086, 5964122, 7471255, 7471303, 8061019, 8061102, 8061119, 8061202, 8061262, 8061268, 8519680, 8519746, 8519768, 8519771, 8519826, 8519829, 8519840, 8519871, 8519912, 8519990, 8520003, 8520127, 8585360, 8585390, 8585393, 8585495, 8585520, 8585550, 9437247, 9437400, 9437443, 9437494, 9437497, 9437524, 9437530, 9568515, 9568535, 9765096, 9896087, 9896113, 9896152, 9896210, 9896215, 9896274, 10485809, 10485848, 10485934, 10486068, 10486094, 11403330, 11403378, 11403408, 11403463, 11403488, 11403522, 11403610, 11599921, 11600016, 11600021, 11600185, 11600319, 11665413, 11665582, 11665754, 11731168, 11731267, 11731282, 11731290, 11731391, 12517490, 12517506, 12517520, 12517634, 12517689, 12517716, 12779569, 12779643, 12779715, 13041778, 13041813, 13041841, 14155925, 14156015, 14156084, 14156110, 14156223, 14680069, 14680127, 14680243, 14680280, 14680296, 14680303, 14680374, 15204503, 15204526, 15204675, 15204686, 15204692, 15204698, 15663195, 15663234, 15663248, 15663253, 15663281, 15663320, 15663336, 15663417, 15663450, 16908483, 16908487, 16908562, 16908622, 16908628, 16973938, 16974158, 16974164, 17956952, 17957103, 17957210, 18087936, 18088066, 18284674, 18284693, 18284768, 18284802, 18284884, 19923007, 19923058, 19923282, 19923290, 20185219, 20185237, 20185239, 20185428, 20316209, 20316248, 20316306, 20316320, 20316334, 20316439, 20316468, 20512834, 20512856, 20512942, 20512963, 20513091, 21168177, 21168191, 21168272, 21168277, 21889147, 21889173, 21889198, 21889282, 21889303, 21889328, 22151168, 22151234, 22151314, 22151319, 22151363, 22282289, 22282418, 22282479, 22282498, 22282687, 22675461, 22675505, 22675630, 22675634, 22675651, 22675672, 22675715, 29294592])), ('p5', None, ('set', [130, 177, 258, 8519810, 8519811, 8519840, 8519854, 8519858, 8519938, 8519939, 8585376, 8585390, 8585474, 10485760, 10485890, 10485891, 10485934, 10485937, 10485939, 11403264, 11403441, 11403442, 11403523, 11600002, 11600003, 11600046, 11600051, 11600131, 11665408, 11665538, 11665568, 11665582, 11665667, 11731074, 11731075, 11731104, 11731118, 11731121, 11731122, 11731123, 11731202, 11731203, 16908418, 16908419, 16908448, 16908465, 16908466, 16908467, 16973954, 16973998, 16974001, 16974002, 16974082, 16974083])), ('p7', None, ('set', [130, 177, 258, 8519810, 8519811, 8519840, 8519854, 8519858, 8519938, 8519939, 8585376, 8585390, 8585474, 10485760, 10485890, 10485891, 10485934, 10485937, 10485939, 11403264, 11403441, 11403442, 11403523, 11600002, 11600003, 11600046, 11600051, 11600131, 11665408, 11665538, 11665568, 11665582, 11665667, 11731074, 11731075, 11731104, 11731118, 11731121, 11731122, 11731123, 11731202, 11731203, 16908418, 16908419, 16908448, 16908465, 16908466, 16908467, 16973954, 16973998, 16974001, 16974002, 16974082, 16974083])), ('c0', 112, ('range', 0, 300)), ('c0', 116, ('range', 0, 300)), ('c0', 120, ('range', 0, 300)), ('c0', 124, ('range', 0, 300)), ('c0', 128, ('range', 0, 300))]
FILLS = []
ARGRANGE = [(0, 5, 5), (1, 6, 6)]
SLOTBASE = [(96, 6682576, 1)]
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = []
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = []
ANSWERBY = []
ANSWERWIN = []
CDECL = False
TIB = False
ARGEQ = []
SEQ = []
ANSRANGE = []
CELLTHIS = []
VTALSO = [('p2', 0), ('p3', 0)]
SPARSE = []
GTAB = []
RECORD = []
RECORDORDER = False
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
MAXINSNS = 200000
ECXANSWER = [(0, 688, None), (1, 692, None), (2, 696, None), (4, 700, None), (5, 704, None), (6, 708, None), (7, 712, None)]
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
    for i in range(1800):
        h.brk = base  # the bump heap is reused a row; 1800 rows of windows outgrow it
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

# sweep_gen: ["obj", "--item", "3362", "--win", "0x40", "--own", "2", "--argrange", "0:5:5", "--argrange", "1:6:6", "--cell-win", "0xc061f0:0x100", "--place", "0xc061f8:0:0x20", "--place", "p0+0x10:0:0x1c", "--place", "p1+20:0:768", "--place", "p1+24:0:768", "--place", "p2+0x28c:0:16", "--place", "p2+0x2a8:0:16", "--place", "p3+0x28c:0:16", "--place", "p3+0x2a8:0:16", "--stub", "slot:0xc:0", "--stub", "slot:0x14:0", "--stub", "slot:0x1c:0", "--stub", "slot:0x60:2", "--stub", "slot:0xec:0", "--stub", "slot:0x10c:0", "--stub", "slot:0x130:0", "--stub", "slot:0x114:0", "--slotbase", "0x60:0x65f7d0!", "--vtat-also", "p2+0", "--vtat-also", "p3+0", "--ecx-answer", "0:0x2b0", "--ecx-answer", "1:0x2b4", "--ecx-answer", "2:0x2b8", "--ecx-answer", "4:0x2bc", "--ecx-answer", "5:0x2c0", "--ecx-answer", "6:0x2c4", "--ecx-answer", "7:0x2c8", "--wbias", "p2+0x1e4:0,1,2,4,8,9,10,12,16,24,32,33,34,36,40,48,64,72,73,80,96,128,129,130,132,136,144,160,192,256,260,264,288,384,512,513,520,544,640,1024,1025,1026,1028,1032,1040,1056,1088,1152,1280,1536,2048,2056,2080,2176,3072,4096,4097,4098,4100,4104,4112,4128,4160,4224,4352,4608,5120,6144,8192,8196,8200,8210,8224,8320,9216,12288,16384,16385,16386,16388,16392,16400,16416,16448,16512,16640,16896,17408,18432,20480,24576,32768,32776,32800,32896,33024,33792,34816,36864,49152,53248,65536,65537,65538,65540,65544,65552,65568,65600,65664,65792,65800,66048,66560,67584,69632,73728,81920,83080,98304,131072,131080,131104,131200,132096,135168,147456,196608,262144,262152,262160,262176,262272,263168,266240,270336,278528,327680,524288,524296,524320,524322,524416,524832,525312,526336,528384,540672,589824,1048576,1048584,1048592,1048608,1048704,1049088,1049600,1052672,1064960,1114112,2097152,2097160,2097184,2097280,2098176,2098304,2101248,2113536,2162688,2162752,4194304,4194306,4194312,4194336,4194432,4195328,4198400,4210688,4259840,6291456,6307840,8388608,8388616,8388640,8388672,8388736,8389184,8389632,8392704,8397312,8404992,8454144,8455168,8912896,16777216,16777217,16777218,16777220,16777224,16777232,16777248,16777280,16777344,16777472,16777728,16778240,16779264,16781312,16785408,16793600,16809984,16842752,16843264,16908288,17039360,17301504,17825792,18874368,20971520,25165824,33554432,33554440,33554464,33554560,33555456,33558528,33558536,33570816,33574912,33619968,33685504,50331648,67108864,67108868,67108872,67108880,67108896,67108992,67109888,67112960,67125248,67174400,67239936,67371008,76546048,83886080,134217728,134217736,134217760,134217856,134217984,134218752,134221824,134234112,134283264,135266304,150994944,268435456,268435464,268435488,268435584,268436480,268439552,268443664,268451840,268500992,268959744,270532608,285212672,536870912,536870913,536870914,536870916,536870920,536870928,536870944,536870976,536871040,536871168,536871176,536871424,536871936,536872960,536875008,536879104,536887296,536903680,536936448,537001984,537133056,537395200,537919488,538968064,541065216,545259520,553648128,570425344,603979776,671088640,805306368,1073741824,1073741832,1073741856,1073741952,1073742848,1073742850,1073745920,1073758208,1073807360,1073872896,1073872898,1086324736,1090519040,1107296320,1610612736,-2147483648,-2147483640,-2147483616,-2147483520,-2147483136,-2147482624,-2147481600,-2147479552,-2147475456,-2147467264,-2147418112,-2146959360,-2143289342,-2130706432,-1879048192,-1879048191,-1879048190,-1879048188,-1879048184,-1879048176,-1879048160,-1879048128,-1879048064,-1879047936,-1879047680,-1879047168,-1879046144,-1879044096,-1879040000,-1879031808,-1879015424,-1878982656,-1878917120,-1878786048,-1878523904,-1877999616,-1876951040,-1874853888,-1870659584,-1862270976,-1845493760,-1811939328,-1744830464,-1610612736,-1342177280,-805306368,1,2,4,8,16,32,64,128,256,512,1024,2048,4096,8192,16384,32768,65536,131072,262144,524288,1048576,2097152,4194304,8388608,16777216,33554432,67108864,134217728,268435456,536870912,1073741824,-2147483648,1,2,4,8,16,32,64,128,256,512,1024,2048,4096,8192,16384,32768,65536,131072,262144,524288,1048576,2097152,4194304,8388608,16777216,33554432,67108864,134217728,268435456,536870912,1073741824,-2147483648,1,2,4,8,16,32,64,128,256,512,1024,2048,4096,8192,16384,32768,65536,131072,262144,524288,1048576,2097152,4194304,8388608,16777216,33554432,67108864,134217728,268435456,536870912,1073741824,-2147483648,1,2,4,8,16,32,64,128,256,512,1024,2048,4096,8192,16384,32768,65536,131072,262144,524288,1048576,2097152,4194304,8388608,16777216,33554432,67108864,134217728,268435456,536870912,1073741824,-2147483648", "--wbias", "p2+0x218:0,1,2", "--wbias", "p2+4:50,51,52,53,61,62,400,64,256,414,543,216,239,123,131,66,346,323,338,334,340,304,308,310,313,63,88,174,91,191,195,199,224,177,178,179,232,146,160,149,151,144,114,130,258,259,279,276,274,447", "--wbias", "p2+0x2b0:0,0,1", "--wbias", "p2+0x2b4:0,0,1", "--wbias", "p2+0x2b8:0,0,1", "--wbias", "p2+0x2c0:0,0,1", "--wbias", "p2+0x2c4:0,0,1", "--wbias", "p2+0x2c8:0,0,1", "--wbias", "p2+0x2bc:0..6", "--wbias", "p2+0x280:1..4", "--wbias", "p2+0x29c:1..4", "--wbias", "p3+0x1e4:0,1,2,4,8,9,10,12,16,24,32,33,34,36,40,48,64,72,73,80,96,128,129,130,132,136,144,160,192,256,260,264,288,384,512,513,520,544,640,1024,1025,1026,1028,1032,1040,1056,1088,1152,1280,1536,2048,2056,2080,2176,3072,4096,4097,4098,4100,4104,4112,4128,4160,4224,4352,4608,5120,6144,8192,8196,8200,8210,8224,8320,9216,12288,16384,16385,16386,16388,16392,16400,16416,16448,16512,16640,16896,17408,18432,20480,24576,32768,32776,32800,32896,33024,33792,34816,36864,49152,53248,65536,65537,65538,65540,65544,65552,65568,65600,65664,65792,65800,66048,66560,67584,69632,73728,81920,83080,98304,131072,131080,131104,131200,132096,135168,147456,196608,262144,262152,262160,262176,262272,263168,266240,270336,278528,327680,524288,524296,524320,524322,524416,524832,525312,526336,528384,540672,589824,1048576,1048584,1048592,1048608,1048704,1049088,1049600,1052672,1064960,1114112,2097152,2097160,2097184,2097280,2098176,2098304,2101248,2113536,2162688,2162752,4194304,4194306,4194312,4194336,4194432,4195328,4198400,4210688,4259840,6291456,6307840,8388608,8388616,8388640,8388672,8388736,8389184,8389632,8392704,8397312,8404992,8454144,8455168,8912896,16777216,16777217,16777218,16777220,16777224,16777232,16777248,16777280,16777344,16777472,16777728,16778240,16779264,16781312,16785408,16793600,16809984,16842752,16843264,16908288,17039360,17301504,17825792,18874368,20971520,25165824,33554432,33554440,33554464,33554560,33555456,33558528,33558536,33570816,33574912,33619968,33685504,50331648,67108864,67108868,67108872,67108880,67108896,67108992,67109888,67112960,67125248,67174400,67239936,67371008,76546048,83886080,134217728,134217736,134217760,134217856,134217984,134218752,134221824,134234112,134283264,135266304,150994944,268435456,268435464,268435488,268435584,268436480,268439552,268443664,268451840,268500992,268959744,270532608,285212672,536870912,536870913,536870914,536870916,536870920,536870928,536870944,536870976,536871040,536871168,536871176,536871424,536871936,536872960,536875008,536879104,536887296,536903680,536936448,537001984,537133056,537395200,537919488,538968064,541065216,545259520,553648128,570425344,603979776,671088640,805306368,1073741824,1073741832,1073741856,1073741952,1073742848,1073742850,1073745920,1073758208,1073807360,1073872896,1073872898,1086324736,1090519040,1107296320,1610612736,-2147483648,-2147483640,-2147483616,-2147483520,-2147483136,-2147482624,-2147481600,-2147479552,-2147475456,-2147467264,-2147418112,-2146959360,-2143289342,-2130706432,-1879048192,-1879048191,-1879048190,-1879048188,-1879048184,-1879048176,-1879048160,-1879048128,-1879048064,-1879047936,-1879047680,-1879047168,-1879046144,-1879044096,-1879040000,-1879031808,-1879015424,-1878982656,-1878917120,-1878786048,-1878523904,-1877999616,-1876951040,-1874853888,-1870659584,-1862270976,-1845493760,-1811939328,-1744830464,-1610612736,-1342177280,-805306368,1,2,4,8,16,32,64,128,256,512,1024,2048,4096,8192,16384,32768,65536,131072,262144,524288,1048576,2097152,4194304,8388608,16777216,33554432,67108864,134217728,268435456,536870912,1073741824,-2147483648,1,2,4,8,16,32,64,128,256,512,1024,2048,4096,8192,16384,32768,65536,131072,262144,524288,1048576,2097152,4194304,8388608,16777216,33554432,67108864,134217728,268435456,536870912,1073741824,-2147483648,1,2,4,8,16,32,64,128,256,512,1024,2048,4096,8192,16384,32768,65536,131072,262144,524288,1048576,2097152,4194304,8388608,16777216,33554432,67108864,134217728,268435456,536870912,1073741824,-2147483648,1,2,4,8,16,32,64,128,256,512,1024,2048,4096,8192,16384,32768,65536,131072,262144,524288,1048576,2097152,4194304,8388608,16777216,33554432,67108864,134217728,268435456,536870912,1073741824,-2147483648", "--wbias", "p3+0x218:0,1,2", "--wbias", "p3+4:50,51,52,53,61,62,400,64,256,414,543,216,239,123,131,66,346,323,338,334,340,304,308,310,313,63,88,174,91,191,195,199,224,177,178,179,232,146,160,149,151,144,114,130,258,259,279,276,274,447", "--wbias", "p3+0x2b0:0,0,1", "--wbias", "p3+0x2b4:0,0,1", "--wbias", "p3+0x2b8:0,0,1", "--wbias", "p3+0x2c0:0,0,1", "--wbias", "p3+0x2c4:0,0,1", "--wbias", "p3+0x2c8:0,0,1", "--wbias", "p3+0x2bc:0..6", "--wbias", "p3+0x280:1..4", "--wbias", "p3+0x29c:1..4", "--wbias", "p4+*:0,91,123,258,327746,327810,327811,327875,327919,327990,3211264,3211269,3211327,3211604,4128773,4128817,4128919,4129215,4325536,4325555,4325575,4325615,4325634,4325686,5767298,5767312,5767367,5767426,5767427,5767615,5963890,5964080,5964084,5964086,5964122,7471255,7471303,8061019,8061102,8061119,8061202,8061262,8061268,8519680,8519746,8519768,8519771,8519826,8519829,8519840,8519871,8519912,8519990,8520003,8520127,8585360,8585390,8585393,8585495,8585520,8585550,9437247,9437400,9437443,9437494,9437497,9437524,9437530,9568515,9568535,9765096,9896087,9896113,9896152,9896210,9896215,9896274,10485809,10485848,10485934,10486068,10486094,11403330,11403378,11403408,11403463,11403488,11403522,11403610,11599921,11600016,11600021,11600185,11600319,11665413,11665582,11665754,11731168,11731267,11731282,11731290,11731391,12517490,12517506,12517520,12517634,12517689,12517716,12779569,12779643,12779715,13041778,13041813,13041841,14155925,14156015,14156084,14156110,14156223,14680069,14680127,14680243,14680280,14680296,14680303,14680374,15204503,15204526,15204675,15204686,15204692,15204698,15663195,15663234,15663248,15663253,15663281,15663320,15663336,15663417,15663450,16908483,16908487,16908562,16908622,16908628,16973938,16974158,16974164,17956952,17957103,17957210,18087936,18088066,18284674,18284693,18284768,18284802,18284884,19923007,19923058,19923282,19923290,20185219,20185237,20185239,20185428,20316209,20316248,20316306,20316320,20316334,20316439,20316468,20512834,20512856,20512942,20512963,20513091,21168177,21168191,21168272,21168277,21889147,21889173,21889198,21889282,21889303,21889328,22151168,22151234,22151314,22151319,22151363,22282289,22282418,22282479,22282498,22282687,22675461,22675505,22675630,22675634,22675651,22675672,22675715,29294592", "--wbias", "p6+*:0,91,123,258,327746,327810,327811,327875,327919,327990,3211264,3211269,3211327,3211604,4128773,4128817,4128919,4129215,4325536,4325555,4325575,4325615,4325634,4325686,5767298,5767312,5767367,5767426,5767427,5767615,5963890,5964080,5964084,5964086,5964122,7471255,7471303,8061019,8061102,8061119,8061202,8061262,8061268,8519680,8519746,8519768,8519771,8519826,8519829,8519840,8519871,8519912,8519990,8520003,8520127,8585360,8585390,8585393,8585495,8585520,8585550,9437247,9437400,9437443,9437494,9437497,9437524,9437530,9568515,9568535,9765096,9896087,9896113,9896152,9896210,9896215,9896274,10485809,10485848,10485934,10486068,10486094,11403330,11403378,11403408,11403463,11403488,11403522,11403610,11599921,11600016,11600021,11600185,11600319,11665413,11665582,11665754,11731168,11731267,11731282,11731290,11731391,12517490,12517506,12517520,12517634,12517689,12517716,12779569,12779643,12779715,13041778,13041813,13041841,14155925,14156015,14156084,14156110,14156223,14680069,14680127,14680243,14680280,14680296,14680303,14680374,15204503,15204526,15204675,15204686,15204692,15204698,15663195,15663234,15663248,15663253,15663281,15663320,15663336,15663417,15663450,16908483,16908487,16908562,16908622,16908628,16973938,16974158,16974164,17956952,17957103,17957210,18087936,18088066,18284674,18284693,18284768,18284802,18284884,19923007,19923058,19923282,19923290,20185219,20185237,20185239,20185428,20316209,20316248,20316306,20316320,20316334,20316439,20316468,20512834,20512856,20512942,20512963,20513091,21168177,21168191,21168272,21168277,21889147,21889173,21889198,21889282,21889303,21889328,22151168,22151234,22151314,22151319,22151363,22282289,22282418,22282479,22282498,22282687,22675461,22675505,22675630,22675634,22675651,22675672,22675715,29294592", "--wbias", "p5+*:130,177,258,8519810,8519811,8519840,8519854,8519858,8519938,8519939,8585376,8585390,8585474,10485760,10485890,10485891,10485934,10485937,10485939,11403264,11403441,11403442,11403523,11600002,11600003,11600046,11600051,11600131,11665408,11665538,11665568,11665582,11665667,11731074,11731075,11731104,11731118,11731121,11731122,11731123,11731202,11731203,16908418,16908419,16908448,16908465,16908466,16908467,16973954,16973998,16974001,16974002,16974082,16974083", "--wbias", "p7+*:130,177,258,8519810,8519811,8519840,8519854,8519858,8519938,8519939,8585376,8585390,8585474,10485760,10485890,10485891,10485934,10485937,10485939,11403264,11403441,11403442,11403523,11600002,11600003,11600046,11600051,11600131,11665408,11665538,11665568,11665582,11665667,11731074,11731075,11731104,11731118,11731121,11731122,11731123,11731202,11731203,16908418,16908419,16908448,16908465,16908466,16908467,16973954,16973998,16974001,16974002,16974082,16974083", "--wbias", "c0+0x70:0..300", "--wbias", "c0+0x74:0..300", "--wbias", "c0+0x78:0..300", "--wbias", "c0+0x7c:0..300", "--wbias", "c0+0x80:0..300", "--rows", "1800", "--port", "0", "Balance::type_damage@0057fb50"]
