#!/usr/bin/env python3
"""ledger.py — one line per fuzzed seed: what it scored, and what it lit up.

    ledger.py append SEED --diff FILE [--trace FILE] [--dump FILE] [--note S]
    ledger.py show [--sort score|seed|when]

The score is `rondata`'s own `ticks before divergence`, so nothing new is
measured here -- this only keeps it. That matters: a number in a commit
message is a claim, and a ledger is a series, which is the difference between
"the port diverges at 40" and "the port used to diverge at 12".

**`survived`, not `ticks`, is the column to read.** `ticks before
divergence` is an absolute sim-frame, so it is mostly a statement about where
the window was put: a run windowed at 15100 scores 15099 by diverging on its
very first compared frame, and a run windowed at 300 cannot score above 299
however perfect it is. `survived = ticks + 1 - lo` is how many frames the
simulation actually held, which is the number that is comparable between
seeds and the one worth moving.

The second column that earns its place is coverage. `tools/trace/report.py
… blind docs/` is the queue of runs, and a seed is worth keeping in the batch
exactly when it enters functions no earlier run did. A seed that scores well
and lights nothing new is a seed the batch can stop paying for.

The ledger is a TSV under `tools/fuzz/` rather than in `docs/`: it is data a
tool writes and reads, it changes on every batch, and prose that changes on
every batch is a changelog.
"""
import argparse
import os
import re
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
DEFAULT = os.path.join(HERE, "ledger.tsv")
COLUMNS = ["when", "seed", "lo", "survived", "ticks", "frames", "compared",
           "unlinked", "funcs", "note"]

TICKS = re.compile(r"ticks before divergence:\s*(\d+)")
STEPPED = re.compile(
    r"(\d+) frames stepped, (\d+) unit-frames compared, "
    r"(\d+) unit-frames the sim has no unit for")


def score(path):
    """(ticks, frames, compared, unlinked) out of a `rondata --diff` capture."""
    text = open(path, encoding="utf-8", errors="replace").read()
    m = TICKS.search(text)
    if not m:
        sys.exit(f"{path}: no `ticks before divergence` line -- did the diff run?")
    ticks = int(m.group(1))
    s = STEPPED.search(text)
    frames, compared, unlinked = (s.groups() if s else ("", "", ""))
    return ticks, frames, compared, unlinked


def functions(trace):
    """How many distinct functions the trace entered, or '' with no trace."""
    if not trace:
        return ""
    report = os.path.join(HERE, "..", "trace", "report.py")
    import subprocess
    try:
        out = subprocess.run([sys.executable, report, trace, "functions"],
                             capture_output=True, text=True, timeout=300)
    except (OSError, subprocess.SubprocessError) as e:
        print(f"warning: trace unread ({e})", file=sys.stderr)
        return ""
    if out.returncode != 0:
        print(f"warning: trace unread (report.py exit {out.returncode})", file=sys.stderr)
        return ""
    return str(len([l for l in out.stdout.split("\n") if l.strip()]))


def rows(path):
    if not os.path.exists(path):
        return []
    with open(path) as fh:
        lines = [l.rstrip("\n") for l in fh if l.strip()]
    if not lines:
        return []
    return [dict(zip(COLUMNS, l.split("\t"))) for l in lines[1:]]


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    a = sub.add_parser("append")
    a.add_argument("seed")
    a.add_argument("--diff", required=True, help="captured `rondata --diff` output")
    a.add_argument("--trace", help="the run's rontrace.log")
    a.add_argument("--lo", type=int, required=True,
                   help="the window's first frame -- what `survived` is measured from")
    a.add_argument("--note", default="")
    a.add_argument("--path", default=DEFAULT)
    s = sub.add_parser("show")
    s.add_argument("--sort", default="score", choices=("score", "seed", "when"))
    s.add_argument("--path", default=DEFAULT)
    args = ap.parse_args()

    if args.cmd == "append":
        ticks, frames, compared, unlinked = score(args.diff)
        row = [time.strftime("%Y-%m-%dT%H:%M:%S"), str(args.seed), str(args.lo),
               str(ticks + 1 - args.lo), str(ticks),
               frames, compared, unlinked, functions(args.trace), args.note]
        fresh = not os.path.exists(args.path)
        with open(args.path, "a") as fh:
            if fresh:
                fh.write("\t".join(COLUMNS) + "\n")
            fh.write("\t".join(row) + "\n")
        print("\t".join(row))
        return

    got = rows(args.path)
    if not got:
        print(f"{args.path}: empty")
        return
    def seed_key(r):
        try:
            return (0, int(r["seed"], 0))
        except ValueError:
            return (1, 0)  # a named baseline row sorts after the numbered seeds
    key = {"score": lambda r: int(r["survived"] or 0),
           "seed": seed_key,
           "when": lambda r: r["when"]}[args.sort]
    print("  ".join(f"{c:>10}" for c in COLUMNS))
    for r in sorted(got, key=key):
        print("  ".join(f"{r.get(c, ''):>10}" for c in COLUMNS))
    lived = [int(r["survived"]) for r in got if r.get("survived")]
    if lived:
        print(f"\n{len(got)} seed(s); survived — worst {min(lived)}, median "
              f"{sorted(lived)[len(lived) // 2]}, best {max(lived)}")


if __name__ == "__main__":
    main()
