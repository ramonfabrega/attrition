#!/usr/bin/env python3
"""ranges.py — step R's record (item 4248, `docs/CHECKSUM.md` §25): every range
the original's checksum fold took on the open lines, on every frame of one
playback.

    python3 tools/trace/ranges.py <rontrace.ranges> summary
    python3 tools/trace/ranges.py <rontrace.ranges> check [--hook <rontrace.log>]
    python3 tools/trace/ranges.py <rontrace.ranges> slice --frame F --out DIR [--ordinal K]
    python3 tools/trace/ranges.py <rontrace.ranges> compare --frame F --stream <checksum.py stream.tsv>

The record is written by `tools/trace/ranges.h` under `-DRON_RANGES`: per due
`check_all` set, the set's sixteen words and every range the fold took on the
lines in its mask (lines 0..7, units to leaders), with the bytes as folded.

`summary` counts the sets, frames, ranges and bytes. `check` re-sums each kept
line's ranges with Adler-32 and holds it against the set's own word for that
line, on every set — a range the hook missed or misplaced parts there — and,
with `--hook`, holds each set's fifteen words against another take's K_CSUM
sets on the same frame and place (run1578: the hook perturbs nothing). Exit 0
only when every set agrees.

`slice` writes one set as `tools/recomp/checksum.py` writes a packet's
instrument files: `DIR/stream.tsv` (its kept lines' ranges, the owner named
from the decompile export's INDEX.tsv) and `DIR/values.tsv` (the fifteen,
headed `# packet logger frame F, trace frame F-1`), so a test that reads a
packet's stream (`rondata::diff::checksum_lines::packets::packet_stream_in`)
reads the record's frame unchanged. `--frame` is the logger frame (our walk's
`sim.frame`); the set is the first of trace frame F-1, as §24.1 pairs them
(`--ordinal` picks another of that frame's sets). `compare` holds the same
slice against a checksum.py stream range by range: line, owner, site, length
and bytes (the start, a heap address, is printed where it differs and not
counted).

The bytes are the original's memory: keep the record and every slice outside
git, under `~/ron-data`.
"""
import argparse
import bisect
import os
import struct
import sys
import zlib

MAGIC = 0x524E4752
SET_MAGIC = 0x54455352
VERSION = 1
OVER = 1
LINES = ["units", "builds", "walls", "ammo", "deaths", "groups", "guys", "leaders", "cities", "items",
         "goods", "world", "rules", "scenario_data", "script_run_time"]
LABELS = ["Units", "Builds", "Walls", "Ammo", "Deaths", "Groups", "Guys", "Leaders", "Cities", "Items",
          "Goods", "World", "Rules", "scenario_data", "script_run_time"]
SET = struct.Struct("<IiIIIIII16I")
ENTRY = struct.Struct("<BBHIII")
PREFERRED_BASE = 0x400000


class Pieces:
    """A record's bytes across `path`, `path.1`, `path.2`, …: a record past a
    gigabyte is kept in pieces so the archive's compression takes it
    (`tools/trace/split.py`, DECISIONS 149 §1), and the pieces in order are
    the record. A record in one file is one piece."""

    def __init__(self, path):
        self.files, self.size, self.at = [], 0, 0
        k = 0
        while k == 0 or os.path.isfile(f"{path}.{k}"):
            f = open(path if k == 0 else f"{path}.{k}", "rb")
            n = os.fstat(f.fileno()).st_size
            self.files.append((f, self.size, n))
            self.size += n
            k += 1

    def seek(self, at):
        self.at = at

    def read(self, n):
        out = []
        while n > 0:
            hit = next(((f, start, size) for f, start, size in self.files if self.at < start + size), None)
            if hit is None:
                break
            f, start, size = hit
            f.seek(self.at - start)
            b = f.read(min(n, start + size - self.at))
            if not b:
                break
            out.append(b)
            self.at += len(b)
            n -= len(b)
        return b"".join(out)


