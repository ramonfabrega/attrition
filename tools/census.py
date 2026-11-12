#!/usr/bin/env python3
r"""census.py — the executable as the denominator.

    census.py [--index INDEX.tsv] [--docs docs/] [--top N] [--never] [--cites DOC ...] [--pin] [<rontrace.log> ...]
    census.py --layers [--json] [--pin [--entered-top N]]
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

`--never --cites DOC` (repeatable; Loop 3415) then prints, per document,
the blind list's functions that document itself cites, each as
`Class::method@addr` with a mark — `residue` for a row of
`rondata::blind::RESIDUE` (read from `blind.rs`), `only here` for one no
other `docs/**/*.md` cites — and ends on the line a pass quotes when it
books a reading item: `cites DOC: N never, M net of residue, K only here`
(K counts within M). Those lines repeat together after the last block.

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

**The entered column** (Loop 3350, the forty-eighth pass). `--layers
--pin` (or `--layers <rontrace.log> …`) adds four columns per layer —
`entered`, the functions a given coverage trace entered (`entered_in |
hooked_in`, the set `--pin --never` measures the blind list against),
`ent unbacked`, those of them not backed, and the same two by bytes —
writes the by-source-file line as `file:functions/entered`, prints a
`Census entered: simulation <E> of <N> functions · <A> entered and
unbacked · <eb> of <t> bytes` line after the `Census bytes:` line, and
ends on the `--entered-top` (20) largest entered-and-unbacked simulation
functions with their source file. It is joined to nothing: `backed` is
not changed by it. Without traces nothing is added — the table, both
lines and the JSON are byte-for-byte what they were; `--json` with traces
adds `entered_layers` and four `simulation_entered*` keys.

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


def cited_per_doc(docs):
    """`{realpath: set of addresses}` — each `docs/**/*.md` file's own
    cites, by the same `CITE` (Loop 3415); `cited_in` is their union."""
    out = {}
    for p in glob.glob(os.path.join(docs, "**", "*.md"), recursive=True):
        with open(p, errors="replace") as f:
            out[os.path.realpath(p)] = {int(m.group(1), 16) for m in CITE.finditer(f.read())}
    return out


def cited_in(docs):
    return set().union(*cited_per_doc(docs).values())


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


def residue_addrs(pin=PIN):
    """The addresses of `rondata::blind::RESIDUE`, read from the source:
    each row's first element, `0x00ab_cdef` with underscores allowed. An
    address inside a row's reason string is not a row."""
    with open(pin) as f:
        text = f.read()
    body = text[text.index("pub const RESIDUE"):]
    body = body[body.index("= &[") + 4:body.index("\n];")]
    return {int(m.group(1).replace("_", ""), 16)
            for m in re.finditer(r'\(\s*0x([0-9a-fA-F_]+)\s*,\s*"', body)}


def doc_never(doc, per_doc, name_of, seen, residue):
    """`--cites DOC` (Loop 3415): `(rows, N, M, K)` for one document —
    `rows` its never-entered cites as `(addr, name, mark)` by address, the
    blind list's own rule (in the export, not in `seen`); `mark` is
    `residue` for a `RESIDUE` row, `only here` for one no other document
    under `docs/` cites, else `-`. N counts the rows, M the rows net of
    residue, K of those M the `only here`. Exits naming a document the
    walk did not find."""
    key = os.path.realpath(doc)
    if key not in per_doc:
        sys.exit(f"--cites {doc}: not a document under the docs walk")
    others = set().union(*(v for k, v in per_doc.items() if k != key))
    rows = []
    for v in sorted(per_doc[key]):
        if v not in name_of or v in seen:
            continue
        mark = "residue" if v in residue else ("only here" if v not in others else "-")
        rows.append((v, name_of[v], mark))
    n = len(rows)
    m = sum(1 for r in rows if r[2] != "residue")
    k = sum(1 for r in rows if r[2] == "only here")
    return rows, n, m, k


def cites_line(doc, n, m, k):
    """The line a pass quotes when it books a reading item."""
    return f"cites {doc}: {n} never, {m} net of residue, {k} only here"


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


def sweep_files(src):
    """Every `.rs` file a sweep test can stand in: under `src`, and under
    the crate's `tests/` beside it, where item 4009 moved the 1,305
    standalone sweeps (`crates/sim/tests/sweep/`)."""
    tests = os.path.join(os.path.dirname(os.path.normpath(src)), "tests")
    return (glob.glob(os.path.join(src, "**", "*.rs"), recursive=True)
            + glob.glob(os.path.join(tests, "**", "*.rs"), recursive=True))


def sweep_backed(src, by_name, emu_dir=None):
    """Every function a `crates/sim` test that runs the original under the
    emulator cites, in its doc comment or its body — less the callees its
    own script hooks, when `emu_dir` is given (the thirty-eighth pass:
    sixteen landed sweeps named a stubbed callee in the port's comment,
    `// Map::point (stub 0) answers …`, and each was counted as backed
    beside the function swept; a function the sweep answers for is one it
    never runs)."""
    out = set()
    for p in sweep_files(src):
        if os.path.basename(p) in NOT_A_SWEEP:
            continue
        with open(p, errors="replace") as f:
            text = f.read()
        for chunk in tests_in(text):
            if not SWEEP.search(chunk):
                continue
            cited = named(chunk, by_name)
            if emu_dir is not None:
                for script in scripts_in(chunk):
                    cited -= hooked_by_script(os.path.join(emu_dir, script))
            out |= cited
    return out


# Not `HOOKS`: that name is the trampolined functions' table above, and the
# thirty-second pass shadowed it with this regex, so `--pin --never` died on
# `HOOKS.items()` for a tranche (the thirty-third pass).
HOOKED_SCRIPT = re.compile(r"Harness\(\s*[^,()]*,\s*\{(.*?)\}", re.S)
# A sweep's script by its path, or an `obj` sweep's row of `sweep_obj.tsv` by
# the `emu_obj("<snake>")` that runs it (item 3731) — `scripts_in` names both
# as `sweep_<snake>.py`, and `hooked_by_script` reads the row when no script
# of that name is on disk. The driver itself is no sweep's script.
SCRIPT = re.compile(r"tools/emu/(sweep_(?!obj\.py)[a-z0-9_]+\.py)|emu_obj\(\s*\"([a-z0-9_]+)\"\s*\)")
STUBS_LIST = re.compile(r"^STUBS = \[(.*)\]$", re.M)
OBJ_TABLE = "sweep_obj.tsv"


