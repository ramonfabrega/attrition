"""The capture archive's census (`tools/archive_census.py`).

The disk ran from 103 GiB free to 30 in one tranche of sixty-seven
captures, and pruning must be by a list nobody guesses at. These tests
author a small archive — run directories, chapters, a packet, the queue
lane's logs — and a small repo whose Rust, `docs/RUNS.md`, `docs/GOLDEN.md`,
journals and `tools/gamelog/captures.txt` say who reads what, and check the
census sorts every entry into pinned, rust-comment, ledger-only or orphan by
the repo's text alone. The tool deletes nothing; the last class checks the
tree is unchanged and that its source holds no call that could delete.
"""
import importlib.util
import io
import json
import os
import re
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('archive_census', ROOT / 'tools/archive_census.py')
ac = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ac)

MiB = 1 << 20

RUST = {
    # code: a pinned log by exact name, a tag in code, a chapter in code
    'crates/rondata/src/diff/battery.rs': '''
const BATTERY: &[(&str, &str)] = &[
    ("gamelog-run100-greatlakes-24k-trace.txt", "rontrace-run100.log"),
    ("gamelog-run500-exact.txt", "x"),
];
fn label() -> &'static str { "run600" }
fn golden_g() { golden("ch2"); let url = "https://example/"; }
''',
    # comments only, and the boundary case: run956 in code, run95 nowhere
    'crates/rondata/src/diff/boundary.rs': '''
// the short sibling is named nowhere; only this comment says run800.
/// A doc comment: `~/ron-golden/run400/map-14/frame-snapshot.bin`.
const W: &str = "rontrace-run956.log"; /* and run801 in a block */
''',
}
RUNS = '''# Runs

## run200 — the only ledger section
Body text naming run201 in passing.

## The early run (run300x, 2026-01-01)
'''
GOLDEN = '''## 5. Chapter seven
The chapter's capture is `ch7`; ch2g4 is the sibling.
'''
CAPTURES = '''# A comment naming run900 and run901.

run: 902
tag: greatlakes-something
'''
JOURNAL = 'The packet at run400 was read on item 9999 and the item landed.\n'


def write(path, size=0, data=None):
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, 'wb') as f:
        if data is not None:
            f.write(data)
        elif size:
            f.write(b'\0' * size)


def make_repo(root):
    for rel, text in RUST.items():
        (root / rel).parent.mkdir(parents=True, exist_ok=True)
        (root / rel).write_text(text)
    (root / 'docs/journal').mkdir(parents=True)
    (root / 'docs/RUNS.md').write_text(RUNS)
    (root / 'docs/GOLDEN.md').write_text(GOLDEN)
    (root / 'docs/journal/2026-10-19-item-9999.md').write_text(JOURNAL)
    (root / 'tools/gamelog').mkdir(parents=True)
    (root / 'tools/gamelog/captures.txt').write_text(CAPTURES)


def make_archive(base):
    golden, logs, data = base / 'golden', base / 'logs', base / 'data'
    for run in ('run100', 'run200', 'run300', 'run95', 'run956', 'run902'):
        write(golden / run / 'map-14' / 'gamelog.txt', 3 * 4096)
        write(golden / run / 'receipt.json', data=b'{}')
    write(golden / 'run400' / 'map-14' / 'gamelog.txt', 4096)
    write(golden / 'run400' / 'map-14' / 'frame-snapshot.bin', 8 * 4096)
    write(golden / 'run100' / 'map-14' / 'frame-snapshot.bin', 8 * 4096)
    write(golden / 'ch7' / 'map-14' / 'gamelog.txt', 4096)
    write(golden / 'ch2' / 'map-14' / 'gamelog.txt', 4096)
    write(golden / 'smoke' / 'x.txt', 10)
    outside = base / 'outside.bin'
    write(outside, 64 * 4096)
    os.symlink(outside, golden / 'run300' / 'map-14' / 'linked.bin')
    for name in ('gamelog-run500-exact.txt', 'gamelog-run600-foo.txt', 'rontrace-run700.log',
                 'gamelog-run800-commented.txt', 'gamelog-run900-x.txt', 'rontrace-run95.log',
                 'gamelog-run100-greatlakes-24k-trace.txt'):
        write(logs / name, 4096)
    write(logs / 'gamelog-run500-exact.txt.ronidx-v1-1-2-3', 100)
    write(logs / '.ron-oracle-v1' / 'cache.bin', 4096)
    write(data / 'recomp' / 'a' / 'b.bin', 2 * 4096)
    write(data / 'sweep-trial-1' / 'c.bin', 4096)
    return golden, logs, data


