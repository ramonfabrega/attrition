#!/usr/bin/env python3
"""paths.py — the simulation's source paths the tree cites, checked and moved.

    python3 tools/paths.py --check                 # every cited path names a file
    python3 tools/paths.py --check --root <tree>   # on another checkout
    python3 tools/paths.py move OLD NEW [OLD NEW …] [--dry-run]

`docs/` cites `crates/sim/src/<file>.rs` about three thousand times, and the
structure track (`docs/STRUCTURE.md` §5) moves those files into directories.
A citation that names a file no longer there is a reader sent nowhere, and
nothing failed on one until item 4008.

**What a citation is.** The text `crates/sim/src/` or `crates/sim/tests/`
(item 4009) and the path characters
after it (letters, digits, `_ . / - *`, and `{a,b}` brace lists), trailing
punctuation dropped. It resolves when every path it spells names a file or
directory that exists; one carrying `*` is a glob and resolves when it
matches at least one. `--check` reads every
`docs/**/*.md`, every `crates/**/*.rs` but the guard that pins the known
list, and `CLAUDE.md`, prints each
citation that resolves to nothing as `file:line: citation`, and exits 1 if
there is one. `crates/sim/src/docs_guard.rs` runs it and holds the known
list (empty at item 4008's base; it may only shrink).

**`move`** rewrites each `OLD` citation to `NEW` in the same files plus the
text files under `tools/` (never `tools/emu/`: the emulator cache's key
hashes every file there, `docs/STRUCTURE.md` §5). Both are repository paths
under `crates/sim/src/` or a sibling. A file path is rewritten where it
stands whole — `air.rs` never inside `airbase.rs` or `air.rs-E`; a path
ending in `/` is a directory, and every citation under it is rewritten by
its prefix. It moves no file: `git mv` does that, and `--check` afterwards
says whether every citation followed.

Tests: `tools/test_paths.py`.
"""
import argparse
import glob
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PREFIX = 'crates/sim/src/'
# Item 4009 moved the standalone sweeps to `crates/sim/tests/sweep/`, so the
# crate's `tests/` is cited and checked as `src/` is.
BARE = (PREFIX, 'crates/sim/tests/')
CITATION = re.compile(r'crates/sim/(?:src|tests)/(?:[A-Za-z0-9_./*-]|\{[A-Za-z0-9_.,*-]*\})*')


# The guard that holds `--check`'s known list quotes the citations it pins.
GUARD = 'crates/sim/src/docs_guard.rs'


def cited_files(root):
    """`docs/**/*.md`, `crates/**/*.rs` and `CLAUDE.md`."""
    out = sorted((root / 'docs').rglob('*.md'))
    out += sorted(p for p in (root / 'crates').rglob('*.rs') if 'target' not in p.parts)
    if (root / 'CLAUDE.md').exists():
        out.append(root / 'CLAUDE.md')
    return out


def checked_files(root):
    """The files whose citations `--check` holds to the tree: every cited
    file but the guard that pins the known list."""
    return [p for p in cited_files(root) if p.relative_to(root).as_posix() != GUARD]


def rewritten_files(root):
    """The files `move` rewrites: the cited ones (the guard's seam table
    is keyed by path) and `tools/`'s text, never `tools/emu/`."""
    out = cited_files(root)
    tools = root / 'tools'
    if tools.is_dir():
        for p in sorted(tools.rglob('*')):
            rel = p.relative_to(root).parts
            if not p.is_file() or rel[:2] == ('tools', 'emu') or '.venv' in rel:
                continue
            if p.suffix in ('.py', '.sh', '.md', '.rs', '.txt', '.toml', '.json'):
                out.append(p)
    return out


def citations(text):
    """`(line, citation)` for each citation in `text`, its trailing
    punctuation dropped."""
    out = []
    for i, line in enumerate(text.split('\n'), 1):
        for m in CITATION.finditer(line):
            cite = m.group(0)
            if not cite.endswith('/'):
                cite = cite.rstrip('.,-')
            if cite not in BARE:
                out.append((i, cite))
    return out


def expand(cite):
    """A citation's brace lists spelt out, shell-style: `a_{b,c}.rs` is
    `a_b.rs` and `a_c.rs`."""
    m = re.search(r'\{([^{}]*)\}', cite)
    if not m:
        return [cite]
    out = []
    for alt in m.group(1).split(','):
        out += expand(cite[:m.start()] + alt + cite[m.end():])
    return out


def resolves(root, cite):
    """Every path the citation spells names something: a glob matches at
    least one."""
    for one in expand(cite):
        if '*' in one:
            if not glob.glob(str(root / one)):
                return False
        elif not (root / one.rstrip('/')).exists():
            return False
    return True


def broken(root):
    """Every citation under `root` that names nothing, as `(file, line,
    citation)`."""
    out = []
    for path in checked_files(root):
        try:
            text = path.read_text()
        except UnicodeDecodeError:
            continue
        for line, cite in citations(text):
            if not resolves(root, cite):
                out.append((str(path.relative_to(root)), line, cite))
    return out


