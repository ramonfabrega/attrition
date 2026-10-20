#!/usr/bin/env python3
r"""census.py — the executable as the denominator.

    census.py [--index INDEX.tsv] [--docs docs/] [--top N] [--never] [--pin] [<rontrace.log> ...]
    census.py --layers [--json]
    census.py --unbacked [--by shortest|bytes] [--top N] [--all] [--claimed DIR]
    census.py --backed-list [--write PATH]
    census.py --check-backed PATH

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

**`--layers`: the census by layer, with the backed column** (item 1467,
parked since DECISIONS 41; DECISIONS 63 (iv) makes it the sweep lane's
score). Every function the export lists is filed under a **layer** by its
PDB source path — the file its line record starts in, read from
`llvm-pdbutil dump --l` of `rise.pdb` (section 1's offset 0 is VA
0x401000) — and marked **backed** or not. Prints one row per layer and
the board's number, the simulation layer's backed share;
`--layers --json` prints the same as JSON for `rondata::census`'s pin.

The layer rule, first match wins, on the lower-cased source path:

1. outside `…\main\` (the compiler's runtime, SDK headers), or under a
   third-party root (`zlib`, `pnglib`, `packages`, `cellsdk`,
   `steamworks_sdk`, `cpclib`, `crossplaynetlib`, `game\minizip`) →
   **engine**; so are `main\basic` and `main\bighuge`;
2. `main\game\script\` (the BHS script VM) and the script API and AI
   files at `game\` (`LAYER_AI`) → **AI**;
3. a `game\` file whose stem `INTERFACE` matches (windows, menus,
   editors, chat, GameSpy, Conquer the World) → **interface**;
4. a `game\` file whose stem `ENGINE` matches (rendering, sound,
   platform, network, logging, saving) → **engine**;
5. any other `game\` file → **simulation**, listed by `--layers` so a
   misfiled file is visible;
6. anything else under `main\`, and a function with no line record
   (28,000-odd of `_global`'s thunks and runtime) → **unknown**, counted
   and listed, never in a denominator.

A function is **backed** when either:

* **a document's coverage section says a diff reaches it**: inside a
  `docs/*.md` section whose heading names coverage, a paragraph that opens
  on a bold marker naming a compare against the original (`**Diff-backed**`,
  `**Dump-backed**`, `**Oracle-backed**`, `**Packet-backed**`) starts a
  backed span, one naming a reading, a listing, a unit test or "not"
  (`**Reading-only**`, `**Listing-backed**`, `**Unit-backed only**`,
  `**Not established**`) ends it, and every `name@00xxxxxx` cited in the span —
  or `Class::method` written whole and naming one function of the export —
  counts; or
* **a sweep `#[test]` in `crates/sim` names it**: a test whose doc comment
  or body runs the original under the emulator (`tools/emu/`,
  `tools/recomp/`, "emulat…") and cites the function the same two ways.

What it cannot count: a function a diff reaches that no coverage section
cites by address or by its whole name (a bare `find_upath` is not
resolved), a function a document calls diff-backed outside a coverage
section, and whether the diff checked the function's predicate rather
than merely passing through it. It is a floor of what is backed, read
from what the documents assert.

**Bytes beside the count.** `--layers` weighs each layer twice: by
function, as above, and by **bytes** — the `bytes`, `backed bytes` and
`byte share` columns, and a `Census bytes:` line after the `Census:` line
(whose text the handoff's guard parses, so it is left as it was). A
function's size is the PDB symbol's when `INDEX.tsv` carries one in a
fourth column, else the decompile file's byte count (`funcs/…@addr.c`) —
a proxy for code size, not code size; the table's header says which. A
count weighs a two-line getter as much as a pathfinder, and the
`--unbacked` list was shortest-first, so the count ran well ahead of the
bytes (the pass's reader: 43.8 % of the functions, about 23.5 % of the
bytes). `--unbacked --by bytes` lists largest-first; the default stays
shortest-first.

**The backed list.** `--backed-list` prints every backed simulation
function, one per line, by address: `ADDR<tab>Class::method<tab>by`,
where `by` is `doc` when a coverage section's diff backs it and `sweep`
otherwise; `--write PATH` writes it with a dated header naming the tree
(`docs/audit/census-backed.txt`, regenerated by the steering pass).
`--check-backed PATH` reads such a file and exits 1 naming every address
in it that is no longer backed — a loss, which a count that holds still
can hide behind a gain. A gain is silent; a function renamed at the same
address is the same function. No gate runs it; the pass decides where.
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
# An operator's symbol may stand between the name and the `@` (parked
# 1722: `TCoord::operator<@00472b30`, `Objects::operator[]@…`).
CITE = re.compile(r"[A-Za-z_][A-Za-z0-9_:~]*[<>=!\[\]+\-*/%&|^]{0,3}@(?:\n[ \t]*)?(00[0-9a-f]{6})")
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


def entry_size(parts, root):
    """`(size, how)` of one `INDEX.tsv` row: the PDB symbol's size when the
    row carries one in a fourth column (`how` = `symbol`), else the
    decompile file's byte count (`decompile`; 0 for a file not on disk)."""
    if len(parts) > 3 and parts[3].strip().isdigit():
        return int(parts[3]), "symbol"
    full = os.path.join(root, parts[2])
    return (os.path.getsize(full) if os.path.isfile(full) else 0), "decompile"


SIZE_HOW = {"symbol": "the PDB symbol's size (INDEX.tsv's fourth column)",
            "decompile": "the decompile file's size in bytes, a proxy (INDEX.tsv carries no symbol size)"}


