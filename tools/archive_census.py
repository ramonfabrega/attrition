#!/usr/bin/env python3
r"""archive_census.py — the capture archive, entry by entry: how big, and who reads it.

    archive_census.py [--root DIR] [--logs DIR] [--data DIR] [--repo DIR] [--top N] [--json]

Read-only. It deletes nothing, and has no flag that could: pruning is a
human's act, taken from the lists this prints (the disk ran from 103 GiB
free to 30 in one tranche of sixty-seven captures, 2026-10-19).

What it scans:

* `--root` (`$RON_GOLDEN_DIR`, default `~/ron-golden`, the same resolution
  `rondata::diff::golden` uses): every top-level entry — run directories
  `runNNN[a-z]`, chapter directories `chNN…`, and the odd ones (`g4`,
  `smoke`, …). A `frame-snapshot*.bin` inside one is a **packet** and is its
  own row (recreatable in about four minutes, `docs/EMULATOR.md` §8); the
  directory's row is the rest.
* `--logs`, the queue lane's archive (`tools/census.py`'s `ARCHIVE`:
  `$RON_GAMELOG_DIR`, default the profile's `Logs`): every file, one row
  each; a `.ronidx-…` sidecar follows the log it indexes; a subdirectory
  (`.ron-oracle-v1`, …) is one row.
* `--data` (`~/ron-data`): every other top-level entry, by size only,
  excluding the logs directory where it holds it.

Sizes are apparent (`st_size`) and allocated (`st_blocks * 512`; the archive
is APFS-compressed in place, `docs/ORACLE.md`), summed over files found
without following a symlink (a run directory links the install).

Who reads it, from the repo's text only (`--repo`, default this checkout):

* **pinned** — a Rust file under `crates/*/src` names it in *code* (a string
  literal: the `diff::` tables, the lab fixtures, `blind::TRACES`,
  `golden("ch2")`). A log is pinned by its exact file name, or through its
  run tag (`"run600"` pins `gamelog-run600-….txt`); a run directory by its
  name, which a pinned log's name carries (`gamelog-run977-…` pins
  `run977/`). Names match whole: `run95` is not `run956`, nor `run95x`.
* **rust-comment** — a Rust comment names it and no code does. Evidence a
  reader may follow, not a fixture; never a candidate.
* **ledger-only** — no Rust file names it; `docs/RUNS.md` (`RUNS.md§` in a
  `##` heading), `docs/GOLDEN.md`, a journal, another document, or
  `tools/gamelog/captures.txt` (a `run: N` stanza; `captures.txt(comment)`
  for a `#` line) does.
* **orphan** — nothing in the repo names it.

The candidate lists: **recreatable** (packets whose run no Rust code
pins — the item that took them has landed), **orphans** and
**ledger-only** (packets excepted, which are under recreatable), each
with its total. Notes: `compressible-unsupported` where one file is over
2 GiB, `raw` where the allocation is at least 90 % of the apparent size
(over 1 MiB; a capture lands raw and the steering pass compresses it).
"""
import argparse
import importlib.util
import json
import os
import re
import sys
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
MiB = 1 << 20
TWO_GIB = 2 << 30

_spec = importlib.util.spec_from_file_location('census', os.path.join(HERE, 'census.py'))
census_mod = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(census_mod)

TOKEN = re.compile(r'[A-Za-z0-9_][A-Za-z0-9_.\-]*')
WHOLE = re.compile(r'[A-Za-z0-9_][A-Za-z0-9_.\-]*\Z')
TAG = re.compile(r'(?<![A-Za-z0-9])run\d+[a-z]*(?![A-Za-z0-9])')
STANZA_RUN = re.compile(r'^\s*run:\s*(\d+[a-z]*)\s*$', re.M)
RUST_SPAN = re.compile(
    r'(?P<line>//[^\n]*)'
    r'|(?P<block>/\*.*?\*/)'
    r'|(?P<raw>(?<![A-Za-z0-9_])b?r(?P<h>#*)".*?"(?P=h))'
    r'|(?P<str>b?"(?:\\.|[^"\\])*")'
    r"|(?P<chr>'(?:\\.|[^\\'\n])')", re.S)
