#!/usr/bin/env python3
"""seams.py — what this crate and its documents say they left out, by name.

    python3 tools/seams.py think_scout do_idle group           # by name
    python3 tools/seams.py --item 1214                         # the queue's chain
    python3 tools/seams.py --chain 'Unit::think_scout+0x941 < Unit::think+0x7da'
    python3 tools/seams.py --doors                             # seams a built door outlived

A word's cause has been written down before it was a word, more than once:
a `SEAM` in `crates/sim` or a "not modelled" row in a specification, under
a name the reader of the word's frame did not think to grep. In one tranche
(the twentieth pass, parked 1146, 1159, 1193): `soft_collision`'s
`TRADE_ROUTE` seam held East Indies at 6151, ORDERS §4.4's "not modelled"
region check held it at 6321, and `World::set_blocked_at`'s road clearing
stood as a seam four days after its door was built and held it at 7512.

Given the names on a word's draw chain and its parted fields, this prints:

- every **live** `SEAM` (not struck through) whose comment names one of
  them, or that sits inside a function of that name — flagged `unscanned`
  when it cites the disk for an absence and names no `scan:`;
- every line of a specification that says something is left out — "not
  modelled", "not built", "unbuilt", "simplif…", "not carried", "stub" —
  in a paragraph that names one of them.

`--doors` lists the live seams that name, as missing, a function this crate
now carries: a seam that left an arm out because its door was not built
outlives the door in silence.

It reads the tree and prints; it decides nothing. A name matches as a whole
word, case-blind, with `Class::` stripped: `Unit::think_scout` is
`think_scout`.
"""
import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SIM = ROOT / 'crates/sim/src'
DOCS = ROOT / 'docs'
QUEUE = DOCS / 'QUEUE.md'

# What a seam says when it cites the disk for an absence: docs_guard's list.
ABSENCES = ('no capture', 'no run', 'never entered', 'no dump', 'not on disk',
            'has not been captured', 'no trace')
LEFT_OUT = re.compile(
    r'not (?:yet )?model+ed|not (?:yet )?built|unbuilt|simplif|not carried|does not carry'
    r'|\bstub\b|left out|not wired|unwired', re.I)
MISSING = re.compile(
    r'not (?:yet )?built|unbuilt|not carried|does not carry|is not here|until .{0,40}? is built'
    r'|has no\b|not (?:yet )?model+ed', re.I)
# Documents that are not specifications: the story, the ledger, the rules.
NOT_A_SPEC = ('JOURNAL.md', 'QUEUE.md', 'PARKED.md', 'DECISIONS.md', 'RUNS.md')


def bare(name):
    """`Unit::think_scout+0x941` → `think_scout`."""
    name = re.sub(r'\+0x[0-9a-f]+$', '', name.strip().strip('`'))
    return name.split('::')[-1].split('@')[0]


def chain_names(text):
    """The function names of a draw chain as the queue writes one."""
    return [bare(m) for m in re.findall(r'[A-Za-z_][A-Za-z0-9_]*::[A-Za-z_][A-Za-z0-9_]*', text)]


def item_names(queue, item):
    a = queue.index('## The queue\n')
    m = re.search(rf'^{item}\. \*\*.*?(?=^\d{{1,4}}\. \*\*|^## |\Z)', queue[a:], re.S | re.M)
    if not m:
        raise ValueError(f'docs/QUEUE.md books no item {item}')
    # The chain's functions, and not the parted keys beside them: `group`,
    # `order` and `kind` are in every seam of the tree. A name that is one
    # common word (`think`) is left to be passed by hand.
    names = [n for n in chain_names(m.group(0)) if '_' in n or len(n) > 8]
    return list(dict.fromkeys(names))


def matcher(names):
    names = [n for n in dict.fromkeys(bare(n) for n in names) if n]
    if not names:
        return None
    return re.compile(r'(?<![A-Za-z0-9_])(' + '|'.join(re.escape(n) for n in names)
                      + r')(?![A-Za-z0-9_])', re.I)


def seams(text):
    """`(line, enclosing fn, the seam's text, struck)` for every `SEAM` of a
    source text. A comment block is read whole and split on the word."""
    lines = text.split('\n')
    out = []
    fn = ''
    i = 0
    while i < len(lines):
        s = lines[i].lstrip()
        m = re.match(r'(?:pub(?:\([a-z]+\))? )?(?:const )?fn ([a-z0-9_]+)', s)
        if m:
            fn = m.group(1)
        if not s.startswith('//'):
            i += 1
            continue
        start = i
        block = []
        while i < len(lines) and lines[i].lstrip().startswith('//'):
            block.append(lines[i].lstrip().lstrip('/!').strip())
            i += 1
        # A doc comment belongs to the function under it.
        under = ''
        j = i
        while j < len(lines) and (lines[j].lstrip().startswith('#[') or not lines[j].strip()):
            j += 1
        if j < len(lines):
            m = re.match(r'(?:pub(?:\([a-z]+\))? )?(?:const )?fn ([a-z0-9_]+)', lines[j].lstrip())
            if m:
                under = m.group(1)
        joined = ' '.join(block)
        for m in re.finditer(r'(~~)?SEAM\b', joined):
            nxt = re.search(r'SEAM\b', joined[m.end():])
            end = m.end() + nxt.start() if nxt else len(joined)
            one = joined[m.start():end].strip()
            struck = bool(m.group(1))
            out.append((start + 1, under or fn, one, struck))
    return out


