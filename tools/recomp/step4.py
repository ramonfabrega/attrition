#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4", "capstone==5.0.3"]
# ///
"""step4.py — how far a frame runs on the snapshot, and whether what it ran lifts.

    uv run tools/recomp/step4.py <exe> <frame-snapshot.bin> [--max-insns N] [--out DIR]

The charter's step 4, measured with unicorn as the explorer: the lab's
end-frame snapshot loaded at its addresses, the capturing thread's TEB found
inside it (by its self-pointer) and made `fs:`, a fresh stack, every import
stubbed — the ones with a semantic here answer, the rest stop the run and
name themselves — and `Game::do_frame@00591ef0` entered on the game. The
report is what stopped the frame (an import, an unmapped address, the
instruction cap, or the frame's own return), how many instructions and
blocks ran, which functions were entered, and — the recompiler's own
question — how many of those functions lift and compile, with the
failures by kind. Nothing here is a fidelity claim: a stubbed import
answers what the stub says, not what Windows said.
"""
import bisect
import os
import struct
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "emu"))
sys.path.insert(0, HERE)
import callfn  # noqa: E402
from difftest import IMPORT_STUBS, STUBS, EmuMachine, load_into_unicorn  # noqa: E402
from image import Functions, Image  # noqa: E402
from snapshot import Snapshot  # noqa: E402

# the lifter (capstone) is imported where it is used, so difftest.py — which
# declares only unicorn — can import this module's guest helpers

DO_FRAME = 0x00591EF0
FRAME_STACK, FRAME_STACK_SIZE = 0x7D00_0000, 0x0010_0000
SENTINEL = callfn.SENTINEL


def find_teb(snap, thread):
    """A TEB points at itself at +0x18 and carries its thread id at +0x24."""
    hits = []
    for base, size, _ in snap.ranges:
        if base < 0x7000_0000:
            continue
        for p in range(base, base + size, 0x1000):
            try:
                if snap.u32(p + 0x18) == p:
                    hits.append((p, snap.u32(p + 0x20), snap.u32(p + 0x24)))
            except KeyError:
                pass
    mine = [h for h in hits if h[2] == thread]
    return (mine or hits or [(None, None, None)])[0], hits


GDT = 0x7C00_0000


def descriptor(base, access):
    """A 32-bit, 4 KiB-granular, present ring-0 GDT entry covering 4 GiB from `base`."""
    limit = 0xFFFFF
    lo = (limit & 0xFFFF) | ((base & 0xFFFF) << 16)
    hi = ((base >> 16) & 0xFF) | (access << 8) | (((limit >> 16) & 0xF) << 16) | (0xC << 20) | (base & 0xFF000000)
    return struct.pack("<II", lo, hi)


def set_fs(uc, base):
    """`fs` for a 32-bit guest: unicorn ignores `FS_BASE` here (its own
    warning says so), so a GDT — flat code and data for the segments the
    guest already runs in, since loading GDTR re-reads them all, and one
    data descriptor at `base` — with the selectors loaded."""
    from unicorn.x86_const import (UC_X86_REG_CS, UC_X86_REG_DS, UC_X86_REG_ES, UC_X86_REG_FS, UC_X86_REG_GDTR,
                                   UC_X86_REG_SS)
    uc.mem_map(GDT, 0x1000)
    uc.mem_write(GDT + 8 * 15, descriptor(base, 0x93))  # fs: the TEB
    uc.mem_write(GDT + 8 * 16, descriptor(0, 0x93))     # flat data
    uc.mem_write(GDT + 8 * 17, descriptor(0, 0x9B))     # flat code
    uc.reg_write(UC_X86_REG_GDTR, (0, GDT, 8 * 18 - 1, 0))
    for seg in (UC_X86_REG_DS, UC_X86_REG_ES, UC_X86_REG_SS):
        uc.reg_write(seg, 16 << 3)
    uc.reg_write(UC_X86_REG_CS, 17 << 3)
    uc.reg_write(UC_X86_REG_FS, 15 << 3)


