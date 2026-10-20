"""`tools/tranche.py`: a tranche's workers folded from `lore trace` (the
twenty-first pass, 2026-09-30).

The nineteenth pass measured a landing's wall clock by hand, the twentieth
by a scratch script, the twenty-first is the third reach. The fixture is
one worker's shape: a turn that backgrounds a gate and ends, the gap, the
re-invocation that reads the log — the gap is waiting, and its class is the
gate, not the `tail` that came last.
"""
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
