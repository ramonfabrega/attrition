"""`tools/tranche.py`: a tranche's workers folded from `lore trace` (the
twenty-first pass, 2026-09-30).

The nineteenth pass measured a landing's wall clock by hand, the twentieth
by a scratch script, the twenty-first is the third reach. The fixture is
one worker's shape: a turn that backgrounds a gate and ends, the gap, the
re-invocation that reads the log — the gap is waiting, and its class is the
gate, not the `tail` that came last.
"""
import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import tranche  # noqa: E402


def req(ts, out=100, cache=200_000, usd=0.5):
    return {'requestId': ts, 'ts': ts, 'model': 'claude-opus-5-5', 'stopReason': 'end_turn',
            'input': 2, 'cacheWrite': 1_000, 'cacheRead': cache, 'output': out, 'thinking': 0,
            'listUsd': usd}


def ins(ts, command, request):
    return {'tool': 'Bash', 'input': '{"command":"%s"}' % command, 'ts': ts, 'ms': 1000,
            'error': False, 'result': '', 'requestId': request}


TRACE = {'transactions': [
    {'ts': '2026-09-30T10:00:00.000Z',
     'requests': [req('2026-09-30T10:00:00.000Z'), req('2026-09-30T10:00:30.000Z', cache=310_000, usd=1.0)],
     'instructions': [ins('2026-09-30T10:00:01.000Z', 'python3 tools/release_gate.py game --lane > g.log',
                          '2026-09-30T10:00:00.000Z'),
                      ins('2026-09-30T10:00:31.000Z', 'git log --oneline -1', '2026-09-30T10:00:30.000Z')]},
    {'ts': '2026-09-30T10:12:00.000Z',
     'requests': [req('2026-09-30T10:12:00.000Z'), req('2026-09-30T10:12:20.000Z')],
     'instructions': [ins('2026-09-30T10:12:01.000Z', 'tail -1 g.log', '2026-09-30T10:12:00.000Z')]},
]}


class Fold(unittest.TestCase):
    def test_a_gap_is_waiting_and_its_class_is_what_was_backgrounded(self):
        row = tranche.fold(TRACE, gap_s=90, deep_k=300)
        self.assertEqual(row['requests'], 4)
        self.assertEqual(row['price'], 2.5)
        self.assertEqual(row['peak_k'], 311)
        self.assertEqual(row['deep_share'], 0.4)
        self.assertEqual(row['clock_min'], 12.3)
        self.assertEqual(row['waiting_min'], 11.5)
        self.assertEqual(row['on'], {'gate': 11.5})

    def test_a_short_gap_is_working(self):
        row = tranche.fold(TRACE, gap_s=800, deep_k=300)
        self.assertEqual(row['waiting_min'], 0.0)
        self.assertEqual(row['on'], {})

    def test_the_classes(self):
        for command, cls in [('python3 tools/release_gate.py x --lane', 'gate'),
                             # The forty-first pass: the read of a gate's log after the
                             # wake is the gate's wait, not `other`.
                             ('tail -c 1200 /Users/x/.claude/jobs/a40adc8e/tmp/gate2.log; grep -E Lane x', 'gate'),
                             ('grep -aE \\"Lane verdict|^Gate steps\\" $CLAUDE_JOB_DIR/tmp/gate.log | tail -5', 'gate'),
                             ('zsh tools/gamelog/waitrun.sh /tmp/ron-runs/viadriver-1.log', 'capture'),
                             ('cat $J/tmp/chain1221.log | tail -5', 'capture'),
                             ('zsh tools/memcap.sh 16 cargo test --release', 'suite'),
                             ('uv run tools/emu/callfn.py', 'emulator'),
                             ('sed -n 1,5p docs/AI.md', 'other')]:
            self.assertEqual(tranche.classify({'input': '{"command":"%s"}' % command}), cls, command)
        self.assertEqual(tranche.classify({'input': '{"to":"attrition","message":"1291 status"}'}), 'status')
        self.assertEqual(tranche.classify(None), 'other')

    def test_the_classes_the_twenty_fifth_pass_asked_for(self):
        # Parked 1421: a gate run through a script in the job's tmp, and a
        # fanned batch's subagents, folded into `other` — 86 of 1477's 176
        # minutes, 56 of 1619's 63.
        self.assertEqual(tranche.classify({'input': '{"command":"zsh $CLAUDE_JOB_DIR/tmp/gate.sh"}'}), 'script')
        self.assertEqual(tranche.classify(
            {'input': '{"command":"python3 /Users/x/.claude/jobs/abc/tmp/probe.py 3395"}'}), 'script')
        self.assertEqual(tranche.classify({'tool': 'Agent', 'input': '{"prompt":"Sweep find_angle"}'}),
                         'subagent')

    def test_a_gap_that_ends_at_an_inbound_message_is_waiting_on_it(self):
        # Parked 1421's second class: 1398 waited 334 minutes for a
        # commander's reply and the gap read as the suite.
        trace = {'transactions': [
            {'ts': '2026-09-30T10:00:00.000Z', 'kind': 'prompt',
             'requests': [req('2026-09-30T10:00:00.000Z')],
             'instructions': [ins('2026-09-30T10:00:01.000Z', 'cargo test --release -p rondata x',
                                  '2026-09-30T10:00:00.000Z')]},
            {'ts': '2026-09-30T12:00:00.000Z', 'kind': 'relay',
             'requests': [req('2026-09-30T12:00:00.000Z')],
             'instructions': []},
        ]}
        row = tranche.fold(trace, gap_s=90, deep_k=300)
        self.assertEqual(row['on'], {'message': 120.0})

    def test_a_gap_after_an_agent_call_is_the_subagent_s_even_when_its_answer_arrives_as_a_relay(self):
        # The forty-third pass: a sweep lane's subagent answers as a relay
        # turn, and 3239's 85 minutes on five subagents read as `message`
        # (the shape lanes' message class 1.8 -> 13.3 minutes a session).
        trace = {'transactions': [
            {'ts': '2026-10-18T10:00:00.000Z', 'kind': 'prompt',
             'requests': [req('2026-10-18T10:00:00.000Z')],
             'instructions': [{'ts': '2026-10-18T10:00:01.000Z', 'tool': 'Agent',
                               'input': '{"description":"Sweep Unit::find_goody_box","subagent_type":"lean"}',
                               'turn': '2026-10-18T10:00:00.000Z'}]},
            {'ts': '2026-10-18T10:30:00.000Z', 'kind': 'relay',
             'requests': [req('2026-10-18T10:30:00.000Z')],
             'instructions': []},
        ]}
        row = tranche.fold(trace, gap_s=90, deep_k=300)
        self.assertEqual(row['on'], {'subagent': 30.0})



