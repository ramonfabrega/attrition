"""`tools/lanewait.py`: the commander's wait on its lanes (the
twenty-fourth pass).

The decision is a pure function of what two polls saw, so the shapes the
tranches met are authored here: a lane that pushed, a lane that ended its
turn having sent nothing (parked 1412), a turn that ended on a backgrounded
wait and came back, a commit that was not a push, and a quiet lane.
"""
import sys
import tempfile
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

    def test_a_reaped_tree_has_no_tip_and_no_crash(self):
        # The fifty-ninth tranche's lanewait-30: a waiter on lane 3690 ran
        # after the commander removed its worktree and died in subprocess
        # on the missing cwd — a waiter that dies names nothing.
        self.assertIsNone(lanewait.origin_tip('/nonexistent/worktrees/att-0', 'worktree-att-0'))
        self.assertIsNone(lanewait.origin_tip(None, 'worktree-att-0'))

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


class ADraftUnderAClear(unittest.TestCase):
    """Item 3368 (the forty-sixth pass): att-3365 crossed DEEP, pushed its
    draft journal as the commander's answer asks, and ended its turn for
    the armed `ccc clear`; the row left `working` for the seconds the
    clear took and the waiter exited `LANDED att-3365 … none..2caf3aee`
    with the row back at work under a new session. A push that arrives
    with the session's uuid changed, or with the lane's newest transcript
    ending on a `/clear`, is a draft: recorded, said on one line, and
    watched past."""

    def watching(self, tip='1' * 40, session='s1'):
        watch = watching(a=tip)
        watch['a']['session'] = session
        return watch

    def test_a_push_with_the_session_changed_is_a_draft_not_a_landing(self):
        watch, drafts = self.watching(), []
        found = lanewait.event(watch, {'a': 'idle'}, {'a': '2' * 40}, 5, 100, sessions={'a': 's2'},
                               drafts=drafts)
        self.assertIsNone(found)
        self.assertEqual(len(drafts), 1)
        self.assertTrue(drafts[0].startswith('DRAFT a worktree-a 11111111..22222222'), drafts)
        # The ref is recorded and the new session is the one watched: the
        # same tip again is no push, and the row still out of `working` a
        # second poll is an ended row (a clear whose prompt never ran),
        # never the draft's landing.
        self.assertEqual(lanewait.event(watch, {'a': 'idle'}, {'a': '2' * 40}, 6, 100, sessions={'a': 's2'}),
                         (3, 'ENDED a idle'))
        watch, drafts = self.watching(), []
        lanewait.event(watch, {'a': 'idle'}, {'a': '2' * 40}, 5, 100, sessions={'a': 's2'}, drafts=drafts)
        self.assertIsNone(lanewait.event(watch, {'a': 'working'}, {'a': '2' * 40}, 6, 100,
                                         sessions={'a': 's2'}))

    def test_the_new_session_s_own_push_lands(self):
        watch = self.watching()
        self.assertIsNone(lanewait.event(watch, {'a': 'working'}, {'a': '2' * 40}, 5, 100, sessions={'a': 's2'},
                                         drafts=[]))
        self.assertEqual(lanewait.event(watch, {'a': 'done'}, {'a': '3' * 40}, 6, 100, sessions={'a': 's2'}),
                         (0, 'LANDED a worktree-a 22222222..33333333'))

    def test_a_push_with_the_session_unchanged_still_lands(self):
        watch, drafts = self.watching(), []
        found = lanewait.event(watch, {'a': 'done'}, {'a': '2' * 40}, 5, 100, sessions={'a': 's1'},
                               drafts=drafts)
        self.assertEqual(found, (0, 'LANDED a worktree-a 11111111..22222222'))
        self.assertEqual(drafts, [])

    def test_a_clear_with_no_push_does_not_make_the_next_push_a_draft(self):
        # A DEEP answered by a clear with nothing pushed: the session moves
        # on one poll, and the new session's landing a later poll is real.
        watch = self.watching()
        self.assertIsNone(lanewait.event(watch, {'a': 'working'}, {'a': '1' * 40}, 5, 100, sessions={'a': 's2'}))
        self.assertEqual(lanewait.event(watch, {'a': 'done'}, {'a': '2' * 40}, 6, 100, sessions={'a': 's2'}),
                         (0, 'LANDED a worktree-a 11111111..22222222'))

    def test_a_push_made_while_working_and_then_cleared_is_a_draft(self):
        # The draft pushed while the old turn still ran; the clear fired by
        # the next poll. The pending push is the old session's draft.
        watch, notes, drafts = self.watching(), [], []
        self.assertIsNone(lanewait.event(watch, {'a': 'working'}, {'a': '2' * 40}, 5, 100, notes,
                                         sessions={'a': 's1'}))
        self.assertEqual(notes, ['PUSHED a worktree-a 11111111..22222222'])
        self.assertIsNone(lanewait.event(watch, {'a': 'working'}, {'a': '2' * 40}, 6, 100, sessions={'a': 's2'},
                                         drafts=drafts))
        self.assertEqual(len(drafts), 1)
        self.assertIsNone(lanewait.event(watch, {'a': 'idle'}, {'a': '2' * 40}, 7, 100, sessions={'a': 's2'}))
        self.assertEqual(lanewait.event(watch, {'a': 'idle'}, {'a': '2' * 40}, 8, 100, sessions={'a': 's2'}),
                         (3, 'ENDED a idle'))

    def test_a_push_whose_transcript_ends_on_a_clear_is_a_draft(self):
        # The clear typed, the uuid not yet moved in the roster: the
        # transcript's own `/clear` tail is the tell.
        watch, drafts = self.watching(), []
        found = lanewait.event(watch, {'a': 'idle'}, {'a': '2' * 40}, 5, 100, sessions={'a': 's1'},
                               cleared={'a'}, drafts=drafts)
        self.assertIsNone(found)
        self.assertEqual(len(drafts), 1)
        self.assertIn('clear', drafts[0])

    def test_a_tail_that_ends_on_a_clear_is_one(self):
        self.assertTrue(lanewait.ends_on_clear((100.0, None)))
        self.assertTrue(lanewait.ends_on_clear((100.0, 90.0)))
        self.assertFalse(lanewait.ends_on_clear((100.0, 110.0)))
        self.assertFalse(lanewait.ends_on_clear((None, 110.0)))
        self.assertFalse(lanewait.ends_on_clear(None))

    def test_an_unknown_session_is_read_as_unchanged(self):
        watch = self.watching(session=None)
        self.assertEqual(lanewait.event(watch, {'a': 'done'}, {'a': '2' * 40}, 5, 100, sessions={'a': 's2'}),
                         (0, 'LANDED a worktree-a 11111111..22222222'))
        watch = self.watching()
        self.assertEqual(lanewait.event(watch, {'a': 'done'}, {'a': '2' * 40}, 5, 100, sessions={'a': None}),
                         (0, 'LANDED a worktree-a 11111111..22222222'))


