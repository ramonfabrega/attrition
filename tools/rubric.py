#!/usr/bin/env python3
r"""rubric.py — a steering pass's holistic scorecard of how much of the
original's simulation this port has verified.

    rubric.py [--tables 1,2,3,4] [--limit N] [--dumps PATH ... | --traces] [--top N]
    rubric.py --sources                         # every pinned word's game, the lockstep runs and files
    rubric.py --record "the fiftieth pass"     # at a steer: append the ledger's row, print it against the last
    rubric.py --delta                           # the last two ledger rows as a table, reading no dump

Four tables, each from data the tree (or the machine's own archive) already
holds, and a one-line `Rubric:` summary at the end.

**Lockstep** (the fifty-first pass, Ramon 2026-10-22: the rubric answers
how close the port is to a 1:1 sim). Tables 2, 3 and 4 read only **lockstep
games**: a game whose pinned word stands at the game's full length.
`lockstep_runs` reads the words from the Rust source — every `Lobby`/`Pair`
literal under `crates/rondata/src/diff/` (`battery::BATTERY`, the word
modules, the pairs: word `min(count, sequence)`, its start and long
captures), every `AI_WORDS` row whose word is not a struct's (its run from
the `endpoint` test's `run<N>_` prefix, else the const's comment, "every
frame of run<N>'s trace"), and the `LONG_WORD_*` consts (run and length from
that comment); then each `blind::TRACES` entry whose comment calls it
`run<N>'s game on every frame` of a whole game's run — never "… before
<frame>", a staged take that parts — joins as that game's twin. `FLOORS`
carries no run or length and is the long words' games. `--traces` reads the
pinned coverage traces' dumps instead (the definition before), `--dumps`
the dumps given. It is a measurement
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
2. **By age.** The lockstep games' dumps in the archive
   (`$RON_GAMELOG_DIR`, `census.ARCHIVE`): a struct's named captures, a
   run's `gamelog-<run>-*.txt` otherwise (a `-refused` take is not the
   game). Each `LEADERDATA`
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
   **The denominator is the skirmish types**: a record whose `TRIBE_MASK`
   is all zero — no nation's bit, so `LeaderData::tribe_can_type` answers 0
   for every player (`docs/TECH.md`, "Nation variants") — is excluded
   unless the types dump files it as gaia (`age` -1); the excluded list is
   printed, and any of them seen in a dump would falsify the criterion.
4. **Bytes in lockstep.** Of the simulation layer's functions
   (`census.layer_of`), how many and how many bytes the lockstep games'
   `rontrace` logs entered — `census.entered_in | census.hooked_in`, the
   census's own `entered` reader — in integer tenths of a percent.

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


def print_types(rules, ages, types, excluded=(), criterion="", source=""):
    """Table 3. `excluded` the type ids out of a skirmish's reach
    (`no_tribe_types`): the denominator is the rest; an excluded type seen
    in a dump is printed, since it would falsify the criterion."""
    all_ids = {r[0] for r in rules}
    out = set(excluded)
    rules_in = [r for r in rules if r[0] not in out]
    ids = {r[0] for r in rules_in}
    seen = {t for t in types if t in ids}
    print(f"# 3. by unit type — GUY type ids in the {source or 'given'} dumps against unitrules.xml's "
          f"{len(rules)} records")
    never = [r for r in rules_in if r[0] not in seen]
    print(f"seen {len(seen)} of {len(rules_in)} skirmish types ({len(out)} Conquer-the-World/scenario-only "
          f"excluded by {criterion or 'nothing'}) · never {len(never)} · ids outside the unit range seen: "
          f"{len({t for t in types if t not in all_ids})}")
    if out:
        print("excluded: " + ", ".join(f"{r[0]}:{r[1]}" + (f"[{r[2]}]" if r[2] and r[2].lower() != r[1].lower()
                                                           else "") for r in rules if r[0] in out))
        hit = sorted(t for t in types if t in out)
        print("excluded and seen (would falsify the criterion): " + (", ".join(map(str, hit)) or "none"))
    print("seen: " + ", ".join(f"{r[0]}:{r[1]}" + (f"[{r[2]}]" if r[2] and r[2].lower() != r[1].lower() else "")
                               + f" ×{types[r[0]]}" for r in rules if r[0] in seen))
    groups = defaultdict(list)
    for tid, name, graph, domain, cat in never:
        groups[(domain, ages.get(tid, "?"))].append(f"{tid}:{name}" + (f"[{graph}]" if graph and
                                                                       graph.lower() != name.lower() else ""))
    for (domain, age), items in sorted(groups.items(), key=lambda kv: (kv[0][0], str(kv[0][1]))):
        print(f"{domain or '?':6} age {age!s:>2} ({len(items):3}): " + ", ".join(items))
    return len(seen), len(never), len(rules_in)


# --- the lockstep source (the fifty-first pass, Ramon 2026-10-22) -----------
#
# A dump counts only when its game is one this port matched frame for frame:
# a game whose pinned word stands at the game's full length. The words are
# the Rust consts the handoff's lines are checked against, read from their
# source the way `census.pinned_traces` reads `blind.rs`.

DIFF_SRC = os.path.join(HERE, "..", "crates", "rondata", "src", "diff")
BLIND_SRC = os.path.join(HERE, "..", "crates", "rondata", "src", "blind.rs")

STRUCT = re.compile(r"(?:const (\w+): (?:Lobby|Pair) = )?\b(?:Lobby|Pair) \{\n(.*?)\n\s*\}", re.S)
AIWORD = re.compile(r"\bAiWord \{\n(.*?)\n\s*\}", re.S)
RUN = re.compile(r"\b(run\d+[a-z]?)\b")
FILE_RUN = re.compile(r"^(?:gamelog|rontrace)-(run\d+[a-z]?)[-.]")


def strip_comments(text):
    """Rust source without its `//` comments (a file name holds no `//`)."""
    return re.sub(r"//[^\n]*", "", text)


def rust_int(s):
    return int(s.replace("_", ""))


def struct_fields(body):
    """A `Lobby`/`Pair` literal's fields, or None when it is not one (the
    struct's own definition, a literal without a name)."""
    body = strip_comments(body)
    name = re.search(r'\bname: "([^"]+)"', body)
    long_ = re.search(r'\blong: \(\s*"([^"]+)",\s*"([^"]+)",?\s*\)', body)
    if not name or not long_:
        return None
    out = {"name": name.group(1), "long": (long_.group(1), long_.group(2))}
    start = re.search(r'\bstart: "([^"]+)"', body)
    out["start"] = start.group(1) if start else None
    for k in ("length", "count", "sequence"):
        m = re.search(rf"\b{k}: ([\d_]+),", body)
        if not m:
            return None
        out[k] = rust_int(m.group(1))
    return out


