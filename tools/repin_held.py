#!/usr/bin/env python3
"""repin_held.py — a held-out row's `moved` score from the score test's own print.

`rondata::diff::held_out`'s `…_held_out_games_score_as_pinned` prints one
line a row it scores,

    HELD OUT row 93 run1799: states 12001 agreed [4288, …] first [("units", (4288, Some(1), 2)), …] over None

and holds the row's whole lines and first parting against `SCORES_…[k]`'s
`moved` column (DECISIONS 154 §5); the rest it prints as `HELD OUT DRIFT`.
Every row landing copied that line into the literal by hand, thirteen in
the tranche of 2026-11-27, and a row another row's rule had moved past its
first parting printed drift into three lanes' scores until one of them
proved by mutation that the lines were its base's. This writes the literal:

    RON_INSTALL=… cargo test --release -p rondata -- held_out_games_score_as_pinned --nocapture 2>&1 | tee run.log
    python3 tools/repin_held.py run.log                 # every row the log moved: report
    python3 tools/repin_held.py run.log --row 93        # one row
    python3 tools/repin_held.py run.log --row 93 --write

It rewrites `moved` alone, `found` never (DECISIONS 144 §2), and leaves the
comment above `moved` where it stands: the landing's sentence there is the
worker's to write. A row is found by its battery (`row % 20`) and its place
(`row // 20`), and the entry's own comment must name the row or nothing is
written. `cargo fmt` afterwards. Tests: `tools/test_repin_held.py`.
"""
import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HELD = ROOT / 'crates/rondata/src/diff/held_out.rs'
# A battery's rows are one residue of 20: the first's 0, 20, …, 140.
BATTERIES = {0: 'SCORES', 10: 'SCORES_SECOND', 5: 'SCORES_THIRD', 15: 'SCORES_FOURTH',
             3: 'SCORES_FIFTH', 13: 'SCORES_SIXTH', 8: 'SCORES_SEVENTH', 18: 'SCORES_EIGHTH'}
LINES = ('units', 'builds', 'walls', 'ammo', 'deaths', 'groups', 'guys', 'leaders', 'cities',
         'items', 'goods', 'world')
PRINT = re.compile(r'HELD OUT row (?P<row>\d+) run(?P<run>\w+): states (?P<states>\d+) '
                   r'agreed \[(?P<agreed>[\d, ]+)\] first \[(?P<first>.*)\] over (?P<over>None|Some\(-?\d+\))\s*$')
FIRST = re.compile(r'\("(\w+)", \((-?\d+), (None|Some\(\d+\)), (\d+)\)\)')
ENTRY = re.compile(r'^    (?:Some\(FirstScore \{|None,)$', re.M)


def parse(log):
    """`{row: (run, states, agreed, first, over)}`, the last print of each row."""
    got = {}
    for line in log.splitlines():
        m = PRINT.search(line)
        if not m:
            continue
        agreed = [int(x) for x in m['agreed'].split(',')]
        if len(agreed) != 12:
            continue
        first = [(n, int(f), ours, int(theirs)) for n, f, ours, theirs in FIRST.findall(m['first'])]
        got[int(m['row'])] = (m['run'], int(m['states']), agreed, first, m['over'])
    return got


def entry_span(text, row):
    """`(start, end, const)` of the row's entry in its battery's table."""
    const = BATTERIES.get(row % 20)
    if const is None:
        raise ValueError(f'row {row}: no battery holds rows of residue {row % 20}')
    head = re.search(rf'^pub\(crate\) const {const}: \[Option<FirstScore>; 8\] = \[$', text, re.M)
    if not head:
        raise ValueError(f'row {row}: no `{const}` table')
    close = text.index('\n];', head.end())
    starts = [m.start() for m in ENTRY.finditer(text, head.end(), close)]
    k = row // 20
    if k >= len(starts):
        raise ValueError(f'row {row}: `{const}` holds {len(starts)} entries, none at {k}')
    # The entry's comment stands between the entry before it and its own head.
    before = starts[k - 1] if k else head.end()
    if not re.search(rf'//.*\brow {row}\b', text[before:starts[k]]):
        raise ValueError(f'row {row}: the comment over `{const}[{k}]` names no row {row}')
    end = starts[k + 1] if k + 1 < len(starts) else close
    return starts[k], end, const


def moved_span(text, start, end):
    """`(start, end)` of the entry's `moved: …,` lines, the trailing newline in."""
    m = re.compile(r'^        moved: None,\n', re.M).search(text, start, end)
    if m:
        return m.start(), m.end()
    m = re.compile(r'^        moved: Some\(Score \{\n', re.M).search(text, start, end)
    if not m:
        raise ValueError('the entry holds no `moved`')
    tail = re.compile(r'^        \}\),\n', re.M).search(text, m.end(), end)
    if not tail:
        raise ValueError('the entry\'s `moved` does not close')
    return m.start(), tail.end()


