#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4", "capstone==5.0.3"]
# ///
"""checksum.py — the original's fifteen checksum walks on a packet, with the fold hooked.

    uv run tools/recomp/checksum.py <exe> <frame-snapshot.bin>
        [--gamelog FILE --record N] [--stream OUT.tsv] [--ledger OUT.tsv] [--values OUT.tsv]
        [--types types.txt] [--index INDEX.tsv]

Item 4060 (`docs/CHECKSUM.md`). `CheckSums::check_all@00936560` is entered
on the packet under unicorn, exactly as `step4.py` enters a function: the
capturing thread's TEB as `fs:`, a fresh stack, the clock/heap imports
stubbed. Its sync-logger calls (`SyncLogger::logToMemory`, two template
instances) are answered here and end a line: each one's `%u` argument is
the line's `CheckSum.accum`, as the original computed it. The fold,
`CheckSum::walk_function@00936ff0`, is hooked at its entry and left to run:
every byte range a walk hands over is recorded — the line, the walker that
owns it (the innermost live `*::walk_data` frame and its `this`), the
call site, the address, the length — and its bytes are read before the
original's own `adler32@00a46830` folds them. This tool's own Adler-32 over
that stream is a second sum, checked against the original's per line.

With `--gamelog` and `--record N`, the printed `CHECKSUM N` record is read
from the log and the fifteen lines are held against it, line for line;
a line that parts is printed with both values. The exit is 0 only when all
fifteen agree (or when no record was given).

`--stream` writes every range with its bytes in hex (the original's memory:
keep it outside git, under the item's lab-experiments directory).
`--ledger` writes the ranges grouped per line by (owner, offset in owner,
length): the count, the bytes, and — with `--types` (the decompile export's
`types.txt`) — the fields of the owner's type record the range covers,
pointers and floats marked. Nothing here is atomic across threads (the
packet says so), and nothing read from a packet enters the repository.
"""
import bisect
import os
import struct
import sys
import time
import zlib

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "emu"))
sys.path.insert(0, HERE)
import callfn  # noqa: E402
from difftest import EmuMachine, load_into_unicorn  # noqa: E402
from image import Image  # noqa: E402
from snapshot import Snapshot  # noqa: E402
from step4 import FRAME_STACK, FRAME_STACK_SIZE, SENTINEL, Stubs, find_teb, set_fs  # noqa: E402

CHECK_ALL = 0x00936560
WALK_FUNCTION = 0x00936FF0
FOLD_ADLER = 0x00A46830     # the fold's Adler-32 (ecx seed, edx buffer, pushed length)
FOLD_RETURN = 0x0093700F    # walk_function's return from it
ZLIB_ADLER = 0x005089D0     # zlib's adler32(seed, buffer, length), cdecl
LOG_NO_VALUE = 0x004634D0   # SyncLogger::logToMemory<>, `retn 0x10`: file, line, category, format
LOG_VALUE = 0x00463930      # SyncLogger::logToMemory<unsigned long &>, `retn 0x14`: … , &value

# check_all's line labels (its `L"  units: %u"` formats) and the printed
# record's labels for the same subsystem (`GameLog::say_checksum@00930b30`).
# The record's `World` line for the whole world is its sixteenth `World`.
LINES = [
    ("units", "Units"), ("builds", "Builds"), ("walls", "Walls"), ("ammo", "Ammo"),
    ("deaths", "Deaths"), ("groups", "Groups"), ("guys", "Guys"), ("leaders", "Leaders"),
    ("cities", "Cities"), ("items", "Items"), ("goods", "Goods"), ("world", "World"),
    ("rules", "Rules"), ("scenario_data", "scenario_data"), ("script_run_time", "script_run_time"),
]
STATE = {"units", "builds", "walls", "ammo", "deaths", "groups", "guys", "leaders", "cities", "items",
         "goods", "world"}


def adler(chunks):
    a = 1
    for c in chunks:
        a = zlib.adler32(c, a)
    return a


def read_record(path, number):
    """The printed `CHECKSUM number` record: {label: value}, the sixteen `World`
    lines as `World[0]`..`World[15]` (the last is the whole world)."""
    want = f"CHECKSUM {number}"
    rec, world, inside = {}, [], False
    with open(path, "rb") as f:
        for raw in f:
            line = raw.decode("latin-1").strip()
            if not inside:
                if line == want:
                    inside = True
                continue
            if line.startswith("FILE ") or line.startswith("LINE "):
                k, v = line.split(" ", 1)
                rec[k] = v
                continue
            if line.startswith("BEGIN ") or line.startswith("END ") or line.startswith("CHECKSUM "):
                break
            label, _, value = line.rpartition(" ")
            if not value.isdigit():
                break
            if label == "World":
                rec[f"World[{len(world)}]"] = int(value)
                world.append(int(value))
            else:
                rec[label] = int(value)
    if not inside:
        raise SystemExit(f"{path}: no `{want}` record")
    if world:
        rec["World"] = world[-1]
    return rec


