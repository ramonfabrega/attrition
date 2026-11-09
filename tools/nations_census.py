#!/usr/bin/env python3
"""Regenerates the census tables in docs/NATIONS.md (item 2976).

    python3 -I tools/nations_census.py [<export> <rules.xml>] [--write | --check]

It reads the decompile export (a reading tool, never a source) for every
`has_tribe_bonus` call and every `constants->name` read, the install's rules
for the constant names and values, and `crates/sim/src` for what is modelled,
and prints the machine-checked tables (`nations::tests::the_audit_table_*`
reads the same rows). Hand judgements live in HAND and OVERRIDES below, not in
the output: edit those, re-run, paste. Nothing from the install or the export
is written beyond a name, an address and a number.

The export defaults to `RON_DECOMP` or `~/ghidra-projects/decomp`, the rules
to `RON_INSTALL`'s (or the main checkout's `game/`) `Data/rules.xml`.
`--write` splices the tables into `docs/NATIONS.md`; `--check` (Loop 3972)
splices nothing and exits 1 naming every row the tree's tables differ from
the generator's — the round trip is empty or the guard says which rows are
not — and skips with exit 0 on a machine without the export or the install.
`tools/guard.sh` runs it.
"""
import argparse
import glob
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SIM = str(ROOT / "crates/sim/src")
SWEEPS = str(ROOT / "crates/sim/tests/sweep")
SWEEP_ONLY_AT_LEAST = 1200  # Loop 4044: 1,459 on 2026-11-09
DOC = ROOT / "docs/NATIONS.md"

PREFIX = [
    ("AZTEC", 0), ("MAYA", 1), ("INCA", 2), ("BANTU", 3), ("NUBIAN", 4),
    ("GREEK", 5), ("ROMAN", 6), ("EGYPT", 7), ("TURK", 8), ("SPANISH", 9),
    ("FRENCH", 10), ("BRITISH", 11), ("GERMAN", 12), ("RUSSIAN", 13),
    ("CHINESE", 14), ("JAPAN", 15), ("KOREAN", 16), ("MONGOL", 17),
    ("IROQ", 18), ("LAKOTA", 19), ("AMERICAN", 20), ("INDIAN", 21),
    ("DUTCH", 22), ("PERSIAN", 23),
]

src_raw, src_code, load_code = {}, {}, ""
sweep_only = set()  # file names found only under a `sweep/` (`crates/sim/src/sweep/`, `crates/sim/tests/sweep/`)


def best_file(pattern, code=True, literal=None):
    """The non-test file holding `pattern` most often. `literal`, a
    substring every match contains, skips the files that cannot hold it
    (the regex over every file was 16 of the tool's 23 seconds). A test,
    a harness or a `_view` module (a function's control flow over a view
    the emulator sweep answers) comes last; and for a field witness (a
    `modelled at` row) so does a module only `sweep/` holds — the
    document's `modelled at` names the live sim's module, its `carried at`
    may name a sweep's (Loop 3222, Loop 3972)."""
    pool = src_code if code else src_raw
    hits = [(len(re.findall(pattern, t)), k) for k, t in pool.items()
            if k != "tuning.rs" and (literal is None or literal in t) and re.search(pattern, t)]
    if not hits:
        return None
    hits.sort(key=lambda h: ("test" in h[1] or "harness" in h[1] or h[1].endswith("_view.rs")
                             or (code and h[1] in sweep_only), -h[0], h[1]))
    return hits[0][1]


# ---- the export ---------------------------------------------------------
def balanced(t, i):
    d = 0
    for j in range(i, len(t)):
        if t[j] == "(":
            d += 1
        elif t[j] == ")":
            d -= 1
            if d == 0:
                return j
    return len(t)


funcs = {}  # "Class::name@addr" -> (indices set, var flag)
const_readers = {}  # field -> set of Class::name@addr
rules, consts, tuning = "", [], ""