def scripts_in(chunk):
    """The sweep scripts a test chunk names, each as `sweep_<snake>.py`."""
    return [m.group(1) or f"sweep_{m.group(2)}.py" for m in SCRIPT.finditer(chunk)]


def obj_row(script):
    """The row of `sweep_obj.tsv` beside `script` for its snake, as a dict,
    or None (item 3731)."""
    import ast
    table = os.path.join(os.path.dirname(script), OBJ_TABLE)
    snake = os.path.basename(script)[len("sweep_"):-len(".py")]
    if not os.path.isfile(table):
        return None
    with open(table) as f:
        for line in f:
            name, literal, *_ = line.rstrip("\n").split("\t") + [""]
            if name == snake:
                return ast.literal_eval(literal)
    return None


def hooked_by_script(script):
    """The addresses a sweep script hands `Harness(exe, {ADDR: …})` as hooked
    callees, or lists as `('call', ADDR, N)` rows of its `STUBS` — the
    script's, or its `sweep_obj.tsv` row's when the script is not on disk;
    empty when neither is."""
    if not os.path.isfile(script):
        row = obj_row(script)
        if row is None:
            return set()
        return {a for k, a, _ in row.get("STUBS", ()) if k == "call"} - {row.get("ENTRY")}
    with open(script, errors="replace") as f:
        body = f.read()
    hm = HOOKED_SCRIPT.search(body)
    hooked = {int(a, 16) for a in re.findall(r"0x(00[0-9a-f]{6})", hm.group(1))} if hm else set()
    sm = STUBS_LIST.search(body)
    if sm:
        hooked |= {int(a) for a in re.findall(r"\(\s*'call'\s*,\s*(\d+)", sm.group(1))}
    # A recursive function's own entry is hooked on its second visit
    # (`Harness.skip_once`, item 3386): the sweep runs it, it does not answer
    # for another, so it is never one of the callees subtracted (item 3395:
    # `Unit::find_path` was swept and still listed unbacked).
    em = re.search(r"^ENTRY\s*=\s*0x([0-9a-f]+)", body, re.M)
    if em:
        hooked.discard(int(em.group(1), 16))
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
    for p in sorted(sweep_files(src)):
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
            for script in scripts_in(chunk):
                # `hooked_by_script` leaves out the function's own entry: a
                # recursive function is hooked on its second visit
                # (`Harness.skip_once`, item 3386), and the sweep runs it.
                hooked = hooked_by_script(os.path.join(emu_dir, script))
                for addr in sorted(hooked & cited):
                    out.append((os.path.relpath(p, src), script, addr))
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
    return layers_entered(index, docs, sim_src, pdb, None)[:4]


def layers_entered(index, docs, sim_src, pdb, entered):
    """`layers`' four, and a fifth: `entered_rows` on the same inputs for
    `entered` (a set of addresses a trace entered), or None when it is."""
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
    ent = None if entered is None else entered_rows(names, sizes, source, by_doc, by_sweep, entered)
    return rows, sim_files, unknown, size_note(Counter(e[4] for e in entries)), ent


def entered_rows(names, sizes, source, by_doc, by_sweep, entered):
    """The `entered` column (Loop 3350, the forty-eighth pass): per layer,
    the functions a pinned coverage trace entered — `entered_in | hooked_in`,
    the set `--pin --never` measures the blind list against — and those of
    them not backed, by count and by bytes; the simulation layer's entered
    count per source file; and the entered-and-unbacked simulation
    functions as `(bytes, va, name, file)`, largest first. Joined to
    nothing: `backed` is not changed by it (DECISIONS 29 is Ramon's)."""
    out, files, unbacked = {}, Counter(), []
    for va, path in source.items():
        lay = layer_of(path)
        r = out.setdefault(lay, dict(EMPTY_ENTERED))
        if va not in entered:
            continue
        size = sizes.get(va, 0)
        is_backed = va in by_doc or va in by_sweep
        r["entered"] += 1
        r["entered_bytes"] += size
        if not is_backed:
            r["entered_unbacked"] += 1
            r["entered_unbacked_bytes"] += size
        if lay == "simulation":
            file = path.rsplit("\\", 1)[-1]
            files[file] += 1
            if not is_backed:
                unbacked.append((size, va, names.get(va, "?"), file))
    unbacked.sort(key=lambda u: (-u[0], u[1]))
    return {"layers": out, "files": files, "unbacked": unbacked}


EMPTY_ENTERED = {"entered": 0, "entered_unbacked": 0, "entered_bytes": 0, "entered_unbacked_bytes": 0}


def census_entered_line(entered, functions, unbacked, entered_bytes, total_bytes):
    """The `Census bytes:` line's sibling for the `entered` column, printed
    after it, never folded into it: the guard's parse of `Census:` and the
    byte line are unchanged."""
    return (f"Census entered: simulation {entered} of {functions} functions · {unbacked} entered and "
            f"unbacked · {entered_bytes} of {total_bytes} bytes")


def print_entered_unbacked(ent, top=20):
    """The `top` largest entered-and-unbacked simulation functions — what an
    `audit` definition of backed would credit first; information only."""
    rows = ent["unbacked"]
    print(f"# the {min(top, len(rows))} largest of {len(rows)} entered-and-unbacked simulation functions, "
          "by bytes (information: `backed` is not changed by it)")
    for i, (size, _va, name, file) in enumerate(rows[:top], 1):
        print(f"{i:5} {size:7}  {name}  {file}")


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
        elif is_cut(name):
            mark = "cut"
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
    return (cls.endswith("Out") or bool(UI_BODY.match(short)) or short.endswith(("_draw", "_render"))
            or bool(READER_UI.search(name)))