PACKET = re.compile(r'frame-snapshot.*\.bin\Z')
FAILED = re.compile(r'dead|take\d|failed|stalled|truncated')


# ---- the repo's text ---------------------------------------------------------

def split_rust(text):
    """(code, comments): a Rust source's comments apart from the rest; a
    string literal is code, `//` inside one is not a comment."""
    code, comments, at = [], [], 0
    for m in RUST_SPAN.finditer(text):
        if m.lastgroup in ('line', 'block'):
            code.append(text[at:m.start()])
            comments.append(m.group())
            at = m.end()
    code.append(text[at:])
    return '\n'.join(code), '\n'.join(comments)


def names_in(text):
    """Every whole name the text holds: maximal `[A-Za-z0-9_.-]` runs with a
    sentence's trailing punctuation off, and every run tag inside them."""
    out = {t.rstrip('.-_') for t in TOKEN.findall(text)}
    out.update(TAG.findall(text))
    out.discard('')
    return out


class Refs:
    """Name → who names it, three ways: Rust code, Rust comments, the ledgers."""

    def __init__(self):
        self.index = {'code': defaultdict(set), 'comment': defaultdict(set),
                      'ledger': defaultdict(set)}
        self.texts = {'code': [], 'comment': [], 'ledger': []}

    def add(self, way, label, text):
        for name in names_in(text):
            self.index[way][name].add(label)
        self.texts[way].append((label, text))

    def _look(self, way, name):
        if WHOLE.match(name):
            return set(self.index[way].get(name, ()))
        return {label for label, text in self.texts[way] if name in text}

    def rust_code(self, name):
        return self._look('code', name)

    def rust_comment(self, name):
        return self._look('comment', name)

    def ledger(self, name):
        return self._look('ledger', name)


def scan_refs(rust, ledgers):
    """`rust`: {path: source}; `ledgers`: {label: text}. A label ending in
    `RUNS.md` splits its `#` headings (`RUNS.md§`) from its body; one ending
    in `captures.txt` splits `#` comments (`captures.txt(comment)`) from the
    stanzas and reads each `run: N` as `runN`."""
    refs = Refs()
    for path, text in rust.items():
        code, comments = split_rust(text)
        refs.add('code', path, code)
        refs.add('comment', path, comments)
    for label, text in ledgers.items():
        if label.endswith('RUNS.md'):
            heads = [ln for ln in text.splitlines() if ln.startswith('#')]
            body = [ln for ln in text.splitlines() if not ln.startswith('#')]
            refs.add('ledger', label + '§', '\n'.join(heads))
            refs.add('ledger', label, '\n'.join(body))
        elif label.endswith('captures.txt'):
            lines = text.splitlines()
            comments = [ln for ln in lines if ln.lstrip().startswith('#')]
            stanzas = [ln for ln in lines if not ln.lstrip().startswith('#')]
            stanza_text = '\n'.join(stanzas)
            runs = ' '.join('run' + n for n in STANZA_RUN.findall(stanza_text))
            refs.add('ledger', label + '(comment)', '\n'.join(comments))
            refs.add('ledger', label, stanza_text + '\n' + runs)
        else:
            refs.add('ledger', label, text)
    return refs


def _read(path):
    with open(path, encoding='utf-8', errors='replace') as f:
        return f.read()


