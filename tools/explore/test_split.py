"""`tools/trace/split.py`: a step R record past a piece's size, kept as
`rontrace.ranges`, `.1`, `.2`, … so the archive's compression takes it
(DECISIONS 149 §1; the pass of 2026-11-21, second: fourteen records, 48.8
GiB, stood uncompressed past `afsctool`'s 2^31 bytes).

(a) the pieces in order are the record, byte for byte, and `--check`'s
    figures are the whole file's;
(b) a record already in pieces, or one that fits a piece, is left alone;
(c) a record a take may still be writing is refused;
(d) pieces that do not read back as the record are removed and the record
    stands whole;
(e) `join` is the undo;
(f) `tools/trace/ranges.py` reads a split record as it reads the whole one;
(g) `tools/gamelog/filerun.py` splits the take's record when it files it.
"""
import hashlib
import io
import os
import struct
import sys
import tempfile
import time
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/trace'))
sys.path.insert(0, str(ROOT / 'tools/gamelog'))
import filerun  # noqa: E402
import ranges  # noqa: E402
import split  # noqa: E402


def record():
    """A record as `ranges.h` writes one: two sets, a range each."""
    out = struct.pack('<8I', ranges.MAGIC, ranges.VERSION, 0x500000, 0xff, 0, 0, 0, 0)
    for frame, data in ((-1, b'abc'), (0, b'wxyz')):
        out += ranges.SET.pack(ranges.SET_MAGIC, frame, 0, 0x53fb68, 1, 0, 1, len(data), *([1] * 16))
        out += ranges.ENTRY.pack(5, 0, 0, 0x808419, 0x1234, len(data))
        out += data + bytes((4 - (len(data) & 3)) & 3)
    return out


