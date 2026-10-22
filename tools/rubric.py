#!/usr/bin/env python3
r"""rubric.py — a steering pass's holistic scorecard of how much of the
original's simulation this port has verified.

    rubric.py [--tables 1,2,3] [--limit N] [--dumps PATH ...] [--top N]
    rubric.py --record "the fiftieth pass"     # at a steer: append the ledger's row, print it against the last
    rubric.py --delta                           # the last two ledger rows as a table, reading no dump

Three tables, each from data the tree (or the machine's own archive) already
holds, and a one-line `Rubric:` summary at the end. It is a measurement
tool: it changes no score, pins nothing, and no gate runs it.
`docs/RUBRIC.md` says what each table measures and what it does not.

1. **By mechanic.** The PDB's source files are the exhaustive mechanic list.
   Every function of the export the census files under the simulation layer
   (`census.layer_of` on its line record's source path) is counted under its
   source file, a header grouped with the `.cpp` of the same stem: backed
   (doc / sweep, exactly as `census.py --layers` counts), and for the rest the
   mark `census.rank_unbacked` gives — dead weight (`skip`), `ui`,
   `operator`, `cut`, carried (`port` or `name`: the crate defines the
   method's snake-case name) and unmarked (`-`: the crate does not carry
   it). Bytes are the census's (the decompile file's size, a proxy). The
   docs column names the documents under `docs/` that cite any of the
   file's functions by address (`census.cited_per_doc`). Sorted by unbacked
   bytes, largest first. A file is **100 %** when every function is backed
   or marked dead weight, ui or operator.
2. **By age.** Every pinned trace (`rondata::blind::TRACES`, read by
   `census.pinned_traces`) has its matching `gamelog-<run>-*.txt` dumps in
   the archive (`$RON_GAMELOG_DIR`, `census.ARCHIVE`). Each `LEADERDATA`
   block's `ages_get()` — `LeaderData::get_age()`, how many of the seven
   ages after Ancient the leader owns, 0–7 (`docs/TECH.md`, "Per-player
   state"; read by `rondata::diff::leader`, not by `gamelog.rs`'s
   `LeaderDump`) — is paired with the `leader_flags` written *before* the
   block (`docs/ORACLE.md`; bit 0x1 active, 0x4 human). Per age: the
   (dump, frame) pairs with an active human leader in it, with an active
   computer leader, and with either. The start-of-game dump before the
   first `BEGIN FRAME` is its own frame, `start`.
3. **By unit type.** Every `GUY` block's first line, `type N` — the unit
   type id (`docs/FORMATS.md`: units are 50–413, 50 + the `unitrules.xml`
   record) — in the same dumps. Printed against `unitrules.xml`'s 364
   records (name, `DOMAIN`, `CAT`), with each type's loaded `age` from the
   start-of-game types dump (`gamelog-run3-fulldump-types.txt`) when the
   archive holds it; the never-seen types grouped by domain, then age.

The dumps are read one pass each, memory-mapped, with one regular
expression, and each file's progress goes to stderr; `--limit N` reads the
first N dumps only and says so on the summary line.
"""
import argparse
import glob
import mmap
import os
import re
import sys
from collections import Counter, defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import census  # noqa: E402

UNIT_FIRST, UNIT_COUNT = 50, 364

# --- table 1: by mechanic -------------------------------------------------

EXEMPT = ("skip", "ui", "operator")


def group_key(fname):
    """A source file's mechanic: its stem, so `orders.h` sits with
    `orders.cpp`."""
    return fname.rsplit(".", 1)[0]


