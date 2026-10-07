#!/usr/bin/env python3
"""brief.py — compose a worker's brief from the frame and the queue.

    python3 tools/brief.py 1133 --kind residue --runs run423,run424 \\
        --section 'docs/AI.md §87' --read docs/journal/2026-09-28-item-1066.md \\
        --note-file notes-1133.md > brief-1133.md

A brief is four things, and this writes three of them:

- **the frame** (`tools/brief/frame.md`), which the steering pass writes and
  the brief checklist is held to — what every brief of a kind says;
- **the item, in the queue's own words**, quoted from `docs/QUEUE.md` and
  never retold, with the other open items as the other lanes' words;
- **what is reserved**: run numbers and section numbers, the two things two
  lanes can take in silence. Code is not fenced (`docs/DECISIONS.md` 55);
- **the commander's own note** (`--note-file`), which is where its judgement
  goes: what this item alone needs, in its own words, as short as it can be.

Kinds: `residue` — a word on the AI track, a pair's or the third map's;
`chapter` — the rules track's. A refusal is a `ValueError` and exit 2: an
item the queue does not book has no brief.

Why (the nineteenth pass, 2026-09-29): a brief was the previous brief,
copied from the job's scratch directory and edited, and it had grown from
4,700 characters to 11,000 in ten days. `tools/explore/test_brief.py`.
"""
import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FRAME = ROOT / 'tools/brief/frame.md'
# The frame's size, in characters, and it may only fall (parked 1226, the
# twenty-first pass): a row added by a pass is paid for by one struck.
# 12,959 was the twentieth pass's; the twenty-first left it under 13,000.
FRAME_CEILING = 13_000
QUEUE = ROOT / 'docs/QUEUE.md'

# The frame's sections a kind of brief carries, in the order they are read.
KINDS = {
    'residue': ('opening', 'booked', 'lanes', 'before-reading', 'reading',
                'capture', 'build', 'landing'),
    'chapter': ('opening', 'booked', 'lanes', 'before-reading', 'reading',
                'chapter', 'capture', 'build', 'landing'),
    # A sweep batch (DECISIONS 63 (iv), parked 1629): no word, no booking
    # paragraph — its section says how a fanned batch is cut and merged.
    'sweep': ('opening', 'lanes', 'sweep', 'landing'),
}
TRACKS = {'residue': 'the AI track', 'chapter': 'the rules track', 'sweep': 'the sweep track'}
# The lane gate's filters every word lane runs (parked 1623): a widening
# puts its window in the coverage driver, so `coverage::` is a word's own.
ALWAYS_TESTS = {'residue': ('coverage::',), 'chapter': ('coverage::',), 'sweep': ('sweep::',)}
# Rows of the checklist that are the commander's to follow in composing, or
# the pass's, and say nothing to a worker.
NOT_A_BRIEF_S = frozenset()
N = r'\d{1,4}(?!\d)'


def sections(frame):
    """`{name: body}` for every `## name` of the frame."""
    out = {}
    for m in re.finditer(r'^## ([a-z-]+)\n(.*?)(?=^## |\Z)', frame, re.S | re.M):
        out[m.group(1)] = m.group(2).strip('\n')
    return out


def open_items_text(queue):
    """`{number: the item's text as written}` for the queue's open items."""
    a = queue.index('## The queue\n')
    b = queue.find('\n## ', a + 1)
    body = queue[a:b if b > 0 else len(queue)]
    out = {}
    for m in re.finditer(rf'^({N})\. \*\*.*?(?=^{N}\. \*\*|\Z)', body, re.S | re.M):
        out[int(m.group(1))] = m.group(0).rstrip('\n')
    return out


def open_items(queue):
    """`[(number, headline)]`, the bold opening of each open item."""
    out = []
    for n, text in open_items_text(queue).items():
        head = re.search(r'\*\*(.*?)\*\*', text, re.S)
        out.append((n, ' '.join(head.group(1).split()) if head else ''))
    return out