def load(export, rules_path):
    """Reads the sim's sources, the export and the rules into the module's
    tables (the tool was a script; its tables stay module-level)."""
    global src_raw, src_code, load_code, rules, consts, tuning, sweep_only
    # Item 4009 moved the standalone sweeps beside `src/`, into the crate's tests.
    paths = glob.glob(SIM + "/**/*.rs", recursive=True) + glob.glob(SWEEPS + "/**/*.rs", recursive=True)
    # Item 4010 moved `foo.rs`'s test tails to `foo/tests.rs` (and
    # `foo/<name>_tests.rs`): they are still `foo.rs`'s, as they were inline.
    # Item 4011's directories put them at any depth, and a `foo/mod.rs` is
    # `foo.rs`.
    def nested(p):
        d = os.path.dirname(p)
        rel = os.path.relpath(d, SIM)
        return d != SIM and not rel.startswith("..") and "sweep" not in rel.split(os.sep)

    def sibling(p):
        return nested(p) and os.path.basename(p)[:-3].endswith("tests")

    def key(p):
        if sibling(p) or (nested(p) and os.path.basename(p) == "mod.rs"):
            return os.path.basename(os.path.dirname(p)) + ".rs"
        return os.path.basename(p)
    src_raw = {key(p): open(p, errors="replace").read() for p in paths if not sibling(p)}
    for p in sorted(p for p in paths if sibling(p)):
        src_raw[key(p)] += open(p, errors="replace").read()
    dirs = {}
    for p in paths:
        # `sweep.rs` beside `sweep/` is the directory's module file (item 4032).
        dirs.setdefault(key(p), set()).add(os.path.basename(os.path.dirname(p)) == "sweep"
                                           or p == os.path.join(SIM, "sweep.rs"))
    sweep_only = {k for k, v in dirs.items() if v == {True}}
    # Loop 4044: the sweep is known by its directory's name here; the set
    # says its size (1,459 file names on 2026-11-09), so a sweep that moved
    # or was renamed fails the census rather than ranking its rows as the
    # simulation's witnesses.
    if len(sweep_only) < SWEEP_ONLY_AT_LEAST:
        raise SystemExit(f"nations_census: {len(sweep_only)} sweep-only files, under the floor "
                         f"of {SWEEP_ONLY_AT_LEAST}: the sweep moved (Loop 4044)")
    load_code = re.sub(r"//[^\n]*", "", open(ROOT / "crates/rondata/src/load.rs", errors="replace").read())
    src_code = {k: re.sub(r"//[^\n]*", "", v) for k, v in src_raw.items()}
    funcs.clear()
    const_readers.clear()
    scan_export(str(export))
    rules = open(rules_path, errors="replace").read()
    consts = [(n, v or ("[" + e + ", ...]"))
              for n, v, e in re.findall(r'<([A-Z][A-Z_0-9]*) (?:value="([^"]*)"|entry0="([^"]*)")', rules)
              if any(n.startswith(p) for p, _ in PREFIX)]
    tuning = src_raw["tuning.rs"]


def scan_export(export):
    for path in glob.glob(export + "/funcs/*/*.c"):
        cls = path.split("/")[-2]
        base = os.path.basename(path)[:-2]
        name, addr = base.split("@")
        full = f"{cls}::{name}@{addr}"
        t = open(path, errors="replace").read()
        idx, var = set(), False
        for m in re.finditer(r"has_tribe_bonus", t):
            k = t.find("(", m.end())
            if k < 0 or k - m.end() > 8:
                continue
            last = t[k + 1:balanced(t, k)].split(",")[-1].strip()
            if re.fullmatch(r"0x[0-9a-fA-F]+|\d+", last) and int(last, 0) < 24:
                idx.add(int(last, 0))
            elif re.fullmatch(r"0x[0-9a-fA-F]+|\d+", last):
                pass  # a stray number, not a roster index
            else:
                var = True
        if idx or var:
            funcs[full] = (idx, var)
        if name in ("parse", "load", "init_constants") or cls in ("Tribe", "Constants", "ConstantsData"):
            continue  # the loaders write the constant; they do not consume it
        for m in re.finditer(r"->([a-z_0-9]+)\b", t):
            const_readers.setdefault(m.group(1), set()).add(full)


def addr_of(full):
    return full.split("@")[1]


# ---- the rules ----------------------------------------------------------


def nation_of(n):
    for p, i in PREFIX:
        if n.startswith(p):
            return i
    raise KeyError(n)