def mechanic_rows(entries, backed, by_doc, port, spelled, cites):
    """One row per simulation source group. `entries` is `(va, name, path,
    size)` with `path` the lower-cased PDB source path; `backed` the by-doc
    and by-sweep union, `by_doc` its doc part; `port`/`spelled` as
    `census.rank_unbacked` takes them; `cites` `{va: set of doc names}`."""
    sim = [e for e in entries if census.layer_of(e[2]) == "simulation"]
    marks = {va: mark for _size, va, _name, mark, _f in census.rank_unbacked(sim, backed, port, spelled)}
    rows = {}
    for va, _name, path, size in sim:
        fname = path.rsplit("\\", 1)[-1]
        r = rows.setdefault(group_key(fname), {
            "files": set(), "functions": 0, "backed": 0, "doc": 0, "sweep": 0, "skip": 0, "ui": 0,
            "operator": 0, "cut": 0, "carried": 0, "unmarked": 0, "bytes": 0, "backed_bytes": 0,
            "docs": Counter()})
        r["files"].add(fname)
        r["functions"] += 1
        r["bytes"] += size
        if va in backed:
            r["backed"] += 1
            r["backed_bytes"] += size
            r["doc" if va in by_doc else "sweep"] += 1
        else:
            m = marks.get(va, "-")
            key = {"port": "carried", "name": "carried", "-": "unmarked", "claimed": "unmarked"}.get(m, m)
            r[key] += 1
        for d in cites.get(va, ()):
            r["docs"][d] += 1
    for r in rows.values():
        r["full"] = r["backed"] + sum(r[m] for m in EXEMPT) == r["functions"]
        r["unbacked_bytes"] = r["bytes"] - r["backed_bytes"]
    return rows


def doc_names(per_doc, docs_root):
    """`{va: set of doc names}`: a top-level document by its file name, one
    under a subdirectory by its path from `docs/`."""
    root = os.path.realpath(docs_root)
    out = defaultdict(set)
    for path, vas in per_doc.items():
        name = os.path.relpath(path, root)
        for va in vas:
            out[va].add(name)
    return out


def mechanics_now(index, docs, sim_src, pdb):
    full = census.index_entries(index)
    by_name = {}
    for va, name, _, _, _ in full:
        by_name.setdefault(name, []).append(va)
    source = census.source_of(census.source_ranges(pdb), [e[0] for e in full])
    names = {e[0] for e in full}
    entries = [(va, name, source.get(va), size) for va, name, _, size, _ in full]
    by_doc = census.doc_backed(docs, by_name) & names
    by_sweep = census.sweep_backed(sim_src, by_name, os.path.join(HERE, "emu")) & names
    cites = doc_names(census.cited_per_doc(docs), docs)
    return mechanic_rows(entries, by_doc | by_sweep, by_doc, census.port_names(sim_src),
                         census.port_spellings(sim_src), cites)


def docs_cell(counter, top=3):
    """The documents citing a group's functions: top-level mechanic documents
    first, most cites first, then how many more (journal, audit, …)."""
    main = [d for d, _ in sorted(counter.items(), key=lambda kv: (-kv[1], kv[0]))
            if os.sep not in d and d not in LEDGERS]
    rest = len(counter) - min(top, len(main))
    cell = " ".join(main[:top] + ([f"+{rest}"] if rest > 0 else []))
    return cell or "-"


# The documents that cite functions without specifying a mechanic: the
# queue, the ledgers, the chronicle. Counted in `+N`, never named.
LEDGERS = frozenset(("QUEUE.md", "PARKED.md", "JOURNAL.md", "RUNS.md", "CENSUS.md", "DECISIONS.md",
                     "LOOP.md"))


