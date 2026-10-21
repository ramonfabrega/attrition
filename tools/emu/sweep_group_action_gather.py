#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_group_action_gather.py — `Group::action_gather@00700b90` under unicorn (item 3386, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_group_action_gather.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 2272-byte object of random bytes (no bias), 2 stack argument(s);
`p` windows in order: p0..p109 the place(s), in flag order, p110..p113 the stubs' answer window(s); the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00cae5fc points at 256 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c061a8 points at 32 random bytes; the global cell 0x00c061ec points at 2096 random bytes; 4 random bytes at 0x00cc02f8 (each word in 0..0); 4 random bytes at 0x00e3a390 (each word in 0..8191); 4 random bytes at 0x00e4127c (each word in 0..8191); vtable slot +0x14 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x158 (0 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x00711720 (3 args) stubbed; callee 0x006e33a0 (2 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x00675bc0 (3 args) stubbed; callee 0x0064dda0 (2 args) stubbed; callee 0x00608450 (0 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x005e4a60 (7 args) stubbed; callee 0x0061a5c0 (3 args) stubbed; callee 0x00646c40 (1 args) stubbed; callee 0x005f9400 (2 args) stubbed; callee 0x005e3860 (0 args) stubbed; callee 0x0061a2e0 (3 args) stubbed; callee 0x00647080 (3 args) stubbed; callee 0x0062f710 (0 args) stubbed; callee 0x00714700 (0 args) stubbed; callee 0x00704990 (11 args) stubbed; the word at `c0+0x14+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 96 random bytes less `0` (placed); the word at `p0+4` points at 96 random bytes less `0` (placed); the word at `p0+8` points at 96 random bytes less `0` (placed); the word at `p0+12` points at 96 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 96 random bytes less `0` (placed); the word at `p5+4` points at 96 random bytes less `0` (placed); the word at `p5+8` points at 96 random bytes less `0` (placed); the word at `p5+12` points at 96 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 144 random bytes less `0` (placed); the word at `p10+4` points at 144 random bytes less `0` (placed); the word at `p10+8` points at 144 random bytes less `0` (placed); the word at `p10+12` points at 144 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 144 random bytes less `0` (placed); the word at `p15+4` points at 144 random bytes less `0` (placed); the word at `p15+8` points at 144 random bytes less `0` (placed); the word at `p15+12` points at 144 random bytes less `0` (placed); the word at `0xe85ddc` points at 216 random bytes less `0` (placed); the word at `p20+0` points at 8 random bytes less `0` (placed); the word at `p20+4` points at 8 random bytes less `0` (placed); the word at `p20+8` points at 8 random bytes less `0` (placed); the word at `p20+12` points at 8 random bytes less `0` (placed); the word at `p20+16` points at 8 random bytes less `0` (placed); the word at `p20+20` points at 8 random bytes less `0` (placed); the word at `p20+24` points at 8 random bytes less `0` (placed); the word at `p20+28` points at 8 random bytes less `0` (placed); the word at `p20+32` points at 8 random bytes less `0` (placed); the word at `p20+36` points at 8 random bytes less `0` (placed); the word at `p20+40` points at 8 random bytes less `0` (placed); the word at `p20+44` points at 8 random bytes less `0` (placed); the word at `p20+48` points at 8 random bytes less `0` (placed); the word at `p20+52` points at 8 random bytes less `0` (placed); the word at `p20+56` points at 8 random bytes less `0` (placed); the word at `p20+60` points at 8 random bytes less `0` (placed); the word at `p20+64` points at 8 random bytes less `0` (placed); the word at `p20+68` points at 8 random bytes less `0` (placed); the word at `p20+72` points at 8 random bytes less `0` (placed); the word at `p20+76` points at 8 random bytes less `0` (placed); the word at `p20+80` points at 8 random bytes less `0` (placed); the word at `p20+84` points at 8 random bytes less `0` (placed); the word at `p20+88` points at 8 random bytes less `0` (placed); the word at `p20+92` points at 8 random bytes less `0` (placed); the word at `p20+96` points at 8 random bytes less `0` (placed); the word at `p20+100` points at 8 random bytes less `0` (placed); the word at `p20+104` points at 8 random bytes less `0` (placed); the word at `p20+108` points at 8 random bytes less `0` (placed); the word at `p20+112` points at 8 random bytes less `0` (placed); the word at `p20+116` points at 8 random bytes less `0` (placed); the word at `p20+120` points at 8 random bytes less `0` (placed); the word at `p20+124` points at 8 random bytes less `0` (placed); the word at `p20+128` points at 8 random bytes less `0` (placed); the word at `p20+132` points at 8 random bytes less `0` (placed); the word at `p20+136` points at 8 random bytes less `0` (placed); the word at `p20+140` points at 8 random bytes less `0` (placed); the word at `p20+144` points at 8 random bytes less `0` (placed); the word at `p20+148` points at 8 random bytes less `0` (placed); the word at `p20+152` points at 8 random bytes less `0` (placed); the word at `p20+156` points at 8 random bytes less `0` (placed); the word at `p20+160` points at 8 random bytes less `0` (placed); the word at `p20+164` points at 8 random bytes less `0` (placed); the word at `p20+168` points at 8 random bytes less `0` (placed); the word at `p20+172` points at 8 random bytes less `0` (placed); the word at `p20+176` points at 8 random bytes less `0` (placed); the word at `p20+180` points at 8 random bytes less `0` (placed); the word at `p20+184` points at 8 random bytes less `0` (placed); the word at `p20+188` points at 8 random bytes less `0` (placed); the word at `p20+192` points at 8 random bytes less `0` (placed); the word at `p20+196` points at 8 random bytes less `0` (placed); the word at `p20+200` points at 8 random bytes less `0` (placed); the word at `p20+204` points at 8 random bytes less `0` (placed); the word at `p20+208` points at 8 random bytes less `0` (placed); the word at `p20+212` points at 8 random bytes less `0` (placed); the word at `p1+0x18` points at 32 random bytes less `0` (placed); the word at `p2+0x18` points at 32 random bytes less `0` (placed); the word at `p3+0x18` points at 32 random bytes less `0` (placed); the word at `p4+0x18` points at 32 random bytes less `0` (placed); the word at `p6+0x18` points at 32 random bytes less `0` (placed); the word at `p7+0x18` points at 32 random bytes less `0` (placed); the word at `p8+0x18` points at 32 random bytes less `0` (placed); the word at `p9+0x18` points at 32 random bytes less `0` (placed); the word at `p1+0x40` points at 32 random bytes less `0` (placed); the word at `p2+0x40` points at 32 random bytes less `0` (placed); the word at `p3+0x40` points at 32 random bytes less `0` (placed); the word at `p4+0x40` points at 32 random bytes less `0` (placed); the word at `p6+0x40` points at 32 random bytes less `0` (placed); the word at `p7+0x40` points at 32 random bytes less `0` (placed); the word at `p8+0x40` points at 32 random bytes less `0` (placed); the word at `p9+0x40` points at 32 random bytes less `0` (placed); the word at `p83+0x18` points at 32 random bytes less `0` (placed); the word at `p84+0x18` points at 32 random bytes less `0` (placed); the word at `p85+0x18` points at 32 random bytes less `0` (placed); the word at `p86+0x18` points at 32 random bytes less `0` (placed); the word at `p87+0x18` points at 32 random bytes less `0` (placed); the word at `p88+0x18` points at 32 random bytes less `0` (placed); the word at `p89+0x18` points at 32 random bytes less `0` (placed); the word at `p90+0x18` points at 32 random bytes less `0` (placed); the word at `p11+0x18` points at 32 random bytes less `0` (placed); the word at `p12+0x18` points at 32 random bytes less `0` (placed); the word at `p13+0x18` points at 32 random bytes less `0` (placed); the word at `p14+0x18` points at 32 random bytes less `0` (placed); the word at `p16+0x18` points at 32 random bytes less `0` (placed); the word at `p17+0x18` points at 32 random bytes less `0` (placed); the word at `p18+0x18` points at 32 random bytes less `0` (placed); the word at `p19+0x18` points at 32 random bytes less `0` (placed); the word at `c3+0x10` points at 2656 random bytes less `0` (placed); the word at `c2+0x134` points at 512 random bytes less `0` (placed); the word at `c2+0x138` points at 512 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0x158 laid as 0x0060eda0 or its stub, a flag after the answers; stub 1 answered from [7, 1, 3, 0, 2] (90%); stub 2 answered from [0, 1, 1, 1] (90%); stub 3 answered from [0, 1, 1, 1] (90%); stub 5 answered from [1, 1, 1, 1, 0] (90%); stub 6 answered from [1, 1, 1, 1, 0] (90%); stub 7 answered from [1, 1, 1, 0] (90%); stub 8 answered from [1, 1, 1, 0] (90%); stub 10 answered from [0, 1, 1, 1] (90%); stub 13 answered from [0, 0, 1, 2] (90%); stub 14 answered from [1, 1, 1, 1, 0] (90%); stub 16 answered from [0, 0, 0, 1] (90%); stub 17 answered from [0, 1, 1] (90%); stub 22 answered from [0, 1, 1] (90%); stub 28 answered from [0, 1, 1, 2] (90%); stub 18 answers a window of its own to each call, in call order (-64 bytes, ...); stub 1 answers 12 values in call order, printed after the answers; stub 2 answers 12 values in call order, printed after the answers; stub 3 answers 12 values in call order, printed after the answers; stub 5 answers 24 values in call order, printed after the answers; stub 6 answers 24 values in call order, printed after the answers; stub 7 answers 24 values in call order, printed after the answers; stub 8 answers 24 values in call order, printed after the answers; stub 10 answers 24 values in call order, printed after the answers; stub 13 answers 3 values in call order, printed after the answers; stub 14 answers 2 values in call order, printed after the answers; stub 16 answers 16 values in call order, printed after the answers; stub 17 answers 8 values in call order, printed after the answers; stub 22 answers 4 values in call order, printed after the answers; stub 28 answers 2 values in call order, printed after the answers; stub 19 answers 40 values in call order, printed after the answers; stub 19 answers 0..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 19 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 19 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); the stack 2816 bytes under the call is zeroed before each row; stub 4 answers from its own `ecx`: the word at `+0x40`; stub 9 answers from its own `ecx`: the word at `+0x40`; figures p1..p4 link at +0x2c (2 bytes) below their own index or to -1; figures p6..p9 link at +0x2c (2 bytes) below their own index or to -1; figures p1..p4 hold a short at +0x2e drawn from [0, 1], after the links; figures p6..p9 hold a short at +0x2e drawn from [0, 1], after the links; the stubbed vtable laid at `p75+0` as well as at `this`; the stubbed vtable laid at `p76+0` as well as at `this`; the stubbed vtable laid at `p77+0` as well as at `this`; the stubbed vtable laid at `p78+0` as well as at `this`; the stubbed vtable laid at `p79+0` as well as at `this`; the stubbed vtable laid at `p80+0` as well as at `this`; the stubbed vtable laid at `p81+0` as well as at `this`; the stubbed vtable laid at `p82+0` as well as at `this`; the stubbed vtable laid at `p91+0` as well as at `this`; the stubbed vtable laid at `p92+0` as well as at `this`; the stubbed vtable laid at `p93+0` as well as at `this`; the stubbed vtable laid at `p94+0` as well as at `this`; the stubbed vtable laid at `p95+0` as well as at `this`; the stubbed vtable laid at `p96+0` as well as at `this`; the stubbed vtable laid at `p97+0` as well as at `this`; the stubbed vtable laid at `p98+0` as well as at `this`; the stubbed vtable laid at `p110+0` as well as at `this`; the stubbed vtable laid at `p111+0` as well as at `this`; the stubbed vtable laid at `p112+0` as well as at `this`; the stubbed vtable laid at `p113+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`.
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
import sweeplib  # noqa: E402

