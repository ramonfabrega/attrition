import struct
import tempfile
import unittest
from pathlib import Path
from startup_evidence import inspect


def record(*values):
    return struct.pack('<8I', *(values + (0,) * (8-len(values))))


class StartupEvidenceTests(unittest.TestCase):
    def read(self, data):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'trace'
            path.write_bytes(data)
            return inspect(path)

    def test_partial_fault_evidence_is_not_success(self):
        data = record(0x544e4f52, 2, 0x400000) + record(5,175,1)
        data += record(5,176,0x12345678)*10 + b'partial'
        result = self.read(data)
        self.assertTrue(result['diagnostic_only'])
        self.assertNotIn('success', result)
        self.assertEqual(result['driver_events'], {'175':1, '176':10})
        self.assertEqual(result['fault_addresses'], ['0x12345678']*8)
        self.assertEqual(len(result['fault_contexts']), 8)
        self.assertEqual(result['fault_contexts'][0]['ip'], 0x12345678)
        self.assertEqual(result['trailing_bytes'], 7)
        self.assertEqual(result['frames_observed'], 0)

    def test_unknown_header_does_not_decode_witnesses(self):
        result = self.read(record(0x544e4f52,99,0x400000)+record(5,170))
        self.assertFalse(result['header_supported'])
        self.assertEqual(result['driver_events'], {})

    def test_startup_witnesses_are_bounded_and_do_not_certify_success(self):
        result = self.read(record(0x544e4f52,2,0x400000) + record(5,180,1,0,0x20070,0,0)*20)
        self.assertEqual(result['startup_event_count'], 20)
        self.assertEqual(len(result['startup_events']), 16)
        self.assertEqual(result['startup_events'][0], {'site':1,'phase':0,'version':0x20070,'flags':0,'result':0})
        self.assertNotIn('success', result)

    def test_counts_do_not_imply_frame_continuity(self):
        result = self.read(record(0x544e4f52,2,0x400000)+record(2,8)+record(2,3))
        self.assertEqual(result['frames_observed'], 2)
        self.assertEqual(result['last_frame'], 3)
        self.assertNotIn('lifecycle_verified', result)


if __name__ == '__main__':
    unittest.main()