def print_mechanics(rows, top):
    order = sorted(rows.items(), key=lambda kv: (-kv[1]["unbacked_bytes"], kv[0]))
    print("# 1. by mechanic — simulation functions by PDB source file (header with its .cpp), "
          "largest unbacked bytes first")
    print("# bytes: " + census.SIZE_HOW["decompile"] + "; 100 % = every function backed, dead weight, ui or operator")
    print(f"{'source':26} {'fns':>4} {'backed':>6} {'doc':>4} {'swp':>4} {'dead':>4} {'ui':>3} {'op':>3} "
          f"{'cut':>3} {'carr':>4} {'unmk':>4} {'bytes':>9} {'backed b':>9} {'share':>6}  docs")
    shown = order if top is None else order[:top]
    for _key, r in shown:
        label = "+".join(sorted(r["files"], key=lambda f: (not f.endswith(".cpp"), f)))
        share = f"{100 * r['backed_bytes'] / r['bytes']:.0f}%" if r["bytes"] else "-"
        mark = "*" if r["full"] else " "
        print(f"{label[:25]:25}{mark} {r['functions']:4} {r['backed']:6} {r['doc']:4} {r['sweep']:4} "
              f"{r['skip']:4} {r['ui']:3} {r['operator']:3} {r['cut']:3} {r['carried']:4} {r['unmarked']:4} "
              f"{r['bytes']:9} {r['backed_bytes']:9} {share:>6}  {docs_cell(r['docs'])}")
    if top is not None and len(order) > top:
        print(f"… {len(order) - top} more groups (--top)")
    s = mechanics_summary(rows)
    print(f"groups {s['groups']} · at 100 % {s['full']} (marked *) · functions backed {s['backed']} of "
          f"{s['functions']} · bytes backed {s['backed_bytes']} of {s['bytes']} ({s['byte_share']:.1f}%) · "
          f"unmarked {s['unmarked']} (the crate defines no function of the name)")


def mechanics_summary(rows):
    tot = Counter()
    for r in rows.values():
        for k in ("functions", "backed", "bytes", "backed_bytes", "unmarked"):
            tot[k] += r[k]
    return {"groups": len(rows), "full": sum(1 for r in rows.values() if r["full"]),
            **tot, "byte_share": 100 * tot["backed_bytes"] / tot["bytes"] if tot["bytes"] else 0.0}


# --- tables 2 and 3: the dumps ---------------------------------------------

SCAN = re.compile(rb"^ *BEGIN FRAME (-?\d+)\r?$"
                  rb"|^ *BEGIN LEADERDATA\r?\n *who (-?\d+)"
                  rb"|^ *leader_flags (-?\d+)"
                  rb"|^ *ages_get\(\) +(-?\d+)"
                  rb"|^ *BEGIN GUY\r?\n *type (-?\d+)", re.M)


def scan(buf):
    """One pass over a dump's bytes: `(leaders, types)` — `leaders` a list
    of `(frame, who, leader_flags, ages)`, `frame` `"start"` before the
    first `BEGIN FRAME`; `types` a Counter of every `GUY`'s type. A
    `leader_flags` belongs to the `LEADERDATA` block after it; a block's
    first `ages_get()` is its age (a block without one, below `LEADERS=2`,
    holds none)."""
    leaders, types = [], Counter()
    frame, flags, cur = "start", None, None
    for m in SCAN.finditer(buf):
        i = m.lastindex
        v = int(m.group(i))
        if i == 1:
            frame, flags, cur = v, None, None
        elif i == 2:
            cur, flags = (v, flags), None
        elif i == 3:
            flags = v
        elif i == 4:
            if cur is not None:
                leaders.append((frame, cur[0], cur[1], v))
                cur = None
        else:
            types[v] += 1
    return leaders, types


def dumps_for(traces, archive):
    """`(trace, [dump paths])` for every pinned trace: `rontrace-<run>.log`
    to `gamelog-<run>-*.txt` and `gamelog-<run>.txt`."""
    out = []
    for t in traces:
        m = re.match(r"rontrace-(.*)\.log$", t)
        if not m:
            continue
        stem = m.group(1)
        fs = sorted(set(glob.glob(os.path.join(glob.escape(archive), f"gamelog-{stem}-*.txt"))
                        + glob.glob(os.path.join(glob.escape(archive), f"gamelog-{stem}.txt"))))
        out.append((t, fs))
    return out