def load_refs(repo=REPO):
    """The live tree's texts: every `crates/*/src/**/*.rs`; `docs/RUNS.md`,
    `docs/GOLDEN.md`, the journals (one label), every other `docs/**/*.md`,
    and `tools/gamelog/captures.txt`."""
    rust, ledgers = {}, {}
    crates = os.path.join(repo, 'crates')
    for dirpath, _, files in os.walk(crates):
        if os.sep + 'src' not in dirpath[len(crates):] + os.sep:
            continue
        for name in files:
            if name.endswith('.rs'):
                path = os.path.join(dirpath, name)
                rust[os.path.relpath(path, repo)] = _read(path)
    docs = os.path.join(repo, 'docs')
    journal = []
    for dirpath, _, files in os.walk(docs):
        for name in sorted(files):
            if not name.endswith('.md'):
                continue
            path = os.path.join(dirpath, name)
            rel = os.path.relpath(path, docs)
            if rel in ('RUNS.md', 'GOLDEN.md'):
                ledgers[rel] = _read(path)
            elif rel.startswith('journal' + os.sep):
                journal.append(_read(path))
            else:
                ledgers['docs/' + rel] = _read(path)
    if journal:
        ledgers['journal'] = '\n'.join(journal)
    captures = os.path.join(repo, 'tools', 'gamelog', 'captures.txt')
    if os.path.isfile(captures):
        ledgers['captures.txt'] = _read(captures)
    return scan_refs(rust, ledgers)


# ---- sizes -------------------------------------------------------------------

def _lstat(path):
    st = os.lstat(path)
    return st.st_size, st.st_blocks


def sum_sizes(paths, stat=_lstat):
    """Apparent and allocated bytes over `paths`, and the largest one file."""
    total = {'apparent': 0, 'alloc': 0, 'largest': 0}
    for p in paths:
        size, blocks = stat(p)
        total['apparent'] += size
        total['alloc'] += blocks * 512
        total['largest'] = max(total['largest'], size)
    return total


def walk(root, skip=()):
    """Every file and symlink under `root`, never through a symlink, never
    into a directory in `skip`."""
    if not os.path.isdir(root) or os.path.islink(root):
        yield root
        return
    stack = [root]
    while stack:
        d = stack.pop()
        try:
            entries = list(os.scandir(d))
        except OSError:
            continue
        for e in entries:
            if e.is_dir(follow_symlinks=False):
                if os.path.realpath(e.path) not in skip:
                    stack.append(e.path)
            else:
                yield e.path


def notes_for(size):
    notes = []
    if size['largest'] > TWO_GIB:
        notes.append('compressible-unsupported (a file > 2 GiB)')
    if size['apparent'] > MiB and size['alloc'] * 10 >= size['apparent'] * 9:
        notes.append('raw')
    return notes


# ---- the census ----------------------------------------------------------------

def _row(entry, kind, size, path, notes=()):
    return {'entry': entry, 'kind': kind, 'path': path, 'apparent': size['apparent'],
            'alloc': size['alloc'], 'largest': size['largest'], 'class': None,
            'pinned_by': [], 'comment_by': [], 'ledger': [],
            'notes': list(notes) + notes_for(size)}


def _classify(row, refs, names, tag=None):
    """`names` must be named whole; `tag` is a log's run tag, a weaker pin."""
    code, comment, ledger = set(), set(), set()
    for n in names:
        code |= refs.rust_code(n)
        comment |= refs.rust_comment(n)
        ledger |= refs.ledger(n)
    exact = bool(code)
    if tag:
        by_tag = refs.rust_code(tag)
        code |= by_tag
        comment |= refs.rust_comment(tag)
        ledger |= refs.ledger(tag)
        if by_tag and not exact:
            row['notes'].insert(0, 'by tag ' + tag)
    if exact and tag is not None:
        row['notes'].insert(0, 'exact')
    row['pinned_by'] = sorted(code)
    row['comment_by'] = sorted(comment)
    row['ledger'] = sorted(ledger)
    row['class'] = ('pinned' if code else 'rust-comment' if comment
                    else 'ledger-only' if ledger else 'orphan')


def _golden_kind(name):
    if re.match(r'run\d', name):
        return 'run'
    if re.match(r'ch\d', name):
        return 'chapter'
    return 'other'