class AnArmedClearIsADraft(unittest.TestCase):
    """Loop 3458 (the fiftieth pass): att-3446 pushed its draft on a DEEP
    answer and ended its turn for the armed clear; the waiter polled in the
    gap after the turn ended and before the clear fired, saw neither tell
    (the session unmoved, no `/clear` in the transcript) and printed
    `LANDED att-3446`. Before a LANDED the armed clear itself is asked
    (`ccc clear <lane> --status --json`); an armed one makes it a draft."""

    def watching(self):
        watch = watching(a='1' * 40)
        watch['a']['session'] = 's1'
        return watch

    def test_a_push_by_a_stopped_row_under_an_armed_clear_is_a_draft(self):
        watch, drafts, asked = self.watching(), [], []
        found = lanewait.event(watch, {'a': 'idle'}, {'a': '2' * 40}, 5, 100, sessions={'a': 's1'},
                               drafts=drafts, armed=lambda lane: asked.append(lane) or True)
        self.assertIsNone(found)
        self.assertEqual(asked, ['a'])
        self.assertEqual(drafts, ['DRAFT a worktree-a 11111111..22222222 — pushed under an armed clear; '
                                  'watching on'])
        # The ref is recorded: the clear fires, the new session works on.
        self.assertIsNone(lanewait.event(watch, {'a': 'working'}, {'a': '2' * 40}, 6, 100,
                                         sessions={'a': 's2'}, armed=lambda lane: False))

    def test_a_push_noted_while_working_then_stopped_under_an_armed_clear_is_a_draft(self):
        watch, notes, drafts = self.watching(), [], []
        self.assertIsNone(lanewait.event(watch, {'a': 'working'}, {'a': '2' * 40}, 5, 100, notes,
                                         sessions={'a': 's1'}, armed=lambda lane: True))
        self.assertEqual(notes, ['PUSHED a worktree-a 11111111..22222222'])
        for minute in (6, 7):
            self.assertIsNone(lanewait.event(watch, {'a': 'idle'}, {'a': '2' * 40}, minute, 100,
                                             sessions={'a': 's1'}, drafts=drafts, armed=lambda lane: True))
        self.assertEqual(drafts, ['DRAFT a worktree-a 11111111..22222222 — pushed under an armed clear; '
                                  'watching on'])

    def test_no_armed_clear_still_lands(self):
        watch, drafts = self.watching(), []
        self.assertEqual(lanewait.event(watch, {'a': 'done'}, {'a': '2' * 40}, 5, 100, sessions={'a': 's1'},
                                        drafts=drafts, armed=lambda lane: False),
                         (0, 'LANDED a worktree-a 11111111..22222222'))
        self.assertEqual(drafts, [])

    def test_the_status_is_read_from_ccc_and_any_failure_is_no_clear(self):
        import subprocess

        def answer(stdout, code=0):
            def run(command, **kwargs):
                self.assertEqual(command, ['ccc', 'clear', 'att-1', '--status', '--json'])
                return subprocess.CompletedProcess(command, code, stdout=stdout, stderr='')
            return run

        def missing(command, **kwargs):
            raise FileNotFoundError('ccc')

        self.assertTrue(lanewait.armed_clear('att-1', answer('{"armed": true, "ref": "att-1"}')))
        self.assertFalse(lanewait.armed_clear('att-1', answer('{"armed": false, "ref": "att-1"}')))
        self.assertFalse(lanewait.armed_clear('att-1', answer('{"armed": true}', code=1)))
        self.assertFalse(lanewait.armed_clear('att-1', answer('not json')))
        self.assertFalse(lanewait.armed_clear('att-1', answer('[]')))
        self.assertFalse(lanewait.armed_clear('att-1', missing))


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

    def test_a_deep_lane_already_answered_is_not_named_again_until_it_grows(self):
        # Loop 2186 (the commander, 2026-10-14): `reported` lived in the
        # process, so a DEEP answered while the lane's gate ran fired again
        # at the next re-arm (2107 at 303k, then 310k three minutes later),
        # and the commander raised --deep for every lane to quiet one. The
        # answers persist across arms: a lane answered at k is quiet until
        # it has grown a step past k.
        answered = {'a': 303_000}
        self.assertIsNone(lanewait.deep({'a': 310_000}, 300_000, set(), answered=answered))
        self.assertEqual(lanewait.deep({'a': 345_000}, 300_000, set(), answered=answered, step=40_000),
                         (5, 'DEEP a 345k'))
        self.assertEqual(answered['a'], 345_000, 'the answer moves with the exit')
        self.assertEqual(lanewait.deep({'b': 301_000}, 300_000, set(), answered=answered), (5, 'DEEP b 301k'))
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'deep.json'
            self.assertEqual(lanewait.load_answered(path), {})
            lanewait.save_answered(path, {'a': 345_000})
            self.assertEqual(lanewait.load_answered(path), {'a': 345_000})

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

    def test_a_working_row_idle_behind_an_old_child_has_stalled(self):
        # Loop 2820, the thirty-eighth pass: 2793 ended its turn at 08:25 with
        # a subagent's background `cargo test` as its only child, sat idle
        # 1 h 50 while the child ran four hours, and no exit fired — a child
        # of any age read as a wait. A gate takes twenty minutes and a
        # capture eighty; a lane idle past the threshold whose every child
        # has outlived `child_after` is named, with the child.
        idle = {'a': 110.0, 'b': 45.0, 'c': 110.0}
        children = {'a': [(110.0, 'cargo test --release -p sim city_data')],
                    'b': [(20.0, 'python3 tools/release_gate.py')],
                    'c': [(110.0, 'zsh -c sleep 9'), (5.0, 'python3 tools/release_gate.py')]}
        states = {'a': 'working', 'b': 'working', 'c': 'working'}
        found = lanewait.stalled(['b', 'c', 'a'], states, idle, children, 30, set(), child_after=90)
        self.assertEqual(found, (6, 'STALLED a 110 (child 110 min: cargo test --release -p sim city_data)'))
        self.assertIsNone(lanewait.stalled(['a'], states, idle, children, 30, set(), child_after=0),
                          'off at 0: a child of any age is a wait, as before')
        # The old shape — a command line alone — stays a wait of unknown age.
        self.assertIsNone(lanewait.stalled(['a'], states, idle, {'a': ['cargo test']}, 30, set(), child_after=90))
        self.assertEqual(lanewait.etime_minutes('1-02:03:04'), 26 * 60 + 3 + 4 / 60)
        self.assertEqual(lanewait.etime_minutes('02:03'), 2 + 3 / 60)
        self.assertEqual(lanewait.etime_minutes('03:04:05'), 184 + 5 / 60)
        self.assertIsNone(lanewait.etime_minutes('-'))

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


