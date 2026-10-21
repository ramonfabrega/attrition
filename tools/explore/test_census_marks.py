"""The census's marks on the never-backed list (Loop 3405, the forty-seventh
pass): a function cut from v1 — a shell mode's driver, the match lifecycle,
the scenario and multiplayer hosts — is marked `cut` and left off the
sweepable list, like `ui` and the dead weight, and the denominator does not
move (the score's definition is Ramon's, Loop 3237)."""
import importlib.util
import io
import json
import pathlib
import unittest
from contextlib import redirect_stdout

ROOT = pathlib.Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('census', ROOT / 'tools' / 'census.py')
census = importlib.util.module_from_spec(spec)
spec.loader.exec_module(census)


class TheCutMark(unittest.TestCase):
    def rows(self):
        entries = [
            (0x009b8ac0, 'UnitBalance::next', 'e:\\agent\\_work\\2\\s\\main\\game\\unitbalance.cpp', 43058),
            (0x00584590, 'Game::run', 'e:\\agent\\_work\\2\\s\\main\\game\\game.cpp', 66803),
            (0x00604550, 'Unit::check_meet_ship', 'e:\\agent\\_work\\2\\s\\main\\game\\unit.cpp', 17274),
        ]
        return census.rank_unbacked(entries, backed=set(), port=set(), spelled=set(), claimed=set(), by='shortest')

    def test_a_function_cut_from_v1_is_marked_cut_and_hidden(self):
        rows = self.rows()
        marks = {name: mark for _, _, name, mark, _ in rows}
        self.assertEqual(marks['UnitBalance::next'], 'cut')
        self.assertEqual(marks['Game::run'], 'cut')
        self.assertEqual(marks['Unit::check_meet_ship'], '-')
        shown = [name for _, _, name, _, _ in census.worth_sweeping(rows)]
        self.assertEqual(shown, ['Unit::check_meet_ship'])
        # The denominator is the rows, not the shown: a cut row still counts.
        self.assertEqual(len(rows), 3)

    def test_the_cut_list_names_the_four_the_lanes_parked(self):
        for name in ('UnitBalance::next', 'Game::run', 'TimeSync::process', 'MapScenario::make',
                     'Game::run_scenario', 'Game::run_gamespy', 'CommandManager::set_mp_playback'):
            self.assertTrue(census.is_cut(name), name)
        self.assertFalse(census.is_cut('Unit::process'))


# The census's `entered` column beside `backed` (Loop 3350, the forty-eighth
# pass): a simulation function a pinned coverage trace entered — the set
# `--pin --never` measures the blind list against — counted beside the
# backed column and joined to nothing. The definition of `backed` is
# Ramon's (DECISIONS 29) and does not move; the `Census:` and `Census bytes:`
# lines stay as they were.
M = 'e:\\agent\\_work\\2\\s\\main\\'
SIM = M + 'game\\unit.cpp'
PATH = M + 'game\\pathfinder.cpp'
REN = M + 'game\\rendersystem.cpp'