def lobby_tag(fname):
    """A capture's lobby as the runner names its file: the part after the
    run, without the capture's kind — `greatlakes-spanish-diff4` for both
    `gamelog-run1008-greatlakes-spanish-diff4-start.txt` and
    `gamelog-run1008b-greatlakes-spanish-diff4-24k-trace.txt`."""
    s = re.sub(r"^gamelog-run\d+[a-z]?-", "", fname)
    return re.sub(r"(-start|-24k-trace|-trace)?\.txt$", "", s)


def file_run(fname):
    m = FILE_RUN.match(fname)
    return m.group(1) if m else None


def doc_above(text, at):
    """The `///` lines directly above the line holding offset `at`, joined."""
    lines = text[:text.rfind("\n", 0, at) + 1].splitlines()
    out = []
    while lines and lines[-1].lstrip().startswith("///"):
        out.append(lines.pop().lstrip()[3:].strip())
    return " ".join(reversed(out))


EVERY_FRAME = re.compile(r"every frame of (run\d+[a-z]?)'s trace, 0\.\.(\d+)")


def const_value(name, texts, prefer=None):
    """`(value, struct const name or None, file)` of `const NAME: i64 = …;`:
    an integer literal, or the lower of a struct's two partings (the
    `if X.count < X.sequence` shape every word module writes)."""
    order = ([prefer] if prefer in texts else []) + [f for f in sorted(texts) if f != prefer]
    for f in order:
        m = re.search(rf"\bconst {name}: i64 = (.*?);\n", texts[f], re.S)
        if not m:
            continue
        expr = strip_comments(m.group(1)).strip()
        if re.fullmatch(r"[\d_]+", expr):
            return rust_int(expr), None, f
        s = re.match(r"if (\w+)\.count < \1\.sequence", expr)
        if s:
            return None, s.group(1), f
        return None, None, f
    return None, None, None