def golden_rows(root, refs):
    rows = []
    if not os.path.isdir(root):
        return rows
    for name in sorted(os.listdir(root)):
        path = os.path.join(root, name)
        files = list(walk(path))
        packets = [p for p in files if PACKET.match(os.path.basename(p))]
        rest = [p for p in files if p not in packets]
        notes = ['failed take?'] if FAILED.search(name) else []
        row = _row(name, _golden_kind(name), sum_sizes(rest), path, notes)
        _classify(row, refs, [name])
        rows.append(row)
        for p in packets:
            prow = _row(os.path.relpath(p, root), 'packet', sum_sizes([p]), p,
                        ['packet of ' + name])
            for key in ('class', 'pinned_by', 'comment_by', 'ledger'):
                prow[key] = row[key]
            rows.append(prow)
    return rows


def log_rows(logs, refs):
    rows = []
    if not os.path.isdir(logs):
        return rows
    for name in sorted(os.listdir(logs)):
        path = os.path.join(logs, name)
        entry = 'logs/' + name
        if os.path.isdir(path) and not os.path.islink(path):
            row = _row(entry, 'other', sum_sizes(walk(path)), path, ['a directory in the logs'])
            _classify(row, refs, [name])
            rows.append(row)
            continue
        base, notes = name, []
        if '.ronidx' in name:
            base = name[:name.index('.ronidx')]
            notes.append('index of ' + base + ' (regenerable)')
        row = _row(entry, 'log', sum_sizes([path]), path, notes)
        m = TAG.search(base)
        _classify(row, refs, [base], m.group() if m else '')
        rows.append(row)
    return rows


def data_rows(data, logs, skip_names=()):
    rows = []
    if not os.path.isdir(data):
        return rows
    logs_real = os.path.realpath(logs) if logs else None
    for name in sorted(os.listdir(data)):
        path = os.path.join(data, name)
        if name in skip_names or os.path.realpath(path) == logs_real:
            continue
        notes = []
        skip = ()
        if logs_real and logs_real.startswith(os.path.realpath(path) + os.sep):
            skip = (logs_real,)
            notes.append('excluding the logs directory')
        row = _row('data/' + name, 'other', sum_sizes(walk(path, skip)), path, notes)
        row['class'] = 'unscanned'
        rows.append(row)
    return rows


def census(root, logs, data, refs):
    """Every row, sorted by allocated size, largest first."""
    root, logs, data = str(root), str(logs), str(data)
    skip = ()
    if os.path.realpath(root).startswith(os.path.realpath(data) + os.sep):
        skip = (os.path.basename(root),)
    rows = golden_rows(root, refs) + log_rows(logs, refs) + data_rows(data, logs, skip)
    rows.sort(key=lambda r: (-r['alloc'], -r['apparent'], r['entry']))
    return rows


# ---- the summary ---------------------------------------------------------------

def _total(rows):
    return {'count': len(rows), 'alloc': sum(r['alloc'] for r in rows),
            'apparent': sum(r['apparent'] for r in rows)}


def summarize(rows):
    by_kind, by_class = defaultdict(list), defaultdict(list)
    for r in rows:
        by_kind[r['kind']].append(r)
        by_class[r['class']].append(r)
    lists = {
        'recreatable': [r for r in rows if r['kind'] == 'packet' and r['class'] != 'pinned'],
        'orphans': [r for r in rows if r['kind'] != 'packet' and r['class'] == 'orphan'],
        'ledger-only': [r for r in rows if r['kind'] != 'packet' and r['class'] == 'ledger-only'],
    }
    candidates = {}
    for name, entries in lists.items():
        c = _total(entries)
        c['entries'] = entries
        candidates[name] = c
    return {'total': _total(rows),
            'by_kind': {k: _total(v) for k, v in sorted(by_kind.items())},
            'by_class': {k: _total(v) for k, v in sorted(by_class.items())},
            'candidates': candidates}


def mib(n):
    return '%.1f' % (n / MiB)


def _short(path):
    for prefix in ('crates/rondata/src/', 'crates/'):
        if path.startswith(prefix):
            return path[len(prefix):]
    return path