ENTRY = 0x00700b90
WIN = 2272
OWN = 2
BIAS = []
ARGSET = [(1, [0, 2, 3, 2, 1, 2])]
ARGFROM = []
CELLS = [(12607884, 64), (13297148, 256), (12607880, 320), (12607912, 32), (12607980, 2096)]
TLS = []
OUTRET = []
TABLEARG = None
AT = [(13370104, 4), (14918544, 4), (14946940, 4)]
ATRANGE = {13370104: (0, 0, 4), 14918544: (0, 8191, 4), 14946940: (0, 8191, 4)}
PTRARGS = []
PNULLS = []
STUBS = [('slot', 20, 0), ('slot', 16, 0), ('slot', 184, 2), ('slot', 96, 2), ('slot', 172, 0), ('slot', 8, 0), ('slot', 188, 0), ('slot', 12, 0), ('slot', 76, 0), ('slot', 176, 0), ('slot', 144, 0), ('slot', 344, 0), ('call', 7421568, 1), ('call', 7411488, 3), ('call', 7222176, 2), ('call', 7422800, 4), ('call', 6773696, 3), ('call', 6610336, 2), ('call', 6325328, 0), ('call', 7033568, 2), ('call', 6179424, 7), ('call', 6399424, 3), ('call', 6581312, 1), ('call', 6263808, 2), ('call', 6174816, 0), ('call', 6398688, 3), ('call', 6582400, 3), ('call', 6485776, 0), ('call', 7423744, 0), ('call', 7358864, 11)]
PLACES = [('c0', '0x14+0x1c*0', '0', 16), ('p0', '0', '0', 96), ('p0', '4', '0', 96), ('p0', '8', '0', 96), ('p0', '12', '0', 96), ('c0', '0x14+0x1c*1', '0', 16), ('p5', '0', '0', 96), ('p5', '4', '0', 96), ('p5', '8', '0', 96), ('p5', '12', '0', 96), (None, '0xc0aec0+0x1c*0', '0', 16), ('p10', '0', '0', 144), ('p10', '4', '0', 144), ('p10', '8', '0', 144), ('p10', '12', '0', 144), (None, '0xc0aec0+0x1c*1', '0', 16), ('p15', '0', '0', 144), ('p15', '4', '0', 144), ('p15', '8', '0', 144), ('p15', '12', '0', 144), (None, '0xe85ddc', '0', 216), ('p20', '0', '0', 8), ('p20', '4', '0', 8), ('p20', '8', '0', 8), ('p20', '12', '0', 8), ('p20', '16', '0', 8), ('p20', '20', '0', 8), ('p20', '24', '0', 8), ('p20', '28', '0', 8), ('p20', '32', '0', 8), ('p20', '36', '0', 8), ('p20', '40', '0', 8), ('p20', '44', '0', 8), ('p20', '48', '0', 8), ('p20', '52', '0', 8), ('p20', '56', '0', 8), ('p20', '60', '0', 8), ('p20', '64', '0', 8), ('p20', '68', '0', 8), ('p20', '72', '0', 8), ('p20', '76', '0', 8), ('p20', '80', '0', 8), ('p20', '84', '0', 8), ('p20', '88', '0', 8), ('p20', '92', '0', 8), ('p20', '96', '0', 8), ('p20', '100', '0', 8), ('p20', '104', '0', 8), ('p20', '108', '0', 8), ('p20', '112', '0', 8), ('p20', '116', '0', 8), ('p20', '120', '0', 8), ('p20', '124', '0', 8), ('p20', '128', '0', 8), ('p20', '132', '0', 8), ('p20', '136', '0', 8), ('p20', '140', '0', 8), ('p20', '144', '0', 8), ('p20', '148', '0', 8), ('p20', '152', '0', 8), ('p20', '156', '0', 8), ('p20', '160', '0', 8), ('p20', '164', '0', 8), ('p20', '168', '0', 8), ('p20', '172', '0', 8), ('p20', '176', '0', 8), ('p20', '180', '0', 8), ('p20', '184', '0', 8), ('p20', '188', '0', 8), ('p20', '192', '0', 8), ('p20', '196', '0', 8), ('p20', '200', '0', 8), ('p20', '204', '0', 8), ('p20', '208', '0', 8), ('p20', '212', '0', 8), ('p1', '0x18', '0', 32), ('p2', '0x18', '0', 32), ('p3', '0x18', '0', 32), ('p4', '0x18', '0', 32), ('p6', '0x18', '0', 32), ('p7', '0x18', '0', 32), ('p8', '0x18', '0', 32), ('p9', '0x18', '0', 32), ('p1', '0x40', '0', 32), ('p2', '0x40', '0', 32), ('p3', '0x40', '0', 32), ('p4', '0x40', '0', 32), ('p6', '0x40', '0', 32), ('p7', '0x40', '0', 32), ('p8', '0x40', '0', 32), ('p9', '0x40', '0', 32), ('p83', '0x18', '0', 32), ('p84', '0x18', '0', 32), ('p85', '0x18', '0', 32), ('p86', '0x18', '0', 32), ('p87', '0x18', '0', 32), ('p88', '0x18', '0', 32), ('p89', '0x18', '0', 32), ('p90', '0x18', '0', 32), ('p11', '0x18', '0', 32), ('p12', '0x18', '0', 32), ('p13', '0x18', '0', 32), ('p14', '0x18', '0', 32), ('p16', '0x18', '0', 32), ('p17', '0x18', '0', 32), ('p18', '0x18', '0', 32), ('p19', '0x18', '0', 32), ('c3', '0x10', '0', 2656), ('c2', '0x134', '0', 512), ('c2', '0x138', '0', 512)]
NULLS = []
NULLFORCE = {}
WBIAS = [('c4', 2080, ('set', [2048, 2048, 0, 6144])), ('m', 12, ('set', [0, 1, 2, 3, 4, 3, 2, 3])), ('m', 72, ('set', [0, 65536, 0, 65536, 256, 65792])), ('m', 2252, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('m', 2256, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 131072, 131073, 131074, 131075, 196608, 196609, 196610, 196611])), ('p1', 16, ('set', [407095, 407159, 407123, 407295, 407323, 407463, 407491, 406639, 406667, 406667, 406807, 407007, 406507])), ('p2', 16, ('set', [407095, 407159, 407123, 407295, 407323, 407463, 407491, 406639, 406667, 406667, 406807, 407007, 406507])), ('p3', 16, ('set', [407095, 407159, 407123, 407295, 407323, 407463, 407491, 406639, 406667, 406667, 406807, 407007, 406507])), ('p4', 16, ('set', [407095, 407159, 407123, 407295, 407323, 407463, 407491, 406639, 406667, 406667, 406807, 407007, 406507])), ('p1', 20, ('set', [407095, 407159, 407123, 407295, 407323, 407463, 407491, 406639, 406667, 406667, 406807, 407007, 406507])), ('p2', 20, ('set', [407095, 407159, 407123, 407295, 407323, 407463, 407491, 406639, 406667, 406667, 406807, 407007, 406507])), ('p3', 20, ('set', [407095, 407159, 407123, 407295, 407323, 407463, 407491, 406639, 406667, 406667, 406807, 407007, 406507])), ('p4', 20, ('set', [407095, 407159, 407123, 407295, 407323, 407463, 407491, 406639, 406667, 406667, 406807, 407007, 406507])), ('p1', 40, ('set', [0, 1, 65535, 65535, 2, 131071])), ('p2', 40, ('set', [0, 1, 65535, 65535, 2, 131071])), ('p3', 40, ('set', [0, 1, 65535, 65535, 2, 131071])), ('p4', 40, ('set', [0, 1, 65535, 65535, 2, 131071])), ('p6', 16, ('set', [407095, 407159, 407123, 407295, 407323, 407463, 407491, 406639, 406667, 406667, 406807, 407007, 406507])), ('p7', 16, ('set', [407095, 407159, 407123, 407295, 407323, 407463, 407491, 406639, 406667, 406667, 406807, 407007, 406507])), ('p8', 16, ('set', [407095, 407159, 407123, 407295, 407323, 407463, 407491, 406639, 406667, 406667, 406807, 407007, 406507])), ('p9', 16, ('set', [407095, 407159, 407123, 407295, 407323, 407463, 407491, 406639, 406667, 406667, 406807, 407007, 406507])), ('p6', 20, ('set', [407095, 407159, 407123, 407295, 407323, 407463, 407491, 406639, 406667, 406667, 406807, 407007, 406507])), ('p7', 20, ('set', [407095, 407159, 407123, 407295, 407323, 407463, 407491, 406639, 406667, 406667, 406807, 407007, 406507])), ('p8', 20, ('set', [407095, 407159, 407123, 407295, 407323, 407463, 407491, 406639, 406667, 406667, 406807, 407007, 406507])), ('p9', 20, ('set', [407095, 407159, 407123, 407295, 407323, 407463, 407491, 406639, 406667, 406667, 406807, 407007, 406507])), ('p6', 40, ('set', [0, 1, 65535, 65535, 2, 131071])), ('p7', 40, ('set', [0, 1, 65535, 65535, 2, 131071])), ('p8', 40, ('set', [0, 1, 65535, 65535, 2, 131071])), ('p9', 40, ('set', [0, 1, 65535, 65535, 2, 131071])), ('p11', 84, ('set', [0, 1, 2, 3, 4, 5, 6, 7, -1, -1])), ('p12', 84, ('set', [0, 1, 2, 3, 4, 5, 6, 7, -1, -1])), ('p13', 84, ('set', [0, 1, 2, 3, 4, 5, 6, 7, -1, -1])), ('p14', 84, ('set', [0, 1, 2, 3, 4, 5, 6, 7, -1, -1])), ('p11', 60, ('set', [0, 65536])), ('p12', 60, ('set', [0, 65536])), ('p13', 60, ('set', [0, 65536])), ('p14', 60, ('set', [0, 65536])), ('p11', 40, ('set', [0, 1, 2, 3, 65535, 65536])), ('p12', 40, ('set', [0, 1, 2, 3, 65535, 65536])), ('p13', 40, ('set', [0, 1, 2, 3, 65535, 65536])), ('p14', 40, ('set', [0, 1, 2, 3, 65535, 65536])), ('p16', 84, ('set', [0, 1, 2, 3, 4, 5, 6, 7, -1, -1])), ('p17', 84, ('set', [0, 1, 2, 3, 4, 5, 6, 7, -1, -1])), ('p18', 84, ('set', [0, 1, 2, 3, 4, 5, 6, 7, -1, -1])), ('p19', 84, ('set', [0, 1, 2, 3, 4, 5, 6, 7, -1, -1])), ('p16', 60, ('set', [0, 65536])), ('p17', 60, ('set', [0, 65536])), ('p18', 60, ('set', [0, 65536])), ('p19', 60, ('set', [0, 65536])), ('p16', 40, ('set', [0, 1, 2, 3, 65535, 65536])), ('p17', 40, ('set', [0, 1, 2, 3, 65535, 65536])), ('p18', 40, ('set', [0, 1, 2, 3, 65535, 65536])), ('p19', 40, ('set', [0, 1, 2, 3, 65535, 65536])), ('p91', 4, ('set', [417, 418, 419, 420, 421, 422, 416, 0])), ('p92', 4, ('set', [417, 418, 419, 420, 421, 422, 416, 0])), ('p93', 4, ('set', [417, 418, 419, 420, 421, 422, 416, 0])), ('p94', 4, ('set', [417, 418, 419, 420, 421, 422, 416, 0])), ('p95', 4, ('set', [417, 418, 419, 420, 421, 422, 416, 0])), ('p96', 4, ('set', [417, 418, 419, 420, 421, 422, 416, 0])), ('p97', 4, ('set', [417, 418, 419, 420, 421, 422, 416, 0])), ('p98', 4, ('set', [417, 418, 419, 420, 421, 422, 416, 0])), ('p99', 4, ('set', [50, 51, 52, 53, 50, 52, 0, 5])), ('p100', 4, ('set', [50, 51, 52, 53, 50, 52, 0, 5])), ('p101', 4, ('set', [50, 51, 52, 53, 50, 52, 0, 5])), ('p102', 4, ('set', [50, 51, 52, 53, 50, 52, 0, 5])), ('p103', 4, ('set', [50, 51, 52, 53, 50, 52, 0, 5])), ('p104', 4, ('set', [50, 51, 52, 53, 50, 52, 0, 5])), ('p105', 4, ('set', [50, 51, 52, 53, 50, 52, 0, 5])), ('p106', 4, ('set', [50, 51, 52, 53, 50, 52, 0, 5])), ('p21', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p22', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p23', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p24', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p25', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p26', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p27', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p28', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p29', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p30', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p31', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p32', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p33', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p34', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p35', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p36', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p37', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p38', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p39', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p40', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p41', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p42', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p43', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p44', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p45', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p46', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p47', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p48', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p49', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p50', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p51', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p52', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p53', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p54', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p55', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p56', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p57', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p58', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p59', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p60', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p61', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p62', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p63', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p64', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p65', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p66', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p67', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p68', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p69', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p70', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p71', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p72', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p73', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('p74', None, ('set', [50, 51, 52, 53, 0, 1, 2, 54])), ('c6', 0, ('set', [0, 0, 0, 256, 1024, 5, 511])), ('c7', 0, ('set', [0, 0, 0, 256, 1024, 5, 511])), ('c2', 0, ('set', [1, 2, 3, 3])), ('c2', 24, ('range', 4, 16)), ('c2', 28, ('range', 4, 16)), ('p108', 0, ('set', [0, 256, 257, 65792, 0, 256])), ('p108', 4, ('set', [0, 1, 2, 3, 65536, 65537, 196608, 131075])), ('p108', 8, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 0, 0, 1, 65535])), ('p108', 28, ('set', [0, 256, 257, 65792, 0, 256])), ('p108', 32, ('set', [0, 1, 2, 3, 65536, 65537, 196608, 131075])), ('p108', 36, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 0, 0, 1, 65535])), ('p108', 56, ('set', [0, 256, 257, 65792, 0, 256])), ('p108', 60, ('set', [0, 1, 2, 3, 65536, 65537, 196608, 131075])), ('p108', 64, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 0, 0, 1, 65535])), ('p108', 84, ('set', [0, 256, 257, 65792, 0, 256])), ('p108', 88, ('set', [0, 1, 2, 3, 65536, 65537, 196608, 131075])), ('p108', 92, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 0, 0, 1, 65535])), ('p108', 112, ('set', [0, 256, 257, 65792, 0, 256])), ('p108', 116, ('set', [0, 1, 2, 3, 65536, 65537, 196608, 131075])), ('p108', 120, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 0, 0, 1, 65535])), ('p108', 140, ('set', [0, 256, 257, 65792, 0, 256])), ('p108', 144, ('set', [0, 1, 2, 3, 65536, 65537, 196608, 131075])), ('p108', 148, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 0, 0, 1, 65535])), ('p108', 168, ('set', [0, 256, 257, 65792, 0, 256])), ('p108', 172, ('set', [0, 1, 2, 3, 65536, 65537, 196608, 131075])), ('p108', 176, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 0, 0, 1, 65535])), ('p108', 196, ('set', [0, 256, 257, 65792, 0, 256])), ('p108', 200, ('set', [0, 1, 2, 3, 65536, 65537, 196608, 131075])), ('p108', 204, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 0, 0, 1, 65535])), ('p108', 224, ('set', [0, 256, 257, 65792, 0, 256])), ('p108', 228, ('set', [0, 1, 2, 3, 65536, 65537, 196608, 131075])), ('p108', 232, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 0, 0, 1, 65535])), ('p108', 252, ('set', [0, 256, 257, 65792, 0, 256])), ('p108', 256, ('set', [0, 1, 2, 3, 65536, 65537, 196608, 131075])), ('p108', 260, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 0, 0, 1, 65535])), ('p108', 280, ('set', [0, 256, 257, 65792, 0, 256])), ('p108', 284, ('set', [0, 1, 2, 3, 65536, 65537, 196608, 131075])), ('p108', 288, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 0, 0, 1, 65535])), ('p108', 308, ('set', [0, 256, 257, 65792, 0, 256])), ('p108', 312, ('set', [0, 1, 2, 3, 65536, 65537, 196608, 131075])), ('p108', 316, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 0, 0, 1, 65535])), ('p108', 336, ('set', [0, 256, 257, 65792, 0, 256])), ('p108', 340, ('set', [0, 1, 2, 3, 65536, 65537, 196608, 131075])), ('p108', 344, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 0, 0, 1, 65535])), ('p108', 364, ('set', [0, 256, 257, 65792, 0, 256])), ('p108', 368, ('set', [0, 1, 2, 3, 65536, 65537, 196608, 131075])), ('p108', 372, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 0, 0, 1, 65535])), ('p108', 392, ('set', [0, 256, 257, 65792, 0, 256])), ('p108', 396, ('set', [0, 1, 2, 3, 65536, 65537, 196608, 131075])), ('p108', 400, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 0, 0, 1, 65535])), ('p108', 420, ('set', [0, 256, 257, 65792, 0, 256])), ('p108', 424, ('set', [0, 1, 2, 3, 65536, 65537, 196608, 131075])), ('p108', 428, ('set', [0, 1, 2, 3, 65536, 65537, 65538, 65539, 0, 0, 1, 65535]))]
FILLS = [('c1', 'i//3')]
ARGRANGE = [(0, 0, 3)]
SLOTBASE = [(184, 6633360), (344, 6352288)]
ECXINT = None
EDX = None
EDXWIN = None
XMM = []
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(1, [7, 1, 3, 0, 2]), (2, [0, 1, 1, 1]), (3, [0, 1, 1, 1]), (5, [1, 1, 1, 1, 0]), (6, [1, 1, 1, 1, 0]), (7, [1, 1, 1, 0]), (8, [1, 1, 1, 0]), (10, [0, 1, 1, 1]), (13, [0, 0, 1, 2]), (14, [1, 1, 1, 1, 0]), (16, [0, 0, 0, 1]), (17, [0, 1, 1]), (22, [0, 1, 1]), (28, [0, 1, 1, 2])]
VTAT = None
VTATHALF = []
VTIMAGE = []
ALIAS = []
ANSWERBY = []
ANSWERWIN = [(18, -64), (18, -64), (18, -64), (18, -64)]
CDECL = False
TIB = False
ARGEQ = []
SEQ = [(1, 12, None), (2, 12, None), (3, 12, None), (5, 24, None), (6, 24, None), (7, 24, None), (8, 24, None), (10, 24, None), (13, 3, None), (14, 2, None), (16, 16, None), (17, 8, None), (22, 4, None), (28, 2, None), (19, 40, None)]
ANSRANGE = [(19, 0, 3)]
CELLTHIS = []
VTALSO = [('p75', 0), ('p76', 0), ('p77', 0), ('p78', 0), ('p79', 0), ('p80', 0), ('p81', 0), ('p82', 0), ('p91', 0), ('p92', 0), ('p93', 0), ('p94', 0), ('p95', 0), ('p96', 0), ('p97', 0), ('p98', 0), ('p110', 0), ('p111', 0), ('p112', 0), ('p113', 0), ('p1', 0), ('p2', 0), ('p3', 0), ('p4', 0), ('p6', 0), ('p7', 0), ('p8', 0), ('p9', 0), ('p11', 0), ('p12', 0), ('p13', 0), ('p14', 0), ('p16', 0), ('p17', 0), ('p18', 0), ('p19', 0)]
SPARSE = []
GTAB = []
RECORD = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29]
RECORDORDER = True
RECORDEDX = False
ROWSPLIT = {}
LINKS = [(1, 4, 44, 2, '<', [-1]), (6, 9, 44, 2, '<', [-1])]
LINKHI = [(1, 4, 46, ('set', [0, 1])), (6, 9, 46, ('set', [0, 1]))]
OUTARG = []
CDECLSTUBS = []
STUBWRITE = []
STUBCOPY = []
STUBALLOC = []
STACKCLEAN = 2816
STOPSTUB = None
INITCALLS = []
MAXINSNS = 900000
ECXANSWER = [(4, 64, None), (9, 64, None)]
ECXOUT = []
RING = []
DEREF = [(19, 0, 4, None), (19, 1, 4, None)]
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