def pattern(old):
    """The regex that finds `old` where it stands whole: a directory by its
    prefix, a file not followed by a path character (`air.rs` is not in
    `air.rs-E`, and `air` is not in `airbase.rs`)."""
    if old.endswith('/'):
        return re.compile(re.escape(old))
    return re.compile(re.escape(old) + r'(?![A-Za-z0-9_*-]|\.[A-Za-z0-9]|/)')


def moved(path, moves):
    """`path` after the moves, or `None` when none of them touches it."""
    for old, new in moves:
        if old.endswith('/') and path.startswith(old):
            return new + path[len(old):]
        if path == old:
            return new
    return None


def rewrite_braces(cite, moves):
    """A brace-list citation with each moved alternative rewritten in place
    — `crates/sim/src/{air,nuke}.rs` with `air.rs` → `air/mod.rs` is
    `crates/sim/src/{air/mod,nuke}.rs` — or `None` when an alternative
    moved somewhere the list's prefix and suffix cannot spell. An
    alternative that keeps its name and changes only the directory before
    it starts a list of its own under its new head, the lists joined by
    `, ` in the order their first alternative stood (item 4009: the
    standalone sweeps left `crates/sim/src/sweep/` for
    `crates/sim/tests/sweep/` and `mod.rs` stayed), and a list of one is
    spelt plainly."""
    m = re.search(r'\{([^{}]*)\}', cite)
    head, tail = cite[:m.start()], cite[m.end():]
    groups = {}
    for alt in m.group(1).split(','):
        to = moved(head + alt + tail, moves)
        if to is None:
            groups.setdefault(head, []).append(alt)
        elif to.startswith(head) and to.endswith(tail) and len(to) >= len(head) + len(tail):
            groups.setdefault(head, []).append(to[len(head):len(to) - len(tail)])
        elif to.endswith(alt + tail):
            groups.setdefault(to[:len(to) - len(alt + tail)], []).append(alt)
        else:
            return None
    return ', '.join(h + ('{' + ','.join(a) + '}' if len(a) > 1 else a[0]) + tail
                     for h, a in groups.items())


def rewrite(text, moves, unmoved=None):
    """`text` with each `(old, new)` citation rewritten, and how many. A
    brace list that cannot be rewritten is appended to `unmoved`."""
    n = 0
    for old, new in moves:
        if old in text:
            text, k = pattern(old).subn(new, text)
            n += k

    def braces(m):
        nonlocal n
        whole = m.group(0)
        cite = whole if whole.endswith('/') else whole.rstrip('.,-')
        rest = whole[len(cite):]
        if '{' not in cite or len(re.findall(r'\{', cite)) != 1:
            return whole
        if not any(moved(one, moves) for one in expand(cite)):
            return whole
        out = rewrite_braces(cite, moves)
        if out is None:
            if unmoved is not None:
                unmoved.append(cite)
            return whole
        n += 1
        return out + rest

    return CITATION.sub(braces, text), n


def move(root, moves, dry_run=False):
    """Rewrite each move across the tree; `[(file, count)]` of files touched."""
    for old, new in moves:
        if old.endswith('/') != new.endswith('/'):
            raise SystemExit(f'paths.py: {old} and {new}: a directory moves to a directory')
    touched = []
    for path in rewritten_files(root):
        try:
            text = path.read_text()
        except UnicodeDecodeError:
            continue
        unmoved = []
        new_text, n = rewrite(text, moves, unmoved)
        for cite in unmoved:
            print(f'{path.relative_to(root)}: not rewritten, a brace list: {cite}', file=sys.stderr)
        if n:
            touched.append((str(path.relative_to(root)), n))
            if not dry_run:
                path.write_text(new_text)
    return touched


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('--root', type=Path, default=ROOT)
    ap.add_argument('--check', action='store_true', help='list citations that name nothing')
    ap.add_argument('--dry-run', action='store_true')
    ap.add_argument('verb', nargs='?', choices=['move'])
    ap.add_argument('pairs', nargs='*', metavar='OLD NEW')
    args = ap.parse_args(argv)
    root = args.root.resolve()
    if args.check:
        bad = broken(root)
        for f, line, cite in bad:
            print(f'{f}:{line}: {cite}')
        print(f'paths: {len(bad)} citation(s) of {PREFIX} name nothing', file=sys.stderr)
        return 1 if bad else 0
    if args.verb != 'move' or not args.pairs or len(args.pairs) % 2:
        ap.error('give --check, or move OLD NEW [OLD NEW ...]')
    moves = list(zip(args.pairs[::2], args.pairs[1::2]))
    touched = move(root, moves, args.dry_run)
    for f, n in touched:
        print(f'{f}: {n}')
    total = sum(n for _, n in touched)
    print(f'paths: {total} citation(s) in {len(touched)} file(s)'
          + (' (dry run)' if args.dry_run else ''), file=sys.stderr)
    return 0


if __name__ == '__main__':
    sys.exit(main())