class Record:
    """The record's sets, read lazily: `sets()` yields each set's header with
    its file offset; `body(s)` reads its ranges and bytes."""

    def __init__(self, path):
        self.path = path
        self.f = Pieces(path)
        head = self.f.read(32)
        if len(head) < 32:
            raise SystemExit(f"{path}: no header")
        magic, version, self.base, self.mask = struct.unpack("<4I", head[:16])
        if magic != MAGIC or version != VERSION:
            raise SystemExit(f"{path}: not a step R record (magic {magic:#x}, version {version})")
        self.size = self.f.size

    def kept(self):
        return [i for i in range(len(LINES)) if (self.mask >> i) & 1]

    def sets(self):
        at = 32
        while at + SET.size <= self.size:
            self.f.seek(at)
            raw = self.f.read(SET.size)
            w = SET.unpack(raw)
            if w[0] != SET_MAGIC:
                raise SystemExit(f"{self.path}: no set at offset {at}")
            s = {"at": at, "frame": w[1], "ordinal": w[2], "caller": w[3], "count": w[4], "flags": w[5],
                 "nranges": w[6], "nbytes": w[7], "v": list(w[8:24])}
            n = SET.size + s["nranges"] * ENTRY.size + s["nbytes"]
            n += (4 - (s["nbytes"] & 3)) & 3
            if at + n > self.size:
                s["truncated"] = True
                yield s
                return
            yield s
            at += n

    def body(self, s):
        """`[(line, via, site, start, len, bytes)]` in fold order."""
        self.f.seek(s["at"] + SET.size)
        ents = [ENTRY.unpack(self.f.read(ENTRY.size)) for _ in range(s["nranges"])]
        data = self.f.read(s["nbytes"])
        out, k = [], 0
        for line, via, _pad, site, start, n in ents:
            out.append((line, via, site - self.base + PREFERRED_BASE, start, n, data[k:k + n]))
            k += n
        return out

    def find(self, trace_frame, ordinal=None):
        """The first set of `trace_frame` (or the one at `ordinal`)."""
        for s in self.sets():
            if s["frame"] == trace_frame and (ordinal is None or s["ordinal"] == ordinal):
                return s
            if s["frame"] > trace_frame:
                break
        return None


def adler(chunks):
    a = 1
    for c in chunks:
        a = zlib.adler32(c, a)
    return a


class Index:
    """The decompile export's INDEX.tsv: the function that holds an address."""

    def __init__(self, path):
        pairs = []
        if path and os.path.exists(path):
            for row in open(path, encoding="latin-1"):
                p = row.rstrip("\n").split("\t")
                if len(p) >= 2:
                    try:
                        pairs.append((int(p[0], 16), p[1]))
                    except ValueError:
                        pass
        pairs.sort()
        self.va = [a for a, _ in pairs]
        self.names = [n for _, n in pairs]

    def of(self, va):
        i = bisect.bisect_right(self.va, va) - 1
        return self.names[i] if i >= 0 else "?"


def hook_sets(path):
    """A rontrace.log's K_CSUM sets: `{(trace frame, ordinal): [16 words]}`."""
    out, part = {}, {}
    with open(path, "rb") as f:
        f.read(32)
        while True:
            raw = f.read(32)
            if len(raw) < 32:
                break
            k, a, b, c, d, e, g, frame = struct.unpack("<8I", raw)
            if k != 13:
                continue
            frame = frame - (1 << 32) if frame >= 1 << 31 else frame
            p, o = a & 0xFF, a >> 8
            if p == 0:
                part = {"w": [c, d, e, g]}
            elif p == 1:
                part.setdefault("w", []).extend([b, c, d, e, g])
            elif p == 2:
                part.setdefault("w", []).extend([b, c, d, e, g])
            elif p == 3:
                w = part.get("w", []) + [b, c]
                if len(w) == 16:
                    out[(frame, o)] = w
                part = {}
    return out