def tls_lay(write, tib, tls):
    """`--tls OFF:V` (item 3377, parked 3372d): `fs:[0x2c]` -> a TLS array of 64 slots in the TIB
    page, every slot one zeroed block whose word at OFF is V. A function-local static's `_TSS` guard
    compares the block's epoch (`+8`) with its own word, so 0 reads "already initialised"."""
    arr, block = tib + 0x100, tib + 0x200
    write(tib + 0x2C, arr.to_bytes(4, "little"))
    write(block, bytes(0xE00))
    for k in range(64):
        write(arr + 4 * k, block.to_bytes(4, "little"))
    for off, v in tls:
        write(block + off, (v & 0xFFFFFFFF).to_bytes(4, "little"))


def outret_answer(vals, write):
    """`--outret IDX:N:SPEC/..` (item 3377, parked 3372d): a struct-returning callee. MSVC passes
    the caller's buffer as a hidden first stack argument and the callee answers it; call k writes
    `vals[k]`'s words there (the last call's past N)."""
    turn = [0]

    def answer(ecx, args):
        words = vals[min(turn[0], len(vals) - 1)]
        turn[0] += 1
        at = args[0] & 0xFFFFFFFF
        write(at, b"".join((w & 0xFFFFFFFF).to_bytes(4, "little") for w in words))
        return at
    return answer


