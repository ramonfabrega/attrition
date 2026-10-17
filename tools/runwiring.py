#!/usr/bin/env python3
"""runwiring.py — where a capture is wired through the harness, place by
place, so a closing word's "five things" (parked 1217) and a new pair's six
edits (1728) are read off a print instead of a diff (parked 1674 (g), 1729:
three reaches).

    python3 tools/runwiring.py run763
    python3 tools/runwiring.py run776 --json
    python3 tools/runwiring.py --chapter 91     # a chapter's six places (Loop 2914)

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
GOLDEN = ROOT / 'tools/gamelog/golden'
# Where a chapter is wired (Loop 2914, three reaches: six files by hand for
# every chapter since eighty, the staged-line count and the words array's
# length hand-counted and each wrong once). A chapter is named three ways
# — `chapterN`, `chN` and `chapter_<words>` — and every place is one of them.
CHAPTER_PLACES = (
    ('script list', 'golden.rs', r'"chapter{n}\.cmd"'),
    ('walk', '.rs', r'walk_(?:script|chapter)\("ch{n}"'),
    ('walk test', '.rs', r'fn chapter_{w}_holds_to_the_golden_word'),
    ('widening test', '.rs', r'fn chapter_{w}_s_window_is_widened_whole'),
    ('GOLDEN_WORD pin', 'GOLDEN_WORD_CHAPTER_{W}', r'.'),
    ('WIDENINGS', 'WIDENINGS', r'chapter_{w}_s_window'),
    ('GOLDEN_WORDS row', 'GOLDEN_WORDS', r'"ch{n}"'),
    ('coverage driver', 'every_parsed_field_is_compared_by_the_instrument_or_pinned', r'ch{n}|chapter_{w}'),
    ('runs ledger', 'RUNS.md', r'chapter {n}\b|chapter {h}\b'),
)
ONES = ('zero one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen '
        'sixteen seventeen eighteen nineteen').split()
TENS = ('', '', 'twenty', 'thirty', 'forty', 'fifty', 'sixty', 'seventy', 'eighty', 'ninety')
WALK = re.compile(r'walk_(?:script|chapter)\("ch(\d+)",(?: "chapter\d+",)? (\d+), (\d+), (\d+)\)')


def number_word(n, sep='_'):
    """`ninety_one` for 91, as the harness names a chapter's tests."""
    if n < 20:
        return ONES[n]
    if n < 100:
        return TENS[n // 10] + (sep + ONES[n % 10] if n % 10 else '')
    return ONES[n // 100] + sep + 'hundred' + (sep + number_word(n % 100, sep) if n % 100 else '')


def chapter_mentions(n, texts):
    """Every line naming chapter `n` by any of its spellings, with its scope."""
    w = number_word(n)
    # `GOLDEN_WORD_CHAPTER_NINETY` names the chapter after an underscore, so the uppercase form has no left boundary.
    word = re.compile(rf'(?:(?<![A-Za-z0-9_])(?:chapter{n}|ch{n}|chapter_{w}|chapter {n}|chapter {number_word(n, "-")})|CHAPTER_{w.upper()})(?![0-9A-Za-z])')
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


def chapter_places(n, found):
    """Each expected place of a chapter as `(place, [lines])`."""
    w = number_word(n)
    fill = {'n': n, 'w': w, 'W': w.upper(), 'h': number_word(n, '-')}
    out = []
    for place, where, pattern in CHAPTER_PLACES:
        rx = re.compile(pattern.format(**fill))
        where = where.format(**fill)
        hits = []
        for name, line, scope, text in found:
            in_place = (name.endswith(where) if where.startswith('.') or where.endswith('.md') or where.endswith('.rs')
                        else scope == where)
            if in_place and rx.search(text):
                hits.append(f'{name}:{line}')
        out.append((place, hits))
    return out


def staged_count(n, found, cmd_text):
    """The `.cmd`'s own line count beside what the walk asserts: `(in the
    script, asserted by the walk or None)` — a miscount is a script nobody
    staged, and run889's first walk had one (2915)."""
    lines = sum(1 for l in cmd_text.split('\n') if l.strip() and not l.lstrip().startswith('#'))
    asserted = None
    for _, _, _, text in found:
        m = WALK.search(text)
        if m and int(m.group(1)) == n:
            asserted = int(m.group(3))
    return lines, asserted


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
    ap.add_argument('run', nargs='?', help='runNNN')
    ap.add_argument('--chapter', type=int, metavar='N',
                    help="a chapter's wiring instead of a run's: its three spellings, the places, and the "
                         "staged-line count against the walk's (Loop 2914)")
    ap.add_argument('--json', action='store_true')
    args = ap.parse_args(argv)
    if args.chapter is None and args.run is None:
        ap.error('a run (runNNN) or --chapter N')
    texts = {str(f.relative_to(ROOT)): f.read_text() for f in sorted(HARNESS.rglob('*.rs'))}
    texts[str(RUNS.relative_to(ROOT))] = RUNS.read_text()
    if args.chapter is not None:
        n = args.chapter
        found = chapter_mentions(n, texts)
        checked = chapter_places(n, found)
        cmd = GOLDEN / f'chapter{n}.cmd'
        staged = staged_count(n, found, cmd.read_text()) if cmd.exists() else (None, None)
        if args.json:
            print(json.dumps({'chapter': n, 'mentions': found, 'places': dict(checked),
                              'staged': {'script': staged[0], 'walk': staged[1]}}, indent=1))
            return 0
        for name, line, scope, text in found:
            print(f'{name}:{line}  in {scope or "(file)"}\n    {text[:200]}')
        print(f'\n{len(found)} lines name chapter {n} (chapter{n}, ch{n}, chapter_{number_word(n)})\n')
        for place, hits in checked:
            print(f'{place:22} ' + (', '.join(hits) if hits else 'missing'))
        if staged[0] is None:
            print(f'\nstaged lines: no {cmd.relative_to(ROOT)}')
        else:
            print(f'\nstaged lines: {staged[0]} in {cmd.relative_to(ROOT)}; the walk asserts '
                  + ('none' if staged[1] is None else str(staged[1]))
                  + ('' if staged[1] in (None, staged[0]) else '  <- a miscount'))
        return 0
    run = args.run if args.run.startswith('run') else f'run{args.run}'
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