class Index:
    """The decompile export's INDEX.tsv: entry VA -> demangled name."""

    def __init__(self, path):
        self.va, self.names = [], []
        if path and os.path.exists(path):
            for row in open(path, encoding="latin-1"):
                p = row.rstrip("\n").split("\t")
                if len(p) >= 2:
                    try:
                        self.va.append(int(p[0], 16))
                        self.names.append(p[1])
                    except ValueError:
                        pass
        order = sorted(range(len(self.va)), key=self.va.__getitem__)
        self.va = [self.va[i] for i in order]
        self.names = [self.names[i] for i in order]

    def of(self, va):
        i = bisect.bisect_right(self.va, va) - 1
        return (self.va[i], self.names[i]) if i >= 0 else (0, "?")


class Types:
    """`types.txt`: struct name -> [(offset, type, name)], size. A base class
    folds into its derived record as `_padding_` words, so a range is mapped
    against every record that names a field there (see `fields`)."""

    def __init__(self, path):
        self.structs = {}
        cur = None
        if not path or not os.path.exists(path):
            return
        for row in open(path, encoding="latin-1"):
            if row.startswith("struct /rise.pdb/"):
                head = row[len("struct /rise.pdb/"):].rstrip()
                name, _, size = head.partition("  // size ")
                cur = (name.strip(), [])
                try:
                    self.structs[cur[0]] = (int(size, 16), cur[1])
                except ValueError:
                    cur = None
                continue
            if cur and row.startswith("  +0x"):
                off = int(row[3:12].split()[0], 16)
                rest = row[12:].rstrip("\n")
                ty, name = rest[:41].strip(), rest[41:].strip()
                cur[1].append((off, ty, name))

    def fields(self, names, lo, hi):
        """The named fields of the first of `names` that has any in [lo, hi)."""
        for n in names:
            if n not in self.structs:
                continue
            size, fs = self.structs[n]
            hit = [(o, t, f) for o, t, f in fs if lo <= o < hi and f != "_padding_"]
            if hit:
                return n, hit
        return None, []


def kind(ty):
    t = ty.replace(" ", "")
    if "*" in t:
        return "ptr"
    if t in ("float", "double") or t.startswith("Vector<float") or t.startswith("Vector3<float"):
        return "float"
    return ""


