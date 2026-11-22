"""`tools/gates.py`: a steering pass's table of a tranche's booking
gates, read off their logs (the forty-second pass). Three passes wrote
the same probe by hand — whole or scoped, minutes, the admission and
host-load receipts, the verdict — and CLAUDE.md's third-use rule says
it graduates. Red first: there was no such tool.
"""
import datetime
import os
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import gates  # noqa: E402

SCOPED = """Gate pid: 10718 — kill this gate by its pid, never by `pkill -f` on a name every lane shares (parked 1713)
Booking gate scoped to a5d76b6b: every timed test — a merge touches 3 path(s) that can move the pins (crates/sim/src/economy/cost.rs, …)
test_one ... ok
Gate steps: 2 of 6 ran (offline, clippy); not reached: fmt, survey, release, guard
Admission wait: 999.0 s over 1 leases, longest 999.0 s
Host load: 1.0 at start, 99.0 at end, on 16 cores
Test summary: fixture FAILED 0 passed / 1 failed
  failed: a_fixture_s_own_decoy
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
Test summary: sim ok 3 passed
Gate minutes: offline 0.3, clippy 1.9, fmt 0.0, survey 0.2, release 5.1, guard 0.6
Gate steps: 6 of 6 ran (offline, clippy, fmt, survey, release, guard)
Admission wait: 180.0 s over 967 leases, longest 57.2 s
Host load: 9.6 at start, 24.0 at end, on 16 cores
exit 0
"""
WHOLE_RED = """Gate pid: 91737 — kill this gate by its pid, never by `pkill -f` on a name every lane shares (parked 1713)
test_one ... ok
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
Test summary: rondata FAILED 2 passed / 1 failed
  failed: diff::battery::battery_captures_are_complete_and_say_their_lobby
Gate steps: 5 of 6 ran (offline, clippy, fmt, survey, release); not reached: guard
Admission wait: 64.6 s over 966 leases, longest 23.9 s
Host load: 6.9 at start, 19.7 at end, on 16 cores
"""
NARROW = """Gate pid: 1 — kill this gate by its pid
Booking gate scoped to dab91773: 12 timed tests left to the next whole gate
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
Gate steps: 6 of 6 ran (offline, clippy, fmt, survey, release, guard)
Admission wait: 4.0 s over 975 leases, longest 0.0 s
Host load: 5.5 at start, 8.2 at end, on 16 cores
"""