class Split(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.path = Path(self.tmp.name) / 'rontrace.ranges'
        self.bytes = bytes(range(256)) * 5 + b'tail'
        self.path.write_bytes(self.bytes)
        self.assertEqual(split.WHOLE, 2_000_000_000)
        self.assertLess(split.WHOLE, 1 << 31, "under afsctool's refusal")
        real, split.WHOLE = split.WHOLE, 0
        self.addCleanup(setattr, split, 'WHOLE', real)
        old = time.time() - 3600
        os.utime(self.path, (old, old))

    def names(self):
        return sorted(p.name for p in Path(self.tmp.name).iterdir())

    def test_the_pieces_in_order_are_the_record(self):
        done = split.split(self.path, piece=500, compress=None)
        self.assertEqual(self.names(), ['rontrace.ranges', 'rontrace.ranges.1', 'rontrace.ranges.2'])
        self.assertEqual([p.stat().st_size for p in split.pieces(self.path)], [500, 500, 284])
        self.assertEqual(b''.join(p.read_bytes() for p in split.pieces(self.path)), self.bytes)
        want = (len(self.bytes), hashlib.sha256(self.bytes).hexdigest())
        self.assertEqual((done['size'], done['sha256'], done['pieces']), (*want, 3))
        self.assertEqual(split.digest(split.pieces(self.path)), want)

    def test_a_record_of_whole_pieces_leaves_no_empty_one(self):
        self.path.write_bytes(self.bytes[:1000])
        old = time.time() - 3600
        os.utime(self.path, (old, old))
        split.split(self.path, piece=500, compress=None)
        self.assertEqual(self.names(), ['rontrace.ranges', 'rontrace.ranges.1'])

    def test_a_record_in_pieces_or_inside_one_is_left_alone(self):
        self.assertIsNone(split.split(self.path, piece=len(self.bytes), compress=None))
        self.assertEqual(self.names(), ['rontrace.ranges'])
        # Past a piece and not past what compresses whole: left in one file
        # (run1754's 1.3 GB gained nothing in two).
        self.assertIsNone(split.split(self.path, piece=500, compress=None, whole=len(self.bytes)))
        self.assertEqual(self.names(), ['rontrace.ranges'])
        split.split(self.path, piece=500, compress=None)
        before = {p.name: p.read_bytes() for p in Path(self.tmp.name).iterdir()}
        self.assertIsNone(split.split(self.path, piece=100, compress=None))
        self.assertEqual({p.name: p.read_bytes() for p in Path(self.tmp.name).iterdir()}, before)

    def test_a_record_a_take_may_be_writing_is_refused(self):
        os.utime(self.path, None)
        with self.assertRaisesRegex(split.Refused, 'a take may be writing it'):
            split.split(self.path, piece=500, compress=None)
        self.assertEqual(self.names(), ['rontrace.ranges'])
        self.assertIsNotNone(split.split(self.path, piece=500, compress=None, min_age=0))

    def test_pieces_that_do_not_read_back_leave_the_record_whole(self):
        def spoil(parts):
            data = bytearray(parts[1].read_bytes())
            data[7] ^= 1
            parts[1].write_bytes(data)
            return True
        with self.assertRaisesRegex(split.Refused, 'the record stands whole'):
            split.split(self.path, piece=500, compress=spoil)
        self.assertEqual(self.names(), ['rontrace.ranges'])
        self.assertEqual(self.path.read_bytes(), self.bytes)

    def test_a_run_that_died_leaves_parts_the_next_run_removes(self):
        Path(f'{self.path}.7.part').write_bytes(b'left by a run that died')
        split.split(self.path, piece=500, compress=None)
        self.assertEqual(self.names(), ['rontrace.ranges', 'rontrace.ranges.1', 'rontrace.ranges.2'])

    def test_join_is_the_undo(self):
        self.assertIsNone(split.join(self.path))
        split.split(self.path, piece=500, compress=None)
        self.assertEqual(split.join(self.path), (len(self.bytes), hashlib.sha256(self.bytes).hexdigest()))
        self.assertEqual(self.names(), ['rontrace.ranges'])
        self.assertEqual(self.path.read_bytes(), self.bytes)

    def test_the_command_checks_and_splits_the_archive_s_records(self):
        golden = Path(self.tmp.name) / 'golden'
        d = golden / 'run9' / 'map-7'
        d.mkdir(parents=True)
        os.replace(self.path, d / 'rontrace.ranges')
        lab = golden / 'lab' / 'deeper' / 'map-7'
        lab.mkdir(parents=True)
        (lab / 'rontrace.ranges').write_bytes(self.bytes)
        self.assertEqual(split.archive(golden), [d / 'rontrace.ranges'], 'one level down and no further')
        real, split.PIECE = split.PIECE, 500
        self.addCleanup(setattr, split, 'PIECE', real)
        out, sys.stdout = sys.stdout, io.StringIO()
        try:
            self.assertEqual(split.main(['--archive', '--golden', str(golden)]), 0)
            self.assertEqual(split.main(['--check', str(d / 'rontrace.ranges')]), 0)
            text = sys.stdout.getvalue()
        finally:
            sys.stdout = out
        self.assertIn(f'{len(self.bytes)} bytes in 3 pieces', text)
        self.assertIn(f'{len(self.bytes)} bytes, SHA-256 {hashlib.sha256(self.bytes).hexdigest()}, 3 piece(s)', text)


class TheReader(unittest.TestCase):
    def test_a_split_record_reads_as_the_whole_one(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'rontrace.ranges'
            path.write_bytes(record())

            def read():
                rec = ranges.Record(str(path))
                sets = list(rec.sets())
                return rec.size, sets, [rec.body(s) for s in sets]
            whole = read()
            self.assertEqual(len(whole[1]), 2)
            self.assertEqual(whole[2][1][0][5], b'wxyz')
            # 100 bytes a piece of 264: each cut falls inside a set's header.
            self.assertIsNotNone(split.split(path, piece=100, compress=None, min_age=0, whole=0))
            self.assertEqual(len(split.pieces(path)), 3)
            self.assertEqual(read(), whole)


class TheFiling(unittest.TestCase):
    def test_filing_a_take_splits_its_record(self):
        with tempfile.TemporaryDirectory() as tmp:
            base = Path(tmp)
            golden, logs = base / 'golden', base / 'Logs'
            logs.mkdir()
            d = golden / 'run1772' / 'map-7'
            d.mkdir(parents=True)
            (d / 'gamelog_p.txt').write_text(' MAP_STYLE 7\n')
            (d / 'rontrace.log').write_bytes(b'RONT' + bytes(28))
            (d / 'rontrace.ranges').write_bytes(record())
            real, split.PIECE = split.PIECE, 100
            self.addCleanup(setattr, split, 'PIECE', real)
            whole, split.WHOLE = split.WHOLE, 0
            self.addCleanup(setattr, split, 'WHOLE', whole)
            out = io.StringIO()
            filerun.file_run(golden, logs, 'run1772', 'sample', out=out)
            self.assertEqual(len(split.pieces(d / 'rontrace.ranges')), 3)
            self.assertEqual(b''.join(p.read_bytes() for p in split.pieces(d / 'rontrace.ranges')), record())
            self.assertIn('in 3 pieces', out.getvalue())
            # A dry run touches nothing, and a take with no record files as before.
            for p in split.pieces(d / 'rontrace.ranges'):
                p.unlink()
            out = io.StringIO()
            filerun.file_run(golden, logs, 'run1772', 'sample', force=True, out=out)
            self.assertNotIn('pieces', out.getvalue())


if __name__ == '__main__':
    unittest.main()
