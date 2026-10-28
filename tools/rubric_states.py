#!/usr/bin/env python3
r"""rubric_states.py — what each unit type *did* in the lockstep games' dumps:
seen, moved, attacked, died, boarded (Loop 3727, Ramon's word: beside
`rubric.py` table 3's "seen").

    rubric_states.py [--limit N] [--dumps PATH ...] [--json] [--jobs N] [--no-cache]

The dumps are `rubric.lockstep_files`'s, the set table 3 reads (`--dumps`:
the ones given). Every column counts **dumps**: in how many a unit of the
type was in that state at least once. Each dump is read once, mmapped, with
one regular expression, on `--jobs` processes (235 dumps, 23 GB: 25 s on
four, 2026-10-27). A dump is cut into **snapshots** at every `BEGIN FRAME n`
and `BEGIN FULL DUMP`; one that prints a `UNITDATA` is a **unit snapshot**.
A unit is `(who, o)` from its `SUBOBJECT` and `uid` from its `OBJECT` (the
line after `down_who`; a detail that prints no `uid` leaves the unit
unmatched across snapshots); its type is the `type` of a `GUY` with the same
`who`/`o` in the snapshot, else the same uid's type in the snapshot before.

- **seen** — a `BEGIN GUY` whose next line is `type N`: `rubric.scan`'s
  rule, so the types seen equal table 3's on the same dumps.
- **moved** — the unit's `x_internal`/`y_internal` differ between two
  consecutive unit snapshots with the same `uid`. The position itself, not
  an order or `des_x`. Needs two unit snapshots that print `uid`.
- **attacked** — the unit struck a blow, by either of two printed fields:
  (a) a victim `UNITDATA`'s `damage_frame` != 0 with `damage_o`/`damage_who`
  naming a unit (`o` < 2000): `do_damage`'s overkill record, `T.damage_o =
  A.get_captain()` (`docs/COMBAT.md` §7.1 step 2), resolved through the
  snapshot's `GUY`s (else the latest earlier type of that `(who, o)`), once
  per `(victim, damage_frame, damage_o, damage_who)`; (b) an `AMMO`
  record's shooter `who`/`o` (one capture prints `AMMO`). Not counted: a
  blow on a building (`BUILDDATA` prints no `damage_o`, so siege reads 0),
  a blow inside an earlier hit's overkill window, a formation's blow other
  than its captain's. An attacker nothing resolves is `unresolved`.
- **died** — `DeathObjData::log_data@008d55e0` (category `DEATHS`: `valid
  first_frame cur_anim who o gpiece … skel_gpiece node_flags unit_crew`,
  names read from the PE's strings) is the only per-death record, and **no
  lockstep dump prints it** (0 of 235 hold `skel_gpiece`, 2026-10-27; the
  captures run `DEATHS=0` under `[End Frame]`, `docs/ORACLE.md`). It is
  reported *not printed at this detail*; beside it, as inference:
- **vanished** — a unit (uid printed on both sides) in a unit snapshot and
  absent from the next one (which a later boundary or `GameInfo closing`
  closes). Deaths, but not only: a Transport Barge/Galleon/Freighter
  dissolves when its army lands (1,916 of the 2,981 vanished units on
  2026-10-27), and a bribe or delete vanishes too. `vanished_struck` counts
  those whose last record had `damage_frame` != 0 (weak on a 100-frame
  stride, where a unit can be struck and die between records).
- **boarded** — `UNITDATA`'s `inside_up` >= 0 (`UnitData+0x82`, the host:
  `docs/ANIM.md` §4.12.4); `--json` splits it, host `o` < 2000 a unit (a
  transport) and >= 2000 a building (a garrison, a University's Scholars,
  an Airport's planes, a silo's missiles).

A cell is `-` where no dump that saw the type prints the state's field or
gives it a chance to fire, so a 0 is a real zero. A dump whose `GUY`s print
empty (`GUYS` below 1: run1018's 244,854 unit records) types no unit and
counts toward `seen` only. **Cache**: `~/ron-data/rubric-states/cache.json`,
one result per dump keyed by `VERSION`, name, size and mtime, so a rerun
scans only new dumps (0.3 s warm); `--no-cache` scans all and writes none.
Bump `VERSION` when a definition changes. `unit_states()` is the entry
`rubric.py` calls.
"""
import argparse
import json
import mmap
import os
import re
import sys
from collections import Counter

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import census  # noqa: E402
import rubric  # noqa: E402

