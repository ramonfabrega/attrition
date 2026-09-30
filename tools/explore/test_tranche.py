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
                             ('zsh tools/gamelog/waitrun.sh /tmp/ron-runs/viadriver-1.log', 'capture'),
                             ('cat $J/tmp/chain1221.log | tail -5', 'capture'),
                             ('zsh tools/memcap.sh 16 cargo test --release', 'suite'),
                             ('uv run tools/emu/callfn.py', 'emulator'),
                             ('sed -n 1,5p docs/AI.md', 'other')]:
            self.assertEqual(tranche.classify({'input': '{"command":"%s"}' % command}), cls, command)
        self.assertEqual(tranche.classify({'input': '{"to":"attrition","message":"1291 status"}'}), 'status')
        self.assertEqual(tranche.classify(None), 'other')


if __name__ == '__main__':
    unittest.main()