def size_note(hows):
    """The header's line saying what `bytes` measures, from a Counter of
    each entry's `how`."""
    if len(hows) == 1:
        return "bytes: " + SIZE_HOW[next(iter(hows))]
    return "bytes: " + "; ".join(f"{SIZE_HOW[h]} for {n}" for h, n in sorted(hows.items()))


def index_entries(index, sizes=True):
    """`(va, name, file, size, how)` for every row of `INDEX.tsv` that has
    its three columns; with `sizes` false, size 0 and how `-` (no stat)."""
    root = os.path.dirname(index)
    out = []
    with open(index) as f:
        for line in f:
            parts = line.rstrip("\n").split("\t")
            if len(parts) < 3:
                continue
            size, how = entry_size(parts, root) if sizes else (0, "-")
            out.append((int(parts[0], 16), parts[1], parts[2], size, how))
    return out


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


# **The layer table** (item 1467): the rule is the module docstring's.
THIRD_PARTY = ("zlib\\", "pnglib\\", "packages\\", "cellsdk\\", "steamworks_sdk\\",
               "cpclib\\", "crossplaynetlib\\", "game\\minizip\\", "basic\\", "bighuge\\")
MAIN = "e:\\agent\\_work\\2\\s\\main\\"
LAYER_AI = {"leaders", "leaderoptions", "scriptfunctions", "scriptfuncinits", "scriptstructs",
            "scripttimers", "gamescript"}
INTERFACE = re.compile(
    r"^(iface|conquest|gamespy|optionswin|mainmenu|tutorial|buddy|chat|steamgroupchat|wnd_|mp_?"
    r"|trigger|scripteditor|scriptwatch|scriptlog|editorgroup|rivereditor|scenarioeditor"
    r"|scenariosplash|splashscreen|topmenu|mouseover|popup|pointer|cursors|hotkeygroups|keymap|keys$"
    r"|console|animnotice|selectgroups|saytimer|taunts|dropcontrol|compass|forms|helpxml|motd|mseula"
    r"|objectivesdlg|setupwin|presetupwin|colorpick|parameterwindow|gravwindow|aboutbox|messagebox"
    r"|messagewin|helpbox|statwin|skilltest|credits|replaywin|loadingwin|loadwin|endgamewin"
    r"|achievewin|gamereportwin|netsearchwin|memwin|sectionprofilewin|workshopwin|terrainwin"
    r"|publish|downloading|scripteditbox)|(win|box|dlg|window)$")
ENGINE = re.compile(
    r"(render|graphic|transformmethod|vshader|particle|skybox|shockwave|smoketrails|spline|sound"
    r"|jukebox|steam|workshop|leaderboard|achieve|^scene$|^camera$|^light$|^underlay$|^grassclumps$"
    r"|^terrainout|^tileset$|colors$|^main$|^main_win32$|^gamemain$|^system$|^version$|logger$"
    r"|^gamelog$|^excepthandler$|^watson$|^http$|^patching$|^netdaemon$|^network_event_notifier$"
    r"|^connectiondata$|^gamedaemon$|^packagefifo$|^statbridge$|^profanityfilterer$|^playerprofile$"
    r"|^options$|^istatsandachievements$|elohelper$|^sync(file|dir|display)$|^parsebase$|^macros$"
    r"|^ordmemmgr$|^save$|^autosave$|^recordgame$|^scenariofile$|^scenariodata$|^checksums$"
    r"|^gamemod$|^cdkeyhash$)")


def layer_of(path):
    """A lower-cased PDB source path's layer, by the docstring's rule."""
    if path is None:
        return "unknown"
    if not path.startswith(MAIN):
        return "engine"
    rel = path[len(MAIN):]
    if rel.startswith(THIRD_PARTY):
        return "engine"
    if rel.startswith("game\\script\\"):
        return "AI"
    if not rel.startswith("game\\") or "\\" in rel[len("game\\"):]:
        return "unknown"
    stem = rel[len("game\\"):].rsplit(".", 1)[0]
    if stem in LAYER_AI:
        return "AI"
    if INTERFACE.search(stem):
        return "interface"
    if ENGINE.search(stem):
        return "engine"
    return "simulation"


PDBUTIL = ("llvm-pdbutil", "/opt/homebrew/opt/llvm/bin/llvm-pdbutil")
LINE_FILE = re.compile(r"^([a-zA-Z]:\\.*?) \(MD5")
LINE_RANGE = re.compile(r"^\s+0001:([0-9A-F]{8})-([0-9A-F]{8}),")


def source_ranges(pdb):
    """(start, end, path) for every line record of section 1, as VAs."""
    import shutil
    tool = next((t for t in PDBUTIL if shutil.which(t) or os.path.isfile(t)), None)
    if tool is None:
        sys.exit("census: no llvm-pdbutil (brew install llvm)")
    text = subprocess.run([tool, "dump", "--l", pdb], capture_output=True, text=True,
                          errors="replace", check=True).stdout
    out, cur = [], None
    for line in text.splitlines():
        m = LINE_FILE.match(line)
        if m:
            cur = m.group(1).lower()
            continue
        m = LINE_RANGE.match(line)
        if m and cur:
            out.append((0x401000 + int(m.group(1), 16), 0x401000 + int(m.group(2), 16), cur))
    return sorted(out)


def source_of(ranges, vas):
    import bisect
    starts = [r[0] for r in ranges]
    out = {}
    for va in vas:
        i = bisect.bisect_right(starts, va) - 1
        out[va] = ranges[i][2] if i >= 0 and ranges[i][0] <= va <= ranges[i][1] else None
    return out