def literal(states, agreed, first, over):
    """The `moved` lines as `cargo fmt` leaves them, but for a long `agreed`."""
    if len(set(agreed)) == 1:
        arr = f'            agreed: [{agreed[0]}; 12],\n'
    else:
        arr = f'            agreed: [{", ".join(map(str, agreed))}],\n'
    if first:
        rows = ''.join(f'                ("{n}", ({f}, {ours}, {theirs})),\n' for n, f, ours, theirs in first)
        fst = f'            first: &[\n{rows}            ],\n'
    else:
        fst = '            first: &[],\n'
    return (f'        moved: Some(Score {{\n            states: {states},\n{arr}{fst}'
            f'            over: {over},\n        }}),\n')


def read_moved(block):
    """`(states, agreed, first, over)` of a `moved` block, or None for `moved: None`."""
    if 'moved: None' in block:
        return None
    states = int(re.search(r'states: (\d+)', block)[1])
    same = re.search(r'agreed: \[(\d+); 12\]', block)
    if same:
        agreed = [int(same[1])] * 12
    else:
        agreed = [int(x) for x in re.search(r'agreed: \[([\d,\s]+)\]', block)[1].replace('\n', ' ').split(',') if x.strip()]
    first = [(n, int(f), ours, int(theirs)) for n, f, ours, theirs in FIRST.findall(block)]
    over = re.search(r'over: (None|Some\(-?\d+\))', block)[1]
    return states, agreed, first, over


def found_score(text, start, end):
    """The entry's `found` score: what a row with `moved: None` is held to."""
    m = re.compile(r'^        found: Score \{\n', re.M).search(text, start, end)
    tail = m and re.compile(r'^        \},\n', re.M).search(text, m.end(), end)
    if not tail:
        raise ValueError('the entry holds no `found`')
    return read_moved(text[m.start():tail.end()])


def moves(old, new):
    """What a re-pin changes, a line a person reads for each."""
    out = []
    if old[0] != new[0]:
        out.append(f'states {old[0]} -> {new[0]}')
    was, now = {f[0]: f for f in old[2]}, {f[0]: f for f in new[2]}
    for k, name in enumerate(LINES):
        if old[1][k] != new[1][k]:
            out.append(f'{name} agreeing {old[1][k]:,} -> {new[1][k]:,}')
        a, b = was.get(name), now.get(name)
        if (a and a[1]) != (b and b[1]):
            out.append(f'{name} first {a[1] if a else "none"} -> {b[1] if b else "none"}')
        elif a != b:
            out.append(f'{name} sums on {b[1]}')
    if old[3] != new[3]:
        out.append(f'ours\' end {old[3]} -> {new[3]}')
    return out


def repin(text, got, rows):
    """The file's text with each row's `moved` rewritten; `(text, report, refused)`."""
    report, refused = [], []
    for row in rows:
        if row not in got:
            refused.append(f'row {row}: the log prints no score for it')
            continue
        _, states, agreed, first, over = got[row]
        try:
            start, end, const = entry_span(text, row)
            a, b = moved_span(text, start, end)
        except ValueError as e:
            refused.append(str(e))
            continue
        new = (states, agreed, first, over)
        # A row never moved is held to `found`, and takes a `moved` when it parts from it.
        changed = moves(read_moved(text[a:b]) or found_score(text, start, end), new)
        if not changed:
            continue
        text = text[:a] + literal(*new) + text[b:]
        report.append(f'row {row} ({const}[{row // 20}].moved): ' + '; '.join(changed))
    return text, report, refused


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('log')
    ap.add_argument('--row', type=int, action='append', help='a row to re-pin (default: every row the log scores)')
    ap.add_argument('--write', action='store_true')
    ap.add_argument('--file', default=str(HELD), help=argparse.SUPPRESS)
    args = ap.parse_args(argv)
    got = parse(Path(args.log).read_text(errors='replace'))
    if not got:
        print(f'{args.log}: no `HELD OUT row` line; run the score test with --nocapture', file=sys.stderr)
        return 2
    path = Path(args.file)
    text = path.read_text()
    new, report, refused = repin(text, got, args.row or sorted(got))
    for line in report:
        print(line)
    for line in refused:
        print(f'refused: {line}', file=sys.stderr)
    if not report and not refused:
        print('nothing moved')
    if args.write and new != text and not refused:
        path.write_text(new)
        print(f'wrote {path} — `cargo fmt`, and the sentence above each `moved` is yours')
    elif new != text and not args.write:
        print('(nothing written; --write)')
    return 1 if refused else 0


if __name__ == '__main__':
    sys.exit(main())