def num(v):
    if v.startswith("["):
        return v
    m = re.match(r"\s*(-?\d+(?:/\d+)?)", v)
    return m.group(1) if m else "?"


def const_row(n, v):
    fld = n.lower()
    readers = sorted(const_readers.get(fld, ()))
    where = ", ".join(r.split("::")[0] + "::" + r.split("::")[1].split("@")[0] for r in readers[:3])
    if len(readers) > 3:
        where += f" +{len(readers) - 3}"
    where = where or "no reader in the export"
    if not re.search(r"pub " + fld + r"\s*:", tuning):
        st, wit = "not carried", fld
    else:
        f = best_file(r"\b" + fld + r"\b", literal=fld)
        if f:
            st, wit = f"modelled at {f}", fld
        elif re.search(r"\b" + fld + r"\b", load_code):
            st, wit = "loaded at load.rs", fld
        else:
            st, wit = "carried, unread", fld
    return f"| `{n}` | {num(v)} | {where} | {st} | `{wit}` |"


# ---- hand judgements ----------------------------------------------------
# (nation, arm, original site, status, witness)
HAND = [
    ("Egyptians", "wonders cost 25% less", "TypeData::get_cost@00664090", "modelled at cost.rs", "egyptian_wonders"),
    ("Egyptians", "scholars swap food and wealth (gate is 0 in the rules)", "TypeData::get_cost@00664090", "not modelled", "egyptian_scholars_food"),
    ("Egyptians", "granary available early", "LeaderData::has_preq@006db810", "modelled at tech.rs", "egyptian_granary_early"),
    ("Egyptians", "wonders one age early", "LeaderData::special_preq@006d7030", "modelled at tech.rs", "egyptian_wonders_early"),
    ("Egyptians", "one more wonder per city (placement)", "BuildTypeData::blocked_location@006375b0", "modelled at location.rs", "n.egyptians"),
    ("Egyptians", "the same limit enforced at construction", "Wall::do_construct@006434d0", "modelled at city.rs", "006434d0"),
    ("Egyptians", "farm limit per city", "LeaderData::get_farm_limit@006db270", "modelled at city.rs", "egyptian_farms_per_city_base"),
    ("Egyptians", "food commerce bonus", "Leader::calc_resource_caps@006ce900", "modelled at economy.rs", "egyptian_food_commerce"),
    ("Egyptians", "granary upgrades applied on tech gain", "Leader::gain_tech@006dcb60", "loaded at load.rs", "egyptian_granary_upgrades"),
    ("Russians", "attrition doubled", "Leader::calc_attrition@006cdea0", "modelled at attrition.rs", "russian_attrition"),
    ("Russians", "oil +20% in the rate pass", "LeaderData::calc_resource_bonuses@006db030", "modelled at economy.rs", "russian_oil"),
    ("Russians", "spy cost 50% less", "TypeData::get_cost@00664090", "modelled at cost.rs", "russian_spy_cost"),
    ("Russians", "borders grow per age", "World::compute_reg_territory@006b0bb0", "modelled at territory.rs", "russian_borders_per_age"),
    ("Russians", "gems' territory bonus halved", "World::compute_reg_territory@006b0bb0", "modelled at territory.rs", "n.russian"),
    ("Russians", "plunder steals from the plundered", "Build::plunder@00623660", "modelled at city.rs", "russian_plunder_steal"),
    ("Russians", "a free civic at start", "Leader::init@006e3930", "modelled at tech.rs", "russian_free_civic"),
    ("Russians", "oil well counts as held (has_preq)", "LeaderData::has_preq@006db810", "modelled at tech.rs", "has_tribe_bonus(setup, p, 13)"),
    ("Russians", "attrition upgrades applied on tech gain", "Leader::gain_tech@006dcb60", "loaded at load.rs", "russian_attrition_upgrades"),
    ("Russians", "market prices pinned at 100 after the fifth age (gate is 0)", "LeaderData::calc_market_prices@006dc2a0", "carried, unread", "russian_communism"),
    ("Chinese", "science techs 20% cheaper", "TypeData::get_cost@00664090", "modelled at queue.rs", "chinese_science_discount"),
    ("Chinese", "citizens train twice as fast", "ObjectData::train_time@006508c0", "modelled at queue.rs", "chinese_citizen_creation"),
    ("Chinese", "citizens train instantly (the rules' CHINESE_CITIZEN_INSTANT is 1, the arm returns 1)", "ObjectData::train_time@006508c0", "modelled at queue.rs", "chinese_citizen_instant"),
    ("Chinese", "a large-city building on a new city", "Build::activate@00623e20", "modelled at activate.rs", "chinese_large_cities"),
    ("Chinese", "a large-city building on assimilation", "City::assimilate@00738e90", "modelled at city_views.rs", "chinese_large_cities"),
    ("Chinese", "herbal lore applied on tech gain", "Leader::gain_tech@006dcb60", "loaded at load.rs", "chinese_herbal_lore"),
    ("Chinese", "knowledge upgrades applied on tech gain (0 in the rules)", "Leader::gain_tech@006dcb60", "carried, unread", "chinese_knowledge_upgrades"),
]

