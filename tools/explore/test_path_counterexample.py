#!/usr/bin/env python3
"""Reducer and worker failures tested without original files or an emulator."""
from pathlib import Path
import sys
import tempfile
import unittest
from path_counterexample import Worker, OracleFailure, decode, encode, mismatch, neighbors, reduce_case


class ReductionTest(unittest.TestCase):
    def test_reduces_payload_and_flags_with_certificate(self):
        case = ((0xffffffff, 100, 200, 255),)*8
        fails = lambda c: len(c) >= 2 and any(row[3] & 3 == 3 for row in c)
        result, stats = reduce_case(case, fails)
        self.assertEqual(result, ((0, 0, 0, 0), (0, 0, 0, 3)))
        self.assertFalse(any(fails(c) for c in neighbors(result)))
        self.assertEqual(stats['final_set_bits'], 2)
        self.assertEqual(reduce_case(case, fails), (result, stats))

    def test_chunk_deletion_crosses_single_deletion_barrier(self):
        result, _ = reduce_case(((0, 0, 0, 0),)*8, lambda c: len(c) >= 2 and len(c) % 2 == 0)
        self.assertEqual(len(result), 2)

    def test_agreement_is_not_a_counterexample(self):
        with self.assertRaisesRegex(ValueError, 'does not preserve'):
            reduce_case(((0, 0, 0, 0),), lambda _: False)

    def test_oracle_failure_is_not_minimized(self):
        def broken(_):
            raise OracleFailure('crashed')
        with self.assertRaisesRegex(OracleFailure, 'crashed'):
            reduce_case(((0, 0, 0, 0),), broken)

    def test_budget_exhaustion_is_not_a_minimality_certificate(self):
        with self.assertRaisesRegex(OracleFailure, 'budget exhausted'):
            reduce_case(((7, 7, 7, 7),)*2, lambda _: True, budget=1)

    def test_mismatch_kind_is_specific(self):
        self.assertEqual(mismatch((), ((0, 0, 0, 0),)), 'survivor_count')
        self.assertEqual(mismatch(((0, 0, 0, 0),), ((0, 1, 0, 0),)), 'y')
        self.assertIsNone(mismatch((), ()))

    def test_malformed_response_rejected(self):
        for line in ('', '1 0 0 0', '1 0 0 0 256', '1 -1 0 0 0', '0 surplus'):
            with self.assertRaises(OracleFailure):
                decode(line)


class WorkerTest(unittest.TestCase):
    def worker_file(self, root, body):
        path = Path(root)/'worker'
        path.write_text('#!'+sys.executable+'\nimport sys,time\n'+body)
        path.chmod(0o755)
        return path

    def test_persistent_round_trip(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = self.worker_file(tmp, 'for line in sys.stdin:\n print(line.strip(), flush=True)\n')
            with Worker(path) as worker:
                case = ((0xffffffff, 1, 2, 3),)
                self.assertEqual(worker.evaluate(case), case)
                self.assertEqual(worker.evaluate(()), ())
                self.assertEqual(worker.calls, 2)
                self.assertEqual(decode(encode(case)), case)

    def test_crash_timeout_malformed_and_extra_output_abort(self):
        cases = [('sys.exit(2)\n', 'without a complete response'),
                 ('time.sleep(60)\n', 'timeout'),
                 ('sys.stdin.readline(); print("bad", flush=True)\n', 'malformed'),
                 ('sys.stdin.readline(); print("0\\n0", flush=True)\n', 'unsolicited')]
        for body, message in cases:
            with self.subTest(message=message), tempfile.TemporaryDirectory() as tmp:
                with Worker(self.worker_file(tmp, body), timeout=0.15 if message == 'timeout' else 5) as worker:
                    with self.assertRaisesRegex(OracleFailure, message):
                        worker.evaluate(())


if __name__ == '__main__':
    unittest.main()