class Stubs:
    """Import stubs that perform their own return: `eip = [esp]; esp += 4 +
    args`, `eax` the answer. `stdcall` names pop their arguments."""

    def __init__(self, uc):
        from unicorn.x86_const import UC_X86_REG_EAX, UC_X86_REG_EIP, UC_X86_REG_ESP
        self.uc, self.EAX, self.EIP, self.ESP = uc, UC_X86_REG_EAX, UC_X86_REG_EIP, UC_X86_REG_ESP
        self.ticks = 0x0100_0000
        self.heap = 0x6000_0000  # a bump allocator in a fresh region
        uc.mem_map(self.heap, 0x0400_0000)
        self.calls = {}

    def ret(self, args=0, eax=None):
        esp = self.uc.reg_read(self.ESP)
        self.uc.reg_write(self.EIP, struct.unpack("<I", self.uc.mem_read(esp, 4))[0])
        self.uc.reg_write(self.ESP, esp + 4 + args)
        if eax is not None:
            self.uc.reg_write(self.EAX, eax & 0xFFFF_FFFF)

    def arg(self, k):
        esp = self.uc.reg_read(self.ESP)
        return struct.unpack("<I", self.uc.mem_read(esp + 4 + 4 * k, 4))[0]

    def alloc(self, n):
        p = self.heap
        self.heap = (self.heap + max(n, 1) + 15) & ~15
        return p

    def handle(self, name):
        """True if handled; False if the run must stop here."""
        self.calls[name] = self.calls.get(name, 0) + 1
        if name in IMPORT_STUBS:  # the float imports difftest answers, the same way
            from unicorn.x86_const import UC_X86_REG_XMM0
            self.uc.reg_write(UC_X86_REG_XMM0, IMPORT_STUBS[name](self.uc.reg_read(UC_X86_REG_XMM0)))
            self.ret(0)
        elif name in ("GetTickCount", "timeGetTime"):
            self.ticks += 1
            self.ret(0, self.ticks)
        elif name == "QueryPerformanceCounter":
            self.ticks += 1000
            self.uc.mem_write(self.arg(0), struct.pack("<Q", self.ticks))
            self.ret(4, 1)
        elif name == "QueryPerformanceFrequency":
            self.uc.mem_write(self.arg(0), struct.pack("<Q", 10_000_000))
            self.ret(4, 1)
        elif name in ("malloc", "_aligned_malloc"):
            self.ret(0, self.alloc(self.arg(0)))
        elif name in ("free", "_aligned_free"):
            self.ret(0, 0)
        elif name == "calloc":
            n = self.arg(0) * self.arg(1)
            p = self.alloc(n)
            self.uc.mem_write(p, b"\0" * n)
            self.ret(0, p)
        elif name == "realloc":
            old, n = self.arg(0), self.arg(1)
            p = self.alloc(n)
            if old:
                self.uc.mem_write(p, bytes(self.uc.mem_read(old, n)))
            self.ret(0, p)
        elif name == "HeapAlloc":
            n = self.arg(2)
            p = self.alloc(n)
            if self.arg(1) & 8:
                self.uc.mem_write(p, b"\0" * n)
            self.ret(12, p)
        elif name == "HeapFree":
            self.ret(12, 1)
        elif name in ("EnterCriticalSection", "LeaveCriticalSection"):
            self.ret(4, 0)
        elif name == "GetLastError":
            self.ret(0, 0)
        elif name == "SetLastError":
            self.ret(4, 0)
        elif name == "GetCurrentThreadId":
            self.ret(0, 276)
        elif name == "OutputDebugStringA":
            self.ret(4, 0)
        else:
            return False
        return True