def referenced(row):
    c = row['class']
    if c == 'pinned':
        files = [_short(p) for p in row['pinned_by']]
        more = ' +%d' % (len(files) - 2) if len(files) > 2 else ''
        return 'pinned:' + ', '.join(files[:2]) + more
    if c == 'rust-comment':
        return 'rust-comment:' + _short(row['comment_by'][0])
    if c == 'ledger-only':
        more = ' +%d' % (len(row['ledger']) - 3) if len(row['ledger']) > 3 else ''
        return 'ledger-only (' + ', '.join(row['ledger'][:3]) + more + ')'
    return c


def print_report(rows, summary, top, roots):
    for label, path in roots:
        print('# %s: %s' % (label, path))
    print()
    print('| entry | kind | alloc MiB | apparent MiB | referenced by | notes |')
    print('|---|---|--:|--:|---|---|')
    for r in rows[:top] if top else rows:
        print('| %s | %s | %s | %s | %s | %s |' % (r['entry'], r['kind'], mib(r['alloc']),
                                                  mib(r['apparent']), referenced(r),
                                                  '; '.join(r['notes'])))
    t = summary['total']
    print()
    print('## Totals: %d entries, %s MiB allocated, %s MiB apparent'
          % (t['count'], mib(t['alloc']), mib(t['apparent'])))
    for title, key in (('by kind', 'by_kind'), ('by reference class', 'by_class')):
        print()
        print('## Totals %s' % title)
        for k, v in summary[key].items():
            print('  %-14s %6d entries  %12s MiB alloc  %12s MiB apparent'
                  % (k, v['count'], mib(v['alloc']), mib(v['apparent'])))
    heads = {
        'recreatable': 'Candidates, recreatable: packets no Rust code pins',
        'orphans': 'Candidates, orphans: named by nothing in the repo',
        'ledger-only': 'Candidates, ledger-only: named by RUNS/GOLDEN/journals/captures.txt, no Rust',
    }
    for name, head in heads.items():
        c = summary['candidates'][name]
        print()
        print('## %s — %d entries, %s MiB allocated, %s MiB apparent'
              % (head, c['count'], mib(c['alloc']), mib(c['apparent'])))
        for r in c['entries']:
            extra = '' if name == 'orphans' else '  ' + referenced(r)
            print('  %10s  %s%s' % (mib(r['alloc']), r['entry'], extra))


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('--root', default=os.environ.get(
        'RON_GOLDEN_DIR', os.path.expanduser('~/ron-golden')),
        help='the golden archive (default $RON_GOLDEN_DIR or ~/ron-golden)')
    ap.add_argument('--logs', default=census_mod.ARCHIVE,
                    help="the queue lane's logs (tools/census.py's ARCHIVE: $RON_GAMELOG_DIR)")
    ap.add_argument('--data', default=os.path.expanduser('~/ron-data'),
                    help='the other top-level directories, by size only (default ~/ron-data)')
    ap.add_argument('--repo', default=REPO, help='the checkout whose text says who reads what')
    ap.add_argument('--top', type=int, default=0, help='rows of the table (default all)')
    ap.add_argument('--json', action='store_true', help='everything, as JSON')
    a = ap.parse_args(argv)
    for label, path in (('--root', a.root), ('--logs', a.logs), ('--data', a.data)):
        if not os.path.isdir(path):
            print('archive_census: %s %s is not a directory; its rows are empty' % (label, path),
                  file=sys.stderr)
    refs = load_refs(a.repo)
    rows = census(a.root, a.logs, a.data, refs)
    summary = summarize(rows)
    if a.json:
        json.dump({'roots': {'root': a.root, 'logs': a.logs, 'data': a.data, 'repo': a.repo},
                   'entries': rows, 'summary': summary}, sys.stdout, indent=1)
        print()
        return
    print_report(rows, summary, a.top,
                 (('root', a.root), ('logs', a.logs), ('data', a.data), ('repo', a.repo)))


if __name__ == '__main__':
    main()
