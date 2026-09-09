import json
from pathlib import Path
import struct
import tempfile
import unittest
from compare_unattended import compare


class CompareTest(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.left,self.right=(Path(self.tmp.name)/n for n in ('left','right'))
        def row(*words): return tuple(words)+(0,)*(8-len(words))
        self.rows=[row(0x544E4F52,2,0x400000),row(5,175,1),row(5,170,1,1),
                   row(5,171,4096),row(5,172,4096,8192),row(5,173,4096)]
        self.rows += [row(2,n,n*7) for n in range(37)] + [row(5,170,21,2)]
        self.log=' MAP_STYLE 14\n (int)seed 12345\n'
        self.log+=''.join(f'BEGIN FRAME {n}\n value {n}\n' for n in range(18,36))
        self.log+='BEGIN FRAME 37\n GameInfo closing\n value 99\n'
        for path in (self.left,self.right):
            path.mkdir()
            (path/'receipt.json').write_text(json.dumps(dict(success=True,settings_restored=True,
                map_style=14,end_frame=36,seed_requested=12345,exit_code=0)))
            (path/'gamelog.txt').write_text(self.log)
            self.trace(path,self.rows)

    def trace(self,path,rows):
        (path/'rontrace.log').write_bytes(b''.join(struct.pack('<8I',*r) for r in rows))

    def test_includes_closing_record(self):
        result=compare(self.left,self.right)
        self.assertEqual(result['logged_bodies_equal'],19)
        self.assertEqual(result['frame_seed_pairs_equal'],37)
        self.assertFalse(result['complete_state_parity'])
        (self.right/'gamelog.txt').write_text(self.log.replace('value 99','value 100'))
        with self.assertRaisesRegex(ValueError,'logged body: frame 37'): compare(self.left,self.right)

    def test_any_field_and_missing_frame_refuse(self):
        (self.right/'gamelog.txt').write_text(self.log.replace('value 23','value 24'))
        with self.assertRaisesRegex(ValueError,'frame 23'): compare(self.left,self.right)
        (self.right/'gamelog.txt').write_text(self.log.replace('BEGIN FRAME 18\n value 18\n',''))
        with self.assertRaisesRegex(ValueError,'coverage'): compare(self.left,self.right)

    def test_seed_divergence_and_false_success_refuse(self):
        rows=self.rows[:];rows[6+20]=(2,20,999,0,0,0,0,0);self.trace(self.right,rows)
        with self.assertRaisesRegex(ValueError,'frame seed: frame 20'): compare(self.left,self.right)
        (self.right/'receipt.json').write_text('{"success":false}')
        with self.assertRaisesRegex(ValueError,'did not succeed'):compare(self.left,self.right)


if __name__=='__main__':unittest.main()
