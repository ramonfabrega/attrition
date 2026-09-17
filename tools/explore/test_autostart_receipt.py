import struct
import unittest
import tempfile
from pathlib import Path
from autostart_receipt import receipt, receipt_file


class ReceiptTest(unittest.TestCase):
    def setUp(self):
        def row(*words): return tuple(words) + (0,) * (8-len(words))
        self.rows = [row(0x544E4F52,2,0x400000), row(5,175,1), row(5,170,1,1),
                     row(5,171,4096,0), row(5,172,4096,8192), row(5,173,4096),
                     row(2,0), row(2,1,0,0,0,0,0,1), row(2,2,0,0,0,0,0,2), row(5,170,21,2)]

    def pack(self, rows): return b''.join(struct.pack('<8I',*r) for r in rows)

    def test_complete_lifecycle_is_not_fidelity(self):
        result=receipt(self.pack(self.rows),2,0)
        self.assertTrue(result['lifecycle_verified'])
        self.assertFalse(result['map_verified'])
        self.assertFalse(result['fidelity_verified'])

    def test_streamed_chunk_boundary_and_partial_record(self):
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'trace'
            padding=[(1,0,0,0,0,0,0,0)]*4096
            data=self.pack(self.rows[:1]+padding+self.rows[1:])
            path.write_bytes(data)
            self.assertEqual(receipt_file(path,2,0),receipt(data,2,0))
            path.write_bytes(data+b'x')
            with self.assertRaises(ValueError): receipt_file(path,2,0)

    def test_transport_faults_refuse_in_memory_and_streamed(self):
        bad_frames = self.rows[:]
        bad_frames[7] = (2,1,0,0,0,0,0,2)
        cases = [(bad_frames, 'FRAME fields disagree')]
        for count in (1, 0xffffffff):
            for index in (1, len(self.rows)):
                rows = self.rows[:index] + [(5,14,count,0,0,0,0,0)] + self.rows[index:]
                cases.append((rows, 'dropped records'))
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'trace'
            for rows, message in cases:
                with self.subTest(message=message, rows=rows):
                    data = self.pack(rows)
                    with self.assertRaisesRegex(ValueError, message): receipt(data,2,0)
                    # Put the fault beyond a streamed chunk boundary too.
                    padding = self.pack([(1,0,0,0,0,0,0,0)]*2048)
                    path.write_bytes(data[:32]+padding+data[32:])
                    with self.assertRaisesRegex(ValueError, message): receipt_file(path,2,0)

    def test_zero_loss_notice_is_valid(self):
        rows = self.rows + [(5,14,0,0,0,0,0,0)]
        self.assertEqual(receipt(self.pack(rows),2,0), receipt(self.pack(self.rows),2,0))

    def test_every_missing_record_refuses(self):
        for i in range(len(self.rows)):
            with self.subTest(i=i), self.assertRaises(ValueError):
                receipt(self.pack(self.rows[:i]+self.rows[i+1:]),2,0)

    def test_crash_and_truncation_refuse(self):
        for code in (1,5,137):
            with self.assertRaises(ValueError): receipt(self.pack(self.rows),2,code)
        with self.assertRaises(ValueError): receipt(self.pack(self.rows)[:-1],2,0)
        for kind in (174,176,177):
            with self.assertRaises(ValueError):
                receipt(self.pack(self.rows+[(5,kind,0,0,0,0,0,0)]),2,0)

    def test_repeat_reorder_and_wrong_instance_refuse(self):
        for index in (2,3,4,6):
            with self.assertRaises(ValueError):
                receipt(self.pack(self.rows[:index]+[self.rows[index]]+self.rows[index:]),2,0)
        rows=self.rows[:];rows[4]=(5,172,4097,0,0,0,0,0)
        with self.assertRaises(ValueError): receipt(self.pack(rows),2,0)
        rows=self.rows[:];rows[3],rows[4]=rows[4],rows[3]
        with self.assertRaises(ValueError): receipt(self.pack(rows),2,0)


if __name__ == '__main__': unittest.main()