def bg(ts, command, request, task):
    """A Bash call the harness backgrounded: its result names the task id."""
    return {'tool': 'Bash', 'input': json.dumps({'command': command, 'run_in_background': True}), 'ts': ts,
            'ms': 50, 'error': False, 'requestId': request,
            'result': 'Command running in background with ID: %s. Output is being written to: '
                      '/private/tmp/claude-501/x/s/tasks/%s.output' % (task, task)}


class OtherHeads(unittest.TestCase):
    """Loop 3996 (the seventy-third pass, DECISIONS 113 §1): `other` read 11.9
    minutes a session, 3965's session waited 82.7 minutes in it, and the table
    could not say whether it was a reading, a subagent, a packet or a sleep.
    The heads line names the waited call's head under the class — the
    backgrounded call whose output the wake read, else the last call."""

    def trace(self):
        t0 = '2026-11-08T09:56:00.000Z'
        t1 = '2026-11-08T10:14:00.000Z'
        t2 = '2026-11-08T10:30:00.000Z'
        t3 = '2026-11-08T10:40:00.000Z'
        return {'transactions': [
            {'ts': t0, 'kind': 'prompt', 'requests': [req(t0)],
             'instructions': [bg('2026-11-08T09:56:01.000Z',
                                 'RON_INSTALL=~/game zsh tools/check.sh test --release -p rondata --lib > '
                                 '$CLAUDE_JOB_DIR/tmp/full.log 2>&1', t0, 'bxiul41l3')]},
            # The wake reads the task's output file, stamped with the wake
            # request's own ts: the waited call is the backgrounded suite,
            # not the `cat` (6532f55d's 17.6 minutes).
            {'ts': t1, 'kind': 'prompt', 'requests': [req(t1)],
             'instructions': [ins(t1, 'cat /private/tmp/claude-501/x/s/tasks/bxiul41l3.output | head -60', t1),
                              ins('2026-11-08T10:14:05.000Z',
                                  'export PATH=/x:$PATH; cd /a && T=$CLAUDE_JOB_DIR/tmp; '
                                  'python3 -I tools/cant_reach_pins.py $T/full.log', t1)]},
            {'ts': t2, 'kind': 'prompt', 'requests': [req(t2)],
             'instructions': [{'tool': 'Write', 'input': '{"file_path":"/x/golden180.md","content":"## 180."}',
                               'ts': '2026-11-08T10:30:01.000Z', 'ms': 13, 'error': False, 'result': '',
                               'requestId': t2}]},
            {'ts': t3, 'kind': 'prompt', 'requests': [req(t3)], 'instructions': []},
        ]}

    def test_the_heads_name_what_the_other_minutes_waited_on(self):
        row = tranche.fold(self.trace(), gap_s=90, deep_k=300)
        self.assertEqual(row['on'], {'other': 44.0})
        self.assertEqual(row['other_heads'], {'zsh check.sh': 18.0, 'python3 cant_reach_pins.py': 16.0,
                                              'Write': 10.0})

    def test_a_peek_at_a_running_task_does_not_hide_it_from_the_wake_that_reads_it(self):
        # d9275b73's 13.5 minutes read as `Edit`: the lane `tail`ed the
        # suite's output file mid-run, then edited, then ended its turn; the
        # wake's `cat` of the same file names the suite.
        t0, t1, t2 = '2026-11-08T03:30:00.000Z', '2026-11-08T03:31:40.000Z', '2026-11-08T03:45:31.000Z'
        out = '/private/tmp/claude-501/x/s/tasks/bix1abc.output'
        trace = {'transactions': [
            {'ts': t0, 'kind': 'prompt', 'requests': [req(t0)],
             'instructions': [bg('2026-11-08T03:30:01.000Z', 'zsh tools/check.sh test --release', t0, 'bix1abc')]},
            {'ts': t1, 'kind': 'meta', 'requests': [req(t1)],
             'instructions': [ins('2026-11-08T03:31:45.000Z', 'tail -5 ' + out, t1),
                              {'tool': 'Edit', 'input': '{"file_path":"docs/SUPPLY.md"}',
                               'ts': '2026-11-08T03:31:55.000Z', 'requestId': t1}]},
            {'ts': t2, 'kind': 'meta', 'requests': [req(t2)],
             'instructions': [ins(t2, 'cat ' + out, t2)]},
        ]}
        row = tranche.fold(trace, gap_s=90, deep_k=300)
        self.assertEqual(row['other_heads'], {'zsh check.sh': 15.5})

    def test_a_large_other_prints_its_heads_line_and_a_small_one_does_not(self):
        row = tranche.fold(self.trace(), gap_s=90, deep_k=300)
        self.assertEqual(tranche.other_heads_line(row),
                         '         other: zsh check.sh 18.0 · python3 cant_reach_pins.py 16.0 · Write 10.0')
        # Under the 10-minute floor and under the gate's: no line.
        small = {'on': {'gate': 30.0, 'other': 4.0}, 'other_heads': {'cat': 4.0}}
        self.assertIsNone(tranche.other_heads_line(small))
        # At or above the gate's and over the 2-minute floor: a line.
        self.assertEqual(tranche.other_heads_line({'on': {'gate': 3.0, 'other': 3.0}, 'other_heads': {'git': 3.0}}),
                         '         other: git 3.0')
        self.assertIsNone(tranche.other_heads_line({'on': {'other': 1.9}, 'other_heads': {'git': 1.9}}))

    def test_a_command_s_head(self):
        for command, head in [('export A=1; cd /x && T=$J/tmp; grep -c x y', 'grep'),
                              ('RON_INSTALL=~/g zsh tools/check.sh test --release', 'zsh check.sh'),
                              ("python3 - <<'PYEOF'\np=1\nPYEOF", 'python3 -'),
                              ('python3 -I tools/nations_census.py --check', 'python3 nations_census.py'),
                              ('uv run tools/emu/callfn.py 0x1', 'uv callfn.py'),
                              ('/usr/bin/git -C . log', 'git'),
                              ('sleep 600 && tail x', 'sleep')]:
            self.assertEqual(tranche.head_of({'tool': 'Bash', 'input': json.dumps({'command': command})}),
                             head, command)
        # A loop's head is its body's first command.
        self.assertEqual(tranche.head_of({'tool': 'Bash', 'input': json.dumps(
            {'command': 'for f in a b; do cargo test -q $f; done'})}), 'cargo')
        # `lore trace --head 200` cuts the input mid-string: the prefix still names the head.
        cut = json.dumps({'command': 'RON_INSTALL=~/code/fun/attrition/game zsh tools/check.sh test ' + 'x' * 300})
        self.assertEqual(tranche.head_of({'tool': 'Bash', 'input': cut[:200]}), 'zsh check.sh')
        self.assertEqual(tranche.head_of({'tool': 'Bash', 'input': '{"command":"cd /a && python3 -I tools/x.py \\"q'}),
                         'python3 x.py')
        self.assertEqual(tranche.head_of({'tool': 'Monitor', 'input': '{}'}), 'Monitor')
        self.assertEqual(tranche.head_of(None), '?')


