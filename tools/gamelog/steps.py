#!/usr/bin/env python3
"""steps.py — the production AI's step machine, frame by frame, out of a
`LEADERS=9` window (`docs/AI.md` §2.4–§2.6; run18).

    steps.py [gamelog] [--who N] [--from F] [--to F] [--names types.tsv]
             [--all]

One block per `BEGIN FRAME n` (the end state of sim-frame n−1): the step
fields (`production_step`, `script_step`, `prod_script_run`), the goods
picture `production_ai_setup` writes (`econ[]`, `rate[]`, `shortages`,
`worst_good`, `best_good`, the six `bucket`s), `effective_pop`,
`site_mark`, the queued types, then the eleven `MAKEOBJECT`s that are not
empty (`t = −1`) and the ten `SITE`s with a value. By default a frame is
printed only when something in that set changed since the previous frame;
`--all` prints every frame. `--names` takes a `index<TAB>name` file to
label `t` (the `TypeIndex` enum; `rondata` can write one).
"""
import os
import re
import sys

DEFAULT = os.path.expanduser(
    "~/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/"
    "AppData/Roaming/Microsoft Games/Rise of Nations/Logs/gamelog.txt"
)
GOODS = ["food", "wood", "wealth", "metal", "know", "oil"]
SCALARS = [
    "production_step", "script_step", "prod_script_run", "effective_pop",
    "site_mark", "shortages", "worst_good", "best_good", "ages_get()",
    "city_num", "village_num", "peasants", "free_peasants", "gatherers",
    "combat", "pop", "pop_cap", "active", "control", "scholars", "caras",
    "merchants", "attacked", "full_cities",
]


def leaders(path, who, lo, hi):
    """Yield (frame, record) for every LEADERDATA of `who` in frames [lo, hi]."""
    frame = None
    block = None
    block_indent = 0
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            s = line.rstrip("\n")
            st = s.strip()
            if st.startswith("BEGIN FRAME"):
                frame = int(st.split()[2])
                block = None
                if frame > hi:
                    return
                continue
            if frame is None or frame < lo:
                continue
            indent = len(s) - len(s.lstrip())
            if block is None:
                if st == "BEGIN LEADERDATA":
                    block, block_indent = [], indent
                continue
            if indent <= block_indent and st:
                rec = parse(block)
                block = None
                if rec.get("who") == str(who):
                    yield frame, rec
                if st == "BEGIN LEADERDATA":
                    block, block_indent = [], indent
                continue
            block.append((indent, st))


def parse(block):
    """A LEADERDATA block → {name: value | [values] | {idx: value}}, nested
    blocks prefixed (`MAKEOBJECT.t`), repeated keys collected in order."""
    stack = []
    rec = {}
    for indent, st in block:
        while stack and indent <= stack[-1][0]:
            stack.pop()
        if st.startswith("BEGIN "):
            stack.append((indent, st[6:]))
            continue
        if st.startswith("(int) "):  # `(int) site_mark 16`
            st = st[6:]
        m = re.match(r"^(\S+?)(\[[^\]]*\])?\s+(.*)$", st)
        if not m:
            continue
        name, sub, val = m.group(1), m.group(2), m.group(3)
        key = ".".join(n for _, n in stack + [(0, name)])
        if sub is not None:
            rec.setdefault(key, []).append(val)
        elif key in rec:
            if not isinstance(rec[key], list) or not rec[key] or not isinstance(rec[key][0], str):
                rec[key] = [rec[key]]
            rec[key].append(val)
        else:
            rec[key] = val
    return rec


def first(rec, key):
    v = rec.get(key)
    return v[0] if isinstance(v, list) else v


def picture(rec, names):
    """The fields the step machine writes, as a comparable dict."""
    p = {k: first(rec, k) for k in SCALARS}
    p["econ"] = rec.get("econ", [])
    p["rate"] = rec.get("rate", [])[:6]
    p["bucket"] = rec.get("bucket", [])[:6]
    p["income"] = rec.get("income", [])[:6]
    queued = rec.get("num_queued", [])
    p["queued"] = {i: v for i, v in enumerate(queued) if v not in ("0", "-1")}
    p["queued"] = {label(i, names): v for i, v in p["queued"].items()}
    mk = list(zip(*[rec.get("MAKEOBJECT." + k, []) for k in
                    ("t", "val", "escrow", "city", "up", "o", "num", "cat", "wx", "wy")]))
    p["list"] = [(i, m) for i, m in enumerate(mk) if m[0] != "-1"]
    p["cap"] = rec.get("resource_cap", [])[:6]
    st = list(zip(*[rec.get("SITE." + k, []) for k in ("wx", "wy", "val", "reg", "dist", "rank")]))
    p["sites"] = [s for s in st if s[2] != "0"]
    return p


def label(t, names):
    t = int(t)
    return f"{t}={names[t]}" if t in names else str(t)


def show(frame, p, names):
    print(f"FRAME {frame}  step {p['production_step']} script {p['script_step']} run {p['prod_script_run']}"
          f"  age {p['ages_get()']}  eff_pop {p['effective_pop']} pop {p['pop']}/{p['pop_cap']}"
          f"  cities {p['city_num']}+{p['village_num']} peasants {p['peasants']} free {p['free_peasants']}"
          f" gath {p['gatherers']} combat {p['combat']} site_mark {p['site_mark']}")
    print(f"  econ {p['econ']} rate {p['rate']} short {p['shortages']} worst {p['worst_good']} best {p['best_good']}")
    print(f"  bucket {p['bucket']} income {p['income']} cap {p['cap']}")
    if p["queued"]:
        print(f"  queued {p['queued']}")
    for i, m in p["list"]:
        t, val, escrow, city, up, o, num, cat, wx, wy = m
        print(f"  list[{i}] t {label(t, names)} val {val} city {city} num {num} up {up} o {o}"
              f" cat {cat} escrow {escrow} at ({wx},{wy})")
    for s in p["sites"]:
        wx, wy, val, reg, dist, rank = s
        print(f"  site ({wx},{wy}) val {val} reg {reg} dist {dist} rank {rank}")


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    opts = sys.argv[1:]
    path = args[0] if args else DEFAULT
    who = 1
    lo, hi = 0, 10 ** 9
    names = {}
    for o in opts:
        if o.startswith("--who="):
            who = int(o[6:])
        elif o.startswith("--from="):
            lo = int(o[7:])
        elif o.startswith("--to="):
            hi = int(o[5:])
        elif o.startswith("--names="):
            for line in open(o[8:]):
                a, b = line.rstrip("\n").split("\t")[:2]
                names[int(a)] = b
    every = "--all" in opts
    # `--terse`: compare only the step machine's own fields, not the ledger,
    # whose buckets tick every frame and would print every block.
    terse = "--terse" in opts
    keys = ("production_step", "script_step", "prod_script_run", "list", "queued",
            "econ", "sites") if terse else None
    prev = None
    for frame, rec in leaders(path, who, lo, hi):
        p = picture(rec, names)
        cmp = {k: p[k] for k in keys} if keys else p
        if every or cmp != prev:
            show(frame, p, names)
        prev = cmp


if __name__ == "__main__":
    main()