HAND += [
    ("Greeks", "an age or an epoch researches faster (train_time's research tail)", "ObjectData::train_time@006508c0", "modelled at queue.rs", "greek_research_speed"),
    ("Greeks", "an age or an epoch costs 10% less in every good but knowledge", "TypeData::get_cost@00664090", "modelled at queue.rs", "greek_research_cost"),
    ("Greeks", "a Library or a University costs 66% less", "TypeData::get_cost@00664090", "modelled at city.rs", "greek_library_cost"),
    ("Romans", "legions train faster", "ObjectData::train_time@006508c0", "modelled at queue.rs", "roman_legion_fast"),
    ("French", "siege units train faster", "ObjectData::train_time@006508c0", "modelled at queue.rs", "french_siege_speed"),
    ("French", "special (general) units train faster, 0 in the rules", "ObjectData::train_time@006508c0", "not modelled", "french_special_speed"),
    ("Germans", "aircraft train faster", "ObjectData::train_time@006508c0", "modelled at queue.rs", "german_air_speed"),
    ("Germans", "submarines train faster", "ObjectData::train_time@006508c0", "modelled at queue.rs", "german_submarine_speed"),
    ("Japanese", "barracks units train faster (negative in the rules)", "ObjectData::train_time@006508c0", "modelled at queue.rs", "japanese_barracks_speed"),
    ("Japanese", "carriers train faster", "ObjectData::train_time@006508c0", "modelled at queue.rs", "japanese_carrier_speed"),
    ("Japanese", "a Farm costs 50% less", "TypeData::get_cost@00664090", "modelled at city.rs", "japanese_farms_cheaper"),
    ("Japanese", "a farmer gathers 25% more food (calc_gather's flat branch)", "BuildTypeData::calc_gather@00639e40", "modelled at holdings.rs", "japanese_fishing_boats"),
    ("Indians", "a building's support ramp zeroed, halved rounding up for a wonder or a defence", "TypeData::get_cost@00664090", "modelled at cost.rs", "indian_ramp"),
    ("Indians", "an elephant-line unit costs 15% less, the last step", "TypeData::get_cost@00664090", "modelled at cost.rs", "indian_elephant"),
    ("Lakota", "raze speed (train_time's tail)", "ObjectData::train_time@006508c0", "modelled at queue_disband.rs", "lakota_raze_speed"),
    ("Mongols", "stable units train faster", "ObjectData::train_time@006508c0", "modelled at queue.rs", "mongol_stable_speed"),
    ("British", "ships, archers and anti-air train faster", "ObjectData::train_time@006508c0", "modelled at queue.rs", "british_ship_speed"),
]