class TheDeepAnswersArmGoesInAfterItsPush(unittest.TestCase):
    """Loop 3526 (the fifty-second pass): twice in one tranche (items 3523,
    3524) the commander's `ccc clear <lane> --then …` on a DEEP answer was
    refused — "has N uncommitted changes; commit the bank first" — because
    the lane was mid-edit, and the arm had to go in by hand after the
    lane's draft push. With `--arm-after-push <lane>` the waiter takes the
    lane's next push as the DEEP answer's draft and arms the clear itself,
    once, on the committed tree the push left."""

    def watching(self):
        watch = watching(a='1' * 40, b='5' * 40)
        watch['a']['session'] = 's1'
        watch['b']['session'] = 't1'
        return watch

    def test_the_item_is_the_lane_s_number_and_the_template_takes_both(self):
        self.assertEqual(lanewait.item_of('att-3523'), '3523')
        self.assertEqual(lanewait.item_of('att-3523-2'), '3523')
        self.assertEqual(lanewait.item_of('scout'), 'scout')
        self.assertEqual(lanewait.then_prompt('continue item {item} from the journal ({lane})', 'att-3523'),
                         'continue item 3523 from the journal (att-3523)')

    def test_a_push_arms_the_clear_once_with_the_substituted_prompt(self):
        watch, pending, armed = self.watching(), {'a'}, []
        drafts, lines = lanewait.arm_pushes(
            watch, {'a': '2' * 40, 'b': '5' * 40}, {'a': 's1', 'b': 't1'}, pending,
            'continue item {item} from the journal',
            lambda lane, prompt: armed.append((lane, prompt)) or (True, 'armed'),
            already_armed=lambda lane: False)
        self.assertEqual(armed, [('a', 'continue item a from the journal')])
        self.assertEqual(drafts, ["DRAFT a worktree-a 11111111..22222222 — the DEEP answer's push; "
                                  "arming its clear"])
        self.assertEqual(lines, ['ARMED a — armed'])
        self.assertEqual(pending, set())
        # The push is consumed: the row working on is never a PUSHED of the
        # draft, and a later push is not armed again.
        notes = []
        self.assertIsNone(lanewait.event(watch, {'a': 'working', 'b': 'working'},
                                         {'a': '2' * 40, 'b': '5' * 40}, 6, 100, notes,
                                         sessions={'a': 's1', 'b': 't1'}))
        self.assertEqual(notes, [])
        drafts, lines = lanewait.arm_pushes(watch, {'a': '3' * 40}, {'a': 's1'}, pending, '{item}',
                                            lambda lane, prompt: self.fail('armed twice'),
                                            already_armed=lambda lane: False)
        self.assertEqual((drafts, lines), ([], []))

    def test_a_refusal_is_said_and_the_waiter_watches_on(self):
        watch, pending = self.watching(), {'a'}

        def refused(lane, prompt):
            return False, 'att-a has 3 uncommitted changes; commit the bank first'

        drafts, lines = lanewait.arm_pushes(watch, {'a': '2' * 40}, {'a': 's1'}, pending, '{item}', refused,
                                            already_armed=lambda lane: False)
        self.assertEqual(lines, ['ARM-REFUSED a: att-a has 3 uncommitted changes; commit the bank first'])
        self.assertEqual(pending, set())
        # Nothing exits on the refusal; the row stopping is an ENDED the
        # commander reads beside the ARM-REFUSED line, never a LANDED draft.
        self.assertIsNone(lanewait.event(watch, {'a': 'idle', 'b': 'working'}, {'a': '2' * 40}, 6, 100,
                                         sessions={'a': 's1', 'b': 't1'}))
        self.assertEqual(lanewait.event(watch, {'a': 'idle', 'b': 'working'}, {'a': '2' * 40}, 7, 100,
                                        sessions={'a': 's1', 'b': 't1'}), (3, 'ENDED a idle'))

    def test_a_lane_not_named_and_a_lane_that_has_not_pushed_are_left_alone(self):
        watch, pending = self.watching(), {'a'}
        drafts, lines = lanewait.arm_pushes(watch, {'a': '1' * 40, 'b': '6' * 40}, {'a': 's1', 'b': 't1'},
                                            pending, '{item}', lambda lane, prompt: self.fail('armed'),
                                            already_armed=lambda lane: False)
        self.assertEqual((drafts, lines, pending), ([], [], {'a'}))
        self.assertEqual(watch['b']['tip'], '5' * 40)

    def test_a_clear_already_armed_or_fired_is_not_armed_again(self):
        watch, pending = self.watching(), {'a'}
        drafts, lines = lanewait.arm_pushes(watch, {'a': '2' * 40}, {'a': 's1'}, pending, '{item}',
                                            lambda lane, prompt: self.fail('armed'),
                                            already_armed=lambda lane: True)
        self.assertEqual(lines, ['ARMED a — a clear was already armed; not armed again'])
        self.assertEqual(pending, set())
        # A push whose session moved is the clear's own: left to event() as
        # the draft it already reads it as.
        watch, pending = self.watching(), {'a'}
        drafts, lines = lanewait.arm_pushes(watch, {'a': '2' * 40}, {'a': 's2'}, pending, '{item}',
                                            lambda lane, prompt: self.fail('armed'),
                                            already_armed=lambda lane: False)
        self.assertEqual((drafts, lines, pending),
                         ([], ['ARMED a — its clear has fired (session s1..s2); not armed again'], set()))
        self.assertEqual(watch['a']['tip'], '1' * 40)

    def test_the_arm_is_ccc_s_and_any_failure_is_a_refusal(self):
        import subprocess

        def answer(stdout='', stderr='', code=0):
            def run(command, **kwargs):
                self.assertEqual(command, ['ccc', 'clear', 'att-1', '--then', 'continue item 1'])
                return subprocess.CompletedProcess(command, code, stdout=stdout, stderr=stderr)
            return run

        def missing(command, **kwargs):
            raise FileNotFoundError('ccc')

        def slow(command, **kwargs):
            raise subprocess.TimeoutExpired(command, 60)

        self.assertEqual(lanewait.arm_clear('att-1', 'continue item 1', answer('armed att-1\nfires at idle\n')),
                         (True, 'armed att-1'))
        self.assertEqual(lanewait.arm_clear('att-1', 'continue item 1',
                                            answer(stderr='att-1 has 2 uncommitted changes; commit the bank '
                                                          'first\n', code=1)),
                         (False, 'att-1 has 2 uncommitted changes; commit the bank first'))
        self.assertEqual(lanewait.arm_clear('att-1', 'continue item 1', answer(code=3)), (False, 'exit 3'))
        self.assertFalse(lanewait.arm_clear('att-1', 'continue item 1', missing)[0])
        self.assertFalse(lanewait.arm_clear('att-1', 'continue item 1', slow)[0])

    def test_the_flag_names_a_watched_lane(self):
        self.assertEqual(lanewait.main(['a', '--arm-after-push', 'b', '--roster-wait', '0']), 2)


class TheRosterIsAwaited(unittest.TestCase):
    """Loop 2620 (the thirty-sixth pass): `lanewait.py` armed in the spawn's
    own turn exited 2 at once — "no roster row with a worktree branch for
    att-2615" — because the row had not yet shown its branch; a wait that
    dies at arm leaves the commander waking on nothing."""

    def test_a_lane_whose_branch_is_not_yet_listed_is_waited_for(self):
        answers = [{'a': ('working', '/t/a', None, '1', None)},
                   {'a': ('working', '/t/a', None, '1', None)},
                   {'a': ('working', '/t/a', 'worktree-a', '1', None)}]
        slept = []
        rows, missing = lanewait.rows_for(['a'], 60, 5, roster_fn=lambda: answers.pop(0),
                                          sleep_fn=slept.append)
        self.assertEqual(missing, [])
        self.assertEqual(rows['a'][2], 'worktree-a')
        self.assertEqual(slept, [5, 5])

    def test_a_lane_still_missing_after_the_wait_is_named(self):
        slept = []
        rows, missing = lanewait.rows_for(['a', 'b'], 12, 5,
                                          roster_fn=lambda: {'a': ('working', '/t/a', 'worktree-a', '1', None)},
                                          sleep_fn=slept.append)
        self.assertEqual(missing, ['b'])
        self.assertEqual(slept, [5, 5, 2])

    def test_a_roster_that_does_not_answer_is_none(self):
        rows, missing = lanewait.rows_for(['a'], 60, 5, roster_fn=lambda: None, sleep_fn=lambda s: None)
        self.assertIsNone(rows)


