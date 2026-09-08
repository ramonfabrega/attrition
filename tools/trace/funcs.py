#!/usr/bin/env -S uv run --script
# /// script
# dependencies = ["capstone>=5"]
# ///
"""funcs.py <INDEX.tsv> <riseofnations.exe> <out.funcs> — the coverage table.

Reads the Ghidra export's index (`addr<TAB>name<TAB>file`, one row per
function, virtual addresses at image base 0x400000) and the executable's own
`.text`, and writes one 52-byte record per `.text` entry, sorted by RVA:

    u32 rva
    u8  orig_len        bytes the stub's copy stands in for (0 = excluded)
    u8  code_len        bytes of `code` (the copy, branches rewritten)
    u8  nfix            rel32 fixups in the copy
    u8  pad
    u8  code[24]        the displaced prologue: the smallest whole number of
                        instructions covering the five bytes a `jmp rel32`
                        overwrites, with every EIP-relative branch rewritten
                        to its rel32 form (jcc rel8 -> 0F 8x rel32, jmp rel8
                        -> E9 rel32) and its displacement left for the DLL
    u8  fix_off[4]      offset of each rel32 slot inside `code`
    u32 fix_target[4]   the branch's absolute target (VA)

rontrace.dll builds one stub per record: record the entry, run the copy,
jump back to `rva + orig_len`. **The original's bytes are written once, at
attach, and never again** — a write to translated code while another thread
is mid-syscall breaks that thread's mode switch under free Wine on Apple
Silicon (`docs/ORACLE.md`, "Coverage is back"), so nothing is restored.

An entry is excluded (orig_len 0, but kept on record) when the next listed
entry is under five bytes away (the jmp would overwrite it), when the
prologue cannot be decoded, when a branch in it targets the displaced range
itself, when the copy would not fit, when a direct branch anywhere in
`.text` lands strictly inside the displaced range (a linear sweep finds
them — a jump into the middle of the jmp's bytes would run its
displacement; the first run without this check died on exactly that, at a
label Ghidra had made a function of), or when the entry is one of Ghidra's
unnamed `FUN_` chunks, which are mid-block labels rather than functions.
The reasons are counted on stdout and listed by name in
`<out.funcs>.excluded.txt` beside the table — the exact list of what
coverage does not see.
"""
import bisect
import struct
import sys

import capstone

BASE = 0x400000
TEXT = (0x1000, 0x1000 + 0x6C4000)
MAX_ORIG = 16
MAX_CODE = 24
MAX_FIX = 4
RECORD = struct.Struct("<IBBBB24s4B4I")

index, exe, out = sys.argv[1], sys.argv[2], sys.argv[3]

names = {}
for line in open(index):
    parts = line.rstrip("\n").split("\t")
    rva = int(parts[0], 16) - BASE
    if TEXT[0] <= rva < TEXT[1]:
        names.setdefault(rva, parts[1])
rvas = sorted(names)

# the executable's .text, by its own section table
pe = open(exe, "rb").read()
e_lfanew = struct.unpack_from("<I", pe, 0x3C)[0]
nsec, opt_size = struct.unpack_from("<HxxxxxxxxxxxxH", pe, e_lfanew + 6)[0:2]
sec0 = e_lfanew + 24 + opt_size
text = None
for i in range(nsec):
    name, vsize, vaddr, rawsize, rawptr = struct.unpack_from("<8sIIII", pe, sec0 + 40 * i)
    if name.rstrip(b"\0") == b".text":
        text = (vaddr, pe[rawptr:rawptr + rawsize])
assert text and text[0] == TEXT[0], "the executable's .text is not at RVA 0x1000"
text_bytes = text[1]

md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
md.detail = True

# every direct branch target in .text, by linear sweep (jump tables are
# data and decode as nonsense, which only adds targets — conservative)
sweep = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
sweep.detail = True
targets = set()
pos = 0
va0 = BASE + TEXT[0]
while pos < len(text_bytes):
    n = 0
    for insn in sweep.disasm(text_bytes[pos:pos + (1 << 16)], va0 + pos):
        n += insn.size
        if insn.group(capstone.CS_GRP_BRANCH_RELATIVE):
            targets.add(insn.operands[0].imm)
    pos += n if n else 1  # an undecodable byte: step over it