def walk(paths, limit=None):
    """Scan each dump once; `(per_age, types, read, leader_rows)`."""
    per_age = {a: {"human": set(), "ai": set(), "any": set()} for a in range(8)}
    other_ages = Counter()
    types = Counter()
    read = 0
    rows = 0
    for n, p in enumerate(paths if limit is None else paths[:limit]):
        size = os.path.getsize(p)
        print(f"rubric: [{n + 1}/{len(paths) if limit is None else min(limit, len(paths))}] "
              f"{os.path.basename(p)} ({size // 10**6} MB)", file=sys.stderr, flush=True)
        with open(p, "rb") as f:
            if size == 0:
                continue
            with mmap.mmap(f.fileno(), 0, access=mmap.ACCESS_READ) as mm:
                leaders, t = scan(mm)
        read += 1
        types.update(t)
        base = os.path.basename(p)
        for frame, _who, flags, age in leaders:
            rows += 1
            if flags is None or not flags & 1:
                continue
            if age not in per_age:
                other_ages[age] += 1
                continue
            key = (base, frame)
            per_age[age]["human" if flags & 4 else "ai"].add(key)
            per_age[age]["any"].add(key)
    return per_age, types, read, rows, other_ages


def print_ages(per_age, rows, read, other_ages):
    print("# 2. by age — (dump, frame) pairs with an active leader whose ages_get() is that age")
    print(f"{'age':>3} {'name':12} {'human':>8} {'computer':>8} {'either':>8}")
    for a in range(8):
        c = per_age[a]
        print(f"{a:3} {AGE_NAMES[a]:12} {len(c['human']):8} {len(c['ai']):8} {len(c['any']):8}"
              + ("   never held" if not c["any"] else ""))
    never = [a for a in range(8) if not per_age[a]["any"]]
    print(f"leader records with an age: {rows} in {read} dumps · ages never held: "
          + (",".join(map(str, never)) or "none")
          + (f" · out of range: {dict(other_ages)}" if other_ages else ""))


AGE_NAMES = ("Ancient", "Classical", "Medieval", "Gunpowder", "Enlightenment", "Industrial", "Modern",
             "Information")

UNIT = re.compile(r"<UNIT>(.*?)</UNIT>", re.S)


def tag(body, name):
    m = re.search(rf"<{name}>(.*?)</{name}>", body, re.S)
    return m.group(1).strip() if m else ""


def unit_rules(text):
    """`[(type id, name, graph, domain, cat)]` from `unitrules.xml`'s text,
    record N at type id 50 + N (`docs/FORMATS.md`)."""
    return [(UNIT_FIRST + i, tag(b, "NAME"), tag(b, "GRAPH"), tag(b, "DOMAIN"), tag(b, "CAT"))
            for i, b in enumerate(UNIT.findall(text))]


TYPE_AGE = re.compile(rb"^ *BEGIN UNITTYPE\r?$|^ *type (-?\d+)\r?$|^ *age (-?\d+)\r?$", re.M)


def loaded_ages(buf, want=UNIT_COUNT):
    """`{type id: age}` from the types dump's first copy of each
    `UNITTYPE` (its `TYPE`'s `type`, its `OBJECTTYPE`'s `age`)."""
    out, cur, open_ = {}, None, False
    for m in TYPE_AGE.finditer(buf):
        if m.lastindex is None:
            open_, cur = True, None
        elif m.lastindex == 1 and open_ and cur is None:
            cur = int(m.group(1))
        elif m.lastindex == 2 and open_ and cur is not None:
            out.setdefault(cur, int(m.group(2)))
            open_ = False
            if len(out) >= want:
                break
    return out


def print_types(rules, ages, types):
    ids = {r[0] for r in rules}
    seen = {t for t in types if t in ids}
    print(f"# 3. by unit type — GUY type ids in the dumps against unitrules.xml's {len(rules)} records")
    never = [r for r in rules if r[0] not in seen]
    print(f"seen {len(seen)} of {len(rules)} · never {len(never)} · ids outside the unit range seen: "
          f"{len({t for t in types if t not in ids})}")
    print("seen: " + ", ".join(f"{r[0]}:{r[1]}" + (f"[{r[2]}]" if r[2] and r[2].lower() != r[1].lower() else "")
                               + f" ×{types[r[0]]}" for r in rules if r[0] in seen))
    groups = defaultdict(list)
    for tid, name, graph, domain, cat in never:
        groups[(domain, ages.get(tid, "?"))].append(f"{tid}:{name}" + (f"[{graph}]" if graph and
                                                                       graph.lower() != name.lower() else ""))
    for (domain, age), items in sorted(groups.items(), key=lambda kv: (kv[0][0], str(kv[0][1]))):
        print(f"{domain or '?':6} age {age!s:>2} ({len(items):3}): " + ", ".join(items))
    return len(seen), len(never)