if __name__ == '__main__':
    unittest.main()


class Captures(unittest.TestCase):
    RUNS = (
        "## run981 — the battery's replacement lobby (2026-10-19, item 3347, `att-3347`)\n"
        "text\n"
        "## run981b — the same lobby, the draw stream (2026-10-19, item 3347, `att-3347`)\n"
        "## run999c — the American word (2026-10-20, item 3374, `att-3374`)\n"
        "## run1 — an old capture with no item\n"
    )

    def test_a_lane_s_captures_are_the_runs_sections_booked_under_its_item(self):
        # Loop 3352: the pass reads captures a landing; nobody counted them.
        self.assertEqual(tranche.captures_of(3347, self.RUNS), ["run981", "run981b"])
        self.assertEqual(tranche.captures_of(3374, self.RUNS), ["run999c"])
        self.assertEqual(tranche.captures_of(3345, self.RUNS), [])


class TheHandoff(unittest.TestCase):
    """The fifty-fourth pass (Loop 3564): the drain's handoff said one
    `DEEP` where three lanes crossed 300 k, and the pass before it said no
    red gate over two. The counts are the tables': `--base <sha>` reads the
    tranche's items off its `Book` commits and prints the handoff's line."""

    SUBJECTS = ['Book 3580 and drain the fifty-fourth tranche at twenty: the Chinese word',
                'Merge worktree-att-3580 (7 commits)',
                'Book 3578: the Korean word 4350 -> 5639 of 5639, whole',
                "Queue: 3562's lobby is an Ancient start capped at Classical",
                'Book 3573: the Persian Medieval twin run1216 whole', 'Merge worktree-att-3573 (3 commits)']

    def test_the_items_are_the_book_commits_oldest_first(self):
        self.assertEqual(tranche.booked_items(self.SUBJECTS), [3573, 3578, 3580])

    def test_a_commit_that_books_two_items_is_two_landings(self):
        # The fifty-ninth pass: `Book 3674 and 3690` and `Book 3677 and 3678`
        # each read as one item, and the drain's handoff line priced 22
        # sessions where 26 had run — 25.54 USD and 164 waiting minutes
        # unread. "and drain" is not an item.
        subjects = ['Book 3674 and 3690: the Greeks whole, the Americans 5678 -> 5701',
                    'Merge worktree-att-3690 (13 commits)',
                    'Book 3677 and 3678: the German Industrial word 4323 -> 4348',
                    'Book 3666, 3667 and drain the tranche at twenty']
        self.assertEqual(tranche.booked_items(subjects), [3666, 3667, 3677, 3678, 3674, 3690])

    def test_an_item_a_booking_only_minted_is_counted_once_where_it_landed(self):
        # Loop 3715 (the sixtieth pass): a residue booking is titled `Book
        # <landed> and <minted>` (`Book 3702 and 3709`, then `Book 3709 and
        # 3712`), and 3709, 3710, 3711 and 3712 each appeared under two
        # bookings with their sessions summed twice — the drain's line read
        # 177.78 USD over 29 sessions where 23 had run. An item's sessions
        # are its own however many bookings name it: once, where it landed.
        subjects = ['Book 3712 and 3714: the Mongol word 3989 -> 4220',
                    'Merge worktree-att-3712 (6 commits)',
                    'Book 3709 and 3712: the owner-9 chain moves two words',
                    'Merge worktree-att-3709 (9 commits)',
                    'Book 3702 and 3709: the Mongols part at 2595']
        self.assertEqual(tranche.booked_items(subjects), [3702, 3709, 3712, 3714])

    def test_a_booking_s_landing_count_is_not_an_item(self):
        # The sixty-ninth tranche titled its bookings `Book 3920, 19 of 20
        # from 02f72df7d: …`, and the list read `3920, 19` as two items:
        # the drain's table summed items 1 to 19's wells and read 38
        # landings over 399 sessions where 19 had landed.
        subjects = ['Book 3920, 19 of 20 from 02f72df7d: chapter 117 closed at 900',
                    'Merge worktree-att-3920 (9 commits)',
                    'Book 3923, 18 of 20 from 02f72df7d: the blind second reading',
                    'Book 3674 and 3690, 7 of 20 from 1234abcd: two co-booked lanes']
        self.assertEqual(tranche.booked_items(subjects), [3674, 3690, 3923, 3920])

    def test_the_line_counts_landings_deep_lanes_waiting_and_price(self):
        rows = [{'item': 3567, 'session': 'a', 'peak_k': 328, 'price': 11.91, 'waiting_min': 21.0},
                {'item': 3567, 'session': 'b', 'peak_k': 146, 'price': 2.75, 'waiting_min': 90.9},
                {'item': 3572, 'session': 'c', 'peak_k': 115, 'price': 1.60, 'waiting_min': 23.9}]
        self.assertEqual(tranche.handoff_line(rows, 300),
                         'handoff: 2 landings · one DEEP (3567 at 328 k) · 67.9 minutes waiting a landing · '
                         '8.13 USD a landing (16.26 over 3 sessions)')
        self.assertEqual(tranche.handoff_line(rows[2:], 300),
                         'handoff: 1 landing · no DEEP · 23.9 minutes waiting a landing · '
                         '1.60 USD a landing (1.60 over 1 session)')


if __name__ == '__main__':
    unittest.main()