# Function-level overrides for the census: a function the sim carries without
# citing its address, or a class that never runs in a skirmish.
OVERRIDES = {
    "LeaderData::get_farm_limit@006db270": ("modelled at city.rs", "egyptian_farms_per_city_base"),
    "LeaderData::special_preq@006d7030": ("modelled at tech.rs", "egyptian_wonders_early"),
    "Leader::calc_anti_attrition@006cdcc0": ("modelled at attrition.rs", "mongol_attrition"),
    "BuildTypeData::blocked_tcoord@00636db0": ("modelled at place.rs", "dutch_fort_placement"),
    "BuildData::construct_time@0062d5c0": ("modelled at build.rs", "iroquois_quick_senate"),
    "Build::plunder@00623660": ("modelled at city.rs", "russian_plunder_steal"),
    "Leader::lost_a_city@006dc6d0": ("carried at city.rs", "fn lost_a_city"),
    # Its walk view left city.rs at item 4031; the sweep file cites the
    # address more often, and the census names the live module.
    "City::assimilate@00738e90": ("carried at city_views.rs", "00738e90"),
    # The Despot's arm is built (fight.rs, item 3279), and the Aztec slay
    # arm `take_damage` calls with `param_3 = 1` (item 3657): its pay is read
    # in plunder.rs's view.
    "Unit::plunder@00605b00": ("modelled at plunder.rs", "aztec_slay_bonus"),
    # Its Spanish arm is leader state, not the start: `leader_flags |=
    # 0x1000`, which `sees_every_cell` carries (item 3318).
    "Setup::build_empire@005abb80": ("modelled at ai_sites.rs", "sees_every_cell"),
}
NOT_SIM = ("Setup::", "ConquestGame::", "ScenarioFuncSet::", "IFace", "StatWin::",
           "Options::", "Scene::", "BuildTypeOut::", "BuildOut::", "LeaderOut::",
           "TypeData::get_desc_parsed", "Border::sort_clump_list")
NOT_SIM_WHY = {
    "Setup::": "the lobby's start builder; the harness takes the start state from the dump",
    "ConquestGame::": "Conquer the World, cut from v1",
    "ScenarioFuncSet::": "the scenario trigger set, cut from v1",
}


def census_row(full, idx, var):
    cls, rest = full.split("::", 1)
    name, addr = rest.split("@")
    ids = " ".join(str(i) for i in sorted(idx)) + (" +var" if var else "")
    if full in OVERRIDES:
        st, wit = OVERRIDES[full]
    elif any(full.startswith(p) for p in NOT_SIM):
        why = next((w for p, w in NOT_SIM_WHY.items() if full.startswith(p)), "interface or render side")
        return f"| `{full}` | {ids} | not sim | `{why}` |"
    else:
        f = best_file(addr, code=False, literal=addr)
        if f:
            st, wit = f"carried at {f}", addr
        else:
            st, wit = "not modelled", addr
    return f"| `{full}` | {ids} | {st} | `{wit}` |"


ROSTER = ["Aztecs", "Maya", "Inca", "Bantu", "Nubians", "Greeks", "Romans", "Egyptians",
          "Turks", "Spanish", "French", "British", "Germans", "Russians", "Chinese",
          "Japanese", "Koreans", "Mongols", "Iroquois", "Lakota", "Americans", "Indians",
          "Dutch", "Persians"]
LOBBY = {0: "`GreatSaharaAztecTier2` (AI word, tier 2); `EastIndiesAztecTier1` (map 18, tier 1)", 7: "`GreatSaharaEgyptianTier4` (AI word, tier 4), `EastIndiesEgyptianTier3` (map 18, tier 3)", 12: "`EastIndiesGermanTier3` (AI word, tier 3); `GreatSaharaGermanTier3` (AI word, tier 3)",
         18: "`GreatLakesIroquoisTier3` (AI word, tier 3); `GreatSaharaIroquoisTier3` (AI word, tier 3)", 22: "`EastIndiesDutchTier4` (AI word, tier 4); `GreatSaharaDutchTier3` (AI word, tier 3); `GreatLakesDutchTier4` (map 14, tier 4)",
         21: "`EastIndiesIndianTier2` (AI word, tier 2); `GreatSaharaIndianTier3` (AI word, tier 3)",
         1: "`GreatSaharaMayaTier4` (AI word, tier 4)", 3: "`GreatSaharaBantuTier4` (map 7, tier 4)",
         13: "`GreatSaharaRussianTier4` (map 7, tier 4); `GreatLakesRussianTier2` (AI word, tier 2)",
         19: "`GreatLakesLakotaTier2` (AI word, tier 2); `GreatSaharaLakotaTier2` (map 7, tier 2)", 23: "`GreatLakesPersianTier2` (AI word, tier 2)", 14: "`GreatLakesChineseTier1` (map 14, tier 1)",
         9: "`GreatSaharaSpanishTier3` (AI word, tier 3), `EastIndiesSpanishTier3` (AI word, tier 3); `GreatLakesSpanishTier4` (AI word, tier 4)",
         2: "`EastIndiesIncaTier3` (AI word, tier 3); `GreatSaharaIncaTier3` (AI word, tier 3)", 5: "`GreatSaharaGreekTier1` (AI word, tier 1)",
         15: "`GreatSaharaJapaneseTier1` (map 7, tier 1)",
         6: "`EastIndiesRomanTier2` (AI word, tier 2), `GreatLakesRomanTier2` (map 14, tier 2)", 20: "`EastIndiesAmericanTier1` (AI word, tier 1); `GreatLakesAmericanTier4` (AI word, tier 4)", 8: "`GreatLakesTurkishTier4` (AI word, tier 4); `GreatSaharaTurkishTier3` (map 7, tier 3)",
         4: "the human in every capture", 10: "`GreatSaharaFrenchTier4` (AI word, tier 4)", 11: "scored pairs", 16: "`GreatSaharaKoreanTier3` (AI word, tier 3); scored pairs"}