def install_dir(pdb):
    return os.path.dirname(os.path.dirname(os.path.abspath(pdb)))


LEDGER = os.path.join(HERE, "..", "docs", "audit", "rubric.tsv")
LEDGER_COLUMNS = ("date", "pass", "groups_full", "groups", "backed", "functions", "byte_share",
                  "ages_held", "ages_never", "types_seen", "types", "dumps")


def ledger_row(date, label, mech, per_age, seen, total_types, dumps):
    """One pass's rubric as a ledger row (Ramon, the forty-ninth steer: the
    rubric is shown at every steering pass against the pass before, never
    computed per landing). `mech` is `mechanics_summary`'s dict; `per_age`
    `walk`'s; `seen`/`total_types` table 3's two counts."""
    held = [x for x in range(8) if per_age[x]["any"]]
    never = ",".join(str(x) for x in range(8) if x not in held) or "none"
    return dict(date=date, **{"pass": label}, groups_full=mech["full"], groups=mech["groups"],
                backed=mech["backed"], functions=mech["functions"], byte_share=round(mech["byte_share"], 1),
                ages_held=len(held), ages_never=never, types_seen=seen, types=total_types, dumps=dumps)


def read_ledger(path):
    """The ledger's rows, oldest first; a missing file is no rows."""
    if not os.path.isfile(path):
        return []
    rows = []
    with open(path) as f:
        header = None
        for line in f:
            line = line.rstrip("\n")
            if not line or line.startswith("#"):
                continue
            cells = line.split("\t")
            if header is None:
                header = cells
                continue
            row = dict(zip(header, cells))
            for k in ("groups_full", "groups", "backed", "functions", "ages_held", "types_seen", "types", "dumps"):
                row[k] = int(row[k])
            row["byte_share"] = float(row["byte_share"])
            rows.append(row)
    return rows


def append_ledger(path, row):
    new = not os.path.isfile(path)
    with open(path, "a") as f:
        if new:
            f.write("# tools/rubric.py --record <pass>: one row per steering pass, oldest first (DECISIONS 88)\n")
            f.write("\t".join(LEDGER_COLUMNS) + "\n")
        f.write("\t".join(str(row[c]) for c in LEDGER_COLUMNS) + "\n")


def delta_table(prev, cur):
    """The steer's table: this pass against the one before, one line a
    number, the previous value in the last column — the shape the pass's
    record uses for waiting a landing."""
    def cell(row, k):
        if row is None:
            return "-"
        v = row[k]
        return f"{v:.1f}" if isinstance(v, float) else str(v)
    lines = [f"rubric: {cur['pass']} against {prev['pass'] if prev else 'no earlier pass'}",
             f"{'what':<34}{'now':>14}{'before':>14}"]
    for label, k in (("source groups at 100 %", "groups_full"), ("of groups", "groups"),
                     ("functions backed", "backed"), ("of functions", "functions"),
                     ("bytes backed, %", "byte_share"), ("ages held, of 8", "ages_held"),
                     ("ages never held", "ages_never"), ("unit types seen", "types_seen"),
                     ("of unit types", "types"), ("dumps read", "dumps")):
        lines.append(f"{label:<34}{cell(cur, k):>14}{cell(prev, k):>14}")
    return "\n".join(lines)


