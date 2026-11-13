"""tools/handover.py's rules, on authored rosters: no ccc, no process."""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import handover  # noqa: E402

TREE = '/repo/.claude/worktrees/replan-pdb'


def row(name, ref, state='done', cwd=TREE, archived=False):
    return {'archived': archived, 'job': {'tempo': 'idle'},
            'session': {'name': name, 'id': ref, 'state': state, 'cwd': cwd, 'pid': 1}}


class CheckTests(unittest.TestCase):
    def test_one_driver_row_passes(self):
        ok, line = handover.check_line([row('steer', 'aaaa0001', 'working')], TREE)
        self.assertTrue(ok)
        self.assertIn('steer aaaa0001', line)

    def test_two_driver_rows_fail_and_name_both(self):
        rows = [row('steer', 'aaaa0001', 'working'), row('commander', 'bbbb0002')]
        ok, line = handover.check_line(rows, TREE)
        self.assertFalse(ok)
        self.assertIn('steer aaaa0001', line)
        self.assertIn('commander bbbb0002', line)
        self.assertIn('handover.py reap', line)

    def test_a_stopped_or_archived_row_cannot_wake_and_is_not_counted(self):
        rows = [row('steer', 'aaaa0001', 'working'), row('commander', 'bbbb0002', 'stopped'),
                row('commander', 'cccc0003', archived=True)]
        self.assertTrue(handover.check_line(rows, TREE)[0])

    def test_a_lane_or_another_tree_s_row_is_not_a_driver_row(self):
        rows = [row('steer', 'aaaa0001', 'working'), row('lane-4200', 'bbbb0002', 'working'),
                row('commander', 'cccc0003', cwd='/elsewhere')]
        self.assertTrue(handover.check_line(rows, TREE)[0])

    def test_no_row_passes(self):
        self.assertTrue(handover.check_line([], TREE)[0])


class StrayTests(unittest.TestCase):
    TABLE = {
        10: (1, 'claude --bg'),          # the row
        20: (10, 'zsh -c handover'),     # this call's shell
        30: (20, 'python3 tools/handover.py to commander'),
        40: (10, 'zsh tools/gate_run.sh abc gate.log'),
        50: (40, 'python3 tools/release_gate.py'),
        60: (1, 'unrelated'),
    }

    def test_this_call_s_own_ancestry_is_not_a_stray(self):
        table = {k: v for k, v in self.TABLE.items() if k in (10, 20, 30, 60)}
        self.assertEqual(handover.strays(10, table, 30), [])

    def test_a_gate_under_the_row_is_named(self):
        found = handover.strays(10, self.TABLE, 30)
        self.assertEqual([pid for pid, _ in found], [40])
        self.assertIn('gate_run.sh', found[0][1])


if __name__ == '__main__':
    unittest.main()