QUALIFIED = re.compile(r"\b([A-Z][A-Za-z0-9_]*::~?[A-Za-z_][A-Za-z0-9_]*)\b")
BOLD = re.compile(r"^\s*(?:[-*]\s+)?\*\*([^*]+)\*\*")
HEADING = re.compile(r"^(#+)\s+(.*)")


def named(text, by_name):
    """Addresses a text cites: `name@00xxxxxx`, and `Class::method` written
    whole where it names exactly one function of the export."""
    out = {int(m.group(1), 16) for m in CITE.finditer(text)}
    for m in QUALIFIED.finditer(text):
        hits = by_name.get(m.group(1), ())
        if len(hits) == 1:
            out.add(hits[0])
    return out


def doc_backed(docs, by_name):
    """Every function a coverage section's diff-backed span cites."""
    out = set()
    for p in glob.glob(os.path.join(docs, "*.md")):
        with open(p, errors="replace") as f:
            lines = f.read().splitlines()
        level, backed, span = None, False, []
        for line in lines + ["# end"]:
            h = HEADING.match(line)
            if h:
                if span:
                    out |= named("\n".join(span), by_name)
                span, backed = [], False
                depth = len(h.group(1))
                if "coverage" in h.group(2).lower():
                    level = depth
                elif level is not None and depth <= level:
                    level = None
                continue
            if level is None:
                continue
            b = BOLD.match(line)
            if b:
                word = b.group(1).lower()
                if re.search(r"diff|dump-backed|oracle-backed|packet-backed", word) and "not" not in word:
                    backed = True
                elif re.search(r"reading|read only|not\b|blind|unbacked|seam|listing|unit-|decompile",
                               word):
                    if span:
                        out |= named("\n".join(span), by_name)
                    span, backed = [], False
            if backed:
                span.append(line)
    return out


SWEEP = re.compile(r"tools/emu/|tools/recomp/|emulat", re.I)
# The paperwork guard names the emulator's tables in its own constants.
NOT_A_SWEEP = ("docs_guard.rs",)


def tests_in(text):
    """Each `#[test]` function of a Rust source: its `///` doc comment and
    attributes above it, and its body to the closing brace."""
    lines = text.split("\n")
    starts = [i for i, l in enumerate(lines) if l.strip() == "#[test]"]
    for i in starts:
        top = i
        while top > 0 and re.match(r"\s*(///|#\[)", lines[top - 1]):
            top -= 1
        rest = "\n".join(lines[i:])
        brace = rest.find("{")
        depth, end = 0, len(rest)
        for j in range(brace, len(rest)):
            depth += {"{": 1, "}": -1}.get(rest[j], 0)
            if depth == 0:
                end = j + 1
                break
        yield "\n".join(lines[top:i]) + "\n" + rest[:end]


def sweep_backed(src, by_name, emu_dir=None):
    """Every function a `crates/sim` test that runs the original under the
    emulator cites, in its doc comment or its body — less the callees its
    own script hooks, when `emu_dir` is given (the thirty-eighth pass:
    sixteen landed sweeps named a stubbed callee in the port's comment,
    `// Map::point (stub 0) answers …`, and each was counted as backed
    beside the function swept; a function the sweep answers for is one it
    never runs)."""
    out = set()
    for p in glob.glob(os.path.join(src, "**", "*.rs"), recursive=True):
        if os.path.basename(p) in NOT_A_SWEEP:
            continue
        with open(p, errors="replace") as f:
            text = f.read()
        for chunk in tests_in(text):
            if not SWEEP.search(chunk):
                continue
            cited = named(chunk, by_name)
            if emu_dir is not None:
                for m in SCRIPT.finditer(chunk):
                    cited -= hooked_by_script(os.path.join(emu_dir, m.group(1)))
            out |= cited
    return out


# Not `HOOKS`: that name is the trampolined functions' table above, and the
# thirty-second pass shadowed it with this regex, so `--pin --never` died on
# `HOOKS.items()` for a tranche (the thirty-third pass).
HOOKED_SCRIPT = re.compile(r"Harness\(\s*[^,()]*,\s*\{(.*?)\}", re.S)
SCRIPT = re.compile(r"tools/emu/(sweep_[a-z0-9_]+\.py)")
STUBS_LIST = re.compile(r"^STUBS = \[(.*)\]$", re.M)


def hooked_by_script(script):
    """The addresses a sweep script hands `Harness(exe, {ADDR: …})` as hooked
    callees, or lists as `('call', ADDR, N)` rows of its `STUBS`; empty when
    the script is not on disk."""
    if not os.path.isfile(script):
        return set()
    with open(script, errors="replace") as f:
        body = f.read()
    hm = HOOKED_SCRIPT.search(body)
    hooked = {int(a, 16) for a in re.findall(r"0x(00[0-9a-f]{6})", hm.group(1))} if hm else set()
    sm = STUBS_LIST.search(body)
    if sm:
        hooked |= {int(a) for a in re.findall(r"\(\s*'call'\s*,\s*(\d+)", sm.group(1))}
    return hooked


