#!/usr/bin/env python3
"""census.py — the executable as the denominator.

    census.py [--index INDEX.tsv] [--docs docs/] [--top N] [--never] [--pin] [<rontrace.log> ...]

Every function in the executable, grouped by the class Ghidra's export files
it under, against two things this repo can measure: the functions `docs/`
cites by address (`name@00xxxxxx`) and the functions any given trace entered
(`tools/trace/report.py <log> functions`, run here for each log). The three
counters `docs/DECISIONS.md` 29 names are all relative to what has been
inspected; this table's denominator is the binary itself.

Prints the order family whole (every `*Order` class — the capability
inventory, one row per thing a unit can be told to do), then the core
classes by size, then a one-line total. `--top` bounds the core list.
Nothing here is a completion percentage: a cited function is one a reading
named, an entered one is one a run reached, and neither says the predicate
inside was checked. It is the map, not the score.

The trace logs live outside the repo (`docs/ORACLE.md`); with none given the
entered column is blank, not zero.

`--never` appends the blind list itself — every cited function no given
trace entered, grouped by class, largest group first — and a `never` count
on the total line. It is `report.py`'s `blind` verb grouped, and the
measurement `rondata::blind`'s guard pins; `docs/CENSUS.md`'s "The blind
list, ranked" is written from it (item 923).
"""
import argparse
import glob
import os
import re
import subprocess
import sys
from collections import Counter

HERE = os.path.dirname(os.path.abspath(__file__))
REPORT = os.path.join(HERE, "trace", "report.py")
# A citation broken after its `@` — `name@` ending a line, the address
# opening the next — is joined (parked 835): 49 stood across `docs/` and
# each counted as two functions.
CITE = re.compile(r"[A-Za-z_][A-Za-z0-9_:~]*@(?:\n[ \t]*)?(00[0-9a-f]{6})")
ENTERED = re.compile(r"^([0-9a-f]{8})\s")
NOISE = ("LinkList", "Recycler", "Array", "Stack", "SimpleArray", "Tree_", "allocator", "_dynamic")


def load_index(path):
    total, cls_of, name_of = Counter(), {}, {}
    with open(path) as f:
        for line in f:
            addr, name, file = line.rstrip("\n").split("\t")[:3]
            cls = file.split("/")[1]
            va = int(addr, 16)
            total[cls] += 1
            cls_of[va] = cls
            name_of[va] = name
    return total, cls_of, name_of


def cited_in(docs):
    out = set()
    for p in glob.glob(os.path.join(docs, "**", "*.md"), recursive=True):
        with open(p, errors="replace") as f:
            out |= {int(m.group(1), 16) for m in CITE.finditer(f.read())}
    return out


def entered_in(logs, index):
    out = set()
    for lg in logs:
        text = subprocess.run(
            [sys.executable, REPORT, lg, "--index", index, "functions"],
            capture_output=True, text=True, check=True).stdout
        for line in text.splitlines():
            m = ENTERED.match(line)
            if m:
                out.add(int(m.group(1), 16))
    return out


# The five trampolined functions carry no coverage stub; a trace entered one
# exactly when it holds a record of the kind its hook emits (`report.py`).
HOOKS = {0x591ef0: "FRAME", 0xa39cf0: "get()", 0xa39d70: "get(a,b)",
         0x9e18b0: "rand_real", 0xa39d30: "reseed"}


def hooked_in(logs, index):
    out = set()
    for lg in logs:
        text = subprocess.run([sys.executable, REPORT, lg, "--index", index, "summary"],
                              capture_output=True, text=True, check=True).stdout
        kinds = {line.split()[0] for line in text.splitlines() if line.startswith("  ")}
        out |= {va for va, k in HOOKS.items() if k in kinds}
    return out


PIN = os.path.join(HERE, "..", "crates", "rondata", "src", "blind.rs")
ARCHIVE = os.environ.get(
    "RON_GAMELOG_DIR",
    os.path.expanduser("~/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"))