VERSION = 1
CACHE = os.path.expanduser("~/ron-data/rubric-states/cache.json")
STATES = ("seen", "moved", "attacked", "died", "vanished", "boarded")
N = rb"(-?\d+)"
SKIP = rb"-?\d+\r?\n *"
SCAN = re.compile(
    rb"^ *BEGIN (?:FRAME " + N + rb"\r?$|(FULL DUMP)\r?$"                                      # 1, 2
    rb"|UNITDATA\r?\n *BEGIN OBJECT\r?\n *BEGIN SUBOBJECT\r?\n *flags " + SKIP
    + rb"o " + N + rb"\r?\n *who " + N + rb"\r?\n *x_internal " + N + rb"\r?\n *y_internal " + N   # 3-6
    + rb"(?:\r?\n *z_internal " + SKIP + rb"damage " + SKIP + rb"inside_down " + SKIP + rb"up " + SKIP
    + rb"down " + SKIP + rb"down_who " + SKIP + rb"uid " + N + rb")?"                          # 7
    rb"|GUY\r?\n *type " + N                                                                    # 8
    + rb"(?:\r?\n *x " + SKIP + rb"y " + SKIP + rb"z " + SKIP + rb"angle " + SKIP
    + rb"who " + N + rb"\r?\n *o " + N + rb")?"                                                  # 9, 10
    rb"|AMMO(?:\r?\n *(?:cur_time|total_time) -?\d+)*\r?\n *who " + N + rb"\r?\n *o " + N + rb")"  # 11, 12
    rb"|^ *damage_(frame|o|who) " + N                                                           # 13, 14
    + rb"|^ *inside_up " + N, re.M)                                                             # 15
DMG = {b"frame": 0, b"o": 1, b"who": 2}