def main(argv):
    import argparse
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("exe")
    ap.add_argument("snapshot")
    ap.add_argument("--gamelog")
    ap.add_argument("--record", type=int)
    ap.add_argument("--stream")
    ap.add_argument("--ledger")
    ap.add_argument("--values")
    home = os.path.expanduser("~/ghidra-projects/decomp")
    ap.add_argument("--types", default=os.path.join(home, "types.txt"))
    ap.add_argument("--index", default=os.path.join(home, "INDEX.tsv"))
    ap.add_argument("--max-insns", type=int, default=2_000_000_000)
    a = ap.parse_args(argv[1:])

    from unicorn import UC_HOOK_CODE, UcError
    from unicorn.x86_const import UC_X86_REG_ECX, UC_X86_REG_EIP, UC_X86_REG_ESP

    pe = Image(a.exe)
    image = callfn.Image(a.exe)
    snap = Snapshot(a.snapshot)
    index = Index(a.index)
    m = EmuMachine(image, pe.imports())
    skipped = load_into_unicorn(m, image, snap)
    m.patch_iat()
    uc = m.uc
    stubs = Stubs(uc)
    (teb, _pid, _tid), _ = find_teb(snap, snap.thread)
    if teb is None:
        raise SystemExit("no TEB in the packet")
    set_fs(uc, teb)
    for base, size, why in skipped:
        print(f"  unicorn could not map {base:08x}+{size:x}: {why}")
    uc.mem_map(FRAME_STACK, FRAME_STACK_SIZE)
    print(f"packet: logger frame {snap.frame}, trace frame {snap.trace_frame}, game {snap.game:08x}")

    def u32(va):
        return struct.unpack("<I", bytes(uc.mem_read(va, 4)))[0]

    state = dict(stop=None, line=0)
    ranges = []      # (line, seq, owner, this, site, start, n, bytes)
    values = {}      # line label -> the original's accum
    owners = []      # the live *::walk_data frames: (esp at entry, entry va, this)

    def on_stub(uc_, address, size, _):
        name = m.stubs.get(address, "?")
        if name == "strrchr":  # cdecl (s, c): the last c in s, or NULL
            s, c = stubs.arg(0), stubs.arg(1) & 0xFF
            last, p = 0, s
            while True:
                ch = uc_.mem_read(p, 1)[0]
                if ch == c:
                    last = p
                if ch == 0:
                    break
                p += 1
            stubs.ret(0, last)
            return
        if not stubs.handle(name):
            state["stop"] = f"import not stubbed: {name}"
            uc_.emu_stop()

    uc.hook_del(m.stub_hook)
    from difftest import STUBS
    uc.hook_add(UC_HOOK_CODE, on_stub, begin=STUBS, end=STUBS + 0xFFF)

    def wide(va):
        out = []
        while True:
            ch = struct.unpack("<H", bytes(uc.mem_read(va, 2)))[0]
            if ch == 0:
                return "".join(out)
            out.append(chr(ch))
            va += 2

    def on_log(uc_, address, size, _):
        esp = uc_.reg_read(UC_X86_REG_ESP)
        fmt = wide(u32(esp + 16))
        if address == LOG_VALUE:
            label = fmt.strip().split(":")[0]
            values[label] = u32(u32(esp + 20))
            state["line"] += 1
        stubs.ret(0x14 if address == LOG_VALUE else 0x10)

    uc.hook_add(UC_HOOK_CODE, on_log, begin=LOG_NO_VALUE, end=LOG_NO_VALUE)
    uc.hook_add(UC_HOOK_CODE, on_log, begin=LOG_VALUE, end=LOG_VALUE)

    def on_walker(uc_, address, size, _):
        esp = uc_.reg_read(UC_X86_REG_ESP)
        while owners and owners[-1][0] <= esp:
            owners.pop()
        owners.append((esp, address, uc_.reg_read(UC_X86_REG_ECX)))

    walkers = [va for va, n in zip(index.va, index.names)
               if (n.endswith("::walk_data") or n.endswith("::walk_rules_data")) and va != WALK_FUNCTION]
    for va in walkers:
        uc.hook_add(UC_HOOK_CODE, on_walker, begin=va, end=va)

    def on_fold(uc_, address, size, _):
        esp = uc_.reg_read(UC_X86_REG_ESP)
        site, start, end = u32(esp), u32(esp + 4), u32(esp + 8)
        while owners and owners[-1][0] <= esp:
            owners.pop()
        owner, this = (owners[-1][1], owners[-1][2]) if owners else (index.of(site)[0], 0)
        n = (end - start) & 0xFFFFFFFF
        data = bytes(uc_.mem_read(start, n)) if 0 < n < 0x4000_0000 else b""
        ranges.append((state["line"], len(ranges), owner, this, site, start, n, data))

    uc.hook_add(UC_HOOK_CODE, on_fold, begin=WALK_FUNCTION, end=WALK_FUNCTION)

    # A walk can also fold bytes past the callback: `check_groups` runs the
    # zlib `adler32@005089d0` (cdecl: seed, buffer, length) over eight words
    # it copies to a stack local. Every Adler entry not reached from the
    # fold is a range of its own, owned by its caller.
    def on_direct(uc_, address, size, _):
        esp = uc_.reg_read(UC_X86_REG_ESP)
        site = u32(esp)
        if site == FOLD_RETURN:
            return
        if address == ZLIB_ADLER:
            start, n = u32(esp + 8), u32(esp + 12)
        else:  # the fold's own Adler: ecx seed, edx buffer, one pushed length
            from unicorn.x86_const import UC_X86_REG_EDX
            start, n = uc_.reg_read(UC_X86_REG_EDX), u32(esp + 4)
        data = bytes(uc_.mem_read(start, n)) if 0 < n < 0x4000_0000 else b""
        owner = index.of(site)[0]
        ranges.append((state["line"], len(ranges), owner, 0, site, start, n, data))

    uc.hook_add(UC_HOOK_CODE, on_direct, begin=ZLIB_ADLER, end=ZLIB_ADLER)
    uc.hook_add(UC_HOOK_CODE, on_direct, begin=FOLD_ADLER, end=FOLD_ADLER)

    esp = FRAME_STACK + FRAME_STACK_SIZE - 0x100
    uc.mem_write(esp, struct.pack("<I", SENTINEL))
    uc.reg_write(UC_X86_REG_ESP, esp)
    uc.reg_write(UC_X86_REG_ECX, 0)
    t0 = time.perf_counter()
    try:
        uc.emu_start(CHECK_ALL, SENTINEL, count=a.max_insns)
        if state["stop"] is None and uc.reg_read(UC_X86_REG_EIP) != SENTINEL:
            state["stop"] = f"instruction cap at {uc.reg_read(UC_X86_REG_EIP):08x}"
    except UcError as e:
        f = m.fault
        state["stop"] = f"{e}" if f is None else f"{e}: access {f[0]} at {f[1]:08x} from eip {f[3]:08x}"
    took = time.perf_counter() - t0
    print(f"check_all: {'returned' if state['stop'] is None else 'STOPPED: ' + state['stop']} in {took:.1f}s; "
          f"{len(ranges)} ranges, {sum(r[6] for r in ranges)} bytes")

    record = read_record(a.gamelog, a.record) if a.gamelog and a.record is not None else None
    if record:
        print(f"record: CHECKSUM {a.record} (FILE {record.get('FILE')}, LINE {record.get('LINE')}) of {a.gamelog}")
    agree, ok_sum = 0, True
    print(f"{'line':<16} {'ranges':>8} {'bytes':>10} {'original':>11} {'stream':>11} {'printed':>11}  verdict")
    for i, (label, printed_label) in enumerate(LINES):
        rs = [r for r in ranges if r[0] == i]
        ours = adler(r[7] for r in rs)
        orig = values.get(label)
        if orig is not None and orig != ours:
            ok_sum = False
        printed = record.get(printed_label) if record else None
        if record:
            verdict = "agrees" if printed == orig else "PARTS"
            agree += printed == orig
        else:
            verdict = ""
        print(f"{label:<16} {len(rs):>8} {sum(r[6] for r in rs):>10} {orig!s:>11} {ours:>11} {printed!s:>11}  {verdict}"
              + ("" if orig == ours else "  (stream sum differs from the original's)"))
    total = sum(values.get(lb, 0) for lb, _ in LINES) & 0xFFFFFFFF
    print(f"total (wrapping sum of the fifteen): {total}")
    state_agree = sum(1 for i, (lb, pl) in enumerate(LINES) if lb in STATE and record and record.get(pl) == values.get(lb))
    if record:
        print(f"reproduced: {agree} of {len(LINES)} lines ({state_agree} of 12 state lines)")
    print(f"stream sums equal the original's on every line: {ok_sum}")

    if a.values:
        # The fifteen as the original summed them on this packet: what
        # `rondata::checksum`'s test holds the printed record against.
        with open(a.values, "w") as f:
            f.write(f"# packet logger frame {snap.frame}, trace frame {snap.trace_frame}\n")
            f.write("line\tlabel\tvalue\tranges\tbytes\n")
            for i, (label, printed_label) in enumerate(LINES):
                rs = [r for r in ranges if r[0] == i]
                f.write(f"{label}\t{printed_label}\t{values.get(label)}\t{len(rs)}\t{sum(r[6] for r in rs)}\n")
        print(f"values: {a.values}")
    if a.stream:
        with open(a.stream, "w") as f:
            f.write("line\tseq\towner\tthis\tsite\tstart\tlength\tbytes\n")
            for line, seq, owner, this, site, start, n, data in ranges:
                f.write(f"{LINES[line][0] if line < len(LINES) else line}\t{seq}\t{index.of(owner)[1]}\t{this:08x}\t"
                        f"{site:08x}\t{start:08x}\t{n}\t{data.hex()}\n")
        print(f"stream: {a.stream}")
    if a.ledger:
        types = Types(a.types)
        groups = {}
        for line, seq, owner, this, site, start, n, data in ranges:
            off = (start - this) & 0xFFFFFFFF if this and start >= this and start - this < 0x10000 else None
            key = (line, index.of(owner)[1], site, off, n if off is not None else None)
            g = groups.setdefault(key, [0, 0, start])
            g[0] += 1
            g[1] += n
        with open(a.ledger, "w") as f:
            f.write("line\towner\tsite\toffset\tlength\tcount\tbytes\tfirst_start\ttype\tfields\n")
            for (line, owner, site, off, n), (count, total_b, first) in sorted(groups.items(), key=lambda kv: (kv[0][0], -kv[1][1])):
                cls = owner.split("::")[0]
                tname, fs = (None, [])
                if off is not None and n:
                    cands = [cls, cls + "Data", cls.replace("Data", "")]
                    tname, fs = types.fields(cands, off, off + n)
                desc = "; ".join(f"+{o:#x} {t} {nm}" + (f" [{kind(t)}]" if kind(t) else "") for o, t, nm in fs)
                f.write(f"{LINES[line][0] if line < len(LINES) else line}\t{owner}\t{site:08x}\t"
                        f"{'' if off is None else hex(off)}\t{'' if n is None else n}\t{count}\t{total_b}\t"
                        f"{first:08x}\t{tname or ''}\t{desc}\n")
        print(f"ledger: {a.ledger}")
    if state["stop"] is not None:
        return 2
    if record and agree != len(LINES):
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