def hooked_in_sweeps(src, emu_dir, by_name=None):
    """Every (test file, script, address) where a sweep test's chunk cites
    an address its own script hands `Harness(exe, {ADDR: …})` as a hooked
    callee — a function the sweep answers for, not one it runs, which the
    census would count as backed beside the one swept (parked 2012, Loop
    2025: 1998's first draft read 461 for ten). The brief's rule is to
    cite no hooked callee in the chunk; this is the audit of it. An `obj`
    script hooks the `('call', ADDR, N)` rows of its `STUBS` list, and with
    `by_name` a callee cited by name counts as cited (item 2341: the audit
    read neither, and `UnitData::speed` was counted beside its caller)."""
    out = []
    for p in sorted(glob.glob(os.path.join(src, "**", "*.rs"), recursive=True)):
        if os.path.basename(p) in NOT_A_SWEEP:
            continue
        with open(p, errors="replace") as f:
            text = f.read()
        for chunk in tests_in(text):
            if not SWEEP.search(chunk):
                continue
            cited = {int(m.group(1), 16) for m in CITE.finditer(chunk)}
            if by_name is not None:
                cited |= named(chunk, by_name)
            for m in SCRIPT.finditer(chunk):
                hooked = hooked_by_script(os.path.join(emu_dir, m.group(1)))
                for addr in sorted(hooked & cited):
                    out.append((os.path.relpath(p, src), m.group(1), addr))
    return out


def hooked_verdict(rows, by_addr):
    """(exit, line) for `--hooked` (Loop 3312, the forty-fifth pass): the
    audit exited 1 at 62, 72, 89 and 109 chunks across four passes and
    every row was the name form — a port's comment naming the callee its
    stub answers for, which `sweep_backed` already subtracts (the
    thirty-eighth pass). The address form, a chunk citing `Class::m@ADDR`
    for a callee its script hooks, is pinned at zero on the tree
    (`test_census`), and it alone is red here."""
    named = len(rows) - len(by_addr)
    line = (f"hooked callees cited in sweep chunks: {len(rows)} — {len(by_addr)} by address (red when any: "
            f"the chunk asserts what its stub answers), {named} by name (a port comment naming its stub's "
            "answer, subtracted from `backed` already; information, not a defect)")
    return (1 if by_addr else 0), line


def layers(index, docs, sim_src, pdb):
    """The census by layer: {layer: {functions, backed, doc, sweep, port,
    bytes, backed_bytes}}, the simulation layer's files, the unknowns'
    sources, and the note saying what `bytes` measures."""
    entries = index_entries(index)
    names = {va: name for va, name, _, _, _ in entries}
    sizes = {va: size for va, _, _, size, _ in entries}
    by_name = {}
    for va, name, _, _, _ in entries:
        by_name.setdefault(name, []).append(va)
    source = source_of(source_ranges(pdb), names)
    by_doc = doc_backed(docs, by_name) & names.keys()
    by_sweep = sweep_backed(sim_src, by_name, os.path.join(HERE, "emu")) & names.keys()
    rows, sim_files, unknown = layer_rows(names, sizes, source, by_doc, by_sweep, port_spellings(sim_src))
    return rows, sim_files, unknown, size_note(Counter(e[4] for e in entries))


def layer_rows(names, sizes, source, by_doc, by_sweep, spelled):
    """`layers`' table from its inputs: each address's name, size and
    source path, and the two backed sets."""
    rows, sim_files, unknown = {}, Counter(), Counter()
    for va, path in source.items():
        lay = layer_of(path)
        r = rows.setdefault(lay, {"functions": 0, "backed": 0, "doc": 0, "sweep": 0, "port": 0,
                                  "bytes": 0, "backed_bytes": 0})
        is_backed = va in by_doc or va in by_sweep
        r["functions"] += 1
        r["doc"] += va in by_doc
        r["sweep"] += va in by_sweep
        r["backed"] += is_backed
        r["port"] += is_port_backed(va, names[va], by_doc, by_sweep, spelled)
        r["bytes"] += sizes.get(va, 0)
        r["backed_bytes"] += sizes.get(va, 0) if is_backed else 0
        if lay == "simulation":
            sim_files[path.rsplit("\\", 1)[-1]] += 1
        elif lay == "unknown":
            unknown[path.rsplit("\\", 2)[-2] + "\\" if path else "(no line record)"] += 1
    return rows, sim_files, unknown


def is_port_backed(va, name, by_doc, by_sweep, spelled):
    """Loop 3093 (1725, Ramon's answer at the fortieth steer after twelve
    askings): a backed function is **port-backed** when something in
    `crates/sim` consumes it — a document's coverage section diffs it, or
    the crate spells its own `Class::method` outside `sweep/` (the `port`
    mark's test, `rank_unbacked`) — and **layout-backed** when a sweep test
    alone asserts the raw word: an offset identity backs the layout, not a
    consumer, and the two had moved the `Census:` line the same."""
    if va not in by_doc and va not in by_sweep:
        return False
    return va in by_doc or name in spelled


PORT_FN = re.compile(r"\bfn ([a-z0-9_]+)\s*[(<]")


SPELLED = re.compile(r"\b([A-Z][A-Za-z0-9]*::[a-z_][a-z0-9_]*)\b")


def port_spellings(sim_src):
    """Every `Class::method` the crate spells outside `sweep/` as the thing
    it ports: on a code line, or in the doc comment attached to the `fn`
    whose name is the method's own (Loop 2286: a bare name match marked
    `BorderNodes::find` a port consumer because something defines a `fn
    find`; Loop 2420: a comment alone — a SEAM, a note on a neighbour —
    marked `Farms::add_animals` one, and six of seventeen candidates were
    read and dropped)."""
    out = set()
    for p in glob.glob(os.path.join(sim_src, "**", "*.rs"), recursive=True):
        if os.sep + "sweep" + os.sep in p:
            continue
        with open(p, errors="replace") as f:
            out |= spelled_in(f.read())
    return out