class Dump:
    """One dump's scan state; `result()` is its JSON-able summary."""

    def __init__(self):
        self.seen = Counter()
        self.sets = {k: set() for k in ("moved", "moved_can", "hit", "ammo", "board_unit", "board_build",
                                        "vanished", "vanished_can", "printed_damage", "printed_inside")}
        self.last_type = {}
        self.prev = None  # the last unit snapshot: {(who, o): (uid, type, x, y, struck)}
        self.done_hits = set()
        self.c = Counter()
        self.vanished = Counter()  # type -> [units, struck before]
        self.new_snap()

    def new_snap(self):
        self.units, self.types, self.ammo, self.cur = {}, {}, [], None

    def boundary(self, closed):
        """Close the snapshot; `closed` — a later boundary or the file's
        `GameInfo closing` follows it, so its unit list is whole."""
        self.flush_unit()
        units, types = self.units, self.types
        self.c["snapshots"] += 1
        for k, t in types.items():
            self.last_type[k] = t
        ty = lambda wo: types.get(wo, self.last_type.get(wo))  # noqa: E731
        for who, o in self.ammo:
            self.c["ammo_records"] += 1
            if 0 <= o < 2000:
                t = ty((who, o))
                (self.sets["ammo"].add(t) if t is not None else self.c.update(["unresolved"]))
        if units:
            self.c["unit_snapshots"] += 1
            prev, cur = self.prev or {}, {}
            for wo, (uid, x, y, dmg, inside) in units.items():
                p = prev.get(wo)
                same = p is not None and uid is not None and p[0] == uid
                t = types.get(wo, p[1] if same else None)  # an empty GUY: the same uid's last type
                if t is None:  # kept (no false vanish), counted in no state
                    self.c["untyped_units"] += 1
                self.c["no_uid"] += uid is None
                cur[wo] = (uid, t, x, y, dmg[0] not in (None, 0))
                if same:
                    self.sets["moved_can"].add(t)
                    if (p[2], p[3]) != (x, y):
                        self.sets["moved"].add(t)
                if dmg[1] is not None:
                    self.sets["printed_damage"].add(t)
                    df, do, dw = dmg
                    if df and 0 <= do < 2000 and dw is not None and dw >= 0 \
                            and (wo, uid, df, do, dw) not in self.done_hits:
                        self.done_hits.add((wo, uid, df, do, dw))
                        a = ty((dw, do))
                        (self.sets["hit"].add(a) if a is not None else self.c.update(["unresolved"]))
                if inside is not None:
                    self.sets["printed_inside"].add(t)
                    if inside >= 0:
                        self.sets["board_unit" if inside < 2000 else "board_build"].add(t)
            if self.prev is not None and closed:
                for wo, (uid, t, _x, _y, struck) in prev.items():
                    c = cur.get(wo)
                    if uid is None or (c is not None and c[0] is None):
                        continue  # identity unverifiable: a detail that prints no uid
                    self.sets["vanished_can"].add(t)
                    if c is None or c[0] != uid:
                        self.sets["vanished"].add(t)
                        self.c["vanished_units"] += 1
                        self.c["vanished_struck"] += struck
                        self.vanished[t] += 1
                        self.vanished[(t, "struck")] += struck
            self.prev = cur
        self.new_snap()

    def flush_unit(self):
        if self.cur is not None:
            wo, uid, x, y, dmg, inside = self.cur
            self.units[wo] = (uid, x, y, dmg, inside)
            self.cur = None

    def feed(self, buf):
        for m in SCAN.finditer(buf):
            i = m.lastindex
            if i <= 2:
                self.boundary(True)
            elif i <= 7:
                self.flush_unit()
                o, who, x, y = (int(v) for v in m.group(3, 4, 5, 6))
                uid = int(m.group(7)) if m.group(7) is not None else None
                self.cur = ((who, o), uid, x, y, [None, None, None], None)
                self.c["unit_records"] += 1
            elif i <= 10:
                t = int(m.group(8))
                self.seen[t] += 1
                if m.group(9) is not None:
                    self.types.setdefault((int(m.group(9)), int(m.group(10))), t)
            elif i == 12:
                self.ammo.append((int(m.group(11)), int(m.group(12))))
            elif self.cur is not None:
                if i == 14:
                    self.cur[4][DMG[m.group(13)]] = int(m.group(14))
                else:
                    self.cur = self.cur[:5] + (int(m.group(15)),)
        self.boundary(b"GameInfo closing" in buf[max(0, len(buf) - 400):])

    def result(self):
        out = {k: sorted(t for t in v if t is not None) for k, v in self.sets.items()}
        out["seen"] = {str(t): n for t, n in sorted(self.seen.items())}
        out["vanished_by_type"] = {str(t): [n, self.vanished[(t, "struck")]] for t, n in self.vanished.items()
                                 if not isinstance(t, tuple) and t is not None}
        out.update(self.c)
        return out


def scan_dump(path):
    """`(path, result)` — one dump, one pass."""
    d = Dump()
    if os.path.getsize(path):
        with open(path, "rb") as f, mmap.mmap(f.fileno(), 0, access=mmap.ACCESS_READ) as mm:
            d.feed(mm)
    return path, d.result()


def cache_key(path):
    st = os.stat(path)
    return f"v{VERSION}|{os.path.basename(path)}|{st.st_size}|{st.st_mtime_ns}"


