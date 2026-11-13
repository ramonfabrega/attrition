"""`tools/suite_groups.py`: the release suite's groups, and the audit's
comparison (item 4216). Authored audits only: no gate, no fixtures."""
import collections
import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import suite_groups  # noqa: E402


def audit(rows):
    return {test: collections.Counter(fixtures) for test, fixtures in rows.items()}


AUDIT = audit({
    # One game: a walk, a window on it, and the packets, joined by run10's trace.
    'diff::a_pair::first_parting': ['start.txt', 'gamelog-run10.txt', 'rontrace-run10.log'],
    'diff::a_pair::window': ['gamelog-run10.txt', 'gamelog-run10b.txt', 'rontrace-run10.log', 'rontrace-run10b.log'],
    'diff::packets::a_pair_packets': ['rontrace-run10.log'],
    # Another game, one test: no group.
    'diff::b_pair::first_parting': ['rontrace-run20.log'],
    # Reads the traces of three games: joins none.
    'diff::endpoint::families': ['rontrace-run10.log', 'rontrace-run20.log', 'rontrace-run30.log'],
    # Asks for a dump and no trace: no game.
    'diff::golden::chapter': ['golden-one.txt'],
    # A corpus sweep: two tests of one module over a hundred files each.
    'mapgen::tests::one': [f'gamelog-{i}.txt' for i in range(120)],
    'mapgen::tests::two': [f'gamelog-{i}.txt' for i in range(120)],
    'free::ages': [f'gamelog-{i}.txt' for i in range(120)],
})


class Groups(unittest.TestCase):
    def test_a_game_is_the_tests_one_trace_joins_and_a_sweep_is_a_module_s(self):
        self.assertEqual(suite_groups.groups_of(AUDIT), {
            'sweep:mapgen': ['mapgen::tests::one', 'mapgen::tests::two'],
            'game:run10': ['diff::a_pair::first_parting', 'diff::a_pair::window', 'diff::packets::a_pair_packets'],
        })

    def test_a_wide_test_is_in_no_group(self):
        groups = suite_groups.groups_of(AUDIT, wide={'diff::a_pair::window', 'mapgen::tests::two'})
        self.assertEqual(groups, {'game:run10': ['diff::a_pair::first_parting', 'diff::packets::a_pair_packets']})

    def test_a_test_of_many_games_would_have_joined_them(self):
        # Seen red on the first cut, which had no bound on a test's traces:
        # the endpoint test put both games, and itself, in one group.
        self.assertFalse(suite_groups.walks_a_game(AUDIT['diff::endpoint::families']))
        self.assertTrue(suite_groups.walks_a_game(AUDIT['diff::a_pair::window']))
        self.assertFalse(suite_groups.walks_a_game(AUDIT['diff::golden::chapter']))

    def test_the_stand_ups_are_the_games_and_their_walkers(self):
        self.assertEqual(suite_groups.stand_ups(AUDIT), (2, 4))


class Audit(unittest.TestCase):
    def test_equal_audits_differ_nowhere_and_a_test_one_side_ran_is_named(self):
        other = dict(AUDIT)
        del other['free::ages']
        other['new::test'] = collections.Counter(['x'])
        self.assertEqual(suite_groups.audit_differences(AUDIT, other), ({}, ['free::ages'], ['new::test']))

    def test_a_lost_row_and_a_doubled_row_are_differences(self):
        other = {test: collections.Counter(rows) for test, rows in AUDIT.items()}
        del other['mapgen::tests::two']['gamelog-7.txt']
        other['diff::a_pair::window']['rontrace-run10.log'] += 1
        differing, only_a, only_b = suite_groups.audit_differences(AUDIT, other)
        self.assertEqual(differing, {'diff::a_pair::window': {'rontrace-run10.log': (1, 2)},
                                     'mapgen::tests::two': {'gamelog-7.txt': (1, 0)}})
        self.assertEqual((only_a, only_b), ([], []))

    def test_the_audit_is_read_from_a_report_directory_and_from_a_saved_file(self):
        with tempfile.TemporaryDirectory() as d:
            d = Path(d)
            (d / 'fixture-requests').mkdir()
            rows = [('1', 'gamelog-a.txt', 'one'), ('1', 'gamelog-a.txt', 'one'), ('0', 'gamelog-b.txt', 'two')]
            (d / 'fixture-requests' / '7.tsv').write_text(
                ''.join(f'{ok}\t{name.encode().hex()}\t{test.encode().hex()}\n' for ok, name, test in rows))
            want = {'one': collections.Counter({'gamelog-a.txt': 2}), 'two': collections.Counter({'gamelog-b.txt': 1})}
            self.assertEqual(suite_groups.read_audit(d), want)
            self.assertEqual(suite_groups.read_audit(d / 'fixture-requests'), want)
            (d / 'saved.json').write_text(json.dumps({test: dict(c) for test, c in want.items()}))
            self.assertEqual(suite_groups.read_audit(d / 'saved.json'), want)
            (d / 'fixture-requests' / '8.tsv').write_text('2\tzz\n')
            with self.assertRaises(ValueError):
                suite_groups.read_audit(d)

    def test_the_command_exits_one_on_a_difference(self):
        with tempfile.TemporaryDirectory() as d:
            a, b = Path(d) / 'a.json', Path(d) / 'b.json'
            a.write_text(json.dumps({'t': {'x': 1}}))
            b.write_text(json.dumps({'t': {'x': 1}}))
            self.assertEqual(suite_groups.main(['--audit', str(a), str(b)]), 0)
            b.write_text(json.dumps({'t': {'x': 2}}))
            self.assertEqual(suite_groups.main(['--audit', str(a), str(b)]), 1)


if __name__ == '__main__':
    unittest.main()