class TheWaiterSaysItStarted(unittest.TestCase):
    """Loop 3716 (the sixtieth pass): fourteen of the sixtieth tranche's
    thirty-four waiter logs were empty, and nine of the sixty-first's
    sixteen, because the waiter printed nothing until an event — a waiter
    the commander replaced, a quiet one and a dead one all read the same.
    The waiter's first line, flushed before its first poll, names what it
    watches; an empty log is a waiter that never ran."""

    class Stop(Exception):
        pass

    def run_quiet(self, argv, polls=3):
        import contextlib
        from unittest import mock
        rows = {'lane-1': ('working', '/t/att-1', 'worktree-att-1', '1', None),
                'lane-2': ('working', '/t/att-2', 'worktree-att-2', '1', None)}

        def fake_roster():
            fake_roster.trees = {'lane-1': '/t/att-1', 'lane-2': '/t/att-2'}
            fake_roster.rests = {}
            fake_roster.sessions = {'lane-1': 's1', 'lane-2': 's2'}
            return dict(rows)

        tips = {'/t/att-1': None, '/t/att-2': '5' * 40}
        seen_at_first_poll = []
        with tempfile.TemporaryDirectory() as tmp:
            log = Path(tmp) / 'lanewait-1.log'

            def fake_sleep(seconds):
                if not seen_at_first_poll:
                    seen_at_first_poll.append(log.read_text())
                if len(seen_at_first_poll) > polls:
                    raise self.Stop()
                seen_at_first_poll.append(None)

            with open(log, 'w') as out, contextlib.redirect_stdout(out), \
                    mock.patch.object(lanewait, 'roster', fake_roster), \
                    mock.patch.object(lanewait, 'origin_tip', lambda cwd, branch: tips[cwd]), \
                    mock.patch.object(lanewait.time, 'sleep', fake_sleep):
                with self.assertRaises(self.Stop):
                    lanewait.main(argv + ['--deep', '0', '--stalled-after', '0', '--uncleared-after', '0',
                                          '--quiet-after', '1000', '--interval', '1', '--deep-state', ''])
            return log.read_text(), seen_at_first_poll[0]

    def test_a_wait_whose_roster_never_changes_leaves_exactly_its_start_line(self):
        import re
        text, at_first_poll = self.run_quiet(['lane-1', 'lane-2', '--arm-after-push', 'lane-1'])
        lines = text.splitlines()
        self.assertEqual(len(lines), 1, text)
        self.assertRegex(lines[0], r'^WAITING lane-1 worktree-att-1 none; lane-2 worktree-att-2 55555555 '
                                   r'— \d{4}-\d\d-\d\dT\d\d:\d\d:\d\dZ pid \d+ arm-after-push lane-1$')
        # Flushed before the first poll: a waiter killed in its first sleep
        # still says it ran.
        self.assertEqual(at_first_poll, text)
        self.assertTrue(re.match(r'^WAITING ', at_first_poll))

    def test_the_start_line_is_not_an_exit_line(self):
        # The commander reads a waiter's log with `cat` and its exit from
        # the task's last line; the start line opens on a word no exit and
        # no event line uses.
        text, _ = self.run_quiet(['lane-2'], polls=1)
        self.assertTrue(text.startswith('WAITING lane-2 worktree-att-2 55555555 — '), text)
        for word in ('LANDED', 'ENDED', 'QUIET', 'DEEP', 'STALLED', 'UNCLEARED', 'PUSHED', 'DRAFT', 'ARMED',
                     'ARM-REFUSED'):
            self.assertFalse(text.startswith(word))


def send(at, summary, message='', to='commander'):
    """One assistant record carrying a SendMessage call, as the harness
    writes it."""
    import json
    return json.dumps({'type': 'assistant', 'timestamp': at, 'message': {'content': [
        {'type': 'tool_use', 'name': 'SendMessage', 'input': {'to': to, 'summary': summary, 'message': message}}]}})