class TheEnteredColumn(unittest.TestCase):
    # Four simulation functions, two backed (0x10 by a sweep, 0x20 by a
    # document); the traces entered 0x20 (backed) and 0x30, 0x40 (not), and
    # an engine function the column counts in its own row only.
    NAMES = {0x10: 'Unit::a', 0x20: 'Unit::b', 0x30: 'Unit::think', 0x40: 'PathFinder::find_path',
             0x50: 'Render::d'}
    SIZES = {0x10: 100, 0x20: 300, 0x30: 600, 0x40: 900, 0x50: 50}
    SOURCE = {0x10: SIM, 0x20: SIM, 0x30: SIM, 0x40: PATH, 0x50: REN}
    BY_DOC, BY_SWEEP = {0x20}, {0x10}
    ENTERED = {0x20, 0x30, 0x40, 0x50}

    def entered(self):
        return census.entered_rows(self.NAMES, self.SIZES, self.SOURCE, self.BY_DOC, self.BY_SWEEP,
                                   self.ENTERED)

    def layer_rows(self):
        return census.layer_rows(self.NAMES, self.SIZES, self.SOURCE, self.BY_DOC, self.BY_SWEEP,
                                 spelled=set())

    def test_the_counts_split_entered_by_backed(self):
        got = self.entered()
        sim = got['layers']['simulation']
        self.assertEqual((sim['entered'], sim['entered_unbacked']), (3, 2))
        self.assertEqual((sim['entered_bytes'], sim['entered_unbacked_bytes']), (1800, 1500))
        self.assertEqual(got['layers']['engine']['entered'], 1)
        self.assertEqual(dict(got['files']), {'unit.cpp': 2, 'pathfinder.cpp': 1})

    def test_the_largest_entered_and_unbacked_come_first_with_their_file(self):
        got = self.entered()
        self.assertEqual([(n, b, f) for b, _va, n, f in got['unbacked']],
                         [('PathFinder::find_path', 900, 'pathfinder.cpp'), ('Unit::think', 600, 'unit.cpp')])

    def test_the_line_follows_the_bytes_line_and_the_census_line_is_unchanged(self):
        rows, sim_files, unknown = self.layer_rows()
        out = io.StringIO()
        with redirect_stdout(out):
            census.print_layers(rows, sim_files, unknown, False, size_note='bytes: test',
                                entered=self.entered())
        lines = out.getvalue().splitlines()
        at = lines.index('Census: simulation backed 2 of 4 · port 1 · layout 1')
        self.assertEqual(lines[at + 1], 'Census bytes: simulation 400 of 1900 bytes backed · 21.1%')
        self.assertEqual(lines[at + 2],
                         'Census entered: simulation 3 of 4 functions · 2 entered and unbacked · '
                         '1800 of 1900 bytes')
        head = next(l for l in lines if l.startswith('layer'))
        for col in ('entered', 'ent unbacked', 'ent bytes', 'ent unb bytes'):
            self.assertIn(col, head)
        sim = next(l for l in lines if l.startswith('simulation '))
        self.assertEqual(sim.split()[-4:], ['3', '2', '1800', '1500'], sim)
        files = next(l for l in lines if 'unit.cpp:' in l).split()
        self.assertIn('unit.cpp:3/2', files)
        self.assertIn('pathfinder.cpp:1/1', files)

    def test_without_traces_the_table_and_lines_are_as_they_were(self):
        rows, sim_files, unknown = self.layer_rows()
        plain, none = io.StringIO(), io.StringIO()
        with redirect_stdout(plain):
            census.print_layers(rows, sim_files, unknown, False, size_note='bytes: test')
        with redirect_stdout(none):
            census.print_layers(rows, sim_files, unknown, False, size_note='bytes: test', entered=None)
        self.assertEqual(plain.getvalue(), none.getvalue())
        self.assertNotIn('Census entered:', plain.getvalue())
        self.assertNotIn('entered', next(l for l in plain.getvalue().splitlines() if l.startswith('layer')))
        self.assertIn('unit.cpp:3', plain.getvalue().split())

    def test_the_json_carries_the_entered_keys_only_when_measured(self):
        rows, sim_files, unknown = self.layer_rows()
        out = io.StringIO()
        with redirect_stdout(out):
            census.print_layers(rows, sim_files, unknown, True, size_note='b', entered=self.entered())
        got = json.loads(out.getvalue())
        self.assertEqual((got['simulation_entered'], got['simulation_entered_unbacked'],
                          got['simulation_entered_bytes'], got['simulation_entered_unbacked_bytes']),
                         (3, 2, 1800, 1500))
        self.assertEqual(out.getvalue().count('"simulation_backed":'), 1)
        out = io.StringIO()
        with redirect_stdout(out):
            census.print_layers(rows, sim_files, unknown, True, size_note='b')
        self.assertNotIn('entered', out.getvalue())

    def test_the_top_list_prints_rank_bytes_name_file(self):
        out = io.StringIO()
        with redirect_stdout(out):
            census.print_entered_unbacked(self.entered(), top=1)
        lines = out.getvalue().splitlines()
        self.assertIn('    1     900  PathFinder::find_path  pathfinder.cpp', lines)
        self.assertNotIn('Unit::think', out.getvalue())


if __name__ == '__main__':
    unittest.main()