def table_hook(base, esize, n, draw, cells):
    """`--table-arg` (item 3377, parked 3372d): a read hook over an N-cell table that fills each
    cell the function reads the first time it is read: a value from `draw()`, written before the
    load (unicorn's read hook runs first, and the load sees the write), recorded in `cells` as the
    signed value the cell holds."""
    def hook(uc_, access, addr, size, value, _):
        first = max(addr - base, 0) // esize
        last = min((addr + size - 1 - base) // esize, n - 1)
        for k in range(first, last + 1):
            if k not in cells:
                b = (draw() & ((1 << 8 * esize) - 1)).to_bytes(esize, "little")
                cells[k] = int.from_bytes(b, "little", signed=True)
                uc_.mem_write(base + esize * k, b)
    return hook


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
    rng = random.Random(3386)
    # `--stub-cdecl IDX`: the callee's arguments are the caller's to pop (item 2325).
    h = Harness(argv[1], {a: (f"{a:x}", n, False, i in CDECLSTUBS) for i, (k, a, n) in enumerate(STUBS)
                          if k == "call"})
    uc = h.m.uc
    # `--tib`: an SEH-framed function reads and writes `fs:[0]` (item 3120).
    if TIB:
        set_tib(uc)
    if TLS:
        tls_lay(uc.mem_write, sweeplib.TIB, TLS)
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
    # `--table-arg` (item 3377): the table laid once, zeroed; a read hook fills the cells read.
    tcells = {}
    twritten = [False]
    if TABLEARG is not None:
        from unicorn import UC_HOOK_MEM_READ, UC_HOOK_MEM_WRITE
        tidx, trows, tcols, tesz, tvals = TABLEARG
        tsize = trows * tcols * tesz
        tbase = h.alloc(tsize + 0x20)
        uc.mem_write(tbase, bytes(tsize))
        uc.hook_add(UC_HOOK_MEM_READ, table_hook(tbase, tesz, trows * tcols, lambda: wdraw(rng, tvals), tcells),
                    begin=tbase, end=tbase + tsize - 1)
        uc.hook_add(UC_HOOK_MEM_WRITE, lambda *_: twritten.__setitem__(0, True), begin=tbase, end=tbase + tsize - 1)
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
        # `--stub-alloc IDX` (parked 3282b, 3339a, 3348b/c, 3372): a stubbed `malloc` answering a fresh
        # address of the bump heap each call, so a stack-resident `SimpleArray` grows into its own buffer.
        for idx in STUBALLOC:
            h.answers[names[idx]] = lambda ecx, args: h.alloc(max(args[0] & 0xFFFFFFFF, 16))
        derefs = [[] for _ in DEREF]
        for (idx, k, n, wr), sink in zip(DEREF, derefs):
            h.answers[names[idx]] = read_arg(h.answers[names[idx]], k, n,
                                             lambda a_, n_: bytes(uc.mem_read(a_, n_)), sink, wr)
        # `--ecx-out IDX:V,V,..` (item 2818, parked 2816): stub IDX leaves `ecx`
        # as a callee would, a value drawn from the list, after the other numbers on the row.
        ecxo = [rng.choice(vals) & 0xFFFFFFFF for _, vals in ECXOUT]
        for (idx, _), v in zip(ECXOUT, ecxo):
            h.answers[names[idx]] = pair_answer(h.answers[names[idx]], v)
        # `--outret IDX:N:SPEC/..` (item 3377): N calls' words drawn a row, printed last on the row.
        orets = []
        for idx, n, specs in OUTRET:
            vals = [[wdraw(rng, sp_) for sp_ in specs] for _ in range(n)]
            orets.append(vals)
            h.answers[names[idx]] = outret_answer(vals, uc.mem_write)
        if TABLEARG is not None:
            for k_ in tcells:
                uc.mem_write(tbase + tesz * k_, bytes(tesz))
            tcells.clear()
            twritten[0] = False
            args[tidx] = tbase
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
        assert not twritten[0], "wrote the --table-arg table"
        before = ";".join([bytes(mem).hex()] + [bytes(b).hex() for _, b in wins])
        after = ";".join([bytes(uc.mem_read(this, WIN)).hex()]
                         + [bytes(uc.mem_read(a, len(b))).hex() for a, b in wins])
        shown = ([str(a) for a in args] + [str(v) for v in answers] + [str(v) for v in outs]
                 + [str(v) for s in seqs for v in s] + [str(f) for f in based] + [str(f) for f in nullflags]
                 + ([] if ECXINT is None else [str(ecx)]) + ([] if EDX is None else [str(edx)])
                 + [str(v) for v in xmmv] + [str(v) for v in ecxo]
                 + [str(((w + 0x80000000) & 0xFFFFFFFF) - 0x80000000) for vals in orets for call in vals for w in call])
        tshown = "" if TABLEARG is None else " @ " + (",".join(f"{k_}:{v_}" for k_, v_ in sorted(tcells.items())) or "-")
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
        print(" ".join(shown) + f" | {before} -> {r} | {after}" + tshown + log)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

# sweep_gen: ["obj", "Group::action_gather@00700b90", "--item", "3386", "--name", "group_action_gather", "--win", "0x8e0", "--own", "2", "--ret", "none", "--rows", "2000", "--clean-stack", "0xb00", "--max-insns", "900000", "--cell-win", "0xc0618c:0x40", "--cell-win", "0xcae5fc:0x100", "--cell-win", "0xc06188:0x140", "--cell-win", "0xc061a8:0x20", "--cell-win", "0xc061ec:0x830", "--at", "0xcc02f8:4:0..0", "--at", "0xe3a390:4:0..0x1fff", "--at", "0xe4127c:4:0..0x1fff", "--stub", "slot:0x14:0", "--stub", "slot:0x10:0", "--stub", "slot:0xb8:2", "--stub", "slot:0x60:2", "--stub", "slot:0xac:0", "--stub", "slot:0x8:0", "--stub", "slot:0xbc:0", "--stub", "slot:0xc:0", "--stub", "slot:0x4c:0", "--stub", "slot:0xb0:0", "--stub", "slot:0x90:0", "--stub", "slot:0x158:0", "--stub", "0x713e80:1", "--stub", "0x711720:3", "--stub", "0x6e33a0:2", "--stub", "0x714350:4", "--stub", "0x675bc0:3", "--stub", "0x64dda0:2", "--stub", "0x608450:0", "--stub", "0x6b52e0:2", "--stub", "0x5e4a60:7", "--stub", "0x61a5c0:3", "--stub", "0x646c40:1", "--stub", "0x5f9400:2", "--stub", "0x5e3860:0", "--stub", "0x61a2e0:3", "--stub", "0x647080:3", "--stub", "0x62f710:0", "--stub", "0x714700:0", "--stub", "0x704990:11", "--ptrtab", "c0+0x14+0x1c*0:4:0x60:vt", "--ptrtab", "c0+0x14+0x1c*1:4:0x60:vt", "--ptrtab", "0xc0aec0+0x1c*0:4:0x90:vt", "--ptrtab", "0xc0aec0+0x1c*1:4:0x90:vt", "--ptrtab", "0xe85ddc:54:8", "--place", "p1+0x18:0:0x20", "--place", "p2+0x18:0:0x20", "--place", "p3+0x18:0:0x20", "--place", "p4+0x18:0:0x20", "--place", "p6+0x18:0:0x20", "--place", "p7+0x18:0:0x20", "--place", "p8+0x18:0:0x20", "--place", "p9+0x18:0:0x20", "--place", "p1+0x40:0:0x20", "--place", "p2+0x40:0:0x20", "--place", "p3+0x40:0:0x20", "--place", "p4+0x40:0:0x20", "--place", "p6+0x40:0:0x20", "--place", "p7+0x40:0:0x20", "--place", "p8+0x40:0:0x20", "--place", "p9+0x40:0:0x20", "--place", "p83+0x18:0:0x20", "--place", "p84+0x18:0:0x20", "--place", "p85+0x18:0:0x20", "--place", "p86+0x18:0:0x20", "--place", "p87+0x18:0:0x20", "--place", "p88+0x18:0:0x20", "--place", "p89+0x18:0:0x20", "--place", "p90+0x18:0:0x20", "--place", "p11+0x18:0:0x20", "--place", "p12+0x18:0:0x20", "--place", "p13+0x18:0:0x20", "--place", "p14+0x18:0:0x20", "--place", "p16+0x18:0:0x20", "--place", "p17+0x18:0:0x20", "--place", "p18+0x18:0:0x20", "--place", "p19+0x18:0:0x20", "--place", "c3+0x10:0:0xa60", "--place", "c2+0x134:0:0x200", "--place", "c2+0x138:0:0x200", "--ecx-answer", "4:0x40", "--ecx-answer", "9:0x40", "--slotbase", "0xb8:0x653790", "--slotbase", "0x158:0x60eda0", "--vtat-also", "p75+0", "--vtat-also", "p76+0", "--vtat-also", "p77+0", "--vtat-also", "p78+0", "--vtat-also", "p79+0", "--vtat-also", "p80+0", "--vtat-also", "p81+0", "--vtat-also", "p82+0", "--vtat-also", "p91+0", "--vtat-also", "p92+0", "--vtat-also", "p93+0", "--vtat-also", "p94+0", "--vtat-also", "p95+0", "--vtat-also", "p96+0", "--vtat-also", "p97+0", "--vtat-also", "p98+0", "--fill", "c1:i//3", "--argrange", "0:0:3", "--argset", "1:0,2,3,2,1,2", "--wbias", "c4+0x820:2048,2048,0,6144", "--answers", "1:7,1,3,0,2", "--answers", "2:0,1,1,1", "--answers", "3:0,1,1,1", "--answers", "5:1,1,1,1,0", "--answers", "6:1,1,1,1,0", "--answers", "7:1,1,1,0", "--answers", "8:1,1,1,0", "--answers", "10:0,1,1,1", "--answers", "13:0,0,1,2", "--answers", "14:1,1,1,1,0", "--answers", "16:0,0,0,1", "--answers", "17:0,1,1", "--answers", "22:0,1,1", "--answers", "28:0,1,1,2", "--seq", "1:12", "--seq", "2:12", "--seq", "3:12", "--seq", "5:24", "--seq", "6:24", "--seq", "7:24", "--seq", "8:24", "--seq", "10:24", "--seq", "13:3", "--seq", "14:2", "--seq", "16:16", "--seq", "17:8", "--seq", "22:4", "--seq", "28:2", "--seq", "19:40", "--answer-range", "19:0:3", "--answer-win", "18:-0x40", "--answer-win", "18:-0x40", "--answer-win", "18:-0x40", "--answer-win", "18:-0x40", "--vtat-also", "p110+0", "--vtat-also", "p111+0", "--vtat-also", "p112+0", "--vtat-also", "p113+0", "--wbias", "m+0xc:0,1,2,3,4,3,2,3", "--wbias", "m+0x48:0,65536,0,65536,256,65792", "--wbias", "m+0x8cc:0,1,2,3,65536,65537,65538,65539,131072,131073,131074,131075,196608,196609,196610,196611", "--wbias", "m+0x8d0:0,1,2,3,65536,65537,65538,65539,131072,131073,131074,131075,196608,196609,196610,196611", "--wbias", "p1..4+0x10:407095,407159,407123,407295,407323,407463,407491,406639,406667,406667,406807,407007,406507", "--wbias", "p1..4+0x14:407095,407159,407123,407295,407323,407463,407491,406639,406667,406667,406807,407007,406507", "--wbias", "p1..4+0x28:0,1,65535,65535,2,131071", "--link", "p1..4+0x2c:2:<:-1", "--link-hi", "p1..4+0x2e:0,1", "--wbias", "p6..9+0x10:407095,407159,407123,407295,407323,407463,407491,406639,406667,406667,406807,407007,406507", "--wbias", "p6..9+0x14:407095,407159,407123,407295,407323,407463,407491,406639,406667,406667,406807,407007,406507", "--wbias", "p6..9+0x28:0,1,65535,65535,2,131071", "--link", "p6..9+0x2c:2:<:-1", "--link-hi", "p6..9+0x2e:0,1", "--wbias", "p11..14+0x54:0,1,2,3,4,5,6,7,-1,-1", "--wbias", "p11..14+0x3c:0,65536", "--wbias", "p11..14+0x28:0,1,2,3,65535,65536", "--wbias", "p16..19+0x54:0,1,2,3,4,5,6,7,-1,-1", "--wbias", "p16..19+0x3c:0,65536", "--wbias", "p16..19+0x28:0,1,2,3,65535,65536", "--wbias", "p91..98+4:417,418,419,420,421,422,416,0", "--wbias", "p99..106+4:50,51,52,53,50,52,0,5", "--wbias", "p21..74+*:50,51,52,53,0,1,2,54", "--wbias", "c6+0:0,0,0,256,1024,5,511", "--wbias", "c7+0:0,0,0,256,1024,5,511", "--wbias", "c2+0:1,2,3,3", "--wbias", "c2+0x18:4..16", "--wbias", "c2+0x1c:4..16", "--wbias", "p108+0x0:0,256,257,65792,0,256", "--wbias", "p108+0x4:0,1,2,3,65536,65537,196608,131075", "--wbias", "p108+0x8:0,1,2,3,65536,65537,65538,65539,0,0,1,65535", "--wbias", "p108+0x1c:0,256,257,65792,0,256", "--wbias", "p108+0x20:0,1,2,3,65536,65537,196608,131075", "--wbias", "p108+0x24:0,1,2,3,65536,65537,65538,65539,0,0,1,65535", "--wbias", "p108+0x38:0,256,257,65792,0,256", "--wbias", "p108+0x3c:0,1,2,3,65536,65537,196608,131075", "--wbias", "p108+0x40:0,1,2,3,65536,65537,65538,65539,0,0,1,65535", "--wbias", "p108+0x54:0,256,257,65792,0,256", "--wbias", "p108+0x58:0,1,2,3,65536,65537,196608,131075", "--wbias", "p108+0x5c:0,1,2,3,65536,65537,65538,65539,0,0,1,65535", "--wbias", "p108+0x70:0,256,257,65792,0,256", "--wbias", "p108+0x74:0,1,2,3,65536,65537,196608,131075", "--wbias", "p108+0x78:0,1,2,3,65536,65537,65538,65539,0,0,1,65535", "--wbias", "p108+0x8c:0,256,257,65792,0,256", "--wbias", "p108+0x90:0,1,2,3,65536,65537,196608,131075", "--wbias", "p108+0x94:0,1,2,3,65536,65537,65538,65539,0,0,1,65535", "--wbias", "p108+0xa8:0,256,257,65792,0,256", "--wbias", "p108+0xac:0,1,2,3,65536,65537,196608,131075", "--wbias", "p108+0xb0:0,1,2,3,65536,65537,65538,65539,0,0,1,65535", "--wbias", "p108+0xc4:0,256,257,65792,0,256", "--wbias", "p108+0xc8:0,1,2,3,65536,65537,196608,131075", "--wbias", "p108+0xcc:0,1,2,3,65536,65537,65538,65539,0,0,1,65535", "--wbias", "p108+0xe0:0,256,257,65792,0,256", "--wbias", "p108+0xe4:0,1,2,3,65536,65537,196608,131075", "--wbias", "p108+0xe8:0,1,2,3,65536,65537,65538,65539,0,0,1,65535", "--wbias", "p108+0xfc:0,256,257,65792,0,256", "--wbias", "p108+0x100:0,1,2,3,65536,65537,196608,131075", "--wbias", "p108+0x104:0,1,2,3,65536,65537,65538,65539,0,0,1,65535", "--wbias", "p108+0x118:0,256,257,65792,0,256", "--wbias", "p108+0x11c:0,1,2,3,65536,65537,196608,131075", "--wbias", "p108+0x120:0,1,2,3,65536,65537,65538,65539,0,0,1,65535", "--wbias", "p108+0x134:0,256,257,65792,0,256", "--wbias", "p108+0x138:0,1,2,3,65536,65537,196608,131075", "--wbias", "p108+0x13c:0,1,2,3,65536,65537,65538,65539,0,0,1,65535", "--wbias", "p108+0x150:0,256,257,65792,0,256", "--wbias", "p108+0x154:0,1,2,3,65536,65537,196608,131075", "--wbias", "p108+0x158:0,1,2,3,65536,65537,65538,65539,0,0,1,65535", "--wbias", "p108+0x16c:0,256,257,65792,0,256", "--wbias", "p108+0x170:0,1,2,3,65536,65537,196608,131075", "--wbias", "p108+0x174:0,1,2,3,65536,65537,65538,65539,0,0,1,65535", "--wbias", "p108+0x188:0,256,257,65792,0,256", "--wbias", "p108+0x18c:0,1,2,3,65536,65537,196608,131075", "--wbias", "p108+0x190:0,1,2,3,65536,65537,65538,65539,0,0,1,65535", "--wbias", "p108+0x1a4:0,256,257,65792,0,256", "--wbias", "p108+0x1a8:0,1,2,3,65536,65537,196608,131075", "--wbias", "p108+0x1ac:0,1,2,3,65536,65537,65538,65539,0,0,1,65535", "--record-order", "--record", "0", "--record", "1", "--record", "2", "--record", "3", "--record", "4", "--record", "5", "--record", "6", "--record", "7", "--record", "8", "--record", "9", "--record", "10", "--record", "11", "--record", "12", "--record", "13", "--record", "14", "--record", "15", "--record", "16", "--record", "17", "--record", "18", "--record", "19", "--record", "20", "--record", "21", "--record", "22", "--record", "23", "--record", "24", "--record", "25", "--record", "26", "--record", "27", "--record", "28", "--record", "29", "--deref", "19:0:4", "--deref", "19:1:4", "--port", "{\n                // Window layout. `m` the group (`+0xc` count, `+0x10` form, `+0x4a` player, the shorts at\n                // `+0x8cc`); `c[0]` the objects cell, `c[1]` the `div_3_table`, `c[2]` the world, `c[3]` the\n                // spell types cell, `c[4]` the game, `c[5]` `ignore_orders` (held at 0: the head's list is\n                // `group_action_gather_head`'s), `c[6]`/`c[7]` the first word of players 0 and 1's leaders.\n                // `p[0..=4]` the objects' array of player 0 and its four objects, `p[5..=9]` player 1's,\n                // `p[10..=14]` and `p[15..=19]` the units', `p[20]` the types' array and `p[21..=74]` its\n                // records, `p[75 + 4 * pl + o]` an object's own type, `p[83 + ..]` its data (`+0x40`),\n                // `p[91 + ..]` that data's type, `p[99 + ..]` a unit's type, `p[107]` the spell types,\n                // `p[108]` the tile cells, `p[109]` the world's second array, `p[110..=113]` the orders\n                // `get_action` answers. `a`: the two arguments, the stubs' answers (`a[2 + k]`, stub `k`),\n                // every `--seq` run (`SEQ_AT`), the two `--slotbase` flags and the four null flags.\n                //\n                // **The port is the crate's own steps**, `crate::action_gather_view::action_gather_steps`,\n                // over a view of the emulated original's windows; the crate has no `Group::action_gather`\n                // to call (see that module's head).\n                use crate::action_gather_view::{GarrisonHead, GatherView, action_gather_steps};\n                const SEQ_AT: [(usize, usize, usize); 15] = [\n                    (1, 32, 12),\n                    (2, 44, 12),\n                    (3, 56, 12),\n                    (5, 68, 24),\n                    (6, 92, 24),\n                    (7, 116, 24),\n                    (8, 140, 24),\n                    (10, 164, 24),\n                    (13, 188, 3),\n                    (14, 191, 2),\n                    (16, 193, 16),\n                    (17, 209, 8),\n                    (22, 217, 4),\n                    (28, 221, 2),\n                    (19, 223, 40),\n                ];\n                const FLAG_B8: usize = 263;\n                const FLAG_158: usize = 264;\n                const NULL_AT: usize = 265;\n                struct V<'a> {\n                    a: &'a [i32],\n                    m: &'a mut [u8],\n                    c: &'a [Vec<u8>],\n                    p: &'a mut [Vec<u8>],\n                    log: &'a mut Vec<String>,\n                    turn: [usize; 30],\n                    deref: [Vec<String>; 2],\n                }\n                fn rd32(w: &[u8], o: usize) -> i32 {\n                    i32::from_le_bytes(w[o..o + 4].try_into().expect(\"four bytes\"))\n                }\n                fn rd16(w: &[u8], o: usize) -> i32 {\n                    i32::from(i16::from_le_bytes([w[o], w[o + 1]]))\n                }\n                fn ow(p: i32, o: i32) -> usize {\n                    (if p == 0 { 1 } else { 6 }) + o as usize\n                }\n                fn uw(p: i32, u: i32) -> usize {\n                    (if p == 0 { 11 } else { 16 }) + u as usize\n                }\n                fn k(p: i32, o: i32) -> usize {\n                    4 * p as usize + o as usize\n                }\n                impl V<'_> {\n                    fn seq(&mut self, s: usize) -> i32 {\n                        let &(_, at, n) = SEQ_AT.iter().find(|e| e.0 == s).expect(\"a seq stub\");\n                        let i = self.turn[s].min(n - 1);\n                        self.turn[s] += 1;\n                        self.a[at + i]\n                    }\n                    fn rec(&mut self, s: usize, ecx: &str, args: &[i32]) {\n                        let mut t = format!(\"s{s}:{ecx}\");\n                        for v in args {\n                            t.push_str(&format!(\",{v}\"));\n                        }\n                        self.log.push(t);\n                    }\n                }\n                impl GarrisonHead for V<'_> {\n                    fn who(&self) -> i32 {\n                        i32::from(self.m[0x4a])\n                    }\n                    fn ignore_orders(&self) -> bool {\n                        rd32(&self.c[5], 0) != 0\n                    }\n                    fn ignoring_count(&self, _who: i32) -> i32 {\n                        unreachable!(\"the whole-function sweep keeps ignore_orders at 0\")\n                    }\n                    fn ignoring_at(&self, _who: i32, _i: i32) -> i32 {\n                        unreachable!(\"the whole-function sweep keeps ignore_orders at 0\")\n                    }\n                    fn drop_ignored(&mut self, _o: i32, _who: i32) {\n                        unreachable!(\"the whole-function sweep keeps ignore_orders at 0\")\n                    }\n                    fn refresh(&mut self) {\n                        self.rec(0, \"m+0x0\", &[]);\n                    }\n                }\n                impl GatherView for V<'_> {\n                    fn list_clear(&mut self) {\n                        self.rec(12, \"sp-0xa20\", &[-1]);\n                    }\n                    fn set_form(&mut self) {\n                        self.m[0x10..0x14].copy_from_slice(&(-1i32).to_le_bytes());\n                    }\n                    fn count(&self) -> i32 {\n                        rd32(self.m, 0xc)\n                    }\n                    fn member(&self, i: i32) -> i32 {\n                        rd16(self.m, 0x8cc + 2 * i as usize)\n                    }\n                    fn count_kind(&mut self, kind: i32, ty: i32) -> i32 {\n                        self.rec(13, \"m+0x0\", &[kind, ty, 0]);\n                        self.seq(13)\n                    }\n                    fn world_w(&self) -> i32 {\n                        rd32(&self.c[2], 0)\n                    }\n                    fn world_dim_x(&self) -> i32 {\n                        rd32(&self.c[2], 0x18)\n                    }\n                    fn world_dim_y(&self) -> i32 {\n                        rd32(&self.c[2], 0x1c)\n                    }\n                    fn div3(&self, i: i32) -> i32 {\n                        rd32(&self.c[1], 4 * i as usize)\n                    }\n                    fn cell_i16(&self, idx: i32, off: i32) -> i32 {\n                        rd16(&self.p[108], 0x1c * idx as usize + off as usize)\n                    }\n                    fn ext_byte(&self, i: i32) -> i32 {\n                        i32::from(self.p[109][2 * i as usize])\n                    }\n                    fn get_tregion(&mut self, tx: i32, ty: i32) -> i32 {\n                        let hex = |v: i32| v.to_le_bytes().iter().map(|b| format!(\"{b:02x}\")).collect::<String>();\n                        self.log.push(\"s19:c2+0x0,sp-0x18,sp-0x14\".into());\n                        self.deref[0].push(hex(tx));\n                        self.deref[1].push(hex(ty));\n                        self.seq(19)\n                    }\n                    fn obj_pos_raw(&self, p: i32, o: i32) -> (i32, i32) {\n                        let w = &self.p[ow(p, o)];\n                        (rd32(w, 0x10), rd32(w, 0x14))\n                    }\n                    fn obj_alive(&mut self, p: i32, o: i32) -> bool {\n                        self.rec(5, &format!(\"p{}+0x0\", ow(p, o)), &[]);\n                        self.seq(5) != 0\n                    }\n                    fn obj_bc(&mut self, p: i32, o: i32) -> bool {\n                        self.rec(6, &format!(\"p{}+0x0\", ow(p, o)), &[]);\n                        self.seq(6) != 0\n                    }\n                    fn obj_is(&mut self, p: i32, o: i32, ty: i32) -> bool {\n                        if self.a[FLAG_B8] != 0 {\n                            self.rec(3, &format!(\"p{}+0x0\", 75 + k(p, o)), &[ty, 0]);\n                            self.seq(3) != 0\n                        } else {\n                            self.rec(2, &format!(\"p{}+0x0\", ow(p, o)), &[ty, 0]);\n                            self.seq(2) != 0\n                        }\n                    }\n                    fn obj_data_kind(&mut self, p: i32, o: i32) -> i32 {\n                        self.rec(4, &format!(\"p{}+0x0\", ow(p, o)), &[]);\n                        rd32(&self.p[91 + k(p, o)], 4)\n                    }\n                    fn type_avail(&mut self, p: i32, kind: i32) -> bool {\n                        self.rec(14, &format!(\"c{}+0x0\", 6 + p), &[kind, 1]);\n                        self.seq(14) != 0\n                    }\n                    fn leader_flag(&self, p: i32) -> bool {\n                        rd32(&self.c[6 + p as usize], 0) & 0x700 != 0\n                    }\n                    fn unit_type_override(&self, p: i32, u: i32) -> i32 {\n                        rd32(&self.p[uw(p, u)], 0x54)\n                    }\n                    fn unit_kind(&self, p: i32, u: i32) -> i32 {\n                        rd32(&self.p[99 + k(p, u)], 4)\n                    }\n                    fn type_kind(&self, ty: i32) -> i32 {\n                        rd32(&self.p[21 + ty as usize], 4)\n                    }\n                    fn is_castable(&mut self, p: i32, u: i32) -> bool {\n                        let ecx = rd32(&self.p[107], 0xa4c);\n                        self.log.push(format!(\"s16:{ecx},{u},{p},0\"));\n                        self.seq(16) != 0\n                    }\n                    fn count_inside(&mut self, p: i32, u: i32, kind: i32, ty: i32) -> i32 {\n                        self.rec(17, &format!(\"p{}+0x0\", uw(p, u)), &[kind, ty]);\n                        self.seq(17)\n                    }\n                    fn add_local(&mut self, u: i32, p: i32) {\n                        self.rec(15, \"sp-0xa20\", &[u, p, 0, 0]);\n                    }\n                    fn unit_order_kind(&mut self, p: i32, u: i32) -> Option<i32> {\n                        self.rec(18, &format!(\"p{}+0x0\", uw(p, u)), &[]);\n                        let n = self.turn[18].min(3);\n                        self.turn[18] += 1;\n                        if self.a[NULL_AT + n] != 0 {\n                            return None;\n                        }\n                        self.rec(1, &format!(\"p{}+0x0\", 110 + n), &[]);\n                        Some(self.seq(1))\n                    }\n                    fn obj_next(&self, p: i32, o: i32) -> (i32, i32) {\n                        let w = &self.p[ow(p, o)];\n                        (rd16(w, 0x2c), rd16(w, 0x2e))\n                    }\n                    fn obj_slot_c(&mut self, p: i32, o: i32) -> bool {\n                        self.rec(7, &format!(\"p{}+0x0\", ow(p, o)), &[]);\n                        self.seq(7) != 0\n                    }\n                    fn obj_slot_4c(&mut self, p: i32, o: i32) -> bool {\n                        self.rec(8, &format!(\"p{}+0x0\", ow(p, o)), &[]);\n                        self.seq(8) != 0\n                    }\n                    fn obj_data_slot_90(&mut self, p: i32, o: i32) -> bool {\n                        self.rec(9, &format!(\"p{}+0x0\", ow(p, o)), &[]);\n                        self.rec(10, &format!(\"p{}+0x0\", 91 + k(p, o)), &[]);\n                        self.seq(10) != 0\n                    }\n                    fn unit_clear_80(&mut self, p: i32, u: i32) {\n                        self.p[uw(p, u)][0x80..0x82].copy_from_slice(&[0xff, 0xff]);\n                    }\n                    fn add_cast_order(&mut self, p: i32, u: i32, x: i32, y: i32, queue: i32) {\n                        self.rec(20, &format!(\"p{}+0x0\", uw(p, u)), &[-1, -1, x, y, 0x293, queue, 1]);\n                    }\n                    fn add_gather_order(&mut self, p: i32, u: i32, target: i32, queue: i32) {\n                        self.rec(21, &format!(\"p{}+0x0\", uw(p, u)), &[target, queue, 1]);\n                    }\n                    fn semaphore(&self) -> bool {\n                        self.c[4][0x821] & 8 != 0\n                    }\n                    fn obj_link28(&self, p: i32, o: i32) -> i32 {\n                        rd16(&self.p[ow(p, o)], 0x28)\n                    }\n                    fn can_carry_ground(&mut self, p: i32, u: i32) -> bool {\n                        self.rec(22, &format!(\"p{}+0x0\", uw(p, u)), &[0]);\n                        self.seq(22) != 0\n                    }\n                    fn unit_carrier(&self, p: i32, u: i32) -> (i32, i32) {\n                        let w = &self.p[uw(p, u)];\n                        (i32::from(w[0x3e] as i8), rd16(w, 0x28))\n                    }\n                    fn same_damage(&mut self, owner: i32, cont: i32, u: i32, p: i32) {\n                        self.rec(23, &format!(\"p{}+0x0\", uw(owner, cont)), &[u, p]);\n                    }\n                    fn clear_orders(&mut self, p: i32, u: i32) {\n                        self.rec(24, &format!(\"p{}+0x0\", uw(p, u)), &[]);\n                    }\n                    fn go_inside(&mut self, p: i32, u: i32, target: i32, tp: i32) {\n                        self.rec(25, &format!(\"p{}+0x0\", uw(p, u)), &[target, tp, 0]);\n                    }\n                    fn die(&mut self, p: i32, u: i32) {\n                        if self.a[FLAG_158] != 0 {\n                            self.rec(26, &format!(\"p{}+0x0\", uw(p, u)), &[0, -1, 0]);\n                        } else {\n                            self.rec(11, &format!(\"p{}+0x0\", uw(p, u)), &[]);\n                        }\n                    }\n                    fn check_gatherers(&mut self, p: i32, o: i32) {\n                        self.rec(4, &format!(\"p{}+0x0\", ow(p, o)), &[]);\n                        self.rec(27, &format!(\"p{}+0x0\", 83 + k(p, o)), &[]);\n                    }\n                    fn group_slot10(&mut self, _u: i32, _p: i32) {\n                        self.rec(1, \"m+0x0\", &[]);\n                        self.seq(1);\n                    }\n                    fn get_num(&mut self) -> i32 {\n                        self.rec(28, \"sp-0xa20\", &[]);\n                        self.seq(28)\n                    }\n                    fn move_near(&mut self, x: i32, y: i32) {\n                        self.rec(29, \"sp-0xa20\", &[x, y, 192, 2, 0, 0, 1, 0, -1, -1, 0]);\n                    }\n                }\n                let mut v = V {\n                    a,\n                    m: &mut *m,\n                    c: &*c,\n                    p: &mut *p,\n                    log: &mut log,\n                    turn: [0; 30],\n                    deref: [Vec::new(), Vec::new()],\n                };\n                action_gather_steps(&mut v, a[0], a[1]);\n                let [d0, d1] = std::mem::take(&mut v.deref);\n                let tail = |d: Vec<String>| {\n                    if d.is_empty() {\n                        \"-\".to_string()\n                    } else {\n                        d.join(\"/\")\n                    }\n                };\n                v.log.push(tail(d0));\n                v.log.push(tail(d1));\n                0\n            }"]