class ALandingPushIsNotADraft(unittest.TestCase):
    """Loop 3749 (the sixty-first pass, item 3744): lane-3744 crossed DEEP
    at 308 k while its lane gate ran; the commander answered and re-armed
    with `--arm-after-push lane-3744`, and the lane's next push was its
    finished landing — `none..10e53239`, the very tip the merge took —
    sent with its done line, `3744 landed and pushed`. `arm_pushes` read
    the push as the DEEP draft and armed a clear on a landed lane; the
    clear fired and a second session re-announced the landing. Of the
    twenty-five DEEP-answer pushes on the commander's transcripts two were
    landings (3591, 3744), and every one of the forty-three done lines and
    draft notes those lanes sent reads one way or the other on its summary
    alone. So a pending lane's push is armed only once the row has stopped
    with no done line since the push; a done line makes it a landing."""

    LANDINGS = ['3744 landed and pushed', 'Announce item 3539 landed', 'Item 3543 landed, done line',
                "Announce item 3567's landing", '3591 landed and pushed', "Announce 3615's landing to the commander",
                'Announce 3646 landed', 'Announce item 3654 landing', 'announce 3666 landing',
                'Item 3708 done line', '3711 landed, done line']
    DRAFTS = ['3539 draft pushed, base moved', '3567 handoff draft pushed', '3615 handover pushed',
              '3634 draft journal pushed at DEEP clear', '3661 draft handoff pushed',
              'Draft pushed at 311k for the clear', 'deep exit: tip and state', '3711 handoff pushed, gate owed',
              '3742 handover pushed, word closed', '3710 confirms 4842, run1387 capturing',
              '90-minute status for item 1194', '1477 status: cause found, re-pinning a wide fix',
              '2971 partial landing report to commander', 'Tell commander landing is nearly done',
              '566: run138 landed, mechanism found', '3099 DEEP draft committed and pushed; not landed',
              '919 evidence landed, gate running', 'Lane released, landing the floor now']

    def watching(self):
        watch = watching(a='1' * 40)
        watch['a']['session'] = 's1'
        return watch

    def test_a_done_line_reads_as_a_landing_and_a_draft_note_never(self):
        for summary in self.LANDINGS:
            self.assertTrue(lanewait.is_done_line(summary), summary)
        for summary in self.DRAFTS:
            self.assertFalse(lanewait.is_done_line(summary), summary)
        # No summary: the message's head is read, and "unlanded" is not landed.
        self.assertTrue(lanewait.is_done_line('', 'Item 3588 landed: worktree-att-3588 at 1e4ed3e67'))
        self.assertFalse(lanewait.is_done_line('', '3646 draft pushed, unlanded (not gated): the pair'))
        self.assertFalse(lanewait.is_done_line('', '865 status: I haven\'t landed yet. No gate is running'))
        self.assertTrue(lanewait.is_done_line('', '3099 DONE: tip 61d07908 on worktree-att-3099, pushed'))
        self.assertTrue(lanewait.is_done_line('3346 done: six swept, pushed'))

    def test_the_transcript_s_done_line_since_the_push_is_found(self):
        from datetime import datetime
        since = datetime.fromisoformat('2026-10-28T11:18:40+00:00').timestamp()
        lines = [send('2026-10-28T10:02:00.000Z', 'Item 3744 landed, done line'),
                 '{"type": "user", "timestamp": "2026-10-28T11:18:46.849Z", "message": {"content": "pushed"}}',
                 send('2026-10-28T11:18:57.134Z', '3744 landed and pushed')]
        self.assertEqual(lanewait.done_line(lines, since), '3744 landed and pushed')
        self.assertIsNone(lanewait.done_line(lines[:2], since), 'a done line before the push is not this push')
        self.assertIsNone(lanewait.done_line([send('2026-10-28T11:18:57.134Z', '3744 draft pushed for the clear')],
                                             since))
        self.assertIsNone(lanewait.done_line(['not json', ''], since))
        # A line to the lane's own subagent, or to another lane, is not its done line.
        self.assertIsNone(lanewait.done_line([send('2026-10-28T11:18:57.134Z', '', 'The lane has its ten; '
                                                   'landed work stops now', to='a0651062ff871c606')], since))
        self.assertIsNone(lanewait.done_line([send('2026-10-28T11:18:57.134Z', '1127 landed', to='att-1133')],
                                             since))
        self.assertEqual(lanewait.done_line([send('2026-10-28T11:18:57.134Z', '', '3744 landed: tip 10e53239',
                                                  to='uds:/tmp/cc-socks/76694.sock')], since),
                         '3744 landed: tip 10e53239')

    def test_a_push_sent_with_its_done_line_is_not_armed_and_lands(self):
        watch, pending = self.watching(), {'a'}
        drafts, lines = lanewait.arm_pushes(
            watch, {'a': '2' * 40}, {'a': 's1'}, pending, '{item}',
            lambda lane, prompt: self.fail('armed a landed lane'), already_armed=lambda lane: False,
            states={'a': 'working'}, landed=lambda lane, since: '3744 landed and pushed', since=100.0)
        self.assertEqual(drafts, [])
        self.assertEqual(lines, ['PUSHED a worktree-a 11111111..22222222 — its done line, '
                                 "'3744 landed and pushed'; a landing, not armed"])
        self.assertEqual(pending, set())
        # The landing is event()'s: the row stopping lands it, as any push
        # noted while the row worked.
        states, tips = {'a': 'idle'}, {'a': '2' * 40}
        self.assertIsNone(lanewait.event(watch, states, tips, 6, 100, sessions={'a': 's1'}))
        self.assertEqual(lanewait.event(watch, states, tips, 7, 100, sessions={'a': 's1'},
                                        armed=lambda lane: False),
                         (0, 'LANDED a worktree-a 11111111..22222222'))

    def test_a_push_while_the_row_works_is_held_until_its_done_line_or_its_stop(self):
        asked = []

        def landed(answer):
            return lambda lane, since: asked.append(since) or answer

        # Held: the row works on with no done line yet — nothing armed,
        # nothing said, and event() never reads the push as a PUSHED.
        watch, pending, armed = self.watching(), {'a'}, []
        arm = lambda lane, prompt: armed.append(lane) or (True, 'armed')  # noqa: E731
        out = lanewait.arm_pushes(watch, {'a': '2' * 40}, {'a': 's1'}, pending, '{item}', arm,
                                  already_armed=lambda lane: False, states={'a': 'working'},
                                  landed=landed(None), since=100.0)
        self.assertEqual((out, pending, armed), (([], []), {'a'}, []))
        notes = []
        self.assertIsNone(lanewait.event(watch, {'a': 'working'}, {'a': '2' * 40}, 6, 100, notes,
                                         sessions={'a': 's1'}))
        self.assertEqual(notes, [])
        # The row stops with no done line since the push: the draft, armed
        # on its first push's range and asked from that push's poll.
        drafts, lines = lanewait.arm_pushes(watch, {'a': '2' * 40}, {'a': 's1'}, pending, '{item}', arm,
                                            already_armed=lambda lane: False, states={'a': 'idle'},
                                            landed=landed(None), since=130.0)
        self.assertEqual(drafts, ["DRAFT a worktree-a 11111111..22222222 — the DEEP answer's push; "
                                  "arming its clear"])
        self.assertEqual((lines, armed, pending), (['ARMED a — armed'], ['a'], set()))
        self.assertEqual(asked, [100.0, 100.0])
        # Held, and the done line follows on the next poll: a landing.
        watch, pending = self.watching(), {'a'}
        lanewait.arm_pushes(watch, {'a': '2' * 40}, {'a': 's1'}, pending, '{item}', arm,
                            already_armed=lambda lane: False, states={'a': 'working'}, landed=landed(None),
                            since=100.0)
        drafts, lines = lanewait.arm_pushes(watch, {'a': '2' * 40}, {'a': 's1'}, pending, '{item}',
                                            lambda lane, prompt: self.fail('armed a landed lane'),
                                            already_armed=lambda lane: False, states={'a': 'working'},
                                            landed=landed('Item 3744 landed'), since=130.0)
        self.assertEqual((drafts, pending), ([], set()))
        self.assertEqual(lines, ["PUSHED a worktree-a 11111111..22222222 — its done line, 'Item 3744 landed'; "
                                 "a landing, not armed"])
        self.assertEqual(watch['a']['pushed'], 'a worktree-a 11111111..22222222')

    def test_a_second_push_while_held_keeps_the_first_push_s_range(self):
        watch, pending = self.watching(), {'a'}
        for tip in ('2' * 40, '3' * 40):
            lanewait.arm_pushes(watch, {'a': tip}, {'a': 's1'}, pending, '{item}',
                                lambda lane, prompt: self.fail('armed'), already_armed=lambda lane: False,
                                states={'a': 'working'}, landed=lambda lane, since: None, since=100.0)
        drafts, _ = lanewait.arm_pushes(watch, {'a': '3' * 40}, {'a': 's1'}, pending, '{item}',
                                        lambda lane, prompt: (True, 'armed'), already_armed=lambda lane: False,
                                        states={'a': 'idle'}, landed=lambda lane, since: None, since=160.0)
        self.assertEqual(drafts, ["DRAFT a worktree-a 11111111..33333333 — the DEEP answer's push; "
                                  "arming its clear"])


GATE = 'python3 tools/release_gate.py /Users/x/game --lane --tests melee'


