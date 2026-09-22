import io
import struct
import unittest
from payload_refs import find_refs

class PayloadRefsTest(unittest.TestCase):
    def test_aligned_words_and_file_offsets(self):
        word = struct.pack('<I', 0x12345678)
        data = b'---'+word+b'abcd'+b'x'+word+bytes(7)
        result = find_refs(io.BytesIO(data), [dict(base='0x1000', bytes=len(data)-3, file_offset=3)], 0x12345678)
        self.assertFalse(result['truncated'])
        self.assertEqual([r['address'] for r in result['hits']], ['0x1000'])

    def test_limit_is_explicit_and_short_reads_fail(self):
        stream = io.BytesIO(bytes(16))
        spans = [dict(base='0x1000', bytes=16, file_offset=0)]
        self.assertTrue(find_refs(stream, spans, 0, 1)['truncated'])
        self.assertFalse(find_refs(stream, spans, 0, 4)['truncated'])
        with self.assertRaisesRegex(ValueError, 'short'):
            find_refs(io.BytesIO(bytes(8)), spans, 0)

    def test_chunk_boundary_and_tail(self):
        data = bytes(1024*1024-4)+struct.pack('<I', 42)+struct.pack('<I', 42)
        result = find_refs(io.BytesIO(data), [dict(base='0x1000', bytes=len(data), file_offset=0)], 42)
        self.assertEqual([r['address'] for r in result['hits']], [hex(0x1000+1024*1024-4), hex(0x1000+1024*1024)])
        self.assertEqual(result['hits'][-1]['words'], [42])

if __name__ == '__main__':
    unittest.main()
