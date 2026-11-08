"""`tools/paths.py` — the simulation's source paths the tree cites (item 4008).

These build a small tree in a temporary directory and pin the rules on it:

(a) a citation resolves when every path it spells exists — a file, a
    directory, a glob that matches, every alternative of a brace list —
    and trailing punctuation is not part of it;
(b) `--check` reads `docs/`, `crates/` and `CLAUDE.md`, never the guard that
    holds the known list, and exits 1 on a citation that names nothing;
(c) `move` rewrites a file where it stands whole (`air.rs`, never inside
    `airbase.rs` or `air.rs-E`), a directory by its prefix, an alternative
    inside a brace list, and `tools/` text but never `tools/emu/`;
(d) after `git mv` and `move`, `--check` is clean again.
"""
import contextlib
import io
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import paths  # noqa: E402

FILES = {
    'crates/sim/src/lib.rs': '//! See crates/sim/src/air.rs.\n',
    'crates/sim/src/air.rs': 'fn a() {}\n',
    'crates/sim/src/airbase.rs': 'fn b() {}\n',
    'crates/sim/src/air.rs-E': 'backup\n',
    'crates/sim/src/sweep/one.rs': 'fn s() {}\n',
    'crates/sim/src/docs_guard.rs': 'const P: &str = "crates/sim/src/gone.rs";\n',
    'docs/AIR.md': (
        'The arm is `crates/sim/src/air.rs:12`, beside crates/sim/src/airbase.rs.\n'
        'Backup: crates/sim/src/air.rs-E. Sweeps: crates/sim/src/sweep/one.rs and\n'
        'crates/sim/src/sweep/o*.rs; both crates/sim/src/{air,airbase}.rs.\n'
        'The tree is crates/sim/src/.\n'
    ),
    'CLAUDE.md': 'Read crates/sim/src/air.rs.\n',
    'tools/probe.py': "SRC = 'crates/sim/src/air.rs'\n",
    'tools/emu/sweep_x.py': "SRC = 'crates/sim/src/air.rs'\n",
}


class PathsTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        for rel, text in FILES.items():
            path = self.root / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)

    def tearDown(self):
        self.tmp.cleanup()

    def check(self):
        out = io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(io.StringIO()):
            code = paths.main(['--root', str(self.root), '--check'])
        return code, out.getvalue().splitlines()

    def test_a_citation_and_its_punctuation(self):
        got = paths.citations('see crates/sim/src/air.rs, and crates/sim/src/air.rs:3.\n'
                              'crates/sim/src/{a,b}.rs crates/sim/src/ crates/sim/src/sweep/')
        self.assertEqual(got, [(1, 'crates/sim/src/air.rs'), (1, 'crates/sim/src/air.rs'),
                               (2, 'crates/sim/src/{a,b}.rs'), (2, 'crates/sim/src/sweep/')])
        self.assertEqual(paths.expand('x/{a,b}_{c,d}.rs'),
                         ['x/a_c.rs', 'x/a_d.rs', 'x/b_c.rs', 'x/b_d.rs'])

    def test_a_clean_tree_checks_clean_and_the_guard_is_not_read(self):
        self.assertEqual(self.check(), (0, []))

    def test_a_missing_file_a_dead_glob_and_a_dead_alternative_fail(self):
        (self.root / 'docs/MORE.md').write_text(
            'crates/sim/src/gone.rs and crates/sim/src/sweep/z*.rs and '
            'crates/sim/src/{air,gone}.rs\n')
        code, lines = self.check()
        self.assertEqual(code, 1)
        self.assertEqual(lines, [
            'docs/MORE.md:1: crates/sim/src/gone.rs',
            'docs/MORE.md:1: crates/sim/src/sweep/z*.rs',
            'docs/MORE.md:1: crates/sim/src/{air,gone}.rs',
        ])

    def test_a_move_takes_its_citations_along(self):
        (self.root / 'crates/sim/src/air').mkdir()
        (self.root / 'crates/sim/src/air.rs').rename(self.root / 'crates/sim/src/air/mod.rs')
        code, lines = self.check()
        self.assertEqual(code, 1)
        self.assertIn('docs/AIR.md:1: crates/sim/src/air.rs', lines)
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            paths.main(['--root', str(self.root), 'move',
                        'crates/sim/src/air.rs', 'crates/sim/src/air/mod.rs'])
        self.assertEqual(self.check(), (0, []))
        doc = (self.root / 'docs/AIR.md').read_text()
        self.assertIn('`crates/sim/src/air/mod.rs:12`', doc)
        self.assertIn('crates/sim/src/airbase.rs.', doc)
        self.assertIn('crates/sim/src/air.rs-E.', doc)
        self.assertIn('crates/sim/src/{air/mod,airbase}.rs', doc)
        self.assertIn('crates/sim/src/air/mod.rs', (self.root / 'CLAUDE.md').read_text())
        self.assertIn('crates/sim/src/air/mod.rs', (self.root / 'tools/probe.py').read_text())
        self.assertEqual((self.root / 'tools/emu/sweep_x.py').read_text(),
                         FILES['tools/emu/sweep_x.py'])

    def test_a_directory_moves_by_its_prefix(self):
        (self.root / 'crates/sim/tests/sweep').mkdir(parents=True)
        (self.root / 'crates/sim/src/sweep/one.rs').rename(self.root / 'crates/sim/tests/sweep/one.rs')
        (self.root / 'crates/sim/src/sweep').rmdir()
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            paths.main(['--root', str(self.root), 'move',
                        'crates/sim/src/sweep/', 'crates/sim/tests/sweep/'])
        doc = (self.root / 'docs/AIR.md').read_text()
        self.assertIn('crates/sim/tests/sweep/one.rs and', doc)
        self.assertIn('crates/sim/tests/sweep/o*.rs', doc)
        self.assertEqual(self.check(), (0, []))

    def test_a_brace_list_that_cannot_follow_is_reported(self):
        unmoved = []
        text, n = paths.rewrite('crates/sim/src/{air,airbase}.rs',
                                [('crates/sim/src/air.rs', 'crates/sim/src/air/x.txt')], unmoved)
        self.assertEqual((text, n), ('crates/sim/src/{air,airbase}.rs', 0))
        self.assertEqual(unmoved, ['crates/sim/src/{air,airbase}.rs'])

    def test_the_crate_s_tests_are_cited_and_checked(self):
        self.assertEqual(paths.citations('crates/sim/tests/sweep/one.rs crates/sim/tests/'),
                         [(1, 'crates/sim/tests/sweep/one.rs')])
        (self.root / 'docs/MORE.md').write_text('crates/sim/tests/sweep/gone.rs\n')
        self.assertEqual(self.check(), (1, ['docs/MORE.md:1: crates/sim/tests/sweep/gone.rs']))

    def test_a_brace_list_splits_where_its_alternatives_part(self):
        moves = [('crates/sim/src/sweep/a.rs', 'crates/sim/tests/sweep/a.rs'),
                 ('crates/sim/src/sweep/c.rs', 'crates/sim/tests/sweep/c.rs'),
                 ('crates/sim/src/sweep/d.rs', 'crates/sim/tests/sweep/d.rs')]
        text, n = paths.rewrite('see crates/sim/src/sweep/{a,mod,c}.rs.', moves)
        self.assertEqual((text, n), ('see crates/sim/tests/sweep/{a,c}.rs, crates/sim/src/sweep/mod.rs.', 1))
        text, n = paths.rewrite('crates/sim/src/sweep/{a,d}.rs', moves)
        self.assertEqual((text, n), ('crates/sim/tests/sweep/{a,d}.rs', 1))
        self.assertEqual(paths.rewrite('crates/sim/src/sweep/{d,b}_x.rs', moves)[0],
                         'crates/sim/src/sweep/{d,b}_x.rs')

    def test_a_file_does_not_move_to_a_directory(self):
        with self.assertRaises(SystemExit):
            paths.move(self.root, [('crates/sim/src/air.rs', 'crates/sim/src/air/')])


if __name__ == '__main__':
    unittest.main()