class ALaneGateIsNotAnEnd(unittest.TestCase):
    """Loop 3753 (the sixty-second pass): lane-3747 committed its journal
    at 11:14:31, started its lane gate backgrounded and ended its turn;
    the row read `done` with the gate alive and nothing on origin, and
    two arms of `lanewait.py lane-3747 lane-3748` printed `ENDED lane-3747
    done — …` 69 s after that commit and 60 s after WAITING — the wait on
    lane-3748 lost with it. 3759 the same (ENDED 63 s after its journal
    commit; its gate's quote landed 23 minutes later). A row out of
    `working` whose session has a gate running under it is GATING: said
    once on stderr, held, and read again when the gate has gone."""

    def test_a_row_out_of_working_with_its_gate_running_is_gating_once(self):
        watch, notes = watching(a=None), []
        states, tips, details = {'a': 'done'}, {'a': None}, {'a': '3747 Egyptian bay rows landed'}
        for minutes in (1, 2, 3, 4):
            self.assertIsNone(lanewait.event(watch, states, tips, minutes, 100, details=details,
                                             gating={'a': (3.4, GATE)}, gating_notes=notes))
        self.assertEqual(notes, [f'GATING a done — {GATE} (3 min); held, read again at its exit'])
        # The gate exits: its notice re-invokes the session, so the row
        # must stay out STILL polls past it before it is ENDED.
        self.assertIsNone(lanewait.event(watch, states, tips, 5, 100, details=details, gating={},
                                         gating_notes=notes))
        self.assertEqual(lanewait.event(watch, states, tips, 6, 100, details=details, gating={},
                                        gating_notes=notes),
                         (3, 'ENDED a done — 3747 Egyptian bay rows landed'))
        self.assertEqual(len(notes), 1)

    def test_a_push_while_gating_still_lands(self):
        watch = watching(a=None)
        lanewait.event(watch, {'a': 'done'}, {'a': None}, 1, 100, gating={'a': (1.0, GATE)}, gating_notes=[])
        self.assertEqual(lanewait.event(watch, {'a': 'done'}, {'a': '2' * 40}, 2, 100,
                                        gating={'a': (2.0, GATE)}, gating_notes=[]),
                         (0, 'LANDED a worktree-a none..22222222'))

    def test_a_second_gate_is_said_again(self):
        watch, notes = watching(a=None), []
        for gating in ({'a': (1.0, GATE)}, {'a': (1.0, GATE)}, {}, {'a': (0.5, 'cargo test --release -p sim')},
                       {'a': (1.0, 'cargo test --release -p sim')}):
            lanewait.event(watch, {'a': 'idle'}, {'a': None}, 1, 100, gating=gating, gating_notes=notes)
        self.assertEqual([n.split(' — ')[0] for n in notes], ['GATING a idle', 'GATING a idle'])

    def test_the_gate_is_any_descendant_of_the_lane_s_session(self):
        table = {100: (1, 60.0, 'claude --model opus'),
                 200: (100, 3.0, "/bin/zsh -c source ~/.zshrc && eval '" + GATE + "'"),
                 201: (200, 3.0, GATE),
                 202: (100, 50.0, 'bun run lazy-tools'),
                 300: (1, 9.0, 'claude --model opus'),
                 301: (300, 9.0, 'cargo test --release -p sim'),
                 400: (1, 2.0, 'claude --model opus'),
                 401: (400, 2.0, "/bin/zsh -c eval 'ls tools/explore/test_release_gate.py; grep -n cargo test x'"),
                 500: (1, 2.0, 'claude'),
                 501: (500, 7.0, '/opt/homebrew/Cellar/python@3.13/3.13.1/Frameworks/Python.framework/Versions/3.13/'
                                 'Resources/Python.app/Contents/MacOS/Python tools/release_gate.py /x --lane'),
                 600: (1, 2.0, 'claude'),
                 601: (600, 1.0, '/Users/x/.rustup/toolchains/1.86-aarch64-apple-darwin/bin/cargo test --release'),
                 602: (600, 1.0, 'zsh tools/check.sh test -p sim')}
        # The gate is its own process, never a shell whose text names it.
        self.assertEqual(lanewait.gate_under(100, table), (3.0, GATE))
        self.assertEqual(lanewait.gate_under(300, table), (9.0, 'cargo test --release -p sim'))
        self.assertIsNone(lanewait.gate_under(400, table))
        self.assertEqual(lanewait.gate_under(500, table)[0], 7.0)
        self.assertEqual(lanewait.gate_under(600, table)[0], 1.0)
        self.assertIsNone(lanewait.gate_under(999, table))
        self.assertIsNone(lanewait.gate_under(None, table))


