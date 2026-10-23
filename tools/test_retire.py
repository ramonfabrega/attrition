"""`tools/retire.py` — what nothing in the tree names (Loop 3486, the
fifty-first pass).

A steering pass's measurement, never a deletion: every script under
`tools/` and every top-level `## ` section of a `docs/*.md` file, marked
**unnamed** when no other tracked text file names it. These build a small
git tree in a temporary directory and pin the rules on it:

(a) a script is named by its basename in another file — never by a longer
    name that merely ends in it (`test_unused.py`), never by itself or its
    own `test_<stem>.py` — or, for a `.py`, by an `import` of its stem;
(b) a section is named by its heading text in another file, or by its
    number cited with the document's name (`ALPHA §1`, `` `docs/ALPHA.md` §1.2 ``,
    the spelling `crates/sim/src/docs_guard.rs` reads; a decision by
    `DECISIONS 34` or `entry 34`, the spelling the repo cites it by); the
    document's own cites are self-references, and a dated heading is no
    number;
(c) a journal older than `--since-days` names nothing, a recent one does;
(d) the table puts the unnamed rows first, largest first, and its last line
    is the summary; `--json` carries the same rows.
"""
import contextlib
import datetime
import io
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import retire  # noqa: E402

TODAY = datetime.date(2026, 10, 22)

FILES = {
    'tools/used.py': 'import lib_mod\nprint("used")\n',
    'tools/lib_mod.py': 'X = 1\n',
    'tools/unused.py': '# nobody names me\n' + 'x = 1\n' * 40,
    'tools/test_unused.py': '"""tests tools/unused.py"""\n',
    'tools/explore/oldonly.sh': 'echo old\n',
    'tools/explore/recent.swift': 'print("recent")\n',
    'tools/README.md': 'Run `python3 tools/used.py`.\n',
    'docs/ALPHA.md': ('# ALPHA\n\n## 1. Alpha section\n\nAlpha body; see ALPHA §2 and §2.\n\n'
                  '## 2. Beta section\n\n' + 'beta body\n' * 20 + '\n## Lonely heading\n\nalone\n\n## 5.1–5.3 A range\n\nr\n'),
    'docs/BRAVO.md': ('# BRAVO\n\n## Gamma heading words\n\nThe alpha rule is `docs/ALPHA.md` §1.2.\n\n'
                  '## 3. Delta\n\nd\n\n## 2026-10-01 — a dated entry\n\ne\n'),
    'docs/DECISIONS.md': '# D\n\n## 1. First\n\n## 2. Second\n\n## 3. Third\n\n## 4. Fourth\n\nSee DECISIONS 4.\n',
    'docs/ORDERS.md': '# O\n\nPer DECISIONS 1, and `docs/DECISIONS.md` entries 2 and 3.\n',
    'docs/CHARLIE.md': '# CHARLIE\n\nRead "Gamma heading words" in BRAVO, and BRAVO §3, and ALPHA §5.2.\n',
    'docs/journal/2020-01-01-item-1.md': 'Built tools/explore/oldonly.sh and ALPHA §2.\n',
    'docs/journal/2026-10-20-item-2.md': 'Built tools/explore/recent.swift.\n',
}


def build(tmp):
    root = Path(tmp)
    for rel, text in FILES.items():
        p = root / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)
    git = ['git', '-c', 'user.name=t', '-c', 'user.email=t@example.invalid', '-c', 'commit.gpgsign=false']
    subprocess.run(['git', 'init', '-q'], cwd=root, check=True)
    subprocess.run([*git, 'add', '-A'], cwd=root, check=True)
    subprocess.run([*git, 'commit', '-q', '-m', 'fixture'], cwd=root, check=True,
                   env={'GIT_COMMITTER_DATE': '2026-10-01T12:00:00+00:00',
                        'GIT_AUTHOR_DATE': '2026-10-01T12:00:00+00:00', 'PATH': '/usr/bin:/bin:/opt/homebrew/bin'})
    return root


