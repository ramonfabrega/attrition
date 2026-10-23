#!/usr/bin/env python3
"""retire.py — what nothing in the tree names: the scripts under `tools/`
and the top-level sections of `docs/*.md` (Loop 3486, the fifty-first pass).

    python3 tools/retire.py                # the table, unnamed rows first
    python3 tools/retire.py --json         # the same rows as JSON
    python3 tools/retire.py --since-days 60
    python3 tools/retire.py --no-ledgers   # without RUNS, PARKED, JOURNAL, DECISIONS, docs/audit/
    python3 tools/retire.py --doc GOLDEN --doc QUEUE   # only these documents' sections

`--no-ledgers` and `--doc` (the fifty-second pass) choose which documents'
section rows are listed, never which files name: a ledger's text still
names a tool or a heading. The ledgers' rows are cited by number
(`run1148`, `parked 3505a`, `item 3511`), not by heading, so on the whole
tree they were 1939 of 3056 sections read unnamed. The summary line then
says what was left out. `docs/audit/` is in the ledger list for the flag's
promise, though its files are never listed (only `docs/*.md` is).

A steering pass's measurement, and it deletes nothing. It lists

  (a) every script under `tools/` — `*.py`, `*.sh`, `*.swift`, `*.c`,
      `*.h` — but no `test_*.py` and no `__init__.py`; and
  (b) every top-level `## ` section of each `docs/*.md` (fenced code is not
      a heading),

and marks a row **unnamed** when no other git-tracked text file names it:

  * a script by its basename as a whole token (`tools/explore/poolshot.py`
    names `poolshot.py`; `test_poolshot.py` does not), or, for a `.py`, by an
    `import` of its stem. The script itself and its own `test_<stem>.py` are
    self-references and name nothing;
  * a section by its heading text, verbatim, or by its number cited with the
    document's name — `GOLDEN §13`, `` `docs/GOLDEN.md` §13.2 `` — the
    spelling `crates/sim/src/docs_guard.rs`'s `section_cites` reads (the
    name is the run of capitals right before the sign, two or more, after
    whitespace, backticks and `.md`; a subsection cites its top-level
    section). A decision is also cited by entry, the spelling the repo
    uses for it: `DECISIONS 34`, `` `docs/DECISIONS.md` entry 34 ``,
    `entries 34, 55 and 61` (any document's `entry N`). A dated heading
    (`## 2026-10-01 — …`) is not a number. The document's own text names
    none of its sections.

A journal (`docs/journal/*.md`) older than `--since-days` (default 30; its
date is the filename's `YYYY-MM-DD`, else its last commit) names nothing, so
a tool only old journals name counts as unnamed.

Each row carries the last commit date touching it — a script's from one
`git log` pass, a section's the newest line `git blame` gives it — and its
bytes. Unnamed rows come first, largest first; the last line is
`retire: <A> of <B> scripts and <C> of <D> sections unnamed`.

What it does not establish: a name is a substring, so a section whose heading
is a common phrase reads as named, and two scripts sharing a basename name
each other's references; a tool run only by hand from shell history reads as
unnamed. It is the list a pass reads before it retires, never the retirement.
Tests: `tools/test_retire.py`.
"""
import argparse
import datetime
import json
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT_EXTS = ('.py', '.sh', '.swift', '.c', '.h')
TOKEN = re.compile(r'[\w+-][\w.+-]*')
IMPORT = re.compile(r'^[ \t]*(?:from[ \t]+([\w.]+)[ \t]+import[ \t]+\(?([\w., \t]+)|import[ \t]+([\w., \t]+))', re.M)
CITE = re.compile(r'([A-Z_]{2,})(?:\.md)?`*\s*§\s*(\d+)')
# A decision is cited by entry — `DECISIONS 34`, `` `docs/DECISIONS.md` entry
# 34 ``, `entries 34, 55 and 61` — and never with the sign.
ENTRY = re.compile(r'([A-Z_]{2,})(?:\.md)?`*\s+entr(?:y|ies)\s+(\d+(?:(?:\s*,\s*|\s+and\s+|\s*&\s*)\d+)*)')
DECISION = re.compile(r'DECISIONS(?:\.md)?`*\s+(\d+(?:(?:\s*,\s*|\s+and\s+|\s*&\s*)\d+)*)')
# A section number ends at a dot or a space: `## 2026-10-01 — …` is a date,
# and `## 4.6–4.8 …` (an en dash) is section 4's range.
NUMBER = re.compile(r'(\d+)(?:\.\d+)*(?:–\d+(?:\.\d+)*)?\.?(?=\s)\s*')
JOURNAL_DATE = re.compile(r'^(\d{4}-\d{2}-\d{2})')
WORD = re.compile(r'\w+')


