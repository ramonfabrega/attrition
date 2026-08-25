#!/usr/bin/env python3
"""report.py — read rontrace.log (tools/trace/tracer.c) and name what it holds.

    report.py <log> [--index INDEX.tsv] summary
    report.py <log> draws [FRAME ...]          every draw of the given sim-frames
                                               (default: all; `setup` = before frame 0),
                                               with the caller and the ebp chain named
    report.py <log> sites [FRAME ...]          the same, folded by caller site with counts
    report.py <log> coverage [FRAME ...]       functions entered, per frame (re-armed
                                               frames) — names, counts
    report.py <log> blind <docs dir> [<log>...] every `name@00xxxxxx` cited in the docs
                                               that no given trace entered
    report.py <log> functions                  every function ever entered, with the
                                               frame it was first entered on

Addresses are named through the Ghidra export's INDEX.tsv (default
~/ghidra-projects/decomp/INDEX.tsv): a return address is reported as
`Class::method+0xNN`, the function containing it. The log's frames are the
original's `Game::frame` at `do_frame` entry, i.e. sim-frames as docs/SYNC.md
counts them; the records before the first FRAME record are the setup path
(frame -1, `setup`).
"""
import bisect
import os
import re
import struct
import sys
from collections import Counter, OrderedDict, defaultdict

BASE = 0x400000
KINDS = {0: "HIT", 1: "get()", 2: "FRAME", 3: "get(a,b)", 4: "rand_real", 5: "INFO", 6: "reseed"}
INFO = {1: "attach", 2: "hook-mismatch", 3: "hooked", 4: "no-funcs", 5: "protect-fail",
        6: "armed", 7: "detach"}
RVA_GAME_RANDOM = 0xA37A8C  # VA 0xE37A8C
# the trampolined functions (tracer.c HOOKS): rva -> the record kind they emit
HOOKS = {0x191ef0: 2, 0x639cf0: 1, 0x639d70: 3, 0x5e18b0: 4, 0x639d30: 6}


class Index:
    def __init__(self, path):
        self.addrs, self.names = [], []
        for line in open(path):
            a, n = line.rstrip("\n").split("\t")[:2]
            self.addrs.append(int(a, 16))
            self.names.append(n)
        order = sorted(range(len(self.addrs)), key=lambda i: self.addrs[i])
        self.addrs = [self.addrs[i] for i in order]
        self.names = [self.names[i] for i in order]
        self.by_addr = dict(zip(self.addrs, self.names))

    def name(self, va, base=BASE):
        """`Class::method+0xNN` for the function containing va (an absolute VA)."""
        if not va:
            return "-"
        va = va - base + BASE
        i = bisect.bisect_right(self.addrs, va) - 1
        if i < 0:
            return f"{va:#x}"
        off = va - self.addrs[i]
        return self.names[i] if off == 0 else f"{self.names[i]}+{off:#x}"

    def func(self, va, base=BASE):
        va = va - base + BASE
        i = bisect.bisect_right(self.addrs, va) - 1
        return self.names[i] if i >= 0 else f"{va:#x}"


def read_log(path):
    data = open(path, "rb").read()
    n = len(data) // 32
    recs = [struct.unpack_from("<8I", data, i * 32) for i in range(n)]
    hdr = recs[0]
    assert hdr[0] == 0x544E4F52, "not a rontrace.log"
    head = dict(version=hdr[1], base=hdr[2], text_rva=hdr[3], text_size=hdr[4], nfuncs=hdr[5],
                win_lo=struct.unpack("<i", struct.pack("<I", hdr[6]))[0],
                win_hi=struct.unpack("<i", struct.pack("<I", hdr[7]))[0])
    return head, recs[1:]


def s32(u):
    return u - (1 << 32) if u >= (1 << 31) else u


def frame_of(rec):
    return s32(rec[7])


def frame_arg(a):
    return -1 if a == "setup" else int(a)


