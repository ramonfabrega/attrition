#!/usr/bin/env python3
"""funcs.py <INDEX.tsv> <out.funcs> — the function entries for the coverage arm.

Reads the Ghidra export's index (`addr<TAB>name<TAB>file`, one row per
function, virtual addresses at image base 0x400000) and writes the RVAs as
little-endian u32s, sorted, for rontrace.dll to plant its int3s on. Only
`.text` entries are kept (the DLL bounds-checks again).
"""
import struct
import sys

BASE = 0x400000
TEXT = (0x1000, 0x1000 + 0x6C4000)

index, out = sys.argv[1], sys.argv[2]
rvas = set()
for line in open(index):
    addr = int(line.split("\t", 1)[0], 16) - BASE
    if TEXT[0] <= addr < TEXT[1]:
        rvas.add(addr)
with open(out, "wb") as f:
    for r in sorted(rvas):
        f.write(struct.pack("<I", r))
print(f"{len(rvas)} function entries -> {out}")