def lockstep_games(texts, blind_text):
    """Every pinned word's game: `(games, unresolved)`. `texts` is
    `{file name: source}` of `crates/rondata/src/diff/*.rs`; `blind_text`
    is `blind.rs`. A game is `{name, line, word, length, whole, runs,
    files, source}`: `runs` the run stems its captures carry, `files` the
    dump and trace names a struct gives (None: glob the runs), `whole` its
    word at its length. Three sources, deduplicated by game:

    1. every `Lobby`/`Pair` literal (`battery::BATTERY`, the word modules,
       the pairs): the word is `min(count, sequence)`, the runs the long
       capture's and the start's when the start is the same lobby
       (`lobby_tag`; a battery lobby borrows another's start);
    2. every `AI_WORDS` row whose word is not a struct's: the word const
       resolved, the run from its `endpoint` test's `run<N>_` prefix or
       else the const's comment, `every frame of run<N>'s trace`;
    3. every `LONG_WORD_*` const: its run and length (`0..<last>`) from
       the same comment.

    Then the pinned traces (`blind::TRACES`) whose comment block says
    `run<N>'s game` of a whole game's run join as that game's `twins`."""
    games, unresolved, by_struct = [], [], {}
    for f in sorted(texts):
        for m in STRUCT.finditer(texts[f]):
            fl = struct_fields(m.group(2))
            if fl is None:
                continue
            long_dump, long_trace = fl["long"]
            runs = [file_run(long_dump)]
            files = [long_dump, long_trace]
            if fl["start"] and lobby_tag(fl["start"]) == lobby_tag(long_dump):
                runs.insert(0, file_run(fl["start"]))
                files.insert(0, fl["start"])
                files.append(f"rontrace-{file_run(fl['start'])}.log")
            word = min(fl["count"], fl["sequence"])
            g = dict(name=fl["name"], line="", word=word, length=fl["length"], whole=word >= fl["length"],
                     runs=runs, files=files, source=f"{f} {m.group(1) or 'literal'}", twins=[])
            games.append(g)
            if m.group(1):
                by_struct[(f, m.group(1))] = g
    names = {g["name"] for g in games}
    kit = texts.get("testkit.rs", "")
    for m in AIWORD.finditer(kit):
        body = strip_comments(m.group(1))
        get = lambda k: (re.search(rf'\b{k}: ([^,\n]+),', body) or [None, None])[1]
        line, gmap, expr, length = (get("line") or "").strip('"'), (get("map") or "").strip('"'), get("word"), get("length")
        endpoint = re.search(r'endpoint: Some\("(\w+)"\)', body)
        if not re.search(r'\bmap: "', body):
            continue  # the struct's definition or its impl, not a row
        if gmap in names:
            for g in games:
                if g["name"] == gmap:
                    g["line"] = line
            continue
        parts = (expr or "").split("::")
        const, module = parts[-1], (parts[-2] + ".rs" if len(parts) > 1 else "testkit.rs")
        value, struct, where = const_value(const, texts, module)
        if struct is not None and (where, struct) in by_struct:
            by_struct[(where, struct)]["line"] = line
            continue
        run = None
        if endpoint and re.match(r"run\d+[a-z]?_", endpoint.group(1)):
            run = endpoint.group(1).split("_")[0]
        elif where:
            at = re.search(rf"\bconst {const}: i64", texts[where]).start()
            ev = EVERY_FRAME.findall(doc_above(texts[where], at))
            run = ev[-1][0] if ev else None
        if value is None or run is None or length is None:
            unresolved.append(f"{line or '?'} {gmap or '?'} ({expr})")
            continue
        length = rust_int(length)
        games.append(dict(name=gmap, line=line, word=value, length=length, whole=value >= length, runs=[run],
                          files=None, source=f"testkit.rs AI_WORDS {const}", twins=[]))
    for m in re.finditer(r"\bconst (LONG_WORD_\w+): i64 = ([\d_]+);", kit):
        ev = EVERY_FRAME.findall(doc_above(kit, m.start()))
        if not ev:
            unresolved.append(f"{m.group(1)} (no `every frame of run<N>'s trace` in its comment)")
            continue
        run, last = ev[-1]
        if any(run in g["runs"] for g in games):
            continue
        value, length = rust_int(m.group(2)), int(last) + 1
        games.append(dict(name=m.group(1), line="Long captures", word=value, length=length,
                          whole=value >= length, runs=[run], files=None, source=f"testkit.rs {m.group(1)}",
                          twins=[]))
    whole_runs = {r: g for g in games if g["whole"] for r in g["runs"]}
    for entries, comment in trace_blocks(blind_text):
        # "run847's game on every frame" — never "… on every frame before
        # 1387" (a staged take that parts there) nor "run652's game before
        # 601": those dumps hold the staging, not the game.
        named = {r for chunk in re.findall(r"((?:\brun\d+[a-z]?'s,?\s+(?:and\s+)?)+)game on every frame(?!\s+before)",
                                           comment)
                 for r in RUN.findall(chunk)}
        hit = sorted(r for r in named if r in whole_runs)
        if not hit:
            continue
        g = whole_runs[hit[0]]
        for e in entries:
            run = re.match(r"rontrace-(.*)\.log$", e)
            if run and run.group(1) not in g["runs"] and run.group(1) not in g["twins"]:
                g["twins"].append(run.group(1))
    return games, unresolved