FLAG = {0: "aztecs", 1: "maya", 2: "inca", 3: "bantu", 4: "nubians", 6: "romans", 7: "egyptians",
        8: "turks", 10: "french", 11: "british", 12: "germans", 13: "russians", 14: "chinese",
        16: "koreans", 17: "mongols", 19: "lakota", 21: "indians", 22: "dutch"}


def generate(export, rules_path):
    """The generated tables, each block under its `GENERATED` marker."""
    load(export, rules_path)
    out = []
    w = out.append

    keys = re.findall(r"<FILE>\s*([A-Z]+)\.XML\s*</FILE>\s*<KEY>\s*(\w+)\s*</KEY>",
                      rules.split("<TRIBES_TRIAL_VERSION>")[0].split("<TRIBES>")[1])
    w("<!-- GENERATED:roster -->")
    w("| index | nation | rules key | `city::Nation` flag | census functions | lobbies that hold it |")
    w("|---:|---|---|---|---:|---|")
    for i, name in enumerate(ROSTER):
        sites = sum(1 for _, (idx, _) in funcs.items() if i in idx)
        flag = f"`{FLAG[i]}`" if i in FLAG else "none (read off `has_tribe_bonus`)"
        w(f"| {i} | {name} | {keys[i][1] if i < len(keys) else '?'} | {flag} | {sites} | {LOBBY.get(i, 'none')} |")
    # hand arms
    w("<!-- GENERATED:hand -->")
    w("| nation | arm | original | status | witness |")
    w("|---|---|---|---|---|")
    for nat, arm, site, st, wit in HAND:
        w(f"| {nat} | {arm} | `{site}` | {st} | `{wit}` |")

    groups = [(0, 5), (6, 11), (12, 17), (18, 23)]
    for gi, (a, b) in enumerate(groups, 1):
        w(f"<!-- GENERATED:consts{gi} -->")
        w("| constant | rules | the export reads it in | status | witness |")
        w("|---|---:|---|---|---|")
        for n, v in consts:
            if a <= nation_of(n) <= b:
                w(const_row(n, v))

    ordered = sorted(funcs.items(), key=lambda kv: addr_of(kv[0]))
    half = len(ordered) // 2
    for gi, chunk in enumerate((ordered[:half], ordered[half:]), 1):
        w(f"<!-- GENERATED:census{gi} -->")
        w("| function | nations | status | witness |")
        w("|---|---|---|---|")
        for full, (idx, var) in chunk:
            w(census_row(full, idx, var))
    return "\n".join(out)


def splice(doc, generated):
    blocks = {}
    cur = None
    for line in generated.split("\n"):
        m = re.fullmatch(r"<!-- GENERATED:(\w+) -->", line)
        if m:
            cur = m.group(1)
            blocks[cur] = []
        elif cur:
            blocks[cur].append(line)
    for k, rows in blocks.items():
        pat = re.compile(r"(<!-- GENERATED:" + k + r" -->\n)(?:\|[^\n]*\n)*")
        doc, n = pat.subn(lambda m: m.group(1) + "\n".join(rows) + "\n", doc)
        if n != 1:
            sys.exit(f"marker {k}: {n} places in the document")
    return doc