# A simulation-layer body that is out of v1 by CLAUDE.md's own cut — a shell
# mode's driver, the match lifecycle, the scenario, GameSpy and multiplayer
# playback hosts — whose `name` mark bought two parks in one tranche (3404a,
# 3404b; Loop 3405, the forty-seventh pass). Marked `cut` and left off the
# sweepable list like `ui`; the layer and the denominator do not move (the
# score's definition is Ramon's, Loop 3237).
CUT_FROM_V1 = frozenset((
    "UnitBalance::next",            # the unit-balance shell mode's driver (3404a)
    "Game::run",                    # the match lifecycle, no frame loop in it (3404b)
    "TimeSync::process",            # passed over by every brief of the tranche as cut
    "MapScenario::make",            # the scenario editor's map
    "Game::run_scenario",           # the scenario host
    "Game::run_gamespy",            # GameSpy
    "CommandManager::set_mp_playback",  # multiplayer playback
))


def is_cut(name):
    return name in CUT_FROM_V1 or bool(READER_CUT.search(name))


# The territory readers' classes (item 4170; DECISIONS 114 §6, the 26
# readings `docs/audit/2026-11-06-territory-*.md`, each class beside the
# readings that proposed it). A class lands only when every open function it
# reaches is one a reader ruled excluded and none it reaches is ported or ruled
# a twin, absorbed, a differing twin or unmodelled
# (`tools/explore/test_census_territory.py` holds the conflicts that narrowed
# them: `Doober::inc_time` and `Unit::update_gpiece` are ports, so neither
# `Doober::` nor `gpiece` is a class, and `Unit::dbg_jump_to_action` is a
# differing twin, so `dbg_` is two names). The renderer's go to `ui`; the
# editor's, Conquer the World's, the shell's game types and multiplayer's go
# to `cut`, the finish line's "not in it, by design" (docs/TRACKS.md).
READER_UI = re.compile(
    # the border renderer (borders-1): the strip and morph mesh over the
    # territory the rules compute elsewhere
    r"^(Borders?|BorderNodes|BorderMorphRender|BorderSplineRender|BorderSpline|MorphSideStrip|StripBuilder"
    r"|LinkVert|TerrainBorder)::"
    # the terrain and mini-map renderers and the decorations (rest-1 to 3,
    # terrain-1, object-1): no rule reads their state
    r"|^(TerrainVis|WorldMap|WorldMapFog|WorldMapBackground|Transitions|CircleRender|Surf|RiverDoobers?|Storm"
    r"|FogMaster|CoordInfo)::"
    # the property system's reflection, render reads and interface actions
    # (object-1, object-2, game-1, game-2, build-1, build-3, terrain-2, rest-2)
    r"|::(get_render_\w+|on_field_change|on_redraw|\w*_popup|\w*screen_?shot\w*|zoom_\w+|list_rules"
    r"|device_lost\w*|\w*_to_console|coord_vis.*|\w*weathering|add_\w*for_render|add_geometry|show_\w+"
    r"|play_sound\w*|get_name|recalc_heights|place_tree)$"
    # the frame pacing and the solo shell's checks (game-1)
    r"|^Game::(get_render_mode|calc_graphic_resolution|get_game_fps_average|add_game_fps_to_average|solo_checks"
    r"|solo_init)$")
READER_CUT = re.compile(
    # the editor (terrain-1, terrain-2, rest-2, groups-1)
    r"^(TerrainMap|Signposts)::|::dbg_(pack_or_unpack|jump_to_waypoint)$"
    r"|^Mountains::(verify_template|remove_mountain)$|^Cliffs::delete_cliff$|^TerrainOil::remove_rocky_patch$"
    r"|^World::revert_coast$"
    # Conquer the World (map-1, map-2, rest-2)
    r"|^(MapConquest|ReinforcementArmy)::"
    # the shell's game types (game-1, game-2)
    r"|^UnitBalance::|^Game::(run_(skilltest|campaign|unit_balance|solo)|end_game_close)$"
    # multiplayer and the platform (game-1, game-2)
    r"|^(SyncPoint|NetDaemon|NetSys|NetPlayer|DropControl|SyncFile)::|achievement|[Ll]eaderboard|steam"
    r"|[Ww]orkshop|::OnFind|^Game::(is_tournament|is_observing|unload_net_lib|run_oos_recovery|is_game_ready"
    r"|is_cooperative)$|^Player::(drop|is_active_connection|is_connection)$|^std::")


BARE_CITE = re.compile(r"(?<![A-Za-z0-9_:])([a-z_][a-z0-9_]*)@(00[0-9a-f]{6})")