def trace_blocks(blind_text):
    """`blind::TRACES` as `[(entries, comment)]`: each run of entries with
    the comment lines written above them (a comment after an entry opens
    the next block)."""
    body = blind_text[blind_text.index("pub const TRACES"):]
    body = body[:body.index("\n];")]
    blocks, comment, entries = [], [], []
    for line in body.splitlines()[1:]:
        s = line.strip()
        if s.startswith("//"):
            if entries:
                blocks.append((entries, " ".join(comment)))
                comment, entries = [], []
            comment.append(s[2:].strip())
            continue
        m = re.match(r'"([^"]+)",', s)
        if m:
            entries.append(m.group(1))
    if entries:
        blocks.append((entries, " ".join(comment)))
    return blocks


def lockstep_runs_of(games):
    """The run stems of every whole game, its twins included, sorted."""
    return sorted({r for g in games if g["whole"] for r in g["runs"] + g["twins"]},
                  key=lambda r: (int(re.match(r"run(\d+)", r).group(1)), r))


def lockstep_runs(diff_src=DIFF_SRC, blind_src=BLIND_SRC):
    """`(runs, games, unresolved)` from the tree's own source."""
    texts = {}
    for p in sorted(glob.glob(os.path.join(diff_src, "*.rs"))):
        with open(p) as f:
            texts[os.path.basename(p)] = f.read()
    with open(blind_src) as f:
        blind = f.read()
    games, unresolved = lockstep_games(texts, blind)
    return lockstep_runs_of(games), games, unresolved