def pinned_traces(pin=PIN):
    """`rondata::blind::TRACES`, read from the source in its own order: the
    traces the blind list is measured against are the ones the census reads
    (parked 938, 939). A name inside a comment is not an entry."""
    with open(pin) as f:
        text = f.read()
    body = text[text.index("pub const TRACES"):]
    body = body[:body.index("\n];")]
    out = []
    for line in body.splitlines()[1:]:
        m = re.match(r'\s*"([^"]+)",', line)
        if m:
            out.append(m.group(1))
    return out


def pinned_logs(archive=ARCHIVE):
    """Each pinned trace joined to the archive directory — a path a trace,
    whatever spaces the directory holds (parked 942)."""
    logs = [os.path.join(archive, name) for name in pinned_traces()]
    missing = [os.path.basename(p) for p in logs if not os.path.isfile(p)]
    if missing:
        sys.exit("pinned and not in %s: %s" % (archive, " ".join(missing)))
    return logs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("logs", nargs="*")
    ap.add_argument("--index", default=os.path.expanduser("~/ghidra-projects/decomp/INDEX.tsv"))
    ap.add_argument("--docs", default=os.path.join(HERE, "..", "docs"))
    ap.add_argument("--top", type=int, default=40)
    ap.add_argument("--never", action="store_true")
    ap.add_argument("--pin", action="store_true",
                    help="read the traces rondata::blind::TRACES pins, from $RON_GAMELOG_DIR")
    a = ap.parse_args()
    if a.pin:
        a.logs = pinned_logs() + a.logs

    total, cls_of, name_of = load_index(a.index)
    cited = cited_in(a.docs)
    entered = entered_in(a.logs, a.index) if a.logs else None
    cited_by = Counter(cls_of.get(v, "?") for v in cited)
    entered_by = Counter(cls_of.get(v, "?") for v in entered) if entered is not None else None

    def row(cls):
        n = total[cls]
        e = "" if entered_by is None else entered_by.get(cls, 0)
        return f"{cls:28} {n:6} {cited_by.get(cls, 0):6} {e!s:>8}"

    head = f"{'class':28} {'total':>6} {'cited':>6} {'entered':>8}"
    orders = sorted((c for c in total if c.endswith("Order") and not c.startswith(NOISE)),
                    key=lambda c: -total[c])
    print("# the order family — one row per thing a unit can be told to do")
    print(head)
    for c in orders:
        print(row(c))
    o_tot = sum(total[c] for c in orders)
    o_cit = sum(cited_by.get(c, 0) for c in orders)
    o_ent = "" if entered_by is None else sum(entered_by.get(c, 0) for c in orders)
    print(f"{'(orders)':28} {o_tot:6} {o_cit:6} {o_ent!s:>8}   classes touched by a citation: "
          f"{sum(1 for c in orders if cited_by.get(c, 0))}/{len(orders)}")

    print()
    print("# core classes by size (free functions, CRT, STL and cut modules excluded)")
    print(head)
    core = [c for c in total if c != "_global" and not c.startswith(NOISE) and not c.endswith("Order")
            and (cited_by.get(c, 0) or (entered_by and entered_by.get(c, 0)))]
    for c in sorted(core, key=lambda c: -total[c])[: a.top]:
        print(row(c))

    e_all = "" if entered is None else len(entered)
    never = ""
    if a.never and entered is not None:
        seen = entered | hooked_in(a.logs, a.index)
        blind = sorted(v for v in cited if v in name_of and v not in seen)
        groups = {}
        for v in blind:
            groups.setdefault(cls_of[v], []).append(v)
        print()
        print("# the blind list — cited, in the export, entered by no given trace")
        for c in sorted(groups, key=lambda c: (-len(groups[c]), c)):
            print(f"{c:28} {len(groups[c]):4}  " + " ".join(name_of[v].split("::")[-1] for v in groups[c]))
        never = f"  never {len(blind)}"
    print()
    print(f"functions {sum(total.values())}  classes {len(total)}  cited {len(cited)} in "
          f"{len(cited_by)} classes  entered {e_all}{never}  (logs: {len(a.logs)})")


if __name__ == "__main__":
    main()
