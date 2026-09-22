"""Production C stream and fail-closed Python reader, using authored memory."""
import io, json, struct, subprocess, tempfile, unittest
from pathlib import Path
from frame_snapshot import decode,check_receipts
ROOT=Path(__file__).resolve().parent
PLAN=dict(logger=0x400200,game_slot=0x400000,frame_offset=0,roots=[dict(address=0x401000,size=16)])

class SnapshotTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with tempfile.TemporaryDirectory() as folder:
            p=Path(folder)
            subprocess.run(['cc','-O2','-Wall','-Wextra','-Werror','-Wno-int-to-pointer-cast','-Wno-pointer-to-int-cast',str(ROOT/'test_live_frame_snapshot.c'),'-o',str(p/'test')],check=True)
            subprocess.run([str(p/'test'),str(p/'snapshot')],check=True)
            cls.raw=(p/'snapshot').read_bytes()
    def parse(self,raw=None):return decode(io.BytesIO(self.raw if raw is None else raw),PLAN)
    def test_actual_c_stream(self):
        r=self.parse();self.assertEqual((r['frame'],r['trace_frame'],r['range_count'],r['payload_bytes']),(24,23,1,0x3000))
        self.assertFalse(r['atomic_snapshot']);self.assertFalse(r['logger_compared'])
    def test_missing_footer_and_trailing_data(self):
        for n in (0,127,128,295,len(self.raw)-29,len(self.raw)-1):
            with self.subTest(n=n),self.assertRaises(ValueError):self.parse(self.raw[:n])
        with self.assertRaises(ValueError):self.parse(self.raw+b'x')
    def test_identity_bounds_and_limits(self):
        for word,value in ((0,0),(1,2),(2,0),(3,25),(4,21),(5,0),(7,0),(8,0),(9,0),(10,0),(11,0),(12,0),(13,0),(14,3),(15,0),(16,0),(17,0),(18,0),(19,0),(20,2000),(21,1),(22,0),(23,0),(24,0),(25,0),(26,0),(27,0),(28,4),(29,25),(30,24),(31,1)):
            raw=bytearray(self.raw);struct.pack_into('<I',raw,word*4,value)
            with self.subTest(word=word),self.assertRaises(ValueError):self.parse(raw)
    def test_root_bytes_checked_in_body(self):
        span=self.parse()['spans'][0]
        for address in (0x400000,0x400100,0x40100f):
            raw=bytearray(self.raw);raw[span['file_offset']+address-0x400000]^=1
            with self.subTest(address=address),self.assertRaisesRegex(ValueError,'anchor'):self.parse(raw)
    def test_range_and_anchor_metadata(self):
        for offset in (128,128+28,128+6*28,128+6*28+12,self.parse()['spans'][0]['file_offset']-12):
            raw=bytearray(self.raw);raw[offset]^=1
            with self.subTest(offset=offset),self.assertRaises(ValueError):self.parse(raw)
    def test_footer(self):
        for word in range(7):
            raw=bytearray(self.raw);struct.pack_into('<I',raw,len(raw)-28+4*word,5000 if word==5 else 0)
            with self.subTest(word=word),self.assertRaises(ValueError):self.parse(raw)

    def test_receipts_require_hook_and_continuation(self):
        r=self.parse()
        rows=[(5,3,0x5329d0,0,0,0,0,0),(5,3,0x192586,0,0,0,0,0),
              (2,23,0,0,0,0,0,23),(5,180,24,1,0x3000,0,3,23),
              (5,182,24,3,0,0,0,23),(2,24,0,0,0,0,0,24)]
        self.assertEqual(check_receipts(rows,r),0)
        self.assertTrue(r['logger_return_roots_unchanged'])
        drift=rows[:4]+[(5,184,24,2,0x401000,16,1,23),(5,182,24,3,1,1,0,23)]+rows[5:]
        check_receipts(drift,r);self.assertFalse(r['logger_return_roots_unchanged'])
        for bad in (rows[1:],rows[:-1],rows+rows[3:4],rows[:3]+rows[4:],rows+[(5,181,1,0,0,0,0,0)]):
            with self.assertRaises(ValueError):check_receipts(bad,r)

if __name__=='__main__':unittest.main()