def run_dumps(run, archive):
    """`gamelog-<run>-*.txt` and `gamelog-<run>.txt` — a take the runner
    refused (`-refused`, another lobby's) is not the run's game."""
    fs = set(glob.glob(os.path.join(glob.escape(archive), f"gamelog-{run}-*.txt"))
             + glob.glob(os.path.join(glob.escape(archive), f"gamelog-{run}.txt")))
    return sorted(p for p in fs if "-refused" not in os.path.basename(p))


def lockstep_files(games, archive):
    """`(dumps, traces)` on disk for the whole games: a struct's named
    captures, a run-only game's and a twin's by run stem."""
    dumps, traces = set(), set()
    for g in games:
        if not g["whole"]:
            continue
        if g["files"] is not None:
            for n in g["files"]:
                p = os.path.join(archive, n)
                if os.path.isfile(p):
                    (traces if n.startswith("rontrace-") else dumps).add(p)
        else:
            for r in g["runs"]:
                dumps.update(run_dumps(r, archive))
        for r in (g["runs"] if g["files"] is None else []) + g["twins"]:
            p = os.path.join(archive, f"rontrace-{r}.log")
            if os.path.isfile(p):
                traces.add(p)
        for r in g["twins"]:
            dumps.update(run_dumps(r, archive))
    return sorted(dumps), sorted(traces)


def print_sources(games, unresolved, runs, dumps, traces):
    print("# lockstep sources — every pinned word's game; whole = its word at its length")
    for g in sorted(games, key=lambda g: (not g["whole"], g["name"])):
        print(f"{'whole' if g['whole'] else 'open ':5} {g['name']:30} w{g['word']} of {g['length']:<6} "
              f"runs {' '.join(g['runs'])}" + (f" · twins {' '.join(g['twins'])}" if g["twins"] else "")
              + f"  [{g['line'] or '-'}; {g['source']}]")
    if unresolved:
        print("unresolved: " + "; ".join(unresolved))
    print(f"lockstep runs {len(runs)}: {' '.join(runs)}")
    print(f"lockstep dumps on disk {len(dumps)} · traces {len(traces)}")


# --- table 4: bytes in lockstep --------------------------------------------


def tenths(t):
    return f"{t // 10}.{t % 10}"