class AHeldPushOutlivesItsWait(unittest.TestCase):
    """Loop 3765 (the sixty-second pass): lane-3763 pushed its DEEP
    answer's draft (a4ee8056) under `lanewait.py lane-3763 lane-3731
    --arm-after-push lane-3763`; `arm_pushes` held it while the row
    worked, and the next poll's `DEEP lane-3731 300k` exited 5 with
    nothing said of 3763 — no ARMED, no DRAFT — and the row went idle
    with no clear armed. The re-armed wait starts with the ref already
    moved and never saw the push. So any exit says `HELD` first, and the
    next --arm-after-push wait on the lane arms the push the last one
    held (the base each wait records, beside --deep-state)."""

    class Stop(Exception):
        pass

    def run_main(self, argv, tmp, polls, armed=False, done=None, arms=None, rests=None, kids=None, tail=(),
                 refuse=None):
        """`main` over a scripted roster: `polls[i]` is what poll i sees
        (poll 0 is the start), as {'tips', 'states', 'ctx', 'ps'}; the
        last entry stands for every later poll. `rests` stands for the
        roster's `(pid, tempo, updatedAt)` by lane, `kids` for `children_of`
        by pid, `refuse` for ccc's first line on a refused arm, and `tail`
        is argv read last (it outranks the defaults here). Returns (exit,
        out, err)."""
        import contextlib
        import io
        from unittest import mock
        at = [0]

        def now():
            return polls[min(at[0], len(polls) - 1)]

        def fake_roster():
            fake_roster.trees = {'lane-1': '/t/att-1', 'lane-2': '/t/att-2'}
            fake_roster.rests = rests or {'lane-1': (100, 'idle', None), 'lane-2': (200, 'idle', None)}
            fake_roster.sessions = {'lane-1': 's1', 'lane-2': 's2'}
            states = now().get('states', {})
            return {lane: (states.get(lane, 'working'), f'/t/att-{lane[-1]}', f'worktree-att-{lane[-1]}', '1',
                           None) for lane in ('lane-1', 'lane-2')}

        def fake_sleep(seconds):
            at[0] += 1
            if at[0] >= len(polls) + 3:
                raise self.Stop()

        def fake_arm(lane, prompt):
            (arms if arms is not None else []).append((lane, prompt))
            return (False, refuse) if refuse else (True, 'armed')

        out, err = io.StringIO(), io.StringIO()
        code = None
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err), \
                mock.patch.object(lanewait, 'roster', fake_roster), \
                mock.patch.object(lanewait, 'origin_tip',
                                  lambda cwd, branch: now().get('tips', {}).get(cwd[-5:].replace('att', 'lane'))), \
                mock.patch.object(lanewait, 'contexts', lambda lanes: now().get('ctx', {})), \
                mock.patch.object(lanewait, 'process_table', lambda: now().get('ps', {})), \
                mock.patch.object(lanewait, 'tails_of', lambda lanes, where, sessions: {}), \
                mock.patch.object(lanewait, 'armed_clear', lambda lane, run=None: armed), \
                mock.patch.object(lanewait, 'arm_clear', fake_arm), \
                mock.patch.object(lanewait, 'done_line_of', lambda lane, since, where, sessions: done), \
                mock.patch.object(lanewait, 'children_of', lambda pids: {pid: (kids or {}).get(pid, [])
                                                                         for pid in pids}), \
                mock.patch.object(lanewait.time, 'sleep', fake_sleep):
            try:
                code = lanewait.main(argv + ['--deep-every', '1', '--stalled-after', '0', '--uncleared-after',
                                             '0', '--quiet-after', '1000', '--interval', '1',
                                             '--deep-state', str(Path(tmp) / 'lanewait-deep.json')] + list(tail))
            except self.Stop:
                pass
        return code, out.getvalue(), err.getvalue()

    PUSH_THEN_DEEP = [{'tips': {}},
                      {'tips': {'lane-1': '2' * 40}},
                      {'tips': {'lane-1': '2' * 40}, 'ctx': {'lane-2': 310_000}}]

    def test_an_exit_with_a_push_held_says_held_first(self):
        with tempfile.TemporaryDirectory() as tmp:
            code, out, err = self.run_main(['lane-1', 'lane-2', '--arm-after-push', 'lane-1'], tmp,
                                           self.PUSH_THEN_DEEP)
        lines = out.splitlines()
        self.assertEqual(code, 5, out + err)
        self.assertEqual(lines[1:], ['HELD lane-1 none..22222222 — held for its done line, not armed; '
                                     'the next --arm-after-push lane-1 wait arms it',
                                     'DEEP lane-2 310k'])

    def test_the_next_arm_after_push_wait_arms_the_held_push_at_its_start(self):
        with tempfile.TemporaryDirectory() as tmp:
            self.run_main(['lane-1', 'lane-2', '--arm-after-push', 'lane-1'], tmp, self.PUSH_THEN_DEEP)
            arms = []
            idle = [{'tips': {'lane-1': '2' * 40}, 'states': {'lane-1': 'idle'}}]
            code, out, err = self.run_main(['lane-1', 'lane-2', '--arm-after-push', 'lane-1'], tmp, idle,
                                           arms=arms)
            lines = out.splitlines()
            self.assertEqual(lines[:2], [lines[0], 'ARMED lane-1 — armed'], out)
            self.assertTrue(lines[0].startswith('WAITING lane-1 worktree-att-1 22222222; '), out)
            self.assertIn("DRAFT lane-1 worktree-att-1 none..22222222 — the DEEP answer's push; arming its clear",
                          err)
            self.assertEqual(arms, [('lane-1', 'continue item 1 from the journal')])
            # Armed once: a third wait on the lane arms nothing.
            arms.clear()
            _, out, _ = self.run_main(['lane-1', 'lane-2', '--arm-after-push', 'lane-1'], tmp, idle, arms=arms)
            self.assertEqual(arms, [])
            self.assertNotIn('ARMED', out)

    def test_a_held_push_whose_done_line_followed_is_a_landing_not_an_arm(self):
        with tempfile.TemporaryDirectory() as tmp:
            self.run_main(['lane-1', 'lane-2', '--arm-after-push', 'lane-1'], tmp, self.PUSH_THEN_DEEP)
            arms = []
            idle = [{'tips': {'lane-1': '2' * 40}, 'states': {'lane-1': 'idle'}}]
            code, out, _ = self.run_main(['lane-1', 'lane-2', '--arm-after-push', 'lane-1'], tmp, idle,
                                         done='3763 landed and pushed', arms=arms)
        self.assertEqual(arms, [])
        self.assertIn("PUSHED lane-1 worktree-att-1 none..22222222 — its done line, '3763 landed and pushed'; "
                      "a landing, not armed", out)
        self.assertEqual((code, out.splitlines()[-1]), (0, 'LANDED lane-1 worktree-att-1 none..22222222'))

    def test_a_clear_already_armed_is_not_armed_again_at_start(self):
        with tempfile.TemporaryDirectory() as tmp:
            self.run_main(['lane-1', 'lane-2', '--arm-after-push', 'lane-1'], tmp, self.PUSH_THEN_DEEP)
            arms = []
            idle = [{'tips': {'lane-1': '2' * 40}, 'states': {'lane-1': 'idle'}}]
            _, out, _ = self.run_main(['lane-1', 'lane-2', '--arm-after-push', 'lane-1'], tmp, idle, armed=True,
                                      arms=arms)
        self.assertEqual(arms, [])
        self.assertIn('ARMED lane-1 — a clear was already armed; not armed again', out)

    def test_a_push_no_wait_held_is_never_armed_at_start(self):
        # The ref moved before any --arm-after-push wait recorded a base —
        # a mid-work push, or a lane whose DEEP answer it has not yet read.
        with tempfile.TemporaryDirectory() as tmp:
            arms = []
            idle = [{'tips': {'lane-1': '2' * 40}, 'states': {'lane-1': 'idle'}}]
            _, out, _ = self.run_main(['lane-1', 'lane-2', '--arm-after-push', 'lane-1'], tmp, idle, arms=arms)
        self.assertEqual(arms, [])
        self.assertNotIn('ARMED', out)

    def test_a_wait_without_the_flag_forgets_the_base(self):
        with tempfile.TemporaryDirectory() as tmp:
            self.run_main(['lane-1', 'lane-2', '--arm-after-push', 'lane-1'], tmp, self.PUSH_THEN_DEEP)
            self.run_main(['lane-1', 'lane-2'], tmp, [{'tips': {'lane-1': '2' * 40}, 'ctx': {'lane-2': 360_000}}])
            arms = []
            idle = [{'tips': {'lane-1': '2' * 40}, 'states': {'lane-1': 'idle'}}]
            _, out, _ = self.run_main(['lane-1', 'lane-2', '--arm-after-push', 'lane-1'], tmp, idle, arms=arms)
        self.assertEqual(arms, [])

    def test_a_row_gating_is_said_on_stderr_and_the_wait_goes_on(self):
        ps = {100: (1, 60.0, 'claude'), 101: (100, 4.0, GATE)}
        with tempfile.TemporaryDirectory() as tmp:
            code, out, err = self.run_main(['lane-1', 'lane-2'], tmp, [
                {'tips': {}}, {'states': {'lane-1': 'done'}, 'ps': ps}, {'states': {'lane-1': 'done'}, 'ps': ps},
                {'states': {'lane-1': 'done'}, 'ps': ps}, {'states': {'lane-1': 'done'}, 'ps': ps},
                {'states': {'lane-1': 'done'}, 'ctx': {'lane-2': 320_000}, 'ps': ps}])
        self.assertEqual(code, 5, out + err)
        self.assertEqual(out.splitlines()[1:], ['DEEP lane-2 320k'])
        self.assertEqual(err.splitlines(), [f'GATING lane-1 done — {GATE} (4 min); held, read again at its exit'])