def spelled_in(text):
    """The spellings of one file that count (see `port_spellings`)."""
    out, doc = set(), []
    for line in text.splitlines():
        body = line.strip()
        if body.startswith("///"):
            doc.append(body)
            continue
        if body.startswith("#["):
            continue
        m = PORT_FN.search(line) if not body.startswith("//") else None
        if m and doc:
            out |= {s for s in SPELLED.findall("\n".join(doc)) if snake(s.split("::")[1]) == m.group(1)}
        doc = []
        if not body.startswith("//"):
            # A code line, its trailing comment included: a citation beside
            # the computation is the consumer; a line of comment alone is not.
            out |= set(SPELLED.findall(line))
    return out


def port_names(sim_src):
    """Every function name `crates/sim` defines outside `sweep/`: the names a
    port consumer of an original function would be spelled by (snake case
    of the export's method name)."""
    out = set()
    for p in glob.glob(os.path.join(sim_src, "**", "*.rs"), recursive=True):
        if os.sep + "sweep" + os.sep in p:
            continue
        with open(p, errors="replace") as f:
            out |= set(PORT_FN.findall(f.read()))
    return out


def snake(name):
    return re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_", name).lower()


def default_pdb(here):
    """`RON_INSTALL/sbl/rise.pdb`, else `../game/sbl/rise.pdb` beside the
    tools, else the main checkout's through `git rev-parse --git-common-dir`
    (parked 1792: `../game` is absent under `.claude/worktrees/`, and every
    lane passed the main checkout's path by hand)."""
    if os.environ.get("RON_INSTALL"):
        return os.path.join(os.environ["RON_INSTALL"], "sbl", "rise.pdb")
    beside = os.path.join(here, "..", "game", "sbl", "rise.pdb")
    if os.path.isfile(beside):
        return beside
    try:
        common = subprocess.run(["git", "rev-parse", "--git-common-dir"], cwd=here, capture_output=True,
                                text=True, check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return beside
    if not os.path.isabs(common):
        common = os.path.join(here, common)
    main_checkout = os.path.dirname(os.path.abspath(common))
    candidate = os.path.join(main_checkout, "game", "sbl", "rise.pdb")
    return candidate if os.path.isfile(candidate) else beside


def rank_unbacked(entries, backed, port, spelled=(), claimed=(), by="shortest"):
    """`(size, va, name, mark, file)` for every simulation function not yet
    backed, shortest decompile first — largest first with `by="bytes"`,
    where the unbacked bytes are (parked 1703: four batches wrote this
    lister as a scratch script). `entries` is `(va, name, source path,
    decompile size)`; `backed` the by-doc and by-sweep union; `port` the
    names `crates/sim` defines; `spelled` the `Class::method` names its text
    carries; `claimed` the addresses a sibling lane's tree already backs
    (Loop 2926, the thirty-ninth pass: the two sweep lanes swept the same
    function twice in one tranche, a gate and a half each — `--claimed
    DIR` reads the other lane's worktree, and a function it cites is
    `claimed` here and hidden). The mark says what the sweep lane wants to
    know before it reads a function: `port` when this crate spells the
    function's own `Class::method` and defines the method's snake-case name
    (a consumer the sweep can run against — parked 1725; Loop 2286: the
    class, not the name alone), `name` when only the snake-case name is
    defined somewhere, `operator` for a name the citation pattern spells
    with a symbol, `-` otherwise."""
    rows = []
    for va, name, path, size in entries:
        if layer_of(path) != "simulation" or va in backed:
            continue
        short = name.rsplit("::", 1)[-1]
        if va in claimed:
            mark = "claimed"
        elif "operator" in short:
            mark = "operator"
        elif is_dead_weight(name):
            mark = "skip"
        elif is_ui_body(name):
            mark = "ui"
        elif snake(short) in port and name in spelled:
            mark = "port"
        elif snake(short) in port:
            mark = "name"
        else:
            mark = "-"
        rows.append((size, va, name, mark, (path or "").rsplit("\\", 1)[-1]))
    rows.sort(key=lambda r: (-r[0], r[1]) if by == "bytes" else (r[0], r[1]))
    return rows


# The name classes every sweep lane skipped by hand (parked 1861): a
# destructor, a class's open/close/init/clear, a registration, a logger, a
# compiler's dynamic initializer. None reads a value a port can be held to.
# The export spells an initializer `` `dynamic_initializer_for_'X'' ``, the
# backtick first (Loop 2394: forty of them opened the default listing).
DEAD_WEIGHT = re.compile(r"(^|::)`?(~|close|close_class|init|init_class|clear|reset|registration|"
                         r"log|log_data|log_state|dump|print|print_data|dynamic_initializer|"
                         r"dynamic_atexit|scalar_deleting|vector_deleting)")


def is_dead_weight(name):
    # A backticked name is the compiler's — an initializer, an atexit
    # destructor, a `string'` — and its quoted part may carry a `::` of its
    # own (`` `dynamic_initializer_for_'MiscAccess::intro'' ``), so it is
    # read whole before the short name is cut (Loop 2394).
    if name.startswith("`"):
        return True
    cls, _, short = name.rpartition("::")
    constructor = bool(cls) and short == cls.rsplit("::", 1)[-1]
    return (constructor or short.startswith("one_time_") or bool(DEAD_WEIGHT.search("::" + short))
            or "dynamic initializer" in name or "dynamic atexit" in name)


# A renderer's, an input handler's, a sound's, a debug or cheat action's or
# an editor's body, filed under simulation by its source file (Loop 2819, the
# thirty-eighth pass: the harvest lane's batches were `draw_thick_diamond`,
# `Nuke::draw_ctw`, `Game::run_unit_editor`, `WorldMap::on_right_down`, and
# the `--unbacked` head at the pass was twelve more). The `*Out` classes are
# the engine's renderers of each object kind. The layer and the score do not
# move on this mark; the list the sweep lane reads does.
UI_BODY = re.compile(r"^(draw|render|paint|blit|on_(mouse|left|right|middle|key|char|wheel|paint|size)|"
                     r"setup_render_window|play_\w*sound|say|debug_|add_for_render|add_\w*_poly|"
                     r"post_game_screens|action_cheat_|run_\w*editor|\w*_editor$|\w*_to_screen$|"
                     r"screen_to_\w*|\w*_screen$)")


def is_ui_body(name):
    cls, _, short = name.rpartition("::")
    return cls.endswith("Out") or bool(UI_BODY.match(short)) or short.endswith(("_draw", "_render"))


BARE_CITE = re.compile(r"(?<![A-Za-z0-9_:])([a-z_][a-z0-9_]*)@(00[0-9a-f]{6})")


def cited_bare(src, by_name=None):
    """Each `(file, name, address)` a `crates/sim` sweep test spells as a bare
    `method@addr`, no class before it (Loop 2843: the census counts only the
    `Class::method@addr` form, so a bare citation scores nothing). A free
    function's name is whole as it stands: with `by_name`, a name the export
    knows is not bare."""
    out = []
    for p in sorted(glob.glob(os.path.join(src, "**", "*.rs"), recursive=True)):
        if os.path.basename(p) in NOT_A_SWEEP:
            continue
        with open(p, errors="replace") as f:
            text = f.read()
        chunks = [c for c in tests_in(text) if SWEEP.search(c)]
        if not chunks:
            continue
        # An address the file also spells whole somewhere is counted; only
        # one it never spells whole is a bare citation.
        whole = {int(m.group(1), 16) for m in CITE.finditer(text) if "::" in m.group(0)}
        for chunk in chunks:
            for m in BARE_CITE.finditer(chunk):
                row = (os.path.relpath(p, src), m.group(1), int(m.group(2), 16))
                if row[2] not in whole and row not in out and not (by_name and m.group(1) in by_name):
                    out.append(row)
    return out


def worth_sweeping(rows):
    """The rows the sweep wants: not `skip`, not `operator` (parked 1722),
    not a `ui` body (Loop 2819), not a function a sibling lane's tree
    already cites (`claimed`, Loop 2926)."""
    return [r for r in rows if r[3] not in ("skip", "operator", "ui", "claimed")]


def claimed_in(dirs, by_name):
    """Every function a sweep test on one of the sibling trees `dirs`
    (another lane's worktree, its root) cites — what that lane has built
    or is building, committed or not — so `--unbacked` can hide it (Loop
    2926). A tree with no `crates/sim/src` claims nothing."""
    out = set()
    for d in dirs:
        src = os.path.join(d, "crates", "sim", "src")
        if os.path.isdir(src):
            out |= sweep_backed(src, by_name, os.path.join(d, "tools", "emu"))
    return out


def unbacked(index, docs, sim_src, pdb, top, show_all=False, claimed_dirs=(), by="shortest"):
    """The never-backed simulation functions, shortest decompile first
    (largest first with `by="bytes"`); the dead weight (`skip`) and the
    operators are hidden unless `show_all`; a function a tree in
    `claimed_dirs` cites is `claimed` and hidden too (Loop 2926)."""
    full = index_entries(index)
    by_name = {}
    for va, name, _, _, _ in full:
        by_name.setdefault(name, []).append(va)
    source = source_of(source_ranges(pdb), [e[0] for e in full])
    entries = [(va, name, source.get(va), size) for va, name, _, size, _ in full]
    backed = (doc_backed(docs, by_name) | sweep_backed(sim_src, by_name, os.path.join(HERE, "emu")))
    claimed = claimed_in(claimed_dirs, by_name) - backed
    rows = rank_unbacked(entries, backed, port_names(sim_src), port_spellings(sim_src), claimed, by)
    shown = rows if show_all else worth_sweeping(rows)
    print("# " + size_note(Counter(e[4] for e in full)))
    print(f"{'bytes':>6} {'function':52} {'mark':8} file")
    for size, va, name, mark, fname in shown[:top]:
        print(f"{size:6} {name + '@' + format(va, '08x'):52} {mark:8} {fname}")
    marks = Counter(r[3] for r in rows)
    print(f"\n{len(rows)} never-backed simulation functions: {marks.get('port', 0)} with a port "
          f"consumer (the crate spells the Class::method), {marks.get('name', 0)} matched by name alone, "
          f"{marks.get('operator', 0)} operators, {marks.get('skip', 0)} dead weight "
          f"(destructors, close/init/clear, loggers, initializers), {marks.get('ui', 0)} renderer, input, "
          f"sound, cheat or editor bodies (`ui`, Loop 2819"
          f"{'; shown' if show_all else '; hidden, --all shows them'})"
          + (f", {marks.get('claimed', 0)} claimed by a sibling lane's tree (--claimed, Loop 2926"
             f"{'; shown' if show_all else '; hidden'})" if claimed_dirs else "")
          + f"; the {min(top, len(shown))} {'largest' if by == 'bytes' else 'shortest'} shown (--top"
          + (", --by bytes)" if by == "bytes" else ")"))
    bare = cited_bare(sim_src, by_name)
    if bare:
        print(f"cited bare: {len(bare)} address(es) a sweep test names without the class — the census "
              f"counts only `Class::method@addr` (Loop 2843): "
              + ", ".join(f"{f}:{n}@{va:08x}" for f, n, va in bare[:12]))


EMPTY_ROW = {"functions": 0, "backed": 0, "doc": 0, "sweep": 0, "port": 0, "bytes": 0, "backed_bytes": 0}


def print_layers(rows, sim_files, unknown, as_json, size_note="bytes: " + SIZE_HOW["decompile"]):
    import json
    sim = rows.get("simulation", EMPTY_ROW)
    if as_json:
        print(json.dumps({"layers": rows, "simulation_backed": sim["backed"],
                          "simulation_port_backed": sim["port"],
                          "simulation_functions": sim["functions"],
                          "simulation_bytes": sim.get("bytes", 0),
                          "simulation_backed_bytes": sim.get("backed_bytes", 0),
                          "bytes_source": size_note}, indent=1, sort_keys=True))
        return
    print(f"# {size_note}")
    print(f"{'layer':12} {'functions':>9} {'backed':>7} {'by doc':>7} {'by sweep':>8} {'port':>6} {'layout':>7} "
          f"{'share':>7} {'bytes':>10} {'backed bytes':>12} {'byte share':>10}")
    for lay in ("simulation", "AI", "engine", "interface", "unknown"):
        r = {**EMPTY_ROW, **rows.get(lay, {})}
        share = f"{100 * r['backed'] / r['functions']:.1f}%" if r["functions"] else "-"
        bshare = f"{100 * r['backed_bytes'] / r['bytes']:.1f}%" if r["bytes"] else "-"
        print(f"{lay:12} {r['functions']:9} {r['backed']:7} {r['doc']:7} {r['sweep']:8} {r['port']:6} "
              f"{r['backed'] - r['port']:7} {share:>7} {r['bytes']:10} {r['backed_bytes']:12} {bshare:>10}")
    print()
    print("# simulation, by source file (rule 5's default — a misfiled file shows here)")
    print(" ".join(f"{f}:{n}" for f, n in sim_files.most_common()))
    print()
    print("# unknown, by where the source sits")
    print(" ".join(f"{f}:{n}" for f, n in unknown.most_common()))
    print()
    print(census_line(sim["backed"], sim["port"], sim["functions"]))
    print(census_bytes_line(sim.get("backed_bytes", 0), sim.get("bytes", 0)))


def census_line(backed, port, functions):
    """The handoff's line (Loop 3093): the total first, as every reader of
    `<N> of <M>` expects, then the split the sweep lane is judged on."""
    return (f"Census: simulation backed {backed} of {functions} · port {port} · "
            f"layout {backed - port}")


def census_bytes_line(backed_bytes, total_bytes):
    """The `Census:` line's byte-weighted twin, printed after it as a line
    of its own so the guard's parse of that line is unchanged."""
    share = f"{100 * backed_bytes / total_bytes:.1f}%" if total_bytes else "-"
    return f"Census bytes: simulation {backed_bytes} of {total_bytes} bytes backed · {share}"


# --- the backed list ------------------------------------------------------
BACKED_ROW = re.compile(r"^([0-9a-f]{8})\t([^\t]+)\t(doc|sweep)$")


def backed_rows(names, source, by_doc, by_sweep):
    """`(va, name, by)` for every backed simulation function, by address;
    `by` is `doc` when a coverage section's diff backs it, else `sweep`."""
    out = []
    for va in sorted(names):
        if layer_of(source.get(va)) != "simulation":
            continue
        if va in by_doc:
            out.append((va, names[va], "doc"))
        elif va in by_sweep:
            out.append((va, names[va], "sweep"))
    return out


def backed_now(index, docs, sim_src, pdb):
    """`backed_rows` on this tree: the export, the PDB's line records,
    `docs/` and `crates/sim`."""
    entries = index_entries(index, sizes=False)
    names = {va: name for va, name, _, _, _ in entries}
    by_name = {}
    for va, name, _, _, _ in entries:
        by_name.setdefault(name, []).append(va)
    source = source_of(source_ranges(pdb), names)
    by_doc = doc_backed(docs, by_name) & names.keys()
    by_sweep = sweep_backed(sim_src, by_name, os.path.join(HERE, "emu")) & names.keys()
    return backed_rows(names, source, by_doc, by_sweep)


def format_backed(row):
    va, name, by = row
    return f"{va:08x}\t{name}\t{by}"


def tree_label():
    """The commit the list is measured on: short SHA and branch, and a
    mark when `docs/` or `crates/sim` carry uncommitted changes."""
    def git(*args):
        return subprocess.run(["git", *args], cwd=HERE, capture_output=True, text=True,
                              check=True).stdout.strip()
    try:
        label = f"{git('rev-parse', '--short', 'HEAD')} ({git('rev-parse', '--abbrev-ref', 'HEAD')})"
        if git("status", "--porcelain", "--", "../docs", "../crates/sim"):
            label += ", with uncommitted changes under docs/ or crates/sim"
        return label
    except (OSError, subprocess.CalledProcessError):
        return "(not a git tree)"


def backed_header(date, tree, count):
    return (f"# census-backed.txt — the {count} backed simulation functions, by address, as\n"
            f"# `tools/census.py --backed-list` counts them: ADDR<tab>Class::method<tab>by (doc|sweep).\n"
            f"# Measured {date} on {tree}.\n"
            f"# The steering pass regenerates it with\n"
            f"# `python3 tools/census.py --backed-list --write docs/audit/census-backed.txt`;\n"
            f"# `--check-backed` names every address here that is no longer backed (a gain is silent).\n")


def read_backed(path):
    """`{va: (name, by)}` from a `--backed-list` file; `#` lines and blank
    lines are skipped, and a line that does not parse stops the read."""
    out = {}
    with open(path) as f:
        for n, line in enumerate(f, 1):
            line = line.rstrip("\n")
            if not line.strip() or line.startswith("#"):
                continue
            m = BACKED_ROW.match(line)
            if not m:
                sys.exit(f"{path}:{n}: not ADDR<tab>Class::method<tab>doc|sweep: {line!r}")
            out[int(m.group(1), 16)] = (m.group(2), m.group(3))
    return out


def lost_backed(listed, live):
    """The rows of `listed` (`read_backed`'s dict) whose address `live`
    (a set of addresses) no longer holds — by address, so a rename is not
    a loss."""
    return [(va, name, by) for va, (name, by) in sorted(listed.items()) if va not in live]


def main():
    ap = argparse.ArgumentParser(
        description="The executable as the denominator: the class census (default), the census by "
                    "layer (--layers), the never-backed list (--unbacked), and the backed list and its "
                    "regression check (--backed-list, --check-backed). The module docstring has the rules.")
    ap.add_argument("logs", nargs="*")
    ap.add_argument("--index", default=os.path.expanduser("~/ghidra-projects/decomp/INDEX.tsv"))
    ap.add_argument("--docs", default=os.path.join(HERE, "..", "docs"))
    ap.add_argument("--top", type=int, default=40)
    ap.add_argument("--never", action="store_true")
    ap.add_argument("--pin", action="store_true",
                    help="read the traces rondata::blind::TRACES pins, from $RON_GAMELOG_DIR")
    ap.add_argument("--layers", action="store_true",
                    help="the census by layer with the backed column and the byte-weighted one, and the "
                         "board's number (item 1467) with its `Census bytes:` twin")
    ap.add_argument("--json", action="store_true", help="with --layers: the same as JSON")
    ap.add_argument("--unbacked", action="store_true",
                    help="the never-backed simulation functions, shortest decompile first, "
                         "with a port-consumer mark (parked 1703, 1725); --top rows")
    ap.add_argument("--by", choices=("shortest", "bytes"), default="shortest",
                    help="--unbacked: the order — shortest first (the default), or largest first (bytes)")
    ap.add_argument("--backed-list", action="store_true",
                    help="every backed simulation function, ADDR<tab>Class::method<tab>doc|sweep, by address")
    ap.add_argument("--write", metavar="PATH",
                    help="--backed-list: write the list to PATH with a dated header naming the tree")
    ap.add_argument("--check-backed", metavar="PATH",
                    help="exit 1 naming every function in PATH (a --backed-list file) no longer backed; "
                         "a gain is silent, a rename at the same address is the same function")
    ap.add_argument("--pdb", default=default_pdb(HERE))
    ap.add_argument("--all", action="store_true",
                    help="--unbacked: show the dead weight and the operators too")
    ap.add_argument("--claimed", action="append", default=[], metavar="DIR",
                    help="--unbacked: a sibling lane's worktree (its root); a function a sweep test there cites "
                         "is `claimed` and hidden, so two sweep lanes never build one function (Loop 2926)")
    ap.add_argument("--hooked", action="store_true",
                    help="the audit of Loop 2025: every sweep test chunk that cites an address its own "
                         "script hooks as a callee; exit 1 when any")
    ap.add_argument("--sim-src", default=os.path.join(HERE, "..", "crates", "sim", "src"))
    a = ap.parse_args()
    if a.write and not a.backed_list:
        ap.error("--write goes with --backed-list")
    if a.layers:
        rows, sim_files, unknown, note = layers(a.index, a.docs, a.sim_src, a.pdb)
        print_layers(rows, sim_files, unknown, a.json, size_note=note)
        return
    if a.unbacked:
        unbacked(a.index, a.docs, a.sim_src, a.pdb, a.top, a.all, a.claimed, a.by)
        return
    if a.backed_list:
        import datetime
        rows = backed_now(a.index, a.docs, a.sim_src, a.pdb)
        body = "".join(format_backed(r) + "\n" for r in rows)
        if a.write:
            # The label is read before the file is opened: an open for write
            # would mark the tree dirty with the list itself.
            head = backed_header(datetime.date.today().isoformat(), tree_label(), len(rows))
            with open(a.write, "w") as f:
                f.write(head + body)
            print(f"wrote {len(rows)} backed simulation functions to {a.write}")
        else:
            sys.stdout.write(body)
        return
    if a.check_backed:
        listed = read_backed(a.check_backed)
        live = {va for va, _, _ in backed_now(a.index, a.docs, a.sim_src, a.pdb)}
        lost = lost_backed(listed, live)
        for row in lost:
            print("lost\t" + format_backed(row))
        print(f"census-backed: {len(listed)} listed, {len(lost)} no longer backed, "
              f"{len(live - listed.keys())} backed and not listed (silent)")
        sys.exit(1 if lost else 0)
    if a.hooked:
        by_name = {}
        with open(a.index) as f:
            for line in f:
                addr, name = line.rstrip("\n").split("\t")[:2]
                by_name.setdefault(name, []).append(int(addr, 16))
        rows = hooked_in_sweeps(a.sim_src, os.path.join(HERE, "emu"), by_name)
        by_addr = hooked_in_sweeps(a.sim_src, os.path.join(HERE, "emu"))
        for file, script, addr in rows:
            print(f"{file}\t{script}\t{addr:08x}")
        code, line = hooked_verdict(rows, by_addr)
        print(line)
        sys.exit(code)
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
