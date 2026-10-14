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
    def test_a_deep_lane_is_named_once(self):
        # The twenty-ninth pass: 1707 and 1726 ran to 341 k and neither
        # could arm its own clear; the waiter names a lane once when its
        # next turn would carry 300 k or more, and never one lore cannot see.
        reported = set()
        ctxs = {'a': 120_000, 'b': 341_000, 'c': None}
        self.assertEqual(lanewait.deep(ctxs, 300_000, reported), (5, 'DEEP b 341k'))
        self.assertIsNone(lanewait.deep(ctxs, 300_000, reported))
        ctxs['a'] = 305_500
        self.assertEqual(lanewait.deep(ctxs, 300_000, reported), (5, 'DEEP a 305k'))
        self.assertIsNone(lanewait.deep({'b': 400_000}, 0, set()), 'off at 0')

    def test_a_working_row_idle_with_nothing_running_has_stalled(self):
        # Parked 1871, the thirtieth pass: 1738's lane gate exited and the
        # notice never reached the session; the row read `working` at
        # `tempo=idle` for 178 minutes. A row that is idle past the
        # threshold with no child but its servers waits on nothing.
        idle = {'a': 45.0, 'b': 45.0, 'c': 3.0}
        children = {'a': ['bun server.ts'], 'b': ['bun server.ts', 'python3 tools/release_gate.py'],
                    'c': ['bun server.ts']}
        found = lanewait.stalled(['a', 'b', 'c'], {'a': 'working', 'b': 'working', 'c': 'working'},
                                 idle, children, 30, set())
        self.assertEqual(found, (6, 'STALLED a 45'))

    def test_a_stalled_row_is_named_once_and_a_busy_one_never(self):
        reported = set()
        idle = {'a': 45.0}
        children = {'a': []}
        self.assertEqual(lanewait.stalled(['a'], {'a': 'working'}, idle, children, 30, reported), (6, 'STALLED a 45'))
        self.assertIsNone(lanewait.stalled(['a'], {'a': 'working'}, idle, children, 30, reported))
        self.assertIsNone(lanewait.stalled(['b'], {'b': 'done'}, {'b': 90.0}, {'b': []}, 30, set()))
        self.assertIsNone(lanewait.stalled(['a'], {'a': 'working'}, {'a': 45.0}, {'a': ['sh -c sleep 9']}, 30, set()))

    def test_a_server_child_is_not_a_wait(self):
        self.assertTrue(lanewait.is_server('bun /x/lazy/server.ts'))
        self.assertTrue(lanewait.is_server('bun run --cwd /x/hail --shell=bun --silent start'))
        self.assertFalse(lanewait.is_server('python3 tools/lanewait.py att-1'))
        self.assertFalse(lanewait.is_server('zsh -c python3 tools/release_gate.py'))

    def test_a_clear_with_no_turn_after_it_is_uncleared_once(self):
        # Parked 2127, the thirty-second pass: twice a cross-session clear
        # fired, the lane's new session opened on the `/clear` record, and
        # the --then prompt never ran; the row sat twenty minutes until a
        # message carrying the prompt woke it. A lane whose newest
        # transcript ends on a `/clear` that has stood the threshold is
        # named once; a lane with a record after its `/clear`, or none, is not.
        now = 10_000.0
        tails = {'a': (now - 6 * 60, None), 'b': (now - 6 * 60, now - 5 * 60), 'c': (None, now - 60),
                 'd': (now - 60, None), 'e': None}
        reported = set()
        self.assertEqual(lanewait.uncleared(tails, now, 5, reported), (7, 'UNCLEARED a 6'))
        self.assertIsNone(lanewait.uncleared(tails, now, 5, reported))
        self.assertIsNone(lanewait.uncleared({'d': (now - 60, None)}, now, 5, set()), 'under the threshold')
        self.assertIsNone(lanewait.uncleared(tails, now, 0, set()), 'off at 0')

    def test_a_transcript_s_tail_is_its_last_clear_and_its_last_record(self):
        # Two listings, as the harness writes them: 2055's second session
        # (its `/clear` at 22:34:07, the commander's prompt at 22:35:35) and
        # the shape 2127 found, the `/clear` with nothing after it.
        import json
        head = [json.dumps({'type': 'custom-title', 'customTitle': 'att-2055'}),
                json.dumps({'type': 'file-history-snapshot', 'snapshot': {}})]
        clear = json.dumps({'type': 'user', 'timestamp': '2026-10-13T22:34:07.000Z',
                            'message': {'role': 'user', 'content': '<command-name>/clear</command-name> '
                                        '<command-message>clear</command-message>'}})
        prompt = json.dumps({'type': 'user', 'timestamp': '2026-10-13T22:35:35.000Z',
                             'message': {'role': 'user', 'content': [{'type': 'text', 'text':
                                         'Continue item 2055 from your journal draft'}]}})
        answer = json.dumps({'type': 'assistant', 'timestamp': '2026-10-13T22:35:40.000Z',
                             'message': {'role': 'assistant', 'content': [{'type': 'text', 'text': 'Reading.'}]}})
        from datetime import datetime, timezone
        at = lambda s: datetime.fromisoformat(s).replace(tzinfo=timezone.utc).timestamp()  # noqa: E731
        self.assertEqual(lanewait.transcript_tail(head + [clear]), (at('2026-10-13T22:34:07'), None))
        self.assertEqual(lanewait.transcript_tail(head + [clear, prompt, answer]),
                         (at('2026-10-13T22:34:07'), at('2026-10-13T22:35:40')))
        self.assertEqual(lanewait.transcript_tail(head + [answer, 'not json']), (None, at('2026-10-13T22:35:40')))
        # A session that ended on a clear and was cleared again: the newest
        # clear is the one that counts.
        self.assertEqual(lanewait.transcript_tail(head + [clear, prompt, clear]), (at('2026-10-13T22:34:07'), at('2026-10-13T22:35:35')))

    def test_an_ended_row_s_detail_is_on_the_line(self):
        watch = watching(a='1' * 40)
        states, tips = {'a': 'blocked'}, {'a': '1' * 40}
        details = {'a': 'reply `go` to merge tip a22bdbef'}
        self.assertIsNone(lanewait.event(watch, states, tips, 5, 100, details=details))
        self.assertEqual(lanewait.event(watch, states, tips, 6, 100, details=details),
                         (3, 'ENDED a blocked — reply `go` to merge tip a22bdbef'))