def blocks_of(text):
    """`{marker: [row, ...]}` for every `GENERATED` block in `text`: the
    table rows that follow the marker, as `splice` replaces them."""
    blocks, cur = {}, None
    for line in text.split("\n"):
        m = re.fullmatch(r"<!-- GENERATED:(\w+) -->", line)
        if m:
            cur = m.group(1)
            blocks[cur] = []
        elif cur and line.startswith("|"):
            blocks[cur].append(line)
        else:
            cur = None
    return blocks


def row_key(row):
    """A row's identity: its first cell, and the second where the first
    repeats (a hand arm's nation)."""
    cells = [c.strip() for c in row.strip().strip("|").split("|")]
    return cells[0] if cells[0].startswith("`") or cells[0].isdigit() else " | ".join(cells[:2])


def keyed(rows):
    """`{key: row}`, a key that repeats numbered by its occurrence (`#2`),
    so that a second row under one key is never hidden by the first."""
    out, seen = {}, {}
    for r in rows:
        k = row_key(r)
        seen[k] = seen.get(k, 0) + 1
        out[k if seen[k] == 1 else f"{k} #{seen[k]}"] = r
    return out


def drift(doc, generated):
    """Every row the document's tables and the generator's disagree on, as
    `(block, key, tree row or None, generated row or None)` in the
    generator's block order; a block the document lacks is `(block, None,
    None, None)`. The two header rows never count."""
    tree, gen = blocks_of(doc), blocks_of(generated)
    out = []
    for b, rows in gen.items():
        if b not in tree:
            out.append((b, None, None, None))
            continue
        t, g = keyed(tree[b][2:]), keyed(rows[2:])
        for k in list(t) + [k for k in g if k not in t]:
            if t.get(k) != g.get(k):
                out.append((b, k, t.get(k), g.get(k)))
    return out


def check(export, rules_path, doc_path=DOC, generate=generate):
    """The guard: 0 when the round trip is empty, 1 naming every row it is
    not, 0 with `skipped` when the export or the rules are not here."""
    export, rules_path = Path(export), Path(rules_path)
    if not (export / "funcs").is_dir():
        print(f"nations_census --check: skipped, no decompile export at {export}")
        return 0
    if not rules_path.is_file():
        print(f"nations_census --check: skipped, no install rules at {rules_path}")
        return 0
    rows = drift(Path(doc_path).read_text(), generate(export, rules_path))
    if not rows:
        print("nations_census --check: the round trip is empty")
        return 0
    for b, k, t, g in rows:
        if k is None:
            print(f"[{b}] the document has no GENERATED:{b} block")
            continue
        print(f"[{b}] {k}")
        print(f"  tree: {t if t is not None else '(absent)'}")
        print(f"  gen:  {g if g is not None else '(absent)'}")
    print(f"nations_census --check: {len(rows)} rows differ from the generator — "
          "re-run with --write, or move a hand judgement into HAND/OVERRIDES")
    return 1


def default_rules():
    inst = os.environ.get("RON_INSTALL")
    base = Path(inst) if inst else ROOT.parents[2] / "game" if ROOT.parent.name == "worktrees" else ROOT / "game"
    return base / "Data/rules.xml"


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("export", nargs="?",
                    default=os.environ.get("RON_DECOMP") or str(Path.home() / "ghidra-projects/decomp"))
    ap.add_argument("rules", nargs="?", default=None)
    mode = ap.add_mutually_exclusive_group()
    mode.add_argument("--write", action="store_true", help="splice the tables into docs/NATIONS.md")
    mode.add_argument("--check", action="store_true", help="exit 1 naming every row that differs")
    a = ap.parse_args(argv)
    rules_path = a.rules or default_rules()
    if a.check:
        return check(a.export, rules_path)
    gen = generate(a.export, rules_path)
    if a.write:
        DOC.write_text(splice(DOC.read_text(), gen))
    else:
        print(gen)
    return 0


if __name__ == "__main__":
    sys.exit(main())