class Retire(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = build(self.tmp.name)
        self.rows = retire.measure(self.root, since_days=30, today=TODAY)
        self.by = {r['name']: r for r in self.rows}

    def tearDown(self):
        self.tmp.cleanup()

    def test_scripts_are_named_by_basename_or_import_and_never_by_themselves(self):
        scripts = {r['name']: r['unnamed'] for r in self.rows if r['kind'] == 'script'}
        self.assertEqual(scripts, {
            'tools/used.py': False,             # tools/README.md names it
            'tools/lib_mod.py': False,          # `import lib_mod` in used.py
            'tools/unused.py': True,            # only its own test names it
            'tools/explore/oldonly.sh': True,   # only an old journal names it
            'tools/explore/recent.swift': False,  # a recent journal names it
        })
        self.assertNotIn('tools/test_unused.py', scripts)
        self.assertEqual(self.by['tools/unused.py']['date'], '2026-10-01')
        self.assertEqual(self.by['tools/unused.py']['bytes'], len(FILES['tools/unused.py']))

    def test_sections_are_named_by_heading_or_by_a_cited_number(self):
        sections = {r['name']: r['unnamed'] for r in self.rows if r['kind'] == 'section'}
        self.assertEqual(sections, {
            'ALPHA §1 Alpha section': False,    # `docs/ALPHA.md` §1.2 in BRAVO
            'ALPHA §2 Beta section': True,      # ALPHA's own cite and an old journal's
            'ALPHA § Lonely heading': True,
            'ALPHA §5 A range': False,          # a range heading is its first number's
            'BRAVO § Gamma heading words': False,  # its heading text in CHARLIE
            'BRAVO §3 Delta': False,            # BRAVO §3 in CHARLIE
            # A dated heading is not a number (the journal's `## 2026-10-01 — …`).
            'BRAVO § 2026-10-01 — a dated entry': True,
            # DECISIONS is cited by entry: `DECISIONS 1`, `entries 2 and 3`.
            'DECISIONS §1 First': False,
            'DECISIONS §2 Second': False,
            'DECISIONS §3 Third': False,
            'DECISIONS §4 Fourth': True,       # only its own text cites it
        })
        beta = self.by['ALPHA §2 Beta section']
        self.assertEqual(beta['path'], 'docs/ALPHA.md')
        self.assertEqual(beta['bytes'], len(('## 2. Beta section\n\n' + 'beta body\n' * 20 + '\n').encode()))

    def test_the_since_days_window_lets_an_old_journal_name_again(self):
        rows = retire.measure(self.root, since_days=10000, today=TODAY)
        by = {r['name']: r['unnamed'] for r in rows}
        self.assertFalse(by['tools/explore/oldonly.sh'])
        self.assertFalse(by['ALPHA §2 Beta section'])
        rows = retire.measure(self.root, since_days=1, today=TODAY)
        self.assertTrue({r['name']: r['unnamed'] for r in rows}['tools/explore/recent.swift'])

    def test_the_table_puts_unnamed_first_largest_first_and_ends_on_the_summary(self):
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            self.assertEqual(retire.main(['--root', str(self.root), '--today', TODAY.isoformat()]), 0)
        lines = out.getvalue().rstrip('\n').split('\n')
        self.assertEqual(lines[-1], 'retire: 2 of 5 scripts and 4 of 11 sections unnamed')
        body = [l for l in lines[1:-1] if l.strip()]
        flags = [l.split()[0] for l in body]
        self.assertEqual(flags, ['unnamed'] * 6 + ['named'] * 10)
        sizes = [int(l.split()[3]) for l in body[:6]]
        self.assertEqual(sizes, sorted(sizes, reverse=True))
        self.assertIn('tools/unused.py', body[0] + body[1])
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            retire.main(['--root', str(self.root), '--today', TODAY.isoformat(), '--json'])
        data = json.loads(out.getvalue())
        self.assertEqual(data['summary'], {'scripts': 5, 'scripts_unnamed': 2,
                                           'sections': 11, 'sections_unnamed': 4})
        self.assertEqual([r['unnamed'] for r in data['rows']][:6], [True] * 6)


if __name__ == '__main__':
    unittest.main()