def main():
    ap = argparse.ArgumentParser(description="A steering pass's holistic scorecard: by mechanic, by age, "
                                             "by unit type. The module docstring has the rules.")
    ap.add_argument("--tables", default="1,2,3")
    ap.add_argument("--limit", type=int, help="read only the first N dumps (tables 2 and 3)")
    ap.add_argument("--dumps", nargs="*", help="these dumps instead of the pinned traces' own")
    ap.add_argument("--top", type=int, help="table 1: show only the first N groups")
    ap.add_argument("--index", default=os.path.expanduser("~/ghidra-projects/decomp/INDEX.tsv"))
    ap.add_argument("--docs", default=os.path.join(HERE, "..", "docs"))
    ap.add_argument("--sim-src", default=os.path.join(HERE, "..", "crates", "sim", "src"))
    ap.add_argument("--pdb", default=census.default_pdb(HERE))
    ap.add_argument("--archive", default=census.ARCHIVE)
    ap.add_argument("--record", metavar="PASS", help="append this run as PASS's row to the ledger "
                    "(docs/audit/rubric.tsv) and print the table against the row before; all three tables")
    ap.add_argument("--ledger", default=LEDGER)
    ap.add_argument("--delta", action="store_true", help="print the ledger's last row against the one before, "
                    "reading nothing")
    a = ap.parse_args()
    if a.delta:
        rows = read_ledger(a.ledger)
        if not rows:
            print(f"no rows in {a.ledger}")
            return 1
        print(delta_table(rows[-2] if len(rows) > 1 else None, rows[-1]))
        return 0
    tables = {int(t) for t in a.tables.split(",") if t.strip()}
    if a.record:
        tables = {1, 2, 3}
    parts = []
    mech = seen = nrules = None
    if 1 in tables:
        rows = mechanics_now(a.index, a.docs, a.sim_src, a.pdb)
        print_mechanics(rows, a.top)
        s = mech = mechanics_summary(rows)
        parts.append(f"mechanics {s['full']} of {s['groups']} source groups at 100 % "
                     f"(backed {s['backed']} of {s['functions']} fns, {s['byte_share']:.1f}% of bytes)")
        print()
    if tables & {2, 3}:
        if a.dumps:
            paths = list(a.dumps)
            unmatched = []
        else:
            matched = dumps_for(census.pinned_traces(), a.archive)
            paths = [p for _t, fs in matched for p in fs]
            unmatched = [t for t, fs in matched if not fs]
        if unmatched:
            print(f"# pinned traces with no gamelog dump in the archive: {' '.join(unmatched)}")
        per_age, types, read, nrows, other = walk(paths, a.limit)
        partial = f" (partial: {read} of {len(paths)} dumps, --limit)" if a.limit and a.limit < len(paths) else ""
        if 2 in tables:
            print_ages(per_age, nrows, read, other)
            print()
            held = [x for x in range(8) if per_age[x]["any"]]
            parts.append(f"ages held {len(held)} of 8 (never: {','.join(str(x) for x in range(8) if x not in held) or 'none'})"
                         + partial)
        if 3 in tables:
            inst = install_dir(a.pdb)
            xml = os.path.join(inst, "data", "unitrules.xml")
            if not os.path.isfile(xml):
                print(f"# 3. by unit type — no unitrules.xml at {xml}; ids seen: {len(types)}")
                parts.append(f"unit type ids seen {len(types)} (no unitrules.xml)")
            else:
                with open(xml, errors="replace") as f:
                    rules = unit_rules(f.read())
                ages = {}
                tdump = os.path.join(a.archive, "gamelog-run3-fulldump-types.txt")
                if os.path.isfile(tdump):
                    with open(tdump, "rb") as f, mmap.mmap(f.fileno(), 0, access=mmap.ACCESS_READ) as mm:
                        ages = loaded_ages(mm)
                seen, never = print_types(rules, ages, types)
                nrules = len(rules)
                parts.append(f"unit types seen {seen} of {len(rules)} ({never} never)" + partial)
            print()
    print("Rubric: " + " · ".join(parts))
    if a.record:
        if mech is None or seen is None or a.limit:
            print("--record needs all three tables whole (no --limit, unitrules.xml present)")
            return 1
        import datetime
        row = ledger_row(datetime.date.today().isoformat(), a.record, mech, per_age, seen, nrules, len(paths))
        before = read_ledger(a.ledger)
        append_ledger(a.ledger, row)
        print()
        print(delta_table(before[-1] if before else None, row))
    return 0


if __name__ == "__main__":
    sys.exit(main())