def git(root, *args):
    return subprocess.run(['git', *args], cwd=root, capture_output=True, text=True, check=True).stdout


def tracked(root):
    return [p for p in git(root, 'ls-files', '-z').split('\0') if p]


def read_texts(root, paths):
    """`{path: text}` for every tracked file that reads as text (no NUL in
    its first 8 KiB)."""
    out = {}
    for rel in paths:
        try:
            data = (root / rel).read_bytes()
        except OSError:
            continue
        if b'\0' in data[:8192]:
            continue
        out[rel] = data.decode('utf-8', 'replace')
    return out


def is_script(rel):
    if not rel.startswith('tools/') or '__pycache__' in rel.split('/'):
        return False
    name = rel.rsplit('/', 1)[-1]
    if not name.endswith(SCRIPT_EXTS):
        return False
    return not (name.startswith('test_') and name.endswith('.py')) and name != '__init__.py'


def old_journals(root, texts, since_days, today):
    """The journals older than `since_days`, which name nothing."""
    old = set()
    for rel in texts:
        if not (rel.startswith('docs/journal/') and rel.endswith('.md')):
            continue
        m = JOURNAL_DATE.match(rel.rsplit('/', 1)[-1])
        if m:
            day = datetime.date.fromisoformat(m.group(1))
        else:
            stamp = git(root, 'log', '-1', '--format=%cI', '--', rel).strip()
            if not stamp:
                continue
            day = datetime.date.fromisoformat(stamp[:10])
        if (today - day).days > since_days:
            old.add(rel)
    return old


def path_dates(root):
    """`{path: YYYY-MM-DD}`, the last commit touching each path under
    tools/ and docs/, from one `git log` pass."""
    out = {}
    for block in git(root, 'log', '--format=%x00%cs', '--name-only', '--', 'tools', 'docs').split('\0'):
        lines = block.strip('\n').split('\n')
        if not lines or not lines[0]:
            continue
        day = lines[0].strip()
        for p in lines[1:]:
            p = p.strip()
            if p and p not in out:
                out[p] = day
    return out


# --- the sections ----------------------------------------------------------

def sections(text):
    """`[(number or None, heading, first line, last line, bytes)]` for every
    top-level `## ` heading outside a code fence; a section runs to the next
    one or the end."""
    lines = text.splitlines(keepends=True)
    heads, fence = [], False
    for i, line in enumerate(lines):
        if line.lstrip().startswith(('```', '~~~')):
            fence = not fence
            continue
        if not fence and line.startswith('## '):
            heads.append(i)
    out = []
    for k, start in enumerate(heads):
        end = heads[k + 1] if k + 1 < len(heads) else len(lines)
        body = lines[start][3:].strip().strip('~*§ ').lstrip('~*§ ')
        m = NUMBER.match(body)
        number = m.group(1) if m else None
        heading = (body[m.end():] if m else body).strip().strip('~*').strip()
        size = sum(len(l.encode('utf-8')) for l in lines[start:end])
        out.append((number, heading, start, end, size))
    return out


def blame_dates(root, rel):
    """`[YYYY-MM-DD per line]` of `rel`, the committer's own date."""
    try:
        text = git(root, 'blame', '--line-porcelain', '--', rel)
    except subprocess.CalledProcessError:
        return []
    out, when, tz = [], 0, '+0000'
    for line in text.split('\n'):
        if line.startswith('committer-time '):
            when = int(line.split()[1])
        elif line.startswith('committer-tz '):
            tz = line.split()[1]
        elif line.startswith('\t'):
            sign = -1 if tz.startswith('-') else 1
            offset = datetime.timedelta(hours=int(tz[1:3]), minutes=int(tz[3:5])) * sign
            out.append((datetime.datetime.fromtimestamp(when, datetime.timezone(offset))).date().isoformat())
    return out


# --- the measurement -------------------------------------------------------

LEDGERS = ('DECISIONS', 'JOURNAL', 'PARKED', 'RUNS')
LEDGER_DIRS = ('docs/audit/',)


