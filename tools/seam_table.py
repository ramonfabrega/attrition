#!/usr/bin/env python3
"""seam_table.py — the pass's seam table from `held_seams`' files (docs/LOOP.md, the pass's first act 10).

    RON_SEAMS_DIR=<dir> zsh tools/check.sh test --release -p rondata --features seam-count -- held_seams --test-threads 6
    python3 tools/seam_table.py <dir>

`held_seams` writes `seams-<battery>.txt`, every seam hook each held-out
walk enters with its first state marked against the walk's first parting.
This prints the hooks an open row enters and no whole row does, nearest
its row's first parting first, and marks the ones inside DECISIONS 157 §5's
reach: entered before the first parting or within 100 states past it. Four
passes built this table by hand before it was a file (the pass of
2026-11-27).
"""
import re
import sys
from pathlib import Path

ROW = re.compile(r'^row (\d+) run\w+: \d+ states, (?:first parting state (\d+)|whole)')
HOOK = re.compile(r'^\s+([<=>])\s+(\d+)\s+(\d+)\s+(\S+)$')
REACH = 100


def read(directory):
    """`[(battery, row, parting or None, [(first, entries, hook)])]`."""
    walks = []
    for path in sorted(Path(directory).glob('seams-*.txt')):
        battery = path.stem.split('-', 1)[1]
        for line in path.read_text().splitlines():
            m = ROW.match(line)
            if m:
                walks.append((battery, int(m[1]), int(m[2]) if m[2] else None, []))
                continue
            m = HOOK.match(line)
            if m and walks:
                walks[-1][3].append((int(m[2]), int(m[3]), m[4]))
    return walks


def table(walks):
    """The rows of the table, `(past, battery, row, parting, first, entries, hook)`."""
    whole = {h for _, _, parting, hooks in walks if parting is None for _, _, h in hooks}
    rows = [(first - parting, battery, row, parting, first, entries, hook)
            for battery, row, parting, hooks in walks if parting is not None
            for first, entries, hook in hooks if hook not in whole]
    return sorted(rows)


def main(argv):
    if len(argv) != 2:
        print(__doc__, file=sys.stderr)
        return 64
    walks = read(argv[1])
    if not walks:
        print(f'{argv[1]}: no seams-<battery>.txt holds a walk', file=sys.stderr)
        return 2
    rows = table(walks)
    print(f'{len(walks)} walks, {sum(1 for w in walks if w[2] is not None)} open; '
          'hooks an open row enters and no whole row does (first state against first parting, entries)')
    for past, battery, row, parting, first, entries, hook in rows:
        mark = 'IN REACH' if past <= REACH else ''
        print(f'{battery:8} row {row:3} parting {parting:5}  first {first:5} ({past:+6}) {entries:7}  {hook}  {mark}'.rstrip())
    if not rows:
        print('none')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