def scan_all(paths, jobs=4, cache=CACHE):
    """`{path: result}` for every dump, from the cache where it holds the
    dump's key; the rest scanned on `jobs` processes and written back."""
    store = {}
    if cache and os.path.isfile(cache):
        try:
            with open(cache) as f:
                store = json.load(f)
        except (OSError, ValueError):
            store = {}
    out, todo = {}, []
    for p in paths:
        k = cache_key(p)
        (out.__setitem__(p, store[k]) if k in store else todo.append(p))
    print(f"rubric_states: {len(paths) - len(todo)} cached, {len(todo)} to scan", file=sys.stderr, flush=True)
    if todo:
        todo.sort(key=os.path.getsize, reverse=True)
        if jobs > 1 and len(todo) > 1:
            import multiprocessing as mp
            with mp.get_context("spawn").Pool(jobs) as pool:
                it = pool.imap_unordered(scan_dump, todo)
                for n, (p, r) in enumerate(it):
                    print(f"rubric_states: [{n + 1}/{len(todo)}] {os.path.basename(p)}", file=sys.stderr, flush=True)
                    out[p] = r
        else:
            for p in todo:
                out[p] = scan_dump(p)[1]
        if cache:
            for p in todo:
                store[cache_key(p)] = out[p]
            os.makedirs(os.path.dirname(cache), exist_ok=True)
            tmp = cache + ".tmp"
            with open(tmp, "w") as f:
                json.dump(store, f)
            os.replace(tmp, cache)
    return out


def tally(results):
    """Per type id, in how many dumps each state fired and could fire."""
    per = {}
    tot = Counter()
    for r in results.values():
        seen = {int(t) for t in r["seen"]}
        fired = {"seen": seen, "moved": set(r["moved"]), "attacked": set(r["hit"]) | set(r["ammo"]),
                 "attacked_hit": set(r["hit"]), "attacked_ammo": set(r["ammo"]),
                 "vanished": set(r["vanished"]), "boarded": set(r["board_unit"]) | set(r["board_build"]),
                 "boarded_unit": set(r["board_unit"]), "boarded_build": set(r["board_build"])}
        can = {"moved": set(r["moved_can"]), "vanished": set(r["vanished_can"]),
               "attacked": seen if (r["printed_damage"] or r.get("ammo_records")) else set(),
               "boarded": set(r["printed_inside"])}
        for k, ts in fired.items():
            for t in ts:
                per.setdefault(t, Counter())[k] += 1
        for t, (n, struck) in r.get("vanished_by_type", {}).items():
            per.setdefault(int(t), Counter()).update(vanished_units=n, vanished_struck=struck)
        for k, ts in can.items():
            for t in ts:
                per.setdefault(t, Counter())[k + "_can"] += 1
        for k in ("unit_records", "unit_snapshots", "snapshots", "unresolved", "vanished_units",
                  "vanished_struck", "ammo_records", "untyped_units", "no_uid"):
            tot[k] += r.get(k, 0)
        tot["dumps_with_units"] += bool(r.get("unit_snapshots"))
    return per, tot


def rules_and_denominator(pdb, archive):
    """`(rules, excluded)` exactly as table 3 builds them."""
    xml = os.path.join(rubric.install_dir(pdb), "data", "unitrules.xml")
    with open(xml, errors="replace") as f:
        text = f.read()
    ages = {}
    tdump = os.path.join(archive, "gamelog-run3-fulldump-types.txt")
    if os.path.isfile(tdump):
        with open(tdump, "rb") as f, mmap.mmap(f.fileno(), 0, access=mmap.ACCESS_READ) as mm:
            ages = rubric.loaded_ages(mm)
    return rubric.unit_rules(text), set(rubric.no_tribe_types(text, ages))