def doc_path(name):
    """`ALPHA`, `ALPHA.md` and `docs/ALPHA.md` all name `docs/ALPHA.md`."""
    name = name.strip()
    if name.startswith('docs/'):
        name = name[len('docs/'):]
    if not name.endswith('.md'):
        name += '.md'
    return f'docs/{name}'


def is_ledger(rel):
    return rel.startswith(LEDGER_DIRS) or rel in {f'docs/{name}.md' for name in LEDGERS}


def choose(found, only_docs=None, no_ledgers=False, left_out=None):
    """The `(rel, sections)` pairs whose section rows are listed. The
    ledgers' rows are cited by number, never by heading, so `no_ledgers`
    drops them; `only_docs` keeps the documents it names (a name the tree
    does not hold raises ValueError). Only the rows are chosen: every
    tracked file still names. `left_out`, when given, is filled with what
    was left out, for the summary line."""
    kept = list(found)
    if no_ledgers:
        dropped = [(rel, secs) for rel, secs in kept if is_ledger(rel)]
        kept = [(rel, secs) for rel, secs in kept if not is_ledger(rel)]
        if left_out is not None:
            left_out['excluded'] = [*LEDGERS, *LEDGER_DIRS]
            left_out['excluded_sections'] = sum(len(secs) for _, secs in dropped)
    if only_docs:
        wanted = [doc_path(name) for name in only_docs]
        known = {rel for rel, _ in found}
        unknown = [name for name, rel in zip(only_docs, wanted) if rel not in known]
        if unknown:
            raise ValueError(f"no such document: {', '.join(unknown)}")
        others = [(rel, secs) for rel, secs in kept if rel not in wanted and secs]
        kept = [(rel, secs) for rel, secs in kept if rel in wanted]
        if left_out is not None:
            left_out['only'] = sorted({rel[len('docs/'):-len('.md')] for rel in wanted})
            left_out['only_left_out_docs'] = len(others)
            left_out['only_left_out_sections'] = sum(len(secs) for _, secs in others)
    return kept


def measure(root=ROOT, since_days=30, today=None, blame=True, only_docs=None, no_ledgers=False,
            left_out=None):
    root = Path(root)
    today = today or datetime.date.today()
    paths = tracked(root)
    texts = read_texts(root, paths)
    silent = old_journals(root, texts, since_days, today)
    namers = {rel: text for rel, text in texts.items() if rel not in silent}
    scripts = sorted(p for p in paths if is_script(p))
    docs = sorted(p for p in paths if re.fullmatch(r'docs/[^/]+\.md', p))
    dates = path_dates(root)

    # One pass over every namer: its whole tokens, its imports, its cites.
    tokens, imports, cites = {}, {}, {}
    for rel, text in namers.items():
        for tok in set(TOKEN.findall(text)):
            tok = tok.strip('.-')
            if tok.endswith(SCRIPT_EXTS):
                tokens.setdefault(tok, set()).add(rel)
        if rel.endswith('.py'):
            for m in IMPORT.finditer(text):
                for group in m.groups():
                    for part in re.split(r'[,\s]+', group or ''):
                        for name in part.split('.'):
                            if name and name != 'as':
                                imports.setdefault(name, set()).add(rel)
        found = set(CITE.findall(text))
        found |= {(doc, n) for doc, ns in ENTRY.findall(text) for n in re.findall(r'\d+', ns)}
        found |= {('DECISIONS', n) for ns in DECISION.findall(text) for n in re.findall(r'\d+', ns)}
        for doc, n in found:
            cites.setdefault((doc, n), set()).add(rel)

    rows = []
    for rel in scripts:
        name = rel.rsplit('/', 1)[-1]
        stem = name.rsplit('.', 1)[0]
        own = {rel}
        own |= {p for p in texts if p.rsplit('/', 1)[-1] == f'test_{stem}.py'}
        by = (tokens.get(name, set()) | (imports.get(stem, set()) if name.endswith('.py') else set())) - own
        rows.append({'kind': 'script', 'name': rel, 'path': rel, 'date': dates.get(rel, '-'),
                     'bytes': len(texts.get(rel, '').encode('utf-8')) if rel in texts else (root / rel).stat().st_size,
                     'unnamed': not by, 'named_by': sorted(by)[:3]})

    found = choose([(rel, sections(texts[rel])) for rel in docs if rel in texts], only_docs, no_ledgers,
                   left_out)
    words = {}
    for rel, secs in found:
        for _, heading, *_ in secs:
            for w in WORD.findall(heading):
                words.setdefault(w, set())
    for rel, text in namers.items():
        for w in set(WORD.findall(text)) & words.keys():
            words[w].add(rel)
    blames = {}
    if blame:
        with ThreadPoolExecutor(max_workers=8) as pool:
            blames = dict(zip([rel for rel, _ in found], pool.map(lambda r: blame_dates(root, r), [rel for rel, _ in found])))
    for rel, secs in found:
        doc = rel[len('docs/'):-len('.md')]
        for number, heading, start, end, size in secs:
            by = set()
            if number is not None:
                by |= cites.get((doc, number), set())
            if heading:
                keys = [w for w in WORD.findall(heading) if len(w) >= 3] or WORD.findall(heading)
                pool_ = (min((words.get(w, set()) for w in keys), key=len) if keys else set(namers))
                by |= {p for p in pool_ if p != rel and heading in namers[p]}
            by -= {rel}
            day = max(blames.get(rel, [])[start:end], default=None) or dates.get(rel, '-')
            rows.append({'kind': 'section', 'name': f'{doc} §{number or ""} {heading}',
                         'path': rel, 'number': number, 'heading': heading, 'date': day,
                         'bytes': size, 'unnamed': not by, 'named_by': sorted(by)[:3]})
    rows.sort(key=lambda r: (not r['unnamed'], -r['bytes'], r['name']))
    return rows