def idle_for(minutes):
    """A roster `updatedAt` (ccc's ISO instant, `Z`) `minutes` ago."""
    from datetime import datetime, timedelta, timezone
    return (datetime.now(timezone.utc) - timedelta(minutes=minutes)).strftime('%Y-%m-%dT%H:%M:%S.000Z')


class AHeldDraftOnAStalledRowIsArmed(unittest.TestCase):
    """Loop 3795 (the sixty-third pass): lane-3788 answered its `DEEP
    339k`, pushed its draft (73765697) and ended its turn, but its row read
    `working` throughout — its own word, at `tempo=idle`. lanewait-28
    exited `HELD lane-3788 none..73765697` on the other lane's DEEP,
    lanewait-29 seeded the push held, held it again because the row read
    `working`, and exited `STALLED lane-3788 30`; `ccc clear --status` read
    no clear armed the whole time, and the commander armed it by hand. Its
    sequel, Loop 3796: the stall nudge then reached the fresh session. A
    held draft on a row idle with nothing running is the draft the arm
    waits for: the stall is the arm, never an exit."""

    Stop = AHeldPushOutlivesItsWait.Stop
    run_main = AHeldPushOutlivesItsWait.run_main
    PUSH = [{'tips': {}}, {'tips': {'lane-1': '2' * 40}}]
    ARGS = ['lane-1', 'lane-2', '--arm-after-push', 'lane-1']
    PROMPT = [('lane-1', 'continue item 1 from the journal')]
    DRAFT = "DRAFT lane-1 worktree-att-1 none..22222222 — held draft on a stalled row; arming its clear"

    def rests(self, minutes):
        return {'lane-1': (100, 'idle', idle_for(minutes)), 'lane-2': (200, 'idle', None)}

    def test_a_held_draft_on_a_stalled_row_is_armed_once_and_the_wait_goes_on(self):
        # The 3788 shape: the row reads `working`, idle 45 minutes past a
        # 30-minute --stalled-after, nothing under it but its servers.
        arms = []
        with tempfile.TemporaryDirectory() as tmp:
            code, out, err = self.run_main(self.ARGS, tmp, self.PUSH, arms=arms, rests=self.rests(45),
                                           kids={100: [(90.0, 'bun /x/lazy/server.ts')]},
                                           tail=['--stalled-after', '30'])
        self.assertIsNone(code, out + err)
        self.assertEqual(arms, self.PROMPT)
        self.assertEqual(out.splitlines()[1:], ['ARMED lane-1 — armed'], out)
        self.assertEqual(err.splitlines(), [self.DRAFT], err)

    def test_the_held_arm_waits_its_own_threshold_and_falls_back_to_stalled_after_at_0(self):
        # Ten minutes by default: idle 12 arms, idle 7 does not.
        for minutes, extra, want in ((12, [], self.PROMPT), (7, [], []),
                                     (25, ['--held-arm-after', '0'], []),
                                     (31, ['--held-arm-after', '0'], self.PROMPT)):
            arms = []
            with tempfile.TemporaryDirectory() as tmp:
                code, out, _ = self.run_main(self.ARGS, tmp, self.PUSH, arms=arms, rests=self.rests(minutes),
                                             tail=['--stalled-after', '30'] + extra)
            self.assertEqual((code, arms), (None, want), f'{minutes} {extra}: {out}')
        # Its own: armed with --stalled-after off.
        arms = []
        with tempfile.TemporaryDirectory() as tmp:
            code, out, _ = self.run_main(self.ARGS, tmp, self.PUSH, arms=arms, rests=self.rests(12),
                                         tail=['--stalled-after', '0'])
        self.assertEqual((code, arms), (None, self.PROMPT), out)

    def test_a_held_draft_with_a_gate_or_a_capture_running_is_not_armed(self):
        for child in ('python3 tools/release_gate.py /x/game --lane', 'zsh tools/gamelog/capture_pair.sh 3'):
            arms = []
            with tempfile.TemporaryDirectory() as tmp:
                code, out, err = self.run_main(self.ARGS, tmp, self.PUSH, arms=arms, rests=self.rests(45),
                                               kids={100: [(90.0, 'bun /x/lazy/server.ts'), (20.0, child)]},
                                               tail=['--stalled-after', '30'])
            self.assertEqual((code, arms), (None, []), out + err)
            self.assertNotIn('ARMED', out)
            self.assertNotIn('DRAFT', err)

    def test_a_held_draft_whose_clear_is_already_armed_is_not_armed_again(self):
        arms = []
        with tempfile.TemporaryDirectory() as tmp:
            code, out, err = self.run_main(self.ARGS, tmp, self.PUSH, armed=True, arms=arms, rests=self.rests(45),
                                           tail=['--stalled-after', '30'])
        self.assertEqual((code, arms), (None, []), out + err)
        self.assertEqual(out.splitlines()[1:], ['ARMED lane-1 — a clear was already armed; not armed again'])
        self.assertEqual(err.splitlines(), [self.DRAFT])

    def test_a_held_draft_whose_done_line_followed_lands_and_is_not_armed(self):
        arms = []
        with tempfile.TemporaryDirectory() as tmp:
            code, out, _ = self.run_main(self.ARGS, tmp, self.PUSH, done='3788 landed and pushed', arms=arms,
                                         rests=self.rests(45), tail=['--stalled-after', '30'])
        self.assertEqual(arms, [])
        self.assertIn("PUSHED lane-1 worktree-att-1 none..22222222 — its done line, '3788 landed and pushed'; "
                      "a landing, not armed", out)

    def test_a_refused_held_arm_leaves_the_stall_to_the_commander(self):
        # ccc refused: said, the lane leaves the arm, and the old exit fires
        # on the next poll — the row needs a hand, and nothing armed.
        arms = []
        with tempfile.TemporaryDirectory() as tmp:
            code, out, _ = self.run_main(self.ARGS, tmp, self.PUSH, arms=arms, rests=self.rests(45),
                                         refuse='Error: uncommitted changes', tail=['--stalled-after', '30'])
        self.assertEqual(arms, self.PROMPT)
        self.assertEqual(out.splitlines()[1:], ['ARM-REFUSED lane-1: Error: uncommitted changes',
                                                'STALLED lane-1 45'], out)
        self.assertEqual(code, 6)

    def test_a_stalled_row_with_no_held_draft_still_exits_stalled(self):
        arms = []
        with tempfile.TemporaryDirectory() as tmp:
            # No --arm-after-push: the old exit, as before.
            code, out, _ = self.run_main(['lane-1', 'lane-2'], tmp, self.PUSH, arms=arms, rests=self.rests(45),
                                         tail=['--stalled-after', '30'])
            self.assertEqual((code, out.splitlines()[-1]), (6, 'STALLED lane-1 45'), out)
            # --arm-after-push, but nothing pushed: nothing held, the old exit.
            code, out, _ = self.run_main(self.ARGS, tmp, [{'tips': {}}], arms=arms, rests=self.rests(45),
                                         tail=['--stalled-after', '30'])
            self.assertEqual((code, out.splitlines()[-1]), (6, 'STALLED lane-1 45'), out)
        self.assertEqual(arms, [])


if __name__ == '__main__':
    unittest.main()