def unit_states(paths, rules, excluded, limit=None, jobs=4, cache=CACHE):
    """The wiring for `rubric.py`: `(rows, totals)`. `rows` one dict per
    skirmish type seen (`id name domain seen moved attacked died vanished
    boarded` dump counts, `died` None — not printed — and a state `None`
    where it could not fire; `attacked_hit/_ammo`, `boarded_unit/_build`),
    `totals` the types in each state, the denominator and the scan's
    counters."""
    paths = list(paths)[:limit] if limit else list(paths)
    per, tot = tally(scan_all(paths, jobs, cache))
    rules_in = [r for r in rules if r[0] not in excluded]
    rows = []
    for tid, name, _graph, domain, _cat in rules_in:
        c = per.get(tid)
        if not c or not c["seen"]:
            continue
        row = {"id": tid, "name": name, "domain": domain, "seen": c["seen"], "died": None}
        for s in ("moved", "attacked", "vanished", "boarded"):
            row[s] = c[s] if c[s + "_can"] or c[s] else None
        for s in ("attacked_hit", "attacked_ammo", "boarded_unit", "boarded_build", "vanished_units",
                  "vanished_struck"):
            row[s] = c[s]
        rows.append(row)
    totals = {s: sum(1 for r in rows if r[s]) for s in ("seen", "moved", "attacked", "vanished", "boarded")}
    totals.update(died=None, types=len(rules_in), excluded=len(excluded), dumps=len(paths),
                  outside=len([t for t in per if t not in {r[0] for r in rules}]), **tot)
    return rows, totals


def cell(v):
    return "-" if v is None else str(v)


def summary(t, partial=""):
    return (f"unit type states: seen {t['seen']} · moved {t['moved']} · attacked {t['attacked']} · died n/p "
            f"(not printed: no DeathObjData; vanished {t['vanished']}) · boarded {t['boarded']} "
            f"(of {t['types']} skirmish types; {t['dumps']} dumps, {t['dumps_with_units']} with unit records)"
            + partial)


def print_states(rows, t, partial=""):
    print("# 3b. unit type states — dumps in which a unit of the type was in the state "
          "(`-`: not printed / could not fire there; died: no lockstep dump prints DeathObjData)")
    print(f"{'id':>4} {'name':24} {'domain':6} {'seen':>5} {'moved':>5} {'attkd':>5} {'died':>4} "
          f"{'vansh':>5} {'board':>5}")
    for r in rows:
        print(f"{r['id']:4} {r['name'][:24]:24} {r['domain'][:6]:6} {r['seen']:5} {cell(r['moved']):>5} "
              f"{cell(r['attacked']):>5} {'n/p':>4} {cell(r['vanished']):>5} {cell(r['boarded']):>5}")
    print(f"scan: {t['unit_records']} unit records in {t['unit_snapshots']} unit snapshots of {t['snapshots']} · "
          f"attackers unresolved {t['unresolved']} · ammo records {t['ammo_records']} · untyped units "
          f"{t['untyped_units']} · without uid {t['no_uid']} · vanished units {t['vanished_units']}, struck before {t['vanished_struck']}")
    print(summary(t, partial))


def main():
    ap = argparse.ArgumentParser(description="Per unit type: seen, moved, attacked, died, boarded, in the "
                                             "lockstep games' dumps. The module docstring defines each state.")
    ap.add_argument("--limit", type=int, help="read only the first N dumps")
    ap.add_argument("--dumps", nargs="*", help="these dumps instead of the lockstep games'")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--jobs", type=int, default=4)
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--cache", default=CACHE)
    ap.add_argument("--pdb", default=census.default_pdb(HERE))
    ap.add_argument("--archive", default=census.ARCHIVE)
    a = ap.parse_args()
    if a.dumps:
        paths = list(a.dumps)
    else:
        _runs, games, _u = rubric.lockstep_runs()
        paths = rubric.lockstep_files(games, a.archive)[0]
    rules, excluded = rules_and_denominator(a.pdb, a.archive)
    rows, t = unit_states(paths, rules, excluded, a.limit, a.jobs, None if a.no_cache else a.cache)
    partial = f" (partial: {t['dumps']} of {len(paths)} dumps, --limit)" if a.limit and a.limit < len(paths) else ""
    if a.json:
        json.dump({"rows": rows, "totals": t}, sys.stdout, indent=1)
        print()
    else:
        print_states(rows, t, partial)
    return 0


if __name__ == "__main__":
    sys.exit(main())