def summary(rec):
    sets = frames = ranges = data = over = 0
    lo = hi = None
    per_line = [0] * len(LINES)
    last = None
    for s in rec.sets():
        sets += 1
        ranges += s["nranges"]
        data += s["nbytes"]
        over += s["flags"] & OVER != 0
        if s["frame"] != last:
            frames += 1
            last = s["frame"]
        lo = s["frame"] if lo is None else lo
        hi = s["frame"]
        if s.get("truncated"):
            print(f"truncated set at offset {s['at']} (frame {s['frame']})")
    print(f"record: {rec.path}, {rec.size} bytes, base {rec.base:#x}, lines {[LINES[i] for i in rec.kept()]}")
    print(f"sets {sets} on {frames} trace frames {lo}..{hi}; {ranges} ranges, {data} bytes; {over} over")
    return 0


def check(rec, hook):
    theirs = hook_sets(hook) if hook else None
    sets = bad = over = 0
    hook_bad = hook_missing = 0
    first_bad = None
    kept = rec.kept()
    for s in rec.sets():
        sets += 1
        if s.get("truncated"):
            print(f"truncated at frame {s['frame']}")
            bad += 1
            break
        if s["flags"] & OVER:
            over += 1
            continue
        body = rec.body(s)
        for li in kept:
            got = adler(r[5] for r in body if r[0] == li)
            if got != s["v"][li]:
                bad += 1
                if first_bad is None:
                    first_bad = (s["frame"], s["ordinal"], LINES[li], got, s["v"][li])
        if sum(s["v"][:15]) & 0xFFFFFFFF != s["v"][15]:
            bad += 1
            first_bad = first_bad or (s["frame"], s["ordinal"], "total", sum(s["v"][:15]) & 0xFFFFFFFF, s["v"][15])
        if theirs is not None:
            t = theirs.get((s["frame"], s["ordinal"]))
            if t is None:
                hook_missing += 1
            elif t != s["v"]:
                hook_bad += 1
                if hook_bad == 1:
                    parted = [LINES[i] if i < 15 else "total" for i in range(16) if t[i] != s["v"][i]]
                    print(f"first set parting from the hook's: frame {s['frame']} ordinal {s['ordinal']}: {parted}")
    print(f"sets {sets}; over {over}; lines whose ranges do not sum to the set's word: {bad}")
    if first_bad:
        print(f"first: frame {first_bad[0]} ordinal {first_bad[1]} {first_bad[2]}: ranges {first_bad[3]}, "
              f"word {first_bad[4]}")
    if theirs is not None:
        print(f"against {hook}: {len(theirs)} sets there; {hook_bad} part, {hook_missing} of ours not there, "
              f"{len(theirs) - (sets - hook_missing)} of theirs not here")
    ok = bad == 0 and over == 0 and (theirs is None or (hook_bad == 0 and hook_missing == 0
                                                        and len(theirs) == sets))
    print("verdict:", "every set agrees" if ok else "PARTS")
    return 0 if ok else 1


def rows(rec, s, index):
    """A set's kept ranges as checksum.py's stream rows."""
    out = []
    for seq, (line, via, site, start, n, data) in enumerate(rec.body(s)):
        out.append((LINES[line], seq, index.of(site), 0, site, start, n, data))
    return out


def pick(rec, frame, ordinal):
    s = rec.find(frame - 1, ordinal)
    if s is None:
        raise SystemExit(f"no set on trace frame {frame - 1}" + (f" at ordinal {ordinal}" if ordinal is not None else ""))
    if s["flags"] & OVER:
        raise SystemExit(f"trace frame {frame - 1}'s set did not fit: no ranges")
    return s