def lockstep_share(sizes, source, entered):
    """Table 4's arithmetic: of the simulation layer's functions (`source`
    `{va: PDB path}`, `census.layer_of`), how many and how many bytes
    (`sizes`) are in `entered`, through `census.entered_rows`; the byte
    share in integer tenths of a percent, floored."""
    sim = [va for va, path in source.items() if census.layer_of(path) == "simulation"]
    ent = census.entered_rows({}, sizes, source, set(), set(), entered)["layers"].get(
        "simulation", census.EMPTY_ENTERED)
    total = sum(sizes.get(va, 0) for va in sim)
    return dict(functions=len(sim), entered=ent["entered"], bytes=total, entered_bytes=ent["entered_bytes"],
                bytes_tenths=(1000 * ent["entered_bytes"] // total) if total else 0)


def lockstep_now(index, pdb, traces):
    full = census.index_entries(index)
    sizes = {va: size for va, _n, _, size, _ in full}
    source = census.source_of(census.source_ranges(pdb), list(sizes))
    entered = (census.entered_in(traces, index) | census.hooked_in(traces, index)) if traces else set()
    return lockstep_share(sizes, source, entered)


def print_lockstep(s, ntraces):
    print("# 4. bytes in lockstep — simulation functions a whole game's trace entered "
          "(census `entered_in | hooked_in`)")
    print(f"lockstep entered {s['entered']} of {s['functions']} simulation functions · "
          f"{s['entered_bytes']} of {s['bytes']} bytes ({tenths(s['bytes_tenths'])}%) · {ntraces} traces")


# --- the denominator of table 3 ---------------------------------------------


def no_tribe_types(text, ages):
    """The type ids no skirmish trains: `TRIBE_MASK` all zero — no
    nation's bit, so `LeaderData::tribe_can_type` answers 0 for every
    player and `get_graft` never names it for another type
    (`docs/TECH.md`, "Nation variants"; `docs/FORMATS.md`, `TRIBE_MASK`)
    — unless the loader files it as gaia (`age` -1 in the types dump, the
    map's animals)."""
    out = []
    for i, b in enumerate(UNIT.findall(text)):
        mask = tag(b, "TRIBE_MASK")
        if mask and set(mask) <= {"0"} and ages.get(UNIT_FIRST + i) != -1:
            out.append(UNIT_FIRST + i)
    return out


def install_dir(pdb):
    return os.path.dirname(os.path.dirname(os.path.abspath(pdb)))


LEDGER = os.path.join(HERE, "..", "docs", "audit", "rubric.tsv")
LEDGER_COLUMNS = ("date", "pass", "groups_full", "groups", "backed", "functions", "byte_share",
                  "ages_held", "ages_never", "types_seen", "types", "dumps",
                  # the fifty-first pass's lockstep definition: which dumps tables 2 and 3 read,
                  # and table 4; a row recorded before it carries empty cells
                  "types_source", "lockstep_entered", "lockstep_functions", "lockstep_bytes")
INT_COLUMNS = ("groups_full", "groups", "backed", "functions", "ages_held", "types_seen", "types", "dumps",
               "lockstep_entered", "lockstep_functions")
FLOAT_COLUMNS = ("byte_share", "lockstep_bytes")


def ledger_row(date, label, mech, per_age, seen, total_types, dumps, source="traces", lockstep=None):
    """One pass's rubric as a ledger row (Ramon, the forty-ninth steer: the
    rubric is shown at every steering pass against the pass before, never
    computed per landing). `mech` is `mechanics_summary`'s dict; `per_age`
    `walk`'s; `seen`/`total_types` table 3's two counts; `source` the dumps
    tables 2 and 3 read; `lockstep` table 4's `lockstep_share`, or None."""
    held = [x for x in range(8) if per_age[x]["any"]]
    never = ",".join(str(x) for x in range(8) if x not in held) or "none"
    ls = lockstep or {}
    return dict(date=date, **{"pass": label}, groups_full=mech["full"], groups=mech["groups"],
                backed=mech["backed"], functions=mech["functions"], byte_share=round(mech["byte_share"], 1),
                ages_held=len(held), ages_never=never, types_seen=seen, types=total_types, dumps=dumps,
                types_source=source, lockstep_entered=ls.get("entered"), lockstep_functions=ls.get("functions"),
                lockstep_bytes=float(tenths(ls["bytes_tenths"])) if "bytes_tenths" in ls else None)


def read_ledger(path):
    """The ledger's rows, oldest first; a missing file is no rows, and a
    cell a row's header lacks (or leaves empty) is None."""
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
            row = {c: None for c in LEDGER_COLUMNS}
            row.update({k: (v if v != "" else None) for k, v in zip(header, cells)})
            for k in INT_COLUMNS:
                if row.get(k) is not None:
                    row[k] = int(row[k])
            for k in FLOAT_COLUMNS:
                if row.get(k) is not None:
                    row[k] = float(row[k])
            rows.append(row)
    return rows


def append_ledger(path, row):
    """Append `row`; a ledger whose header predates a column is rewritten
    under the current header first, its rows' new cells empty."""
    cell = lambda v: "" if v is None else str(v)
    if not os.path.isfile(path):
        with open(path, "w") as f:
            f.write("# tools/rubric.py --record <pass>: one row per steering pass, oldest first (DECISIONS 88)\n")
            f.write("\t".join(LEDGER_COLUMNS) + "\n")
    with open(path) as f:
        lines = f.read().splitlines()
    head = next((i for i, l in enumerate(lines) if l and not l.startswith("#")), None)
    if head is not None and lines[head].split("\t") != list(LEDGER_COLUMNS):
        old = lines[head].split("\t")
        out = lines[:head] + ["\t".join(LEDGER_COLUMNS)]
        for l in lines[head + 1:]:
            if not l or l.startswith("#"):
                out.append(l)
                continue
            d = dict(zip(old, l.split("\t")))
            out.append("\t".join(d.get(c, "") for c in LEDGER_COLUMNS))
        with open(path, "w") as f:
            f.write("\n".join(out) + "\n")
    with open(path, "a") as f:
        f.write("\t".join(cell(row.get(c)) for c in LEDGER_COLUMNS) + "\n")


def delta_table(prev, cur):
    """The steer's table: this pass against the one before, one line a
    number, the previous value in the last column — the shape the pass's
    record uses for waiting a landing. A cell an older row lacks is `-`."""
    def cell(row, k):
        if row is None or row.get(k) is None:
            return "-"
        v = row[k]
        return f"{v:.1f}" if isinstance(v, float) else str(v)
    lines = [f"rubric: {cur['pass']} against {prev['pass'] if prev else 'no earlier pass'}",
             f"{'what':<34}{'now':>14}{'before':>14}"]
    for label, k in (("source groups at 100 %", "groups_full"), ("of groups", "groups"),
                     ("functions backed", "backed"), ("of functions", "functions"),
                     ("bytes backed, %", "byte_share"),
                     ("lockstep bytes, %", "lockstep_bytes"), ("lockstep functions entered", "lockstep_entered"),
                     ("of simulation functions", "lockstep_functions"),
                     ("dumps read from", "types_source"),
                     ("ages held, of 8", "ages_held"),
                     ("ages never held", "ages_never"), ("unit types seen", "types_seen"),
                     ("of unit types", "types"), ("dumps read", "dumps")):
        lines.append(f"{label:<34}{cell(cur, k):>14}{cell(prev, k):>14}")
    return "\n".join(lines)


def main():
    ap = argparse.ArgumentParser(description="A steering pass's holistic scorecard: by mechanic, by age, "
                                             "by unit type, bytes in lockstep. The module docstring has the rules.")
    ap.add_argument("--tables", default="1,2,3,4")
    ap.add_argument("--limit", type=int, help="read only the first N dumps (tables 2 and 3)")
    src = ap.add_mutually_exclusive_group()
    src.add_argument("--dumps", nargs="*", help="tables 2 and 3: these dumps instead of the lockstep games'")
    src.add_argument("--traces", action="store_true", help="tables 2 and 3: the pinned coverage traces' dumps "
                     "(rondata::blind::TRACES), the definition before the fifty-first pass")
    ap.add_argument("--sources", action="store_true", help="print every pinned word's game, the lockstep runs "
                    "and their files, then stop")
    ap.add_argument("--top", type=int, help="table 1: show only the first N groups")
    ap.add_argument("--index", default=os.path.expanduser("~/ghidra-projects/decomp/INDEX.tsv"))
    ap.add_argument("--docs", default=os.path.join(HERE, "..", "docs"))
    ap.add_argument("--sim-src", default=os.path.join(HERE, "..", "crates", "sim", "src"))
    ap.add_argument("--pdb", default=census.default_pdb(HERE))
    ap.add_argument("--archive", default=census.ARCHIVE)
    ap.add_argument("--record", metavar="PASS", help="append this run as PASS's row to the ledger "
                    "(docs/audit/rubric.tsv) and print the table against the row before; all four tables")
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
    runs, games, unresolved = lockstep_runs()
    ls_dumps, ls_traces = lockstep_files(games, a.archive)
    if a.sources:
        print_sources(games, unresolved, runs, ls_dumps, ls_traces)
        return 0
    tables = {int(t) for t in a.tables.split(",") if t.strip()}
    if a.record:
        tables = {1, 2, 3, 4}
    parts = {}
    mech = seen = ntypes = share = None
    source = "lockstep"
    if 1 in tables:
        rows = mechanics_now(a.index, a.docs, a.sim_src, a.pdb)
        print_mechanics(rows, a.top)
        s = mech = mechanics_summary(rows)
        parts[1] = (f"mechanics {s['full']} of {s['groups']} source groups at 100 % "
                    f"(backed {s['backed']} of {s['functions']} fns, {s['byte_share']:.1f}% of bytes)")
        print()
    if 4 in tables:
        share = lockstep_now(a.index, a.pdb, ls_traces)
        print_lockstep(share, len(ls_traces))
        parts[4] = f"lockstep bytes {tenths(share['bytes_tenths'])} % ({share['entered']} of {share['functions']} fns)"
        print()
    if tables & {2, 3}:
        if a.dumps:
            paths, unmatched, source = list(a.dumps), [], "given"
        elif a.traces:
            matched = dumps_for(census.pinned_traces(), a.archive)
            paths = [p for _t, fs in matched for p in fs]
            unmatched = [t for t, fs in matched if not fs]
            source = "traces"
        else:
            paths, unmatched = ls_dumps, []
            print(f"# tables 2 and 3 read the lockstep games' dumps: {len(paths)} of {len(runs)} runs' "
                  f"(`--sources` lists them)")
        if unmatched:
            print(f"# pinned traces with no gamelog dump in the archive: {' '.join(unmatched)}")
        per_age, types, read, nrows, other = walk(paths, a.limit)
        partial = f" (partial: {read} of {len(paths)} dumps, --limit)" if a.limit and a.limit < len(paths) else ""
        if 2 in tables:
            print_ages(per_age, nrows, read, other)
            print(f"source: {source} dumps")
            print()
            held = [x for x in range(8) if per_age[x]["any"]]
            parts[2] = (f"ages held {len(held)} of 8 (never: "
                        f"{','.join(str(x) for x in range(8) if x not in held) or 'none'}; {source})" + partial)
        if 3 in tables:
            inst = install_dir(a.pdb)
            xml = os.path.join(inst, "data", "unitrules.xml")
            if not os.path.isfile(xml):
                print(f"# 3. by unit type — no unitrules.xml at {xml}; ids seen: {len(types)}")
                parts[3] = f"unit type ids seen {len(types)} (no unitrules.xml)"
            else:
                with open(xml, errors="replace") as f:
                    text = f.read()
                rules = unit_rules(text)
                ages = {}
                tdump = os.path.join(a.archive, "gamelog-run3-fulldump-types.txt")
                if os.path.isfile(tdump):
                    with open(tdump, "rb") as f, mmap.mmap(f.fileno(), 0, access=mmap.ACCESS_READ) as mm:
                        ages = loaded_ages(mm)
                excluded = no_tribe_types(text, ages)
                seen, never, ntypes = print_types(rules, ages, types, excluded, CRITERION, source)
                parts[3] = (f"unit types seen {seen} of {ntypes} ({never} never; {len(excluded)} excluded; "
                            f"{source})" + partial)
            print()
    print("Rubric: " + " · ".join(parts[k] for k in (1, 4, 2, 3) if k in parts))
    if a.record:
        if mech is None or seen is None or share is None or a.limit:
            print("--record needs all four tables whole (no --limit, unitrules.xml present)")
            return 1
        import datetime
        row = ledger_row(datetime.date.today().isoformat(), a.record, mech, per_age, seen, ntypes, len(paths),
                         source=source, lockstep=share)
        before = read_ledger(a.ledger)
        append_ledger(a.ledger, row)
        print()
        print(delta_table(before[-1] if before else None, row))
    return 0


CRITERION = "TRIBE_MASK 0, not gaia"


if __name__ == "__main__":
    sys.exit(main())
