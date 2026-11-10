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
                off = int(row[3:11].split()[0], 16)
                rest = row[11:].rstrip("\n")
                if len(rest) <= 41 or rest[40] == " ":
                    ty, name = rest[:41].strip(), rest[41:].strip()  # the type's column, then the name
                else:
                    parts = rest.rsplit(None, 1)  # a type wider than its column, a name or none
                    ty, name = (parts[0], parts[1]) if len(parts) == 2 else (rest.strip(), "")
                cur[1].append((off, ty, name))

    PRIMITIVE = {"char": 1, "uchar": 1, "bool": 1, "byte": 1, "undefined1": 1, "short": 2, "ushort": 2,
                 "wchar_t": 2, "int": 4, "uint": 4, "long": 4, "ulong": 4, "float": 4, "Coord": 4, "double": 8,
                 "longlong": 8, "ulonglong": 8, "__int64": 8}

    def size_of(self, ty):
        t = ty.strip()
        if "*" in t:
            return 4
        if t in self.PRIMITIVE:
            return self.PRIMITIVE[t]
        if t in self.structs:
            return self.structs[t][0]
        return None

    def fields(self, names, lo, hi):
        """The named fields of the first of `names` that has any in [lo, hi),
        each with its size (by its type, else to the next field), and the
        bytes of [lo, hi) no field covers: padding, or a base class's
        `_padding_` words."""
        for n in names:
            if n not in self.structs:
                continue
            size, fs = self.structs[n]
            named = [(o, t, f) for o, t, f in fs if f != "_padding_"]
            if not any(lo <= o < hi for o, _, _ in named):
                continue
            out, covered = [], set()
            for k, (o, t, f) in enumerate(named):
                if not lo <= o < hi:
                    continue
                nxt = named[k + 1][0] if k + 1 < len(named) else size
                w = self.size_of(t)
                w = nxt - o if w is None or w > nxt - o else w
                out.append((o, t, f, w))
                covered.update(range(o, o + w))
            runs = []
            for g in sorted(set(range(lo, hi)) - covered):
                if runs and runs[-1][1] == g:
                    runs[-1][1] = g + 1
                else:
                    runs.append([g, g + 1])
            return n, out, runs
        return None, [], []


# A walker whose ranges are elements of a buffer: the element's type record.
ELEMENTS = {
    "Stack<PathData>::walk_data": "PathData",
    "BuildQueue::walk_data": "QueueItem",
    "Array<TCoordData>::walk_data": "TCoordData",
    "Array<MakeObject>::walk_data": "MakeObject",
    "Array<Site>::walk_data": "Site",
    "Array<CaravanLink>::walk_data": "CaravanLink",
    "Array<WCoordData>::walk_data": "WCoordData",
    "SimpleArray<int>::walk_data": "int",
    "SimpleArray<WCoord>::walk_data": "WCoord",
}
# An order's walk is entered on its `UnitOrder` virtual base, which sits at
# the end of the derived record (the PDB's field lists: `TargetOrder`'s
# vbptr at +4 and the base at +0x2c of a 0x34 record; `MoveOrder`'s at +0
# and +0x54 of 0x5c): the record starts this many bytes before `ecx`.
VBASE = {"GatherOrder::walk_data": 0x2C, "TradeOrder::walk_data": 0x2C, "AttackToOrder::walk_data": 0x54}
# `World::walk_data@006b5cf0` reads its world from this global, not `ecx`.
WORLD_GLOBAL = 0x00C06188


def array_of(ty):
    """`T[a][b]` -> (T, a*b); anything else -> (ty, None)."""
    t = ty.strip()
    if "[" not in t:
        return t, None
    elem, _, dims = t.partition("[")
    n = 1
    for d in dims.rstrip("]").split("]["):
        n *= int(d)
    return elem.strip(), n


def leaves(types, sname, lo, hi, base=0, path="", depth=0):
    """The leaf fields of record `sname` (placed at `base`) that overlap
    [lo, hi): (offset, width, type, path, kind). A struct member or an
    array of structs is expanded; a record with two named fields at one
    offset is a union and is one leaf whose kind lists its members'."""
    if sname not in types.structs:
        return
    size, fs = types.structs[sname]
    named = [(o, t, f) for o, t, f in fs if f != "_padding_"]
    offsets = [o for o, _, _ in named]
    if len(set(offsets)) < len(offsets):  # a union
        kinds = sorted({kind(t) for _, t, _ in named} - {""})
        yield (base, size, sname, path.rstrip(".") or sname, "union(" + ",".join(kinds) + ")" if kinds else "union")
        return
    for k, (o, t, f) in enumerate(named):
        nxt = named[k + 1][0] if k + 1 < len(named) else size
        at = base + o
        if at >= hi or at + max(nxt - o, 1) <= lo:
            continue
        elem, n = array_of(t)
        inner = types.structs.get(elem)
        if inner and depth < 6 and any(x[2] != "_padding_" for x in inner[1]):
            esz = inner[0]
            for i in range(n or 1):
                eo = at + i * esz
                if eo < hi and eo + esz > lo:
                    yield from leaves(types, elem, lo, hi, eo, f"{path}{f}" + (f"[{i}]." if n else "."), depth + 1)
            continue
        w = types.size_of(elem)
        w = (w * (n or 1)) if w is not None else None
        w = nxt - o if w is None or w > nxt - o else w
        yield (at, w, t, path + f, kind(elem) if kind(t) == "" else kind(t))


