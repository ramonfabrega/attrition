#!/usr/bin/env python3
"""scenario.py — a seed becomes a scenario the original can be told to play.

    scenario.py SEED [--install DIR] [--lo N] [--hi N] [--extent LO HI]
    scenario.py SEED --no-stage      map variation only -- the control run
    scenario.py SEED --json          the same scenario as metadata, for the ledger

Writes a `rontrace.cmd` body on stdout: `<sim-frame> <text>` lines, `!` for a
console-only command and a bare word for a `cheat ` one, which is the split
`tools/gamelog/console.py` reports and `docs/ORACLE.md` explains.

**What a scenario may contain, and why it is so narrow.** The channel reaches
45 of the original's 102 console commands, and not one of them issues an
order — `move` is a teleport (`Unit::set_new_location`), and every
`add_*_order` / `do_*` / `action_*` / `process_*` path needs the real UI.  So
a Tier 1 scenario is *staging*: spawn, kill, damage, tech, resource,
diplomacy.  What it buys is not order coverage but **map and matchup
variety** — `!restart <seed>` regenerates the world, and no capture before
this has ever varied the map.

Determinism is the point, so the generator carries its own LCG rather than
`random`: the same seed must produce the same scenario on any machine and any
Python, because the seed is the ledger's primary key.
"""
import argparse
import json
import os
import re
import sys

# Numerical Recipes' LCG. Small, exactly specified, and reproducible anywhere
# -- which `random` is not promised to be across versions.
MASK = (1 << 32) - 1


class Rng:
    def __init__(self, seed):
        self.s = seed & MASK

    def next(self):
        self.s = (self.s * 1664525 + 1013904223) & MASK
        return self.s

    def below(self, n):
        # The high bits, not the low ones: an LCG modulo a power of two has a
        # period-2 bit 0, so `next() % 2` is worthless and every `who` in the
        # scenario comes out the same player. Caught by reading the first
        # generated scenario, which is the only reason it is not still there.
        return ((self.next() >> 16) * n) >> 16

    def between(self, lo, hi):
        """Inclusive, matching the original's own `Random::get(a, b)`."""
        return lo + self.below(hi - lo + 1)

    def pick(self, seq):
        return seq[self.below(len(seq))]


def unit_types(install):
    """Single-word `unitrules.xml` names, which are what `add` can be given.

    `ConsoleWin::parse_type` takes one token, so a name with a space in it
    ("Supply Wagon") cannot be reached from a `.cmd` line at all -- the
    scenario would silently spawn nothing. Filtering here rather than
    discovering it in a dump is the whole reason this reads the install.
    """
    path = os.path.join(install, "Data", "unitrules.xml")
    with open(path, encoding="utf-8", errors="replace") as fh:
        names = re.findall(r"<NAME>([^<]*)</NAME>", fh.read())
    seen, out = set(), []
    for n in names:
        n = n.strip()
        if n and " " not in n and n.lower() not in seen:
            seen.add(n.lower())
            out.append(n.lower())
    return out


# The chat-reachable commands this generator is willing to emit, with the
# vocabulary each takes. Kept deliberately small: every line here has been
# read in `run_cmd`, and a command whose argument grammar has not been read
# does not belong in an unattended overnight batch.
GOODS = ["food", "timber", "metal", "wealth", "knowledge", "oil"]


def scenario(seed, install, lo, hi, extent, stage=True):
    rng = Rng(seed)
    types = unit_types(install)
    lines = []

    def at(frame, text):
        lines.append((frame, text))

    # Fast-forward costs nothing with the dump gated off (~500 sim-frames a
    # second), so the window can sit anywhere without paying for the frames
    # before it. `docs/ORACLE.md`, run18.
    at(5, "!ffwd 30")

    # `--no-stage`: the control run, and the only shape that measures
    # *fidelity* rather than window placement. An early window leaves no room
    # before it for staging, and every cheat the scenario would issue is a
    # state change the harness cannot model -- `ai off` most of all, since the
    # sim has its own `docs/AI.md` leader and would keep playing while the
    # original's stopped. So the control issues nothing at all: the seed still
    # varies the whole generated map, which is the point.
    # `docs/ORACLE.md`, "The 300-frame window".
    if not stage:
        at(hi + 1, "!quit")
        return lines

    # The AI's units are noise the diff would have to be told to skip, and
    # `!ai off` stops the leader's strategy (though not, per run17, the
    # buildings' queues).
    at(100, "ai off")

    players = 2
    # Staging happens before the window opens, so the window sees a settled
    # world rather than the frame a spawn landed on.
    frame = 200
    for _ in range(rng.between(4, 10)):
        who = rng.below(players)
        ty = rng.pick(types)
        n = rng.between(1, 3)
        x = rng.between(extent[0], extent[1])
        y = rng.between(extent[0], extent[1])
        at(frame, f"add {n} {ty} who={who} {x},{y}")
        frame += rng.between(20, 80)

    # A few state pokes, each of which changes what the window will show.
    for _ in range(rng.between(2, 5)):
        kind = rng.below(4)
        who = rng.below(players)
        if kind == 0:
            at(frame, f"resource who={who} {rng.pick(GOODS)} +{rng.between(100, 5000)}")
        elif kind == 1:
            at(frame, "war" if rng.below(2) else "peace")
        elif kind == 2:
            at(frame, f"military {rng.between(1, 4)} {who}")
        else:
            at(frame, f"commerce {rng.between(1, 4)} {who}")
        frame += rng.between(20, 60)

    assert frame < lo, f"staging ran past the window: {frame} >= {lo}"
    at(hi + 1, "!quit")
    return lines


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("seed", type=lambda s: int(s, 0))
    ap.add_argument("--install", default=os.environ.get("RON_INSTALL"))
    ap.add_argument("--lo", type=int, default=3000)
    ap.add_argument("--hi", type=int, default=3100)
    ap.add_argument("--extent", type=int, nargs=2, default=(100, 200))
    ap.add_argument("--no-stage", action="store_true",
                    help="map variation only: no spawns, no state pokes")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    if not args.install:
        sys.exit("no install: pass --install or set RON_INSTALL")

    lines = scenario(args.seed, args.install, args.lo, args.hi,
                     tuple(args.extent), stage=not args.no_stage)
    if args.json:
        json.dump({"seed": args.seed, "lo": args.lo, "hi": args.hi,
                   "extent": list(args.extent),
                   "lines": [{"frame": f, "text": t} for f, t in lines]},
                  sys.stdout, indent=2)
        print()
        return
    print(f"# scenario.py seed {args.seed}, window [{args.lo}, {args.hi})")
    for f, t in lines:
        print(f"{f} {t}")


if __name__ == "__main__":
    main()
