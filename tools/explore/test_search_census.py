#!/usr/bin/env python3
import tempfile
from pathlib import Path
import struct
import unittest
from search_census import scan, records


def info(tag, *data, frame=7):
    return (5, tag, *data, *((0,)*(5-len(data))), frame)


def fixture():
    rows = [info(136, 0, 1, 64), info(12, 0x283770), (2, 7, 0, 0, 0, 0, 0, 0),
            (7, 0, 123, 456, 48, 0, 0, 7), (8, 0, 0xffffffff, 0, 0, 0, 0, 7),
            info(130, 1, 100, 300, 1)]
    rows += [info(131, 1, i, 100+i, 2, 200+i) for i in range(5)]
    rows += [info(132, 1, i, 300+i, 64, 3) for i in range(7)]
    return rows+[info(134, 1, 5, 7)]


class CensusTests(unittest.TestCase):
    def test_complete(self):
        report = scan(fixture())
        self.assertEqual(report['status'], 'witnesses')
        self.assertEqual(report['suspension_returns'], 1)

    def test_every_missing_receipt_fails(self):
        rows = fixture()
        for i in range(len(rows)):
            with self.subTest(i=i), self.assertRaises(ValueError):
                scan(rows[:i]+rows[i+1:])

    def test_wrong_frame_and_sequence(self):
        for index in (2, 7):
            rows = fixture()
            r = list(rows[6]); r[index] += 1; rows[6] = tuple(r)
            with self.assertRaises(ValueError):
                scan(rows)

    def test_health_and_read_failures(self):
        for tag in (2, 5, 14, 133):
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                scan(fixture()+[info(tag)])

    def test_negative_run_is_not_witness(self):
        self.assertEqual(scan(fixture()[:2])['status'], 'no_game_frames')
        self.assertEqual(scan(fixture()[:2]+[(2, 7, 0, 0, 0, 0, 0, 0)])['status'], 'no_suspension_witness')

    def test_archive_returns_are_not_metadata(self):
        report = scan(fixture()[3:5], experimental=False)
        self.assertEqual(report['suspension_returns'], 1)
        self.assertEqual(report['status'], 'suspension_returns_without_metadata')

    def test_cap_is_explicit_and_not_a_missing_witness(self):
        template = fixture()
        rows = template[:3]
        for sequence in range(1, 65):
            for row in template[3:]:
                if row[0] == 5:
                    row = (*row[:2], sequence, *row[3:])
                rows.append(row)
        rows.extend(template[3:5])
        with self.assertRaises(ValueError):
            scan(rows)
        rows.append(info(135, 65, 64))
        report = scan(rows)
        self.assertEqual((report['status'], len(report['events'])), ('capped', 64))
        with self.assertRaises(ValueError):
            scan(rows+[info(135, 65, 64)])

    def test_legacy_header_requires_archive_mode(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)/'trace'
            p.write_bytes(struct.pack('<8I', 0x544e4f52, 1, 0x400000, 0, 0, 0, 0, 0))
            self.assertEqual(list(records(p, archive=True)), [])
            with self.assertRaises(ValueError):
                list(records(p))

    def test_truncation_and_identity(self):
        header = struct.pack('<8I', 0x544e4f52, 2, 0x400000, 0, 0, 0, 0, 0)
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)/'trace'
            for data in (b'', header[:-1], header+b'x', bytes(32)):
                p.write_bytes(data)
                with self.assertRaises(ValueError):
                    list(records(p))


if __name__ == '__main__':
    unittest.main()