def summary(rows, left_out=None):
    scripts = [r for r in rows if r['kind'] == 'script']
    secs = [r for r in rows if r['kind'] == 'section']
    return {'scripts': len(scripts), 'scripts_unnamed': sum(r['unnamed'] for r in scripts),
            'sections': len(secs), 'sections_unnamed': sum(r['unnamed'] for r in secs), **(left_out or {})}


def summary_line(s):
    line = (f"retire: {s['scripts_unnamed']} of {s['scripts']} scripts and "
            f"{s['sections_unnamed']} of {s['sections']} sections unnamed")
    if 'excluded' in s:
        names = [n for n in s['excluded'] if n not in LEDGER_DIRS]
        dirs = [n for n in s['excluded'] if n in LEDGER_DIRS]
        line += (f"; excluded the ledgers {', '.join(names)} and {', '.join(dirs)} "
                 f"({s['excluded_sections']} sections)")
    if 'only' in s:
        line += (f"; only {', '.join(s['only'])} ({s['only_left_out_docs']} other documents, "
                 f"{s['only_left_out_sections']} sections, left out)")
    return line


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('--root', type=Path, default=ROOT, help='the git tree to read (default: this one)')
    ap.add_argument('--since-days', type=int, default=30,
                    help='a journal older than this names nothing (default 30)')
    ap.add_argument('--today', type=datetime.date.fromisoformat, help='the day ages are counted from (YYYY-MM-DD)')
    ap.add_argument('--no-blame', action='store_true', help="date a section by its document's last commit")
    ap.add_argument('--json', action='store_true', help='the rows and the summary as JSON')
    ap.add_argument('--doc', action='append', default=[], metavar='NAME',
                    help='list only this document\'s sections (`RUNS`, `RUNS.md` or `docs/RUNS.md`); repeatable')
    ap.add_argument('--no-ledgers', action='store_true',
                    help=f"leave out the ledgers' sections — {', '.join(LEDGERS)} and "
                         f"{', '.join(LEDGER_DIRS)} — whose rows are cited by number, never by heading")
    a = ap.parse_args(argv)
    left_out = {}
    try:
        rows = measure(a.root, a.since_days, a.today, blame=not a.no_blame, only_docs=a.doc,
                       no_ledgers=a.no_ledgers, left_out=left_out)
    except ValueError as e:
        print(f"retire: {e}", file=sys.stderr)
        return 2
    s = summary(rows, left_out)
    if a.json:
        json.dump({'rows': rows, 'summary': s}, sys.stdout, indent=1)
        sys.stdout.write('\n')
        return 0
    print(f"{'status':<8} {'kind':<8} {'date':<10} {'bytes':>8}  name")
    for r in rows:
        print(f"{'unnamed' if r['unnamed'] else 'named':<8} {r['kind']:<8} {r['date']:<10} {r['bytes']:>8}  {r['name']}")
    print(summary_line(s))
    return 0


if __name__ == '__main__':
    sys.exit(main())
