#!/usr/bin/env python3
r"""census.py — the executable as the denominator.

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


def sweep_backed(src, by_name):
    """Every function a `crates/sim` test that runs the original under the
    emulator cites, in its doc comment or its body."""
    out = set()
    for p in glob.glob(os.path.join(src, "**", "*.rs"), recursive=True):
        if os.path.basename(p) in NOT_A_SWEEP:
            continue
        with open(p, errors="replace") as f:
            text = f.read()
        for chunk in tests_in(text):
            if SWEEP.search(chunk):
                out |= named(chunk, by_name)
    return out


HOOKS = re.compile(r"Harness\(\s*[^,()]*,\s*\{(.*?)\}", re.S)
SCRIPT = re.compile(r"tools/emu/(sweep_[a-z0-9_]+\.py)")


def hooked_in_sweeps(src, emu_dir):
    """Every (test file, script, address) where a sweep test's chunk cites
    an address its own script hands `Harness(exe, {ADDR: …})` as a hooked
    callee — a function the sweep answers for, not one it runs, which the
    census would count as backed beside the one swept (parked 2012, Loop
    2025: 1998's first draft read 461 for ten). The brief's rule is to
    cite no hooked callee in the chunk; this is the audit of it."""
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
            for m in SCRIPT.finditer(chunk):
                script = os.path.join(emu_dir, m.group(1))
                if not os.path.isfile(script):
                    continue
                with open(script, errors="replace") as f:
                    hm = HOOKS.search(f.read())
                hooked = {int(a, 16) for a in re.findall(r"0x(00[0-9a-f]{6})", hm.group(1))} if hm else set()
                for addr in sorted(hooked & cited):
                    out.append((os.path.relpath(p, src), m.group(1), addr))
    return out


def layers(index, docs, sim_src, pdb):
    """The census by layer: {layer: {functions, backed, doc, sweep}}, the
    unknowns' sources, and the simulation layer's files."""
    names = {}
    by_name = {}
    with open(index) as f:
        for line in f:
            addr, name = line.rstrip("\n").split("\t")[:2]
            va = int(addr, 16)
            names[va] = name
            by_name.setdefault(name, []).append(va)
    source = source_of(source_ranges(pdb), names)
    by_doc = doc_backed(docs, by_name) & names.keys()
    by_sweep = sweep_backed(sim_src, by_name) & names.keys()
    rows, sim_files, unknown = {}, Counter(), Counter()
    for va, path in source.items():
        lay = layer_of(path)
        r = rows.setdefault(lay, {"functions": 0, "backed": 0, "doc": 0, "sweep": 0})
        r["functions"] += 1
        r["doc"] += va in by_doc
        r["sweep"] += va in by_sweep
        r["backed"] += va in by_doc or va in by_sweep
        if lay == "simulation":
            sim_files[path.rsplit("\\", 1)[-1]] += 1
        elif lay == "unknown":
            unknown[path.rsplit("\\", 2)[-2] + "\\" if path else "(no line record)"] += 1
    return rows, sim_files, unknown


PORT_FN = re.compile(r"\bfn ([a-z0-9_]+)\s*[(<]")


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


def rank_unbacked(entries, backed, port):
    """`(size, va, name, mark, file)` for every simulation function not yet
    backed, shortest decompile first (parked 1703: four batches wrote this
    lister as a scratch script). `entries` is `(va, name, source path,
    decompile size)`; `backed` the by-doc and by-sweep union; `port` the
    names `crates/sim` defines. The mark says what the sweep lane wants to
    know before it reads a function: `port` when this crate defines the
    method's snake-case name (a consumer the sweep can run against — parked
    1725), `operator` for a name the citation pattern spells with a symbol,
    `-` otherwise."""
    rows = []
    for va, name, path, size in entries:
        if layer_of(path) != "simulation" or va in backed:
            continue
        short = name.rsplit("::", 1)[-1]
        if "operator" in short:
            mark = "operator"
        elif is_dead_weight(name):
            mark = "skip"
        elif snake(short) in port:
            mark = "port"
        else:
            mark = "-"
        rows.append((size, va, name, mark, (path or "").rsplit("\\", 1)[-1]))
    rows.sort(key=lambda r: (r[0], r[1]))
    return rows


# The name classes every sweep lane skipped by hand (parked 1861): a
# destructor, a class's open/close/init/clear, a registration, a logger, a
# compiler's dynamic initializer. None reads a value a port can be held to.
DEAD_WEIGHT = re.compile(r"(^|::)(~|close|close_class|init|init_class|clear|reset|registration|"
                         r"log|log_data|log_state|dump|print|print_data|dynamic_initializer|"
                         r"dynamic_atexit|scalar_deleting|vector_deleting)")


def is_dead_weight(name):
    cls, _, short = name.rpartition("::")
    constructor = bool(cls) and short == cls.rsplit("::", 1)[-1]
    return (constructor or short.startswith("one_time_") or bool(DEAD_WEIGHT.search("::" + short))
            or "dynamic initializer" in name or "dynamic atexit" in name)


def worth_sweeping(rows):
    """The rows the sweep wants: not `skip`, not `operator` (parked 1722)."""
    return [r for r in rows if r[3] not in ("skip", "operator")]


def unbacked(index, docs, sim_src, pdb, top, show_all=False):
    """The never-backed simulation functions, shortest decompile first;
    the dead weight (`skip`) and the operators are hidden unless `show_all`."""
    root = os.path.dirname(index)
    entries, by_name = [], {}
    with open(index) as f:
        for line in f:
            parts = line.rstrip("\n").split("\t")
            if len(parts) < 3:
                continue
            va, name, rel = int(parts[0], 16), parts[1], parts[2]
            by_name.setdefault(name, []).append(va)
            full = os.path.join(root, rel)
            entries.append((va, name, None, os.path.getsize(full) if os.path.isfile(full) else 0))
    source = source_of(source_ranges(pdb), [e[0] for e in entries])
    entries = [(va, name, source.get(va), size) for va, name, _, size in entries]
    backed = (doc_backed(docs, by_name) | sweep_backed(sim_src, by_name))
    rows = rank_unbacked(entries, backed, port_names(sim_src))
    shown = rows if show_all else worth_sweeping(rows)
    print(f"{'bytes':>6} {'function':52} {'mark':8} file")
    for size, va, name, mark, fname in shown[:top]:
        print(f"{size:6} {name + '@' + format(va, '08x'):52} {mark:8} {fname}")
    marks = Counter(r[3] for r in rows)
    print(f"\n{len(rows)} never-backed simulation functions: {marks.get('port', 0)} with a port "
          f"consumer by name, {marks.get('operator', 0)} operators, {marks.get('skip', 0)} dead weight "
          f"(destructors, close/init/clear, loggers, initializers"
          f"{'; shown' if show_all else '; hidden, --all shows them'}); the {min(top, len(shown))} "
          f"shortest shown (--top)")


def print_layers(rows, sim_files, unknown, as_json):
    import json
    sim = rows.get("simulation", {"functions": 0, "backed": 0})
    if as_json:
        print(json.dumps({"layers": rows, "simulation_backed": sim["backed"],
                          "simulation_functions": sim["functions"]}, indent=1, sort_keys=True))
        return
    print(f"{'layer':12} {'functions':>9} {'backed':>7} {'by doc':>7} {'by sweep':>8} {'share':>7}")
    for lay in ("simulation", "AI", "engine", "interface", "unknown"):
        r = rows.get(lay, {"functions": 0, "backed": 0, "doc": 0, "sweep": 0})
        share = f"{100 * r['backed'] / r['functions']:.1f}%" if r["functions"] else "-"
        print(f"{lay:12} {r['functions']:9} {r['backed']:7} {r['doc']:7} {r['sweep']:8} {share:>7}")
    print()
    print("# simulation, by source file (rule 5's default — a misfiled file shows here)")
    print(" ".join(f"{f}:{n}" for f, n in sim_files.most_common()))
    print()
    print("# unknown, by where the source sits")
    print(" ".join(f"{f}:{n}" for f, n in unknown.most_common()))
    print()
    print(f"Census: simulation backed {sim['backed']} of {sim['functions']}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("logs", nargs="*")
    ap.add_argument("--index", default=os.path.expanduser("~/ghidra-projects/decomp/INDEX.tsv"))
    ap.add_argument("--docs", default=os.path.join(HERE, "..", "docs"))
    ap.add_argument("--top", type=int, default=40)
    ap.add_argument("--never", action="store_true")
    ap.add_argument("--pin", action="store_true",
                    help="read the traces rondata::blind::TRACES pins, from $RON_GAMELOG_DIR")
    ap.add_argument("--layers", action="store_true",
                    help="the census by layer with the backed column, and the board's number (item 1467)")
    ap.add_argument("--json", action="store_true", help="with --layers: the same as JSON")
    ap.add_argument("--unbacked", action="store_true",
                    help="the never-backed simulation functions, shortest decompile first, "
                         "with a port-consumer mark (parked 1703, 1725); --top rows")
    ap.add_argument("--pdb", default=default_pdb(HERE))
    ap.add_argument("--all", action="store_true",
                    help="--unbacked: show the dead weight and the operators too")
    ap.add_argument("--hooked", action="store_true",
                    help="the audit of Loop 2025: every sweep test chunk that cites an address its own "
                         "script hooks as a callee; exit 1 when any")
    ap.add_argument("--sim-src", default=os.path.join(HERE, "..", "crates", "sim", "src"))
    a = ap.parse_args()
    if a.layers:
        print_layers(*layers(a.index, a.docs, a.sim_src, a.pdb), a.json)
        return
    if a.unbacked:
        unbacked(a.index, a.docs, a.sim_src, a.pdb, a.top, a.all)
        return
    if a.hooked:
        rows = hooked_in_sweeps(a.sim_src, os.path.join(HERE, "emu"))
        for file, script, addr in rows:
            print(f"{file}\t{script}\t{addr:08x}")
        print(f"hooked callees cited in sweep chunks: {len(rows)}")
        sys.exit(1 if rows else 0)
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
