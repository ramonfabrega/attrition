"""hooks.py — a function of the original run with every callee answered.

`callfn.py` enters a pure function. A game function calls a dozen others and
reads objects through vtables; items 976, 1009 and 1019 each built the same
scaffold by hand to run one under unicorn (`docs/PRODUCTION.md`, "The launch
commands" and "The Helicopter and the missile under a point"), and the third
reach graduates it here:

- **direct callees** are hooked by address: on entry the hook records
  `(name, ecx, args)`, sets `eax` from an answer (a value, or a function of
  `ecx` and the arguments), pops the return address and `4 × nargs` bytes —
  the callee's own `ret imm` — and returns. `nargs` is read off the listing;
  `retimm()` reads it for you.
- **vtable slots** are stubs in a page of their own: `vtable({off: (name,
  nargs, answer)})` lays out a fake vtable whose named slots are hooked the
  same way, so an object's `(*this + off)()` is answered and recorded.
- **objects** are laid out by hand in a bump heap (`alloc`, `w8/w16/w32`) at
  the offsets `types.txt` names; page 0 is mapped for the SEH chain's `fs:0`.

A read of unmapped memory still faults and names the address: that is the
next field to lay out. Nothing here is a source; it is a reading instrument,
the way the decompiler is (`CLAUDE.md`).
"""
import bisect
import re
import struct
import subprocess

from callfn import Image, Machine
from unicorn import UC_HOOK_CODE
from unicorn.x86_const import UC_X86_REG_EAX, UC_X86_REG_ECX, UC_X86_REG_EIP, UC_X86_REG_ESP

HEAP = 0x2000_0000
STUBS = 0x3000_0000


class Harness:
    def __init__(self, exe, callees):
        """`callees`: `{address: (name, nargs)}`, each hooked on entry."""
        self.m = Machine(Image(exe))
        uc = self.m.uc
        uc.mem_map(0, 0x1000)  # fs:0 (the SEH chain) reads linear 0
        uc.mem_map(HEAP, 0x10_0000)
        uc.mem_map(STUBS, 0x1000)
        self.brk = HEAP
        self.calls = []
        self.answers = {}
        self.hooked = dict(callees)
        self.next_stub = STUBS
        self.marks = set()
        uc.hook_add(UC_HOOK_CODE, self._on_code, begin=STUBS, end=STUBS + 0xFFF)
        for a in callees:
            uc.hook_add(UC_HOOK_CODE, self._on_code, begin=a, end=a)

    def mark(self, addr):
        """Record whether execution reaches `addr` (`addr in h.marks`)."""
        self.m.uc.hook_add(UC_HOOK_CODE, lambda uc, a, s, _: self.marks.add(a), begin=addr, end=addr)

    def alloc(self, n):
        p = self.brk
        self.brk += (n + 15) & ~15
        return p

    def w32(self, a, v):
        self.m.uc.mem_write(a, struct.pack("<i" if v < 0 else "<I", v))

    def w16(self, a, v):
        self.m.uc.mem_write(a, struct.pack("<h", v))

    def w8(self, a, v):
        self.m.uc.mem_write(a, bytes([v & 0xFF]))

    def r32(self, a):
        return struct.unpack("<i", self.m.uc.mem_read(a, 4))[0]

    def stub(self, name, nargs, answer):
        a = self.next_stub
        self.next_stub += 0x10
        self.hooked[a] = (name, nargs)
        self.answers[name] = answer
        return a

    def vtable(self, slots):
        """A fake vtable of 0x200 bytes, each named slot a stub:
        `{offset: (name, nargs, answer)}`."""
        v = self.alloc(0x200)
        for off, (name, nargs, ans) in slots.items():
            self.w32(v + off, self.stub(name, nargs, ans))
        return v

    def _on_code(self, uc, addr, size, _):
        if addr not in self.hooked:
            return
        name, nargs = self.hooked[addr]
        esp = uc.reg_read(UC_X86_REG_ESP)
        ret = struct.unpack("<I", uc.mem_read(esp, 4))[0]
        args = list(struct.unpack(f"<{nargs}i", uc.mem_read(esp + 4, 4 * nargs))) if nargs else []
        ecx = uc.reg_read(UC_X86_REG_ECX)
        ans = self.answers.get(name, 0)
        eax = ans(ecx, args) if callable(ans) else ans
        self.calls.append((name, ecx, args, eax))
        uc.reg_write(UC_X86_REG_EAX, eax & 0xFFFF_FFFF)
        uc.reg_write(UC_X86_REG_ESP, esp + 4 + 4 * nargs)
        uc.reg_write(UC_X86_REG_EIP, ret)

    def call(self, entry, stack_args=(), ecx=0, max_insns=200_000):
        eax = self.m.call(entry, stack_args, ecx=ecx, max_insns=max_insns)
        return eax - (1 << 32) if eax & 0x8000_0000 else eax


def retimm(exe, index_tsv, addrs, objdump="llvm-objdump"):
    """`{address: sorted set of the function's ret spellings}`, read off the
    listing between the address and the next function `INDEX.tsv` names."""
    starts = []
    for line in open(index_tsv):
        p = line.split("\t")
        try:
            starts.append(int(p[0], 16))
        except ValueError:
            pass
    starts.sort()
    out = {}
    for a in addrs:
        nxt = starts[bisect.bisect_right(starts, a)]
        text = subprocess.run(
            [objdump, "-d", "--no-show-raw-insn", f"--start-address={a:#x}", f"--stop-address={nxt:#x}", exe],
            capture_output=True, text=True, check=True).stdout
        out[a] = sorted(set(re.findall(r"retl(?:\s+\$0x[0-9a-f]+)?", text)))
    return out