def main(argv):
    args = argv[1:]
    max_insns, out_dir = 20_000_000, os.path.join(HERE, "..", "..", "target", "recomp")
    entry, this, dump, watch, find = DO_FRAME, None, [], [], set()
    for flag in ("--max-insns", "--out", "--entry", "--this", "--dump", "--watch", "--find"):
        while flag in args:
            k = args.index(flag)
            v = args[k + 1]
            if flag == "--max-insns":
                max_insns = int(v)
            elif flag == "--out":
                out_dir = v
            elif flag == "--entry":
                entry = int(v, 16)
            elif flag == "--this":
                this = int(v, 16)
            elif flag == "--find":
                find.add(int(v, 0) & 0xFFFFFFFF)
            else:
                va, n = v.split("+")
                (dump if flag == "--dump" else watch).append((int(va, 16), int(n, 16)))
            del args[k:k + 2]
    if len(args) < 2:
        print(__doc__)
        return 2
    exe, snap_path = args
    from unicorn import UC_HOOK_BLOCK, UC_HOOK_CODE, UcError
    from unicorn.x86_const import UC_X86_REG_ECX, UC_X86_REG_EIP, UC_X86_REG_ESP

    pe = Image(exe)
    image = callfn.Image(exe)
    snap = Snapshot(snap_path)
    funcs = Functions(pe, os.path.dirname(os.path.abspath(exe)))
    m = EmuMachine(image, pe.imports())
    skipped = load_into_unicorn(m, image, snap)
    m.patch_iat()
    uc = m.uc
    stubs = Stubs(uc)
    (teb, pid, tid), tebs = find_teb(snap, snap.thread)
    print(f"snapshot: frame {snap.frame}, game {snap.game:08x}; TEBs found: {[(f'{p:08x}', t) for p, _, t in tebs]}")
    if teb is None:
        print("no TEB in the snapshot; fs: reads will fault")
    else:
        set_fs(uc, teb)
        print(f"fs: -> TEB {teb:08x} (thread {tid})")
    for base, size, why in skipped:
        print(f"  unicorn could not map {base:08x}+{size:x}: {why}")
    uc.mem_map(FRAME_STACK, FRAME_STACK_SIZE)

    state = dict(insns=0, blocks=set(), stop=None, entries={entry}, pending=False)

    def on_stub(uc, address, size, _):
        name = m.stubs.get(address, "?")
        if not stubs.handle(name):
            state["stop"] = f"import not stubbed: {name} (esp {uc.reg_read(UC_X86_REG_ESP):08x})"
            uc.emu_stop()

    uc.hook_del(m.stub_hook)  # difftest's stubs stop on anything but sqrt; these answer more
    uc.hook_add(UC_HOOK_CODE, on_stub, begin=STUBS, end=STUBS + 0xFFF)

    def on_block(uc, address, size, _):
        state["blocks"].add(address)
        if state["pending"]:  # the block after a call or an indirect jump is a function's entry
            state["entries"].add(address)
            state["pending"] = False

    from unicorn.x86_const import (UC_X86_REG_EAX, UC_X86_REG_EBX, UC_X86_REG_EDI, UC_X86_REG_EDX, UC_X86_REG_ESI)
    REGS = (("eax", UC_X86_REG_EAX), ("ecx", UC_X86_REG_ECX), ("edx", UC_X86_REG_EDX), ("ebx", UC_X86_REG_EBX),
            ("esi", UC_X86_REG_ESI), ("edi", UC_X86_REG_EDI))
    found, last = {}, [entry]

    def on_insn(uc, address, size, _):
        state["insns"] += 1
        b = uc.mem_read(address, 3)
        # call rel32 or call r/m32 (FF /2): the next block entered is a callee; so is the
        # target of an indirect jmp (FF /4) unless it is a jump table's — `[4*reg + disp32]`,
        # a SIB with scale 4 and no base — whose cases the lifter keeps inside the function
        pending = b[0] == 0xE8
        if b[0] == 0xFF and (b[1] >> 3) & 7 in (2, 4):
            modrm = b[1]
            table = (modrm >> 3) & 7 == 4 and modrm >> 6 == 0 and modrm & 7 == 4 and b[2] >> 6 == 2 and b[2] & 7 == 5
            pending = not table
        state["pending"] = pending
        if find:
            # the hook runs before `address` executes, so the registers are the previous instruction's result
            for name, reg in REGS:
                v = uc.reg_read(reg)
                if v in find and v not in found:
                    found[v] = (last[0], name, state["insns"])
            last[0] = address

    text_lo, text_hi = pe.text[1], pe.text[1] + pe.text[2]
    uc.hook_add(UC_HOOK_BLOCK, on_block, begin=text_lo, end=text_hi)
    uc.hook_add(UC_HOOK_CODE, on_insn, begin=text_lo, end=text_hi)

    writes = []

    def on_write(uc, access, address, size, value, _):
        # who wrote a watched address: the instruction, its function, the value
        eip = uc.reg_read(UC_X86_REG_EIP)
        writes.append((state["insns"], eip, address, size, value & 0xFFFFFFFF))

    if watch:
        from unicorn import UC_HOOK_MEM_WRITE
        for va, n in watch:
            uc.hook_add(UC_HOOK_MEM_WRITE, on_write, begin=va, end=va + n - 1)

    esp = FRAME_STACK + FRAME_STACK_SIZE - 0x100
    uc.mem_write(esp, struct.pack("<I", SENTINEL))
    uc.reg_write(UC_X86_REG_ESP, esp)
    uc.reg_write(UC_X86_REG_ECX, snap.game if this is None else this)
    before = [(va, n, bytes(uc.mem_read(va, n))) for va, n in dump]
    print(f"entering {funcs.name(entry)}@{entry:08x} with this={uc.reg_read(UC_X86_REG_ECX):08x}")
    t0 = time.perf_counter()
    try:
        uc.emu_start(entry, SENTINEL, count=max_insns)
        eip = uc.reg_read(UC_X86_REG_EIP)
        if state["stop"] is None:
            state["stop"] = "the frame returned" if eip == SENTINEL else f"instruction cap at eip {eip:08x}"
    except UcError as e:
        f = m.fault
        state["stop"] = f"{e}; fault {f}" if f is None else f"{e}: access {f[0]} at {f[1]:08x} from eip {f[3]:08x}"
    t1 = time.perf_counter()

    entered = sorted(e for e in state["entries"] if text_lo <= e < text_hi)
    print(f"stopped: {state['stop']}")
    print(f"ran: {state['insns']} instructions, {len(state['blocks'])} blocks, {len(entered)} functions entered, {t1 - t0:.2f}s")
    for v, (eip, name, i) in found.items():
        fn = funcs.sorted[bisect.bisect_right(funcs.sorted, eip) - 1]
        print(f"found {v} ({v:#x}) first in {name} after instruction #{i} at {eip:08x} {funcs.name(fn)}+{eip - fn:#x}")
    if writes:
        print(f"writes to the watched ranges: {len(writes)}")
        for i, eip, address, size, value in writes[:60]:
            fn = funcs.sorted[bisect.bisect_right(funcs.sorted, eip) - 1]
            print(f"  #{i:>8}  {eip:08x} {funcs.name(fn)}+{eip - fn:#x}: [{address:08x}] <- {value:#x} ({value if value < 0x80000000 else value - (1 << 32)}) size {size}")
    for va, n, old in before:
        new = bytes(uc.mem_read(va, n))
        print(f"dump {va:08x}+{n:x}: {'unchanged' if old == new else 'CHANGED'}")
        for off in range(0, n, 16):
            o, w = old[off:off + 16].hex(), new[off:off + 16].hex()
            print(f"  {va + off:08x}  {w}" + ("" if o == w else f"   (was {o})"))
    print("imports answered: " + ", ".join(f"{k}×{v}" for k, v in sorted(stubs.calls.items(), key=lambda kv: -kv[1])))

    from lift import LiftError, Lifter
    from scan import compile_one, kind

    os.makedirs(os.path.join(out_dir, "scan"), exist_ok=True)
    lifted, failed = {}, {}
    for e in entered:
        lifter = Lifter(pe, funcs)
        try:
            callees = lifter.lift(e)
            protos = "".join(f"void f_{c:08x}(cpu_t *c);\n" for c in sorted(callees | {e}))
            lifted[e] = '#include "recomp.h"\n#include <stdint.h>\n' + protos + lifter.lifted[e] + "\n"
        except LiftError as err:
            failed[e] = err
    import concurrent.futures
    from collections import Counter
    ok = 0
    with concurrent.futures.ThreadPoolExecutor(max_workers=os.cpu_count() or 4) as pool:
        for e, good, _ in pool.map(lambda e: compile_one(out_dir, e, lifted[e]), lifted):
            ok += good
    print(f"of the {len(entered)} functions the frame entered: lifted {len(lifted)}, lifted and compiled {ok}")
    for k, v in Counter(kind(err) for err in failed.values()).most_common():
        print(f"  {v:5d}  {k}")
    with open(os.path.join(out_dir, "step4-entered.txt"), "w") as f:
        for e in entered:
            f.write(f"{e:08x}\t{funcs.name(e)}\t{'lifts' if e in lifted else failed[e]}\n")
    print(f"rows: {os.path.join(out_dir, 'step4-entered.txt')}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
