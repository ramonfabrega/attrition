"""`tools/lanewait.py`: the commander's wait on its lanes (the
twenty-fourth pass).

The decision is a pure function of what two polls saw, so the shapes the
tranches met are authored here: a lane that pushed, a lane that ended its
turn having sent nothing (parked 1412), a turn that ended on a backgrounded
wait and came back, a commit that was not a push, and a quiet lane.
"""
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import lanewait  # noqa: E402


def watching(**lanes):
    return {lane: {'branch': f'worktree-{lane}', 'tip': tip, 'still': 0} for lane, tip in lanes.items()}


class Event(unittest.TestCase):
    def test_a_push_by_a_stopped_row_is_a_landing(self):
        watch = watching(a='1' * 40, b=None)
        found = lanewait.event(watch, {'a': 'blocked', 'b': 'working'},
                               {'a': '2' * 40, 'b': None}, 5, 100)
        self.assertEqual(found, (0, 'LANDED a worktree-a 11111111..22222222'))

    def test_a_first_push_by_a_stopped_row_is_a_landing(self):
        watch = watching(a=None)
        found = lanewait.event(watch, {'a': 'done'}, {'a': '3' * 40}, 5, 100)
        self.assertEqual(found, (0, 'LANDED a worktree-a none..33333333'))

    def test_a_push_while_the_row_works_is_noted_and_watched_on(self):
        # The twenty-fifth tranche: 1500 pushed four times and 1502 five
        # before each landed, and the waiter woke the commander on every one.
        watch = watching(a='1' * 40)
        notes = []
        found = lanewait.event(watch, {'a': 'working'}, {'a': '2' * 40}, 5, 100, notes)
        self.assertIsNone(found)
        self.assertEqual(notes, ['PUSHED a worktree-a 11111111..22222222'])
        # The same tip again is nothing new.
        notes = []
        self.assertIsNone(lanewait.event(watch, {'a': 'working'}, {'a': '2' * 40}, 6, 100, notes))
        self.assertEqual(notes, [])

    def test_a_row_that_pushed_and_then_stopped_has_landed(self):
        watch = watching(a='1' * 40)
        self.assertIsNone(lanewait.event(watch, {'a': 'working'}, {'a': '2' * 40}, 5, 100))
        self.assertIsNone(lanewait.event(watch, {'a': 'done'}, {'a': '2' * 40}, 6, 100))
        self.assertEqual(lanewait.event(watch, {'a': 'done'}, {'a': '2' * 40}, 7, 100),
                         (0, 'LANDED a worktree-a 11111111..22222222'))

    def test_nothing_moved_is_no_event(self):
        watch = watching(a='1' * 40)
        self.assertIsNone(lanewait.event(watch, {'a': 'working'}, {'a': '1' * 40}, 5, 100))

    def test_a_row_out_of_working_twice_has_ended(self):
        watch = watching(a='1' * 40)
        states, tips = {'a': 'done'}, {'a': '1' * 40}
        self.assertIsNone(lanewait.event(watch, states, tips, 5, 100))
        self.assertEqual(lanewait.event(watch, states, tips, 6, 100), (3, 'ENDED a done'))

    def test_a_turn_that_ended_on_a_backgrounded_wait_and_came_back_has_not(self):
        watch = watching(a='1' * 40)
        tips = {'a': '1' * 40}
        self.assertIsNone(lanewait.event(watch, {'a': 'done'}, tips, 5, 100))
        self.assertIsNone(lanewait.event(watch, {'a': 'working'}, tips, 6, 100))
        self.assertIsNone(lanewait.event(watch, {'a': 'done'}, tips, 7, 100))

    def test_a_row_the_roster_lost_is_gone(self):
        watch = watching(a='1' * 40)
        lanewait.event(watch, {'a': None}, {'a': '1' * 40}, 5, 100)
        self.assertEqual(lanewait.event(watch, {'a': None}, {'a': '1' * 40}, 6, 100), (3, 'ENDED a gone'))

    def test_a_push_outranks_an_ended_row(self):
        watch = watching(a='1' * 40, b='4' * 40)
        watch['a']['still'] = 5
        found = lanewait.event(watch, {'a': 'done', 'b': 'done'}, {'a': '1' * 40, 'b': '5' * 40}, 5, 100)
        self.assertEqual(found, (0, 'LANDED b worktree-b 44444444..55555555'))

    def test_quiet_names_every_lane(self):
        watch = watching(a='1' * 40, b=None)
        found = lanewait.event(watch, {'a': 'working', 'b': 'working'}, {'a': '1' * 40, 'b': None}, 100, 100)
        self.assertEqual(found, (4, 'QUIET a b 100'))


class StateOf(unittest.TestCase):
    def test_a_running_turn_is_working_whatever_the_row_says(self):
        # Parked 1572: a lane's self-written `blocked` stranded the waiter.
        self.assertEqual(lanewait.state_of({'state': 'blocked'}, {'tempo': 'working'}), 'working')
        self.assertEqual(lanewait.state_of({'state': 'working'}, {'tempo': 'working'}), 'working')

    def test_an_attached_but_resting_row_keeps_its_state(self):
        # `status: busy` means something is attached (a Monitor, a background
        # bash), not that the turn runs — ccc's word, 2026-10-07.
        self.assertEqual(lanewait.state_of({'state': 'blocked', 'status': 'busy'}, {'tempo': 'idle'}), 'blocked')
        self.assertEqual(lanewait.state_of({'state': 'done', 'status': 'busy'}, {}), 'done')
        self.assertEqual(lanewait.state_of({'state': 'done'}), 'done')
        self.assertIsNone(lanewait.state_of({}))

if __name__ == '__main__':
    unittest.main()


class SecondRowOnATree(unittest.TestCase):
    # Parked 1671: the commander gave a racer its second landing with
    # `ccc spawn --cwd <its tree> --name att-1648-sm-2`, so one tree carried
    # two rows for a day. A lane's next session is `ccc clear <its row>
    # --then`; the waiter refuses to start on a lane whose tree holds a
    # second row, whatever that row's state.
    def test_a_second_row_on_a_watched_lane_s_tree_is_refused(self):
        trees = {'att-1': '/w/att-1', 'att-1-2': '/w/att-1', 'commander': '/w/replan', 'steer': '/w/replan'}
        self.assertEqual(lanewait.second_rows(trees, ['att-1-2']), [('att-1-2', 'att-1', '/w/att-1')])

    def test_the_two_permanent_rows_share_the_integration_tree_by_design(self):
        trees = {'att-1': '/w/att-1', 'commander': '/w/replan', 'steer': '/w/replan'}
        self.assertEqual(lanewait.second_rows(trees, ['att-1']), [])


class EndedCarriesTheRowsDetail(unittest.TestCase):
    # Parked 1671's sibling: five of the race's twenty landings ended at
    # `tempo=blocked` asking for the commander's queue lines, and an armed
    # clear fires only on idle. The exit line carries the row's own detail
    # so the one unblocking message is written in the turn that reads it.
    def test_an_ended_row_s_detail_is_on_the_line(self):
        watch = watching(a='1' * 40)
        states, tips = {'a': 'blocked'}, {'a': '1' * 40}
        details = {'a': 'reply `go` to merge tip a22bdbef'}
        self.assertIsNone(lanewait.event(watch, states, tips, 5, 100, details=details))
        self.assertEqual(lanewait.event(watch, states, tips, 6, 100, details=details),
                         (3, 'ENDED a blocked — reply `go` to merge tip a22bdbef'))