def cited_bare(src, by_name=None):
    """Each `(file, name, address)` a `crates/sim` sweep test spells as a bare
    `method@addr`, no class before it (Loop 2843: the census counts only the
    `Class::method@addr` form, so a bare citation scores nothing). A free
    function's name is whole as it stands: with `by_name`, a name the export
    knows is not bare."""
    out = []
    for p in sorted(sweep_files(src)):
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
    not a `ui` body (Loop 2819), not a body cut from v1 (`cut`, Loop 3405),
    not a function a sibling lane's tree already cites (`claimed`, Loop
    2926)."""
    return [r for r in rows if r[3] not in ("skip", "operator", "ui", "cut", "claimed")]


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
          f"{'; shown' if show_all else '; hidden, --all shows them'}), {marks.get('cut', 0)} cut from v1 "
          f"(`cut`, Loop 3405{'; shown' if show_all else '; hidden'})"
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


def print_layers(rows, sim_files, unknown, as_json, size_note="bytes: " + SIZE_HOW["decompile"],
                 entered=None):
    """The table, the two files lines and the board's lines; with `entered`
    (`entered_rows`' dict, Loop 3350) four columns more, `file:n/e` in the
    by-source-file line and a `Census entered:` line after the byte line.
    Without it the output is what it was before the column existed."""
    import json
    sim = rows.get("simulation", EMPTY_ROW)
    ent_sim = {**EMPTY_ENTERED, **(entered["layers"].get("simulation", {}) if entered else {})}
    if as_json:
        doc = {"layers": rows, "simulation_backed": sim["backed"],
               "simulation_port_backed": sim["port"],
               "simulation_functions": sim["functions"],
               "simulation_bytes": sim.get("bytes", 0),
               "simulation_backed_bytes": sim.get("backed_bytes", 0),
               "bytes_source": size_note}
        if entered is not None:
            doc.update({"entered_layers": entered["layers"],
                        **{"simulation_" + k: v for k, v in ent_sim.items()}})
        print(json.dumps(doc, indent=1, sort_keys=True))
        return
    print(f"# {size_note}")
    if entered is not None:
        print("# entered: a pinned coverage trace entered it (`entered_in | hooked_in`, as `--pin --never`); "
              "ent unbacked: entered and not backed (Loop 3350, joined to nothing)")
    ehead = (f" {'entered':>8} {'ent unbacked':>12} {'ent bytes':>10} {'ent unb bytes':>13}"
             if entered is not None else "")
    print(f"{'layer':12} {'functions':>9} {'backed':>7} {'by doc':>7} {'by sweep':>8} {'port':>6} {'layout':>7} "
          f"{'share':>7} {'bytes':>10} {'backed bytes':>12} {'byte share':>10}" + ehead)
    for lay in ("simulation", "AI", "engine", "interface", "unknown"):
        r = {**EMPTY_ROW, **rows.get(lay, {})}
        share = f"{100 * r['backed'] / r['functions']:.1f}%" if r["functions"] else "-"
        bshare = f"{100 * r['backed_bytes'] / r['bytes']:.1f}%" if r["bytes"] else "-"
        ecols = ""
        if entered is not None:
            e = {**EMPTY_ENTERED, **entered["layers"].get(lay, {})}
            ecols = (f" {e['entered']:8} {e['entered_unbacked']:12} {e['entered_bytes']:10} "
                     f"{e['entered_unbacked_bytes']:13}")
        print(f"{lay:12} {r['functions']:9} {r['backed']:7} {r['doc']:7} {r['sweep']:8} {r['port']:6} "
              f"{r['backed'] - r['port']:7} {share:>7} {r['bytes']:10} {r['backed_bytes']:12} {bshare:>10}"
              + ecols)
    print()
    if entered is None:
        print("# simulation, by source file (rule 5's default — a misfiled file shows here)")
        print(" ".join(f"{f}:{n}" for f, n in sim_files.most_common()))
    else:
        print("# simulation, by source file, as functions/entered (rule 5's default — a misfiled file shows here)")
        print(" ".join(f"{f}:{n}/{entered['files'].get(f, 0)}" for f, n in sim_files.most_common()))
    print()
    print("# unknown, by where the source sits")
    print(" ".join(f"{f}:{n}" for f, n in unknown.most_common()))
    print()
    print(census_line(sim["backed"], sim["port"], sim["functions"]))
    print(census_bytes_line(sim.get("backed_bytes", 0), sim.get("bytes", 0)))
    if entered is not None:
        print(census_entered_line(ent_sim["entered"], sim["functions"], ent_sim["entered_unbacked"],
                                  ent_sim["entered_bytes"], sim.get("bytes", 0)))


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
# The file carries addresses and the backing kind only — never names:
# CLAUDE.md keeps symbol lists out of the repo, and a name is looked up in
# the export when a loss is reported. The old three-column form still reads.
BACKED_ROW = re.compile(r"^([0-9a-f]{8})\t(?:([^\t]+)\t)?(doc|sweep)$")


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
    """A file row: ADDR<tab>by. The name stays in the export (CLAUDE.md)."""
    va, _name, by = row
    return f"{va:08x}\t{by}"


def format_lost(row):
    """A reported loss: ADDR<tab>Class::method<tab>by, the name from the export."""
    va, name, by = row
    return f"{va:08x}\t{name or '?'}\t{by}"


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
            f"# `tools/census.py --backed-list` counts them: ADDR<tab>by (doc|sweep) — addresses only, the\n"
            f"# names stay in the export (CLAUDE.md keeps symbol lists out of the repo).\n"
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
                sys.exit(f"{path}:{n}: not ADDR<tab>doc|sweep: {line!r}")
            out[int(m.group(1), 16)] = (m.group(2) or '', m.group(3))
    return out


def lost_backed(listed, live):
    """The rows of `listed` (`read_backed`'s dict) whose address `live`
    (a set of addresses) no longer holds — by address, so a rename is not
    a loss."""
    return [(va, name, by) for va, (name, by) in sorted(listed.items()) if va not in live]


# --- the territory map (Loop 3727; Ramon, the sixtieth pass's talk) ----------
#
# The scoreboard's first table: every simulation-layer function classified
# by what this port has done with it, per PDB source stem. The headline
# counters flattered — `lockstep bytes` counts functions a whole game
# entered and says nothing of whether their outputs were compared, and the
# census's `backed` holds 1,167 functions whose only backing is a sweep test
# asserting a raw word (a layout, not a port). The kinds, in the order the
# table prints them:
#
#   port      backed, and `crates/sim` consumes it (`is_port_backed`, the
#             census's own `port` column: a coverage section diffs it, or the
#             crate spells its `Class::method` outside `sweep/` — item 4039;
#             before it the map also asked for a `fn` of the method's own
#             snake-case name, so a twin by another name never counted)
#   replaced  not ported, in a layer the project replaced on purpose — the
#             mark below, given per group on Ramon's word, reversible per
#             group if a port is wanted after all
#   layout    backed by a sweep alone: the offset identity, not a consumer
#   name      unbacked; the crate defines the method's snake-case name
#   open      unbacked, unmarked — the work
#   dead/ui/op/cut  `is_dead_weight`, `is_ui_body`, an `operator`, `is_cut`
#
# `port + replaced + open-ish (layout + name + open) + excluded (dead + ui +
# op + cut)` is the simulation layer whole; the `Territory:` line names the
# first three against the layer's count. The mark never touches a ported
# function (a port in a replaced layer stays a port) nor an excluded one.
#
# DECISIONS 114 §6 (the seventy-fourth pass, Ramon and lore's scoring of the
# seventy-first tranche's 26 readings, `docs/audit/2026-11-06-territory-*.md`)
# extended three groups by the readers' rows — the XML loaders by 21 names,
# the scripting VM by the trigger VM's five accessors, the turn pump by
# `Game::loop` — named the port's issuer the owner of the 25 `GroupOut` and
# `UnitOut` issuers, and added the recorded-game playback; item 4039 wrote them, each group with
# its owner and the trigger that reopens it.
REPLACED_BY_DESIGN = (
    # label, the stems marked whole, a name rule for functions marked by
    # name, and the owner or reversal trigger the group carries
    ("the command packaging and manager", frozenset(("commandpackage", "commandmanager")),
     # The client half of the command stream is the 34 `GroupOut`/`UnitOut`
     # `issue_*` bodies: the interface packs a selection's intent into a
     # `CommandPackage`. The port mints its own (`client::input`,
     # docs/CLIENT_INPUT.md), and the issuers are that issuer's specification
     # (lore's group B, 25 of them open on 2026-11-06). The census's `*Out`
     # mark excludes them as `ui` before this mark is asked (defect A's
     # ordering), so no rule here names them; the owner does.
     None,
     "owner of the `GroupOut`/`UnitOut` issuers (excluded as `ui`): the port's issuer, `client::input` "
     "(docs/CLIENT_INPUT.md); reopened if the client must package commands byte for byte"),
    ("the turn pump", frozenset(("turncontrol", "timesync")), re.compile(r"^Game::loop$"),
     "reopened at networking: the lockstep scheduler is the first group reversed when that phase opens"),
    # The engine's XML parser is the `engine` layer already; these are the
    # simulation-layer bodies that walk a parsed document into a table —
    # `ObjectType::load_support`, `Type::load_cost`, `Tribe::parse`,
    # `Map::load_map_data` — which `crates/rondata`'s typed loaders replace;
    # and the 21 the readers named, each with its twin in `crates/rondata`
    # (lore's group A). `Map::scale_number` is in it as proposed, though it
    # carries the one rule inside a loader (`SCALEWATER`/`SCALELAND`, the
    # original's float path; 4039a).
    ("the XML loaders", frozenset(),
     re.compile(r"::(load_(map_data|from_config|from_playback|support|upgrade|cost|sound_tables|old_bh3)"
                r"|parse(_cmd|_element)?|get_parse_name|set_parse_data|get_(abbrev|desc)_parsed)$"
                r"|^(Balance::(fill_tables|lookup_name|lookup_absolute_name)|Constants::get_(fraction|item)"
                r"|Map::(scale_number|set_file|scale_by_axis)|Categories::find_key|Lands::find_key"
                r"|TechType::set_research|Mountains::add_range|RoadsPieces::rotate_flags"
                r"|Type::(lookup_display_name|set_name)|Types::(get_type_index|good_key|build_key|unit_key"
                r"|tech_key|object_key))$"),
     "reopened if a table `crates/rondata` loads parts from the original's (the twins are there)"),
    # The VM itself (`game\script\`) is the AI layer; in the simulation
    # layer the VM's only hands are the map script's, and the trigger VM's
    # accessor table (no static caller in the export) and its one helper
    # (lore's group C).
    ("the scripting VM", frozenset(),
     re.compile(r"::(run_map_script|get_full_script_path)$"
                r"|^(GameAccessConst::(get_scary_message_win|get_scary_graphic_pieces|get_scary_console_who"
                r"|find_data_z)|ObjectData::in_a_ship)$"),
     "reopened with the trigger system, cut from v1 (CLAUDE.md)"),
    # `orders.cpp` is `walk_data`, `get_type`, the `get_*_order` casts and
    # `copy_order`/`get_new_order`; an order's *semantics* live in
    # `unit.cpp` and `groups.cpp`, which the mark does not touch.
    ("the order classes' plumbing", frozenset(("orders",)), None,
     "reopened if an order's walk must be the original's (the checksum track's data walk)"),
    # No skirmish enters it; run1036's playback does. The port replays from
    # the original's draw stream and its own orders, never from a `.rcx`
    # (lore's group F).
    ("the recorded-game playback", frozenset(),
     re.compile(r"^(Game::run_playback|RecordGame::(read_header|read_package|read_ctw_header))$"),
     "reopened if the port must play the original's recorded-game file itself"),
)

# The rows the territory line names inside `open` (DECISIONS 114 §6): two
# groups the readers proposed as `replaced` that are tracks' work instead —
# lore's E, the start builder, the start track's (docs/TRACKS.md; today the
# harness installs the `[Start Game]` state from the dump), and G, the data
# walk, the checksum track's (the original's full-state checksum, CHECKSUM.md)
# — and the fourth mark: the 66 rows the readers ruled `unmodelled`, a
# playing game's rule the crate does not have, the 1:1's finish and nothing
# more. Three of the 66 the adjudication ruled "B (twin, unwired)"
# (`Map{Himalayas,AmazonBasin,OldWorld}::make_continents`: Great Sahara's
# listing, with no route to it) and they stay in it until a style routes
# there. The mark never moves a row out of `open`: it says whose it is.
TERRITORY_NAMED = (
    ("the start track", re.compile(
        r"^Setup::(large_city_buildings|small_city_buildings|build_units|build_civ_specific|build_cities"
        r"|build_empire|place_unit|build_game|get_starting_citizens)$")),
    # Item 4170: and every other walk of state, the readers' `walk` class
    # (`saveload` in their rulings): the checksum's walk is these bodies,
    # `docs/CHECKSUM.md` §22 credits the ones the whole lines diff, and the
    # save track's file is seed and orders, never this walk (docs/TRACKS.md).
    ("the checksum track", re.compile(
        r"^(SpellType::walk_rules_data|SubObject::(walk_data|must_walk)|UnitType::walk_rules_data"
        r"|Tribe::walk_rules_data|Type::walk_rules_data|Types::(walk_data|walk_rules_data)|World::walk_data)$"
        r"|::(walk_(\w+_)?data|must_walk|walk_roads)$")),
    ("the rules' 1:1 finish", None),
)
# The fourth mark's rows, by name: the first readings' `unmodelled`.
UNMODELLED = frozenset((
    "Build::replace_gather", "BuildData::attack", "BuildData::get_base_garrison_arrows",
    "BuildData::hits", "BuildTypeData::check_enemy_adjacent", "Caravan::restart_trade_route",
    "Caravans::new_danger", "CliffsData::find_nearest", "CliffsData::gather_size", "Game::check_victory",
    "Game::defeat_all", "Game::get_armageddon", "Game::is_victory_timer", "Game::popwin_timer",
    "Game::retake_capital", "Game::wonder_timer", "GoodData::is_seen", "GoodTypeData::is_herd_type",
    "Group::action_siege_attack", "Group::action_unitmask", "Group::kill_group_order",
    "Map::eliminate_edge_canals", "Map::fill_cont", "Map::find_region_centroid", "Map::is_near_ocean",
    "Map::make", "Map::place_start_in_region", "Map::set_map", "MapAfricanWateringHole::make_continents",
    "MapAmazonBasin::make_continents", "MapAustralianOutback::make_continents",
    "MapBritishIsles::make_continents", "MapColonialPowers::make_continents",
    "MapEastMeetsWest::make_continents", "MapFairness::calc_distances", "MapHimalayas::make_continents",
    "MapMediterranean::make_continents", "MapNewWorld::make_continents", "MapNileDelta::make_continents",
    "MapOldWorld::make_continents", "MapSeaPower::make_continents", "MapWarringStates::make_continents",
    "Object::attempt_launch", "Objects::remove", "Region::release", "RoadsPieces::find_map",
    "SpecialsData::find_special", "SpellType::cast_blow_tread", "SpellType::cast_depopulate",
    "Terrain::mark_forest", "Terrain::object_placed", "Terrain::refresh_good_z",
    "TypeData::get_score_value", "TypeData::is_cancel_alliance", "TypeData::is_declare_war",
    "Unit::action_come_out", "Unit::action_unqueue", "Unit::think_carry_idle",
    "Unit::update_ceo_position", "UnitData::get_next_action", "UnitData::has_repeat_air",
    "UnitData::is_gathering", "UnitData::next_action_type", "UnitTypeData::get_kill_value",
    "World::analyze_map", "WorldData::is_really_seen",
))


def named_group(name):
    """The `TERRITORY_NAMED` label an open function carries, or None."""
    for label, rule in TERRITORY_NAMED:
        if (name in UNMODELLED) if rule is None else rule.search(name):
            return label
    return None

TERRITORY_KINDS = ("port", "replaced", "layout", "name", "open", "dead", "ui", "op", "cut")
TERRITORY_OPEN = ("layout", "name", "open")


def replaced_group(stem, name):
    """The `REPLACED_BY_DESIGN` label a simulation function falls under, or
    None: by its source stem, else by its name."""
    for label, stems, rule, _owner in REPLACED_BY_DESIGN:
        if stem in stems or (rule is not None and rule.search(name)):
            return label
    return None


def default_exe(pdb):
    """The executable beside the PDB: `<install>/riseofnations.exe` for
    `<install>/sbl/rise.pdb`."""
    return os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(pdb))), "riseofnations.exe")


def unreferenced(exe, targets, starts):
    """The targets the image holds no reference to (item 4170; the dead-scan
    of `tools/trace/report.py refs`, docs/EMULATOR.md §4, run over a set): no
    `call`/`jmp`/`jcc rel32` from another function, no rel8 jump from another
    function, and no four-byte copy of the address or its RVA at any offset of
    any section but `.reloc` — `.rdata`'s vtables, the function-pointer
    tables and `.text`'s immediates included. It counts every byte offset and
    confirms no site against the listing, so a word inside another
    instruction counts as a reference: it may call a dead function live (the
    blind list's scan, which confirms its sites, finds 8 such of the 121 open
    rows it calls unreferenced), never a live one dead. `starts` is every
    function's entry, sorted, to name the function a site is in."""
    import bisect
    import importlib.util
    import struct
    from array import array
    spec = importlib.util.spec_from_file_location("report", os.path.join(HERE, "trace", "report.py"))
    report = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(report)
    with open(exe, "rb") as f:
        data = f.read()
    base, secs = report.pe_sections(data)
    tva, traw = next((va, raw) for n, va, raw in secs if n == ".text")
    want = set(targets)

    def func(a):
        i = bisect.bisect_right(starts, a) - 1
        return starts[i] if i >= 0 else None

    live = set()
    for opcode, width in ((b"\xe8", 1), (b"\xe9", 1)) + tuple((bytes([0x0F, c]), 2) for c in range(0x80, 0x90)):
        i = traw.find(opcode)
        while i >= 0:
            if i + width + 4 <= len(traw):
                site = tva + i
                to = (site + width + 4 + struct.unpack_from("<i", traw, i + width)[0]) & 0xFFFFFFFF
                if to in want and func(site) != to:
                    live.add(to)
            i = traw.find(opcode, i + 1)
    for t in want - live:
        for lo in range(max(tva, t - 0x81), min(tva + len(traw) - 1, t + 0x7F)):
            op = traw[lo - tva]
            if (op == 0xEB or 0x70 <= op <= 0x7F) and func(lo) != t and \
                    lo + 2 + struct.unpack_from("<b", traw, lo - tva + 1)[0] == t:
                live.add(t)
                break
    words = want | {t - base for t in want}
    for n, _va, raw in secs:
        if n == ".reloc":
            continue
        for k in range(4):
            a = array("I")
            a.frombytes(raw[k:k + 4 * ((len(raw) - k) // 4)])
            live |= {w if w in want else w + base for w in set(a) & words}
    return want - live


def territory_kind(va, name, by_doc, by_sweep, port, spelled, unref=()):
    """A simulation function's kind before the mark (`TERRITORY_KINDS` less
    `replaced`), by the tests `is_port_backed` and `rank_unbacked` apply.

    A port first, then the census's own excluded marks, then a layout
    (item 4039, defect A of DECISIONS 112 §3, accepted in 114 §6): the map
    asked `backed` before the marks, so 82 sweep-asserted bodies the census
    excludes — the `*Out` issuers, the loggers, the editor's — read `layout`
    and counted open. A port stays a port whatever its name says."""
    short = name.rsplit("::", 1)[-1]
    if is_port_backed(va, name, by_doc, by_sweep, spelled):
        return "port"
    if "operator" in short:
        return "op"
    if is_dead_weight(name):
        return "dead"
    if is_ui_body(name):
        return "ui"
    if is_cut(name):
        return "cut"
    # Item 4170: nothing in the image reaches it (`unreferenced`), or the
    # blind list's residue holds it — two readings ruled that no game this
    # project can stage enters it (`rondata::blind::RESIDUE`). The out-of-line
    # copy of a body every caller inlined is no work: its rule is the
    # caller's.
    if va in unref:
        return "dead"
    if va in by_doc or va in by_sweep:
        return "layout"
    if snake(short) in port:
        return "name"
    return "open"


def territory(index, docs, sim_src, pdb, exe=None):
    """The territory map: `(rows, funcs, totals, groups)` — `rows` `{stem:
    Counter}` over `TERRITORY_KINDS` plus `bytes` and `open_bytes`; `funcs`
    every simulation function as `(va, name, stem, kind, group)` with
    `kind` after the mark and `group` the mark's label or None; `totals`
    the Counter over kinds with `functions`; `groups` `{label: Counter}` of
    what each replaced group covers, by the kind it would have had."""
    full = index_entries(index)
    by_name = {}
    for va, name, _, _, _ in full:
        by_name.setdefault(name, []).append(va)
    source = source_of(source_ranges(pdb), [e[0] for e in full])
    by_doc = doc_backed(docs, by_name)
    by_sweep = sweep_backed(sim_src, by_name, os.path.join(HERE, "emu"))
    port, spelled = port_names(sim_src), port_spellings(sim_src)
    # The image scan runs over the functions the marks leave open and no
    # replaced group takes (item 4170): a group is a design ruling, and the
    # command manager's issuers are called by address from the DLL, which the
    # scan does not read. A machine without the executable says so rather
    # than counting them open quietly.
    exe = exe or default_exe(pdb)
    if not os.path.isfile(exe):
        sys.exit(f"territory: no executable at {exe} (the dead scan reads it; RON_INSTALL)")
    unmarked = set()
    for va, name, _, _, _ in full:
        path = source.get(va)
        if layer_of(path) == "simulation" and replaced_group(
                path.rsplit("\\", 1)[-1].rsplit(".", 1)[0], name) is None and territory_kind(
                va, name, by_doc, by_sweep, port, spelled) in TERRITORY_OPEN:
            unmarked.add(va)
    unref = unreferenced(exe, unmarked, sorted(e[0] for e in full)) | (unmarked & residue_addrs())
    rows, funcs, totals, groups = {}, [], Counter(), {}
    for va, name, _, size, _ in full:
        path = source.get(va)
        if layer_of(path) != "simulation":
            continue
        stem = path.rsplit("\\", 1)[-1].rsplit(".", 1)[0]
        kind = territory_kind(va, name, by_doc, by_sweep, port, spelled, unref)
        group = replaced_group(stem, name) if kind in TERRITORY_OPEN else None
        if group is not None:
            groups.setdefault(group, Counter())[kind] += 1
            kind = "replaced"
        elif kind in TERRITORY_OPEN:
            named = named_group(name)
            if named is not None:
                totals["named:" + named] += 1
        r = rows.setdefault(stem, Counter())
        r[kind] += 1
        r["bytes"] += size
        if kind in TERRITORY_OPEN:
            r["open_bytes"] += size
        totals[kind] += 1
        totals["functions"] += 1
        funcs.append((va, name, stem, kind, group))
    return rows, funcs, totals, groups


def implicit(funcs, entered, cited):
    """The implicit residue (DECISIONS 29 §2, the sixty-eighth pass): every
    simulation function no pinned trace entered and no document cites —
    the rows the blind list cannot hold, because it holds cited functions
    only. `funcs` is `territory()`'s, `entered` and `cited` sets of
    addresses; returns `(rows, kinds)` — `rows` the `open` and `name` kind
    as `(stem, va, name)` sorted by stem then address, `kinds` a Counter of
    every never-entered uncited function by its territory kind (the
    excluded kinds are counted and not listed: a dead-weight or editor
    body is residue on the census's own mark)."""
    kinds, rows = Counter(), []
    for va, name, stem, kind, _group in funcs:
        if va in entered or va in cited:
            continue
        kinds[kind] += 1
        if kind in ("open", "name"):
            rows.append((stem, va, name))
    return sorted(rows), kinds


def implicit_line(rows, kinds):
    never = sum(kinds.values())
    return (f"Implicit: {len(rows)} never entered and uncited (open {kinds.get('open', 0)} · "
            f"name {kinds.get('name', 0)}) of {never} never entered"
            + "".join(f" · {k} {kinds[k]}" for k in TERRITORY_KINDS if k not in ("open", "name") and kinds.get(k)))


def print_implicit(rows, kinds):
    by_stem = {}
    for stem, va, name in rows:
        by_stem.setdefault(stem, []).append((va, name))
    print("# the implicit residue: simulation functions no pinned trace entered and no document cites, "
          "by stem (DECISIONS 29 §2; `--territory --pin --implicit`)")
    for stem, lst in sorted(by_stem.items(), key=lambda kv: (-len(kv[1]), kv[0])):
        print(f"{stem:20} {len(lst):3}  " + ", ".join(f"{n}@{va:08x}" for va, n in lst))
    print()
    print(implicit_line(rows, kinds))


def territory_line(totals):
    """`Territory: simulation ported P · replaced by design R · open O of N`
    — `open` is every function neither ported, replaced nor excluded
    (`TERRITORY_OPEN`: layout-only, name-only, unmarked), the number the
    territory map says is left."""
    named = " · ".join(f"{label} {totals['named:' + label]}" for label, _ in TERRITORY_NAMED)
    return (f"Territory: simulation ported {totals['port']} · replaced by design {totals['replaced']} · "
            f"open {sum(totals[k] for k in TERRITORY_OPEN)} of {totals['functions']} (in it: {named})")


def print_territory(rows, totals, groups, top=None, as_json=False):
    if as_json:
        import json
        print(json.dumps({"rows": {k: dict(v) for k, v in rows.items()}, "totals": dict(totals),
                          "groups": {k: dict(v) for k, v in groups.items()}}, indent=1, sort_keys=True))
        return
    order = sorted(rows.items(), key=lambda kv: (-sum(kv[1][k] for k in TERRITORY_OPEN), kv[0]))
    print("# the territory map — simulation functions by PDB source stem and what this port did with each "
          "(Loop 3727); most open first")
    print("# open = layout + name + open: neither ported, replaced by design, nor dead/ui/operator/cut")
    print(f"{'stem':18} {'fns':>4} " + " ".join(f"{k:>8}" for k in TERRITORY_KINDS) + f" {'open b':>8}  replaced by design")
    shown = order if top is None else order[:top]
    for stem, r in shown:
        label = replaced_group(stem, "") or ("by name" if r["replaced"] else "")
        print(f"{stem:18} {sum(r[k] for k in TERRITORY_KINDS):4} " + " ".join(f"{r[k]:8}" for k in TERRITORY_KINDS)
              + f" {r['open_bytes']:8}  {label}")
    if top is not None and len(order) > top:
        print(f"… {len(order) - top} more stems (--top)")
    print(f"{'total':18} {totals['functions']:4} " + " ".join(f"{totals[k]:8}" for k in TERRITORY_KINDS))
    print("# replaced by design, per group (Ramon's word, 2026-10-27; reversible per group): "
          + "; ".join(f"{label} {sum(c.values())} ({', '.join(f'{k} {n}' for k, n in sorted(c.items()))})"
                      for label, c in groups.items()))
    for label, _stems, _rule, owner in REPLACED_BY_DESIGN:
        print(f"#   {label}: {owner}")
    print(territory_line(totals))


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
    ap.add_argument("--cites", action="append", default=[], metavar="DOC",
                    help="with --never: DOC's never-entered cites net of RESIDUE, `only here` marking "
                         "those no other document cites, ending on the line a booking quotes (Loop 3415)")
    ap.add_argument("--pin", action="store_true",
                    help="read the traces rondata::blind::TRACES pins, from $RON_GAMELOG_DIR")
    ap.add_argument("--territory", action="store_true",
                    help="the territory map (Loop 3727): simulation functions by source stem as ported, "
                         "replaced by design, layout-only, name-only, open, or dead/ui/operator/cut; "
                         "--json for the same as JSON, --top N for the first N stems")
    ap.add_argument("--implicit", action="store_true",
                    help="with --territory and traces (--pin or logs): the implicit residue — every simulation "
                         "function no pinned trace entered and no document cites, by stem, ending on an "
                         "`Implicit:` line (DECISIONS 29 §2; the sixty-eighth pass)")
    ap.add_argument("--layers", action="store_true",
                    help="the census by layer with the backed column and the byte-weighted one, and the "
                         "board's number (item 1467) with its `Census bytes:` twin")
    ap.add_argument("--json", action="store_true", help="with --layers: the same as JSON")
    ap.add_argument("--entered-top", type=int, default=20, metavar="N",
                    help="--layers with traces (--pin or logs): the N largest entered-and-unbacked "
                         "simulation functions after the board's lines (Loop 3350; 0 for none)")
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
    if a.cites and not a.never:
        ap.error("--cites goes with --never")
    if a.implicit and not a.territory:
        ap.error("--implicit goes with --territory")
    if a.territory:
        rows, funcs, totals, groups = territory(a.index, a.docs, a.sim_src, a.pdb)
        if a.implicit:
            logs = (pinned_logs() if a.pin else []) + a.logs
            if not logs:
                ap.error("--implicit needs traces: --pin, or logs")
            seen = entered_in(logs, a.index) | hooked_in(logs, a.index)
            irows, kinds = implicit(funcs, seen, cited_in(a.docs))
            print_implicit(irows, kinds)
            return
        print_territory(rows, totals, groups, None if a.top == 40 else a.top, a.json)
        return
    if a.layers:
        # The `entered` column needs traces (`--pin`, or logs given); with
        # none the table, the lines and the JSON are what they were.
        logs = (pinned_logs() if a.pin else []) + a.logs
        seen = (entered_in(logs, a.index) | hooked_in(logs, a.index)) if logs else None
        rows, sim_files, unknown, note, ent = layers_entered(a.index, a.docs, a.sim_src, a.pdb, seen)
        print_layers(rows, sim_files, unknown, a.json, size_note=note, entered=ent)
        if ent is not None and not a.json and a.entered_top:
            print()
            print_entered_unbacked(ent, a.entered_top)
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
        _total, _cls_of, name_of = load_index(a.index)
        lost = [(va, name or name_of.get(va, ''), by) for va, name, by in lost_backed(listed, live)]
        for row in lost:
            print("lost\t" + format_lost(row))
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
        if a.cites:
            per_doc = cited_per_doc(a.docs)
            residue = residue_addrs()
            lines = []
            for doc in a.cites:
                rows, n, m, k = doc_never(doc, per_doc, name_of, seen, residue)
                print()
                print(f"# {doc} — cited there, in the export, entered by no given trace")
                for v, name, mark in rows:
                    print(f"{name}@{v:08x}  {mark}")
                lines.append(cites_line(doc, n, m, k))
                print(lines[-1])
            print()
            for line in lines:
                print(line)
    print()
    print(f"functions {sum(total.values())}  classes {len(total)}  cited {len(cited)} in "
          f"{len(cited_by)} classes  entered {e_all}{never}  (logs: {len(a.logs)})")


if __name__ == "__main__":
    main()