targets = sorted(targets)

# jcc rel8 opcodes 0x70..0x7F widen to 0F 80..8F with the same low nibble
JCC_SHORT = range(0x70, 0x80)


def displace(rva):
    """(orig_len, code, fixups) for one entry, or (0, reason)."""
    off = rva - TEXT[0]
    raw = text_bytes[off:off + MAX_ORIG + 15]
    insns = []
    length = 0
    for insn in md.disasm(raw, BASE + rva):
        insns.append(insn)
        length += insn.size
        if length >= 5:
            break
    if length < 5:
        return 0, "undecodable prologue"
    if length > MAX_ORIG:
        return 0, "prologue over 16 bytes"
    code = bytearray()
    fixups = []
    for insn in insns:
        if not insn.group(capstone.CS_GRP_BRANCH_RELATIVE):
            code += insn.bytes
            continue
        target = insn.operands[0].imm
        if BASE + rva <= target < BASE + rva + length:
            return 0, "branch into the displaced range"
        op = insn.bytes[0]
        if op == 0xE9 or op == 0xE8:  # jmp / call rel32
            code += bytes([op])
        elif op == 0xEB:  # jmp rel8
            code += b"\xE9"
        elif op in JCC_SHORT:  # jcc rel8
            code += bytes([0x0F, 0x80 | (op & 0x0F)])
        elif op == 0x0F and 0x80 <= insn.bytes[1] <= 0x8F:  # jcc rel32
            code += insn.bytes[:2]
        else:  # loop, jecxz, ...
            return 0, f"unrelocatable branch ({insn.mnemonic})"
        if len(fixups) == MAX_FIX:
            return 0, "too many branches"
        fixups.append((len(code), target))
        code += b"\0\0\0\0"
    if len(code) > MAX_CODE:
        return 0, "copy over 24 bytes"
    return length, bytes(code), fixups


reasons = {}
excluded = []
records = []
for i, rva in enumerate(rvas):
    gap = rvas[i + 1] - rva if i + 1 < len(rvas) else 1 << 30
    if names[rva].startswith("FUN_"):
        result = (0, "unnamed chunk (Ghidra FUN_)")
    elif gap < 5:
        result = (0, "next entry under 5 bytes away")
    elif rva + 5 > TEXT[1]:
        result = (0, "at the end of .text")
    else:
        result = displace(rva)
    if result[0]:
        lo = bisect.bisect_right(targets, BASE + rva)
        if lo < len(targets) and targets[lo] < BASE + rva + result[0]:
            result = (0, "branch target inside the displaced range")
    if result[0] == 0:
        reason = result[1]
        reasons[reason] = reasons.get(reason, 0) + 1
        excluded.append((rva, names[rva], reason, gap))
        records.append(RECORD.pack(rva, 0, 0, 0, 0, b"", 0, 0, 0, 0, 0, 0, 0, 0))
    else:
        orig_len, code, fixups = result
        offs = [f[0] for f in fixups] + [0] * (MAX_FIX - len(fixups))
        tgts = [f[1] for f in fixups] + [0] * (MAX_FIX - len(fixups))
        records.append(RECORD.pack(rva, orig_len, len(code), len(fixups), 0, code, *offs, *tgts))

with open(out, "wb") as f:
    for r in records:
        f.write(r)
with open(out + ".excluded.txt", "w") as f:
    for rva, name, reason, gap in excluded:
        f.write(f"{BASE + rva:08x}\t{name}\t{reason}\tgap {gap}\n")
print(f"{len(records)} function entries -> {out}; {len(records) - len(excluded)} displaceable, "
      f"{len(excluded)} excluded: " + ", ".join(f"{v} {k}" for k, v in sorted(reasons.items())))
