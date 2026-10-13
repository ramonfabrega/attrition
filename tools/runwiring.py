#!/usr/bin/env python3
"""runwiring.py — where a capture is wired through the harness, place by
place, so a closing word's "five things" (parked 1217) and a new pair's six
edits (1728) are read off a print instead of a diff (parked 1674 (g), 1729:
three reaches).

    python3 tools/runwiring.py run763
    python3 tools/runwiring.py run776 --json

Every line of `crates/rondata/src` and `docs/RUNS.md` that names the run is
listed under the constant or function it sits in, and the places a run is
expected to touch are then checked off: the runs ledger's section, the
archive file constant, the widening test (`runNNN_s_…_is_widened_whole`),
`WIDENINGS`, `GROUND_INEXACT`, the coverage driver (`coverage.rs`), the
compared pin's window (`UNCOMPARED_BY_THE_INSTRUMENT`), `AI_WORDS`, the
floors and `ENDPOINTS`. A place with no line is printed as `missing`; a run
need not touch every place — a packet touches two — so the list is what
to read, never a verdict.
"""
import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HARNESS = ROOT / 'crates/rondata/src'
RUNS = ROOT / 'docs/RUNS.md'
# Where a run is expected to be wired, and the text that marks the place:
# a constant's or function's name the enclosing scope carries.
PLACES = (
    ('runs ledger', 'RUNS.md', r'^## run{n}\b'),
    ('archive file', '.rs', r'gamelog-run{n}-'),
    ('widening test', '.rs', r'fn run{n}_s_\w+_is_widened_whole'),
    ('WIDENINGS', 'WIDENINGS', r'run{n}'),
    ('GROUND_INEXACT', 'GROUND_INEXACT', r'run{n}'),
    ('coverage driver', 'every_parsed_field_is_compared_by_the_instrument_or_pinned', r'run{n}'),
    ('compared pin window', 'UNCOMPARED_BY_THE_INSTRUMENT', r'run{n}'),
    ('AI_WORDS', 'AI_WORDS', r'run{n}'),
    ('floors', 'FLOORS', r'run{n}'),
    ('ENDPOINTS', 'ENDPOINTS', r'run{n}'),
)
SCOPE = re.compile(r'^\s*(?:pub(?:\([a-z]+\))?\s+)?(?:const\s+([A-Z_][A-Z0-9_]*)|(?:async\s+)?fn\s+([a-z0-9_]+))')


def mentions(run, texts):
    """`[(file, line, scope, text)]` for every line naming `run` in `texts`
    (`{file name: text}`), with the constant or function the line sits in."""
    n = run[3:] if run.startswith('run') else run
    # `run763_s_…` is a mention too, so the boundary is a digit, not `\b`.
    word = re.compile(rf'(?<![A-Za-z0-9])run{n}(?![0-9])')
    out = []
    for name, text in sorted(texts.items()):
        scope = ''
        for i, line in enumerate(text.split('\n'), 1):
            m = SCOPE.match(line)
            if m:
                scope = m.group(1) or m.group(2)
            if word.search(line):
                out.append((name, i, scope, line.strip()))
    return out


def places(run, found):
    """Each expected place as `(place, [lines])`, from `mentions`' rows."""
    n = run[3:] if run.startswith('run') else run
    out = []
    for place, where, pattern in PLACES:
        rx = re.compile(pattern.format(n=n))
        hits = []
        for name, line, scope, text in found:
            in_place = (name.endswith(where) if where.startswith('.') or where.endswith('.md')
                        else scope == where)
            if in_place and rx.search(text):
                hits.append(f'{name}:{line}')
        out.append((place, hits))
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('run', help='runNNN')
    ap.add_argument('--json', action='store_true')
    args = ap.parse_args(argv)
    run = args.run if args.run.startswith('run') else f'run{args.run}'
    texts = {str(f.relative_to(ROOT)): f.read_text() for f in sorted(HARNESS.rglob('*.rs'))}
    texts[str(RUNS.relative_to(ROOT))] = RUNS.read_text()
    found = mentions(run, texts)
    checked = places(run, found)
    if args.json:
        print(json.dumps({'run': run, 'mentions': found, 'places': dict(checked)}, indent=1))
        return 0
    for name, line, scope, text in found:
        print(f'{name}:{line}  in {scope or "(file)"}\n    {text[:200]}')
    print(f'\n{len(found)} lines name {run}\n')
    for place, hits in checked:
        print(f'{place:22} ' + (', '.join(hits) if hits else 'missing'))
    return 0


if __name__ == '__main__':
    sys.exit(main())