def main():
    args = sys.argv[1:]
    index_path = os.path.expanduser("~/ghidra-projects/decomp/INDEX.tsv")
    if "--index" in args:
        i = args.index("--index")
        index_path = args[i + 1]
        del args[i:i + 2]
    log, cmd, rest = args[0], args[1] if len(args) > 1 else "summary", args[2:]
    head, recs = read_log(log)
    base = head["base"]
    idx = Index(index_path)

    def nm(va):
        return idx.name(va, base)

    def rng(self):
        return "game" if self - base == RVA_GAME_RANDOM else f"rng@{self:#x}"

    if cmd == "summary":
        kinds = Counter(KINDS.get(r[0], r[0]) for r in recs)
        print(f"base {base:#x}  functions listed {head['nfuncs']}  window "
              f"{head['win_lo']}..{head['win_hi']}  records {len(recs)}")
        for k, v in sorted(kinds.items(), key=lambda kv: str(kv[0])):
            print(f"  {k:<10} {v}")
        for r in recs:
            if r[0] == 5:
                print(f"  INFO {INFO.get(r[1], r[1]):<14} {r[2]:#x} {r[3]:#x} {r[4]:#x} {r[5]:#x} {r[6]:#x}")
        frames = [r for r in recs if r[0] == 2]
        draws = defaultdict(int)
        for r in recs:
            if r[0] in (1, 3, 4) and r[2] - base == RVA_GAME_RANDOM:
                draws[frame_of(r)] += 1
        print(f"frames {len(frames)}: " + ", ".join(
            f"{s32(r[1])}:{draws[s32(r[1])]}@{r[2]:08x}" for r in frames[:12])
              + (" ..." if len(frames) > 12 else ""))
        print(f"setup draws (before frame 0): {draws[-1]}")
        return

    want = set(frame_arg(a) for a in rest) if rest and cmd != "blind" else None

    if cmd in ("draws", "sites"):
        per_site = Counter()
        k = 0
        for r in recs:
            if r[0] not in (1, 3, 4, 6):
                continue
            f = frame_of(r)
            if want is not None and f not in want:
                continue
            kind = KINDS[r[0]]
            chain = f"{nm(r[1])} < {nm(r[4])} < {nm(r[5])}"
            if cmd == "draws":
                extra = f" arg0={s32(r[6])}" if r[0] in (3, 6) else ""
                print(f"f{f:<5} {k:>4} {kind:<10} {rng(r[2]):<5} seed {r[3]:08x}  {chain}{extra}")
                k += 1
            else:
                per_site[(f, kind, rng(r[2]), chain)] += 1
        if cmd == "sites":
            for (f, kind, g, chain), n in sorted(per_site.items(), key=lambda kv: (kv[0][0], -kv[1])):
                print(f"f{f:<5} {n:>6}  {kind:<10} {g:<5} {chain}")
        return

    if cmd == "coverage":
        per_frame = OrderedDict()
        for r in recs:
            if r[0] == 0:
                per_frame.setdefault(frame_of(r), []).append(r[1])
        for f, hits in per_frame.items():
            if want is not None and f not in want:
                continue
            names = sorted(idx.func(a, base) for a in hits)
            print(f"== frame {f}: {len(hits)} functions entered")
            for n in names:
                print(f"   {n}")
        return

    if cmd == "functions":
        first = OrderedDict()
        for r in recs:
            if r[0] == 0 and r[1] not in first:
                first[r[1]] = frame_of(r)
        for a, f in sorted(first.items(), key=lambda kv: kv[0]):
            print(f"{a - base + BASE:08x} f{f:<6} {idx.func(a, base)}")
        print(f"{len(first)} functions entered", file=sys.stderr)
        return

    if cmd == "blind":
        docs = rest[0]
        logs = [log] + rest[1:]
        entered = set()
        for l in logs:
            h, rs = read_log(l)
            kinds = set()
            for r in rs:
                if r[0] == 0:
                    entered.add(r[1] - h["base"] + BASE)
                else:
                    kinds.add(r[0])
            # the hooked functions carry no int3; they were entered iff their records exist
            for rva, kind in HOOKS.items():
                if kind in kinds:
                    entered.add(rva + BASE)
        cited = defaultdict(set)
        pat = re.compile(r"([A-Za-z_][A-Za-z0-9_:~<>]*)@(00[0-9a-f]{6})")
        for root, _, files in os.walk(docs):
            for fn in files:
                if not fn.endswith(".md"):
                    continue
                p = os.path.join(root, fn)
                for m in pat.finditer(open(p, encoding="utf-8", errors="replace").read()):
                    cited[int(m.group(2), 16)].add(os.path.relpath(p, docs))
        known = [a for a in cited if a in idx.by_addr]
        never = sorted(a for a in known if a not in entered)
        hit = [a for a in known if a in entered]
        print(f"{len(known)} functions cited by address under {docs}; "
              f"{len(hit)} entered by {len(logs)} trace(s), {len(never)} never:")
        for a in never:
            print(f"  {a:08x} {idx.by_addr[a]:<50} {', '.join(sorted(cited[a]))}")
        return

    print(__doc__)


if __name__ == "__main__":
    main()