def slice_out(rec, frame, ordinal, out, index):
    s = pick(rec, frame, ordinal)
    os.makedirs(out, exist_ok=True)
    rs = rows(rec, s, index)
    with open(os.path.join(out, "stream.tsv"), "w") as f:
        f.write("line\tseq\towner\tthis\tsite\tstart\tlength\tbytes\n")
        for line, seq, owner, this, site, start, n, data in rs:
            f.write(f"{line}\t{seq}\t{owner}\t{this:08x}\t{site:08x}\t{start:08x}\t{n}\t{data.hex()}\n")
    with open(os.path.join(out, "values.tsv"), "w") as f:
        f.write(f"# packet logger frame {frame}, trace frame {frame - 1}\n")
        f.write("line\tlabel\tvalue\tranges\tbytes\n")
        for i, (line, label) in enumerate(zip(LINES, LABELS)):
            these = [r for r in rs if r[0] == line]
            kept = (rec.mask >> i) & 1
            f.write(f"{line}\t{label}\t{s['v'][i]}\t{len(these) if kept else ''}\t"
                    f"{sum(r[6] for r in these) if kept else ''}\n")
    print(f"trace frame {frame - 1} ordinal {s['ordinal']} (caller {s['caller']:#x}): {len(rs)} ranges -> {out}")
    return 0


def compare(rec, frame, ordinal, stream, index):
    s = pick(rec, frame, ordinal)
    ours = rows(rec, s, index)
    kept = {LINES[i] for i in rec.kept()}
    theirs = []
    for row in open(stream).read().splitlines()[1:]:
        c = row.split("\t")
        if len(c) == 8 and c[0] in kept:
            theirs.append((c[0], int(c[1]), c[2], int(c[3], 16), int(c[4], 16), int(c[5], 16), int(c[6]),
                           bytes.fromhex(c[7])))
    print(f"trace frame {frame - 1} ordinal {s['ordinal']}: record {len(ours)} ranges, stream {len(theirs)}")
    worst = 0
    for line in [LINES[i] for i in rec.kept()]:
        a = [r for r in ours if r[0] == line]
        b = [r for r in theirs if r[0] == line]
        parted = starts = 0
        first = None
        for i in range(max(len(a), len(b))):
            x = a[i] if i < len(a) else None
            y = b[i] if i < len(b) else None
            same = x and y and (x[2], x[4], x[6], x[7]) == (y[2], y[4], y[6], y[7])
            if not same:
                parted += 1
                if first is None:
                    first = (i, x and (x[2], hex(x[4]), x[6], x[7][:16].hex()), y and (y[2], hex(y[4]), y[6], y[7][:16].hex()))
            elif x[5] != y[5]:
                starts += 1
        nb = sum(r[6] for r in a)
        print(f"{line:<8} record {len(a):>5} ranges {nb:>7} bytes; stream {len(b):>5}; parted {parted}; "
              f"starts differing {starts}" + (f"; first at {first[0]}: record {first[1]}, stream {first[2]}" if first else ""))
        worst += parted
    print("verdict:", "byte for byte" if worst == 0 else f"PARTS on {worst} ranges")
    return 0 if worst == 0 else 1


def main(argv):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("record")
    ap.add_argument("verb", choices=["summary", "check", "slice", "compare"])
    ap.add_argument("--hook")
    ap.add_argument("--frame", type=int)
    ap.add_argument("--ordinal", type=int)
    ap.add_argument("--out")
    ap.add_argument("--stream")
    ap.add_argument("--index", default=os.path.expanduser("~/ghidra-projects/decomp/INDEX.tsv"))
    a = ap.parse_args(argv[1:])
    rec = Record(a.record)
    if a.verb == "summary":
        return summary(rec)
    if a.verb == "check":
        return check(rec, a.hook)
    if a.frame is None:
        raise SystemExit(f"{a.verb} needs --frame (the logger frame)")
    index = Index(a.index)
    if a.verb == "slice":
        if not a.out:
            raise SystemExit("slice needs --out")
        return slice_out(rec, a.frame, a.ordinal, a.out, index)
    if not a.stream:
        raise SystemExit("compare needs --stream")
    return compare(rec, a.frame, a.ordinal, a.stream, index)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