class TheTable(unittest.TestCase):
    def test_each_log_is_one_row_and_the_totals_are_summed(self):
        # The gate runs the tools' own tests, whose fixtures print receipt
        # lines; the first run of this tool on the forty-second tranche's
        # logs read a fixture's `Gate steps: 2 of 6` and `  failed:` decoys
        # as the gate's. The last receipt is the gate's.
        with tempfile.TemporaryDirectory() as d:
            for name, text, start in (('gate-b4ece7b6.log', SCOPED, 1_000_000), ('gate-e183b2e5.log', WHOLE_RED, 1_001_000),
                                      ('gate-06aa7685.log', NARROW, 1_002_000)):
                f = Path(d) / name
                f.write_text(text)
                os.utime(f, (start + 500, start + 500))
            rows = gates.rows(sorted(Path(d).glob('gate-*.log')))
            self.assertEqual([r['name'] for r in rows], ['gate-06aa7685', 'gate-b4ece7b6', 'gate-e183b2e5'])
            scoped = next(r for r in rows if r['name'] == 'gate-b4ece7b6')
            self.assertEqual(scoped['scope'], 'scoped, widened')
            self.assertEqual(scoped['verdict'], 'green')
            self.assertEqual(scoped['admission_s'], 180.0)
            self.assertEqual(scoped['longest_s'], 57.2)
            self.assertEqual((scoped['load_start'], scoped['load_end'], scoped['cores']), (9.6, 24.0, 16))
            self.assertEqual(scoped['steps'], '6 of 6')
            self.assertEqual(scoped['release_min'], 5.1)
            red = next(r for r in rows if r['name'] == 'gate-e183b2e5')
            self.assertEqual(red['scope'], 'whole')
            self.assertEqual(red['verdict'], 'red: diff::battery::battery_captures_are_complete_and_say_their_lobby')
            self.assertEqual(red['steps'], '5 of 6')
            self.assertIsNone(red['release_min'])
            narrow = next(r for r in rows if r['name'] == 'gate-06aa7685')
            self.assertEqual(narrow['scope'], 'scoped, 12 left')
            totals = gates.totals(rows)
            self.assertEqual(totals['gates'], 3)
            self.assertEqual(totals['whole'], 1)
            self.assertEqual(totals['widened'], 1)
            self.assertEqual(totals['red'], 1)
            self.assertAlmostEqual(totals['admission_s'], 248.6)
            self.assertEqual(totals['longest_s'], 57.2)
            self.assertEqual(totals['past_cores'], 2)
            text = gates.render(rows, totals)
            self.assertIn('gate-b4ece7b6', text)
            self.assertIn('3 gates: 1 whole, 2 scoped (1 widened), 1 red', text)
            self.assertIn('admission 248.6 s summed, longest 57.2 s; 2 ended past the cores', text)

    def test_a_gate_s_minutes_are_from_its_own_start_line_not_the_file_s_birth(self):
        # `gate_run.sh` opens its log with `>`, which keeps the birth time
        # of a file already there: gate-4159r12b and -r12c (the tranche of
        # 2026-11-16) were names written once the afternoon before, and
        # the table read 443.1 and 481.9 minutes on runs of 43.5 and 33.8.
        # The log now says when it began, and the table believes the log.
        with tempfile.TemporaryDirectory() as d:
            f = Path(d) / 'gate-reused.log'
            f.write_text('gate exit 93\n')
            os.utime(f, (1_000_000, 1_000_000))  # born the afternoon before
            end = 1_000_000 + 26_586
            f.write_text(f'Gate tree: /x at abc\nGate start: {end - 1200} 1970-01-12 00:00:00 +0000\n' + SCOPED)
            os.utime(f, (end, end))
            [row] = gates.rows([f])
            self.assertEqual(row['minutes'], 20.0)
            self.assertEqual(row['started'], datetime.datetime.fromtimestamp(end - 1200).strftime('%H:%M'))

    def test_a_log_without_a_start_line_is_as_long_as_its_steps(self):
        # The logs written before the start line: the steps' own minutes
        # (`Gate minutes:`, the last one — fixtures print decoys) are the
        # run, and its clock step is a column, because the clock's drained
        # lease waited 24.9 of gate-4159r12b's 43.5 minutes and no column
        # showed it.
        with tempfile.TemporaryDirectory() as d:
            f = Path(d) / 'gate-old.log'
            f.write_text('gate exit 93\n')
            os.utime(f, (1_000_000, 1_000_000))
            f.write_text('Gate tree: /x at abc\nGate pid: 5\n'
                         'Gate minutes: offline 0.3, release 5.1\n'
                         'Gate minutes: pins 0.0, offline 2.1, clippy 0.2, fmt 0.0, survey 0.1, guard 1.5, '
                         'merge pins 1.4, release 13.2, clock 25.0\n'
                         'Gate steps: 9 of 9 ran (pins, offline, clippy, fmt, survey, guard, merge pins, release, clock)\n'
                         'gate exit 0\n')
            os.utime(f, (1_026_586, 1_026_586))
            [row] = gates.rows([f])
            self.assertEqual(row['minutes'], 43.5)
            self.assertEqual(row['clock_min'], 25.0)
            self.assertEqual(row['release_min'], 13.2)
            self.assertIn(' 25.0 ', gates.render([row], gates.totals([row])))

    def test_a_log_with_no_receipt_is_a_row_that_says_so(self):
        with tempfile.TemporaryDirectory() as d:
            f = Path(d) / 'gate-dead.log'
            f.write_text('Gate pid: 5\ntest_one ... ok\n')
            [row] = gates.rows([f])
            self.assertEqual(row['verdict'], 'no receipt')
            self.assertIsNone(row['admission_s'])

    def test_a_gate_the_driver_stopped_is_stopped_whatever_its_fixtures_printed(self):
        # gate-4524.log and gate-4525.log, 2026-11-21: the driver stopped two
        # booking gates to book three landings under one, each log ends
        # `gate exit 143`, and the table read both `9 of 9 … green` off the
        # receipts the tools' own fixtures print in the offline step.
        with tempfile.TemporaryDirectory() as d:
            f = Path(d) / 'gate-stopped.log'
            f.write_text(SCOPED.replace('exit 0\n', '') + 'Driver: 18 landings since the last pass\ngate exit 143\n')
            [row] = gates.rows([f])
            self.assertEqual(row['verdict'], 'stopped (exit 143)')
            self.assertIsNone(row['steps'])
            self.assertIsNone(row['release_min'])
            total = gates.totals([row])
            self.assertEqual((total['red'], total['stopped']), (0, 1))
            self.assertIn('1 stopped', gates.render([row], total))
            # A gate that ran to its end and was red exits non-zero too, and stays red.
            g = Path(d) / 'gate-red.log'
            g.write_text(WHOLE_RED + 'gate exit 1\n')
            self.assertTrue(gates.rows([g])[0]['verdict'].startswith('red: '))

    def test_a_red_with_many_failures_names_three_and_counts_the_rest(self):
        # gate-4435.log, 2026-11-19: 282 widenings red on one merge, and
        # the table printed every name on one row of 30 kB (the pass of
        # 2026-11-20). The row names three; `failed` keeps them all.
        names = [f'diff::second::tests::run{n}_s_word_frame_is_widened_whole' for n in range(282)]
        text = WHOLE_RED.replace(
            '  failed: diff::battery::battery_captures_are_complete_and_say_their_lobby\n',
            ''.join(f'  failed: {n}\n' for n in names))
        with tempfile.TemporaryDirectory() as d:
            f = Path(d) / 'gate-wide.log'
            f.write_text(text)
            [row] = gates.rows([f])
            self.assertEqual(row['verdict'], 'red: ' + ', '.join(names[:3]) + ' and 279 more')
            self.assertEqual(len(row['failed']), 282)
            self.assertEqual(gates.totals([row])['red'], 1)


if __name__ == '__main__':
    unittest.main()