def describe(types, sname, lo, hi):
    """A range's fields by the type record: the span's first and last leaf,
    the pointer, float and union leaves named, and the bytes no leaf
    covers (padding, or an unnamed base's words)."""
    ls = list(leaves(types, sname, lo, hi))
    covered = set()
    for at, w, _, _, _ in ls:
        covered.update(range(max(at, lo), min(at + w, hi)))
    gaps, runs = sorted(set(range(lo, hi)) - covered), []
    for g in gaps:
        if runs and runs[-1][1] == g:
            runs[-1][1] = g + 1
        else:
            runs.append([g, g + 1])
    flagged = [f"+{at:#x} {path} ({ty}) [{kd}]" for at, w, ty, path, kd in ls if kd]
    span = f"{ls[0][3]}..{ls[-1][3]} ({len(ls)} leaves)" if ls else ""
    return span, flagged, runs


def write_ledger(path, ranges, index, types, world):
    """The ranges per line, grouped by (owner, call site, offset in the
    owner's record, length): count, bytes, the record and its leaves."""
    groups = {}
    for line, seq, owner, this, site, start, n, data in ranges:
        name = index.of(owner)[1]
        local = FRAME_STACK <= start < FRAME_STACK + FRAME_STACK_SIZE
        if name == "World::walk_data":
            this = world
        base = this - VBASE[name] if name in VBASE and this else this
        off = (start - base) & 0xFFFFFFFF if base and start >= base and start - base < 0x10000 else None
        if local:
            where = "stack"
        elif off is not None:
            where = hex(off)
        elif name in ELEMENTS:
            where = "element"
        else:
            where = ""
        key = (line, name, site, where, n if where in ("stack", "element") or off is not None else None)
        g = groups.setdefault(key, [0, 0, start, off])
        g[0] += 1
        g[1] += n
    with open(path, "w") as f:
        f.write("line\towner\tsite\toffset\tlength\tcount\tbytes\tfirst_start\trecord\tspan\tflagged\tuncovered\n")
        for (line, owner, site, where, n), (count, total_b, first, off) in sorted(
                groups.items(), key=lambda kv: (kv[0][0], -kv[1][1])):
            cls = owner.split("::")[0]
            rec, span, flagged, gaps = "", "", [], []
            if where == "element" and n:
                rec = ELEMENTS[owner]
                esz = types.size_of(rec) or n
                span, flagged, gaps = describe(types, rec, 0, min(n, esz)) if rec in types.structs else (
                    f"{rec} x{n // esz}", [], [])
            elif off is not None and n:
                for cand in (cls, cls + "Data", cls.replace("Data", "")):
                    span, flagged, gaps = describe(types, cand, off, off + n)
                    if span:
                        rec = cand
                        break
            gap = "; ".join(f"+{lo:#x}..+{hi:#x}" for lo, hi in gaps)
            f.write(f"{LINES[line][0] if line < len(LINES) else line}\t{owner}\t{site:08x}\t"
                    f"{where}\t{'' if n is None else n}\t{count}\t{total_b}\t"
                    f"{first:08x}\t{rec}\t{span}\t{'; '.join(flagged)}\t{gap}\n")


def kind(ty):
    t = ty.replace(" ", "")
    if "*" in t:
        return "ptr"
    elem = t.split("[")[0]
    if elem in ("float", "double") or elem.startswith("Vector<float") or elem.startswith("Vector3<float"):
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

    # A frame is live while its entry slot still holds the return address
    # it was entered with: a returned frame's slot is overwritten by the
    # next call's arguments, and a sibling call with more arguments sits
    # below it, so depth alone does not say a frame is dead.
    def prune(esp):
        while owners and (owners[-1][0] <= esp or u32(owners[-1][0]) != owners[-1][3]):
            owners.pop()

    def on_walker(uc_, address, size, _):
        esp = uc_.reg_read(UC_X86_REG_ESP)
        prune(esp)
        owners.append((esp, address, uc_.reg_read(UC_X86_REG_ECX), u32(esp)))

    walkers = [va for va, n in zip(index.va, index.names)
               if (n.endswith("::walk_data") or n.endswith("::walk_rules_data") or n.startswith("CheckSums::check_"))
               and va != WALK_FUNCTION]
    for va in walkers:
        uc.hook_add(UC_HOOK_CODE, on_walker, begin=va, end=va)

    def on_fold(uc_, address, size, _):
        esp = uc_.reg_read(UC_X86_REG_ESP)
        site, start, end = u32(esp), u32(esp + 4), u32(esp + 8)
        prune(esp)
        # the owner is the function the call site is in; its `this` is known
        # when that function is the innermost live walker frame
        owner, this = index.of(site)[0], 0
        if owners and owners[-1][1] == owner:
            this = owners[-1][2]
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
        write_ledger(a.ledger, ranges, index, Types(a.types), world=u32(WORLD_GLOBAL))
        print(f"ledger: {a.ledger}")
    if state["stop"] is not None:
        return 2
    if record and agree != len(LINES):
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
