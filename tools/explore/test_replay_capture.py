import gzip
from pathlib import Path
import struct
import tempfile
import unittest

from autostart_receipt import receipt
import unattended_capture as runner


class ReplayCaptureTest(unittest.TestCase):
    def test_playback_lifecycle_is_distinct_from_solo(self):
        rows = [(0x544E4F52, 2, 0x400000, 0, 0, 0, 0, 0),
                (5, 175, 1, 0, 0, 0, 0, 0), (5, 170, 13, 1, 0, 0, 0, 0),
                (2, 0, 0, 0, 0, 0, 0, 0), (2, 1, 0, 0, 0, 0, 0, 1),
                (5, 170, 21, 2, 0, 0, 0, 0)]
        pack = lambda rs: b''.join(struct.pack('<8I', *r) for r in rs)
        self.assertTrue(receipt(pack(rows), 1, 0, playback=True)['lifecycle_verified'])
        with self.assertRaises(ValueError): receipt(pack(rows), 1, 0)
        for index, replacement in [(2, (5, 170, 1, 1, 0, 0, 0, 0)),
                                   (4, (2, 2, 0, 0, 0, 0, 0, 2)),
                                   (5, (5, 174, 4, 0, 0, 0, 0, 0))]:
            bad = rows[:]; bad[index] = replacement
            with self.assertRaises(ValueError): receipt(pack(bad), 1, 0, playback=True)
        with self.assertRaises(ValueError): receipt(pack(rows[:-1]), 1, 0, playback=True)
        with self.assertRaises(ValueError): receipt(pack(rows), 1, 6, playback=True)

    def test_header_identity_raw_and_gzip_and_truncation(self):
        version = '(Version: test)'.encode('utf-16le')
        data = b'\x16\x42' + struct.pack('<I', len(version)//2) + version
        data += struct.pack('<6I', 1, 340513920, 0, 0, 0, 0) + bytes([0, 7] + [0]*28)
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)/'input.rcx'
            for encoded in (data, gzip.compress(data)):
                path.write_bytes(encoded)
                self.assertEqual(runner.recording_identity(path), (340513920, 7))
            for bad in (data[:10], b'\0\0'+data[2:], data[:-1]):
                path.write_bytes(bad)
                with self.assertRaises(ValueError): runner.recording_identity(path)

    def test_playback_log_name_prevents_false_retake(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp)
            (output/'gamelog_p.txt').write_text('BEGIN FRAME 1\n')
            self.assertEqual(runner.capture_log(output, True).name, 'gamelog_p.txt')
            self.assertFalse(runner.stalled_before_frame_zero(runner.capture_log(output, True)))
            self.assertTrue(runner.stalled_before_frame_zero(runner.capture_log(output, False)))


if __name__ == '__main__': unittest.main()
