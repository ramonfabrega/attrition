#!/usr/bin/env python3
"""refill.py — who already holds a lobby triple, read off the tree.

    python3 tools/refill.py 18 6 4        # (map_style, ai_tribe, difficulty)
    python3 tools/refill.py --list        # every occupied triple
    python3 tools/refill.py --nation 6    # every triple one nation holds

A refill (the battery's next lobby, a pair's next age) is booked on a triple
that nothing already scores. Item 3752's booking and brief named
`(14, 11, 5)` free by entry 72's rule (3), and the triple already stood in
`battery::SCORED_LOBBIES` as the second pair's lobby; the lane entered the
pair there rather than minting (Loop 3769). A rule says which triple ought
to be free; only the tree says which one is.

It reads, from `crates/rondata/src/diff/`:

- `battery.rs`'s `SCORED_LOBBIES` entries, each with the comment lines
  above it (which name the item and the pair); a triple a comment cites
  is not an entry, so comments are stripped before a tuple is matched;
- every struct literal spelling `map_style: N, ai_tribe: N, difficulty: N`
  — a `Pair` in a `*_pair.rs` module (or `sahara_*.rs`), a `Lobby` in an
  AI word's `*_word.rs`, and the held-out `battery::BATTERY` lobbies.

The first three pairs (`second.rs`, `third.rs`, `third_pair.rs`) carry no
literal — their lobbies are their dumps' `GAME INFO` — and `SCORED_LOBBIES`
is where they are held; `--list` names any pair module with no literal,
and any `map_style:` the three-field pattern did not parse.

Exit 0 and `free` when nothing holds the triple; exit 1 and the occupants
otherwise. It reads and prints; it decides nothing.
"""
import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DIFF = 'crates/rondata/src/diff'

TRIPLE = re.compile(
    r'map_style:\s*(-?\d+)\s*,\s*ai_tribe:\s*(-?\d+)\s*,\s*difficulty:\s*(-?\d+)')
ANY_MAP_STYLE = re.compile(r'\bmap_style:\s*-?\d+')
SCORED = re.compile(r'const\s+SCORED_LOBBIES\s*:[^=]*=\s*&\[(.*?)\n\];', re.S)
TUPLE = re.compile(r'\(\s*(-?\d+)\s*,\s*(-?\d+)\s*,\s*(-?\d+)\s*\)')
CONST = re.compile(r'const\s+(\w+)\s*:\s*\w+\s*=\s*\w+\s*\{')
NAME = re.compile(r'name:\s*"([^"]*)"')


@dataclass
class Occupant:
    kind: str    # scored | pair | word | battery | module
    where: str   # file, const, name
    note: str = ''


def strip_line(line):
    """The line without its `//` comment; a `//` inside a string stays."""
    in_str = False
    i = 0
    while i < len(line):
        c = line[i]
        if c == '\\' and in_str:
            i += 2
            continue
        if c == '"':
            in_str = not in_str
        elif not in_str and line.startswith('//', i):
            return line[:i]
        i += 1
    return line


def strip(text):
    return '\n'.join(strip_line(l) for l in text.split('\n'))


def scored(text):
    """`SCORED_LOBBIES`' entries, each with the comment block above it."""
    m = SCORED.search(text)
    if not m:
        return []
    out, notes = [], []
    for line in m.group(1).split('\n'):
        s = line.strip()
        if s.startswith('//'):
            notes.append(s.lstrip('/').strip())
            continue
        code = strip_line(line)
        found = TUPLE.findall(code)
        for t in found:
            out.append((tuple(int(x) for x in t), ' '.join(notes)))
        if found:
            notes = []
    return out


def kind_of(path):
    n = path.name
    if n == 'battery.rs':
        return 'battery'
    if n.endswith('_pair.rs') or n.startswith('sahara_'):
        return 'pair'
    if n.endswith('_word.rs'):
        return 'word'
    return 'module'


def literals(path, root):
    """Every struct literal in `path` that spells the three lobby fields."""
    code = strip(path.read_text())
    rel = path.relative_to(root / DIFF).as_posix()
    out = []
    for m in TRIPLE.finditer(code):
        head = code[:m.start()]
        opener = head.rfind('{')
        consts = list(CONST.finditer(head))
        const = consts[-1].group(1) if consts and consts[-1].end() - 1 == opener else ''
        names = NAME.findall(code[opener:m.start()]) if opener >= 0 else []
        where = ' '.join(x for x in (rel, const, f'"{names[-1]}"' if names else '') if x)
        out.append((tuple(int(x) for x in m.groups()), Occupant(kind_of(path), where)))
    return out


def sources(root):
    return sorted((root / DIFF).rglob('*.rs'))


def occupants(root=ROOT):
    """`{triple: [Occupant]}` for every triple the tree holds."""
    occ = {}
    battery = root / DIFF / 'battery.rs'
    if battery.exists():
        for t, note in scored(battery.read_text()):
            occ.setdefault(t, []).append(Occupant('scored', 'battery.rs SCORED_LOBBIES', note))
    for path in sources(root):
        for t, o in literals(path, root):
            occ.setdefault(t, []).append(o)
    return occ


def unparsed(root=ROOT):
    """Every `map_style: N` (comments stripped) the three-field pattern missed."""
    out = []
    for path in sources(root):
        code = strip(path.read_text())
        hit = {m.start() for m in TRIPLE.finditer(code)}
        for m in ANY_MAP_STYLE.finditer(code):
            if m.start() not in hit:
                line = code.count('\n', 0, m.start()) + 1
                out.append(f'{path.relative_to(root).as_posix()}:{line}')
    return out


def bare_pairs(root=ROOT):
    """Pair modules with no lobby literal (their lobby is a dump's)."""
    return [p.name for p in sorted((root / DIFF).glob('*_pair.rs'))
            if not TRIPLE.search(strip(p.read_text()))]


def show(t, occ):
    print(f'{t}')
    for o in occ:
        print(f'  {o.kind:8} {o.where}')
        if o.note:
            print(f'           // {o.note}')


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('triple', nargs='*', type=int,
                    help='map_style ai_tribe difficulty')
    ap.add_argument('--list', action='store_true', help='every occupied triple')
    ap.add_argument('--nation', type=int, help='every triple an ai_tribe holds')
    ap.add_argument('--root', type=Path, default=ROOT, help=argparse.SUPPRESS)
    a = ap.parse_args(argv)
    occ = occupants(a.root)
    if a.list or a.nation is not None:
        keys = sorted(k for k in occ if a.nation is None or k[1] == a.nation)
        for k in keys:
            show(k, occ[k])
        if a.list:
            for b in bare_pairs(a.root):
                print(f'note: {b} carries no lobby literal (its lobby is its dumps\' GAME INFO)')
            for u in unparsed(a.root):
                print(f'note: {u} spells map_style without ai_tribe, difficulty after it')
        if not keys:
            print('free')
        return 0
    if len(a.triple) != 3:
        ap.error('give map_style ai_tribe difficulty, --list or --nation')
    t = tuple(a.triple)
    if t not in occ:
        print(f'{t} free')
        return 0
    show(t, occ[t])
    return 1


if __name__ == '__main__':
    sys.exit(main())
