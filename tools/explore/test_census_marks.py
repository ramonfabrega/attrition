"""The census's marks on the never-backed list (Loop 3405, the forty-seventh
pass): a function cut from v1 — a shell mode's driver, the match lifecycle,
the scenario and multiplayer hosts — is marked `cut` and left off the
sweepable list, like `ui` and the dead weight, and the denominator does not
move (the score's definition is Ramon's, Loop 3237)."""
import importlib.util
import pathlib
import unittest

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


if __name__ == '__main__':
    unittest.main()
