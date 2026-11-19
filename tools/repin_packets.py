#!/usr/bin/env python3
"""repin_packets.py — rewrite the packets' `PINNED` rows a walk moved, and the score.

    RON_PACKETS_PRINT=1 zsh tools/check.sh test --release -p rondata checksum_lines::packets \\
        -- --test-threads 1 --nocapture 2> packets.log
    python3 tools/repin_packets.py packets.log --item 4415 --note "a round's cur_time"

`tools/repin.py` reads `pin_eq!` reports, and the packets' table is not one:
`rondata::diff::checksum_lines::packets::PINNED` is a row a packet, twelve
`(the stream's ranges, ours, ranges parted)` tuples in `STATE_LINES`' order,
held by `hold`. Under `RON_PACKETS_PRINT` the walk asserts nothing and prints
`PIN\\t("<run>", <frame>, [<twelve tuples>]),` for every packet. Item 4411
re-pinned 27 rows from those lines by a scratch script and item 4412 thirty
by hand (their journals' "For the Loop"); this is the shape as a tool, the
pass of 2026-11-19.

For each `PIN` line whose row differs from the table's it replaces the
twelve tuples and puts one comment above the row,

    // Item <N>, <note>: units (a, b, c) -> (d, e, f), guys (…) -> (…).

then recomputes `PACKETS_SCORE` from the whole table as
`the_packets_score_is_the_pins` does (a line counts when theirs or ours is
non-zero, and agrees when none parts) and rewrites its tuple. It prints each
moved line as turned, shrunk or grown, and the score's move. A `PIN` whose
run the table does not hold, or whose frame differs, is listed and left
alone. The census must be printed on one test thread with `--nocapture`
(items 4400, 4402): at four the walks' lines interleave, and without
`--nocapture` it prints nothing.

The queue's `Checksum:` line is the driver's to write. Run `cargo fmt`
after. Tests: `tools/test_repin_packets.py`.
"""
import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TABLE = ROOT / 'crates/rondata/src/diff/checksum_lines/packets.rs'
LINES = ['units', 'builds', 'walls', 'ammo', 'deaths', 'groups', 'guys', 'leaders', 'cities',
         'items', 'goods', 'world']
TUPLE = re.compile(r'\((\d+), (\d+), (\d+)\)')
PIN = re.compile(r'^PIN\t\("([^"]+)", (-?\d+), \[(.*)\]\),\s*$')
ROW = re.compile(r'^(?P<indent>[ \t]*)\(\s*"(?P<run>[^"]+)",\s*(?P<frame>-?\d+),\s*\['
                 r'(?P<body>(?:\s*\(\d+, \d+, \d+\),?){12})\s*\],?\s*\)', re.M)
SCORE = re.compile(r'(const PACKETS_SCORE: \(usize, usize, usize\) = )\((\d+), (\d+), (\d+)\);')


def pins(log):
    """`{run: (frame, twelve tuples)}` from a `RON_PACKETS_PRINT` log; the last line a run wins."""
    out = {}
    for line in log.splitlines():
        m = PIN.match(line)
        if m:
            got = [tuple(map(int, t)) for t in TUPLE.findall(m.group(3))]
            if len(got) == 12:
                out[m.group(1)] = (int(m.group(2)), got)
    return out


def table_rows(text):
    """The table's rows after `const PINNED`, as regex matches."""
    start = text.index('const PINNED')
    return [m for m in ROW.finditer(text, start)]


def score(text):
    """`(packets, agreeing lines, counted lines)` over the table."""
    rows = table_rows(text)
    agree = counted = 0
    for m in rows:
        for t, o, p in TUPLE.findall(m.group('body')):
            if int(t) > 0 or int(o) > 0:
                counted += 1
                agree += p == '0'
    return len(rows), agree, counted


def kind(old, new):
    """What a line's move is called: turned whole, parted anew, shrunk, grown or reshaped."""
    counted = lambda t: t[0] > 0 or t[1] > 0
    if counted(old) and old[2] > 0 and new[2] == 0:
        return 'turned'
    if old[2] == 0 and new[2] > 0:
        return 'parted'
    if new[2] < old[2]:
        return 'shrunk'
    return 'grown' if new[2] > old[2] else 'reshaped'


def repin(text, got, item, note):
    """The table's text with every moved row rewritten, the moves, and the rows left alone."""
    moves, left = [], []
    held = {m.group('run'): m for m in table_rows(text)}
    for run, (frame, _) in got.items():
        if run not in held:
            left.append(f'{run}: no pinned row')
        elif int(held[run].group('frame')) != frame:
            left.append(f'{run}: the table pins frame {held[run].group("frame")}, the log walked {frame}')
    edits = []
    for run, m in held.items():
        if run not in got or int(m.group('frame')) != got[run][0]:
            continue
        old = [tuple(map(int, t)) for t in TUPLE.findall(m.group('body'))]
        new = got[run][1]
        if old == new:
            continue
        parts = []
        for name, a, b in zip(LINES, old, new):
            if a != b:
                parts.append(f'{name} {a} -> {b}')
                moves.append((run, name, a, b, kind(a, b)))
        indent = m.group('indent')
        inner = indent + '        '
        body = ''.join(f'\n{inner}{t},' for t in new)
        row = (f'{indent}// Item {item}, {note}: {", ".join(parts)}.\n'
               f'{indent}(\n{indent}    "{run}",\n{indent}    {m.group("frame")},\n'
               f'{indent}    [{body}\n{indent}    ],\n{indent})')
        edits.append((m.start(), m.end(), row))
    for start, end, row in sorted(edits, reverse=True):
        text = text[:start] + row + text[end:]
    return text, moves, left


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('log', type=Path, help='the stderr of a RON_PACKETS_PRINT walk')
    ap.add_argument('--item', required=True, help='the item whose landing moved the rows')
    ap.add_argument('--note', required=True, help='what moved them, for the row comments')
    ap.add_argument('--file', type=Path, default=TABLE)
    ap.add_argument('--dry-run', action='store_true', help='print the moves, write nothing')
    a = ap.parse_args(argv)
    got = pins(a.log.read_text(errors='replace'))
    if not got:
        print(f'{a.log}: no PIN line — was the walk run with RON_PACKETS_PRINT=1, '
              '--test-threads 1 and --nocapture?', file=sys.stderr)
        return 2
    text = a.file.read_text()
    before = score(text)
    text, moves, left = repin(text, got, a.item, a.note)
    after = score(text)
    m = SCORE.search(text)
    if m is None:
        print(f'{a.file}: no PACKETS_SCORE', file=sys.stderr)
        return 2
    text = text[:m.start()] + f'{m.group(1)}({after[0]}, {after[1]}, {after[2]});' + text[m.end():]
    for run, name, old, new, what in moves:
        print(f'{what}\t{run}\t{name}\t{old} -> {new}')
    for row in left:
        print(f'left alone\t{row}')
    tally = {}
    for *_, what in moves:
        tally[what] = tally.get(what, 0) + 1
    print(f'{len(got)} packets printed, {len({m[0] for m in moves})} rows moved: '
          + (', '.join(f'{n} {k}' for k, n in sorted(tally.items())) or 'no line moved'))
    print(f'PACKETS_SCORE lines {before[1]} -> {after[1]} of {after[2]} '
          f'(pinned {m.group(3)} of {m.group(4)}); the queue\'s `Checksum:` line is the driver\'s')
    if not a.dry_run:
        a.file.write_text(text)
    return 0


if __name__ == '__main__':
    sys.exit(main())
