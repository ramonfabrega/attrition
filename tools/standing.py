#!/usr/bin/env python3
"""standing.py — a widening's standing keys, beside who in `sim` reads them.

    RON_FIRSTS=1 cargo test --release -p rondata <the widening> -- --nocapture \\
        2>&1 | python3 tools/standing.py
    … | python3 tools/standing.py --block 8815      # another block's keys
    … | python3 tools/standing.py --all             # every block, by frame

A widening prints every key's first parting under `RON_FIRSTS`, as
`first <frame> <who>/<o> <what>: <values>`. The keys that part on the
window's first block were apart before the window opened: they *stand*, a
landing pins their count, and the count is read as noise. Three words of one
tranche were among them (the twentieth pass, parked 1162 and 1187): who=1's
`pop_cap` held Great Sahara at 12783, its `pop` and `escrow` stood on two
widenings' first blocks before they were East Indies' 7382 and 7512, its
wealth and `trade_val` before they were Great Sahara's 15982.

What told them from noise, each time, was that **this crate reads the
field**. So this prints each standing field with the lines of `crates/sim`
that read it, most-read first, and the fields nothing reads last and
folded. A field `sim` reads and the original holds a different value of is
a decision the two sides are already making apart.

The name is matched as the dump writes it, as a field access or a method
(`.pop_cap`, `.pop_cap(`), outside the crate's tests. A field this crate
carries under another name is not found: it is a list to read, not a proof.
"""
import argparse
import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SIM = ROOT / 'crates/sim/src'
# `first <frame> …` is the AI track's widening; `ch43 f<frame> …` is a
# golden chapter's, the same row in the chapter's spelling (parked 1250);
# `standing <block> …` and `f<block> …` are `widen_on_siblings`' — the
# third pair's and the coverage pair's — which items 1461 and 1466 fed
# through `sed` (parked 1471, the twenty-fifth pass).
FIRST = re.compile(r'^\s*(?:first (-?\d+)|ch[0-9a-z]+ f(-?\d+)|standing (-?\d+)|f(-?\d+))'
                   r' (-?\d+)/(-?\d+) (\S+?): (.*)$')
# Segments of a key that are the record's own spelling and not a field.
NOT_A_FIELD = frozenset(('order', 'leader', 'group', 'g', 'build', 'guy', 'path', 'city',
                         'move', 'attack', 'unit', 'pool', 'death', 'ammo'))


def firsts(lines):
    """`[(frame, who, o, what, values)]` from a widening's print."""
    out = []
    for line in lines:
        m = FIRST.match(re.sub(r'\x1b\[[0-9;]*m', '', line.rstrip('\n')))
        if m:
            f1, f2, f3, f4, who, o, what, values = m.groups()
            frame = next(f for f in (f1, f2, f3, f4) if f is not None)
            out.append((int(frame), int(who), int(o), what, values))
    return out


def field_of(what):
    """The field a key names: `order:move.dest` → `dest`, `g.cur_time[0]` →
    `cur_time`, `leader:pop_cap` → `pop_cap`, `group` → `group`."""
    parts = [p for p in re.split(r'[:.]', re.sub(r'\[[^\]]*\]', '', what)) if p]
    named = [p for p in parts if p not in NOT_A_FIELD]
    return (named or parts)[-1]


def live_source(text):
    """A source text without its test module."""
    at = re.search(r'^#\[cfg\(test\)\]\s*\n\s*(?:pub(?:\([a-z]+\))? )?mod \w+', text, re.M)
    return text[:at.start()] if at else text


def readers(field, root=SIM):
    """`[(path, line)]` of every line of the crate, outside its tests, that
    reads `.field`. A line that only assigns it is a writer and is left out."""
    rx = re.compile(r'\.' + re.escape(field) + r'\b(?!\s*(?:=[^=]|\+=|-=|\|=|&=))')
    out = []
    for f in sorted(root.rglob('*.rs')):
        if f.name.endswith('_tests.rs') or f.name in ('docs_guard.rs', 'no_float.rs', 'soak.rs'):
            continue
        for n, line in enumerate(live_source(f.read_text()).split('\n'), 1):
            s = line.lstrip()
            if s.startswith('//'):
                continue
            if rx.search(line):
                out.append((f.relative_to(root.parent.parent.parent) if root == SIM
                            else f.relative_to(root), n))
    return out


def report(rows, block=None, every=False, show=4):
    if not rows:
        return ['no `first <frame> <who>/<o> <what>: …` line on the input: '
                'run the widening with RON_FIRSTS=1 and --nocapture'], 0
    frames = sorted({r[0] for r in rows})
    if every:
        chosen = frames
    else:
        chosen = [frames[0] if block is None else block]
    out = []
    read = 0
    for frame in chosen:
        keys = defaultdict(list)
        for f, who, o, what, values in rows:
            if f == frame:
                keys[field_of(what)].append((who, o, what, values))
        kind = 'stand on' if frame == frames[0] else 'first part on'
        out.append(f'block {frame}: {sum(len(v) for v in keys.values())} keys {kind} it, '
                   f'{len(keys)} fields')
        found = {field: readers(field) for field in keys}
        quiet = sorted(f for f in keys if not found[f])
        for field in sorted(keys, key=lambda f: (-len(found[f]), f)):
            if not found[field]:
                continue
            read += 1
            units = keys[field]
            who, o, what, values = units[0]
            more = f' and {len(units) - 1} more' if len(units) > 1 else ''
            out.append(f'  {field}: {len(found[field])} readers in sim — '
                       f'`{who}/{o} {what}` {values}{more}')
            for path, n in found[field][:show]:
                out.append(f'      {path}:{n}')
            if len(found[field]) > show:
                out.append(f'      … {len(found[field]) - show} more')
        if quiet:
            out.append(f'  nothing in sim reads, by that name: {", ".join(quiet)}')
    return out, read


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('--block', type=int, help='the block whose keys to list (default: the first)')
    ap.add_argument('--all', action='store_true', help='every block, by frame')
    ap.add_argument('--show', type=int, default=4, help='reader lines a field')
    args = ap.parse_args()
    lines, _ = report(firsts(sys.stdin), args.block, args.all, args.show)
    print('\n'.join(lines))
    return 0


if __name__ == '__main__':
    sys.exit(main())