def unscanned(seam):
    low = seam.lower()
    return any(a in low for a in ABSENCES) and 'scan: `' not in low


def carried():
    """Every function name `crates/sim/src` defines."""
    out = set()
    for f in sorted(SIM.rglob('*.rs')):
        out.update(re.findall(r'\bfn ([a-z0-9_]+)', f.read_text()))
    return out


def snake(name):
    name = bare(name)
    return re.sub(r'(?<=[a-z0-9])(?=[A-Z])', '_', name).lower()


def doors(root=SIM):
    """Live seams that name, as missing, a function this crate now carries."""
    have = carried()
    out = []
    for f in sorted(root.rglob('*.rs')):
        for line, fn, seam, struck in seams(f.read_text()):
            if struck or not MISSING.search(seam):
                continue
            live = re.sub(r'~~.*?~~', '', seam)
            named = {snake(n) for n in re.findall(r'`([A-Za-z_][A-Za-z0-9_:]*)(?:\(\))?`', live)}
            built = sorted(n for n in named if n in have and n != fn and len(n) > 5)
            if built:
                out.append((f.relative_to(ROOT), line, fn, built, seam))
    return out


def spec_rows(rx, docs=DOCS):
    out = []
    for f in sorted(docs.glob('*.md')):
        if f.name in NOT_A_SPEC:
            continue
        section = ''
        para = []
        start = 0
        for n, line in enumerate(f.read_text().split('\n') + [''], 1):
            if line.startswith('#'):
                section = line.lstrip('# ').strip()
            if line.strip():
                if not para:
                    start = n
                para.append(line.strip())
                continue
            text = re.sub(r'~~.*?~~', '', ' '.join(para))
            para = []
            if text and LEFT_OUT.search(text) and rx.search(text):
                out.append((f.relative_to(ROOT), start, section, text))
    return out


def clip(text, n):
    return text if len(text) <= n else text[:n - 1].rstrip() + '…'


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('names', nargs='*', help='function, field or constant names')
    ap.add_argument('--item', type=int, help="read the names off a queue item's chain and keys")
    ap.add_argument('--chain', help='a draw chain as the queue writes one')
    ap.add_argument('--doors', action='store_true',
                    help='list the live seams that name a function this crate now carries')
    ap.add_argument('--width', type=int, default=260)
    args = ap.parse_args()

    if args.doors:
        rows = doors()
        for f, line, fn, built, seam in rows:
            print(f'{f}:{line}  in `{fn}`  names {", ".join("`%s`" % b for b in built)}')
            print(f'    {clip(seam, args.width)}')
        print(f'{len(rows)} live seams name, as missing, a function this crate carries')
        return 0

    names = list(args.names)
    try:
        if args.item:
            names += item_names(QUEUE.read_text(), args.item)
    except ValueError as e:
        print(f'seams.py: {e}', file=sys.stderr)
        return 2
    if args.chain:
        names += chain_names(args.chain)
    rx = matcher(names)
    if rx is None:
        ap.error('no name given: pass names, --item or --chain')
    print('names: ' + ', '.join(dict.fromkeys(bare(n) for n in names)))

    found = 0
    for f in sorted(SIM.rglob('*.rs')):
        for line, fn, seam, struck in seams(f.read_text()):
            if struck:
                continue
            live = re.sub(r'~~.*?~~', '', seam)
            hit = rx.search(live) or (fn and rx.fullmatch(fn))
            if not hit:
                continue
            found += 1
            flag = '  unscanned' if unscanned(live) else ''
            print(f'\n{f.relative_to(ROOT)}:{line}  in `{fn}`{flag}')
            print(f'    {clip(live, args.width)}')
    print(f'\n{found} live seams name one of them, or sit in a function of that name')

    rows = spec_rows(rx)
    for f, line, section, text in rows:
        print(f'\n{f}:{line}  {clip(section, 70)}')
        print(f'    {clip(text, args.width)}')
    print(f'\n{len(rows)} paragraphs of a specification say something is left out and name one of them')
    return 0


if __name__ == '__main__':
    sys.exit(main())