class Archive(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        base = Path(self.tmp.name)
        self.repo = base / 'repo'
        make_repo(self.repo)
        self.golden, self.logs, self.data = make_archive(base)
        self.refs = ac.load_refs(self.repo)
        self.rows = ac.census(self.golden, self.logs, self.data, self.refs)
        self.by = {r['entry']: r for r in self.rows}

    def tearDown(self):
        self.tmp.cleanup()

    def cls(self, entry):
        return self.by[entry]['class']


class Classes(Archive):
    def test_a_run_pinned_by_a_rust_fixture_is_pinned_and_names_the_file(self):
        row = self.by['run100']
        self.assertEqual(row['class'], 'pinned')
        self.assertIn('crates/rondata/src/diff/battery.rs', row['pinned_by'])

    def test_a_run_only_in_runs_md_is_ledger_only(self):
        self.assertEqual(self.cls('run200'), 'ledger-only')
        self.assertIn('RUNS.md§', self.by['run200']['ledger'])

    def test_a_run_read_by_nothing_is_an_orphan(self):
        self.assertEqual(self.cls('run300'), 'orphan')

    def test_run95_does_not_borrow_run956s_pin(self):
        self.assertEqual(self.cls('run956'), 'pinned')
        self.assertEqual(self.cls('run95'), 'orphan')
        self.assertEqual(self.cls('logs/rontrace-run95.log'), 'orphan')

    def test_run300x_in_a_heading_does_not_name_run300(self):
        self.assertNotIn('RUNS.md§', self.by['run300']['ledger'])

    def test_a_log_pinned_by_its_exact_name(self):
        row = self.by['logs/gamelog-run500-exact.txt']
        self.assertEqual(row['class'], 'pinned')
        self.assertIn('exact', '; '.join(row['notes']))

    def test_a_log_pinned_only_through_its_tag(self):
        row = self.by['logs/gamelog-run600-foo.txt']
        self.assertEqual(row['class'], 'pinned')
        self.assertIn('by tag run600', '; '.join(row['notes']))

    def test_a_sidecar_index_follows_its_log(self):
        row = self.by['logs/gamelog-run500-exact.txt.ronidx-v1-1-2-3']
        self.assertEqual(row['class'], 'pinned')
        self.assertIn('index of', '; '.join(row['notes']))

    def test_a_rust_comment_is_not_a_pin(self):
        self.assertEqual(self.cls('logs/gamelog-run800-commented.txt'), 'rust-comment')

    def test_a_captures_comment_is_a_mention_flagged_as_such(self):
        row = self.by['logs/gamelog-run900-x.txt']
        self.assertEqual(row['class'], 'ledger-only')
        self.assertIn('captures.txt(comment)', row['ledger'])

    def test_a_captures_stanza_names_its_run(self):
        self.assertEqual(self.cls('run902'), 'ledger-only')
        self.assertIn('captures.txt', self.by['run902']['ledger'])

    def test_a_chapter_is_read_by_golden_md_or_by_a_fixture(self):
        self.assertEqual(self.by['ch7']['kind'], 'chapter')
        self.assertEqual(self.cls('ch7'), 'ledger-only')
        self.assertIn('GOLDEN.md', self.by['ch7']['ledger'])
        self.assertEqual(self.cls('ch2'), 'pinned')

    def test_kinds(self):
        self.assertEqual(self.by['run100']['kind'], 'run')
        self.assertEqual(self.by['smoke']['kind'], 'other')
        self.assertEqual(self.by['logs/rontrace-run700.log']['kind'], 'log')
        self.assertEqual(self.by['run400/map-14/frame-snapshot.bin']['kind'], 'packet')
        self.assertEqual(self.by['data/recomp']['kind'], 'other')
        self.assertEqual(self.by['data/sweep-trial-1']['kind'], 'other')


class RustText(unittest.TestCase):
    def test_comments_split_from_code_and_strings_stay_code(self):
        code, comments = ac.split_rust('let a = "x // not a comment"; // run1\n/* run2 */ "run3"')
        self.assertIn('run3', code)
        self.assertIn('not a comment', code)
        self.assertIn('run1', comments)
        self.assertIn('run2', comments)
        self.assertNotIn('run1', code)

    def test_injected_text_needs_no_tree(self):
        refs = ac.scan_refs(rust={'a.rs': 'const X: &str = "gamelog-run7-a.txt";'},
                            ledgers={'RUNS.md': '## run8 — x\n'})
        self.assertEqual(refs.rust_code('run7'), {'a.rs'})
        self.assertEqual(refs.ledger('run8'), {'RUNS.md§'})
        self.assertEqual(refs.rust_code('run8'), set())


class Sizes(Archive):
    def test_a_packet_is_its_own_row_and_leaves_its_run(self):
        packet = self.by['run400/map-14/frame-snapshot.bin']
        self.assertEqual(packet['apparent'], 8 * 4096)
        self.assertEqual(self.by['run400']['apparent'], 4096)

    def test_a_symlink_is_not_followed(self):
        # gamelog.txt and receipt.json; the 256 KiB the link points at is not counted
        self.assertLess(self.by['run300']['apparent'], 4 * 4096 + 100)

    def test_a_data_directory_is_summed_whole(self):
        self.assertEqual(self.by['data/recomp']['apparent'], 2 * 4096)

    def test_allocated_and_apparent_are_summed_separately(self):
        sizes = {'a': (10, 1), 'b': (5 * MiB, 8), 'c': (3 * 1024 * MiB, 16)}
        total = ac.sum_sizes(sizes.keys(), stat=lambda p: sizes[p])
        self.assertEqual(total['apparent'], 10 + 5 * MiB + 3 * 1024 * MiB)
        self.assertEqual(total['alloc'], (1 + 8 + 16) * 512)
        self.assertEqual(total['largest'], 3 * 1024 * MiB)

    def test_a_sparse_file_allocates_less_than_it_claims(self):
        path = Path(self.tmp.name) / 'sparse.bin'
        with open(path, 'wb') as f:
            f.seek(16 * MiB)
            f.write(b'x')
        total = ac.sum_sizes([str(path)])
        self.assertEqual(total['apparent'], 16 * MiB + 1)
        if total['alloc'] >= total['apparent']:
            self.skipTest('this file system does not keep sparse files')
        self.assertLess(total['alloc'], MiB)

    def test_a_file_over_2_gib_is_flagged(self):
        self.assertIn('compressible-unsupported', '; '.join(ac.notes_for({'largest': 2 * 1024 * MiB + 1,
                                                                'apparent': 3 * 1024 * MiB,
                                                                'alloc': 3 * 1024 * MiB})))
        self.assertNotIn('compressible-unsupported', '; '.join(ac.notes_for({'largest': 10, 'apparent': 10,
                                                                    'alloc': 4096})))


class Summary(Archive):
    def test_the_table_is_sorted_by_allocated_size(self):
        allocs = [r['alloc'] for r in self.rows]
        self.assertEqual(allocs, sorted(allocs, reverse=True))

    def test_candidates(self):
        s = ac.summarize(self.rows)
        recreatable = {r['entry'] for r in s['candidates']['recreatable']['entries']}
        orphans = {r['entry'] for r in s['candidates']['orphans']['entries']}
        ledger = {r['entry'] for r in s['candidates']['ledger-only']['entries']}
        # run400's only Rust mention is a doc comment: its item landed
        self.assertEqual(recreatable, {'run400/map-14/frame-snapshot.bin'})
        self.assertIn('run300', orphans)
        self.assertIn('run95', orphans)
        self.assertIn('logs/rontrace-run700.log', orphans)
        self.assertNotIn('run100', orphans | ledger)
        self.assertIn('run200', ledger)
        self.assertIn('ch7', ledger)
        for name in ('recreatable', 'orphans', 'ledger-only'):
            c = s['candidates'][name]
            self.assertEqual(c['alloc'], sum(r['alloc'] for r in c['entries']))

    def test_totals_add_up(self):
        s = ac.summarize(self.rows)
        self.assertEqual(sum(v['alloc'] for v in s['by_kind'].values()),
                         sum(r['alloc'] for r in self.rows))
        self.assertEqual(sum(v['alloc'] for v in s['by_class'].values()),
                         sum(r['alloc'] for r in self.rows))


class Cli(Archive):
    def run_main(self, *extra):
        out = io.StringIO()
        with redirect_stdout(out):
            ac.main(['--root', str(self.golden), '--logs', str(self.logs), '--data', str(self.data),
                     '--repo', str(self.repo), *extra])
        return out.getvalue()

    def test_json_is_the_whole_thing(self):
        doc = json.loads(self.run_main('--json'))
        self.assertEqual({r['entry'] for r in doc['entries']}, set(self.by))
        self.assertIn('candidates', doc['summary'])

    def test_the_table_has_the_three_lists(self):
        text = self.run_main('--top', '3')
        for heading in ('recreatable', 'orphans', 'ledger-only'):
            self.assertRegex(text, re.compile(r'^#+ .*%s' % heading, re.M | re.I))
        table = [ln for ln in text.splitlines() if ln.startswith('| ')]
        self.assertEqual(len(table), 3 + 1)  # the header and three rows

    def test_the_tree_is_unchanged(self):
        def listing(root):
            return sorted((str(p), p.lstat().st_size) for p in Path(root).rglob('*'))
        before = listing(self.tmp.name)
        self.run_main()
        self.run_main('--json')
        self.assertEqual(listing(self.tmp.name), before)

    def test_the_source_holds_nothing_that_deletes(self):
        src = (ROOT / 'tools/archive_census.py').read_text()
        for call in ('remove(', 'unlink(', 'rmtree(', 'rmdir(', 'rename(', '.replace(', 'truncate(',
                     'shutil', 'subprocess', "'w'", '"w"', "'a'", '"a"'):
            self.assertNotIn(call, src, call)


if __name__ == '__main__':
    unittest.main()
