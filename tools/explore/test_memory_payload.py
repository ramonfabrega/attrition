"""The actual C writer feeding the streaming Python reader; authored bytes only."""
import hashlib
import io
from pathlib import Path
import struct
import subprocess
import tempfile
import unittest
from memory_payload import decode, check_receipts, plan, CAP, CHUNK
from memory_inventory import decode as decode_inventory
from restore_context import decode_context
from test_restore_context import unit_packet
from test_restore_prefix import fixture

ROOT = Path(__file__).resolve().parent


class PayloadTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.prefix = fixture(2)
        cls.context_raw = unit_packet()[0]
        cls.context = decode_context(cls.context_raw, cls.prefix)
        with tempfile.TemporaryDirectory() as folder:
            p = Path(folder); (p/'context').write_bytes(cls.context_raw)
            subprocess.run(['cc', '-O2', '-Wall', '-Wextra', '-Werror', '-Wno-int-to-pointer-cast',
                            str(ROOT/'test_live_memory_payload.c'), '-o', str(p/'test')], check=True)
            subprocess.run([str(p/'test'), str(p/'context'), str(p/'payload')], check=True)
            cls.raw = (p/'payload').read_bytes()
            shifted = bytearray(cls.context_raw)
            struct.pack_into('<2I', shifted, 28, 0x2000000+CHUNK-96, 0x2000000+CHUNK)
            (p/'context').write_bytes(shifted)
            subprocess.run([str(p/'test'), str(p/'context'), str(p/'shifted')], check=True)
            cls.shifted_context = decode_context(shifted, cls.prefix)
            cls.shifted_raw = (p/'shifted').read_bytes()
        n = struct.unpack_from('<I', cls.raw, 36)[0]
        cls.inventory = cls.raw[64:64+n]

    def parse(self, raw=None):
        return decode(io.BytesIO(self.raw if raw is None else raw), self.inventory, self.prefix, self.context)

    def test_c_writer_and_streamed_body_agree(self):
        r = self.parse()
        self.assertEqual((r['range_count'], r['payload_bytes']), (3, 0x1c1000))
        self.assertEqual(r['sha256'], hashlib.sha256(self.raw).hexdigest())
        self.assertFalse(r['atomic_snapshot']); self.assertFalse(r['replay_closure_established'])
        self.assertEqual(r['world_pointer'], '0x1008000')
        class Limited(io.BytesIO):
            def read(self, n=-1):
                if not 0 <= n <= CHUNK: raise AssertionError('unbounded read')
                return super().read(n)
        self.assertEqual(decode(Limited(self.raw), self.inventory, self.prefix, self.context), r)

    def test_anchor_split_across_stream_chunks(self):
        n = struct.unpack_from('<I', self.shifted_raw, 36)[0]
        inv = self.shifted_raw[64:64+n]
        r = decode(io.BytesIO(self.shifted_raw), inv, self.prefix, self.shifted_context)
        span = r['spans'][-1]
        for delta in (-1, 0):
            bad = bytearray(self.shifted_raw); bad[span['file_offset']+CHUNK+delta] ^= 1
            with self.assertRaisesRegex(ValueError, 'anchor'):
                decode(io.BytesIO(bad), inv, self.prefix, self.shifted_context)

    def test_every_header_word_is_bound(self):
        for word in range(16):
            raw = bytearray(self.raw); raw[word*4] ^= 1
            with self.subTest(word=word), self.assertRaises(ValueError): self.parse(raw)

    def test_inventory_and_range_metadata_are_bound(self):
        raw = bytearray(self.raw); raw[64+64] ^= 1
        with self.assertRaises(ValueError): self.parse(raw)
        for span in self.parse()['spans']:
            for word in range(3):
                raw = bytearray(self.raw); raw[span['file_offset']-12+4*word] ^= 1
                with self.subTest(span=span['base'], word=word), self.assertRaises(ValueError): self.parse(raw)

    def test_truncation_trailer_and_extra_data(self):
        for stop in (0, 63, 64, 64+len(self.inventory)-1, len(self.raw)-33, len(self.raw)-1):
            with self.subTest(stop=stop), self.assertRaises(ValueError): self.parse(self.raw[:stop])
        with self.assertRaises(ValueError): self.parse(self.raw+b'x')
        for word in (0, 1, 2, 3, 5, 6, 7):
            raw = bytearray(self.raw); raw[-32+4*word] ^= 1
            with self.subTest(word=word), self.assertRaises(ValueError): self.parse(raw)
        raw = bytearray(self.raw); struct.pack_into('<I', raw, len(raw)-16, 5000)
        with self.assertRaises(ValueError): self.parse(raw)

    def test_all_anchors_checked_in_body(self):
        r = self.parse()
        for address in (self.context['unit']+343, self.context['origin']+191, 0xcab3ac, 0xcae5fc, 0xc06188):
            span = next(s for s in r['spans'] if int(s['base'],16) <= address < int(s['base'],16)+s['bytes'])
            raw = bytearray(self.raw); raw[span['file_offset']+address-int(span['base'],16)] ^= 1
            with self.subTest(address=address), self.assertRaisesRegex(ValueError, 'anchor'): self.parse(raw)

    def test_cap_and_exclusions(self):
        inv = decode_inventory(self.inventory, self.prefix)
        spans, total, _ = plan(inv, self.context)
        self.assertEqual([b for _, b, _ in spans], [0x10000, 0xc00000, 0x2000000])
        self.assertEqual(plan(inv, self.context, total)[1], total)
        with self.assertRaisesRegex(ValueError, 'cap'): plan(inv, self.context, total-1)
        rows = list(inv['rows']); row = list(rows[-1]); row[1:]=[row[0],4,CAP,0x1000,4,0x20000]; rows[-1] = tuple(row)
        with self.assertRaisesRegex(ValueError, 'cap'): plan({**inv,'rows':rows,'end':2**32-1}, self.context)

    def test_completion_receipt_order_and_failure(self):
        r = self.parse(); begin = (5,165,0,0,0,0,0,230); end = (7,0,0,0,0,0,0,230)
        success = (5,167,0,r['unit'],r['payload_bytes'],r['range_count'],0,r['frame'])
        self.assertEqual(check_receipts([begin,success,end],r),0)
        for rows in ([begin,end], [success,begin,end], [begin,end,success], [begin,success],
                     [begin,success,success,end], [begin,success,end,(5,168,10,0,0,0,0,230)]):
            with self.assertRaises(ValueError): check_receipts(rows,r)
        for word in (2,3,4,5,7):
            bad = list(success);bad[word] ^= 1
            with self.assertRaises(ValueError): check_receipts([begin,tuple(bad),end],r)
        bad = list(success);bad[6] = 5000
        with self.assertRaises(ValueError): check_receipts([begin,tuple(bad),end],r)


if __name__ == '__main__': unittest.main()