def filters(kind, tests):
    """The lane gate's `--tests` for a brief: what the commander names,
    and the kind's own always among them (1623: 1611's lane gate never ran
    the coverage pins its widening moved, and the booking gate was red)."""
    tests = [t for t in (tests or ()) if t]
    if kind != 'sweep' and not tests:
        raise ValueError("a word's brief names its lane gate's filters: --tests FILTER [FILTER ...]")
    return tests + [t for t in ALWAYS_TESTS[kind] if t not in tests]


def compose(item, kind, *, queue, frame, base, model, runs, sections: list, read, note, tests=()):
    if kind not in KINDS:
        raise ValueError(f'no such kind of brief: {kind}; one of {sorted(KINDS)}')
    tests = filters(kind, tests)
    items = open_items_text(queue)
    if item not in items:
        raise ValueError(f'docs/QUEUE.md books no item {item}; its open items are {sorted(items)}')
    body = globals()['sections'](frame)
    missing = [n for n in KINDS[kind] if n not in body]
    if missing:
        raise ValueError(f'tools/brief/frame.md has no section {missing}')

    def fill(text):
        return (text.replace('{item}', str(item)).replace('{model}', model)
                .replace('{base}', base).replace('{track}', TRACKS[kind])
                .replace('{tests}', ' '.join(tests)))

    others = [(n, h) for n, h in open_items(queue) if n != item]
    out = [fill(body['opening']), '']
    out += ['## The item, as the queue books it', '', items[item], '']
    if 'booked' in KINDS[kind]:
        out += [fill(body['booked']), '']
    out += ['## Read first', '']
    out += [f'- `docs/QUEUE.md`: the opener, and item {item}.']
    out += [f'- `{r}`' if not r.startswith('`') else f'- {r}' for r in read]
    out += ['- `docs/audit/README.md`, "The brief checklist": the story behind any row '
            'below, by its number, when a row is not clear.', '']
    out += ['## The other lanes', '', fill(body['lanes']), '']
    out += [f'- **{n}**: {h}' for n, h in others] or ['- None is booked.']
    out += ['', '## Reserved for you', '']
    out += ['- **Run numbers**: ' + (', '.join(runs) if runs else 'none; ask for one before a capture') + '.']
    out += ['- **Sections**: ' + ('; '.join(sections) if sections else
                                  'amend your documents\' sections in place; name a new '
                                  'section to me before you write it') + '.']
    titles = {'before-reading': 'Before any reading', 'reading': 'Reading the original',
              'capture': 'A capture or a packet', 'chapter': 'A chapter', 'sweep': 'The sweep',
              'build': 'Build', 'landing': 'Landing'}
    for name in KINDS[kind]:
        if name in ('opening', 'booked', 'lanes'):
            continue
        if name == 'landing' and note.strip():
            out += ['', '## This item', '', note.strip()]
        out += ['', f'## {titles[name]}', '', fill(body[name])]
    return '\n'.join(out) + '\n'


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('item', type=int)
    ap.add_argument('--kind', choices=sorted(KINDS), required=True)
    ap.add_argument('--runs', default='', help='comma-separated run numbers reserved for the item')
    ap.add_argument('--section', action='append', default=[], dest='sections',
                    help='a section reserved for the item, repeatable')
    ap.add_argument('--read', action='append', default=[],
                    help='a document or journal to read first, repeatable')
    ap.add_argument('--note-file', type=Path, help="the commander's own note for this item")
    ap.add_argument('--model', default='Opus 5.5')
    ap.add_argument('--tests', nargs='+', default=[],
                    help="the lane gate's filters for the word; `coverage::` is added (1623)")
    args = ap.parse_args()
    base = subprocess.run(['git', 'rev-parse', '--short=8', 'HEAD'], cwd=ROOT,
                          capture_output=True, text=True, check=True).stdout.strip()
    try:
        text = compose(args.item, args.kind, queue=QUEUE.read_text(), frame=FRAME.read_text(),
                       base=base, model=args.model,
                       runs=[r for r in args.runs.split(',') if r],
                       sections=args.sections, read=args.read,
                       note=args.note_file.read_text() if args.note_file else '',
                       tests=args.tests)
    except ValueError as e:
        print(f'brief.py: {e}', file=sys.stderr)
        sys.exit(2)
    sys.stdout.write(text)


if __name__ == '__main__':
    main()
